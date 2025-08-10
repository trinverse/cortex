# Publishing Cortex to Ubuntu PPA (APT Repository)

This guide will help you publish Cortex to Ubuntu's Personal Package Archive (PPA) so users can install it with `sudo apt install cortex`.

## Prerequisites

### 1. Create Launchpad Account
1. Go to https://launchpad.net
2. Sign up with your Ubuntu One account
3. Verify your email

### 2. Create Your PPA
1. Go to your Launchpad profile: https://launchpad.net/~YOUR_USERNAME
2. Click "Create a new PPA"
3. Fill in:
   - URL: `cortex` (will become ppa:YOUR_USERNAME/cortex)
   - Display name: `Cortex File Manager`
   - Description: `Modern terminal file manager built with Rust`

## Step-by-Step Setup

### Step 1: Install Required Tools

```bash
# Install packaging tools
sudo apt-get update
sudo apt-get install -y \
    devscripts \
    debhelper \
    dput \
    gnupg \
    lintian \
    build-essential \
    cargo \
    rustc

# Install additional tools for Rust packages
sudo apt-get install -y \
    dh-cargo \
    cargo-lock
```

### Step 2: Generate GPG Key

```bash
# Generate a new GPG key (if you don't have one)
gpg --full-generate-key

# Choose:
# - (1) RSA and RSA
# - 4096 bits
# - Key does not expire (0)
# - Your real name
# - Your email (same as Launchpad account)

# List your keys
gpg --list-secret-keys --keyid-format=long

# Export your public key (replace KEY_ID with your key ID)
gpg --armor --export KEY_ID > public_key.asc

# Upload to Ubuntu keyserver
gpg --keyserver keyserver.ubuntu.com --send-keys KEY_ID
```

### Step 3: Add GPG Key to Launchpad
1. Go to https://launchpad.net/~YOUR_USERNAME/+editpgpkeys
2. Paste your public key or use the fingerprint
3. Launchpad will send you an encrypted email
4. Decrypt it: `gpg --decrypt launchpad-email.txt`
5. Follow the link to confirm

### Step 4: Configure SSH Key
```bash
# Generate SSH key if needed
ssh-keygen -t rsa -b 4096 -C "your.email@example.com"

# Add to Launchpad
cat ~/.ssh/id_rsa.pub
```
Go to https://launchpad.net/~YOUR_USERNAME/+editsshkeys and add the key

## Building and Uploading Package

### Option 1: Using Our Script (Recommended)

Run the automated script:
```bash
./packaging/debian/build-ppa-package.sh
```

### Option 2: Manual Process

#### 1. Prepare Source Directory
```bash
# Create working directory
mkdir -p ~/ppa-build
cd ~/ppa-build

# Export source (without .git)
cp -r /path/to/cortex cortex-0.1.0
cd cortex-0.1.0
rm -rf .git

# Create orig tarball
cd ..
tar czf cortex_0.1.0.orig.tar.gz cortex-0.1.0
cd cortex-0.1.0
```

#### 2. Create/Update Debian Files
```bash
# Initialize debian directory if needed
dh_make --native --single --packagename cortex_0.1.0 --email your.email@example.com

# Or use existing debian directory
cp -r /path/to/cortex/packaging/debian debian/
```

#### 3. Update Changelog
```bash
# Update changelog for PPA
dch -v 0.1.0-1ubuntu1 -D focal "Initial release for Ubuntu 20.04"
dch -r
```

#### 4. Build Source Package
```bash
# Build source package only (no binary)
debuild -S -sa -k<YOUR_GPG_KEY_ID>

# This creates:
# ../cortex_0.1.0-1ubuntu1.dsc
# ../cortex_0.1.0-1ubuntu1_source.changes
# ../cortex_0.1.0-1ubuntu1.debian.tar.xz
```

#### 5. Upload to PPA
```bash
# Upload to your PPA
cd ..
dput ppa:YOUR_USERNAME/cortex cortex_0.1.0-1ubuntu1_source.changes
```

## Supporting Multiple Ubuntu Versions

To support multiple Ubuntu versions, you need to upload separate packages:

```bash
# For each Ubuntu version
for DIST in focal jammy noble; do
    # Update changelog
    dch -v 0.1.0-1ubuntu1~${DIST}1 -D $DIST "Build for $DIST"
    
    # Build source package
    debuild -S -sa
    
    # Upload
    dput ppa:YOUR_USERNAME/cortex cortex_0.1.0-1ubuntu1~${DIST}1_source.changes
done
```

Ubuntu version codenames:
- `focal` - 20.04 LTS (Focal Fossa)
- `jammy` - 22.04 LTS (Jammy Jellyfish)
- `noble` - 24.04 LTS (Noble Numbat)
- `oracular` - 24.10 (Oracular Oriole)
- `plucky` - 25.04 (Plucky Puffin) - Development version

## After Upload

1. **Wait for Build**: 
   - Check status at: https://launchpad.net/~YOUR_USERNAME/+archive/ubuntu/cortex
   - Builds usually take 15-60 minutes
   - You'll receive email notifications

2. **Test Installation**:
   ```bash
   # Add your PPA
   sudo add-apt-repository ppa:YOUR_USERNAME/cortex
   sudo apt update
   
   # Install
   sudo apt install cortex
   
   # Test
   cortex --version
   ```

3. **Share Installation Instructions**:
   ```bash
   # Users can install with:
   sudo add-apt-repository ppa:YOUR_USERNAME/cortex
   sudo apt update
   sudo apt install cortex
   ```

## Troubleshooting

### Common Issues

1. **GPG Key Not Found**
   - Ensure key is uploaded to keyserver.ubuntu.com
   - Wait a few minutes for propagation

2. **Build Failures**
   - Check build logs on Launchpad
   - Common issue: missing dependencies in debian/control

3. **Version Conflicts**
   - Use proper version scheme: `0.1.0-1ubuntu1~focal1`
   - Higher versions for newer Ubuntu releases

4. **Rejected Upload**
   - Check email for specific rejection reason
   - Common: invalid signature, version already exists

## Credentials Needed

Summary of what you need:
1. ✅ Launchpad account (free)
2. ✅ GPG key (for signing packages)
3. ✅ SSH key (optional, for direct access)
4. ✅ PPA created on Launchpad

## Next Steps

1. Set up your Launchpad account
2. Create GPG key and add to Launchpad
3. Create your PPA
4. Run our build script
5. Upload to PPA
6. Share PPA with users

Your PPA URL will be: `ppa:YOUR_USERNAME/cortex`

Users will install with:
```bash
sudo add-apt-repository ppa:YOUR_USERNAME/cortex
sudo apt update
sudo apt install cortex
```