#!/bin/bash

# Setup script for Cortex development environment

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

echo -e "${CYAN}========================================${NC}"
echo -e "${CYAN}     Cortex Development Setup${NC}"
echo -e "${CYAN}========================================${NC}"
echo ""

# Check for Rust
echo -e "${YELLOW}Checking for Rust...${NC}"
if command -v cargo &> /dev/null; then
    RUST_VERSION=$(rustc --version | cut -d' ' -f2)
    echo -e "${GREEN}✅ Rust found: $RUST_VERSION${NC}"
else
    echo -e "${RED}❌ Rust not found!${NC}"
    echo "Please install Rust from: https://rustup.rs/"
    exit 1
fi

# Check for OpenSSL development files
echo -e "${YELLOW}Checking for OpenSSL development files...${NC}"

# Try to find openssl with pkg-config
if pkg-config --exists openssl 2>/dev/null; then
    OPENSSL_VERSION=$(pkg-config --modversion openssl)
    echo -e "${GREEN}✅ OpenSSL found: $OPENSSL_VERSION${NC}"
else
    echo -e "${RED}❌ OpenSSL development files not found!${NC}"
    echo ""
    echo "Please install OpenSSL development package:"
    echo ""
    
    # Detect distribution
    if [ -f /etc/os-release ]; then
        . /etc/os-release
        case "$ID" in
            ubuntu|debian)
                echo -e "${YELLOW}For Ubuntu/Debian:${NC}"
                echo "  sudo apt-get update"
                echo "  sudo apt-get install libssl-dev pkg-config"
                ;;
            fedora|rhel|centos)
                echo -e "${YELLOW}For Fedora/RHEL/CentOS:${NC}"
                echo "  sudo dnf install openssl-devel pkg-config"
                ;;
            arch|manjaro)
                echo -e "${YELLOW}For Arch/Manjaro:${NC}"
                echo "  sudo pacman -S openssl pkg-config"
                ;;
            opensuse*)
                echo -e "${YELLOW}For openSUSE:${NC}"
                echo "  sudo zypper install libopenssl-devel pkg-config"
                ;;
            *)
                echo -e "${YELLOW}For your distribution, install:${NC}"
                echo "  - OpenSSL development headers (usually libssl-dev or openssl-devel)"
                echo "  - pkg-config"
                ;;
        esac
    fi
    
    echo ""
    echo -e "${YELLOW}After installing, run this script again.${NC}"
    exit 1
fi

# Check for other optional dependencies
echo -e "${YELLOW}Checking optional dependencies...${NC}"

# Check for X11 (for clipboard on Linux)
if pkg-config --exists x11 2>/dev/null; then
    echo -e "${GREEN}✅ X11 found (clipboard support)${NC}"
else
    echo -e "${YELLOW}⚠️  X11 not found (clipboard may not work)${NC}"
    echo "   Install with: sudo apt-get install libx11-dev"
fi

echo ""
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}     Setup Check Complete!${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""

# Try to build
echo -e "${YELLOW}Attempting to build Cortex...${NC}"
echo ""

if cargo build; then
    echo ""
    echo -e "${GREEN}✅ Build successful!${NC}"
    echo ""
    echo -e "${CYAN}You can now run Cortex with:${NC}"
    echo "  ./target/debug/cortex"
    echo ""
    echo -e "${CYAN}Or use the development scripts:${NC}"
    echo "  ./dev.sh        # Build and run once"
    echo "  ./dev-live.sh   # Live reload mode"
    echo ""
else
    echo ""
    echo -e "${RED}❌ Build failed!${NC}"
    echo ""
    echo "Please check the error messages above."
    echo "Common issues:"
    echo "  1. Missing libssl-dev (see instructions above)"
    echo "  2. Network issues downloading crates"
    echo "  3. Insufficient disk space"
    exit 1
fi