# GitHub Secrets Setup for PPA Publishing

## Required GitHub Secrets

Go to: https://github.com/trinverse/cortex/settings/secrets/actions

Click "New repository secret" for each of these:

### 1. GPG_PRIVATE_KEY
```bash
# Copy the contents of this file:
cat ~/cortex-private-key-base64.txt
```
- Name: `GPG_PRIVATE_KEY`
- Value: [Paste the entire base64 encoded content]

### 2. GPG_PASSPHRASE
- Name: `GPG_PASSPHRASE`
- Value: [Your GPG key passphrase - the password you used when creating the key]

### 3. GPG_KEY_ID
- Name: `GPG_KEY_ID`
- Value: `B8C79B9465D499A2`

### 4. LAUNCHPAD_USERNAME
- Name: `LAUNCHPAD_USERNAME`
- Value: `ashishtyagi10`

### 5. DEBEMAIL
- Name: `DEBEMAIL`
- Value: `ashishtyagi10@gmail.com`

### 6. DEBFULLNAME
- Name: `DEBFULLNAME`
- Value: `Ashish Tyagi`

## How to Add Each Secret

1. Go to: https://github.com/trinverse/cortex/settings/secrets/actions
2. Click "New repository secret"
3. Enter the Name exactly as shown above
4. Enter the Value
5. Click "Add secret"
6. Repeat for all 6 secrets

## Verify Setup

After adding all secrets, you should see these in your repository secrets:
- ✅ GPG_PRIVATE_KEY
- ✅ GPG_PASSPHRASE
- ✅ GPG_KEY_ID
- ✅ LAUNCHPAD_USERNAME
- ✅ DEBEMAIL
- ✅ DEBFULLNAME

## Test the Workflow

### Option 1: Manual Trigger
1. Go to: https://github.com/trinverse/cortex/actions/workflows/publish-ppa.yml
2. Click "Run workflow"
3. Enter version: `0.1.0`
4. Enter distributions: `focal jammy noble`
5. Click "Run workflow"

### Option 2: Command Line Trigger
```bash
# First, push all changes
git add .
git commit -m "feat: Add PPA publishing workflow"
git push origin main

# Then trigger the workflow using GitHub CLI
gh workflow run publish-ppa.yml \
  -f version=0.1.0 \
  -f distributions="focal jammy noble"
```

## Security Notes

⚠️ **IMPORTANT SECURITY REMINDERS:**
- Never commit the private key files to git
- Delete local copies after adding to GitHub:
  ```bash
  rm ~/cortex-private-key.asc
  rm ~/cortex-private-key-base64.txt
  ```
- Keep your GPG passphrase secure
- These secrets are encrypted by GitHub and only accessible during workflow runs

## Troubleshooting

### "Invalid signature" error
- Make sure GPG_PASSPHRASE is correct
- Verify GPG_KEY_ID matches your key

### "Authentication failed" on Launchpad
- Verify LAUNCHPAD_USERNAME is correct
- Make sure your GPG key is confirmed on Launchpad

### Build failures
- Check the Actions log for specific errors
- Common issue: missing dependencies in debian/control

## Next Steps

1. Add all 6 secrets to GitHub
2. Run the workflow
3. Monitor at: https://github.com/trinverse/cortex/actions
4. Check PPA build status at: https://launchpad.net/~ashishtyagi10/+archive/ubuntu/cortex

## Clean Up

After successfully adding secrets to GitHub:
```bash
# Remove sensitive files
rm ~/cortex-private-key.asc
rm ~/cortex-private-key-base64.txt

# Keep the public key for reference
# ~/cortex-public-key.asc
```