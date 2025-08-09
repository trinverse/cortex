#!/bin/bash

# Build script that skips SSH/FTP features (no OpenSSL required)

set -e

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${CYAN}========================================${NC}"
echo -e "${CYAN}     Building Cortex without SSH/FTP${NC}"
echo -e "${CYAN}========================================${NC}"
echo ""
echo -e "${YELLOW}This build skips SSH/FTP features to avoid OpenSSL dependency${NC}"
echo ""

# Build without SSH features
echo -e "${YELLOW}Building...${NC}"
if cargo build --no-default-features; then
    echo ""
    echo -e "${GREEN}✅ Build successful!${NC}"
    echo ""
    echo -e "${CYAN}You can now run Cortex with:${NC}"
    echo "  ./target/debug/cortex"
    echo ""
    echo -e "${YELLOW}Note: SSH/FTP connections will not be available in this build${NC}"
    echo ""
    
    # Offer to run it
    echo -e "${CYAN}Would you like to run Cortex now? (y/n)${NC}"
    read -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        ./target/debug/cortex
    fi
else
    echo ""
    echo -e "${RED}❌ Build failed!${NC}"
    echo ""
    echo "Even without SSH, the build failed. Please check the errors above."
    exit 1
fi