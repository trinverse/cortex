use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant, SystemTime};

use crate::fs::FileEntry;
use crate::vfs::{VfsEntry, VfsPath};

/// Cached directory information
#[derive(Debug, Clone)]
pub struct CachedDirectory {
    pub entries: Vec<FileEntry>,
    pub last_modified: SystemTime,
    pub last_accessed: Instant,
    pub hit_count: u64,
    pub size_bytes: u64,
}

/// Cached VFS directory information
#[derive(Debug, Clone)]
pub struct CachedVfsDirectory {
    pub entries: Vec<VfsEntry>,
    pub last_accessed: Instant,
    pub hit_count: u64,
}

/// Configuration for the directory cache
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Maximum number of directories to cache
    pub max_entries: usize,
    /// Time-to-live for cache entries
    pub ttl: Duration,
    /// Maximum memory usage in bytes
    pub max_memory_bytes: u64,
    /// Enable background refresh for frequently accessed directories
    pub enable_background_refresh: bool,
    /// Threshold for considering a directory "frequently accessed"
    pub frequent_access_threshold: u64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 1000,
            ttl: Duration::from_secs(300),       // 5 minutes
            max_memory_bytes: 100 * 1024 * 1024, // 100MB
            enable_background_refresh: true,
            frequent_access_threshold: 10,
        }
    }
}

/// LRU cache for directory listings
pub struct DirectoryCache {
    config: CacheConfig,
    cache: Arc<RwLock<HashMap<PathBuf, CachedDirectory>>>,
    vfs_cache: Arc<RwLock<HashMap<String, CachedVfsDirectory>>>,
    access_order: Arc<RwLock<Vec<PathBuf>>>,
    vfs_access_order: Arc<RwLock<Vec<String>>>,
    total_memory_usage: Arc<RwLock<u64>>,
    statistics: Arc<RwLock<CacheStatistics>>,
}

/// Cache statistics for monitoring and optimization
#[derive(Debug, Default, Clone)]
pub struct CacheStatistics {
    pub total_hits: u64,
    pub total_misses: u64,
    pub total_evictions: u64,
    pub total_refreshes: u64,
    pub average_entry_size: u64,
    pub current_entry_count: usize,
    pub current_memory_usage: u64,
}

impl DirectoryCache {
    /// Create a new directory cache with default configuration
    pub fn new() -> Self {
        Self::with_config(CacheConfig::default())
    }

    /// Create a new directory cache with custom configuration
    pub fn with_config(config: CacheConfig) -> Self {
        Self {
            config,
            cache: Arc::new(RwLock::new(HashMap::new())),
            vfs_cache: Arc::new(RwLock::new(HashMap::new())),
            access_order: Arc::new(RwLock::new(Vec::new())),
            vfs_access_order: Arc::new(RwLock::new(Vec::new())),
            total_memory_usage: Arc::new(RwLock::new(0)),
            statistics: Arc::new(RwLock::new(CacheStatistics::default())),
        }
    }

    /// Get a directory listing from cache or None if not cached/expired
    pub fn get(&self, path: &Path) -> Option<Vec<FileEntry>> {
        let mut cache = self.cache.write().unwrap();
        let mut access_order = self.access_order.write().unwrap();
        let mut stats = self.statistics.write().unwrap();

        if let Some(cached) = cache.get_mut(path) {
            // Check TTL
            if cached.last_accessed.elapsed() > self.config.ttl {
                // Entry expired
                stats.total_misses += 1;
                return None;
            }

            // Update access tracking
            cached.last_accessed = Instant::now();
            cached.hit_count += 1;

            // Update LRU order
            if let Some(pos) = access_order.iter().position(|p| p == path) {
                access_order.remove(pos);
            }
            access_order.push(path.to_path_buf());

            stats.total_hits += 1;

            Some(cached.entries.clone())
        } else {
            stats.total_misses += 1;
            None
        }
    }

    /// Get a VFS directory listing from cache
    pub fn get_vfs(&self, path: &VfsPath) -> Option<Vec<VfsEntry>> {
        let path_key = format!("{:?}", path);
        let mut cache = self.vfs_cache.write().unwrap();
        let mut access_order = self.vfs_access_order.write().unwrap();
        let mut stats = self.statistics.write().unwrap();

        if let Some(cached) = cache.get_mut(&path_key) {
            // Check TTL
            if cached.last_accessed.elapsed() > self.config.ttl {
                stats.total_misses += 1;
                return None;
            }

            // Update access tracking
            cached.last_accessed = Instant::now();
            cached.hit_count += 1;

            // Update LRU order
            if let Some(pos) = access_order.iter().position(|p| p == &path_key) {
                access_order.remove(pos);
            }
            access_order.push(path_key.clone());

            stats.total_hits += 1;

            Some(cached.entries.clone())
        } else {
            stats.total_misses += 1;
            None
        }
    }

    /// Store a directory listing in the cache
    pub fn put(&self, path: &Path, entries: Vec<FileEntry>) -> Result<()> {
        let mut cache = self.cache.write().unwrap();
        let mut access_order = self.access_order.write().unwrap();
        let mut memory_usage = self.total_memory_usage.write().unwrap();
        let mut stats = self.statistics.write().unwrap();

        // Calculate memory usage for this entry
        let entry_size = Self::calculate_entry_size(&entries);

        // Check if we need to evict entries
        while cache.len() >= self.config.max_entries
            || *memory_usage + entry_size > self.config.max_memory_bytes
        {
            if access_order.is_empty() {
                break;
            }

            // Evict least recently used entry
            let lru_path = access_order.remove(0);
            if let Some(evicted) = cache.remove(&lru_path) {
                *memory_usage = memory_usage.saturating_sub(evicted.size_bytes);
                stats.total_evictions += 1;
            }
        }

        // Get file system metadata for the directory
        let metadata = std::fs::metadata(path)?;
        let last_modified = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);

        // Create cached entry
        let cached = CachedDirectory {
            entries: entries.clone(),
            last_modified,
            last_accessed: Instant::now(),
            hit_count: 0,
            size_bytes: entry_size,
        };

        // Update cache
        cache.insert(path.to_path_buf(), cached);
        access_order.push(path.to_path_buf());
        *memory_usage += entry_size;

        // Update statistics
        stats.current_entry_count = cache.len();
        stats.current_memory_usage = *memory_usage;
        if stats.current_entry_count > 0 {
            stats.average_entry_size = *memory_usage / stats.current_entry_count as u64;
        }

        Ok(())
    }

    /// Store a VFS directory listing in the cache
    pub fn put_vfs(&self, path: &VfsPath, entries: Vec<VfsEntry>) -> Result<()> {
        let path_key = format!("{:?}", path);
        let mut cache = self.vfs_cache.write().unwrap();
        let mut access_order = self.vfs_access_order.write().unwrap();
        let mut stats = self.statistics.write().unwrap();

        // Check if we need to evict entries
        while cache.len() >= self.config.max_entries {
            if access_order.is_empty() {
                break;
            }

            // Evict least recently used entry
            let lru_path = access_order.remove(0);
            cache.remove(&lru_path);
            stats.total_evictions += 1;
        }

        // Create cached entry
        let cached = CachedVfsDirectory {
            entries: entries.clone(),
            last_accessed: Instant::now(),
            hit_count: 0,
        };

        // Update cache
        cache.insert(path_key.clone(), cached);
        access_order.push(path_key);

        // Update statistics
        stats.current_entry_count = cache.len() + self.cache.read().unwrap().len();

        Ok(())
    }

    /// Invalidate a specific cache entry
    pub fn invalidate(&self, path: &Path) {
        let mut cache = self.cache.write().unwrap();
        let mut access_order = self.access_order.write().unwrap();
        let mut memory_usage = self.total_memory_usage.write().unwrap();
        let mut stats = self.statistics.write().unwrap();

        if let Some(evicted) = cache.remove(path) {
            *memory_usage = memory_usage.saturating_sub(evicted.size_bytes);
            if let Some(pos) = access_order.iter().position(|p| p == path) {
                access_order.remove(pos);
            }
            stats.current_entry_count = cache.len();
            stats.current_memory_usage = *memory_usage;
        }
    }

    /// Invalidate a VFS cache entry
    pub fn invalidate_vfs(&self, path: &VfsPath) {
        let path_key = format!("{:?}", path);
        let mut cache = self.vfs_cache.write().unwrap();
        let mut access_order = self.vfs_access_order.write().unwrap();
        let mut stats = self.statistics.write().unwrap();

        if cache.remove(&path_key).is_some() {
            if let Some(pos) = access_order.iter().position(|p| p == &path_key) {
                access_order.remove(pos);
            }
            stats.current_entry_count = cache.len() + self.cache.read().unwrap().len();
        }
    }

    /// Clear all cache entries
    pub fn clear(&self) {
        let mut cache = self.cache.write().unwrap();
        let mut vfs_cache = self.vfs_cache.write().unwrap();
        let mut access_order = self.access_order.write().unwrap();
        let mut vfs_access_order = self.vfs_access_order.write().unwrap();
        let mut memory_usage = self.total_memory_usage.write().unwrap();
        let mut stats = self.statistics.write().unwrap();

        cache.clear();
        vfs_cache.clear();
        access_order.clear();
        vfs_access_order.clear();
        *memory_usage = 0;

        stats.current_entry_count = 0;
        stats.current_memory_usage = 0;
    }

    /// Get cache statistics
    pub fn get_statistics(&self) -> CacheStatistics {
        self.statistics.read().unwrap().clone()
    }

    /// Get list of frequently accessed directories for background refresh
    pub fn get_frequent_directories(&self) -> Vec<PathBuf> {
        let cache = self.cache.read().unwrap();
        cache
            .iter()
            .filter(|(_, entry)| entry.hit_count >= self.config.frequent_access_threshold)
            .map(|(path, _)| path.clone())
            .collect()
    }

    /// Check if a directory has been modified since it was cached
    pub fn is_stale(&self, path: &Path) -> bool {
        let cache = self.cache.read().unwrap();

        if let Some(cached) = cache.get(path) {
            if let Ok(metadata) = std::fs::metadata(path) {
                if let Ok(modified) = metadata.modified() {
                    return modified > cached.last_modified;
                }
            }
        }

        false
    }

    /// Calculate approximate memory usage for a list of file entries
    fn calculate_entry_size(entries: &[FileEntry]) -> u64 {
        let mut size = 0u64;
        for entry in entries {
            // Approximate memory usage per entry
            size += std::mem::size_of::<FileEntry>() as u64;
            size += entry.name.len() as u64;
            size += entry.path.to_string_lossy().len() as u64;
            size += entry.permissions.len() as u64;
            size += entry.size_display.len() as u64;
            size += entry.extension.as_ref().map(|s| s.len()).unwrap_or(0) as u64;
        }
        size
    }

    /// Update configuration
    pub fn update_config(&self, _new_config: CacheConfig) {
        // This would require making config mutable with RwLock
        // For now, cache would need to be recreated with new config
    }
}

/// Background cache refresher
pub struct CacheRefresher {
    cache: Arc<DirectoryCache>,
    running: Arc<RwLock<bool>>,
}

impl CacheRefresher {
    pub fn new(cache: Arc<DirectoryCache>) -> Self {
        Self {
            cache,
            running: Arc::new(RwLock::new(false)),
        }
    }

    /// Start the background refresh task
    pub async fn start(&self) {
        use tokio::time::{sleep, Duration};

        let mut running = self.running.write().unwrap();
        if *running {
            return; // Already running
        }
        *running = true;
        drop(running);

        let cache = self.cache.clone();
        let running = self.running.clone();

        tokio::spawn(async move {
            while *running.read().unwrap() {
                // Get frequently accessed directories
                let frequent_dirs = cache.get_frequent_directories();

                for dir in frequent_dirs {
                    // Check if the directory is stale
                    if cache.is_stale(&dir) {
                        // Refresh the cache entry
                        if let Ok(entries) = crate::fs::FileSystem::list_directory(&dir, false) {
                            let _ = cache.put(&dir, entries);

                            let mut stats = cache.statistics.write().unwrap();
                            stats.total_refreshes += 1;
                        }
                    }
                }

                // Sleep for a while before next refresh cycle
                sleep(Duration::from_secs(30)).await;
            }
        });
    }

    /// Stop the background refresh task
    pub fn stop(&self) {
        let mut running = self.running.write().unwrap();
        *running = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_cache_basic_operations() {
        let cache = DirectoryCache::new();
        let dir = tempdir().unwrap();
        let path = dir.path();

        // Create some test files
        fs::write(path.join("file1.txt"), "content1").unwrap();
        fs::write(path.join("file2.txt"), "content2").unwrap();

        // Cache miss
        assert!(cache.get(path).is_none());

        // Put entries in cache
        let entries = crate::fs::FileSystem::list_directory(path, false).unwrap();
        cache.put(path, entries.clone()).unwrap();

        // Cache hit
        let cached = cache.get(path).unwrap();
        assert_eq!(cached.len(), entries.len());

        // Check statistics
        let stats = cache.get_statistics();
        assert_eq!(stats.total_hits, 1);
        assert_eq!(stats.total_misses, 1);
    }

    #[test]
    fn test_cache_lru_eviction() {
        let config = CacheConfig {
            max_entries: 2,
            ..Default::default()
        };
        let cache = DirectoryCache::with_config(config);

        let dir1 = tempdir().unwrap();
        let dir2 = tempdir().unwrap();
        let dir3 = tempdir().unwrap();

        // Add three directories (one will be evicted)
        let entries1 = vec![];
        let entries2 = vec![];
        let entries3 = vec![];

        cache.put(dir1.path(), entries1).unwrap();
        cache.put(dir2.path(), entries2).unwrap();
        cache.put(dir3.path(), entries3).unwrap();

        // First directory should be evicted
        assert!(cache.get(dir1.path()).is_none());
        assert!(cache.get(dir2.path()).is_some());
        assert!(cache.get(dir3.path()).is_some());

        let stats = cache.get_statistics();
        assert_eq!(stats.total_evictions, 1);
    }

    #[test]
    fn test_cache_invalidation() {
        let cache = DirectoryCache::new();
        let dir = tempdir().unwrap();
        let path = dir.path();

        // Put entry in cache
        let entries = vec![];
        cache.put(path, entries).unwrap();

        // Verify it's cached
        assert!(cache.get(path).is_some());

        // Invalidate
        cache.invalidate(path);

        // Should be gone
        assert!(cache.get(path).is_none());
    }
}
