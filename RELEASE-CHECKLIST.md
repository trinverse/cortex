# Release Checklist for Cortex v0.1.0

## 📋 What You Need From Your Side

### 1. **GitHub Account Setup**
- [ ] GitHub account created
- [ ] Git configured locally:
  ```bash
  git config --global user.name "Your Name"
  git config --global user.email "your.email@example.com"
  ```

### 2. **Create GitHub Repository**
- [ ] Go to https://github.com/new
- [ ] Repository name: `cortex`
- [ ] Make it **Public**
- [ ] **DON'T** initialize with README (we already have one)
- [ ] Click "Create repository"

### 3. **Push Code to GitHub**
Run the release script:
```bash
./release-v0.1.0.sh
```

Or manually:
```bash
# Set your git identity
git config user.name "Your Name"
git config user.email "your.email@example.com"

# Add GitHub remote (replace YOUR_USERNAME)
git remote add origin https://github.com/YOUR_USERNAME/cortex.git

# Commit all changes
git add -A
git commit -m "Release v0.1.0 - Initial public release"

# Create tag
git tag -a v0.1.0 -m "Release v0.1.0"

# Push everything
git push -u origin main --tags
```

### 4. **Create GitHub Release**
1. Go to `https://github.com/YOUR_USERNAME/cortex/releases`
2. Click "Create a new release"
3. Choose tag: `v0.1.0`
4. Title: `Cortex v0.1.0 - Initial Release`
5. Copy release notes from `CHANGELOG.md`
6. Check "This is a pre-release" (since it's alpha)
7. Click "Publish release"

## 🍺 Homebrew Setup (macOS/Linux)

### Option 1: Personal Tap (Easiest)
1. **Create tap repository**:
   - Go to https://github.com/new
   - Name: `homebrew-cortex`
   - Make it public
   - Initialize with README

2. **Add formula**:
   ```bash
   # Clone your tap
   git clone https://github.com/YOUR_USERNAME/homebrew-cortex.git
   cd homebrew-cortex
   mkdir -p Formula
   
   # Copy the formula (update YOUR_USERNAME in the file first)
   cp /path/to/cortex/packaging/homebrew/cortex.rb Formula/
   
   # Update the formula with your GitHub username
   sed -i 's/YOUR_USERNAME/your-actual-username/g' Formula/cortex.rb
   
   # Commit and push
   git add Formula/cortex.rb
   git commit -m "Add Cortex formula"
   git push
   ```

3. **Install via Homebrew**:
   ```bash
   # Add your tap
   brew tap YOUR_USERNAME/cortex
   
   # Install Cortex
   brew install cortex
   ```

### Option 2: Homebrew Core (Later)
- For official Homebrew, need more testing and popularity
- Submit PR to homebrew-core after stable release

## 🐧 Ubuntu/Debian Setup

### Quick Local Install
The release script already built a .deb package:
```bash
# Install the package
sudo dpkg -i cortex_0.1.0_amd64.deb

# Test it
cortex --version
cortex
```

### PPA Setup (Optional - More Complex)
1. **Create Launchpad Account**:
   - Sign up at https://launchpad.net
   - Add your GPG key
   - Add your SSH key

2. **Create PPA**:
   - Go to https://launchpad.net/~YOUR_USERNAME
   - Click "Create a new PPA"
   - Name: `cortex`
   - Display name: `Cortex File Manager`

3. **Build and Upload**:
   ```bash
   # Install build tools
   sudo apt-get install devscripts dput debhelper
   
   # Build source package
   cd cortex
   debuild -S -sa
   
   # Upload to PPA
   dput ppa:YOUR_USERNAME/cortex ../cortex_0.1.0_source.changes
   ```

4. **Users Install via**:
   ```bash
   sudo add-apt-repository ppa:YOUR_USERNAME/cortex
   sudo apt update
   sudo apt install cortex
   ```

## 🚀 Quick Testing Commands

After installation, test with:
```bash
# Check version
cortex --version

# Run the file manager
cortex

# Check for updates (once GitHub release is live)
cortex --check-updates

# View help
cortex --help
```

## 📝 Update Documentation

After release, update these files with your actual GitHub username:
1. `README.md` - Installation instructions
2. `packaging/homebrew/cortex.rb` - Homebrew formula URLs
3. `.github/workflows/*.yml` - If using different repository name

## ⚡ Quick Start for Testing

### On Ubuntu/Debian:
```bash
# Using the .deb package
sudo dpkg -i cortex_0.1.0_amd64.deb
cortex
```

### On macOS/Linux with Homebrew:
```bash
# After setting up your tap
brew tap YOUR_USERNAME/cortex
brew install cortex
cortex
```

### From Source (Any Platform):
```bash
# Clone and build
git clone https://github.com/YOUR_USERNAME/cortex.git
cd cortex
cargo build --release
./target/release/cortex
```

## 🎯 What Happens Next

Once you complete the above steps:

1. **GitHub Actions** will automatically:
   - Build binaries for all platforms
   - Run tests
   - Create release assets

2. **Users can install** via:
   - Direct download from GitHub Releases
   - Homebrew (your tap)
   - Debian package
   - Build from source

3. **You can test** the full workflow:
   - Installation process
   - Auto-updates
   - All features

## 📧 Need Help?

If you encounter issues:
1. Check the build logs in GitHub Actions
2. Test locally first with `cargo run`
3. Start with the .deb package for Ubuntu
4. Homebrew tap is easiest for macOS

## 🎉 Congratulations!

Once completed, you'll have:
- ✅ Public GitHub repository
- ✅ Tagged release v0.1.0
- ✅ Debian package for Ubuntu
- ✅ Homebrew formula for macOS/Linux
- ✅ Binary releases for all platforms
- ✅ Auto-update system ready

Happy testing! 🚀