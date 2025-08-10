# Branching Strategy

## Overview

This project follows a Git Flow-inspired branching strategy with automated versioning and releases.

## Branch Structure

### `main`
- **Purpose**: Production-ready code
- **Protected**: Yes
- **Merge from**: `develop` only (via PR)
- **Triggers**: Automatic versioning, tagging, and GitHub releases
- **Version bumps**: Automatic patch version by default

### `develop`
- **Purpose**: Integration branch for features
- **Default branch**: Yes
- **Protected**: Yes (requires PR reviews)
- **Merge from**: Feature branches
- **Triggers**: CI/CD tests and builds

### Feature Branches
- **Naming**: `feature/*`, `fix/*`, `chore/*`, or personal branches (e.g., `atyagi`)
- **Branch from**: `develop`
- **Merge to**: `develop` (via PR)
- **Deleted after merge**: Recommended

## Workflow

### 1. Feature Development
```bash
# Create feature branch from develop
git checkout develop
git pull origin develop
git checkout -b feature/my-feature

# Make changes and commit
git add .
git commit -m "feat: add new feature"

# Push and create PR to develop
git push -u origin feature/my-feature
# Create PR via GitHub UI or CLI
```

### 2. Merge to Develop
- Create PR from feature branch to `develop`
- CI runs automatically (lint, test, build)
- After approval and checks pass, merge PR
- Branch can be deleted after merge

### 3. Release to Main
- When ready to release, use GitHub Actions workflow "Promote to Main"
- Select version bump type (patch/minor/major)
- This creates a PR from `develop` to `main`
- When PR is merged:
  - Version is automatically bumped
  - Git tag is created
  - GitHub release is generated
  - Build artifacts are created

## Version Management

### Automatic Versioning
- **Patch version** (x.x.+1): Default for all merges to main
- **Minor version** (x.+1.0): Manually triggered for new features
- **Major version** (+1.0.0): Manually triggered for breaking changes

### Version Bump Rules
- Bug fixes, small improvements → Patch
- New features, non-breaking changes → Minor
- Breaking changes, major refactors → Major

## CI/CD Pipelines

### On Push to Develop
- Runs linting (rustfmt, clippy)
- Runs tests on multiple platforms
- Builds the project

### On Merge to Main
1. Automatically bumps version in Cargo.toml
2. Creates git tag
3. Generates changelog
4. Creates GitHub release
5. Triggers build workflow for release artifacts

## Commands

### Using GitHub CLI
```bash
# Create PR to develop
gh pr create --base develop

# Trigger release (from GitHub Actions UI or)
gh workflow run promote-to-main.yml -f version_bump=minor

# View workflows
gh workflow list
```

### Git Commands
```bash
# Update your feature branch with latest develop
git checkout develop
git pull origin develop
git checkout feature/my-feature
git merge develop

# Clean up old branches
git branch -d feature/my-feature
git push origin --delete feature/my-feature
```

## Best Practices

1. **Always branch from `develop`** for new features
2. **Keep PRs small and focused** on a single feature/fix
3. **Write meaningful commit messages** following conventional commits
4. **Run tests locally** before pushing
5. **Delete feature branches** after merging
6. **Update develop frequently** to avoid conflicts

## Commit Message Convention

Follow conventional commits format:
- `feat:` New feature
- `fix:` Bug fix
- `docs:` Documentation changes
- `style:` Code style changes (formatting)
- `refactor:` Code refactoring
- `perf:` Performance improvements
- `test:` Test additions/changes
- `chore:` Build process or auxiliary tool changes

Example:
```
feat: add dark mode support
fix: resolve memory leak in file watcher
docs: update API documentation
```