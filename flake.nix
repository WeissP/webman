{
  inputs = {
    cargo2nix.url = "github:cargo2nix/cargo2nix/unstable";
    flake-utils.follows = "cargo2nix/flake-utils";
    nixpkgs.follows = "cargo2nix/nixpkgs";
    devshell.url = "github:numtide/devshell";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = inputs:
    with inputs;
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [
            cargo2nix.overlays.default
            devshell.overlays.default
            (import rust-overlay)
          ];
        };

        rustPkgs = pkgs.rustBuilder.makePackageSet {
          rustVersion = "1.66.1";
          packageFun = import ./Cargo.nix;
          # Use the existing all list of overrides and append your override
          packageOverrides = pkgs:
            pkgs.rustBuilder.overrides.all ++ [
              (pkgs.rustBuilder.rustLib.makeOverride {
                name = "sqlx-macros";
                overrideAttrs = drv: {
                  propagatedBuildInputs = with pkgs;
                    drv.propagatedBuildInputs or [ ]
                    ++ (lib.optional stdenv.isDarwin
                      (with darwin.apple_sdk.frameworks;
                        [ SystemConfiguration ]));
                };
              })
            ];
        };

      in with pkgs; rec {
        devShells.default = pkgs.devshell.mkShell {
          imports = map pkgs.devshell.importTOML [ ./devshell.toml ];
          packages = with pkgs;
            [ rust-bin.stable."1.66.1".default ] ++ (if stdenv.isDarwin then
              with darwin.apple_sdk.frameworks; [
                IOKit
                Security
                CoreServices
                SystemConfiguration
              ]
            else [
              gcc
              openssl
              libiconv
              pkg-config
            ]);
          env = [{
            name = "PKG_CONFIG_PATH";
            value = "${pkgs.openssl.dev}/lib/pkgconfig";
          }];
        };
        packages = {
          webman-cli = (rustPkgs.workspace.webman-cli { }).bin;
          webman-server = (rustPkgs.workspace.webman-server { }).bin;
          default = packages.webman-cli;
        };
      });
}
