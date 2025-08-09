#!/bin/bash

# Auto-rebuild development mode using built-in tools
# No cargo-watch required!

set -e

echo "🚀 Cortex Auto-Development Mode"
echo "================================"
echo ""
echo "Watching for file changes..."
echo "Save a .rs file to trigger rebuild"
echo "Press Ctrl+C to stop"
echo ""

# Function to get last modified time of Rust files
get_last_modified() {
    if [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS
        find cortex-* -name "*.rs" -type f -exec stat -f %m {} \; 2>/dev/null | sort -n | tail -1
    else
        # Linux
        find cortex-* -name "*.rs" -type f -exec stat -c %Y {} \; 2>/dev/null | sort -n | tail -1
    fi
}

# Function to build and run
build_and_run() {
    echo "🔨 Building..."
    if cargo build; then
        echo "✅ Build successful!"
        echo "Starting Cortex (press Ctrl+Q to quit and rebuild)..."
        echo "----------------------------------------"
        ./target/debug/cortex || true
        echo "----------------------------------------"
        echo "Cortex exited. Watching for changes..."
    else
        echo "❌ Build failed! Fix errors and save a file to retry."
    fi
}

# Initial build
build_and_run

# Store last modified time
LAST_MODIFIED=$(get_last_modified)

# Watch loop
echo "Watching for changes in .rs files..."
while true; do
    sleep 2  # Check every 2 seconds
    
    CURRENT_MODIFIED=$(get_last_modified)
    
    if [ "$CURRENT_MODIFIED" != "$LAST_MODIFIED" ]; then
        echo ""
        echo "📝 Changes detected!"
        LAST_MODIFIED=$CURRENT_MODIFIED
        build_and_run
    fi
done