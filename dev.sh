#!/bin/bash

# SIMPLEST DEVELOPMENT MODE - One command to rule them all!
# This watches, builds, and runs Cortex automatically

set -e

echo "🚀 Cortex Quick Development Mode"
echo "================================"
echo ""
echo "Making changes? This will:"
echo "✓ Watch for file changes"
echo "✓ Rebuild automatically (debug - fast!)"  
echo "✓ Run Cortex immediately after build"
echo ""
echo "Just press Ctrl+Q in Cortex to test new changes!"
echo "Press Ctrl+C here to stop everything"
echo ""

# Install cargo-watch if needed
if ! command -v cargo-watch &> /dev/null; then
    echo "Installing cargo-watch..."
    cargo install cargo-watch
fi

# Watch, build, and run!
cargo watch \
    --clear \
    --watch cortex-core \
    --watch cortex-tui \
    --watch cortex-cli \
    --watch cortex-plugins \
    --exec "build" \
    --exec "run --bin cortex"