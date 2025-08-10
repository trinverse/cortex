#!/bin/bash

# Build and upload Cortex package to Ubuntu PPA
# This script prepares proper Debian source packages for Launchpad

set -e

# Configuration
VERSION="0.1.0"
DEBIAN_VERSION="1"
PACKAGE_NAME="cortex"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}   Cortex PPA Package Builder${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# Check prerequisites
echo -e "${YELLOW}Checking prerequisites...${NC}"

# Check for required tools
MISSING_TOOLS=""
for tool in dpkg-buildpackage debuild dch dput gpg; do
    if ! command -v $tool &> /dev/null; then
        MISSING_TOOLS="$MISSING_TOOLS $tool"
    fi
done

if [ ! -z "$MISSING_TOOLS" ]; then
    echo -e "${RED}Missing required tools:${MISSING_TOOLS}${NC}"
    echo "Install with:"
    echo "  sudo apt-get install devscripts debhelper dput gnupg"
    exit 1
fi

echo -e "${GREEN}✓ All required tools found${NC}"
echo ""

# Get user information
echo -e "${YELLOW}Configuration:${NC}"
read -p "Your Launchpad username: " LAUNCHPAD_USER
read -p "Your email (same as GPG key): " EMAIL
read -p "Your GPG key ID: " GPG_KEY

# Select Ubuntu versions to build for
echo ""
echo -e "${YELLOW}Select Ubuntu versions to build for:${NC}"
echo "1) focal (20.04 LTS)"
echo "2) jammy (22.04 LTS)"
echo "3) noble (24.04 LTS)"
echo "4) oracular (24.10)"
echo "5) plucky (25.04 dev)"
echo "6) All stable releases (focal, jammy, noble, oracular)"
echo "7) All including development (focal, jammy, noble, oracular, plucky)"
read -p "Choice [6]: " DIST_CHOICE
DIST_CHOICE=${DIST_CHOICE:-6}

case $DIST_CHOICE in
    1) DISTRIBUTIONS="focal" ;;
    2) DISTRIBUTIONS="jammy" ;;
    3) DISTRIBUTIONS="noble" ;;
    4) DISTRIBUTIONS="oracular" ;;
    5) DISTRIBUTIONS="plucky" ;;
    6) DISTRIBUTIONS="focal jammy noble oracular" ;;
    7) DISTRIBUTIONS="focal jammy noble oracular plucky" ;;
    *) DISTRIBUTIONS="focal jammy noble oracular" ;;
esac

echo -e "${GREEN}Building for: $DISTRIBUTIONS${NC}"
echo ""

# Create build directory
BUILD_DIR="$HOME/ppa-build-cortex"
echo -e "${YELLOW}Creating build directory: $BUILD_DIR${NC}"
rm -rf "$BUILD_DIR"
mkdir -p "$BUILD_DIR"

# Get the source directory
SOURCE_DIR="$(cd "$(dirname "$0")/../.." && pwd)"
echo -e "${YELLOW}Source directory: $SOURCE_DIR${NC}"

# For each distribution
for DIST in $DISTRIBUTIONS; do
    echo ""
    echo -e "${BLUE}========================================${NC}"
    echo -e "${BLUE}Building for Ubuntu $DIST${NC}"
    echo -e "${BLUE}========================================${NC}"
    
    # Set version with distribution suffix
    if [ "$DIST" == "focal" ]; then
        FULL_VERSION="${VERSION}-${DEBIAN_VERSION}ubuntu1"
    else
        FULL_VERSION="${VERSION}-${DEBIAN_VERSION}ubuntu1~${DIST}1"
    fi
    
    WORK_DIR="$BUILD_DIR/${PACKAGE_NAME}-${VERSION}"
    
    # Create working directory
    echo -e "${YELLOW}Preparing source...${NC}"
    rm -rf "$WORK_DIR"
    mkdir -p "$WORK_DIR"
    
    # Copy source files (excluding .git and target)
    cd "$SOURCE_DIR"
    tar --exclude='.git' \
        --exclude='target' \
        --exclude='*.deb' \
        --exclude='*.tar.gz' \
        --exclude='ppa-build*' \
        -cf - . | (cd "$WORK_DIR" && tar -xf -)
    
    # Create orig tarball
    cd "$BUILD_DIR"
    tar czf "${PACKAGE_NAME}_${VERSION}.orig.tar.gz" "${PACKAGE_NAME}-${VERSION}"
    
    # Prepare debian directory
    cd "$WORK_DIR"
    mkdir -p debian
    
    # Create debian/control
    cat > debian/control << EOF
Source: cortex
Section: utils
Priority: optional
Maintainer: $EMAIL
Build-Depends: debhelper-compat (= 12),
               cargo,
               rustc (>= 1.70),
               libssl-dev,
               pkg-config
Standards-Version: 4.5.1
Homepage: https://github.com/trinverse/cortex
Vcs-Browser: https://github.com/trinverse/cortex
Vcs-Git: https://github.com/trinverse/cortex.git

Package: cortex
Architecture: any
Depends: \${shlibs:Depends}, \${misc:Depends}
Description: Modern terminal file manager
 Cortex is a powerful and fast terminal file manager written in Rust.
 It features a dual-pane interface, extensive keyboard shortcuts,
 plugin system with Lua support, and many advanced features for
 power users.
 .
 Features include:
  - Dual-pane orthodox file manager interface
  - Vim-style keyboard navigation
  - File preview and editing
  - Archive support (ZIP, TAR, 7Z)
  - Plugin system with Lua scripting
  - Bookmarks and quick navigation
  - Batch operations
  - Cross-platform support
EOF

    # Create debian/rules
    cat > debian/rules << 'EOF'
#!/usr/bin/make -f

export DH_VERBOSE = 1
export CARGO_HOME = $(CURDIR)/debian/cargo

%:
	dh $@

override_dh_auto_build:
	cargo build --release

override_dh_auto_install:
	install -Dm755 target/release/cortex debian/cortex/usr/bin/cortex
	# Install desktop file if exists
	if [ -f assets/cortex.desktop ]; then \
		install -Dm644 assets/cortex.desktop debian/cortex/usr/share/applications/cortex.desktop; \
	fi
	# Install icon if exists
	if [ -f assets/icons/svg/cortex-icon.svg ]; then \
		install -Dm644 assets/icons/svg/cortex-icon.svg \
			debian/cortex/usr/share/icons/hicolor/scalable/apps/cortex.svg; \
	fi

override_dh_auto_test:
	# Skip tests during build

override_dh_auto_clean:
	cargo clean
	dh_auto_clean
EOF
    chmod +x debian/rules

    # Create debian/compat
    echo "12" > debian/compat

    # Create debian/source/format
    mkdir -p debian/source
    echo "3.0 (quilt)" > debian/source/format

    # Create initial changelog
    cat > debian/changelog << EOF
cortex (${FULL_VERSION}) ${DIST}; urgency=medium

  * Initial release for Ubuntu ${DIST}
  * Full feature set including:
    - Dual-pane file manager interface
    - Plugin system with Lua support
    - File operations with progress tracking
    - Archive support
    - Keyboard shortcuts and vim bindings

 -- ${EMAIL}  $(date -R)
EOF

    # Create debian/copyright
    cat > debian/copyright << 'EOF'
Format: https://www.debian.org/doc/packaging-manuals/copyright-format/1.0/
Upstream-Name: cortex
Upstream-Contact: Trinverse <admin@trinverse.com>
Source: https://github.com/trinverse/cortex

Files: *
Copyright: 2024-2025 Trinverse
License: MIT

License: MIT
 Permission is hereby granted, free of charge, to any person obtaining a
 copy of this software and associated documentation files (the "Software"),
 to deal in the Software without restriction, including without limitation
 the rights to use, copy, modify, merge, publish, distribute, sublicense,
 and/or sell copies of the Software, and to permit persons to whom the
 Software is furnished to do so, subject to the following conditions:
 .
 The above copyright notice and this permission notice shall be included
 in all copies or substantial portions of the Software.
 .
 THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS
 OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
 MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
 IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
 CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT,
 TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE
 SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
EOF

    # Build source package
    echo ""
    echo -e "${YELLOW}Building source package...${NC}"
    debuild -S -sa -k${GPG_KEY}
    
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✓ Source package built successfully${NC}"
        
        # Ask to upload
        echo ""
        read -p "Upload to PPA ppa:${LAUNCHPAD_USER}/cortex? (y/n): " UPLOAD
        if [[ $UPLOAD == "y" || $UPLOAD == "Y" ]]; then
            cd "$BUILD_DIR"
            dput ppa:${LAUNCHPAD_USER}/cortex ${PACKAGE_NAME}_${FULL_VERSION}_source.changes
            
            if [ $? -eq 0 ]; then
                echo -e "${GREEN}✓ Uploaded successfully${NC}"
            else
                echo -e "${RED}✗ Upload failed${NC}"
            fi
        else
            echo -e "${YELLOW}Skipped upload. You can upload manually:${NC}"
            echo "  cd $BUILD_DIR"
            echo "  dput ppa:${LAUNCHPAD_USER}/cortex ${PACKAGE_NAME}_${FULL_VERSION}_source.changes"
        fi
    else
        echo -e "${RED}✗ Build failed for $DIST${NC}"
    fi
done

echo ""
echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}           Summary${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""
echo "Build directory: $BUILD_DIR"
echo ""
echo "If upload was successful, packages will be built on Launchpad."
echo "Check status at: https://launchpad.net/~${LAUNCHPAD_USER}/+archive/ubuntu/cortex"
echo ""
echo -e "${GREEN}Users can install with:${NC}"
echo "  sudo add-apt-repository ppa:${LAUNCHPAD_USER}/cortex"
echo "  sudo apt update"
echo "  sudo apt install cortex"
echo ""
echo -e "${YELLOW}Note:${NC} Building on Launchpad typically takes 15-60 minutes."