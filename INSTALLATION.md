# Installation Guide for Cortex File Manager

## Homebrew Installation (Recommended for macOS)

Since we're using the main repository for Homebrew formulas, you can install directly:

```bash
# Tap the repository
brew tap trinverse/cortex https://github.com/trinverse/cortex

# Install cortex-fm
brew install trinverse/cortex/cortex-fm
```

## Direct Download

### macOS ARM64 (Apple Silicon)

```bash
# Download the binary
curl -L https://github.com/trinverse/cortex/releases/download/v0.1.1/cortex-v0.1.1-aarch64-apple-darwin.tar.gz -o cortex.tar.gz

# Extract
tar xzf cortex.tar.gz

# Move to your PATH
sudo mv cortex /usr/local/bin/cortex-fm

# Make executable
chmod +x /usr/local/bin/cortex-fm

# Verify installation
cortex-fm --version
```

### Build from Source

For other platforms or if you prefer building from source:

```bash
# Clone the repository
git clone https://github.com/trinverse/cortex.git
cd cortex

# Build with Cargo
cargo build --release

# The binary will be at ./target/release/cortex
./target/release/cortex --version
```

## Verification

After installation, verify it's working:

```bash
cortex-fm --version
# Should output: Cortex v0.1.1
```

## Troubleshooting

### Homebrew Issues

If you encounter SHA256 mismatch errors:

1. Clear Homebrew cache:
   ```bash
   rm -rf ~/Library/Caches/Homebrew/downloads/*cortex*
   ```

2. Untap and re-tap:
   ```bash
   brew untap trinverse/cortex
   brew tap trinverse/cortex https://github.com/trinverse/cortex
   brew install trinverse/cortex/cortex-fm
   ```

### Permission Issues

If you get permission denied errors:
```bash
chmod +x /usr/local/bin/cortex-fm
```

## Notes

- The binary is installed as `cortex-fm` to avoid conflicts with other packages
- Currently, pre-built binaries are only available for macOS ARM64
- Other platforms will build from source when installed via Homebrew