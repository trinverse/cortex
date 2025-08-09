#!/bin/bash

# Cortex v0.1.0 Release Script
# This script will help you release Cortex to GitHub, Homebrew, and Ubuntu

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}================================================${NC}"
echo -e "${BLUE}     Cortex v0.1.0 Release Process${NC}"
echo -e "${BLUE}================================================${NC}"
echo ""

# Step 1: Git Configuration
echo -e "${YELLOW}Step 1: Git Configuration${NC}"
echo "------------------------"

# Check if git identity is set
if ! git config user.email > /dev/null 2>&1; then
    echo -e "${RED}Git identity not configured!${NC}"
    echo ""
    echo "Please enter your Git information:"
    read -p "Your Name: " git_name
    read -p "Your Email: " git_email
    
    git config user.name "$git_name"
    git config user.email "$git_email"
    echo -e "${GREEN}✓ Git identity configured${NC}"
else
    echo -e "${GREEN}✓ Git identity already configured${NC}"
    echo "  Name: $(git config user.name)"
    echo "  Email: $(git config user.email)"
fi

echo ""

# Step 2: GitHub Repository
echo -e "${YELLOW}Step 2: GitHub Repository Setup${NC}"
echo "--------------------------------"
echo "You need to create a GitHub repository first."
echo ""
echo "1. Go to https://github.com/new"
echo "2. Create a new repository named 'cortex'"
echo "3. Make it public"
echo "4. Don't initialize with README (we already have one)"
echo ""
read -p "Enter your GitHub username: " github_username
read -p "Enter repository name (default: cortex): " repo_name
repo_name=${repo_name:-cortex}

echo ""
echo -e "${YELLOW}Setting up GitHub remote...${NC}"

# Check if origin exists
if git remote | grep -q "^origin$"; then
    echo "Origin already exists. Updating URL..."
    git remote set-url origin "https://github.com/${github_username}/${repo_name}.git"
else
    git remote add origin "https://github.com/${github_username}/${repo_name}.git"
fi

echo -e "${GREEN}✓ GitHub remote configured${NC}"
echo ""

# Step 3: Commit Changes
echo -e "${YELLOW}Step 3: Committing Changes${NC}"
echo "--------------------------"

# Check if there are uncommitted changes
if [[ -n $(git status -s) ]]; then
    echo "Committing all changes..."
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
else
    echo -e "${GREEN}✓ No uncommitted changes${NC}"
fi

echo ""

# Step 4: Create and Push Tag
echo -e "${YELLOW}Step 4: Creating Release Tag${NC}"
echo "-----------------------------"

# Create tag
git tag -a v0.1.0 -m "Release v0.1.0 - Initial public release"
echo -e "${GREEN}✓ Tag v0.1.0 created${NC}"

echo ""

# Step 5: Push to GitHub
echo -e "${YELLOW}Step 5: Push to GitHub${NC}"
echo "----------------------"
echo "This will push your code and tags to GitHub."
echo -e "${YELLOW}You may need to authenticate with GitHub.${NC}"
echo ""
read -p "Ready to push? (y/n): " push_confirm

if [[ $push_confirm == "y" || $push_confirm == "Y" ]]; then
    echo "Pushing to GitHub..."
    git push -u origin main --tags
    echo -e "${GREEN}✓ Code pushed to GitHub${NC}"
else
    echo -e "${YELLOW}Skipped pushing to GitHub${NC}"
fi

echo ""

# Step 6: GitHub Actions Setup
echo -e "${YELLOW}Step 6: GitHub Actions Configuration${NC}"
echo "-------------------------------------"
echo "To enable automatic builds and releases, you need to set up GitHub Secrets."
echo ""
echo -e "${BLUE}Required GitHub Secrets:${NC}"
echo "1. Go to: https://github.com/${github_username}/${repo_name}/settings/secrets/actions"
echo "2. Add the following secrets (as needed):"
echo ""
echo "For Ubuntu/Debian releases:"
echo "  - None required (uses GitHub token)"
echo ""
echo "For Homebrew:"
echo "  - HOMEBREW_TAP_TOKEN: Personal Access Token with repo scope"
echo ""
echo -e "${YELLOW}Optional secrets for other platforms:${NC}"
echo "  - CHOCOLATEY_API_KEY (Windows)"
echo "  - SNAPCRAFT_STORE_CREDENTIALS (Snap Store)"
echo "  - AUR_SSH_KEY (Arch Linux)"
echo ""

# Step 7: Homebrew Setup
echo -e "${YELLOW}Step 7: Homebrew Tap Setup${NC}"
echo "--------------------------"
echo "To publish to Homebrew, you need a tap repository."
echo ""
echo "1. Create a new repository: https://github.com/new"
echo "2. Name it: homebrew-${repo_name}"
echo "3. Create a 'Formula' directory in it"
echo "4. We'll add the formula after the first release"
echo ""
read -p "Have you created the Homebrew tap repository? (y/n): " tap_created

if [[ $tap_created == "y" || $tap_created == "Y" ]]; then
    echo -e "${GREEN}✓ Homebrew tap ready${NC}"
else
    echo -e "${YELLOW}⚠ Remember to create it later${NC}"
fi

echo ""

# Step 8: Build Debian Package Locally
echo -e "${YELLOW}Step 8: Build Debian Package${NC}"
echo "-----------------------------"
echo "Building Debian package for local testing..."

if command -v cargo &> /dev/null; then
    cargo build --release
    
    # Create debian package structure
    mkdir -p debian-build/DEBIAN
    mkdir -p debian-build/usr/bin
    mkdir -p debian-build/usr/share/applications
    mkdir -p debian-build/usr/share/icons/hicolor/scalable/apps
    
    # Copy binary
    cp target/release/cortex debian-build/usr/bin/
    chmod 755 debian-build/usr/bin/cortex
    
    # Copy desktop file
    cp assets/cortex.desktop debian-build/usr/share/applications/
    
    # Copy icon
    cp assets/icons/svg/cortex-icon.svg debian-build/usr/share/icons/hicolor/scalable/apps/cortex.svg
    
    # Create control file
    cat > debian-build/DEBIAN/control << EOF
Package: cortex
Version: 0.1.0
Section: utils
Priority: optional
Architecture: amd64
Maintainer: ${git_name:-Cortex Team} <${git_email:-support@cortex-fm.io}>
Description: Modern terminal file manager
 Cortex is a powerful, fast terminal file manager written in Rust
 with advanced features for power users.
EOF
    
    # Build package
    dpkg-deb --build debian-build cortex_0.1.0_amd64.deb
    echo -e "${GREEN}✓ Debian package built: cortex_0.1.0_amd64.deb${NC}"
else
    echo -e "${RED}✗ Cargo not found, skipping Debian package build${NC}"
fi

echo ""

# Step 9: Next Steps
echo -e "${BLUE}================================================${NC}"
echo -e "${BLUE}              Next Steps${NC}"
echo -e "${BLUE}================================================${NC}"
echo ""
echo -e "${GREEN}Your code is ready for release!${NC}"
echo ""
echo "1. ${YELLOW}GitHub Release:${NC}"
echo "   - Go to: https://github.com/${github_username}/${repo_name}/releases"
echo "   - Click 'Create a new release'"
echo "   - Select tag: v0.1.0"
echo "   - Add release notes from CHANGELOG.md"
echo "   - Upload cortex_0.1.0_amd64.deb if built"
echo ""
echo "2. ${YELLOW}Install Locally (Ubuntu/Debian):${NC}"
echo "   sudo dpkg -i cortex_0.1.0_amd64.deb"
echo ""
echo "3. ${YELLOW}Test the Installation:${NC}"
echo "   cortex --version"
echo "   cortex"
echo ""
echo "4. ${YELLOW}Homebrew Formula:${NC}"
echo "   After GitHub release is created, we'll update the tap"
echo ""
echo "5. ${YELLOW}Ubuntu PPA (Advanced):${NC}"
echo "   - Create Launchpad account: https://launchpad.net"
echo "   - Create PPA: https://launchpad.net/~${github_username}/+activate-ppa"
echo "   - Upload package using dput"
echo ""
echo -e "${BLUE}================================================${NC}"
echo -e "${GREEN}Congratulations on your first release! 🎉${NC}"
echo -e "${BLUE}================================================${NC}"