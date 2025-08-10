# 🍺 Homebrew Installation - Simple Setup

Since Cortex is hosted on GitHub, we can use the repository itself as a Homebrew tap!

## For Users - How to Install

Users can install Cortex directly from your GitHub repository:

```bash
# Install directly from GitHub
brew install trinverse/cortex/cortex

# Or add the tap first (optional)
brew tap trinverse/cortex
brew install cortex
```

That's it! Homebrew will automatically find the Formula in your repository.

## For You - How to Release

### 1. Create a Release (One-Time Setup)

Just create a git tag and push it:

```bash
# Tag the current version
git tag v0.1.0
git push origin v0.1.0
```

The GitHub Actions workflow will:
- Build binaries for macOS (Intel & Apple Silicon) and Linux
- Create a GitHub release with all artifacts
- Update the Formula/cortex.rb file with correct SHA256 hashes
- Commit the changes back to your repository

### 2. That's It!

Users can now install with:
```bash
brew install trinverse/cortex/cortex
```

## How It Works

Homebrew has a feature called "GitHub repository taps". When users run:
- `brew tap trinverse/cortex` - Homebrew looks for Formula directory in your repo
- `brew install trinverse/cortex/cortex` - Installs directly without adding tap

Your repository structure:
```
cortex/
├── Formula/
│   └── cortex.rb      # Homebrew formula (auto-updated by GitHub Actions)
├── .github/
│   └── workflows/
│       └── release-homebrew.yml  # Handles everything automatically
└── ... (your code)
```

## Manual Release (If Needed)

If you want to create a release manually:

1. Build on your Linux machine:
   ```bash
   ./scripts/create-release.sh 0.1.0
   ```

2. Create a GitHub release:
   - Go to https://github.com/trinverse/cortex/releases/new
   - Tag: v0.1.0
   - Upload the .tar.gz files from dist/
   - Publish

3. Update Formula/cortex.rb with the SHA256 values from your release

## Testing

Test the formula locally:
```bash
# Test installation
brew install --verbose Formula/cortex.rb

# Or test from GitHub
brew install --verbose trinverse/cortex/cortex
```

## Updating

When releasing a new version:
```bash
git tag v0.2.0
git push origin v0.2.0
```

Users update with:
```bash
brew update
brew upgrade cortex
```

## FAQ

**Q: Do I need a separate tap repository?**
A: No! Homebrew can use your main repository as a tap.

**Q: What's the Formula directory for?**
A: Homebrew looks for formulas in the Formula/ or HomebrewFormula/ directory.

**Q: How do users know about this?**
A: Add to your README:
```markdown
### Install via Homebrew
\`\`\`bash
brew install trinverse/cortex/cortex
\`\`\`
```

**Q: Can I submit to homebrew-core?**
A: Yes, once your project has 30+ forks, 30+ watchers, and 75+ stars.

That's all! Much simpler than maintaining a separate tap repository.