# Ubuntu Version Support

## Supported Ubuntu Versions

Cortex is built for the following Ubuntu releases:

### LTS (Long Term Support) Releases
- **Ubuntu 20.04 LTS** (Focal Fossa) - `focal`
  - Released: April 2020
  - End of Life: April 2030
  - Status: ✅ Fully supported
  
- **Ubuntu 22.04 LTS** (Jammy Jellyfish) - `jammy`
  - Released: April 2022
  - End of Life: April 2032
  - Status: ✅ Fully supported
  
- **Ubuntu 24.04 LTS** (Noble Numbat) - `noble`
  - Released: April 2024
  - End of Life: April 2034
  - Status: ✅ Fully supported

### Current Releases
- **Ubuntu 24.10** (Oracular Oriole) - `oracular`
  - Released: October 2024
  - End of Life: July 2025
  - Status: ✅ Fully supported

### Development Version
- **Ubuntu 25.04** (Plucky Puffin) - `plucky`
  - Release Date: April 2025 (planned)
  - Status: ⚠️ Development version, may be unstable

## Installation Instructions

### For Stable Releases (Recommended)

```bash
# Add the PPA
sudo add-apt-repository ppa:ashishtyagi10/cortex
sudo apt update

# Install Cortex
sudo apt install cortex
```

### Version-Specific Installation

If you need to install a specific version or the PPA isn't available for your Ubuntu version:

#### Ubuntu 24.10 (Oracular Oriole)
```bash
# Latest Ubuntu release
sudo add-apt-repository ppa:ashishtyagi10/cortex
sudo apt update
sudo apt install cortex
```

#### Ubuntu 25.04 (Plucky Puffin) - Development
```bash
# Note: This is a development version of Ubuntu
# Packages may not be immediately available
sudo add-apt-repository ppa:ashishtyagi10/cortex
sudo apt update
sudo apt install cortex
```

## Building from Source

If a package isn't available for your Ubuntu version:

```bash
# Install build dependencies
sudo apt update
sudo apt install -y cargo rustc pkg-config libssl-dev

# Clone and build
git clone https://github.com/trinverse/cortex.git
cd cortex
cargo build --release

# Install
sudo install -Dm755 target/release/cortex /usr/local/bin/cortex
```

## Package Availability

| Ubuntu Version | Codename | Package Status | Notes |
|---------------|----------|----------------|-------|
| 20.04 LTS | focal | ✅ Available | Stable |
| 22.04 LTS | jammy | ✅ Available | Stable |
| 24.04 LTS | noble | ✅ Available | Stable |
| 24.10 | oracular | ✅ Available | Current release |
| 25.04 | plucky | 🔄 Building | Development version |

## Checking Your Ubuntu Version

```bash
# Check your Ubuntu version
lsb_release -a

# Or
cat /etc/os-release
```

## Troubleshooting

### "Package not found" error
If you get this error for newer Ubuntu versions:
1. The package might still be building (check https://launchpad.net/~ashishtyagi10/+archive/ubuntu/cortex)
2. Try installing from the GitHub release page
3. Build from source as shown above

### Development Version Warning
Ubuntu 25.04 (Plucky Puffin) is currently in development. Packages built for it may:
- Have unexpected issues
- Require newer dependencies
- Change frequently

For production use, we recommend sticking to LTS releases (20.04, 22.04, or 24.04).

## Release Schedule

- **Stable releases**: Built for all supported Ubuntu versions
- **Beta releases**: Built for current and LTS versions
- **Development builds**: May be limited to recent versions

## Support

For version-specific issues:
- GitHub Issues: https://github.com/trinverse/cortex/issues
- PPA Page: https://launchpad.net/~ashishtyagi10/+archive/ubuntu/cortex