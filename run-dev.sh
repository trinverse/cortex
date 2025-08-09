#!/bin/bash

# Simple development runner - watches for changes and rebuilds automatically
# Run this in one terminal while developing

set -e

echo "🚀 Cortex Development Mode"
echo "=========================="
echo ""
echo "This will:"
echo "1. Watch for code changes"
echo "2. Rebuild automatically (debug mode - fast!)"
echo "3. Show you when it's ready to test"
echo ""
echo "📝 Instructions:"
echo "  - Keep this terminal open"
echo "  - When you see '✅ Ready to test!', run: ./target/debug/cortex"
echo "  - Press Ctrl+C to stop"
echo ""
echo "Installing cargo-watch if needed..."

# Install cargo-watch if not present
if ! command -v cargo-watch &> /dev/null; then
    cargo install cargo-watch
fi

echo ""
echo "Starting file watcher..."
echo "----------------------------------------"

# Watch and rebuild on changes (debug mode for fast builds)
cargo watch \
    --clear \
    --watch cortex-core \
    --watch cortex-tui \
    --watch cortex-cli \
    --watch cortex-plugins \
    --exec "build" \
    --shell "echo '✅ Ready to test! Run: ./target/debug/cortex'"