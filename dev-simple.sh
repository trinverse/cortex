#!/bin/bash

# Simple development mode without cargo-watch
# This uses basic file watching that works on all systems

set -e

echo "🚀 Cortex Simple Development Mode"
echo "================================="
echo ""
echo "This will rebuild and run Cortex when you press Enter"
echo "No external tools required!"
echo ""
echo "Workflow:"
echo "1. Make code changes"
echo "2. Press Enter here to rebuild & run"
echo "3. Press Ctrl+Q in Cortex to return here"
echo "4. Press Ctrl+C here to exit dev mode"
echo ""
echo "Building initial version..."

# Initial build
cargo build
echo "✅ Build complete!"
echo ""

# Main development loop
while true; do
    echo "----------------------------------------"
    echo "Ready! Press Enter to rebuild & run, or Ctrl+C to exit"
    read -r
    
    echo "🔨 Building..."
    if cargo build; then
        echo "✅ Build successful! Starting Cortex..."
        echo ""
        ./target/debug/cortex || true
        echo ""
        echo "Cortex exited."
    else
        echo "❌ Build failed! Fix errors and press Enter to retry."
    fi
done