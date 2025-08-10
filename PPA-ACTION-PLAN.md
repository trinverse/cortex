# 🚀 PPA Publishing Action Plan

## Your Information
- **Email**: ashishtyagi10@gmail.com
- **GPG Key ID**: B8C79B9465D499A2
- **Key Files Ready**:
  - ✅ Public key: ~/cortex-public-key.asc
  - ✅ Private key (base64): ~/cortex-private-key-base64.txt

## Step-by-Step Instructions

### Step 1: Add Your Launchpad Username
First, you need to tell me your Launchpad username so I can complete the setup.

### Step 2: Add GitHub Secrets (5 minutes)

Go to: https://github.com/trinverse/cortex/settings/secrets/actions

Add these 6 secrets:

#### Secret 1: GPG_PRIVATE_KEY
```bash
cat ~/cortex-private-key-base64.txt
```
Copy the entire output and paste as the value.

#### Secret 2: GPG_PASSPHRASE
Your GPG key password (the one you entered when creating the key)

#### Secret 3: GPG_KEY_ID
Value: `B8C79B9465D499A2`

#### Secret 4: LAUNCHPAD_USERNAME
Value: `ashishtyagi10`

#### Secret 5: DEBEMAIL
Value: `ashishtyagi10@gmail.com`

#### Secret 6: DEBFULLNAME
Value: `Ashish Tyagi`

### Step 3: Confirm Launchpad GPG Key (2 minutes)

1. Go to https://launchpad.net/~/+editpgpkeys
2. If your key isn't there yet:
   ```bash
   cat ~/cortex-public-key.asc
   ```
3. Copy the entire content and paste in "Import an OpenPGP key"
4. Check your email for the encrypted confirmation
5. Decrypt and confirm:
   ```bash
   gpg --decrypt launchpad-email.txt
   ```

### Step 4: Create Your PPA (if not done)

1. Go to https://launchpad.net/~ashishtyagi10
2. Click "Create a new PPA"
3. URL: `cortex`
4. Display name: `Cortex File Manager`
5. Click "Activate"

### Step 5: Push Changes to GitHub

```bash
cd /home/ashish/code/cortex

# Add all changes
git add .

# Commit
git commit -m "feat: Add PPA publishing workflow with GitHub Actions

- Add automated PPA publishing workflow
- Support for focal, jammy, and noble distributions
- Manual trigger workflow for testing
- Complete GitHub secrets documentation"

# Push to GitHub
git push origin main
```

### Step 6: Run the Workflow

#### Option A: From GitHub Website
1. Go to: https://github.com/trinverse/cortex/actions
2. Click on "Manual PPA Publish" workflow
3. Click "Run workflow"
4. Version: `0.1.0`
5. Distribution: `all`
6. Click "Run workflow" (green button)

#### Option B: Using GitHub CLI
```bash
# Install GitHub CLI if needed
sudo apt install gh

# Authenticate
gh auth login

# Run workflow
gh workflow run publish-ppa-manual.yml \
  -f version=0.1.0 \
  -f distribution=all
```

### Step 7: Monitor Progress

1. **GitHub Actions**: https://github.com/trinverse/cortex/actions
   - Watch the workflow run (takes ~5-10 minutes)
   - Check for any errors

2. **Launchpad PPA**: https://launchpad.net/~ashishtyagi10/+archive/ubuntu/cortex
   - After GitHub Actions completes, packages appear here
   - Build takes 15-60 minutes on Launchpad servers
   - You'll get email when complete

### Step 8: Test Installation

Once Launchpad builds are complete:

```bash
# Add your PPA
sudo add-apt-repository ppa:ashishtyagi10/cortex
sudo apt update

# Install Cortex
sudo apt install cortex

# Test it works
cortex --version
```

## 📋 Quick Checklist

Before running the workflow, confirm:
- [ ] Launchpad account created
- [ ] GPG key uploaded to Launchpad and confirmed
- [ ] PPA created (ppa:ashishtyagi10/cortex)
- [ ] All 6 GitHub secrets added
- [ ] Changes pushed to GitHub

## 🎯 Success Indicators

You'll know it's working when:
1. ✅ GitHub Actions workflow shows green checkmark
2. ✅ Packages appear on Launchpad PPA page
3. ✅ Build status shows "Successfully built" on Launchpad
4. ✅ You can install with `sudo apt install cortex`

## 🚨 Troubleshooting

### "GPG signature failed"
- Check GPG_PASSPHRASE secret is correct
- Verify GPG_KEY_ID matches

### "Upload to PPA failed"  
- Check LAUNCHPAD_USERNAME is correct
- Verify GPG key is confirmed on Launchpad

### "Package build failed on Launchpad"
- Check build log on Launchpad
- Common: missing dependencies, cargo version

## 🧹 Cleanup After Success

Once everything is working:
```bash
# Remove sensitive files
rm ~/cortex-private-key.asc
rm ~/cortex-private-key-base64.txt

# Keep public key for reference
# ~/cortex-public-key.asc is safe to keep
```

## 📢 Share With Users

Once your PPA is working, users can install Cortex with:
```bash
sudo add-apt-repository ppa:ashishtyagi10/cortex
sudo apt update
sudo apt install cortex
```

## Need Help?

- Check workflow logs in GitHub Actions
- Check build logs on Launchpad
- The workflow will show detailed error messages

Ready to publish! Your Launchpad username (ashishtyagi10) has been configured.