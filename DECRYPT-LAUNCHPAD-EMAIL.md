# How to Decrypt and Confirm Launchpad Email

## Step 1: Save the Email

1. Copy the ENTIRE encrypted email content (including the PGP headers)
2. Save it to a file:

```bash
# Create a file for the encrypted email
nano ~/launchpad-email.txt
```

3. Paste the entire email content (should look like this):
```
-----BEGIN PGP MESSAGE-----

[lots of encrypted text]

-----END PGP MESSAGE-----
```

4. Save and exit (Ctrl+X, then Y, then Enter)

## Step 2: Decrypt the Email

Run this command:

```bash
gpg --decrypt ~/launchpad-email.txt
```

You should see output like:
```
Please follow the link below to confirm that you control this key:

https://launchpad.net/token/[SOME-LONG-TOKEN]

[Additional instructions from Launchpad]
```

## Step 3: Confirm the Key

1. Copy the URL from the decrypted message
2. Open it in your web browser
3. You'll be taken to Launchpad to confirm
4. Click "Confirm" or "Continue"

## Alternative Method: Decrypt and Save Output

If you want to save the decrypted content:

```bash
gpg --decrypt ~/launchpad-email.txt > ~/launchpad-decrypted.txt
cat ~/launchpad-decrypted.txt
```

## If Decryption Fails

### Error: "No secret key"
```bash
# Check if your key is available
gpg --list-secret-keys
```

### Error: "Bad passphrase"
Make sure you're entering the correct GPG passphrase (the password you used when creating the key)

## Quick One-Liner

If you're comfortable with the terminal, you can decrypt and view in one command:

```bash
gpg --decrypt ~/launchpad-email.txt 2>/dev/null | grep -A 5 "https://"
```

This will show you just the confirmation link.

## After Confirmation

Once you click the link and confirm:
1. Your GPG key will be active on Launchpad
2. You can start uploading packages to your PPA
3. Check your key status at: https://launchpad.net/~ashishtyagi10

## Verify Key is Active

After confirming, verify at:
https://launchpad.net/~ashishtyagi10/+editpgpkeys

You should see your key listed with fingerprint:
`81C06C5CAE7E111DFA0386CBB8C79B9465D499A2`

That's it! Your key will be confirmed and ready for PPA uploads.