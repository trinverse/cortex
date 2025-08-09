#!/bin/bash

# Auto-restart Cortex when rebuild completes
# Run this in a second terminal

set -e

echo "🔄 Cortex Auto-Test Mode"
echo "========================"
echo ""
echo "This will automatically restart Cortex when changes are detected."
echo "Press 'q' in Cortex to quit, and it will restart with new changes."
echo "Press Ctrl+C here to stop auto-restart."
echo ""
echo "Waiting for initial build..."

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Build first
cargo build

# Function to check if build is complete
wait_for_build() {
    echo -e "${YELLOW}Waiting for rebuild...${NC}"
    # Simple file watcher - checks if binary was updated
    if [ "$(uname)" = "Darwin" ]; then
        # macOS
        stat -f %m target/debug/cortex > /tmp/cortex_build_time
    else
        # Linux
        stat -c %Y target/debug/cortex > /tmp/cortex_build_time
    fi
    
    while true; do
        sleep 1
        if [ "$(uname)" = "Darwin" ]; then
            NEW_TIME=$(stat -f %m target/debug/cortex 2>/dev/null || echo "0")
        else
            NEW_TIME=$(stat -c %Y target/debug/cortex 2>/dev/null || echo "0")
        fi
        
        OLD_TIME=$(cat /tmp/cortex_build_time)
        
        if [ "$NEW_TIME" != "$OLD_TIME" ] && [ "$NEW_TIME" != "0" ]; then
            echo "$NEW_TIME" > /tmp/cortex_build_time
            echo -e "${GREEN}✅ New build detected!${NC}"
            return 0
        fi
    done
}

# Main loop
while true; do
    echo -e "${GREEN}Starting Cortex...${NC}"
    echo "----------------------------------------"
    
    # Run Cortex
    ./target/debug/cortex || true
    
    echo ""
    echo "Cortex exited."
    
    # Wait for new build
    wait_for_build
    
    echo ""
done