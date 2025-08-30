# Getting Started with Cortex File Manager

Welcome to Cortex, a powerful and modern orthodox file manager for the terminal! This guide will help you get up and running quickly.

## Table of Contents
- [Installation](#installation)
- [First Launch](#first-launch)
- [Basic Navigation](#basic-navigation)
- [Essential Operations](#essential-operations)
- [Keyboard Shortcuts](#keyboard-shortcuts)
- [Configuration](#configuration)
- [Tips & Tricks](#tips--tricks)

## Installation

### From Source (Recommended)
```bash
# Clone the repository
git clone https://github.com/trinverse/cortex.git
cd cortex

# Build the release version
cargo build --release

# Run Cortex
./target/release/cortex
```

### Using Cargo
```bash
cargo install cortex
```

### Package Managers (Coming Soon)
```bash
# macOS
brew install cortex

# Ubuntu/Debian
apt install cortex

# Windows
winget install cortex
```

## First Launch

When you first launch Cortex, you'll see:
- **Dual Panels**: Left and right file panels showing directory contents
- **Command Line**: Bottom area for typing commands
- **Status Bar**: Shows current path and file information
- **Function Keys**: F1-F10 shortcuts displayed at the bottom

```
┌─────────────────────────┬─────────────────────────┐
│ LEFT PANEL              │ RIGHT PANEL             │
│ /home/user              │ /home/user/Documents    │
│                         │                         │
│ .. [DIR]                │ .. [DIR]                │
│ Documents [DIR]         │ project.txt      1.2KB  │
│ Downloads [DIR]         │ notes.md         3.4KB  │
│ Pictures [DIR]          │ todo.txt         567B   │
│                         │                         │
└─────────────────────────┴─────────────────────────┘
[Command Line]
F1 Help F2 Menu F3 View F4 Edit F5 Copy F6 Move F7 Mkdir F8 Delete F9 Config F10 Quit
```

## Basic Navigation

### Moving Around
- **↑/↓**: Move selection up/down
- **←/→**: Go to parent directory / Enter selected directory
- **Tab**: Switch between panels
- **Enter**: Open file or enter directory
- **Backspace**: Go to parent directory

### Page Navigation
- **PageUp/PageDown**: Move one page up/down
- **Home**: Go to first item
- **End**: Go to last item
- **Ctrl+Home**: Go to root directory
- **Alt+Left/Right**: Navigate history back/forward

### Mouse Support
- **Click**: Select file or switch panel
- **Double-click**: Open file or enter directory
- **Right-click**: Open context menu
- **Scroll wheel**: Scroll through files

## Essential Operations

### Selecting Files
- **Space** or **Insert**: Mark/unmark current file
- **Ctrl+A**: Select all files
- **Ctrl+U**: Unmark all files
- **+**: Select by pattern (e.g., *.txt)
- **-**: Deselect all
- **\***: Invert selection

### File Operations
- **F5**: Copy selected files to other panel
- **F6**: Move selected files to other panel
- **F7**: Create new directory
- **F8**: Delete selected files
- **Shift+F4**: Create new file
- **Shift+F6**: Rename file

### Viewing and Editing
- **F3**: View file (read-only)
- **F4**: Edit file
- **Alt+Enter**: Show file properties

### Searching
- **Alt+F7**: Advanced search
- **Ctrl+F**: Quick filter in current panel
- **Ctrl+L**: Clear filter
- **/**: Start command or search

## Keyboard Shortcuts

### Essential Shortcuts

| Shortcut | Action |
|----------|--------|
| **F1** | Help |
| **F2** | Command menu |
| **F3** | View file |
| **F4** | Edit file |
| **F5** | Copy |
| **F6** | Move/Rename |
| **F7** | Create directory |
| **F8** | Delete |
| **F9** | Settings |
| **F10** | Quit |

### Quick Actions

| Shortcut | Action |
|----------|--------|
| **Ctrl+R** | Refresh panels |
| **Ctrl+H** | Toggle hidden files |
| **Ctrl+O** | Open terminal here |
| **Ctrl+P** | Command palette |
| **Ctrl+Q** | Quick quit |
| **Ctrl+S** | Sync panels |
| **Ctrl+U** | Swap panels |

### Quick Directories

| Shortcut | Action |
|----------|--------|
| **Ctrl+1..9** | Go to quick directory 1-9 |
| **Alt+1..9** | Go to bookmark 1-9 |
| **Ctrl+D** | Add bookmark |
| **Ctrl+B** | Show bookmarks |

### Advanced Features

| Shortcut | Action |
|----------|--------|
| **Alt+F7** | Advanced search |
| **Ctrl+Shift+F** | Find in files |
| **Ctrl+Shift+C** | Compare files |
| **Ctrl+Shift+D** | Find duplicates |
| **Ctrl+Z** | Undo |
| **Ctrl+Y** | Redo |

## Vim Mode

Cortex supports Vim-style navigation for power users.

### Enabling Vim Mode
Press **:** to enter command mode, then type:
```
:set vim
```

### Vim Navigation
- **h/j/k/l**: Left/Down/Up/Right
- **g**: Go to top
- **G**: Go to bottom
- **i**: Enter insert mode
- **v**: Visual selection mode
- **Esc**: Return to normal mode
- **/**: Search
- **n/N**: Next/Previous search result
- **y**: Yank (copy)
- **p**: Paste
- **d**: Delete
- **u**: Undo
- **Ctrl+R**: Redo

## Configuration

### Configuration File
Cortex stores its configuration in:
- **Linux/macOS**: `~/.config/cortex/config.toml`
- **Windows**: `%APPDATA%\cortex\config.toml`

### Example Configuration
```toml
[general]
show_hidden = false
confirm_delete = true
auto_reload = true

[panels]
default_path = "~"
sort_by = "name" # name, size, modified, extension
sort_reverse = false

[colors]
theme = "dark" # dark, light, custom

[shortcuts]
# Custom key bindings
copy = "F5"
move = "F6"
delete = "Delete"

[cache]
max_entries = 1000
ttl_seconds = 300
max_memory_mb = 100

[performance]
enable_cache = true
virtual_scrolling = true
lazy_loading = true
```

### Quick Settings
Press **F9** to open the settings dialog and modify options interactively.

## Command Line

### Built-in Commands
- **cd [path]**: Change directory
- **mkdir [name]**: Create directory
- **touch [name]**: Create file
- **rm [file]**: Delete file
- **cp [src] [dst]**: Copy file
- **mv [src] [dst]**: Move/rename file
- **find [pattern]**: Search for files
- **filter [pattern]**: Filter current view
- **!command**: Execute shell command

### Command Examples
```bash
# Change to home directory
cd ~

# Create multiple directories
mkdir project docs tests

# Find all Python files
find *.py

# Execute shell command
!git status

# Filter view to show only text files
filter *.txt
```

## Tips & Tricks

### 1. Quick Navigation
- Use **Ctrl+1-9** to set up quick access directories
- Press **Alt+Left/Right** to navigate through history
- Use **Tab** to quickly switch between panels

### 2. Efficient Selection
- Use patterns with **+** to select multiple files quickly
- **\*** inverts selection - useful for "select all except"
- Hold **Shift** while navigating to select range

### 3. Productivity Boosters
- **F2** opens command palette for quick access to all commands
- Use **/** to quickly start typing commands
- **Ctrl+P** for fuzzy command search

### 4. Archive Support
- Navigate into ZIP files like directories
- Copy files directly from archives
- Search within archives using Alt+F7

### 5. Remote Access
- Use `/sftp` or `/ftp` commands to connect to remote servers
- Navigate remote directories like local ones
- Copy files between local and remote seamlessly

### 6. Performance Tips
- Directory contents are cached for 5 minutes
- Use **Ctrl+R** to force refresh if needed
- Virtual scrolling handles directories with 100,000+ files smoothly

## Advanced Features

### File Monitoring
Cortex automatically monitors directories for changes and updates the view in real-time.

### Search Capabilities
- **Name search**: Wildcards, regex, exact match
- **Content search**: Search within file contents
- **Size/Date filters**: Find files by size or modification date
- **Archive search**: Search within ZIP files

### Plugin System
Cortex supports Lua plugins for extending functionality:
```lua
-- ~/.config/cortex/plugins/example.lua
function on_file_selected(file)
    if file.extension == "md" then
        cortex.show_message("Markdown file selected!")
    end
end
```

### Network Features
- **SFTP/FTP support**: Browse remote servers
- **Authentication**: Password and key-based auth
- **Bookmarks**: Save remote connections

## Troubleshooting

### Common Issues

**Q: Cortex doesn't start**
A: Check that your terminal supports 256 colors and UTF-8

**Q: Colors look wrong**
A: Try changing the theme in settings (F9)

**Q: Performance is slow**
A: Enable caching in settings, check if virtual scrolling is enabled

**Q: Can't see hidden files**
A: Press Ctrl+H to toggle hidden files

**Q: Mouse doesn't work**
A: Ensure your terminal emulator supports mouse events

## Getting Help

- **F1**: Built-in help
- **GitHub**: [github.com/trinverse/cortex](https://github.com/trinverse/cortex)
- **Documentation**: [/docs](https://github.com/trinverse/cortex/tree/main/docs)

## Next Steps

Now that you know the basics, explore:
- [Keyboard Shortcuts Reference](./KEYBOARD-SHORTCUTS.md)
- [Advanced Features Guide](./ADVANCED-FEATURES.md)
- [Configuration Guide](./CONFIGURATION.md)
- [Plugin Development](./PLUGIN-DEVELOPMENT.md)

Happy file managing with Cortex! 🚀