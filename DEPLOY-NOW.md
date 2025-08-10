# 🚀 Deploy Cortex to PPA - Final Steps

## ✅ Prerequisites Completed
- [x] GPG Key created and confirmed on Launchpad
- [x] Workflows pushed to GitHub
- [x] All files ready for deployment

## 📋 Deployment Checklist

### Step 1: Add GitHub Secrets (2 minutes)

Go to: https://github.com/trinverse/cortex/settings/secrets/actions/new

Add these 6 secrets one by one:

#### 1. GPG_PRIVATE_KEY
```bash
cat ~/cortex-private-key-base64.txt
```
Copy entire output and paste as value

#### 2. GPG_PASSPHRASE
Your GPG key password

#### 3. GPG_KEY_ID
```
B8C79B9465D499A2
```

#### 4. LAUNCHPAD_USERNAME
```
ashishtyagi10
```

#### 5. DEBEMAIL
```
ashishtyagi10@gmail.com
```

#### 6. DEBFULLNAME
```
Ashish Tyagi
```

### Step 2: Create Your PPA on Launchpad (1 minute)

1. Go to: https://launchpad.net/~ashishtyagi10/+activate-ppa
2. Fill in:
   - **URL**: `cortex`
   - **Display name**: `Cortex File Manager`
   - **Description**: 
   ```
   Modern terminal file manager built with Rust.
   Features dual-pane interface, vim-style navigation, 
   plugin system, and advanced file operations.
   ```
3. Click "Activate"

Your PPA URL will be: `ppa:ashishtyagi10/cortex`

### Step 3: Run the Deployment (1 minute)

#### Option A: Via GitHub Web UI
1. Go to: https://github.com/trinverse/cortex/actions/workflows/publish-ppa-fixed.yml
2. Click "Run workflow"
3. Set:
   - Version: `0.1.0`
   - Distribution: `focal` (test with one first)
4. Click green "Run workflow" button

#### Option B: Via Command Line
```bash
# Using GitHub CLI
gh workflow run publish-ppa-fixed.yml \
  -f version=0.1.0 \
  -f distribution=focal
```

### Step 4: Monitor Progress

1. **GitHub Actions** (2-5 minutes):
   - https://github.com/trinverse/cortex/actions
   - Watch for green checkmark

2. **Launchpad Build** (15-60 minutes):
   - https://launchpad.net/~ashishtyagi10/+archive/ubuntu/cortex
   - You'll get email when build completes

### Step 5: Deploy to All Ubuntu Versions

Once focal test succeeds, deploy to all versions:

```bash
# Run for each distribution
for dist in jammy noble oracular; do
  gh workflow run publish-ppa-fixed.yml \
    -f version=0.1.0 \
    -f distribution=$dist
  echo "Started build for $dist"
  sleep 5
done
```

Or use the web UI to run for each distribution.

### Step 6: Test Installation

After builds complete on Launchpad:

```bash
# Add your PPA
sudo add-apt-repository ppa:ashishtyagi10/cortex
sudo apt update

# Install Cortex
sudo apt install cortex

# Verify it works
cortex --version
```

## 🎯 Quick Commands Summary

```bash
# 1. Check if secrets are set (optional)
gh secret list

# 2. Run workflow for focal (test)
gh workflow run publish-ppa-fixed.yml -f version=0.1.0 -f distribution=focal

# 3. Check workflow status
gh run list --workflow=publish-ppa-fixed.yml

# 4. After success, run for all distributions
for dist in focal jammy noble oracular; do
  gh workflow run publish-ppa-fixed.yml -f version=0.1.0 -f distribution=$dist
  sleep 5
done
```

## 📊 Expected Timeline

- **Now**: Add GitHub secrets (2 min)
- **+2 min**: Run workflow
- **+5 min**: GitHub Actions completes, uploads to Launchpad
- **+20-60 min**: Launchpad builds packages
- **+1 hour**: Cortex available via `apt install`!

## 🚨 If Something Goes Wrong

### GitHub Actions fails
- Check the logs in Actions tab
- Usually: missing secret or GPG issue

### Launchpad rejects upload
- Check email for rejection reason
- Common: version already exists (bump version number)

### Build fails on Launchpad
- Check build log on Launchpad
- Common: missing build dependency

## 🎉 Success Indicators

You'll know it worked when:
1. ✅ GitHub Actions shows green checkmark
2. ✅ Launchpad shows "Published" status
3. ✅ `sudo apt install cortex` works!

## 📢 Share with Users

Once live, users can install with:
```bash
sudo add-apt-repository ppa:ashishtyagi10/cortex
sudo apt update
sudo apt install cortex
```

Ready? Start with Step 1 - Add the GitHub secrets!