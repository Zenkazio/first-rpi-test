{
  description = "first-rpi-test – native + cross builds mit caching";

  inputs = {
    system-flake.url = "git+ssh://git@github.com/Zenkazio/.store.git";
    nixpkgs.follows = "system-flake/nixpkgs";
    crane.url = "github:ipetkov/crane";
    # crane.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs =
    {
      self,
      nixpkgs,
      crane,
      ...
    }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs { inherit system; };

      # Crane Library initialisieren
      craneLib = crane.mkLib pkgs;

      htmlFilter =
        path: type:
        (builtins.match ".*\\.html$" path != null)
        || (builtins.match ".*\\.css$" path != null)
        || (craneLib.filterCargoSources path type);

      # Den Filter auf den Source anwenden
      src = pkgs.lib.cleanSourceWith {
        src = ./.; # Oder craneLib.path ./.
        filter = htmlFilter;
      };
      # Gemeinsame Argumente (Abhängigkeiten & Version)
      commonArgs = {
        inherit src;
        strictDeps = true;
        pname = "first-rpi-test";
        version = "1.0.0";

        nativeBuildInputs = with pkgs; [
          llvmPackages.llvm
          llvmPackages.libclang
          pkg-config
          git
        ];

        LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
        LLVM_CONFIG_PATH = "${pkgs.llvmPackages.llvm.dev}/bin/llvm-config";
      };

      # 1. Nur Abhängigkeiten bauen (Caching-Layer)
      cargoArtifacts = craneLib.buildDepsOnly commonArgs;

      # 2. Das eigentliche Paket (Native)
      nativePkg = craneLib.buildPackage (
        commonArgs
        // {
          inherit cargoArtifacts;
        }
      );

      # 3. Cross-Build Setup (aarch64)
      pkgsAarch64 = import nixpkgs {
        inherit system;
        crossSystem = {
          config = "aarch64-unknown-linux-musl";
        };
      };
      craneLibAarch64 = crane.mkLib pkgsAarch64;

      # Cross-Abhängigkeiten bauen
      cargoArtifactsAarch64 = craneLibAarch64.buildDepsOnly (
        commonArgs
        // {
          # Wichtig für musl-static
          CARGO_BUILD_RUSTFLAGS = "-C target-feature=+crt-static";
        }
      );

      aarch64Pkg = craneLibAarch64.buildPackage (
        commonArgs
        // {
          inherit cargoArtifactsAarch64;
          CARGO_BUILD_RUSTFLAGS = "-C target-feature=+crt-static";
        }
      );

    in
    {
      packages.${system} = {
        native = nativePkg;
        aarch64 = aarch64Pkg;
        default = nativePkg;
        all = pkgs.linkFarm "all-builds" [
          {
            name = "native";
            path = nativePkg;
          }
          {
            name = "aarch64";
            path = aarch64Pkg;
          }
        ];
      };

      devShells.${system}.default = pkgs.mkShell {
        buildInputs = with pkgs; [
          rustc
          cargo
          rustfmt
          clippy
          rust-analyzer

          llvmPackages.llvm
          llvmPackages.libclang
          pkg-config
        ];

        RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";

        LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
        LLVM_CONFIG_PATH = "${pkgs.llvmPackages.llvm.dev}/bin/llvm-config";
      };

      apps.${system} = {
        deploy = {
          type = "app";
          program = builtins.toString (
            pkgs.writeShellScript "deploy" ''
              set -euo pipefail
              nix build .#all
              TARGET_HOST="zenkazio@192.168.178.36"
              TARGET_PATH="/home/zenkazio/first-rpi-test"
              rsync -avc --delete result/aarch64/bin/first-rpi-test $TARGET_HOST:$TARGET_PATH
              ssh "$TARGET_HOST" "sudo systemctl restart rpi-program.service"
            ''
          );
        };
        stop = {
          type = "app";
          program = builtins.toString (
            pkgs.writeShellScript "stop-remote" ''
              ssh "zenkazio@192.168.178.36" "sudo systemctl stop rpi-program.service"
            ''
          );
        };
        default = self.apps.${system}.deploy;
      };
    };
}
