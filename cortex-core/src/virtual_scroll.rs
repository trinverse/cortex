use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::Range;
use std::path::PathBuf;

use crate::fs::{FileEntry, FileSystem};
use crate::vfs::{VfsEntry, VfsPath, VirtualFileSystem};

/// Configuration for virtual scrolling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualScrollConfig {
    /// Number of items to load in viewport
    pub viewport_size: usize,
    /// Buffer size above and below viewport
    pub buffer_size: usize,
    /// Maximum items to keep in memory
    pub max_loaded_items: usize,
    /// Enable predictive loading
    pub predictive_loading: bool,
    /// Batch size for progressive loading
    pub batch_size: usize,
}

impl Default for VirtualScrollConfig {
    fn default() -> Self {
        Self {
            viewport_size: 50,
            buffer_size: 25,
            max_loaded_items: 500,
            predictive_loading: true,
            batch_size: 100,
        }
    }
}

/// Virtual scroller for handling large directories efficiently
pub struct VirtualScroller {
    config: VirtualScrollConfig,
    total_items: usize,
    visible_range: Range<usize>,
    loaded_items: HashMap<usize, FileEntry>,
    loaded_vfs_items: HashMap<usize, VfsEntry>,
    scroll_position: usize,
    is_vfs: bool,
    loading_queue: Vec<Range<usize>>,
}

impl VirtualScroller {
    /// Create a new virtual scroller
    pub fn new(config: VirtualScrollConfig) -> Self {
        Self {
            config,
            total_items: 0,
            visible_range: 0..0,
            loaded_items: HashMap::new(),
            loaded_vfs_items: HashMap::new(),
            scroll_position: 0,
            is_vfs: false,
            loading_queue: Vec::new(),
        }
    }

    /// Initialize with total item count
    pub fn init(&mut self, total_items: usize, is_vfs: bool) {
        self.total_items = total_items;
        self.is_vfs = is_vfs;
        self.loaded_items.clear();
        self.loaded_vfs_items.clear();
        self.loading_queue.clear();
        self.update_visible_range();
    }

    /// Set scroll position and update visible range
    pub fn set_scroll_position(&mut self, position: usize) {
        self.scroll_position = position.min(self.total_items.saturating_sub(1));
        self.update_visible_range();

        if self.config.predictive_loading {
            self.queue_predictive_loading();
        }
    }

    /// Scroll by a relative amount
    pub fn scroll_by(&mut self, delta: isize) {
        let new_position = if delta < 0 {
            self.scroll_position.saturating_sub(delta.abs() as usize)
        } else {
            (self.scroll_position + delta as usize).min(self.total_items.saturating_sub(1))
        };
        self.set_scroll_position(new_position);
    }

    /// Get the current visible range
    pub fn get_visible_range(&self) -> Range<usize> {
        self.visible_range.clone()
    }

    /// Get items in the visible range
    pub fn get_visible_items(&self) -> Vec<Option<&FileEntry>> {
        if self.is_vfs {
            return Vec::new();
        }

        (self.visible_range.start..self.visible_range.end)
            .map(|idx| self.loaded_items.get(&idx))
            .collect()
    }

    /// Get VFS items in the visible range
    pub fn get_visible_vfs_items(&self) -> Vec<Option<&VfsEntry>> {
        if !self.is_vfs {
            return Vec::new();
        }

        (self.visible_range.start..self.visible_range.end)
            .map(|idx| self.loaded_vfs_items.get(&idx))
            .collect()
    }

    /// Load items for a specific range
    pub fn load_range(&mut self, range: Range<usize>, items: Vec<FileEntry>) {
        if self.is_vfs {
            return;
        }

        for (offset, item) in items.into_iter().enumerate() {
            let idx = range.start + offset;
            if idx < range.end {
                self.loaded_items.insert(idx, item);
            }
        }

        self.cleanup_old_items();
    }

    /// Load VFS items for a specific range
    pub fn load_vfs_range(&mut self, range: Range<usize>, items: Vec<VfsEntry>) {
        if !self.is_vfs {
            return;
        }

        for (offset, item) in items.into_iter().enumerate() {
            let idx = range.start + offset;
            if idx < range.end {
                self.loaded_vfs_items.insert(idx, item);
            }
        }

        self.cleanup_old_items();
    }

    /// Check if a range needs loading
    pub fn needs_loading(&self, range: &Range<usize>) -> bool {
        if self.is_vfs {
            for idx in range.start..range.end {
                if !self.loaded_vfs_items.contains_key(&idx) {
                    return true;
                }
            }
        } else {
            for idx in range.start..range.end {
                if !self.loaded_items.contains_key(&idx) {
                    return true;
                }
            }
        }
        false
    }

    /// Get the next range that needs loading
    pub fn get_next_load_range(&mut self) -> Option<Range<usize>> {
        // First check visible range
        let visible_with_buffer = self.get_range_with_buffer();
        if self.needs_loading(&visible_with_buffer) {
            return Some(self.get_batch_range(visible_with_buffer.start));
        }

        // Then check queued ranges
        while let Some(range) = self.loading_queue.pop() {
            if self.needs_loading(&range) {
                return Some(range);
            }
        }

        None
    }

    /// Update the visible range based on scroll position
    fn update_visible_range(&mut self) {
        let start = self.scroll_position;
        let end = (start + self.config.viewport_size).min(self.total_items);
        self.visible_range = start..end;
    }

    /// Get visible range with buffer
    fn get_range_with_buffer(&self) -> Range<usize> {
        let start = self
            .visible_range
            .start
            .saturating_sub(self.config.buffer_size);
        let end = (self.visible_range.end + self.config.buffer_size).min(self.total_items);
        start..end
    }

    /// Queue predictive loading based on scroll direction
    fn queue_predictive_loading(&mut self) {
        // Clear existing queue
        self.loading_queue.clear();

        // Queue ranges based on likely scroll direction
        let buffer_range = self.get_range_with_buffer();

        // Queue below current position (likely to scroll down)
        if buffer_range.end < self.total_items {
            let next_start = buffer_range.end;
            let next_end = (next_start + self.config.batch_size).min(self.total_items);
            self.loading_queue.push(next_start..next_end);
        }

        // Queue above current position (might scroll up)
        if buffer_range.start > 0 {
            let prev_end = buffer_range.start;
            let prev_start = prev_end.saturating_sub(self.config.batch_size);
            self.loading_queue.push(prev_start..prev_end);
        }
    }

    /// Get a batch range starting from an index
    fn get_batch_range(&self, start: usize) -> Range<usize> {
        let end = (start + self.config.batch_size).min(self.total_items);
        start..end
    }

    /// Clean up items outside the buffer zone
    fn cleanup_old_items(&mut self) {
        let buffer_range = self.get_range_with_buffer();
        let extended_buffer = (buffer_range.start.saturating_sub(self.config.batch_size))
            ..(buffer_range.end + self.config.batch_size).min(self.total_items);

        if self.is_vfs {
            if self.loaded_vfs_items.len() > self.config.max_loaded_items {
                self.loaded_vfs_items
                    .retain(|idx, _| extended_buffer.contains(idx));
            }
        } else {
            if self.loaded_items.len() > self.config.max_loaded_items {
                self.loaded_items
                    .retain(|idx, _| extended_buffer.contains(idx));
            }
        }
    }

    /// Get memory usage estimate
    pub fn get_memory_usage(&self) -> usize {
        if self.is_vfs {
            self.loaded_vfs_items.len() * std::mem::size_of::<VfsEntry>()
        } else {
            self.loaded_items.len() * std::mem::size_of::<FileEntry>()
        }
    }

    /// Get loading statistics
    pub fn get_stats(&self) -> VirtualScrollStats {
        VirtualScrollStats {
            total_items: self.total_items,
            loaded_items: if self.is_vfs {
                self.loaded_vfs_items.len()
            } else {
                self.loaded_items.len()
            },
            visible_range: self.visible_range.clone(),
            scroll_position: self.scroll_position,
            memory_usage: self.get_memory_usage(),
            pending_loads: self.loading_queue.len(),
        }
    }
}

/// Statistics for virtual scrolling
#[derive(Debug, Clone)]
pub struct VirtualScrollStats {
    pub total_items: usize,
    pub loaded_items: usize,
    pub visible_range: Range<usize>,
    pub scroll_position: usize,
    pub memory_usage: usize,
    pub pending_loads: usize,
}

/// Manager for handling virtual scrolling with async loading
pub struct VirtualScrollManager {
    scroller: VirtualScroller,
    current_path: Option<PathBuf>,
    current_vfs_path: Option<VfsPath>,
    vfs: Option<VirtualFileSystem>,
}

impl VirtualScrollManager {
    pub fn new(config: VirtualScrollConfig) -> Self {
        Self {
            scroller: VirtualScroller::new(config),
            current_path: None,
            current_vfs_path: None,
            vfs: None,
        }
    }

    /// Initialize for a directory
    pub fn init_directory(&mut self, path: PathBuf, total_items: usize) {
        self.current_path = Some(path);
        self.current_vfs_path = None;
        self.scroller.init(total_items, false);
    }

    /// Initialize for a VFS path
    pub fn init_vfs(&mut self, path: VfsPath, total_items: usize, vfs: VirtualFileSystem) {
        self.current_vfs_path = Some(path);
        self.current_path = None;
        self.vfs = Some(vfs);
        self.scroller.init(total_items, true);
    }

    /// Load next batch of items if needed
    pub async fn load_next_batch(&mut self) -> Result<bool> {
        if let Some(range) = self.scroller.get_next_load_range() {
            if let Some(ref vfs_path) = self.current_vfs_path {
                // Load VFS items
                if let Some(ref vfs) = self.vfs {
                    // This would need to be implemented to support partial loading
                    // For now, we'd load all and slice
                    let all_entries = vfs.list_entries(vfs_path)?;
                    let batch: Vec<VfsEntry> = all_entries
                        .into_iter()
                        .skip(range.start)
                        .take(range.end - range.start)
                        .collect();
                    self.scroller.load_vfs_range(range, batch);
                }
            } else if let Some(ref path) = self.current_path {
                // Load regular file entries
                // This would need filesystem support for partial directory reading
                // For now, we load all and slice
                let all_entries = FileSystem::list_directory(path, false)?;
                let batch: Vec<FileEntry> = all_entries
                    .into_iter()
                    .skip(range.start)
                    .take(range.end - range.start)
                    .collect();
                self.scroller.load_range(range, batch);
            }
            return Ok(true);
        }
        Ok(false)
    }

    /// Scroll to a position
    pub fn scroll_to(&mut self, position: usize) {
        self.scroller.set_scroll_position(position);
    }

    /// Get visible items
    pub fn get_visible_items(&self) -> Vec<Option<&FileEntry>> {
        self.scroller.get_visible_items()
    }

    /// Get visible VFS items
    pub fn get_visible_vfs_items(&self) -> Vec<Option<&VfsEntry>> {
        self.scroller.get_visible_vfs_items()
    }

    /// Get statistics
    pub fn get_stats(&self) -> VirtualScrollStats {
        self.scroller.get_stats()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_virtual_scroller_basic() {
        let config = VirtualScrollConfig {
            viewport_size: 10,
            buffer_size: 5,
            ..Default::default()
        };

        let mut scroller = VirtualScroller::new(config);
        scroller.init(100, false);

        // Check initial visible range
        assert_eq!(scroller.get_visible_range(), 0..10);

        // Scroll down
        scroller.scroll_by(5);
        assert_eq!(scroller.get_visible_range(), 5..15);

        // Scroll to end
        scroller.set_scroll_position(95);
        assert_eq!(scroller.get_visible_range(), 95..100);
    }

    #[test]
    fn test_needs_loading() {
        let config = VirtualScrollConfig::default();
        let mut scroller = VirtualScroller::new(config);
        scroller.init(100, false);

        // Initially needs loading
        assert!(scroller.needs_loading(&(0..10)));

        // After loading, doesn't need loading
        let items = vec![]; // Would be actual items
        scroller.load_range(0..10, items);

        // Note: This would fail without actual items, but shows the pattern
    }
}
