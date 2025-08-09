# Changelog

All notable changes to Cortex File Manager will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2025-01-10

### 🎉 Initial Release

First public release of Cortex File Manager - a modern, powerful terminal file manager built with Rust.

### ✨ Features

#### Core Functionality
- **Dual-pane interface** - Split-screen file browsing for efficient navigation
- **Keyboard-driven navigation** - Full keyboard shortcuts with vim-style bindings
- **File operations** - Copy, move, delete, rename with progress tracking
- **Directory navigation** - Quick navigation with bookmarks and history
- **Search and filter** - Advanced search with regex support and real-time filtering
- **File preview** - Built-in viewer for text files and images
- **Archive support** - Browse and extract ZIP, TAR, 7Z archives
- **Batch operations** - Select multiple files for bulk operations

#### Advanced Features
- **Plugin system** - Lua-based plugin architecture for extensibility
- **Command mode** - Execute commands with `:` prefix
- **Virtual File System** - Abstraction layer for archives and remote files
- **Smart caching** - Directory cache for improved performance
- **File monitoring** - Auto-reload on external changes
- **Configurable** - TOML-based configuration system
- **Memory efficient** - Optimized memory management with pooling

#### Platform Integration
- **Trash support** - Move to trash/recycle bin (Linux fully implemented)
- **Clipboard integration** - System clipboard copy/paste operations
- **Auto-update system** - Built-in update checker and installer
- **Package manager ready** - Configurations for Homebrew, APT, AUR, Snap

### 🏗️ Architecture
- **Modular design** - Separated into core, TUI, CLI, and platform modules
- **Async operations** - Non-blocking file operations with Tokio
- **Cross-platform** - Supports Linux, macOS, and Windows
- **Performance optimized** - Virtual scrolling, lazy loading, parallel processing

### 📦 Installation
- Binary releases for Linux, macOS, and Windows
- Package manager support (Homebrew, APT, AUR, Snap)
- Build from source with Cargo

### 🎯 Known Limitations
- SSH/SFTP temporarily disabled (OpenSSL dependency)
- Windows/macOS platform features need testing
- Some package formats in development

### 🙏 Acknowledgments
- Built with Rust and Ratatui
- Inspired by Midnight Commander, ranger, and nnn
- Community feedback and contributions welcome

---

**Note**: This is an alpha release. Please report bugs and feature requests on [GitHub Issues](https://github.com/cortex-fm/cortex/issues).