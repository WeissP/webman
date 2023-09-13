{
  inputs = {
    cargo2nix.url = "github:cargo2nix/cargo2nix/unstable";
    flake-utils.follows = "cargo2nix/flake-utils";
    nixpkgs.follows = "cargo2nix/nixpkgs";
  };

  outputs = inputs:
    with inputs;
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ cargo2nix.overlays.default ];
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
        devShells.default = mkShell {
          buildInputs = [
            pkg-config
            rust-analyzer
            rust-bin.nightly."2023-01-01".default
            cargo-limit
          ] ++ (if stdenv.isDarwin then
            with darwin.apple_sdk.frameworks; [
              IOKit
              Security
              CoreServices
              SystemConfiguration
            ]
          else
            [ ]);

          # RUST_BACKTRACE = 1;
          # CC_LOG_LEVEL = "trace";
          CC_LOG_LEVEL = "debug";
          CC_INSTANCES_LOCATION = if stdenv.isDarwin then
            "/Users/bozhoubai/projects/coin-change/instances"
          else
            "/home/weiss/projects/coin-change/instances";
        };
        packages = {
          webman-cli = (rustPkgs.workspace.webman-cli { }).bin;
          webman-server = (rustPkgs.workspace.webman-server { }).bin;
          default = packages.webman-cli;
        };
      });
}
