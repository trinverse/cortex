#!/bin/bash

# Cortex Development Mode - Smart auto-rebuild and run
# Combines the best features from all dev scripts

set -e

# Color codes for better output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
NC='\033[0m' # No Color

# Default values
MODE="watch"
BUILD_TYPE="debug"

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --release)
            BUILD_TYPE="release"
            shift
            ;;
        --simple)
            MODE="simple"
            shift
            ;;
        --hot)
            MODE="hot"
            shift
            ;;
        --help)
            echo "Cortex Development Script"
            echo ""
            echo "Usage: ./dev.sh [options]"
            echo ""
            echo "Options:"
            echo "  --release    Build in release mode (optimized but slower build)"
            echo "  --simple     Use simple file watching (no cargo-watch needed)"
            echo "  --hot        Use hot reload mode (rebuild in background)"
            echo "  --help       Show this help message"
            echo ""
            echo "Default: Uses cargo-watch for automatic rebuild and run"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Display header
echo -e "${CYAN}╔══════════════════════════════════════╗${NC}"
echo -e "${CYAN}║     🚀 Cortex Development Mode 🚀     ║${NC}"
echo -e "${CYAN}╚══════════════════════════════════════╝${NC}"
echo ""

# Set build flags based on type
if [ "$BUILD_TYPE" = "release" ]; then
    BUILD_FLAGS="--release"
    TARGET_DIR="target/release"
    echo -e "${YELLOW}Build mode: RELEASE (optimized)${NC}"
else
    BUILD_FLAGS=""
    TARGET_DIR="target/debug"
    echo -e "${GREEN}Build mode: DEBUG (fast compile)${NC}"
fi

# Function to check if cargo-watch is installed
check_cargo_watch() {
    if ! command -v cargo-watch &> /dev/null; then
        echo -e "${YELLOW}cargo-watch not found.${NC}"
        echo -e "Would you like to install it? (y/n)"
        read -r response
        if [[ "$response" == "y" ]]; then
            echo "Installing cargo-watch..."
            cargo install cargo-watch
            return 0
        else
            echo "Falling back to simple mode..."
            MODE="simple"
            return 1
        fi
    fi
    return 0
}

# Function for simple file watching
simple_watch() {
    echo -e "${BLUE}Simple watch mode - checking for changes every 2 seconds${NC}"
    echo ""
    
    get_checksum() {
        if command -v md5sum >/dev/null 2>&1; then
            find cortex-* -name "*.rs" -o -name "*.toml" 2>/dev/null | xargs md5sum 2>/dev/null | md5sum | cut -d' ' -f1
        elif command -v md5 >/dev/null 2>&1; then
            find cortex-* -name "*.rs" -o -name "*.toml" 2>/dev/null | xargs md5 -q 2>/dev/null | md5 -q
        else
            date +%s
        fi
    }
    
    LAST_CHECKSUM=$(get_checksum)
    
    while true; do
        echo -e "${GREEN}Building and running Cortex...${NC}"
        cargo build $BUILD_FLAGS
        
        echo -e "${CYAN}═══════════════════════════════════════${NC}"
        echo -e "${GREEN}✓ Build complete! Starting Cortex...${NC}"
        echo -e "${CYAN}═══════════════════════════════════════${NC}"
        echo ""
        
        cargo run $BUILD_FLAGS --bin cortex
        
        echo ""
        echo -e "${YELLOW}Cortex exited. Checking for changes...${NC}"
        
        # Wait for changes
        while true; do
            CURRENT_CHECKSUM=$(get_checksum)
            if [ "$CURRENT_CHECKSUM" != "$LAST_CHECKSUM" ]; then
                LAST_CHECKSUM=$CURRENT_CHECKSUM
                echo -e "${GREEN}Changes detected! Rebuilding...${NC}"
                break
            fi
            sleep 2
        done
    done
}

# Function for hot reload mode
hot_reload() {
    echo -e "${MAGENTA}Hot reload mode - rebuilds in background${NC}"
    echo -e "${YELLOW}Workflow:${NC}"
    echo "  1. Edit your code"
    echo "  2. Save the file"
    echo "  3. Wait for rebuild notification"
    echo "  4. Exit Cortex (Ctrl+Q) to use new version"
    echo ""
    
    # Initial build
    echo -e "${GREEN}Initial build...${NC}"
    cargo build $BUILD_FLAGS
    
    # Start file watcher in background
    (
        get_checksum() {
            if command -v md5sum >/dev/null 2>&1; then
                find cortex-* -name "*.rs" -o -name "*.toml" 2>/dev/null | xargs md5sum 2>/dev/null | md5sum | cut -d' ' -f1
            elif command -v md5 >/dev/null 2>&1; then
                find cortex-* -name "*.rs" -o -name "*.toml" 2>/dev/null | xargs md5 -q 2>/dev/null | md5 -q
            else
                date +%s
            fi
        }
        
        LAST_CHECKSUM=$(get_checksum)
        
        while true; do
            CURRENT_CHECKSUM=$(get_checksum)
            if [ "$CURRENT_CHECKSUM" != "$LAST_CHECKSUM" ]; then
                LAST_CHECKSUM=$CURRENT_CHECKSUM
                echo -e "\n${YELLOW}🔨 Changes detected, rebuilding...${NC}"
                if cargo build $BUILD_FLAGS 2>&1 > /tmp/cortex-build.log; then
                    echo -e "${GREEN}✅ Rebuild complete! Exit Cortex to reload${NC}"
                    # Notification sound on macOS
                    if command -v afplay >/dev/null 2>&1; then
                        afplay /System/Library/Sounds/Glass.aiff 2>/dev/null || true
                    fi
                else
                    echo -e "${RED}❌ Build failed! Check /tmp/cortex-build.log${NC}"
                fi
            fi
            sleep 2
        done
    ) &
    
    WATCHER_PID=$!
    
    # Run Cortex in a loop
    while true; do
        echo -e "${CYAN}═══════════════════════════════════════${NC}"
        echo -e "${GREEN}Starting Cortex...${NC}"
        echo -e "${CYAN}═══════════════════════════════════════${NC}"
        echo ""
        
        cargo run $BUILD_FLAGS --bin cortex
        
        echo ""
        echo -e "${YELLOW}Restarting with latest build...${NC}"
        sleep 1
    done
    
    # Cleanup on exit
    trap "kill $WATCHER_PID 2>/dev/null" EXIT
}

# Function for cargo-watch mode
cargo_watch_mode() {
    echo -e "${GREEN}Using cargo-watch for automatic rebuild${NC}"
    echo -e "${YELLOW}Features:${NC}"
    echo "  ✓ Instant rebuild on file save"
    echo "  ✓ Clear terminal before each build"
    echo "  ✓ Automatic restart after successful build"
    echo ""
    echo -e "${BLUE}Tips:${NC}"
    echo "  • Exit Cortex (Ctrl+Q) to see rebuild"
    echo "  • Press Ctrl+C here to stop watching"
    echo ""
    
    cargo watch \
        --clear \
        --watch cortex-core \
        --watch cortex-tui \
        --watch cortex-cli \
        --watch cortex-plugins \
        --watch cortex-platform \
        --watch cortex-updater \
        --exec "build $BUILD_FLAGS" \
        --exec "run $BUILD_FLAGS --bin cortex"
}

# Main execution
echo -e "${BLUE}Mode: $MODE${NC}"
echo ""

case $MODE in
    watch)
        if check_cargo_watch; then
            cargo_watch_mode
        else
            simple_watch
        fi
        ;;
    simple)
        simple_watch
        ;;
    hot)
        hot_reload
        ;;
    *)
        echo -e "${RED}Unknown mode: $MODE${NC}"
        exit 1
        ;;
esac