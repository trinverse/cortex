# 🚀 Quick Start: Publish to PPA NOW!

## Your Info (All Set!)
- **Launchpad Username**: ashishtyagi10
- **Email**: ashishtyagi10@gmail.com
- **GPG Key**: B8C79B9465D499A2
- **PPA URL**: ppa:ashishtyagi10/cortex

## Step 1: Add GitHub Secrets (2 minutes)

Go to: https://github.com/trinverse/cortex/settings/secrets/actions

Click "New repository secret" and add these 6 secrets:

| Secret Name | Value |
|------------|-------|
| GPG_PRIVATE_KEY | Run: `cat ~/cortex-private-key-base64.txt` and copy output |
| GPG_PASSPHRASE | Your GPG key password |
| GPG_KEY_ID | `B8C79B9465D499A2` |
| LAUNCHPAD_USERNAME | `ashishtyagi10` |
| DEBEMAIL | `ashishtyagi10@gmail.com` |
| DEBFULLNAME | `Ashish Tyagi` |

## Step 2: Push to GitHub (30 seconds)

```bash
cd /home/ashish/code/cortex
git add .
git commit -m "feat: Add PPA publishing workflow for Ubuntu

- Automated PPA publishing via GitHub Actions
- Support for Ubuntu 20.04, 22.04, and 24.04
- Manual trigger workflow for testing
- Complete documentation and setup guides"
git push origin main
```

## Step 3: Run the Workflow (1 minute)

### Option A: Via Web
1. Go to: https://github.com/trinverse/cortex/actions/workflows/publish-ppa-manual.yml
2. Click "Run workflow"
3. Version: `0.1.0`
4. Distribution: `all`
5. Click green "Run workflow" button

### Option B: Via CLI
```bash
gh workflow run publish-ppa-manual.yml -f version=0.1.0 -f distribution=all
```

## Step 4: Monitor (5-60 minutes)

1. **GitHub Actions** (5-10 min): https://github.com/trinverse/cortex/actions
2. **Launchpad Builds** (15-60 min): https://launchpad.net/~ashishtyagi10/+archive/ubuntu/cortex

## Step 5: Test Installation

Once Launchpad shows "Successfully built":

```bash
# Add your PPA
sudo add-apt-repository ppa:ashishtyagi10/cortex
sudo apt update

# Install
sudo apt install cortex

# Test
cortex --version
```

## 🎯 Success!

Users can now install Cortex with:
```bash
sudo add-apt-repository ppa:ashishtyagi10/cortex
sudo apt update
sudo apt install cortex
```

## 🧹 Cleanup

After confirming everything works:
```bash
# Remove sensitive files
rm ~/cortex-private-key.asc
rm ~/cortex-private-key-base64.txt
```

## Quick URLs

- **Your PPA**: https://launchpad.net/~ashishtyagi10/+archive/ubuntu/cortex
- **GitHub Actions**: https://github.com/trinverse/cortex/actions
- **Add Secrets**: https://github.com/trinverse/cortex/settings/secrets/actions

That's it! Your PPA will be live in about an hour! 🎉