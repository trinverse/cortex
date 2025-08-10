# Packaging Checklist for Cortex

## Pre-Release Checklist

### Code Preparation
- [ ] Remove all debug statements
- [ ] Update version in Cargo.toml files
- [ ] Update CHANGELOG.md
- [ ] Run full test suite
- [ ] Run clippy and fix all warnings
- [ ] Build in release mode
- [ ] Test on target platforms

### Assets
- [ ] Application icon (multiple sizes)
  - [ ] 16x16, 32x32, 64x64, 128x128, 256x256, 512x512
  - [ ] .ico for Windows
  - [ ] .icns for macOS
  - [ ] .png for Linux
- [ ] License file
- [ ] README for distribution
- [ ] Desktop entry file (Linux)

## macOS Packaging

### DMG Creation
```bash
# Install create-dmg
brew install create-dmg

# Build release binary
cargo build --release

# Create app bundle structure
mkdir -p Cortex.app/Contents/MacOS
mkdir -p Cortex.app/Contents/Resources
cp target/release/cortex Cortex.app/Contents/MacOS/
cp assets/icon.icns Cortex.app/Contents/Resources/

# Create Info.plist
cat > Cortex.app/Contents/Info.plist << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>cortex</string>
    <key>CFBundleIdentifier</key>
    <string>com.trinverse.cortex</string>
    <key>CFBundleName</key>
    <string>Cortex</string>
    <key>CFBundleShortVersionString</key>
    <string>0.1.0</string>
    <key>CFBundleVersion</key>
    <string>1</string>
    <key>CFBundleIconFile</key>
    <string>icon</string>
    <key>LSMinimumSystemVersion</key>
    <string>10.15</string>
    <key>NSHighResolutionCapable</key>
    <true/>
</dict>
</plist>
EOF

# Create DMG
create-dmg \
  --volname "Cortex Installer" \
  --volicon "assets/icon.icns" \
  --window-pos 200 120 \
  --window-size 600 400 \
  --icon-size 100 \
  --icon "Cortex.app" 175 120 \
  --hide-extension "Cortex.app" \
  --app-drop-link 425 120 \
  "Cortex-0.1.0.dmg" \
  "Cortex.app"
```

### Homebrew Formula
```ruby
class Cortex < Formula
  desc "Modern orthodox file manager"
  homepage "https://github.com/trinverse/cortex"
  url "https://github.com/trinverse/cortex/archive/v0.1.0.tar.gz"
  sha256 "HASH_HERE"
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    assert_match "Cortex", shell_output("#{bin}/cortex --version")
  end
end
```

## Windows Packaging

### MSI with WiX
```xml
<?xml version="1.0" encoding="UTF-8"?>
<Wix xmlns="http://schemas.microsoft.com/wix/2006/wi">
  <Product Id="*" Name="Cortex" Language="1033" Version="0.1.0.0" 
           Manufacturer="Trinverse" UpgradeCode="PUT-GUID-HERE">
    <Package InstallerVersion="200" Compressed="yes" InstallScope="perMachine" />
    
    <MajorUpgrade DowngradeErrorMessage="A newer version is already installed." />
    <MediaTemplate EmbedCab="yes" />
    
    <Feature Id="ProductFeature" Title="Cortex" Level="1">
      <ComponentGroupRef Id="ProductComponents" />
    </Feature>
    
    <Directory Id="TARGETDIR" Name="SourceDir">
      <Directory Id="ProgramFilesFolder">
        <Directory Id="INSTALLFOLDER" Name="Cortex" />
      </Directory>
    </Directory>
    
    <ComponentGroup Id="ProductComponents" Directory="INSTALLFOLDER">
      <Component Id="ProductComponent">
        <File Source="target/release/cortex.exe" />
      </Component>
    </ComponentGroup>
  </Product>
</Wix>
```

### Build Commands
```powershell
# Build release
cargo build --release

# Create MSI
candle cortex.wxs
light -ext WixUIExtension cortex.wixobj -o Cortex-0.1.0.msi
```

## Linux Packaging

### AppImage
```bash
# Install linuxdeploy
wget https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-x86_64.AppImage
chmod +x linuxdeploy-x86_64.AppImage

# Create AppDir structure
mkdir -p AppDir/usr/bin
mkdir -p AppDir/usr/share/applications
mkdir -p AppDir/usr/share/icons/hicolor/256x256/apps

# Copy files
cp target/release/cortex AppDir/usr/bin/
cp assets/icon.png AppDir/usr/share/icons/hicolor/256x256/apps/cortex.png

# Create desktop entry
cat > AppDir/usr/share/applications/cortex.desktop << EOF
[Desktop Entry]
Type=Application
Name=Cortex
Comment=Modern orthodox file manager
Exec=cortex
Icon=cortex
Terminal=true
Categories=Utility;FileManager;
EOF

# Create AppImage
./linuxdeploy-x86_64.AppImage --appdir AppDir --output appimage
```

### Flatpak Manifest
```yaml
app-id: com.trinverse.cortex
runtime: org.freedesktop.Platform
runtime-version: '23.08'
sdk: org.freedesktop.Sdk
sdk-extensions:
  - org.freedesktop.Sdk.Extension.rust-stable
command: cortex

finish-args:
  - --filesystem=home
  - --share=network
  - --socket=wayland
  - --socket=fallback-x11
  - --device=dri

modules:
  - name: cortex
    buildsystem: simple
    build-commands:
      - cargo build --release
      - install -Dm755 target/release/cortex /app/bin/cortex
    sources:
      - type: git
        url: https://github.com/trinverse/cortex.git
        tag: v0.1.0
```

## GitHub Actions Workflow

```yaml
name: Package Release

on:
  push:
    tags:
      - 'v*'

jobs:
  build-macos:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: cargo build --release
      - run: ./scripts/package-macos.sh
      - uses: actions/upload-artifact@v2
        with:
          name: macos-dmg
          path: Cortex-*.dmg

  build-windows:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: cargo build --release
      - run: ./scripts/package-windows.ps1
      - uses: actions/upload-artifact@v2
        with:
          name: windows-msi
          path: Cortex-*.msi

  build-linux:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: cargo build --release
      - run: ./scripts/package-linux.sh
      - uses: actions/upload-artifact@v2
        with:
          name: linux-appimage
          path: Cortex-*.AppImage

  create-release:
    needs: [build-macos, build-windows, build-linux]
    runs-on: ubuntu-latest
    steps:
      - uses: actions/download-artifact@v2
      - uses: softprops/action-gh-release@v1
        with:
          files: |
            macos-dmg/*.dmg
            windows-msi/*.msi
            linux-appimage/*.AppImage
          draft: false
          prerelease: false
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

## Post-Release

- [ ] Upload to package repositories
  - [ ] Homebrew tap
  - [ ] AUR (Arch Linux)
  - [ ] Flathub
  - [ ] Snapcraft
- [ ] Update website/documentation
- [ ] Announce release
  - [ ] GitHub release notes
  - [ ] Social media
  - [ ] Reddit (r/rust, r/commandline)
- [ ] Monitor for issues
- [ ] Plan next release