# Phase 4 Implementation Plan - Advanced Power User Features

## Executive Summary

Phase 4 focuses on transforming Cortex into a truly powerful file manager that matches and exceeds Far Manager's capabilities. This phase emphasizes advanced search, archive support, network capabilities, and a robust plugin ecosystem while maintaining the fast, keyboard-driven experience power users expect.

## Architecture Foundation

Building on the existing modular architecture:
- **cortex-core**: Extended with search engine, archive handlers, VFS layer
- **cortex-tui**: New dialogs for search, archive browsing, settings
- **cortex-plugins**: Enhanced API with hooks and events
- **cortex-vfs**: New module for Virtual File System abstraction
- **cortex-search**: New module for advanced search capabilities

## Core Features for Phase 4

### 1. Advanced Search System (Alt+F7)
**Priority: HIGH** - Essential for power users

#### Features
- **Multi-criteria search**
  - Name patterns (wildcards, regex)
  - Content search with context preview
  - Size/date ranges
  - File attributes
- **Search scopes**
  - Current directory
  - Subdirectories (with depth limit)
  - Multiple directories
  - Indexed search for instant results
- **Search results panel**
  - Navigate results like a file panel
  - Preview matches with highlighting
  - Direct file operations on results
  - Export results to file

#### Implementation
```rust
// New module: cortex-search/src/lib.rs
pub struct SearchEngine {
    index: Option<SearchIndex>,
    searcher: FileSearcher,
    content_searcher: ContentSearcher,
}

pub struct SearchCriteria {
    name_pattern: Option<Pattern>,
    content_pattern: Option<String>,
    size_range: Option<Range<u64>>,
    date_range: Option<Range<DateTime>>,
    file_types: Vec<FileType>,
    search_depth: SearchDepth,
}

pub struct SearchResult {
    path: PathBuf,
    matches: Vec<Match>,
    preview: Option<String>,
    score: f32,
}
```

### 2. Archive Support - Browse Like Directories
**Priority: HIGH** - Critical for productivity

#### Features
- **Transparent archive browsing**
  - ZIP, TAR, GZ, 7Z, RAR support
  - Navigate archives like directories
  - Nested archive support
- **Archive operations**
  - Extract files/folders (F5 from archive)
  - Add to archive (F5 to archive)
  - Create new archives (Alt+F5)
  - Test integrity (Ctrl+T)
- **Virtual File System layer**
  - Uniform interface for files and archives
  - Memory-efficient streaming
  - Caching for performance

#### Implementation
```rust
// New module: cortex-vfs/src/lib.rs
pub trait VirtualFileSystem {
    async fn list(&self, path: &VirtualPath) -> Result<Vec<VFSEntry>>;
    async fn read(&self, path: &VirtualPath) -> Result<Box<dyn AsyncRead>>;
    async fn write(&self, path: &VirtualPath, data: Box<dyn AsyncRead>) -> Result<()>;
    async fn metadata(&self, path: &VirtualPath) -> Result<VFSMetadata>;
}

pub struct ArchiveVFS {
    archive_type: ArchiveType,
    cache: LruCache<VirtualPath, Vec<VFSEntry>>,
}

// Integration with existing FileSystem
impl FileSystem {
    pub fn open_virtual(&self, path: &Path) -> Result<Box<dyn VirtualFileSystem>> {
        if ArchiveDetector::is_archive(path)? {
            Ok(Box::new(ArchiveVFS::open(path)?))
        } else {
            Ok(Box::new(DirectoryVFS::new(path)))
        }
    }
}
```

### 3. Network & Remote File Systems
**Priority: MEDIUM** - Important for modern workflows

#### Features
- **Protocol support**
  - SSH/SFTP connections
  - FTP/FTPS
  - WebDAV
  - Cloud storage (S3, Azure, GCS)
- **Connection manager**
  - Save connection profiles
  - Quick connect (Ctrl+N)
  - Connection pool management
  - Auto-reconnect
- **Seamless integration**
  - Browse remote like local
  - Copy between local and remote
  - Background transfers

#### Implementation
```rust
// New module: cortex-net/src/lib.rs
pub struct RemoteFileSystem {
    protocol: Protocol,
    connection: Box<dyn RemoteConnection>,
    cache: FileCache,
}

pub trait RemoteConnection: Send + Sync {
    async fn connect(&mut self, config: &ConnectionConfig) -> Result<()>;
    async fn list(&self, path: &Path) -> Result<Vec<RemoteEntry>>;
    async fn download(&self, remote: &Path, local: &Path) -> Result<()>;
    async fn upload(&self, local: &Path, remote: &Path) -> Result<()>;
}

pub struct ConnectionManager {
    profiles: Vec<ConnectionProfile>,
    active_connections: HashMap<String, Box<dyn RemoteConnection>>,
}
```

### 4. Enhanced Plugin System
**Priority: MEDIUM** - Extensibility for power users

#### Features
- **Plugin hooks**
  - File operations (pre/post)
  - Panel events
  - Menu integration
  - Custom commands
- **Plugin API v2**
  - Access to panels and dialogs
  - Custom UI components
  - Background tasks
  - Inter-plugin communication
- **Plugin marketplace**
  - Built-in plugin browser
  - Install/update from registry
  - Plugin dependencies

#### Implementation
```rust
// Enhanced cortex-plugins/src/lib.rs
pub trait PluginV2: Plugin {
    async fn on_file_operation(&self, op: &FileOperation) -> OperationResult;
    async fn on_panel_change(&self, panel: &PanelState) -> Result<()>;
    async fn register_commands(&self) -> Vec<PluginCommand>;
    async fn create_ui(&self) -> Option<Box<dyn PluginUI>>;
}

pub struct PluginHook {
    event: HookEvent,
    handler: Box<dyn Fn(&Event) -> Result<()>>,
    priority: i32,
}

pub struct PluginRegistry {
    local_plugins: Vec<Box<dyn PluginV2>>,
    remote_registry: RegistryClient,
    hooks: HashMap<HookEvent, Vec<PluginHook>>,
}
```

### 5. Configuration & Customization
**Priority: HIGH** - Essential for user satisfaction

#### Features
- **Settings dialog (F9)**
  - Tabbed interface
  - Live preview
  - Import/export settings
  - Reset to defaults
- **Customizable keybindings**
  - Override any shortcut
  - Multi-key sequences
  - Context-specific bindings
  - Vim mode support
- **Themes and colors**
  - Built-in theme presets
  - Custom color schemes
  - Syntax highlighting themes
  - Panel layouts

#### Implementation
```rust
// Enhanced cortex-core/src/config.rs
pub struct ConfigurationV2 {
    general: GeneralConfig,
    keybindings: KeyBindings,
    theme: Theme,
    plugins: PluginConfig,
    connections: Vec<ConnectionProfile>,
}

pub struct KeyBindings {
    global: HashMap<KeySequence, Action>,
    context: HashMap<Context, HashMap<KeySequence, Action>>,
}

pub struct Theme {
    name: String,
    colors: ColorScheme,
    syntax: SyntaxTheme,
    layout: LayoutConfig,
}
```

### 6. Performance Optimizations
**Priority: HIGH** - Critical for large directories

#### Features
- **Directory caching**
  - Intelligent cache invalidation
  - Background refresh
  - Incremental updates
- **Lazy loading**
  - Virtual scrolling for huge lists
  - Progressive directory reading
  - Thumbnail generation queue
- **Parallel operations**
  - Multi-threaded file operations
  - Concurrent directory scanning
  - Background indexing
- **Memory optimization**
  - Streaming for large files
  - Entry pooling
  - Smart garbage collection

#### Implementation
```rust
// Performance enhancements
pub struct DirectoryCache {
    entries: Arc<RwLock<HashMap<PathBuf, CachedDirectory>>>,
    watcher: FileWatcher,
    refresh_queue: Arc<Mutex<VecDeque<PathBuf>>>,
}

pub struct VirtualScroller<T> {
    visible_range: Range<usize>,
    cache: LruCache<usize, T>,
    loader: Box<dyn Fn(Range<usize>) -> Vec<T>>,
}

pub struct ParallelOperationExecutor {
    thread_pool: ThreadPool,
    progress_aggregator: ProgressAggregator,
}
```

## Implementation Phases

### Phase 4.1: Search Foundation (Week 1-2)
1. Implement SearchEngine core
2. Create search dialog UI (Alt+F7)
3. Add name and content search
4. Build search results panel
5. Add result navigation and preview

### Phase 4.2: Archive Support (Week 3-4)
1. Implement VFS abstraction layer
2. Add ZIP and TAR support
3. Create archive browsing UI
4. Implement extract/compress operations
5. Add streaming and caching

### Phase 4.3: Configuration System (Week 5)
1. Create settings dialog (F9)
2. Implement keybinding customization
3. Add theme support
4. Build import/export functionality
5. Add live configuration reload

### Phase 4.4: Network Support (Week 6-7)
1. Implement SSH/SFTP client
2. Create connection manager
3. Add remote browsing UI
4. Implement background transfers
5. Add connection profiles

### Phase 4.5: Plugin System v2 (Week 8)
1. Design new plugin API
2. Add hook system
3. Create plugin UI framework
4. Build plugin manager dialog
5. Implement plugin marketplace client

### Phase 4.6: Performance & Polish (Week 9-10)
1. Implement directory caching
2. Add virtual scrolling
3. Optimize file operations
4. Add background indexing
5. Performance profiling and tuning

## Testing Strategy

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    // Search engine tests
    #[tokio::test]
    async fn test_content_search() { /* ... */ }
    
    // Archive operations tests
    #[tokio::test]
    async fn test_zip_extraction() { /* ... */ }
    
    // Network operations tests
    #[tokio::test]
    async fn test_sftp_connection() { /* ... */ }
}
```

### Integration Tests
- Search across large directories
- Archive operations with nested archives
- Network operations with disconnections
- Plugin loading and execution
- Configuration changes and persistence

### Performance Benchmarks
```rust
#[bench]
fn bench_large_directory_listing(b: &mut Bencher) {
    // Benchmark with 100,000 files
}

#[bench]
fn bench_content_search(b: &mut Bencher) {
    // Search in 10GB of files
}
```

## Dependencies to Add

```toml
# Cargo.toml additions
[workspace.dependencies]
# Search
tantivy = "0.22"  # Full-text search engine
regex = "1.10"
grep = "0.3"

# Archives
zip = "2.1"
tar = "0.4"
flate2 = "1.0"
sevenz-rust = "0.6"

# Network
ssh2 = "0.9"
async-ftp = "9.0"
reqwest = { version = "0.12", features = ["stream"] }
aws-sdk-s3 = "1.45"

# Performance
rayon = "1.10"
lru = "0.12"
dashmap = "6.0"

# Configuration
figment = { version = "0.10", features = ["toml", "json"] }
```

## Migration Path

### For Users
1. Phase 4 features are additive - no breaking changes
2. Default keybindings remain the same
3. New features activated through new shortcuts
4. Configuration auto-migrates from v0.1 to v0.2

### For Developers
1. Existing plugin API remains supported
2. New PluginV2 trait extends original
3. VFS layer transparently wraps existing FileSystem
4. All new modules follow established patterns

## Success Metrics

### Performance Targets
- Directory listing: <100ms for 10,000 files
- Search: <1s for 100,000 files
- Archive browsing: <500ms to open
- Network operations: Match native OS speed
- Memory usage: <100MB for typical usage

### User Experience Goals
- Zero learning curve for Far Manager users
- Instant response for common operations
- Seamless archive and network integration
- Rich plugin ecosystem
- Full keyboard accessibility

## Risk Mitigation

### Technical Risks
1. **Archive format compatibility**
   - Solution: Use well-tested libraries
   - Fallback: External tool integration

2. **Network reliability**
   - Solution: Robust retry logic
   - Fallback: Queue failed operations

3. **Performance regression**
   - Solution: Continuous benchmarking
   - Fallback: Feature flags for optimization

### Schedule Risks
1. **Scope creep**
   - Mitigation: Strict phase boundaries
   - Each phase independently valuable

2. **Dependency issues**
   - Mitigation: Vendor critical dependencies
   - Alternative implementations ready

## Next Steps

1. **Immediate Actions**
   - Set up search module structure
   - Create VFS trait definition
   - Design settings UI mockup

2. **Team Coordination**
   - Assign module ownership
   - Set up weekly progress reviews
   - Create testing checklist

3. **Documentation**
   - Update architecture diagrams
   - Create plugin development guide
   - Write performance tuning guide

## Conclusion

Phase 4 transforms Cortex from a solid file manager into a powerhouse tool for professionals. By focusing on search, archives, network support, and extensibility, we're creating a modern alternative to Far Manager that respects its heritage while embracing contemporary development practices.

The modular architecture established in earlier phases provides a solid foundation for these advanced features. Each component is designed to integrate seamlessly while maintaining independence for testing and maintenance.

With this implementation, Cortex will offer:
- **Power**: Advanced search and operations
- **Flexibility**: Extensive customization and plugins
- **Performance**: Optimized for large-scale operations
- **Connectivity**: Seamless local, archive, and remote access
- **Extensibility**: Rich plugin ecosystem

This positions Cortex as the definitive orthodox file manager for the modern era.