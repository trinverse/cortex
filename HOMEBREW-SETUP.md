# 🍺 Homebrew Setup for Cortex

## Prerequisites

1. **Create a Homebrew Tap Repository**
   - Go to https://github.com/new
   - Repository name: `homebrew-cortex`
   - Description: "Homebrew tap for Cortex file manager"
   - Make it public
   - Create repository

2. **Set up the tap structure**:
   ```bash
   git clone https://github.com/trinverse/homebrew-cortex.git
   cd homebrew-cortex
   mkdir Formula
   # Copy the formula file we created
   cp /path/to/cortex/homebrew/cortex.rb Formula/
   git add .
   git commit -m "Initial tap setup"
   git push
   ```

## Release Process

### Option 1: Automated Release (Recommended)

1. **Create a git tag and push**:
   ```bash
   git tag v0.1.0
   git push origin v0.1.0
   ```

2. **The GitHub Action will automatically**:
   - Build binaries for macOS (Intel & Apple Silicon) and Linux
   - Create a GitHub release with all artifacts
   - Update your Homebrew tap with correct SHA256 values

### Option 2: Manual Release

1. **Build binaries locally** (on macOS):
   ```bash
   # For Apple Silicon Macs
   cargo build --release --target aarch64-apple-darwin
   
   # For Intel Macs
   cargo build --release --target x86_64-apple-darwin
   
   # Create tarballs
   VERSION=0.1.0
   mkdir -p dist
   cp target/release/cortex dist/
   cd dist
   tar czf cortex-${VERSION}-macos-$(uname -m).tar.gz cortex
   shasum -a 256 cortex-${VERSION}-macos-$(uname -m).tar.gz
   ```

2. **Create GitHub Release**:
   - Go to https://github.com/trinverse/cortex/releases/new
   - Tag: v0.1.0
   - Upload the .tar.gz files
   - Publish release

3. **Update Homebrew formula**:
   - Get SHA256 values from the uploaded files
   - Update Formula/cortex.rb in your tap repository
   - Commit and push

## Installation Instructions for Users

Once your tap is set up, users can install Cortex with:

```bash
# Add your tap
brew tap trinverse/cortex

# Install Cortex
brew install cortex

# Or in one command
brew install trinverse/cortex/cortex
```

## Testing Your Formula

Before releasing, test locally:

```bash
# Test the formula
brew install --build-from-source Formula/cortex.rb

# Audit the formula
brew audit --strict Formula/cortex.rb

# Test installation
cortex --version
```

## Updating Cortex

When you release a new version:

1. Tag and push:
   ```bash
   git tag v0.2.0
   git push origin v0.2.0
   ```

2. The GitHub Action updates everything automatically

3. Users update with:
   ```bash
   brew update
   brew upgrade cortex
   ```

## Alternative: Submit to Homebrew Core

Once Cortex is popular (300+ GitHub stars, established project), you can submit to homebrew-core:

1. Fork https://github.com/Homebrew/homebrew-core
2. Add your formula to Formula/cortex.rb
3. Create a pull request
4. Follow Homebrew's review process

## Troubleshooting

### "SHA256 mismatch" error
- Ensure you're using the correct SHA256 from the release artifacts
- Use `shasum -a 256 file.tar.gz` on macOS
- Use `sha256sum file.tar.gz` on Linux

### "Formula not found" error
- Ensure the tap repository is public
- Check Formula directory exists with cortex.rb inside
- Run `brew tap trinverse/cortex` first

### Cross-compilation issues
- Use GitHub Actions for building (handles cross-compilation)
- Or use `cross` tool for local cross-compilation

## Quick Start Commands

```bash
# 1. Create tap repository on GitHub (manual step)

# 2. Clone and set up
git clone https://github.com/trinverse/homebrew-cortex.git
cd homebrew-cortex
mkdir Formula
echo "# Homebrew Tap for Cortex" > README.md
git add .
git commit -m "Initial commit"
git push

# 3. Back to cortex repo
cd /path/to/cortex

# 4. Create and push tag
git tag v0.1.0
git push origin v0.1.0

# 5. Wait for GitHub Actions to complete

# 6. Test installation
brew tap trinverse/cortex
brew install cortex
cortex --version
```

That's it! Your Homebrew distribution is ready!