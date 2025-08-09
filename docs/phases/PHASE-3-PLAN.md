# Cortex Phase 3 Implementation Plan

## Overview
Phase 3 focuses on advanced file management features that Far Manager users expect, building upon the solid foundation of dual panels, file operations, and command-line integration established in Phases 1 and 2.

## Phase 3 Goals
1. Implement file viewing and editing capabilities (F3/F4)
2. Add comprehensive search and filter functionality (Alt+F7)
3. Introduce archive support (Enter to browse, operations on archives)
4. Add file attributes and permissions management
5. Implement quick view panel (Ctrl+Q)
6. Add file comparison features
7. Activate and enhance the plugin system
8. Implement network/remote file system support

## Detailed Implementation Plan

### 3.1 File Viewing System (F3)
**Priority: HIGH**
**Estimated effort: 3-4 days**

#### Features:
- Internal viewer for text files with syntax highlighting
- Binary/hex viewer mode
- Large file support with streaming
- Search within viewer (F7 in viewer)
- Line wrapping toggle
- Encoding detection and switching (UTF-8, ASCII, etc.)

#### Implementation:
```rust
// New modules needed:
// cortex-viewer/src/lib.rs
pub struct FileViewer {
    pub view_mode: ViewMode,
    pub encoding: Encoding,
    pub search_pattern: Option<String>,
    pub line_wrap: bool,
    pub scroll_position: usize,
}

pub enum ViewMode {
    Text,
    Binary,
    Hex,
}
```

#### Key bindings in viewer:
- `F3/Esc` - Exit viewer
- `F7` - Search
- `F8` - Toggle wrap/unwrap
- `Shift+F8` - Switch encoding
- `F4` - Switch to edit mode

### 3.2 File Editing System (F4)
**Priority: HIGH**
**Estimated effort: 4-5 days**

#### Features:
- Basic text editor with syntax highlighting
- Multi-line editing with proper cursor management
- Undo/redo functionality
- Find and replace (Ctrl+F, Ctrl+H)
- Save with backup option
- Integration with external editors ($EDITOR)

#### Implementation:
```rust
// cortex-editor/src/lib.rs
pub struct FileEditor {
    pub buffer: TextBuffer,
    pub cursor: CursorPosition,
    pub selection: Option<Selection>,
    pub undo_stack: Vec<EditOperation>,
    pub syntax_highlighter: Option<SyntaxHighlighter>,
    pub modified: bool,
}
```

#### Configuration:
```toml
[editor]
external_editor = "vim"  # or "nano", "emacs", etc.
use_internal = true      # Use internal editor by default
create_backup = true
syntax_highlighting = true
```

### 3.3 Search System (Alt+F7)
**Priority: HIGH**
**Estimated effort: 3-4 days**

#### Features:
- Find files by name (with wildcards and regex)
- Find text in files
- Search filters (size, date, attributes)
- Search results panel
- Background search with progress
- Search history

#### Implementation:
```rust
// cortex-core/src/search.rs
pub struct SearchEngine {
    pub search_type: SearchType,
    pub pattern: String,
    pub case_sensitive: bool,
    pub whole_words: bool,
    pub include_hidden: bool,
    pub filters: SearchFilters,
}

pub enum SearchType {
    FileName(PatternType),
    FileContent(String),
    Combined { name: String, content: String },
}

pub struct SearchFilters {
    pub min_size: Option<u64>,
    pub max_size: Option<u64>,
    pub date_from: Option<SystemTime>,
    pub date_to: Option<SystemTime>,
    pub file_types: Vec<FileType>,
}
```

#### Search dialog UI:
- Text input for file mask
- Text input for containing text
- Checkboxes for options
- Date/size filter inputs
- Search scope (current dir, subdirs, drives)

### 3.4 Quick Filter (Ctrl+F)
**Priority: MEDIUM**
**Estimated effort: 2 days**

#### Features:
- Real-time filtering of panel contents
- Filter by name pattern
- Filter indicator in panel
- Quick clear (Esc)

#### Implementation:
```rust
// Add to PanelState
pub struct PanelState {
    // ... existing fields
    pub filter: Option<String>,
    pub filtered_entries: Vec<FileEntry>,
}
```

### 3.5 Archive Support
**Priority: HIGH**
**Estimated effort: 5-6 days**

#### Features:
- Browse archives as directories (ZIP, TAR, GZ, RAR, 7Z)
- Extract files (F5 from archive)
- Add files to archive (F5 to archive)
- Create new archives
- Archive operations progress

#### Implementation:
```rust
// cortex-archive/src/lib.rs
pub trait ArchiveHandler {
    fn can_handle(&self, path: &Path) -> bool;
    fn list_contents(&self, path: &Path) -> Result<Vec<ArchiveEntry>>;
    fn extract(&self, archive: &Path, files: Vec<String>, dest: &Path) -> Result<()>;
    fn compress(&self, files: Vec<PathBuf>, archive: &Path) -> Result<()>;
}

// Virtual file system for archives
pub struct VirtualPath {
    pub archive_path: PathBuf,
    pub internal_path: PathBuf,
}
```

#### Archive types support:
- ZIP (using zip crate)
- TAR/TAR.GZ (using tar crate)
- RAR (read-only, using unrar)
- 7Z (using sevenz-rust)

### 3.6 File Attributes & Permissions
**Priority: MEDIUM**
**Estimated effort: 2-3 days**

#### Features:
- View/edit file attributes dialog (Ctrl+A)
- Unix permissions editor
- Windows attributes support
- Batch attribute changes
- Extended attributes support

#### Implementation:
```rust
// cortex-core/src/attributes.rs
pub struct FileAttributes {
    #[cfg(unix)]
    pub permissions: UnixPermissions,
    #[cfg(windows)]
    pub attributes: WindowsAttributes,
    pub readonly: bool,
    pub hidden: bool,
    pub system: bool,
}

pub struct AttributesDialog {
    pub files: Vec<PathBuf>,
    pub attributes: FileAttributes,
    pub apply_recursively: bool,
}
```

### 3.7 Quick View Panel (Ctrl+Q)
**Priority: MEDIUM**
**Estimated effort: 2-3 days**

#### Features:
- Toggle panel to quick view mode
- Show file preview in inactive panel
- Preview text files (first N lines)
- Show image thumbnails (if possible)
- Display file information for binaries
- Directory statistics

#### Implementation:
```rust
pub enum PanelMode {
    Normal,
    QuickView,
    Tree,
    Info,
}

pub struct QuickViewPanel {
    pub file: Option<PathBuf>,
    pub preview_type: PreviewType,
    pub content: PreviewContent,
}
```

### 3.8 File Comparison (Ctrl+F3)
**Priority: LOW**
**Estimated effort: 3-4 days**

#### Features:
- Compare two files side by side
- Highlight differences
- Compare by content or date/size
- Directory comparison mode
- Synchronization suggestions

#### Implementation:
```rust
// cortex-compare/src/lib.rs
pub struct FileComparator {
    pub left_file: PathBuf,
    pub right_file: PathBuf,
    pub diff_algorithm: DiffAlgorithm,
    pub ignore_whitespace: bool,
}

pub struct CompareDialog {
    pub differences: Vec<Difference>,
    pub sync_direction: SyncDirection,
}
```

### 3.9 Enhanced Plugin System
**Priority: MEDIUM**
**Estimated effort: 3-4 days**

#### Features:
- Plugin discovery and auto-loading
- Plugin configuration UI
- Plugin hotkeys binding
- Plugin menu (F11)
- API extensions for file operations
- Event hooks for plugins

#### Implementation:
```rust
// Enhance existing plugin system
pub trait PluginAPI {
    fn register_command(&mut self, cmd: Command);
    fn register_menu_item(&mut self, item: MenuItem);
    fn register_hotkey(&mut self, key: KeyBinding, action: Action);
    fn hook_event(&mut self, event: EventType, handler: EventHandler);
}

pub struct PluginMenu {
    pub items: Vec<PluginMenuItem>,
    pub selected: usize,
}
```

#### Plugin API extensions:
- File system operations
- Panel manipulation
- Dialog creation
- Configuration access

### 3.10 Network/Remote Support (Future)
**Priority: LOW**
**Estimated effort: 7-10 days**

#### Features:
- FTP/SFTP client
- SMB/CIFS support
- WebDAV support
- Cloud storage integration (S3, etc.)

#### Implementation:
```rust
// cortex-remote/src/lib.rs
pub trait RemoteFileSystem {
    async fn connect(&mut self, config: ConnectionConfig) -> Result<()>;
    async fn list(&self, path: &Path) -> Result<Vec<RemoteEntry>>;
    async fn download(&self, remote: &Path, local: &Path) -> Result<()>;
    async fn upload(&self, local: &Path, remote: &Path) -> Result<()>;
}
```

## Implementation Schedule

### Week 1-2: Core Viewing/Editing
- [ ] File viewer (F3) - 3 days
- [ ] File editor (F4) - 4 days
- [ ] Integration and testing - 1 day

### Week 3: Search and Filter
- [ ] Search system (Alt+F7) - 3 days
- [ ] Quick filter (Ctrl+F) - 2 days
- [ ] Search results panel - 1 day

### Week 4-5: Archives and Attributes
- [ ] Archive support - 5 days
- [ ] File attributes dialog - 2 days
- [ ] Testing and refinement - 1 day

### Week 6: Advanced Features
- [ ] Quick view panel - 2 days
- [ ] Enhanced plugin system - 3 days
- [ ] File comparison - 3 days (if time permits)

## Technical Considerations

### Performance
- Use async/await for all I/O operations
- Implement virtual scrolling for large directories
- Stream large files in viewer/editor
- Background operations for search and archive operations

### Cross-Platform
- Abstract platform-specific code (permissions, attributes)
- Test on Windows, macOS, and Linux
- Handle path separators correctly
- Support platform-specific features conditionally

### UI/UX
- Maintain consistency with Far Manager keybindings
- Provide visual feedback for long operations
- Implement proper error handling and recovery
- Support for terminal resize during operations

### Testing Strategy
- Unit tests for core functionality
- Integration tests for file operations
- Manual testing checklist for UI features
- Performance benchmarks for large files/directories

## Dependencies to Add

```toml
# Cargo.toml additions
[dependencies]
# Syntax highlighting
syntect = "5.0"

# Archive support
zip = "0.6"
tar = "0.4"
flate2 = "1.0"
sevenz-rust = "0.2"

# Text diffing
similar = "2.2"

# Enhanced async support
futures = "0.3"

# Network protocols (future)
async-ftp = "6.0"
ssh2 = "0.9"
```

## Success Criteria

1. **File Viewing/Editing**: Users can view and edit files without leaving Cortex
2. **Search**: Fast, comprehensive search with filters and background operation
3. **Archives**: Seamless browsing and manipulation of common archive formats
4. **Plugin System**: Working plugins that extend functionality
5. **Performance**: Operations remain responsive even with large files/directories
6. **Stability**: No crashes or data loss during operations
7. **Documentation**: Complete user guide and plugin development documentation

## Risk Mitigation

1. **Complexity**: Start with basic features, iterate and enhance
2. **Performance**: Profile and optimize critical paths early
3. **Cross-platform issues**: Test continuously on all platforms
4. **Large files**: Implement streaming and chunking from the start
5. **Plugin security**: Sandbox plugin execution, limit API access

## Post-Phase 3 Roadmap

### Phase 4: Advanced Features
- Multi-tab interface
- Session management
- Scripting and macros
- Advanced file synchronization
- Built-in terminal

### Phase 5: Modern Enhancements
- GUI mode (optional)
- Cloud integration
- Version control integration
- AI-powered features
- Mobile companion app

## Notes

This plan prioritizes features that Far Manager users expect most, while laying groundwork for future enhancements. The modular architecture allows for parallel development of features once the core viewer/editor infrastructure is in place.

The implementation order can be adjusted based on user feedback and technical dependencies. Each feature should be developed with thorough testing and documentation.