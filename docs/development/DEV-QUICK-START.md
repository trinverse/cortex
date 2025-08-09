# 🚀 Quick Development Start

## Option 1: SIMPLEST - One Command (Recommended!)
```bash
./dev.sh
```
This watches files, rebuilds, and runs Cortex automatically!
- Make code changes
- Save file
- Wait 5-10 seconds for rebuild
- Cortex restarts automatically
- Press Ctrl+Q to quit and it rebuilds/restarts

## Option 2: Two Terminals
**Terminal 1:**
```bash
./run-dev.sh
```
Watches and rebuilds on file changes.

**Terminal 2:**
```bash
./target/debug/cortex
```
Run Cortex. Press Ctrl+Q to quit, then ↑ + Enter to restart with new code.

## Option 3: Auto-Restart
**Terminal 1:**
```bash
./run-dev.sh
```

**Terminal 2:**
```bash
./test-dev.sh
```
Automatically restarts Cortex when rebuild completes.

## Build Times
- **Debug mode**: 5-10 seconds (what we use for development)
- **Release mode**: 30-60 seconds (only for final testing)

## Making Changes

1. **Edit code** in your favorite editor:
```bash
vim cortex-cli/src/main.rs
```

2. **Save the file**

3. **Wait for rebuild** (you'll see "✅ Ready to test!" or Cortex restarts)

4. **Test your changes** immediately!

## Tips
- Always use debug builds during development (5-10x faster)
- Keep `./dev.sh` running in a terminal
- Make small, incremental changes
- Config changes (`~/.config/cortex/config.toml`) apply instantly - no rebuild!

## Example Session
```bash
# Start development mode
./dev.sh

# In another terminal/tab, edit code
code .  # or vim, emacs, etc.

# Make changes, save, and Cortex auto-restarts!
# Press Ctrl+Q in Cortex to trigger rebuild/restart
```

That's it! You're ready to develop! 🎉