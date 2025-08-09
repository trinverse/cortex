use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

/// String interning pool for reducing memory usage of duplicate strings
pub struct StringPool {
    pool: Arc<RwLock<HashMap<String, Arc<str>>>>,
    stats: Arc<RwLock<StringPoolStats>>,
}

#[derive(Debug, Default)]
pub struct StringPoolStats {
    pub total_strings: usize,
    pub unique_strings: usize,
    pub memory_saved: usize,
}

impl StringPool {
    pub fn new() -> Self {
        Self {
            pool: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(StringPoolStats::default())),
        }
    }
    
    /// Intern a string - returns a reference to the pooled string
    pub fn intern(&self, s: &str) -> Arc<str> {
        let mut pool = self.pool.write().unwrap();
        let mut stats = self.stats.write().unwrap();
        
        if let Some(interned) = pool.get(s) {
            stats.memory_saved += s.len();
            interned.clone()
        } else {
            let arc_str: Arc<str> = Arc::from(s);
            pool.insert(s.to_string(), arc_str.clone());
            stats.total_strings += 1;
            stats.unique_strings = pool.len();
            arc_str
        }
    }
    
    /// Get statistics about the string pool
    pub fn get_stats(&self) -> StringPoolStats {
        let stats = self.stats.read().unwrap();
        StringPoolStats {
            total_strings: stats.total_strings,
            unique_strings: stats.unique_strings,
            memory_saved: stats.memory_saved,
        }
    }
    
    /// Clear the string pool
    pub fn clear(&self) {
        let mut pool = self.pool.write().unwrap();
        let mut stats = self.stats.write().unwrap();
        pool.clear();
        *stats = StringPoolStats::default();
    }
}

/// Object pool for reusing frequently allocated objects
pub struct ObjectPool<T: Clone + Default> {
    available: Arc<RwLock<Vec<T>>>,
    in_use: Arc<RwLock<usize>>,
    _max_size: usize,
}

impl<T: Clone + Default> ObjectPool<T> {
    pub fn new(initial_size: usize, max_size: usize) -> Self {
        let mut available = Vec::with_capacity(initial_size);
        for _ in 0..initial_size {
            available.push(T::default());
        }
        
        Self {
            available: Arc::new(RwLock::new(available)),
            in_use: Arc::new(RwLock::new(0)),
            _max_size: max_size,
        }
    }
    
    /// Acquire an object from the pool
    pub fn acquire(&self) -> PooledObject<T> {
        let mut available = self.available.write().unwrap();
        let mut in_use = self.in_use.write().unwrap();
        
        let obj = if let Some(obj) = available.pop() {
            obj
        } else {
            T::default()
        };
        
        *in_use += 1;
        
        PooledObject {
            object: Some(obj),
            pool: self.available.clone(),
            in_use: self.in_use.clone(),
        }
    }
    
    /// Get current pool statistics
    pub fn stats(&self) -> (usize, usize) {
        let available = self.available.read().unwrap().len();
        let in_use = *self.in_use.read().unwrap();
        (available, in_use)
    }
}

/// A pooled object that returns to the pool when dropped
pub struct PooledObject<T: Clone> {
    object: Option<T>,
    pool: Arc<RwLock<Vec<T>>>,
    in_use: Arc<RwLock<usize>>,
}

impl<T: Clone> PooledObject<T> {
    pub fn get(&self) -> &T {
        self.object.as_ref().unwrap()
    }
    
    pub fn get_mut(&mut self) -> &mut T {
        self.object.as_mut().unwrap()
    }
}

impl<T: Clone> Drop for PooledObject<T> {
    fn drop(&mut self) {
        if let Some(obj) = self.object.take() {
            let mut pool = self.pool.write().unwrap();
            let mut in_use = self.in_use.write().unwrap();
            
            // Only return to pool if not at capacity
            if pool.len() < pool.capacity() {
                pool.push(obj);
            }
            
            *in_use = in_use.saturating_sub(1);
        }
    }
}

/// Compressed file entry for memory-efficient storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressedFileEntry {
    pub name: String,
    pub path_index: u32,  // Index into path table
    pub file_type: u8,    // Compact representation
    pub size: u64,
    pub modified: u64,    // Unix timestamp
    pub permissions: u16, // Compact permissions
}

impl CompressedFileEntry {
    /// Convert from a regular FileEntry
    pub fn from_entry(entry: &crate::fs::FileEntry, path_table: &PathTable, _string_pool: &StringPool) -> Self {
        let name = entry.name.clone();
        let path_index = path_table.get_or_add(&entry.path);
        
        let file_type = match entry.file_type {
            crate::fs::FileType::File => 0,
            crate::fs::FileType::Directory => 1,
            crate::fs::FileType::Symlink => 2,
            crate::fs::FileType::Other => 3,
        };
        
        let modified = entry.modified
            .map(|dt| dt.timestamp() as u64)
            .unwrap_or(0);
        
        let permissions = parse_permissions(&entry.permissions);
        
        Self {
            name,
            path_index,
            file_type,
            size: entry.size,
            modified,
            permissions,
        }
    }
    
    /// Convert back to a regular FileEntry
    pub fn to_entry(&self, path_table: &PathTable) -> Option<crate::fs::FileEntry> {
        let path = path_table.get(self.path_index)?;
        
        let file_type = match self.file_type {
            0 => crate::fs::FileType::File,
            1 => crate::fs::FileType::Directory,
            2 => crate::fs::FileType::Symlink,
            _ => crate::fs::FileType::Other,
        };
        
        let modified = if self.modified > 0 {
            Some(chrono::DateTime::from_timestamp(self.modified as i64, 0)?)
        } else {
            None
        };
        
        Some(crate::fs::FileEntry {
            name: self.name.to_string(),
            path: path.clone(),
            file_type,
            size: self.size,
            size_display: format_size(self.size),
            modified,
            permissions: format_permissions(self.permissions),
            is_hidden: self.name.starts_with('.'),
            extension: Path::new(&self.name)
                .extension()
                .and_then(|s| s.to_str())
                .map(|s| s.to_string()),
            is_selected: false,
        })
    }
}

/// Path table for deduplicating path storage
pub struct PathTable {
    paths: Arc<RwLock<Vec<PathBuf>>>,
    index: Arc<RwLock<HashMap<PathBuf, u32>>>,
}

impl PathTable {
    pub fn new() -> Self {
        Self {
            paths: Arc::new(RwLock::new(Vec::new())),
            index: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Get or add a path to the table
    pub fn get_or_add(&self, path: &Path) -> u32 {
        let mut index = self.index.write().unwrap();
        
        if let Some(&idx) = index.get(path) {
            return idx;
        }
        
        let mut paths = self.paths.write().unwrap();
        let idx = paths.len() as u32;
        paths.push(path.to_path_buf());
        index.insert(path.to_path_buf(), idx);
        
        idx
    }
    
    /// Get a path by index
    pub fn get(&self, index: u32) -> Option<PathBuf> {
        let paths = self.paths.read().unwrap();
        paths.get(index as usize).cloned()
    }
    
    /// Get statistics
    pub fn stats(&self) -> (usize, usize) {
        let paths = self.paths.read().unwrap();
        let total_size: usize = paths.iter().map(|p| p.as_os_str().len()).sum();
        (paths.len(), total_size)
    }
}

/// Memory manager for tracking and optimizing memory usage
pub struct MemoryManager {
    string_pool: Arc<StringPool>,
    path_table: Arc<PathTable>,
    entry_pool: Arc<ObjectPool<crate::fs::FileEntry>>,
    memory_limit: usize,
}

impl MemoryManager {
    pub fn new(memory_limit_mb: usize) -> Self {
        Self {
            string_pool: Arc::new(StringPool::new()),
            path_table: Arc::new(PathTable::new()),
            entry_pool: Arc::new(ObjectPool::new(100, 1000)),
            memory_limit: memory_limit_mb * 1024 * 1024,
        }
    }
    
    /// Compress a list of file entries
    pub fn compress_entries(&self, entries: &[crate::fs::FileEntry]) -> Vec<CompressedFileEntry> {
        entries.iter()
            .map(|e| CompressedFileEntry::from_entry(e, &self.path_table, &self.string_pool))
            .collect()
    }
    
    /// Decompress a list of compressed entries
    pub fn decompress_entries(&self, compressed: &[CompressedFileEntry]) -> Vec<crate::fs::FileEntry> {
        compressed.iter()
            .filter_map(|c| c.to_entry(&self.path_table))
            .collect()
    }
    
    /// Get memory usage statistics
    pub fn get_stats(&self) -> MemoryStats {
        let string_stats = self.string_pool.get_stats();
        let (path_count, path_size) = self.path_table.stats();
        let (pool_available, pool_in_use) = self.entry_pool.stats();
        
        MemoryStats {
            string_pool_strings: string_stats.total_strings,
            string_pool_unique: string_stats.unique_strings,
            string_pool_saved: string_stats.memory_saved,
            path_table_entries: path_count,
            path_table_size: path_size,
            entry_pool_available: pool_available,
            entry_pool_in_use: pool_in_use,
            estimated_total: Self::estimate_memory_usage(&string_stats, path_size, pool_in_use),
        }
    }
    
    /// Check if memory limit is exceeded
    pub fn is_over_limit(&self) -> bool {
        let stats = self.get_stats();
        stats.estimated_total > self.memory_limit
    }
    
    /// Perform garbage collection
    pub fn gc(&self) {
        // Clear unused strings
        self.string_pool.clear();
        // Path table is not cleared as paths are persistent
    }
    
    fn estimate_memory_usage(string_stats: &StringPoolStats, path_size: usize, entries_in_use: usize) -> usize {
        let string_memory = string_stats.unique_strings * 50; // Estimate 50 bytes per unique string
        let entry_memory = entries_in_use * std::mem::size_of::<crate::fs::FileEntry>();
        string_memory + path_size + entry_memory
    }
}

#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub string_pool_strings: usize,
    pub string_pool_unique: usize,
    pub string_pool_saved: usize,
    pub path_table_entries: usize,
    pub path_table_size: usize,
    pub entry_pool_available: usize,
    pub entry_pool_in_use: usize,
    pub estimated_total: usize,
}

// Helper functions

fn parse_permissions(perm_str: &str) -> u16 {
    // Convert permission string to compact u16
    // This is a simplified version
    let mut perms = 0u16;
    
    for (i, ch) in perm_str.chars().enumerate() {
        if i >= 10 { break; }
        match ch {
            'r' => perms |= 1 << (9 - i),
            'w' => perms |= 1 << (9 - i),
            'x' => perms |= 1 << (9 - i),
            _ => {}
        }
    }
    
    perms
}

fn format_permissions(perms: u16) -> String {
    // Convert compact permissions back to string
    let mut s = String::with_capacity(10);
    
    // Simplified version - just return a default
    if perms & 0x100 != 0 {
        s.push_str("drwxr-xr-x");
    } else {
        s.push_str("-rw-r--r--");
    }
    
    s
}

fn format_size(size: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = size as f64;
    let mut unit_index = 0;
    
    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }
    
    if unit_index == 0 {
        format!("{} {}", size as u64, UNITS[unit_index])
    } else {
        format!("{:.2} {}", size, UNITS[unit_index])
    }
}

/// Memory-efficient batch processor
pub struct BatchProcessor<T> {
    batch_size: usize,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> BatchProcessor<T> {
    pub fn new(batch_size: usize) -> Self {
        Self {
            batch_size,
            _phantom: std::marker::PhantomData,
        }
    }
    
    /// Process items in batches to reduce memory usage
    pub fn process<F, R>(&self, items: Vec<T>, mut processor: F) -> Vec<R>
    where
        F: FnMut(&[T]) -> Vec<R>,
    {
        let mut results = Vec::new();
        
        for chunk in items.chunks(self.batch_size) {
            let batch_results = processor(chunk);
            results.extend(batch_results);
        }
        
        results
    }
    
    /// Process items in parallel batches
    pub async fn process_parallel<F, R>(&self, items: Vec<T>, processor: F) -> Vec<R>
    where
        T: Clone + Send + Sync + 'static,
        R: Send + 'static,
        F: Fn(&[T]) -> Vec<R> + Send + Sync + Clone + 'static,
    {
        use tokio::task;
        
        let mut handles = Vec::new();
        
        for chunk in items.chunks(self.batch_size) {
            let chunk = chunk.to_vec();
            let proc = processor.clone();
            
            let handle = task::spawn(async move {
                proc(&chunk)
            });
            
            handles.push(handle);
        }
        
        let mut results = Vec::new();
        for handle in handles {
            if let Ok(batch_results) = handle.await {
                results.extend(batch_results);
            }
        }
        
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_string_pool() {
        let pool = StringPool::new();
        
        let s1 = pool.intern("hello");
        let s2 = pool.intern("hello");
        let s3 = pool.intern("world");
        
        // Same strings should be the same Arc
        assert!(Arc::ptr_eq(&s1, &s2));
        assert!(!Arc::ptr_eq(&s1, &s3));
        
        let stats = pool.get_stats();
        assert_eq!(stats.unique_strings, 2);
        assert!(stats.memory_saved > 0);
    }
    
    #[test]
    fn test_object_pool() {
        let pool: ObjectPool<Vec<u8>> = ObjectPool::new(2, 5);
        
        let mut obj1 = pool.acquire();
        obj1.get_mut().push(1);
        
        let (available, in_use) = pool.stats();
        assert_eq!(available, 1);
        assert_eq!(in_use, 1);
        
        drop(obj1);
        
        let (available, in_use) = pool.stats();
        assert_eq!(available, 2);
        assert_eq!(in_use, 0);
    }
    
    #[test]
    fn test_path_table() {
        let table = PathTable::new();
        
        let idx1 = table.get_or_add(Path::new("/home/user"));
        let idx2 = table.get_or_add(Path::new("/home/user"));
        let idx3 = table.get_or_add(Path::new("/tmp"));
        
        assert_eq!(idx1, idx2);
        assert_ne!(idx1, idx3);
        
        assert_eq!(table.get(idx1), Some(PathBuf::from("/home/user")));
        assert_eq!(table.get(idx3), Some(PathBuf::from("/tmp")));
    }
}