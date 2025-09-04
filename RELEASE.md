# Release Process for Cortex

This document describes the automated release process for Cortex.

## Pre-release Steps

1.  **Update Version**: Ensure the version number in `Cargo.toml` is updated to the desired version (e.g., `0.2.0`).
2.  **Update Changelog**: Make sure `CHANGELOG.md` is up-to-date with the latest changes for the new version.
3.  **Run Local Tests**: It's always a good practice to run the test suite locally before creating a release.
    ```bash
    cargo test
    ```
4.  **Commit Changes**: Commit the version and changelog updates.
    ```bash
    git add Cargo.toml CHANGELOG.md
    git commit -m "chore: Prepare for release v0.2.0"
    ```

## Automated Release Workflow

The entire release process is automated using the `.github/workflows/release.yml` GitHub Actions workflow.

To trigger a new release, create and push a new Git tag starting with `v` (e.g., `v0.2.0`):

```bash
git tag -a v0.2.0 -m "Release v0.2.0"
git push origin main --follow-tags
```

Pushing a new tag will trigger the following process:

1.  **Build and Test**: A matrix build job compiles and tests the application on macOS (x86_64, aarch64), Linux (x86_64, aarch64), and Windows (x86_64).
2.  **Code Signing and Packaging**:
    *   **macOS**: The binary is signed and notarized with Apple. The final artifact is a `.tar.gz` archive.
    *   **Windows**: An MSI installer is created using the WiX Toolset. If configured, the installer is also code-signed.
    *   **Linux**: The binary is packaged into a `.tar.gz` archive.
3.  **Create GitHub Release**: A new public GitHub Release is created with the tag. All the generated artifacts (`.tar.gz` and `.msi` files) are uploaded to this release.
4.  **Publish to Package Managers**: Once the GitHub Release is created, a series of jobs are triggered to publish the application to various package managers:
    *   **Homebrew**: The Homebrew formula in the `trinverse/homebrew-cortex` tap is automatically updated with the new version and checksums.
    *   **Chocolatey**: A Chocolatey package is created and pushed to the Chocolatey Community Repository.
    *   **Winget**: A pull request is automatically created to the `microsoft/winget-pkgs` repository with the new Winget manifest.
    *   **PPA (Ubuntu)**: The `publish-ppa-all.yml` workflow is triggered, which builds and publishes the new version to the PPA for all supported Ubuntu distributions.

## Manual Release

If a manual release is necessary, you can trigger the workflow from the GitHub Actions tab:

1.  Go to **Actions** -> **Release**.
2.  Click **Run workflow**.
3.  Enter the tag you want to release (e.g., `v0.2.0`).
4.  Click **Run workflow**.

## Post-release Steps

After the workflow completes successfully, you should:

1.  **Verify Release Assets**: Check the GitHub Release to ensure all artifacts were uploaded correctly.
2.  **Check Package Managers**: Verify that the new version is available on Homebrew, Chocolatey, and the PPA. Check that the Winget PR was created.
3.  **Announce the Release**: Share the good news with the community!
