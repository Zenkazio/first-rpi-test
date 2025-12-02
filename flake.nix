{
  description = "first-rpi-test – native + cross builds";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs =
    { self, nixpkgs }:
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
              nix build .#aarch64

              TARGET_HOST="zenkazio@192.168.178.36"
              TARGET_PATH="/home/zenkazio/first-rpi-test"  # beliebiger Ort, wir nutzen /home
              REMOTE_BIN="/usr/local/bin/first-rpi-test"     # finaler Ort mit root-Rechten

              echo "===> Sync to target via rsync"
              rsync -av --delete result/bin/first-rpi-test $TARGET_HOST:$TARGET_PATH

              echo "===> Restarting program on target (sudo required)"
              ssh "$TARGET_HOST" "
                set -e
                echo 'Stopping old program (ignore errors if not running)'
                sudo killall first-rpi-test 2>/dev/null || true

                echo 'Installing new binary'
                sudo mv $TARGET_PATH $REMOTE_BIN
                sudo chmod +x $REMOTE_BIN

                echo 'Starting new program'
                sudo $REMOTE_BIN &
              "

              echo "===> Deployment complete."
            ''
          );
        };
      apps.x86_64-linux.default = self.apps.x86_64-linux.deploy;

    };
}
