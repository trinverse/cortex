#!/bin/bash

# Development script with auto-rebuild using cargo-watch
# Install cargo-watch if not present: cargo install cargo-watch

set -e

echo "🔄 Starting Cortex development mode with auto-rebuild..."
echo ""
echo "This will watch for file changes and rebuild automatically."
echo "The app will restart when you press Enter after seeing 'Finished'."
echo ""
echo "Install cargo-watch if not installed:"
echo "  cargo install cargo-watch"
echo ""
echo "Press Ctrl+C to stop."
echo "----------------------------------------"

# Check if cargo-watch is installed
if ! command -v cargo-watch &> /dev/null; then
    echo "❌ cargo-watch not found. Installing..."
    cargo install cargo-watch
fi

# Watch and rebuild on changes
cargo watch -x "build --release" -s "echo '✅ Build complete! Run: ./target/release/cortex'"