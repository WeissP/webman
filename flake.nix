{
  inputs = {
    # cargo2nix.url = "github:cargo2nix/cargo2nix/unstable";
    # nixpkgs.follows = "cargo2nix/nixpkgs";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    crane.inputs.nixpkgs.follows = "nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
    devshell.url = "github:numtide/devshell";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, crane, flake-utils, devshell, nixpkgs, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [
            # cargo2nix.overlays.default
            devshell.overlays.default
            (import rust-overlay)
          ];
        };
        extraBuildInputs = with pkgs;
          [ libiconv pkg-config ] ++ (if stdenv.isDarwin then
            with darwin.apple_sdk.frameworks; [
              IOKit
              Security
              CoreServices
              SystemConfiguration
            ]
          else [
            gcc
            openssl
          ]);
      in {
        devShells.default = pkgs.devshell.mkShell {
          imports = map pkgs.devshell.importTOML [ ./devshell.toml ];
          packages = with pkgs;
            [ (rust-bin.stable.latest.complete) ] ++ extraBuildInputs;
          env = [{
            name = "PKG_CONFIG_PATH";
            value = "${pkgs.openssl.dev}/lib/pkgconfig";
          }];
        };
        packages = let
          craneLib = crane.lib.${system};
          sqlxFilter = path: _type:
            null != builtins.match ".*sql$" path || null
            != builtins.match ".*json$" path || null
            != builtins.match ".*env$" path;

          common = {
            version = "0.1";
            # src = craneLib.path ./.;
            src = pkgs.lib.cleanSourceWith {
              src = craneLib.path ./.;
              filter = path: type:
                (sqlxFilter path type)
                || (craneLib.filterCargoSources path type);
            };
            strictDeps = true;
            buildInputs = extraBuildInputs;
            nativeBuildInputs = with pkgs; [ pkg-config ];
            PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
            doCheck = false;
          };
          cargoArtifacts = craneLib.buildDepsOnly common;
        in rec {
          webman-cli = craneLib.buildPackage (common // {
            pname = "webman-cli";
            cargoExtraArgs = "-p webman-cli";
          });

          webman-server = craneLib.buildPackage (common // {
            pname = "webman-server";
            cargoExtraArgs = "-p webman-server";
          });

          default = webman-cli;
        };
      });
}
