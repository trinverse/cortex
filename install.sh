#!/bin/bash

set -e

REPO="cortex-fm/cortex"
INSTALL_DIR="/usr/local/bin"
BINARY_NAME="cortex"

# Detect OS and architecture
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "$ARCH" in
    x86_64)
        ARCH="x86_64"
        ;;
    aarch64|arm64)
        ARCH="aarch64"
        ;;
    *)
        echo "Unsupported architecture: $ARCH"
        exit 1
        ;;
esac

case "$OS" in
    linux)
        TARGET="${ARCH}-unknown-linux-gnu"
        ;;
    darwin)
        TARGET="${ARCH}-apple-darwin"
        ;;
    *)
        echo "Unsupported OS: $OS"
        exit 1
        ;;
esac

echo "Installing Cortex for $TARGET..."

# Check if cargo is available
if command -v cargo &> /dev/null; then
    echo "Building from source with cargo..."
    cargo install --path cortex-cli --root ~/.cargo
    echo "Cortex installed successfully!"
    echo "Make sure ~/.cargo/bin is in your PATH"
else
    echo "Cargo not found. Please install Rust first:"
    echo "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi