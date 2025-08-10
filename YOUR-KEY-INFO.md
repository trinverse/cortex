# Your GPG Key Information

## Your Key Details
- **Email**: ashishtyagi10@gmail.com  
- **Key ID (short)**: 65D499A2
- **Key ID (long)**: B8C79B9465D499A2
- **Full Fingerprint**: 81C06C5CAE7E111DFA0386CBB8C79B9465D499A2
- **Public Key File**: ~/cortex-public-key.asc

## ✅ What We've Done
1. ✅ Exported your public key to `~/cortex-public-key.asc`
2. ✅ Uploaded to keyserver.ubuntu.com
3. ✅ Uploaded to keys.openpgp.org

## 📋 Next Steps for Launchpad

### Option 1: Import Using ASCII Key (RECOMMENDED)
1. Go to: https://launchpad.net/~/+editpgpkeys
2. Look for the section "Import an OpenPGP key"
3. Run this command to view your key:
   ```bash
   cat ~/cortex-public-key.asc
   ```
4. Copy the ENTIRE output (including -----BEGIN and -----END lines)
5. Paste it into the text box on Launchpad
6. Click "Import Key"

### Option 2: Import Using Fingerprint
1. Go to: https://launchpad.net/~/+editpgpkeys  
2. In the fingerprint field, enter (no spaces):
   ```
   81C06C5CAE7E111DFA0386CBB8C79B9465D499A2
   ```
3. Click "Import Key"

## 📧 After Import
1. Launchpad will send an encrypted email to ashishtyagi10@gmail.com
2. Save the email to a file (e.g., `launchpad-email.txt`)
3. Decrypt it:
   ```bash
   gpg --decrypt launchpad-email.txt
   ```
4. Open the link in the decrypted message
5. Your key will be confirmed!

## 🚀 Ready to Upload to PPA

Once your key is confirmed on Launchpad, run:
```bash
cd /home/ashish/code/cortex
./packaging/debian/build-ppa-package.sh
```

When prompted, enter:
- Launchpad username: ashishtyagi10
- Email: ashishtyagi10@gmail.com
- GPG key ID: B8C79B9465D499A2
- Ubuntu versions: 4 (for all)

## Quick Test Commands

```bash
# Check if key is on Ubuntu keyserver (wait 5 minutes)
gpg --keyserver keyserver.ubuntu.com --recv-keys B8C79B9465D499A2

# View your public key
cat ~/cortex-public-key.asc
```

## Your PPA URL

Your PPA will be: `ppa:ashishtyagi10/cortex`

Users will install with:
```bash
sudo add-apt-repository ppa:ashishtyagi10/cortex
sudo apt update
sudo apt install cortex
```

## Important Note for keys.openpgp.org
If you uploaded to keys.openpgp.org via command line, you should:
1. Check your email for a verification message from keys.openpgp.org
2. Click the verification link to publish your key
3. Only verified keys are publicly visible

Your key is ready! Use **Option 1** (ASCII import) on Launchpad - it's the most reliable method.