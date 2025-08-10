# Development Guide

## Quick Start

Run Cortex in development mode with automatic rebuild:

```bash
./dev.sh
```

This will watch for file changes, rebuild automatically, and restart Cortex.

## Development Modes

### Default Mode (cargo-watch)
```bash
./dev.sh
```
- Uses `cargo-watch` for fastest rebuild
- Automatically installs cargo-watch if not present
- Clears terminal and rebuilds on file save
- Best for most development

### Simple Mode (no dependencies)
```bash
./dev.sh --simple
```
- No external tools required
- Checks for file changes every 2 seconds
- Rebuilds when you exit Cortex after making changes
- Good for systems without cargo-watch

### Hot Reload Mode
```bash
./dev.sh --hot
```
- Rebuilds in the background while Cortex is running
- Shows notification when rebuild is complete
- Exit Cortex to restart with new code
- Plays sound on macOS when rebuild completes

### Release Mode
Add `--release` to any mode for optimized builds:
```bash
./dev.sh --release
./dev.sh --hot --release
```

## Keyboard Shortcuts in Cortex

While developing, these shortcuts are useful:

- **Ctrl+Q** - Quit Cortex (to reload after changes)
- **F1** - Show help
- **F9** - Cycle through themes
- **F10** - Toggle random theme mode

## Tips

1. **Fast iteration**: Use default mode for quickest feedback
2. **Background work**: Use `--hot` mode to keep testing while code rebuilds
3. **No dependencies**: Use `--simple` mode on minimal setups
4. **Performance testing**: Use `--release` mode to test optimized performance

## Project Structure

The dev script watches these directories:
- `cortex-core/` - Core functionality
- `cortex-tui/` - Terminal UI components
- `cortex-cli/` - Main application
- `cortex-plugins/` - Plugin system
- `cortex-platform/` - Platform-specific code
- `cortex-updater/` - Update system

## Troubleshooting

If the dev script isn't working:

1. **Permission denied**: Run `chmod +x dev.sh`
2. **cargo-watch issues**: Try `--simple` mode
3. **Build failures**: Check `/tmp/cortex-build.log` in hot mode
4. **Terminal issues**: Ensure you're in a proper terminal (not VSCode's output panel)

## Manual Development

If you prefer manual control:

```bash
# Build only
cargo build

# Build and run
cargo run

# Build optimized version
cargo build --release

# Run tests
cargo test

# Format code
cargo fmt

# Check for issues
cargo clippy
```