# Directory Caching System

## Overview
The directory caching system in Cortex significantly improves performance by storing recently accessed directory listings in memory, reducing filesystem calls and improving responsiveness.

## Features

### 1. LRU Cache Implementation
- **Least Recently Used (LRU)** eviction policy ensures frequently accessed directories stay in cache
- **Configurable cache size** with a default of 1000 directories
- **Time-to-Live (TTL)** of 5 minutes for cache entries

### 2. Memory Management
- **Smart memory limits** - default 100MB maximum memory usage
- **Entry size calculation** - tracks actual memory usage per cached directory
- **Automatic eviction** when memory or entry limits are reached

### 3. Cache Statistics
- Tracks cache hits and misses
- Monitors eviction count
- Calculates average entry size
- Provides current memory usage metrics

### 4. Background Refresh
- **Automatic refresh** for frequently accessed directories (>5 accesses)
- **Stale detection** - checks if directory has been modified since caching
- **Non-blocking updates** using async tasks

### 5. VFS Integration
- Supports caching for Virtual File System paths
- Separate cache for remote (SFTP/FTP) and archive entries
- Unified interface for all filesystem types

## Implementation Details

### Cache Configuration
```rust
pub struct CacheConfig {
    pub max_entries: usize,              // Default: 1000
    pub ttl: Duration,                   // Default: 5 minutes
    pub max_memory_bytes: u64,          // Default: 100MB
    pub enable_background_refresh: bool, // Default: true
    pub frequent_access_threshold: u64,  // Default: 5
}
```

### Cache Structure
```rust
pub struct DirectoryCache {
    config: CacheConfig,
    cache: Arc<RwLock<HashMap<PathBuf, CachedDirectory>>>,
    vfs_cache: Arc<RwLock<HashMap<String, CachedVfsDirectory>>>,
    access_order: Arc<RwLock<Vec<PathBuf>>>,
    statistics: Arc<RwLock<CacheStatistics>>,
}
```

### Usage in File Panels
The cache is automatically used when refreshing file panels:
1. Check cache for existing entry
2. If cache hit and not expired, return cached data
3. If cache miss or expired, fetch from filesystem
4. Store fresh data in cache for future use

## Performance Benefits

### Expected Improvements
- **50-90% reduction** in directory listing time for cached entries
- **Near-instant** navigation to recently visited directories
- **Reduced I/O load** on the filesystem
- **Smoother UI** with less blocking operations

### Cache Hit Scenarios
- Navigating back to parent directory
- Switching between panels showing same directory
- Returning to frequently accessed directories
- Quick navigation up/down directory tree

## Cache Invalidation

### Automatic Invalidation
- TTL expiration (5 minutes)
- File system change detection (when file monitoring is active)
- Manual refresh command (`Ctrl+R` or `reload`)

### Manual Control
- Clear entire cache with `cache clear` command
- Invalidate specific directory on hidden files toggle (`Ctrl+H`)
- Force refresh with `refresh` command

## Memory Optimization

### Efficient Storage
- Stores only essential file metadata
- Compresses string data where possible
- Reuses common string values (interning)

### Adaptive Behavior
- Adjusts cache size based on available memory
- Prioritizes frequently accessed directories
- Evicts least recently used entries first

## Configuration

Users can customize cache behavior in `~/.config/cortex/config.toml`:

```toml
[cache]
max_entries = 1000
ttl_seconds = 300
max_memory_mb = 100
enable_background_refresh = true
frequent_access_threshold = 5
```

## Technical Architecture

### Thread Safety
- All cache operations are thread-safe using `Arc<RwLock>`
- Concurrent reads allowed, exclusive writes
- Lock-free statistics updates

### Integration Points
1. **Main Application**: Creates and manages cache instance
2. **File Panels**: Use cache for directory listings
3. **File Monitor**: Triggers cache invalidation on changes
4. **Background Refresher**: Updates stale entries

## Future Enhancements

### Planned Improvements
1. **Persistent cache** - Save cache to disk between sessions
2. **Predictive prefetching** - Cache likely next directories
3. **Compression** - Compress cached data for larger capacity
4. **Network cache** - Extended TTL for remote directories
5. **Smart invalidation** - Partial invalidation on specific file changes

## Monitoring

### Cache Metrics
Access cache statistics via the application:
- Total hits/misses ratio
- Memory usage percentage
- Eviction rate
- Average response time

### Debug Logging
Enable debug logging to see cache operations:
```bash
RUST_LOG=cortex_core::cache=debug ./cortex
```

## Summary

The directory caching system provides a significant performance boost to Cortex, making navigation feel instant and reducing system load. With smart memory management and automatic refresh, it provides an optimal balance between performance and resource usage.