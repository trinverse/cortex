# Publishing Cortex to APT (Ubuntu PPA)

## Quick Start Guide

### What You Need to Do:

## 1. Create Launchpad Account (5 minutes)
1. Go to https://launchpad.net
2. Click "Sign Up" 
3. Create Ubuntu One account
4. Verify your email

## 2. Generate GPG Key (10 minutes)
```bash
# Generate GPG key
gpg --full-generate-key

# Select:
# (1) RSA and RSA
# 4096 bits  
# 0 (does not expire)
# Your real name
# Your email (MUST match Launchpad account)

# Get your key ID
gpg --list-secret-keys --keyid-format=long
# Look for line like: sec   rsa4096/XXXXXXXXXXXXXXXX

# Upload to Ubuntu keyserver (replace KEYID)
gpg --keyserver keyserver.ubuntu.com --send-keys KEYID
```

## 3. Add GPG Key to Launchpad (5 minutes)
1. Get your key fingerprint:
   ```bash
   gpg --fingerprint your.email@example.com
   ```
2. Go to: https://launchpad.net/~YOUR_USERNAME/+editpgpkeys
3. Paste the fingerprint
4. Launchpad sends encrypted email
5. Decrypt it:
   ```bash
   # Save email to file, then:
   gpg --decrypt email.txt
   ```
6. Click the link in decrypted message to confirm

## 4. Create Your PPA (2 minutes)
1. Go to: https://launchpad.net/~YOUR_USERNAME
2. Click "Create a new PPA"
3. Fill in:
   - URL: `cortex`
   - Display name: `Cortex File Manager`
   - Description: `Modern terminal file manager built with Rust`
4. Click "Activate"

## 5. Install Build Tools (5 minutes)
```bash
sudo apt-get update
sudo apt-get install -y devscripts debhelper dput gnupg lintian
```

## 6. Build and Upload Package (10 minutes)
```bash
# Run our automated script
cd /path/to/cortex
./packaging/debian/build-ppa-package.sh

# Enter when prompted:
# - Your Launchpad username
# - Your email (same as GPG key)
# - Your GPG key ID
# - Choose Ubuntu versions (press 4 for all)
# - Type 'y' to upload
```

## 7. Wait for Build (15-60 minutes)
- Check status: https://launchpad.net/~YOUR_USERNAME/+archive/ubuntu/cortex
- You'll get email when build completes

## 8. Test Installation
```bash
# Add your PPA
sudo add-apt-repository ppa:YOUR_USERNAME/cortex
sudo apt update

# Install
sudo apt install cortex

# Test
cortex --version
```

## Done! 🎉

Users can now install Cortex with:
```bash
sudo add-apt-repository ppa:YOUR_USERNAME/cortex
sudo apt update
sudo apt install cortex
```

---

## Troubleshooting

### "GPG key not found"
- Wait 5 minutes after uploading to keyserver
- Try: `gpg --keyserver hkp://keyserver.ubuntu.com:80 --send-keys KEYID`

### "Package build failed"
- Check build log on Launchpad
- Usually missing dependency - check debian/control

### "Already exists" error
- Increment version in changelog
- Use: `0.1.0-2ubuntu1` for next upload

### Can't decrypt Launchpad email
- Make sure you're using the right GPG key
- Try: `gpg --list-secret-keys`

## Alternative: Quick Local Testing

If you just want to test locally first:
```bash
# Build local .deb package
./release-v0.1.0-quick.sh

# Install locally
sudo dpkg -i cortex_0.1.0_amd64.deb

# Test
cortex
```

## Summary of Credentials Needed

1. **Launchpad Account**
   - Username: (you choose)
   - Email: (your email)
   - Password: (you choose)

2. **GPG Key**
   - Generated locally
   - Must use same email as Launchpad
   - Remember the key ID

3. **PPA Name**
   - Will be: ppa:YOUR_USERNAME/cortex

That's all! No payment or special access required - it's all free.

## Support

If you need help:
1. Launchpad Help: https://help.launchpad.net/
2. Ask Ubuntu: https://askubuntu.com/questions/tagged/ppa
3. Create issue at: https://github.com/trinverse/cortex/issues