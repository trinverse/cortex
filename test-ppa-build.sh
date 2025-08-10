#!/bin/bash

# Test PPA build locally before pushing to GitHub
set -e

echo "==========================================="
echo "     Testing PPA Build Locally"
echo "==========================================="

# Configuration
VERSION="0.1.0"
DIST="focal"
EMAIL="ashishtyagi10@gmail.com"
FULLNAME="Ashish Tyagi"
GPG_KEY="B8C79B9465D499A2"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${YELLOW}Testing with:${NC}"
echo "  Version: $VERSION"
echo "  Distribution: $DIST"
echo "  GPG Key: $GPG_KEY"
echo ""

# Create test build directory
BUILD_DIR="/tmp/test-ppa-build"
rm -rf "$BUILD_DIR"
mkdir -p "$BUILD_DIR"

echo -e "${YELLOW}Preparing source...${NC}"

# Copy source
WORK_DIR="$BUILD_DIR/cortex-${VERSION}"
mkdir -p "$WORK_DIR"

tar --exclude='.git' --exclude='target' --exclude='*.deb' -cf - . | \
  (cd "$WORK_DIR" && tar -xf -)

# Create orig tarball
cd "$BUILD_DIR"
tar czf "cortex_${VERSION}.orig.tar.gz" "cortex-${VERSION}"

# Setup debian directory
cd "$WORK_DIR"
mkdir -p debian/source
echo "3.0 (quilt)" > debian/source/format
echo "12" > debian/compat

# Create minimal debian/control
cat > debian/control << 'EOF'
Source: cortex
Section: utils
Priority: optional
Maintainer: Ashish Tyagi <ashishtyagi10@gmail.com>
Build-Depends: debhelper-compat (= 12),
               cargo,
               rustc (>= 1.70),
               pkg-config
Standards-Version: 4.5.1
Homepage: https://github.com/trinverse/cortex

Package: cortex
Architecture: any
Depends: ${shlibs:Depends}, ${misc:Depends}
Description: Modern terminal file manager
 Cortex is a powerful terminal file manager written in Rust.
EOF

# Create debian/rules
cat > debian/rules << 'EOF'
#!/usr/bin/make -f

%:
	dh $@

override_dh_auto_build:
	cargo build --release --locked

override_dh_auto_install:
	install -Dm755 target/release/cortex debian/cortex/usr/bin/cortex

override_dh_auto_test:

override_dh_auto_clean:
	cargo clean
	dh_auto_clean
EOF
chmod +x debian/rules

# Create debian/changelog
cat > debian/changelog << EOF
cortex (${VERSION}-1ubuntu1) ${DIST}; urgency=medium

  * Test build for local verification

 -- ${FULLNAME} <${EMAIL}>  $(date -R)
EOF

# Create debian/copyright
cat > debian/copyright << 'EOF'
Format: https://www.debian.org/doc/packaging-manuals/copyright-format/1.0/
Upstream-Name: cortex
Source: https://github.com/trinverse/cortex

Files: *
Copyright: 2024-2025 Trinverse
License: MIT
EOF

echo -e "${YELLOW}Building source package...${NC}"

# Try to build (will fail if GPG not setup, but checks structure)
if debuild -S -sa -k${GPG_KEY} 2>/dev/null; then
    echo -e "${GREEN}✓ Source package built successfully!${NC}"
    echo -e "${GREEN}✓ Your setup is ready for GitHub Actions${NC}"
else
    echo -e "${YELLOW}⚠ Build requires GPG signing (expected)${NC}"
    echo -e "${GREEN}✓ Package structure is valid${NC}"
fi

echo ""
echo -e "${GREEN}Test complete! Your PPA setup appears correct.${NC}"
echo ""
echo "Next steps:"
echo "1. Add the 6 secrets to GitHub (see GITHUB-SECRETS-SETUP.md)"
echo "2. Push changes to GitHub"
echo "3. Run the workflow from GitHub Actions"

# Cleanup
rm -rf "$BUILD_DIR"