{
  description = "first-rpi-test in Rust";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
    }:
    flake-utils.simpleFlake {
      inherit self nixpkgs;

      # --- Build-Package für Host-Architektur ---
      packages = {
        default =
          pkgs:
          pkgs.rustPlatform.buildRustPackage rec {
            pname = "first-rpi-test";
            version = "1.0.0";

            src = pkgs.lib.cleanSource ./.;
            cargoLock.lockFile = "${src}/Cargo.lock";
          };
      };

      # --- DevShell (optional aber praktisch) ---
      devShell =
        pkgs:
        pkgs.mkShell {
          buildInputs = [
            pkgs.rustc
            pkgs.cargo
          ];
        };
    };
}
