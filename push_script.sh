#!/usr/bin/bash


cargo test --release --target=x86_64-unknown-linux-gnu
cargo build --release --target=aarch64-unknown-linux-musl

TARGET_HOST="zenkazio@192.168.178.36"
TARGET_PATH="/home/zenkazio/first-rpi-test"
rsync -avc --delete target/aarch64-unknown-linux-musl/release/first-rpi-test $TARGET_HOST:$TARGET_PATH
ssh "$TARGET_HOST" "sudo systemctl restart rpi-program.service && journalctl -u rpi-program.service -f"
