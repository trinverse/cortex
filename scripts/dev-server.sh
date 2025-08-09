#!/bin/bash

# Advanced development server with hot-reload capabilities
# Watches for changes and provides various development features

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
WATCH_DIRS="cortex-core cortex-tui cortex-cli cortex-plugins"
BUILD_MODE=${1:-debug}  # debug or release
AUTO_RUN=${2:-false}     # automatically run after build

echo -e "${GREEN}🚀 Cortex Development Server${NC}"
echo "================================"
echo "Build mode: $BUILD_MODE"
echo "Auto-run: $AUTO_RUN"
echo ""

# Install dependencies if needed
check_dependencies() {
    if ! command -v cargo-watch &> /dev/null; then
        echo -e "${YELLOW}Installing cargo-watch...${NC}"
        cargo install cargo-watch
    fi
    
    if ! command -v watchexec &> /dev/null; then
        echo -e "${YELLOW}Note: Install watchexec for better file watching:${NC}"
        echo "  cargo install watchexec-cli"
    fi
}

# Build function
build_project() {
    echo -e "${YELLOW}Building...${NC}"
    if [ "$BUILD_MODE" = "release" ]; then
        cargo build --release
    else
        cargo build
    fi
    
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✅ Build successful!${NC}"
        
        if [ "$AUTO_RUN" = "true" ]; then
            echo -e "${GREEN}Starting Cortex...${NC}"
            if [ "$BUILD_MODE" = "release" ]; then
                ./target/release/cortex
            else
                ./target/debug/cortex
            fi
        fi
    else
        echo -e "${RED}❌ Build failed!${NC}"
    fi
}

# Watch for changes
watch_changes() {
    echo "Watching for changes in: $WATCH_DIRS"
    echo "Press Ctrl+C to stop"
    echo ""
    
    # Initial build
    build_project
    
    # Watch for changes
    if command -v watchexec &> /dev/null; then
        watchexec -w cortex-core -w cortex-tui -w cortex-cli -w cortex-plugins \
                  -e rs,toml \
                  --clear \
                  -- bash -c "$(declare -f build_project); build_project"
    else
        cargo watch -w cortex-core -w cortex-tui -w cortex-cli -w cortex-plugins \
                    -x "build $([ "$BUILD_MODE" = "release" ] && echo --release)" \
                    -s "echo '✅ Build complete!'"
    fi
}

# Main execution
check_dependencies
watch_changes