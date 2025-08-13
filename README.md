# Cortex - Modern Orthodox File Manager

A fast, cross-platform orthodox file manager inspired by Far Manager, with a modern terminal UI.

## 📚 Documentation

All documentation is organized in the [`docs/`](./docs/) directory:
- [Quick Start Guide](./docs/guides/QUICKSTART.md)
- [Architecture & Planning](./docs/cortex-architecture-plan.md)
- [Development Guide](./docs/development/DEVELOPMENT.md)
- [Full Documentation Index](./docs/README.md)

## Features

- **AI-Powered Assistant** - Built-in AI chat for intelligent file management help (Press Ctrl+A)
- **Dual-panel interface** - Classic orthodox file manager layout
- **Keyboard-driven** - Efficient navigation with extensive keyboard shortcuts
- **Cross-platform** - Works on Windows, macOS, and Linux
- **High performance** - Built with Rust for speed and reliability
- **Plugin system** - Extend functionality with Lua plugins
- **Modern TUI** - Clean, responsive terminal interface

## Installation

### Quick Install

#### Ubuntu/Debian
```bash
# Download the .deb package from releases
wget https://github.com/trinverse/cortex/releases/download/v0.1.0/cortex_0.1.0_amd64.deb
sudo dpkg -i cortex_0.1.0_amd64.deb
```

#### Homebrew (macOS/Linux)
```bash
# Add the tap
brew tap trinverse/cortex
brew install cortex
```

### From Source

```bash
# Clone the repository
git clone https://github.com/trinverse/cortex.git
cd cortex

# Build with cargo
cargo build --release

# Run the application
./target/release/cortex
```

### Other Platforms

#### Arch Linux (AUR)
```bash
yay -S cortex  # Coming soon
```

#### Snap Store
```bash
snap install cortex  # Coming soon
```

#### Windows
Download the MSI installer from the [releases page](https://github.com/trinverse/cortex/releases)
```

## Usage

```bash
# Launch in current directory
cortex

# Launch in specific directory
cortex /path/to/directory

# Show version
cortex --version
```

### AI Assistant

Cortex comes with an **AI assistant built-in**! Press `Ctrl+A` to open the AI chat and get help with:
- File organization strategies
- Complex file operations
- Terminal commands
- And more!

The AI works out-of-the-box with no setup required. For the best experience, get your own free API key from [Groq](https://console.groq.com) (takes 1 minute).

## Keyboard Shortcuts

### Navigation
- `↑/↓` - Move selection up/down
- `←/→` - Navigate directories
- `Enter` - Enter directory / execute file
- `Backspace` - Go to parent directory
- `Tab` - Switch between panels
- `Home/End` - Jump to first/last item
- `PageUp/PageDown` - Scroll page

### File Operations
- `F5` - Copy selected files
- `F6` - Move/rename files
- `F7` - Create directory
- `F8` - Delete files
- `Space` - Mark/unmark file
- `Ctrl+A` - Mark all
- `Ctrl+U` - Unmark all

### View Options
- `Ctrl+H` - Toggle hidden files
- `Alt+1` - Sort by name
- `Alt+2` - Sort by size
- `Alt+3` - Sort by date
- `Alt+4` - Sort by extension

### Other
- `F1` - Help
- `Ctrl+R` - Refresh panels
- `Ctrl+Q` - Quit

## Configuration

Configuration file location:
- Linux/macOS: `~/.config/cortex/config.toml`
- Windows: `%APPDATA%\cortex\config.toml`

Example configuration:

```toml
[general]
show_hidden = false
confirm_delete = true

[panels]
default_sort = "name"

[colors]
selection_bg = "blue"
directory_fg = "cyan"
```

## Plugin Development

Create Lua plugins to extend Cortex functionality:

```lua
-- ~/.config/cortex/plugins/example.lua
plugin = {
    name = "Example Plugin",
    version = "1.0.0",
    author = "Your Name",
    description = "Example plugin for Cortex"
}

function initialize()
    print("Plugin initialized")
end

function execute(command, args)
    if command == "hello" then
        return "Hello from plugin!"
    end
    return ""
end
```

## Building from Source

Requirements:
- Rust 1.70 or higher
- Cargo

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run
```

## Architecture

Cortex is built with a modular architecture:

- **cortex-core** - Core file system operations and state management
- **cortex-tui** - Terminal UI components using Ratatui
- **cortex-plugins** - Plugin system with Lua support
- **cortex-cli** - Main application entry point

## Contributing

Contributions are welcome! Please feel free to submit pull requests.

## License

MIT License - See LICENSE file for details

## Acknowledgments

- Inspired by [Far Manager](https://www.farmanager.com/)
- Built with [Ratatui](https://github.com/ratatui/ratatui)
- Plugin system powered by [mlua](https://github.com/mlua-rs/mlua)