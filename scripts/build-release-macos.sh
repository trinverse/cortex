#!/bin/bash

# Build release binaries for macOS (for Homebrew)
set -e

VERSION="${1:-0.1.0}"
ARCH="${2:-universal}"  # x86_64, aarch64, or universal

echo "Building Cortex v${VERSION} for macOS (${ARCH})"

# Clean previous builds
rm -rf target/release
mkdir -p dist

if [ "$ARCH" = "universal" ]; then
    echo "Building universal binary (Intel + Apple Silicon)..."
    
    # Build for x86_64
    cargo build --release --target x86_64-apple-darwin
    
    # Build for aarch64
    cargo build --release --target aarch64-apple-darwin
    
    # Create universal binary
    lipo -create \
        target/x86_64-apple-darwin/release/cortex \
        target/aarch64-apple-darwin/release/cortex \
        -output dist/cortex
else
    echo "Building for ${ARCH}..."
    cargo build --release --target "${ARCH}-apple-darwin"
    cp "target/${ARCH}-apple-darwin/release/cortex" dist/cortex
fi

# Create tarball for Homebrew
cd dist
tar czf "cortex-${VERSION}-macos-${ARCH}.tar.gz" cortex
sha256sum "cortex-${VERSION}-macos-${ARCH}.tar.gz" > "cortex-${VERSION}-macos-${ARCH}.tar.gz.sha256"

echo "Build complete!"
echo "Archive: dist/cortex-${VERSION}-macos-${ARCH}.tar.gz"
echo "SHA256: $(cat cortex-${VERSION}-macos-${ARCH}.tar.gz.sha256)"