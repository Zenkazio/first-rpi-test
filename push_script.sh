#!/usr/bin/env bash

# distrobox create -n dev_arch --image archlinux:latest
# distrobox enter dev_arch -- sudo pacman -Syu aarch64-linux-gnu-gcc gcc rustup
# distrobox enter dev_arch -- rustup default stable
# distrobox enter dev_arch -- rustup target add aarch64-unknown-linux-musl
set -e

CONFIG_FILE="$HOME/.cargo/config.toml"

if ! grep -q "\[target.aarch64-unknown-linux-musl\]" "$CONFIG_FILE" 2>/dev/null; then
    cat <<EOF >> "$CONFIG_FILE"
[target.aarch64-unknown-linux-musl]
linker = "aarch64-linux-gnu-gcc"
rustflags = ["-C", "target-feature=+crt-static"]
EOF
fi

# echo "Compling for tests"

# cargo test --release --target=x86_64-unknown-linux-gnu

# echo "Tests passed"
echo "Compiling for target"

cargo build --release --target=aarch64-unknown-linux-musl

echo "Target compilation done"
echo "transfer and start"

TARGET_HOST="zenkazio@192.168.178.36"
TARGET_PATH="/home/zenkazio/first-rpi-test"
rsync -avzP target/aarch64-unknown-linux-musl/release/first-rpi-test $TARGET_HOST:$TARGET_PATH
ssh "$TARGET_HOST" "sudo systemctl restart rpi-program.service && journalctl -u rpi-program.service -f"
