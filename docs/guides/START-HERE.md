# 🚀 Start Development - Quick Guide

## Option 1: Auto-Rebuild Mode (Recommended)
```bash
./dev-auto.sh
```
- Automatically detects file changes
- Rebuilds when you save any .rs file
- Runs Cortex after successful build
- No external tools needed!

## Option 2: Manual Rebuild Mode
```bash
./dev-simple.sh
```
- Press Enter to rebuild and run
- Good for controlled testing
- No external tools needed!

## Option 3: Direct Commands
```bash
# Build once
cargo build

# Run Cortex
./target/debug/cortex

# Build and run
cargo build && ./target/debug/cortex
```

## Quick Test
Test that everything works:
```bash
# Build the project
cargo build

# Run Cortex
./target/debug/cortex

# You should see the file manager interface!
# Press Ctrl+Q to quit
```

## Development Workflow

1. **Start dev mode in Terminal 1:**
   ```bash
   ./dev-auto.sh
   ```

2. **Edit code in Terminal 2:**
   ```bash
   vim cortex-cli/src/main.rs
   # or use your favorite editor
   ```

3. **Save the file** - Cortex automatically rebuilds and restarts!

## Troubleshooting

### Build errors?
```bash
# Clean and rebuild
cargo clean
cargo build
```

### Permission denied?
```bash
chmod +x dev-auto.sh dev-simple.sh
```

### Still having issues?
```bash
# Update Rust
rustup update

# Check version
rustc --version  # Should be 1.70+
```

## Features You Can Test

Once Cortex is running:
- **Navigate**: Arrow keys to move around
- **Enter directories**: Enter key
- **Switch panels**: Tab key  
- **Command mode**: Ctrl+O or :
- **File operations**: F5 (copy), F6 (move), F8 (delete)
- **Help**: F1
- **Quit**: Ctrl+Q

That's it! You're ready to develop! 🎉