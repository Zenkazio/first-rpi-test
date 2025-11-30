{
  pkgs ? import <nixpkgs> { },
}:

pkgs.pkgsCross.aarch64-multiplatform-musl.rustPlatform.buildRustPackage rec {
  pname = "first-rpi-test";
  version = "1.0.0";

  src = pkgs.lib.cleanSource ./.;
  cargoLock.lockFile = "${src}/Cargo.lock";

  # Statischer Build (meist eh Standard bei musl)
  RUSTFLAGS = [
    "-C"
    "target-feature=+crt-static"
  ];
}
