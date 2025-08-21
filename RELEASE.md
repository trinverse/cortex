# Release Process for Cortex

## Version 0.1.1 Release Checklist

### Pre-release Steps

1. **Update Version Numbers**
   - [x] Update workspace version in `/Cargo.toml` to `0.1.1`
   - [x] Update version in `/cortex-platform/Cargo.toml` to `0.1.1`
   - [x] Update version in `/cortex-updater/Cargo.toml` to `0.1.1`
   - [x] Update Homebrew formula version to `0.1.1`
   - [x] Update GitHub Actions default versions to `v0.1.1`

2. **Build and Test**
   ```bash
   # Build for current platform
   cargo build --release
   
   # Run tests
   cargo test
   
   # Test the binary
   ./target/release/cortex --version
   ```

3. **Create Git Tag**
   ```bash
   git add .
   git commit -m "Release version 0.1.1"
   git tag -a v0.1.1 -m "Release version 0.1.1"
   git push origin main
   git push origin v0.1.1
   ```

### Release Process

The release is automated via GitHub Actions. When you push a tag starting with `v`, it will:

1. Create a draft release
2. Build binaries for:
   - Linux x86_64
   - Linux ARM64
   - macOS x86_64 (Intel)
   - macOS ARM64 (Apple Silicon)
3. Calculate SHA256 checksums
4. Upload binaries to the release
5. Update the Homebrew formula with correct SHA256 values

### Manual Release (if needed)

If you need to trigger the release manually:

1. Go to Actions → Release workflow
2. Click "Run workflow"
3. Enter the tag (e.g., `v0.1.1`)
4. Click "Run workflow"

### Post-release Steps

1. **Verify Release Assets**
   - Check that all binaries are uploaded to the release
   - Verify SHA256 checksums match

2. **Test Homebrew Installation**
   ```bash
   # Test the Homebrew formula
   brew tap trinverse/cortex
   brew install cortex-fm-full
   cortex-fm --version
   ```

3. **Update Documentation**
   - Update README with new version if needed
   - Update changelog

### Platform-specific Binary Names

The release creates the following artifacts:
- `cortex-0.1.1-x86_64-linux.tar.gz` - Linux Intel/AMD
- `cortex-0.1.1-aarch64-linux.tar.gz` - Linux ARM64
- `cortex-v0.1.1-x86_64-apple-darwin.tar.gz` - macOS Intel
- `cortex-v0.1.1-aarch64-apple-darwin.tar.gz` - macOS Apple Silicon

### Homebrew Formula Updates

The Homebrew formula (`Formula/cortex-fm.rb`) will be automatically updated with:
- Correct SHA256 checksums for all binaries
- Updated URLs pointing to the new release
- Version bump to match the release tag

### Troubleshooting

If SHA256 values show as "PENDING_*" in the Homebrew formula:
1. Wait for the GitHub Actions to complete
2. The workflow will automatically calculate and update SHA256 values
3. If manual update is needed, download the release assets and calculate:
   ```bash
   sha256sum cortex-*.tar.gz
   ```