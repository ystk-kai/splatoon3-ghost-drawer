#!/bin/bash

# Usage: ./scripts/deploy.sh <user>@<host>

if [ -z "$1" ]; then
    echo "Usage: $0 <user>@<host>"
    exit 1
fi

TARGET=$1
BINARY_PATH="target/aarch64-unknown-linux-gnu/release/splatoon3-ghost-drawer"

echo "Building for aarch64..."
cargo build --release --target aarch64-unknown-linux-gnu

if [ ! -f "$BINARY_PATH" ]; then
    echo "Error: Binary not found at $BINARY_PATH"
    exit 1
fi

echo "Deploying to $TARGET..."

# Copy binary to temp location
echo "Copying binary..."
scp "$BINARY_PATH" "$TARGET:/tmp/splatoon3-ghost-drawer"

# Copy install script
echo "Copying install script..."
scp scripts/install_remote.sh "$TARGET:/tmp/install_remote.sh"

# Run install script
echo "Running install script..."
ssh -t "$TARGET" "chmod +x /tmp/install_remote.sh && sudo /tmp/install_remote.sh"

echo "Deployment complete!"
