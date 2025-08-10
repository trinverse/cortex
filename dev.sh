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
MODE="simple"  # Changed default to simple mode - no dependencies needed
BUILD_TYPE="debug"

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --release)
            BUILD_TYPE="release"
            shift
            ;;
        --watch)
            MODE="watch"
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
            echo "  --watch      Use cargo-watch for instant rebuilds (requires cargo-watch)"
            echo "  --hot        Use hot reload mode (rebuild in background)"
            echo "  --help       Show this help message"
            echo ""
            echo "Default: Simple mode - no extra tools required!"
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

# Check for cargo-watch only if watch mode is requested
if [ "$MODE" = "watch" ]; then
    if ! cargo help watch &> /dev/null; then
        echo -e "${YELLOW}⚠️  cargo-watch is not available${NC}"
        echo -e "${GREEN}Switching to simple mode (works great without dependencies!)${NC}"
        MODE="simple"
        echo ""
    fi
fi

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
    # Check if cargo watch command exists
    if cargo watch --version &> /dev/null; then
        return 0
    fi
    
    echo -e "${YELLOW}cargo-watch not found.${NC}"
    echo -e "${YELLOW}Would you like to:${NC}"
    echo "  1) Try installing cargo-watch"
    echo "  2) Use simple mode (no installation needed)"
    echo "  3) Use hot reload mode"
    echo ""
    echo -n "Choose [1-3]: "
    read -r response
    
    case $response in
        1)
            echo "Installing cargo-watch..."
            if cargo install cargo-watch; then
                echo -e "${GREEN}✓ cargo-watch installed successfully${NC}"
                return 0
            else
                echo -e "${RED}Failed to install cargo-watch${NC}"
                echo -e "${YELLOW}Falling back to simple mode...${NC}"
                MODE="simple"
                return 1
            fi
            ;;
        2)
            echo -e "${BLUE}Using simple mode...${NC}"
            MODE="simple"
            return 1
            ;;
        3)
            echo -e "${MAGENTA}Using hot reload mode...${NC}"
            MODE="hot"
            return 1
            ;;
        *)
            echo -e "${YELLOW}Invalid choice. Using simple mode...${NC}"
            MODE="simple"
            return 1
            ;;
    esac
}

# Function for simple file watching
simple_watch() {
    echo -e "${GREEN}📦 Simple Development Mode${NC}"
    echo -e "${BLUE}No external tools required!${NC}"
    echo ""
    echo -e "${YELLOW}How it works:${NC}"
    echo "  1. Builds and runs Cortex"
    echo "  2. When you exit (Ctrl+Q), checks for file changes"
    echo "  3. Automatically rebuilds if changes detected"
    echo "  4. Restarts Cortex with your changes"
    echo ""
    echo -e "${CYAN}Press Ctrl+C here to stop development mode${NC}"
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
    FIRST_RUN=true
    
    while true; do
        if [ "$FIRST_RUN" = true ]; then
            echo -e "${GREEN}🔨 Building Cortex...${NC}"
            FIRST_RUN=false
        else
            echo -e "${GREEN}🔄 Rebuilding with your changes...${NC}"
        fi
        
        if cargo build $BUILD_FLAGS; then
            echo -e "${CYAN}═══════════════════════════════════════${NC}"
            echo -e "${GREEN}✅ Build successful! Starting Cortex...${NC}"
            echo -e "${CYAN}═══════════════════════════════════════${NC}"
            echo ""
            
            # Run Cortex
            cargo run $BUILD_FLAGS --bin cortex
            
            echo ""
            echo -e "${YELLOW}Cortex exited. Checking for changes...${NC}"
            
            # Wait for changes
            CHANGES_FOUND=false
            for i in {1..5}; do
                CURRENT_CHECKSUM=$(get_checksum)
                if [ "$CURRENT_CHECKSUM" != "$LAST_CHECKSUM" ]; then
                    LAST_CHECKSUM=$CURRENT_CHECKSUM
                    echo -e "${GREEN}✨ Changes detected!${NC}"
                    CHANGES_FOUND=true
                    break
                fi
                if [ $i -lt 5 ]; then
                    echo -n "."
                    sleep 1
                fi
            done
            
            if [ "$CHANGES_FOUND" = false ]; then
                echo ""
                echo -e "${BLUE}No changes detected. Restarting...${NC}"
            fi
        else
            echo -e "${RED}❌ Build failed! Fix errors and save to retry.${NC}"
            echo -e "${YELLOW}Waiting for file changes...${NC}"
            
            # Wait for changes before retrying
            while true; do
                CURRENT_CHECKSUM=$(get_checksum)
                if [ "$CURRENT_CHECKSUM" != "$LAST_CHECKSUM" ]; then
                    LAST_CHECKSUM=$CURRENT_CHECKSUM
                    echo -e "${GREEN}Changes detected! Retrying build...${NC}"
                    break
                fi
                sleep 2
            done
        fi
        
        echo ""
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
case $MODE in
    watch)
        echo -e "${BLUE}Mode: cargo-watch${NC}"
        echo ""
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