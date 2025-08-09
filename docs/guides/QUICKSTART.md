# Cortex Quick Start Guide

## Running Cortex

```bash
# Run from the build directory
./target/release/cortex

# Or if installed
cortex
```

## Basic Navigation

Once Cortex is running, you'll see:
- **Two panels** showing directory contents
- **Active panel** highlighted in cyan
- **Status bar** at the bottom showing file info

## Essential Commands

| Key | Action |
|-----|--------|
| `↑/↓` | Navigate files |
| `Enter` | Enter directory |
| `Tab` | Switch panels |
| `Space` | Mark file |
| `Ctrl+H` | Show hidden files |
| `Ctrl+Q` | Quit |

## First Steps

1. **Launch Cortex**: `./target/release/cortex`
2. **Navigate**: Use arrow keys to move around
3. **Enter directories**: Press Enter on a directory
4. **Switch panels**: Press Tab
5. **Mark files**: Press Space to select files
6. **Exit**: Press Ctrl+Q

## Features Implemented

✅ Dual-panel orthodox file manager interface
✅ Keyboard navigation
✅ File type color coding
✅ Directory navigation
✅ File marking/selection
✅ Hidden files toggle
✅ Status bar with file information
✅ Cross-platform support
✅ Plugin system architecture
✅ High-performance Rust implementation

## Next Steps

To continue development, you can:
1. Add file operations (F5 copy, F6 move, F8 delete)
2. Implement search functionality
3. Add file viewer (F3)
4. Create configuration system
5. Add theme support
6. Implement archive support

The architecture is fully set up to easily add these features!