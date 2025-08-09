#!/bin/bash

# Hot reload development mode - rebuilds in background
set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

echo -e "${CYAN}🔥 Cortex Hot Reload Mode${NC}"
echo "======================================="
echo ""
echo -e "${YELLOW}How it works:${NC}"
echo "1. Watches for .rs file changes in background"
echo "2. Auto-rebuilds when you save"
echo "3. Shows notification when rebuild is ready"
echo "4. Exit Cortex and it will restart with new code"
echo ""
echo -e "${GREEN}Workflow:${NC}"
echo "  1. Edit your code"
echo "  2. Save the file"
echo "  3. Wait for 'Rebuild complete' message"
echo "  4. Press Ctrl+Q in Cortex to reload"
echo ""
echo -e "${BLUE}Press Ctrl+C here to stop${NC}"
echo "======================================="
echo ""

# Function to get checksum
get_checksum() {
    if command -v md5sum >/dev/null 2>&1; then
        find cortex-* -name "*.rs" -o -name "*.toml" 2>/dev/null | xargs md5sum 2>/dev/null | md5sum | cut -d' ' -f1
    elif command -v md5 >/dev/null 2>&1; then
        find cortex-* -name "*.rs" -o -name "*.toml" 2>/dev/null | xargs md5 -q 2>/dev/null | md5 -q
    else
        find cortex-* -name "*.rs" -o -name "*.toml" -type f -exec stat -f %m {} \; 2>/dev/null | sort | md5 -q
    fi
}

# Function to rebuild in background
rebuild_in_background() {
    echo -e "${YELLOW}🔨 Rebuilding in background...${NC}"
    if cargo build 2>&1 > /tmp/cortex-build.log; then
        echo -e "${GREEN}✅ Rebuild complete! Exit Cortex (Ctrl+Q) to use new version${NC}"
        # Play a sound if available (macOS)
        if command -v afplay >/dev/null 2>&1; then
            afplay /System/Library/Sounds/Glass.aiff 2>/dev/null || true
        fi
    else
        echo -e "${RED}❌ Build failed! Check /tmp/cortex-build.log${NC}"
        tail -5 /tmp/cortex-build.log
    fi
}

# Initial build
echo -e "${YELLOW}🔨 Initial build...${NC}"
if cargo build; then
    echo -e "${GREEN}✅ Build successful!${NC}"
else
    echo -e "${RED}❌ Initial build failed!${NC}"
    exit 1
fi

# Store initial checksum
LAST_CHECKSUM=$(get_checksum)

# Start Cortex in background
echo -e "${CYAN}Starting Cortex...${NC}"
echo "----------------------------------------"

while true; do
    # Run Cortex
    ./target/debug/cortex || true
    
    echo "----------------------------------------"
    echo -e "${YELLOW}Cortex exited. Checking for updates...${NC}"
    
    # Check if files changed while Cortex was running
    CURRENT_CHECKSUM=$(get_checksum)
    if [ "$CURRENT_CHECKSUM" != "$LAST_CHECKSUM" ]; then
        echo -e "${GREEN}📝 Changes detected! Rebuilding...${NC}"
        LAST_CHECKSUM=$CURRENT_CHECKSUM
        
        if cargo build 2>&1 | tee /tmp/cortex-build.log; then
            echo -e "${GREEN}✅ Build successful! Restarting...${NC}"
        else
            echo -e "${RED}❌ Build failed! Fix errors and press Enter to retry${NC}"
            grep -E "^error" /tmp/cortex-build.log || true
            read -r
            continue
        fi
    fi
    
    echo -e "${CYAN}Restarting Cortex...${NC}"
    echo "----------------------------------------"
done &

# Background file watcher
(
    while true; do
        sleep 2
        CURRENT_CHECKSUM=$(get_checksum)
        if [ "$CURRENT_CHECKSUM" != "$LAST_CHECKSUM" ]; then
            LAST_CHECKSUM=$CURRENT_CHECKSUM
            rebuild_in_background
        fi
    done
) &

# Wait for background processes
wait