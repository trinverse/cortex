# GPG Key Upload Instructions

## Step 1: Find Your GPG Key ID

Run these commands to find your key in different formats:

```bash
# Method 1: List all secret keys with long format
gpg --list-secret-keys --keyid-format=long

# Method 2: List with fingerprints
gpg --list-keys --fingerprint

# Method 3: Show just the key IDs
gpg --list-keys --keyid-format=short
```

Look for output like:
```
sec   rsa4096/XXXXXXXXXXXXXXXX 2025-01-09 [SC]
      Key fingerprint = XXXX XXXX XXXX XXXX XXXX  XXXX XXXX XXXX XXXX XXXX
uid                 [ultimate] Your Name <your.email@example.com>
```

The key ID is the part after `rsa4096/` (16 characters) or the last 8 characters for short format.

## Step 2: Export Your Public Key

Once you find your key ID, export it:

```bash
# Replace YOUR_EMAIL with the email you used when creating the key
gpg --armor --export YOUR_EMAIL > ~/cortex-public-key.asc

# Or use the key ID (replace KEY_ID with your actual key ID)
gpg --armor --export KEY_ID > ~/cortex-public-key.asc
```

## Step 3: Upload to keys.openpgp.org

### Option A: Web Upload (Recommended)
1. Go to https://keys.openpgp.org/upload
2. Click "Choose File" and select `~/cortex-public-key.asc`
3. Click "Upload"
4. You'll receive a verification email - check your inbox
5. Click the verification link in the email

### Option B: Command Line Upload
```bash
# Read your key ID from the list command above
# Use the 16-character or 8-character key ID
gpg --keyserver keys.openpgp.org --send-keys YOUR_KEY_ID
```

## Step 4: Add to Launchpad

### Method 1: Using the ASCII Key (Most Reliable)
1. Go to https://launchpad.net/~/+editpgpkeys
2. Look for "Import an OpenPGP key"
3. Open the file `~/cortex-public-key.asc` in a text editor
4. Copy the ENTIRE contents (including the BEGIN and END lines)
5. Paste it into the text box on Launchpad
6. Click "Import Key"

Example of what to copy:
```
-----BEGIN PGP PUBLIC KEY BLOCK-----

[Long block of characters]
[Multiple lines of base64 encoded data]

-----END PGP PUBLIC KEY BLOCK-----
```

### Method 2: Using Fingerprint from keys.openpgp.org
After uploading to keys.openpgp.org:
1. Wait 5-10 minutes for synchronization
2. Get your fingerprint:
   ```bash
   gpg --fingerprint YOUR_EMAIL
   ```
3. Go to https://launchpad.net/~/+editpgpkeys
4. Enter the fingerprint (remove spaces): `XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX`
5. Click "Import Key"

## Step 5: Confirm the Key

1. Launchpad will send you an encrypted email
2. Save the email to a file: `launchpad-confirm.txt`
3. Decrypt it:
   ```bash
   gpg --decrypt launchpad-confirm.txt
   ```
4. Click the link in the decrypted message
5. Your key is now verified!

## Troubleshooting

### "Key not found" on Launchpad
- Make sure you uploaded to keys.openpgp.org and verified the email
- Wait 10-15 minutes for synchronization
- Try using the ASCII import method instead

### Can't decrypt Launchpad email
```bash
# Check which keys you have
gpg --list-secret-keys

# Make sure you're using the right key
gpg --decrypt --local-user YOUR_EMAIL launchpad-confirm.txt
```

### Finding the right key to upload
You should upload your PRIMARY key (the one with [ultimate] trust level).
This is typically the first key shown when you run:
```bash
gpg --list-secret-keys
```

## Quick Commands Summary

```bash
# 1. Find your key
gpg --list-secret-keys --keyid-format=long

# 2. Export it
gpg --armor --export YOUR_EMAIL > ~/cortex-public-key.asc

# 3. View it to confirm
cat ~/cortex-public-key.asc

# 4. Upload to keys.openpgp.org (web interface is easier)
# Go to https://keys.openpgp.org/upload

# 5. Copy contents to Launchpad
# Go to https://launchpad.net/~/+editpgpkeys
```

## What You Need

1. **Your GPG Key ID**: The 8 or 16 character identifier
2. **Your Email**: The email used when creating the GPG key
3. **The Public Key**: The ASCII-armored export (the .asc file)

That's the key you should upload to both keys.openpgp.org and Launchpad!