# Cortex Development Guide

## Hot Reload & Fast Development

While Rust doesn't support true hot-reload like interpreted languages, Cortex provides several development features for rapid iteration:

## 1. Auto-Rebuild on File Changes

### Basic Setup
```bash
# Install cargo-watch
cargo install cargo-watch

# Run auto-rebuild
./dev.sh
```

### Advanced Development Server
```bash
# Debug build with auto-rebuild
./scripts/dev-server.sh debug

# Release build with auto-rebuild
./scripts/dev-server.sh release

# Auto-run after build
./scripts/dev-server.sh debug true
```

## 2. Configuration Hot-Reload

Cortex can reload configuration files without restarting:

```toml
# ~/.config/cortex/config.toml
[general]
show_hidden = true
editor = "nvim"

[colors]
selection_bg = "magenta"
directory_fg = "blue"
```

Changes to this file are detected and applied automatically!

## 3. Plugin Hot-Reload

Lua plugins can be modified and reloaded without restarting:

```lua
-- plugins/my-plugin.lua
plugin = {
    name = "My Plugin",
    version = "1.0.0"
}

function execute(command, args)
    -- Modify this function and reload!
    return "Current time: " .. os.date()
end
```

Press `Ctrl+R` in Cortex to reload plugins.

## 4. Faster Build Times

### Use Debug Builds During Development
```bash
# Fast debug build (5-10 seconds)
cargo build

# Instead of release build (30-60 seconds)
cargo build --release
```

### Incremental Compilation
Already enabled in `Cargo.toml` for faster rebuilds.

### Use mold Linker (Linux)
```bash
# Install mold for faster linking
sudo apt install mold

# Use with cargo
mold -run cargo build
```

### Use lld Linker (macOS/Windows)
```bash
# Install LLVM
brew install llvm  # macOS

# Add to ~/.cargo/config.toml
[target.x86_64-apple-darwin]
rustflags = ["-C", "link-arg=-fuse-ld=lld"]
```

## 5. Development Workflow

### Terminal 1: Auto-rebuild
```bash
./scripts/dev-server.sh debug
```

### Terminal 2: Run Cortex
```bash
# Watch for "Build complete!" then:
./target/debug/cortex
```

### Terminal 3: Edit Code
```bash
# Your favorite editor
vim cortex-cli/src/main.rs
```

## 6. Live Development Features

### Command Mode Testing
In Cortex, press `Ctrl+O` and run:
```bash
# Reload configuration
:reload-config

# Test plugin commands
:plugin my-plugin test

# Run build from within Cortex
:!cargo build
```

## 7. Testing Changes Quickly

### Unit Tests (Fast)
```bash
# Run specific test
cargo test test_name

# Run tests for a package
cargo test -p cortex-core

# Watch mode for tests
cargo watch -x test
```

### Integration Testing
```bash
# Create test script
./test-integration.sh
```

## 8. Performance Profiling

```bash
# Build with profiling
cargo build --release
perf record --call-graph=dwarf ./target/release/cortex
perf report
```

## Tips for Fast Development

1. **Use Debug Mode**: 5-10x faster builds than release
2. **Modify Plugins**: Instant reload without rebuilding
3. **Edit Config**: Changes apply immediately
4. **Small Changes**: Make incremental changes for faster rebuilds
5. **Parallel Builds**: `cargo build -j 8` (use multiple cores)
6. **Cache Dependencies**: Dependencies only build once

## Hot-Keys During Development

While running Cortex:
- `Ctrl+R`: Refresh/reload (plugins, file lists)
- `Ctrl+O`: Command mode for testing
- `F1`: Check help for new shortcuts

## Example Development Session

```bash
# Terminal 1: Start watcher
./scripts/dev-server.sh debug

# Terminal 2: Run Cortex
./target/debug/cortex

# Make changes to code...
# See "Build complete!" in Terminal 1
# Press Ctrl+Q in Terminal 2 to quit
# Restart Cortex with up arrow + Enter

# For config changes:
# Edit ~/.config/cortex/config.toml
# Changes apply immediately!
```

## Build Time Comparison

- **Debug Build**: ~5-10 seconds
- **Release Build**: ~30-60 seconds
- **Incremental Debug**: ~2-5 seconds
- **Config Reload**: Instant
- **Plugin Reload**: Instant

## Troubleshooting

### Slow Builds?
1. Use debug mode during development
2. Enable incremental compilation (already done)
3. Use faster linker (mold/lld)
4. Upgrade to latest Rust version

### Changes Not Applying?
1. Make sure file is saved
2. Check watcher output for errors
3. Restart Cortex after rebuild
4. For plugins: Press Ctrl+R to reload

### Memory Usage High?
1. Close other Rust projects
2. Run `cargo clean` occasionally
3. Limit parallel jobs: `cargo build -j 2`