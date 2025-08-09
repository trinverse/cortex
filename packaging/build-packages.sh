#!/bin/bash

# Build script for creating distribution packages for Cortex

set -e

VERSION="0.1.0"
PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BUILD_DIR="$PROJECT_ROOT/target/packages"

echo "Building Cortex v$VERSION packages..."

# Create build directory
mkdir -p "$BUILD_DIR"

# Build release binary
echo "Building release binary..."
cd "$PROJECT_ROOT"
cargo build --release

# Linux AppImage
build_appimage() {
    echo "Building AppImage..."
    mkdir -p "$BUILD_DIR/appimage"
    # AppImage build commands would go here
    echo "AppImage build not yet implemented"
}

# Debian package
build_deb() {
    echo "Building Debian package..."
    if command -v dpkg-deb &> /dev/null; then
        mkdir -p "$BUILD_DIR/deb/cortex_${VERSION}_amd64"
        cp -r "$PROJECT_ROOT/packaging/debian" "$BUILD_DIR/deb/cortex_${VERSION}_amd64/DEBIAN"
        mkdir -p "$BUILD_DIR/deb/cortex_${VERSION}_amd64/usr/bin"
        cp "$PROJECT_ROOT/target/release/cortex" "$BUILD_DIR/deb/cortex_${VERSION}_amd64/usr/bin/"
        dpkg-deb --build "$BUILD_DIR/deb/cortex_${VERSION}_amd64"
        echo "Debian package created: $BUILD_DIR/deb/cortex_${VERSION}_amd64.deb"
    else
        echo "dpkg-deb not found, skipping Debian package"
    fi
}

# RPM package
build_rpm() {
    echo "Building RPM package..."
    if command -v rpmbuild &> /dev/null; then
        # RPM build commands would go here
        echo "RPM build not yet implemented"
    else
        echo "rpmbuild not found, skipping RPM package"
    fi
}

# macOS DMG
build_dmg() {
    echo "Building macOS DMG..."
    if [[ "$OSTYPE" == "darwin"* ]]; then
        mkdir -p "$BUILD_DIR/dmg"
        cp "$PROJECT_ROOT/target/release/cortex" "$BUILD_DIR/dmg/"
        # DMG creation would go here
        echo "DMG build not yet implemented"
    else
        echo "Not on macOS, skipping DMG build"
    fi
}

# Windows MSI
build_msi() {
    echo "Building Windows MSI..."
    if command -v candle &> /dev/null && command -v light &> /dev/null; then
        # WiX build commands would go here
        echo "MSI build not yet implemented"
    else
        echo "WiX toolset not found, skipping MSI build"
    fi
}

# Tarball
build_tarball() {
    echo "Building tarball..."
    mkdir -p "$BUILD_DIR/tarball/cortex-$VERSION"
    cp "$PROJECT_ROOT/target/release/cortex" "$BUILD_DIR/tarball/cortex-$VERSION/"
    cp "$PROJECT_ROOT/README.md" "$BUILD_DIR/tarball/cortex-$VERSION/"
    cp "$PROJECT_ROOT/LICENSE" "$BUILD_DIR/tarball/cortex-$VERSION/" 2>/dev/null || true
    cd "$BUILD_DIR/tarball"
    tar czf "cortex-$VERSION-$(uname -s)-$(uname -m).tar.gz" "cortex-$VERSION"
    echo "Tarball created: $BUILD_DIR/tarball/cortex-$VERSION-$(uname -s)-$(uname -m).tar.gz"
}

# Parse arguments
if [ $# -eq 0 ]; then
    # Build all packages
    build_tarball
    build_deb
    build_rpm
    build_appimage
    build_dmg
    build_msi
else
    for arg in "$@"; do
        case $arg in
            tarball) build_tarball ;;
            deb) build_deb ;;
            rpm) build_rpm ;;
            appimage) build_appimage ;;
            dmg) build_dmg ;;
            msi) build_msi ;;
            *) echo "Unknown package type: $arg" ;;
        esac
    done
fi

echo "Package build complete!"
ls -la "$BUILD_DIR/"*/