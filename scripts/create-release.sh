#!/bin/bash

# Create release artifacts for Homebrew from Linux
set -e

VERSION="${1:-0.1.0}"

echo "Creating Cortex v${VERSION} release artifacts"

# Build for Linux x86_64
echo "Building for Linux x86_64..."
cargo build --release
mkdir -p dist

# Create Linux x86_64 tarball
cp target/release/cortex dist/
cd dist
tar czf "cortex-${VERSION}-linux-x86_64.tar.gz" cortex
sha256sum "cortex-${VERSION}-linux-x86_64.tar.gz" > "cortex-${VERSION}-linux-x86_64.tar.gz.sha256"
rm cortex
cd ..

echo ""
echo "✅ Build complete!"
echo ""
echo "Artifacts created:"
ls -la dist/
echo ""
echo "SHA256 checksums:"
cat dist/*.sha256
echo ""
echo "Next steps:"
echo "1. Create a GitHub release at: https://github.com/trinverse/cortex/releases/new"
echo "2. Tag: v${VERSION}"
echo "3. Upload the .tar.gz files from dist/"
echo "4. The GitHub Action will handle macOS builds automatically"