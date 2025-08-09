#!/bin/bash

# Simple live reload - rebuilds when Cortex exits
set -e

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

echo -e "${CYAN}🔄 Cortex Live Reload${NC}"
echo "================================"
echo -e "${YELLOW}Instructions:${NC}"
echo "1. Make your code changes"
echo "2. Save the file"  
echo "3. Exit Cortex with Ctrl+Q"
echo "4. It auto-rebuilds and restarts"
echo ""
echo "Press Ctrl+C here to stop"
echo "================================"
echo ""

# Initial build
cargo build || exit 1

# Main loop
while true; do
    # Run Cortex
    ./target/debug/cortex || true
    
    # After Cortex exits, rebuild
    echo ""
    echo -e "${YELLOW}🔨 Rebuilding...${NC}"
    
    if cargo build; then
        echo -e "${GREEN}✅ Build successful!${NC}"
        echo -e "${CYAN}Restarting Cortex...${NC}"
        echo ""
    else
        echo ""
        echo -e "${YELLOW}⚠️  Build failed. Press Enter to retry or Ctrl+C to exit${NC}"
        read -r
    fi
done