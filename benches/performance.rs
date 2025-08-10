use cortex_core::{
    CacheConfig, DirectoryCache, FileSystem, MemoryManager, ShortcutManager, StringPool,
    VirtualScrollConfig, VirtualScroller,
};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::path::Path;
use std::time::Duration;

/// Benchmark directory listing performance
fn bench_directory_listing(c: &mut Criterion) {
    let mut group = c.benchmark_group("directory_listing");

    // Test with different directory sizes
    let test_dirs = vec![("/tmp", "small"), ("/usr/bin", "medium"), ("/usr", "large")];

    for (path, size) in test_dirs {
        if Path::new(path).exists() {
            group.bench_with_input(
                BenchmarkId::new("list_directory", size),
                &path,
                |b, &path| {
                    b.iter(|| FileSystem::list_directory(Path::new(path), false));
                },
            );
        }
    }

    group.finish();
}

/// Benchmark cache performance
fn bench_cache_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache");

    // Setup cache
    let config = CacheConfig {
        max_entries: 1000,
        ttl: Duration::from_secs(300),
        max_memory_bytes: 100 * 1024 * 1024,
        enable_background_refresh: false,
        frequent_access_threshold: 5,
    };
    let cache = DirectoryCache::with_config(config);

    // Prepare test data
    let test_path = Path::new("/tmp");
    let entries = FileSystem::list_directory(test_path, false).unwrap_or_default();

    // Benchmark cache write
    group.bench_function("cache_put", |b| {
        b.iter(|| {
            cache
                .put(black_box(test_path), black_box(entries.clone()))
                .ok();
        });
    });

    // Prime the cache
    cache.put(test_path, entries.clone()).ok();

    // Benchmark cache read (hit)
    group.bench_function("cache_get_hit", |b| {
        b.iter(|| cache.get(black_box(test_path)));
    });

    // Benchmark cache read (miss)
    let missing_path = Path::new("/nonexistent");
    group.bench_function("cache_get_miss", |b| {
        b.iter(|| cache.get(black_box(missing_path)));
    });

    group.finish();
}

/// Benchmark virtual scrolling
fn bench_virtual_scrolling(c: &mut Criterion) {
    let mut group = c.benchmark_group("virtual_scroll");

    let config = VirtualScrollConfig::default();
    let mut scroller = VirtualScroller::new(config);

    // Test with different item counts
    let item_counts = vec![100, 1000, 10000, 100000];

    for count in item_counts {
        group.bench_with_input(
            BenchmarkId::new("scroll_init", count),
            &count,
            |b, &count| {
                b.iter(|| {
                    scroller.init(black_box(count), false);
                });
            },
        );

        // Initialize for scroll tests
        scroller.init(count, false);

        group.bench_with_input(
            BenchmarkId::new("scroll_position", count),
            &count,
            |b, &count| {
                b.iter(|| {
                    scroller.set_scroll_position(black_box(count / 2));
                });
            },
        );
    }

    group.finish();
}

/// Benchmark string pooling
fn bench_string_pool(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_pool");

    let pool = StringPool::new();
    let test_strings = vec![
        "file.txt",
        "document.pdf",
        "image.png",
        "file.txt",     // Duplicate
        "document.pdf", // Duplicate
    ];

    group.bench_function("intern_new", |b| {
        b.iter(|| pool.intern(black_box("unique_string.txt")));
    });

    // Prime the pool
    for s in &test_strings {
        pool.intern(s);
    }

    group.bench_function("intern_existing", |b| {
        b.iter(|| pool.intern(black_box("file.txt")));
    });

    group.finish();
}

/// Benchmark memory manager
fn bench_memory_manager(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_manager");

    let manager = MemoryManager::new(100);

    // Create test entries
    let entries = (0..100)
        .map(|i| cortex_core::FileEntry {
            name: format!("file_{}.txt", i),
            path: Path::new("/tmp").join(format!("file_{}.txt", i)),
            file_type: cortex_core::FileType::File,
            size: 1024 * i,
            size_display: format!("{} KB", i),
            modified: None,
            permissions: "-rw-r--r--".to_string(),
            is_hidden: false,
            extension: Some("txt".to_string()),
            is_selected: false,
        })
        .collect::<Vec<_>>();

    group.bench_function("compress_entries", |b| {
        b.iter(|| manager.compress_entries(black_box(&entries)));
    });

    let compressed = manager.compress_entries(&entries);

    group.bench_function("decompress_entries", |b| {
        b.iter(|| manager.decompress_entries(black_box(&compressed)));
    });

    group.finish();
}

/// Benchmark shortcut manager
fn bench_shortcuts(c: &mut Criterion) {
    let mut group = c.benchmark_group("shortcuts");

    let manager = ShortcutManager::new();

    use crossterm::event::{KeyCode, KeyModifiers};

    // Test common shortcuts
    let test_keys = vec![
        (KeyCode::F(5), KeyModifiers::NONE),
        (KeyCode::Char('c'), KeyModifiers::CONTROL),
        (KeyCode::Char('h'), KeyModifiers::NONE), // Vim mode
    ];

    for (code, modifiers) in test_keys {
        group.bench_with_input(
            BenchmarkId::new("get_action", format!("{:?}", code)),
            &(code, modifiers),
            |b, &(code, modifiers)| {
                b.iter(|| manager.get_action(black_box(code), black_box(modifiers)));
            },
        );
    }

    group.finish();
}

// Search operations are async and complex, so we'll skip them in benchmarks

/// Benchmark file operations
fn bench_file_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("file_operations");

    use std::fs;
    use tempfile::tempdir;

    let temp_dir = tempdir().unwrap();
    let test_file = temp_dir.path().join("test.txt");

    group.bench_function("create_file", |b| {
        b.iter(|| {
            fs::write(&test_file, "test content").ok();
            fs::remove_file(&test_file).ok();
        });
    });

    // Create file for other tests
    fs::write(&test_file, "test content").unwrap();

    group.bench_function("read_file_metadata", |b| {
        b.iter(|| fs::metadata(black_box(&test_file)));
    });

    group.bench_function("copy_file", |b| {
        let dest = temp_dir.path().join("copy.txt");
        b.iter(|| {
            fs::copy(&test_file, &dest).ok();
            fs::remove_file(&dest).ok();
        });
    });

    group.finish();
}

/// Benchmark UI rendering (simulated)
fn bench_ui_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("ui");

    // Simulate panel sorting
    let mut entries = (0..1000)
        .map(|i| cortex_core::FileEntry {
            name: format!("file_{:04}.txt", i),
            path: Path::new("/tmp").join(format!("file_{:04}.txt", i)),
            file_type: if i % 3 == 0 {
                cortex_core::FileType::Directory
            } else {
                cortex_core::FileType::File
            },
            size: 1024 * i,
            size_display: format!("{} KB", i),
            modified: None,
            permissions: "-rw-r--r--".to_string(),
            is_hidden: i % 10 == 0,
            extension: Some("txt".to_string()),
            is_selected: false,
        })
        .collect::<Vec<_>>();

    group.bench_function("sort_by_name", |b| {
        b.iter(|| {
            entries.sort_by(|a, b| a.name.cmp(&b.name));
        });
    });

    group.bench_function("sort_by_size", |b| {
        b.iter(|| {
            entries.sort_by(|a, b| b.size.cmp(&a.size));
        });
    });

    group.bench_function("filter_hidden", |b| {
        b.iter(|| entries.iter().filter(|e| !e.is_hidden).collect::<Vec<_>>());
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_directory_listing,
    bench_cache_operations,
    bench_virtual_scrolling,
    bench_string_pool,
    bench_memory_manager,
    bench_shortcuts,
    bench_file_operations,
    bench_ui_operations
);

criterion_main!(benches);
