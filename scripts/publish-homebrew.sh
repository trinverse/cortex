#!/bin/bash
set -e

echo "Setting up Homebrew tap for Cortex..."

# Check if tap repository exists
if ! gh repo view trinverse/homebrew-cortex &>/dev/null; then
    echo "Creating homebrew-cortex repository..."
    gh repo create trinverse/homebrew-cortex --public --description "Homebrew tap for Cortex terminal file manager"
else
    echo "Tap repository already exists"
fi

# Clone or update the tap repository
TEMP_DIR=$(mktemp -d)
cd "$TEMP_DIR"

echo "Cloning tap repository..."
git clone git@github.com:trinverse/homebrew-cortex.git
cd homebrew-cortex

# Copy the formula
echo "Copying formula..."
mkdir -p Formula
cp /home/ashish/code/cortex/homebrew-formula/cortex.rb Formula/

# Commit and push
git add Formula/cortex.rb
git commit -m "Update Cortex formula to v0.1.0" || echo "No changes to commit"
git push origin main

echo "✅ Homebrew tap published!"
echo ""
echo "Users can now install Cortex with:"
echo "  brew tap trinverse/cortex"
echo "  brew install cortex"

# Cleanup
cd /
rm -rf "$TEMP_DIR"