#!/bin/bash

# Enhanced auto-rebuild with better feedback
set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

echo -e "${CYAN}🚀 Cortex Auto-Reload Development Mode${NC}"
echo "======================================="
echo ""
echo -e "${YELLOW}How it works:${NC}"
echo "1. This script watches for changes to .rs files"
echo "2. When you save a file, it auto-rebuilds"
echo "3. After build, Cortex restarts automatically"
echo ""
echo -e "${GREEN}Commands in Cortex:${NC}"
echo "  Ctrl+Q - Exit Cortex (returns to watch mode)"
echo "  /reload - Reload file panels (inside Cortex)"
echo ""
echo -e "${BLUE}Press Ctrl+C here to stop watching${NC}"
echo "======================================="
echo ""

# Function to get checksum of all Rust files
get_checksum() {
    if command -v md5sum >/dev/null 2>&1; then
        find cortex-* -name "*.rs" -o -name "*.toml" 2>/dev/null | xargs md5sum 2>/dev/null | md5sum | cut -d' ' -f1
    elif command -v md5 >/dev/null 2>&1; then
        find cortex-* -name "*.rs" -o -name "*.toml" 2>/dev/null | xargs md5 -q 2>/dev/null | md5 -q
    else
        # Fallback to modification time
        find cortex-* -name "*.rs" -o -name "*.toml" -type f -exec stat -f %m {} \; 2>/dev/null | sort | md5 -q
    fi
}

# Function to build and run
build_and_run() {
    echo ""
    echo -e "${YELLOW}🔨 Building...${NC}"
    
    if cargo build 2>&1 | tee /tmp/cortex-build.log; then
        echo -e "${GREEN}✅ Build successful!${NC}"
        echo ""
        echo -e "${CYAN}Starting Cortex...${NC}"
        echo "----------------------------------------"
        
        # Run Cortex
        ./target/debug/cortex || true
        
        echo "----------------------------------------"
        echo -e "${YELLOW}Cortex exited. Watching for changes...${NC}"
    else
        echo -e "${RED}❌ Build failed!${NC}"
        echo "Fix the errors above and save a file to retry."
        echo ""
        # Show just the error summary
        grep -E "^error" /tmp/cortex-build.log || true
    fi
}

# Initial build
build_and_run

# Store initial checksum
LAST_CHECKSUM=$(get_checksum)

# Watch loop with better feedback
echo -e "${BLUE}👁  Watching for changes...${NC}"
DOTS=""
while true; do
    sleep 1  # Check every second for faster response
    
    CURRENT_CHECKSUM=$(get_checksum)
    
    if [ "$CURRENT_CHECKSUM" != "$LAST_CHECKSUM" ]; then
        echo ""
        echo -e "${GREEN}📝 Changes detected!${NC}"
        LAST_CHECKSUM=$CURRENT_CHECKSUM
        build_and_run
        echo -e "${BLUE}👁  Watching for changes...${NC}"
        DOTS=""
    else
        # Show activity indicator
        printf "\r${BLUE}👁  Watching for changes${DOTS}${NC}   "
        DOTS="${DOTS}."
        if [ ${#DOTS} -gt 3 ]; then
            DOTS=""
        fi
    fi
done