# Quick Release Instructions for v0.1.0

## 🚀 Release Steps (5 minutes)

### 1. Run the Release Script
```bash
./release-v0.1.0-quick.sh
```

This will:
- ✅ Build the Debian package (`cortex_0.1.0_amd64.deb`)
- ✅ Create Linux tarball (`cortex-0.1.0-x86_64-linux.tar.gz`)
- ✅ Commit all changes
- ✅ Create v0.1.0 tag
- ✅ Push to GitHub (https://github.com/trinverse/cortex)

### 2. Create GitHub Release
1. Go to: https://github.com/trinverse/cortex/releases/new
2. Select tag: `v0.1.0`
3. Title: `Cortex v0.1.0 - Initial Release`
4. Upload files:
   - `cortex_0.1.0_amd64.deb`
   - `cortex-0.1.0-x86_64-linux.tar.gz`
5. Add description:
```markdown
## 🎉 Initial Release

First public release of Cortex File Manager - a modern, powerful terminal file manager built with Rust.

### ✨ Features
- Dual-pane interface for efficient navigation
- Full keyboard control with vim-style bindings
- Plugin system with Lua support
- File preview and editing
- Archive support (ZIP, TAR, 7Z)
- Trash/clipboard integration
- Auto-update system
- Cross-platform support

### 📦 Installation

#### Ubuntu/Debian
```bash
wget https://github.com/trinverse/cortex/releases/download/v0.1.0/cortex_0.1.0_amd64.deb
sudo dpkg -i cortex_0.1.0_amd64.deb
```

#### Other Linux
```bash
wget https://github.com/trinverse/cortex/releases/download/v0.1.0/cortex-0.1.0-x86_64-linux.tar.gz
tar -xzf cortex-0.1.0-x86_64-linux.tar.gz
./cortex
```

### 🐛 Known Issues
- This is an alpha release for testing
- SSH/SFTP temporarily disabled
- Some features may need refinement

Please report issues at: https://github.com/trinverse/cortex/issues
```
6. Check "☑ This is a pre-release"
7. Click "Publish release"

## 🧪 Test Installation

### Ubuntu/Debian
```bash
# Install
sudo dpkg -i cortex_0.1.0_amd64.deb

# Run
cortex

# Test version
cortex --version
```

## 🍺 Homebrew Setup (Optional)

### Create Tap Repository
1. Create repo: https://github.com/trinverse/homebrew-cortex
2. Create `Formula` directory
3. Copy `packaging/homebrew/cortex.rb` to `Formula/cortex.rb`
4. Update SHA256 after release
5. Commit and push

### Users Install Via
```bash
brew tap trinverse/cortex
brew install cortex
```

## 📊 What Happens After Release

GitHub Actions will automatically:
- Build for all platforms (Windows, macOS, Linux)
- Run tests
- Create additional release assets

## ✅ Done!

Your release will be available at:
https://github.com/trinverse/cortex/releases/tag/v0.1.0

Users can start installing and testing immediately!