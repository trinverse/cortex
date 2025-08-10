#!/bin/bash

echo "==================================="
echo "  Launchpad Email Decryption Tool"
echo "==================================="
echo ""

# Check if email file exists
if [ -f ~/launchpad-email.txt ]; then
    echo "Found existing email file at ~/launchpad-email.txt"
    echo "Decrypting..."
    echo ""
else
    echo "Please paste the encrypted email content from Launchpad."
    echo "Press Ctrl+D when done:"
    echo ""
    cat > ~/launchpad-email.txt
    echo ""
    echo "Email saved to ~/launchpad-email.txt"
    echo "Decrypting..."
    echo ""
fi

# Decrypt the email
echo "--- DECRYPTED MESSAGE ---"
gpg --decrypt ~/launchpad-email.txt 2>/dev/null

echo ""
echo "--- INSTRUCTIONS ---"
echo "1. Copy the URL shown above (starts with https://launchpad.net/token/)"
echo "2. Open it in your web browser"
echo "3. Click 'Confirm' on the Launchpad page"
echo ""
echo "After confirming, check your key at:"
echo "https://launchpad.net/~ashishtyagi10/+editpgpkeys"
echo ""

# Offer to save decrypted content
read -p "Save decrypted message to file? (y/n): " SAVE
if [[ $SAVE == "y" || $SAVE == "Y" ]]; then
    gpg --decrypt ~/launchpad-email.txt > ~/launchpad-decrypted.txt 2>/dev/null
    echo "Decrypted message saved to ~/launchpad-decrypted.txt"
fi

echo ""
echo "Once confirmed, you can run the PPA publishing workflow!"