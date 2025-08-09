#!/bin/bash

# Cortex v0.1.0 Release Script - For Existing Repository
# Repository: https://github.com/trinverse/cortex

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}================================================${NC}"
echo -e "${BLUE}     Cortex v0.1.0 Release Process${NC}"
echo -e "${BLUE}     Repository: github.com/trinverse/cortex${NC}"
echo -e "${BLUE}================================================${NC}"
echo ""

# Step 1: Git Configuration Check
echo -e "${YELLOW}Step 1: Checking Git Configuration${NC}"
echo "-----------------------------------"

if ! git config user.email > /dev/null 2>&1; then
    echo -e "${RED}Git identity not configured!${NC}"
    echo "Please configure git first:"
    echo "  git config --global user.name \"Your Name\""
    echo "  git config --global user.email \"your.email@example.com\""
    exit 1
else
    echo -e "${GREEN}✓ Git configured${NC}"
    echo "  Name: $(git config user.name)"
    echo "  Email: $(git config user.email)"
fi

echo ""

# Step 2: Build Debian Package
echo -e "${YELLOW}Step 2: Building Debian Package${NC}"
echo "--------------------------------"

if command -v cargo &> /dev/null; then
    echo "Building release binary..."
    cargo build --release --quiet
    
    # Create debian package structure
    mkdir -p debian-build/DEBIAN
    mkdir -p debian-build/usr/bin
    mkdir -p debian-build/usr/share/applications
    mkdir -p debian-build/usr/share/icons/hicolor/scalable/apps
    
    # Copy binary
    cp target/release/cortex debian-build/usr/bin/
    chmod 755 debian-build/usr/bin/cortex
    
    # Copy desktop file
    if [ -f assets/cortex.desktop ]; then
        cp assets/cortex.desktop debian-build/usr/share/applications/
    fi
    
    # Copy icon
    if [ -f assets/icons/svg/cortex-icon.svg ]; then
        cp assets/icons/svg/cortex-icon.svg debian-build/usr/share/icons/hicolor/scalable/apps/cortex.svg
    fi
    
    # Create control file
    cat > debian-build/DEBIAN/control << EOF
Package: cortex
Version: 0.1.0
Section: utils
Priority: optional
Architecture: amd64
Maintainer: Trinverse <admin@trinverse.com>
Homepage: https://github.com/trinverse/cortex
Description: Modern terminal file manager
 Cortex is a powerful, fast terminal file manager written in Rust
 with advanced features for power users including dual-pane interface,
 plugin system, and extensive keyboard shortcuts.
EOF
    
    # Build package
    dpkg-deb --build debian-build cortex_0.1.0_amd64.deb 2>/dev/null
    echo -e "${GREEN}✓ Debian package built: cortex_0.1.0_amd64.deb${NC}"
else
    echo -e "${RED}✗ Cargo not found, skipping Debian package build${NC}"
fi

echo ""

# Step 3: Build tarball for Linux
echo -e "${YELLOW}Step 3: Building Release Tarball${NC}"
echo "---------------------------------"

if [ -f target/release/cortex ]; then
    mkdir -p release-build
    cp target/release/cortex release-build/
    cp README.md release-build/
    cp LICENSE release-build/ 2>/dev/null || echo "No LICENSE file"
    tar -czf cortex-0.1.0-x86_64-linux.tar.gz -C release-build .
    rm -rf release-build
    echo -e "${GREEN}✓ Tarball created: cortex-0.1.0-x86_64-linux.tar.gz${NC}"
else
    echo -e "${YELLOW}⚠ No release binary found${NC}"
fi

echo ""

# Step 4: Commit and Tag
echo -e "${YELLOW}Step 4: Preparing Git Release${NC}"
echo "------------------------------"

# Check for uncommitted changes
if [[ -n $(git status -s) ]]; then
    echo "Found uncommitted changes:"
    git status --short
    echo ""
    read -p "Commit all changes for v0.1.0 release? (y/n): " commit_confirm
    
    if [[ $commit_confirm == "y" || $commit_confirm == "Y" ]]; then
        git add -A
        git commit -m "Release v0.1.0 - Initial public release

- Complete file manager implementation with dual-pane interface
- Platform integration (trash, clipboard)  
- Plugin system with Lua support
- Auto-update mechanism
- Package manager configurations
- CI/CD workflows for multi-platform builds
- Application icons and branding
- Comprehensive documentation

First public alpha release of Cortex File Manager."
        echo -e "${GREEN}✓ Changes committed${NC}"
    fi
else
    echo -e "${GREEN}✓ No uncommitted changes${NC}"
fi

# Check if tag exists
if git rev-parse v0.1.0 >/dev/null 2>&1; then
    echo -e "${YELLOW}Tag v0.1.0 already exists${NC}"
    read -p "Delete and recreate tag? (y/n): " tag_confirm
    if [[ $tag_confirm == "y" || $tag_confirm == "Y" ]]; then
        git tag -d v0.1.0
        git tag -a v0.1.0 -m "Release v0.1.0 - Initial public release"
        echo -e "${GREEN}✓ Tag recreated${NC}"
    fi
else
    git tag -a v0.1.0 -m "Release v0.1.0 - Initial public release"
    echo -e "${GREEN}✓ Tag v0.1.0 created${NC}"
fi

echo ""

# Step 5: Push to GitHub
echo -e "${YELLOW}Step 5: Push to GitHub${NC}"
echo "----------------------"
echo "Repository: https://github.com/trinverse/cortex"
echo ""
read -p "Push code and tags to GitHub? (y/n): " push_confirm

if [[ $push_confirm == "y" || $push_confirm == "Y" ]]; then
    echo "Pushing to GitHub..."
    git push origin main
    git push origin v0.1.0
    echo -e "${GREEN}✓ Pushed to GitHub${NC}"
else
    echo -e "${YELLOW}Skipped pushing to GitHub${NC}"
    echo "You can push manually with:"
    echo "  git push origin main"
    echo "  git push origin v0.1.0"
fi

echo ""

# Step 6: Create Release Notes
echo -e "${YELLOW}Step 6: GitHub Release Instructions${NC}"
echo "------------------------------------"
echo "To create the GitHub release:"
echo ""
echo "1. Go to: ${BLUE}https://github.com/trinverse/cortex/releases/new${NC}"
echo ""
echo "2. Select tag: ${GREEN}v0.1.0${NC}"
echo ""
echo "3. Release title: ${GREEN}Cortex v0.1.0 - Initial Release${NC}"
echo ""
echo "4. Upload these files as release assets:"
if [ -f cortex_0.1.0_amd64.deb ]; then
    echo "   - ${GREEN}cortex_0.1.0_amd64.deb${NC} (Ubuntu/Debian)"
fi
if [ -f cortex-0.1.0-x86_64-linux.tar.gz ]; then
    echo "   - ${GREEN}cortex-0.1.0-x86_64-linux.tar.gz${NC} (Generic Linux)"
fi
echo ""
echo "5. Copy release notes from CHANGELOG.md"
echo ""
echo "6. Check '☑ This is a pre-release' (alpha version)"
echo ""
echo "7. Click 'Publish release'"
echo ""

# Step 7: Installation Instructions
echo -e "${BLUE}================================================${NC}"
echo -e "${BLUE}           Installation Instructions${NC}"
echo -e "${BLUE}================================================${NC}"
echo ""
echo -e "${YELLOW}Ubuntu/Debian:${NC}"
echo "  wget https://github.com/trinverse/cortex/releases/download/v0.1.0/cortex_0.1.0_amd64.deb"
echo "  sudo dpkg -i cortex_0.1.0_amd64.deb"
echo "  cortex"
echo ""
echo -e "${YELLOW}Generic Linux:${NC}"
echo "  wget https://github.com/trinverse/cortex/releases/download/v0.1.0/cortex-0.1.0-x86_64-linux.tar.gz"
echo "  tar -xzf cortex-0.1.0-x86_64-linux.tar.gz"
echo "  ./cortex"
echo ""
echo -e "${YELLOW}From Source:${NC}"
echo "  git clone https://github.com/trinverse/cortex.git"
echo "  cd cortex"
echo "  cargo build --release"
echo "  ./target/release/cortex"
echo ""

# Step 8: Homebrew Formula
echo -e "${BLUE}================================================${NC}"
echo -e "${BLUE}             Homebrew Setup (Optional)${NC}"
echo -e "${BLUE}================================================${NC}"
echo ""
echo "To distribute via Homebrew, create a tap repository:"
echo ""
echo "1. Create new repo: ${BLUE}https://github.com/trinverse/homebrew-cortex${NC}"
echo "2. Add Formula/cortex.rb with the content from packaging/homebrew/"
echo "3. Users can then install with:"
echo "   ${GREEN}brew tap trinverse/cortex${NC}"
echo "   ${GREEN}brew install cortex${NC}"
echo ""

echo -e "${BLUE}================================================${NC}"
echo -e "${GREEN}        Release Preparation Complete! 🎉${NC}"
echo -e "${BLUE}================================================${NC}"
echo ""
echo "Files ready for release:"
ls -lh cortex*.{deb,tar.gz} 2>/dev/null || echo "No release files found"
echo ""
echo "Next: Create GitHub release at:"
echo "${BLUE}https://github.com/trinverse/cortex/releases/new${NC}"