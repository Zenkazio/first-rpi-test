{
  description = "first-rpi-test – native + cross builds";

  inputs = {
    system-flake.url = "git+ssh://git@github.com/Zenkazio/.store.git";
    nixpkgs.follows = "system-flake/nixpkgs";
  };

  outputs =
    {
      self,
      nixpkgs,
      ...
    }:
    {

      packages.x86_64-linux.native =
        let
          pkgs = import nixpkgs { system = "x86_64-linux"; };
        in
        pkgs.rustPlatform.buildRustPackage rec {
          pname = "first-rpi-test";
          version = "1.0.0";

          src = pkgs.lib.cleanSource ./.;
          cargoLock.lockFile = "${src}/Cargo.lock";
        };

      packages.x86_64-linux.default = self.packages.x86_64-linux.native;

      packages.x86_64-linux.aarch64 =
        let
          pkgs = import nixpkgs {
            system = "x86_64-linux";
            crossSystem = {
              config = "aarch64-unknown-linux-musl";
            };
          };
        in
        pkgs.rustPlatform.buildRustPackage rec {
          pname = "first-rpi-test";
          version = "1.0.0";

          src = pkgs.lib.cleanSource ./.;
          cargoLock.lockFile = "${src}/Cargo.lock";

          RUSTFLAGS = [
            "-C"
            "target-feature=+crt-static"
          ];
        };

      packages.x86_64-linux.all =
        let
          pkgs = import nixpkgs { system = "x86_64-linux"; };
        in
        pkgs.linkFarm "all-builds" [
          {
            name = "native";
            path = self.packages.x86_64-linux.native;
          }
          {
            name = "aarch64";
            path = self.packages.x86_64-linux.aarch64;
          }
        ];

      devShells.x86_64-linux.default =
        let
          pkgs = import nixpkgs { system = "x86_64-linux"; };
        in
        pkgs.mkShell {
          buildInputs = with pkgs; [
            rustc
            cargo
            rustfmt
            clippy
            rust-analyzer
          ];
          env.RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
        };

      apps.x86_64-linux.deploy =
        let
          pkgs = import nixpkgs { system = "x86_64-linux"; };
        in
        {
          type = "app";
          program = builtins.toString (
            pkgs.writeShellScript "deploy" ''
              set -euo pipefail

              echo "===> Building aarch64 binary"
              nix build .#all

              TARGET_HOST="zenkazio@192.168.178.36"
              TARGET_PATH="/home/zenkazio/first-rpi-test"  # beliebiger Ort, wir nutzen /home
              REMOTE_BIN="/usr/local/bin/first-rpi-test"     # finaler Ort mit root-Rechten

              echo "===> Sync to target via rsync"
              rsync -avc --delete result/aarch64/bin/first-rpi-test $TARGET_HOST:$TARGET_PATH

              echo "===> Restarting program on target (sudo required)"
              ssh "$TARGET_HOST" "sudo systemctl restart start-rpi-program.service
              "

              echo "===> Deployment complete."
            ''
          );
        };
      apps.x86_64-linux.stop =
        let
          pkgs = import nixpkgs { system = "x86_64-linux"; };
        in
        {
          type = "app";
          program = builtins.toString (
            pkgs.writeShellScript "stop-remote" ''
              set -euo pipefail
              TARGET_HOST="zenkazio@192.168.178.36"

              echo "===> Stopping remote program"
              ssh "$TARGET_HOST" "sudo systemctl stop start-rpi-program.service"

              echo "===> Program stopped"
            ''
          );
        };
      apps.x86_64-linux.default = self.apps.x86_64-linux.deploy;

    };
}
