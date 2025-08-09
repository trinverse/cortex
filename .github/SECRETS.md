# GitHub Actions Secrets Configuration

This document lists all the secrets and credentials required for the CI/CD pipelines to build and publish Cortex to various platforms.

## Required GitHub Secrets

### 1. Code Signing & Notarization (macOS)

#### `APPLE_CERTIFICATE`
- **Description**: Base64-encoded Developer ID Application certificate (.p12)
- **How to obtain**:
  1. Export certificate from Keychain Access
  2. Convert to base64: `base64 -i certificate.p12 -o certificate.txt`
  3. Copy contents of certificate.txt

#### `APPLE_CERTIFICATE_PASSWORD`
- **Description**: Password for the .p12 certificate
- **How to obtain**: Password set when exporting the certificate

#### `APPLE_DEVELOPER_ID`
- **Description**: Developer ID for code signing (e.g., "Developer ID Application: Your Name (TEAMID)")
- **How to obtain**: From Apple Developer account

#### `APPLE_INSTALLER_ID`
- **Description**: Developer ID for installer signing
- **How to obtain**: "Developer ID Installer: Your Name (TEAMID)"

#### `APPLE_ID`
- **Description**: Apple ID email for notarization
- **How to obtain**: Your Apple Developer account email

#### `APPLE_TEAM_ID`
- **Description**: Apple Developer Team ID
- **How to obtain**: From Apple Developer account (10-character ID)

#### `APPLE_PASSWORD`
- **Description**: App-specific password for notarization
- **How to obtain**:
  1. Go to https://appleid.apple.com
  2. Generate app-specific password
  3. Save securely

#### `KEYCHAIN_PASSWORD`
- **Description**: Temporary keychain password for CI
- **How to obtain**: Generate a secure random password

### 2. Package Managers

#### `CHOCOLATEY_API_KEY`
- **Description**: API key for Chocolatey package publishing
- **How to obtain**:
  1. Create account at https://chocolatey.org
  2. Go to https://chocolatey.org/account
  3. Copy API key

#### `HOMEBREW_TAP_TOKEN`
- **Description**: GitHub Personal Access Token for updating Homebrew tap
- **How to obtain**:
  1. Create PAT at https://github.com/settings/tokens
  2. Grant `repo` scope
  3. Must have write access to homebrew-cortex repository

#### `SNAPCRAFT_STORE_CREDENTIALS`
- **Description**: Snap Store login credentials (exported)
- **How to obtain**:
  ```bash
  snapcraft login
  snapcraft export-login credentials.txt
  cat credentials.txt  # Copy this content
  ```

#### `AUR_SSH_KEY`
- **Description**: SSH private key for AUR repository access
- **How to obtain**:
  1. Generate SSH key: `ssh-keygen -t ed25519 -f aur_key`
  2. Add public key to AUR account: https://aur.archlinux.org/account
  3. Copy private key content

### 3. Microsoft Store (Optional)

#### `WINDOWS_STORE_TENANT_ID`
- **Description**: Azure AD Tenant ID
- **How to obtain**: From Partner Center account

#### `WINDOWS_STORE_CLIENT_ID`
- **Description**: Azure AD Application ID
- **How to obtain**: From Partner Center API access

#### `WINDOWS_STORE_CLIENT_SECRET`
- **Description**: Azure AD Application secret
- **How to obtain**: Generate in Azure Portal

### 4. Analytics & Monitoring (Optional)

#### `SENTRY_AUTH_TOKEN`
- **Description**: Sentry authentication token for uploading debug symbols
- **How to obtain**: From Sentry project settings

#### `CODECOV_TOKEN`
- **Description**: Codecov upload token
- **How to obtain**: From Codecov project settings

## Setting Up Secrets

### Via GitHub Web Interface

1. Go to your repository on GitHub
2. Navigate to Settings → Secrets and variables → Actions
3. Click "New repository secret"
4. Add each secret with its name and value

### Via GitHub CLI

```bash
# Install GitHub CLI
brew install gh  # or your package manager

# Authenticate
gh auth login

# Add secrets
gh secret set APPLE_CERTIFICATE < certificate.txt
gh secret set APPLE_CERTIFICATE_PASSWORD
gh secret set CHOCOLATEY_API_KEY
# ... etc
```

## Repository Setup Requirements

### 1. Create Homebrew Tap Repository

```bash
# Create repository named 'homebrew-cortex' in your organization
# Structure:
homebrew-cortex/
  └── Formula/
      └── cortex.rb
```

### 2. Register on Package Repositories

1. **Chocolatey**: https://chocolatey.org/register
2. **Snap Store**: https://snapcraft.io/account
3. **AUR**: https://aur.archlinux.org/register
4. **Microsoft Store**: https://partner.microsoft.com/

### 3. Apple Developer Program

- Requires paid membership ($99/year)
- Enroll at: https://developer.apple.com/programs/

## Security Best Practices

1. **Rotate credentials regularly**
   - API keys every 90 days
   - Certificates before expiration

2. **Use least privilege**
   - Create specific tokens/keys for CI/CD only
   - Limit scope to minimum required

3. **Monitor usage**
   - Check GitHub Actions logs regularly
   - Set up alerts for failed publishes

4. **Backup credentials**
   - Store securely in password manager
   - Document expiration dates

## Testing Workflows

### Test without publishing

```yaml
# Add to workflow file for dry-run
env:
  DRY_RUN: true
```

### Test specific workflow

```bash
# Use act for local testing
brew install act
act -W .github/workflows/build-release.yml
```

## Troubleshooting

### Common Issues

1. **Certificate expired**
   - Renew from Apple Developer account
   - Update secrets

2. **API key invalid**
   - Regenerate from service
   - Update secret

3. **Notarization fails**
   - Check Apple Developer account status
   - Verify app-specific password

4. **Package rejected**
   - Review service guidelines
   - Check package metadata

## Support Contacts

- **Apple Developer**: https://developer.apple.com/support/
- **Chocolatey**: https://chocolatey.org/contact
- **Snap Store**: https://forum.snapcraft.io/
- **AUR**: https://bbs.archlinux.org/

## Checklist for New Release

- [ ] All secrets configured
- [ ] Test builds passing
- [ ] Package repositories accessible
- [ ] Certificates valid (not expired)
- [ ] Release notes prepared
- [ ] Version bumped in Cargo.toml

---

**Security Notice**: Never commit credentials to the repository. This file should be in `.gitignore` if it contains actual values.