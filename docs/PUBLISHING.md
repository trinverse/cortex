# Cortex Publishing Guide

This guide covers how to publish Cortex to various platforms and package managers.

## Overview

Cortex can be distributed through multiple channels:

| Platform | Package Type | Automation | Status |
|----------|-------------|------------|--------|
| GitHub Releases | Binary/Source | ✅ Fully Automated | Ready |
| Chocolatey (Windows) | NuGet | ✅ Fully Automated | Ready |
| Homebrew (macOS/Linux) | Formula | ✅ Fully Automated | Ready |
| Snap Store (Linux) | Snap | ✅ Fully Automated | Ready |
| AUR (Arch Linux) | PKGBUILD | ✅ Fully Automated | Ready |
| APT (Debian/Ubuntu) | .deb | ✅ Fully Automated | Ready |
| Mac App Store | .pkg | ⚠️ Semi-Automated | Ready |
| Microsoft Store | MSIX | ❌ Manual | Planned |

## Release Process

### 1. Prepare Release

```bash
# Update version in Cargo.toml
vim Cargo.toml  # Update version = "X.Y.Z"

# Update changelog
vim CHANGELOG.md

# Commit changes
git add -A
git commit -m "Release v$VERSION"

# Create tag
git tag -a v$VERSION -m "Release v$VERSION"

# Push changes
git push origin main --tags
```

### 2. Automated Publishing

When you push a tag starting with `v`, GitHub Actions will automatically:

1. **Build binaries** for all platforms
2. **Create GitHub Release** with all assets
3. **Publish to package managers**:
   - Chocolatey (Windows)
   - Homebrew (macOS/Linux)
   - Snap Store (Linux)
   - AUR (Arch Linux)

### 3. Manual Publishing (if needed)

#### Trigger Individual Workflows

```bash
# Via GitHub CLI
gh workflow run publish-chocolatey.yml -f version=1.0.0
gh workflow run publish-homebrew.yml -f version=1.0.0
gh workflow run publish-snap.yml -f version=1.0.0
gh workflow run publish-aur.yml -f version=1.0.0
```

## Platform-Specific Instructions

### Windows - Chocolatey

**Prerequisites:**
- Chocolatey account
- API key in GitHub Secrets

**Process:**
1. Automated via GitHub Actions
2. Package available within 30 minutes after approval
3. Users install with: `choco install cortex`

**Manual submission:**
```powershell
choco pack cortex.nuspec
choco push cortex.1.0.0.nupkg --source https://push.chocolatey.org/
```

### macOS - Homebrew

**Prerequisites:**
- GitHub repository `homebrew-cortex`
- PAT with repo access

**Process:**
1. Automated via GitHub Actions
2. Creates PR to tap repository
3. Users install with: `brew install cortex`

**Manual update:**
```bash
# Update formula
brew bump-formula-pr --url=https://github.com/org/cortex/archive/v1.0.0.tar.gz cortex
```

### Linux - Snap Store

**Prerequisites:**
- Snapcraft account
- Store credentials exported

**Process:**
1. Automated via GitHub Actions
2. Published to stable channel
3. Users install with: `snap install cortex`

**Manual publishing:**
```bash
snapcraft
snapcraft upload cortex_1.0.0_amd64.snap --release=stable
```

### Linux - AUR

**Prerequisites:**
- AUR account
- SSH key configured

**Process:**
1. Automated via GitHub Actions
2. Updates within minutes
3. Users install with: `yay -S cortex`

**Manual update:**
```bash
git clone ssh://aur@aur.archlinux.org/cortex.git
# Update PKGBUILD
makepkg --printsrcinfo > .SRCINFO
git add PKGBUILD .SRCINFO
git commit -m "Update to 1.0.0"
git push
```

### macOS - Mac App Store

**Prerequisites:**
- Apple Developer Program membership ($99/year)
- Certificates configured
- App Store Connect access

**Process:**
1. Run workflow to prepare package
2. Upload via Transporter app
3. Submit for review in App Store Connect

**Steps:**
```bash
# 1. Build and notarize
gh workflow run prepare-mac-app-store.yml -f version=1.0.0

# 2. Download package
gh run download <run-id>

# 3. Upload with Transporter
# Open Transporter.app
# Sign in with Apple ID
# Add package and deliver

# 4. Submit in App Store Connect
# - Add release notes
# - Submit for review
```

## Version Management

### Semantic Versioning

Cortex follows semantic versioning:
- **Major** (X.0.0): Breaking changes
- **Minor** (0.X.0): New features
- **Patch** (0.0.X): Bug fixes

### Version Locations

Update version in these files:
1. `Cargo.toml` (workspace version)
2. `CHANGELOG.md`
3. `README.md` (badges)
4. `docs/ROADMAP.md` (current version)

## Testing Releases

### Pre-release Testing

```bash
# Create pre-release
git tag -a v1.0.0-beta.1 -m "Pre-release v1.0.0-beta.1"
git push origin v1.0.0-beta.1

# Test installation from each platform
```

### Platform-specific Testing

#### Windows
```powershell
# Test MSI
msiexec /i cortex-1.0.0-x64.msi /quiet

# Test Chocolatey
choco install cortex --version=1.0.0-beta
```

#### macOS
```bash
# Test DMG
hdiutil attach cortex-1.0.0.dmg
cp /Volumes/Cortex/Cortex /Applications/

# Test Homebrew
brew install --HEAD cortex
```

#### Linux
```bash
# Test deb
sudo dpkg -i cortex_1.0.0_amd64.deb

# Test snap
snap install cortex --channel=beta

# Test AUR
yay -S cortex-git
```

## Rollback Procedure

If issues are discovered after release:

### 1. GitHub Release
```bash
# Delete release and tag
gh release delete v1.0.0 --yes
git push --delete origin v1.0.0
```

### 2. Package Managers

#### Chocolatey
- Unlist package version in Chocolatey dashboard
- Push fixed version

#### Homebrew
- Revert PR in tap repository
- Create new PR with fix

#### Snap Store
- Close channel or revert to previous version
```bash
snapcraft close cortex stable
snapcraft release cortex <previous-revision> stable
```

#### AUR
- Quick push with increased pkgrel
```bash
# Increment pkgrel in PKGBUILD
pkgrel=2
```

## Monitoring

### Release Health

Monitor these metrics:
- Download counts (GitHub API)
- Package manager statistics
- User feedback (issues/discussions)
- Crash reports (if telemetry enabled)

### Automation Status

Check workflow runs:
```bash
# List recent workflow runs
gh run list

# View specific run
gh run view <run-id>

# Check for failures
gh run list --status=failure
```

## Best Practices

1. **Always test locally first**
   ```bash
   cargo test --all
   cargo build --release
   ```

2. **Create release branch**
   ```bash
   git checkout -b release/v1.0.0
   ```

3. **Use release candidates**
   - Test with small group first
   - Gather feedback
   - Fix issues before wide release

4. **Document changes**
   - Update CHANGELOG.md
   - Create release notes
   - Update documentation

5. **Communicate**
   - Announce in discussions
   - Post on social media
   - Update website

## Troubleshooting

### Common Issues

#### Build Failures
- Check dependency versions
- Verify Rust toolchain
- Review CI logs

#### Publishing Failures
- Verify credentials are valid
- Check API rate limits
- Ensure package metadata is correct

#### Notarization Issues (macOS)
- Verify Developer ID is valid
- Check entitlements
- Review Apple's guidelines

#### Store Rejections
- Review guidelines compliance
- Check metadata requirements
- Verify screenshots/descriptions

## Support

For publishing issues:
- GitHub Actions: Check workflow logs
- Package managers: Check respective documentation
- Platform stores: Contact support teams

## Resources

- [GitHub Releases API](https://docs.github.com/en/rest/releases)
- [Chocolatey Publishing](https://docs.chocolatey.org/en-us/create/create-packages)
- [Homebrew Formula](https://docs.brew.sh/Formula-Cookbook)
- [Snapcraft Documentation](https://snapcraft.io/docs)
- [AUR Submission](https://wiki.archlinux.org/title/AUR_submission_guidelines)
- [Apple Developer](https://developer.apple.com/documentation/)

---

*Last updated: January 2025*