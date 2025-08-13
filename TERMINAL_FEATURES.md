# Embedded Terminal Features in Cortex

## Overview
Cortex now includes a fully embedded terminal emulator, similar to modern applications like Warp, providing a seamless file management and terminal experience without dependency on external terminal emulators.

## Key Features

### 1. **Cross-Platform Terminal Emulation**
- Built using `portable-pty` for cross-platform PTY (pseudo-terminal) support
- Works on Linux, macOS, and Windows
- Automatic shell detection based on the operating system
- Full VT100/xterm-256color terminal emulation using the `vt100` crate

### 2. **Shell Support**
Automatically detects and supports multiple shells:
- **Unix/Linux/macOS:**
  - Bash (default fallback)
  - Zsh
  - Fish
  - Custom shells via `$SHELL` environment variable
- **Windows:**
  - PowerShell Core (pwsh) - preferred
  - Windows PowerShell
  - Command Prompt (cmd.exe) - fallback

### 3. **Terminal Manager**
- Manages multiple terminal sessions
- Each session runs in its own PTY with isolated environment
- Session persistence and lifecycle management
- Automatic cleanup on application exit

### 4. **Terminal UI Features**
- **Multiple Tabs:** Create and manage multiple terminal sessions
- **Split Views:** 
  - Single view mode
  - Horizontal split (side-by-side terminals)
  - Vertical split (stacked terminals)
- **Tab Navigation:**
  - Visual tab bar for multiple sessions
  - Keyboard shortcuts for switching tabs
  - Active tab highlighting

### 5. **Integration with File Manager**
- Open terminals in current directory
- Launch terminals from file context
- Seamless switching between file manager and terminal
- Shared theme and configuration

## Architecture

### Components

1. **`terminal::emulator`**
   - Core PTY management
   - Input/output handling
   - Terminal state management
   - Event streaming

2. **`terminal::shell`**
   - Shell detection and configuration
   - Environment setup
   - Cross-platform shell handling

3. **`terminal::manager`**
   - Session management
   - Multi-terminal coordination
   - Resource lifecycle

4. **`terminal_view`**
   - UI rendering
   - Tab management
   - Split view handling
   - User interaction

## Usage

### Creating a Terminal Session
```rust
let mut terminal_view = TerminalView::new();
terminal_view.create_new_terminal(
    Some("My Terminal".to_string()),
    Some(current_dir),
)?;
```

### Sending Input
```rust
terminal_view.send_input("ls -la\n")?;
```

### Managing Sessions
```rust
// Switch tabs
terminal_view.next_tab();
terminal_view.prev_tab();

// Close current terminal
terminal_view.close_current_terminal()?;

// Toggle split mode
terminal_view.toggle_split_mode();
```

## Benefits

1. **No External Dependencies**
   - Works without requiring a system terminal
   - Consistent experience across platforms
   - Easier distribution and packaging

2. **Better Integration**
   - Unified theming with file manager
   - Shared configuration
   - Seamless context switching

3. **Enhanced Features**
   - Built-in multiplexing (like tmux/screen)
   - Advanced terminal features
   - Custom key bindings

4. **Portability**
   - Single binary includes everything
   - No need to detect/launch external terminals
   - Works in constrained environments

## Future Enhancements

- Terminal recording and playback
- Built-in SSH client
- Terminal sharing/collaboration
- Custom terminal themes
- Shell integration for better prompts
- Integrated terminal search
- Command history across sessions
- Terminal profiles and presets

## Technical Details

### Dependencies
- `portable-pty`: Cross-platform PTY implementation
- `vt100`: Terminal emulation and parsing
- `strip-ansi-escapes`: ANSI escape sequence handling
- `which`: Shell detection

### Platform Support
- ✅ Linux (x86_64, ARM)
- ✅ macOS (Intel, Apple Silicon)
- ✅ Windows (Windows 10+)
- ✅ BSD variants
- ✅ WSL/WSL2

### Performance
- Efficient terminal output buffering
- Async I/O for non-blocking operations
- Minimal memory overhead per session
- Fast terminal rendering with ratatui

## Comparison with External Terminal Approach

| Feature | External Terminal | Embedded Terminal |
|---------|------------------|-------------------|
| Portability | Depends on system | ✅ Built-in |
| Consistency | Varies by terminal | ✅ Uniform |
| Integration | Limited | ✅ Deep |
| Distribution | Complex | ✅ Simple |
| Features | Terminal-dependent | ✅ Controlled |
| Theming | Separate | ✅ Unified |
| Performance | IPC overhead | ✅ Direct |

The embedded terminal makes Cortex a complete, self-contained file management solution that doesn't rely on external terminal emulators, similar to modern developer tools like Warp, VS Code's integrated terminal, or JetBrains IDEs.