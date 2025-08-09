# Phase 4 Implementation Guide - Getting Started

## Quick Start for Phase 4.1: Advanced Search

This guide provides concrete steps to begin implementing Phase 4 features, starting with the Advanced Search system.

## Step 1: Create Search Module Structure

### 1.1 Add cortex-search to workspace

```bash
# Create new module
mkdir -p cortex-search/src
```

### 1.2 Create Cargo.toml for cortex-search

```toml
# cortex-search/Cargo.toml
[package]
name = "cortex-search"
version = "0.1.0"
edition = "2021"

[dependencies]
cortex-core = { path = "../cortex-core" }
anyhow = "1.0"
async-trait = "0.1"
tokio = { version = "1.40", features = ["full"] }
regex = "1.10"
glob = "0.3"
walkdir = "2.5"
rayon = "1.10"
grep = "0.3"
ignore = "0.4"
chrono = "0.4"
serde = { version = "1.0", features = ["derive"] }
```

### 1.3 Update workspace Cargo.toml

Add to `/Users/atyagi/code/cortex/Cargo.toml`:
```toml
[workspace]
members = [
    "cortex-core",
    "cortex-tui", 
    "cortex-cli",
    "cortex-plugins",
    "cortex-search",  # Add this line
]
```

## Step 2: Implement Core Search Types

### 2.1 Create search types (cortex-search/src/types.rs)

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::ops::Range;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchCriteria {
    pub name_pattern: Option<SearchPattern>,
    pub content_pattern: Option<String>,
    pub content_regex: bool,
    pub case_sensitive: bool,
    pub size_range: Option<Range<u64>>,
    pub modified_range: Option<Range<DateTime<Utc>>>,
    pub file_types: Vec<FileTypeFilter>,
    pub search_depth: SearchDepth,
    pub include_hidden: bool,
    pub follow_symlinks: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SearchPattern {
    Glob(String),
    Regex(String),
    Literal(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileTypeFilter {
    Regular,
    Directory,
    Symlink,
    Executable,
    Archive,
    Image,
    Video,
    Audio,
    Document,
    Source,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SearchDepth {
    CurrentOnly,
    Depth(usize),
    Unlimited,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub path: PathBuf,
    pub size: u64,
    pub modified: DateTime<Utc>,
    pub matches: Vec<ContentMatch>,
    pub score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentMatch {
    pub line_number: usize,
    pub column: usize,
    pub line_text: String,
    pub match_text: String,
}
```

### 2.2 Create search engine (cortex-search/src/engine.rs)

```rust
use crate::types::*;
use anyhow::Result;
use async_trait::async_trait;
use std::path::Path;
use tokio::sync::mpsc;

#[async_trait]
pub trait SearchEngine: Send + Sync {
    async fn search(
        &self,
        root: &Path,
        criteria: &SearchCriteria,
        progress: mpsc::Sender<SearchProgress>,
    ) -> Result<Vec<SearchResult>>;
    
    async fn search_stream(
        &self,
        root: &Path,
        criteria: &SearchCriteria,
    ) -> Result<mpsc::Receiver<SearchResult>>;
    
    fn cancel(&self);
}

pub struct SearchProgress {
    pub current_path: PathBuf,
    pub files_searched: usize,
    pub matches_found: usize,
    pub bytes_processed: u64,
}

pub struct FileSearchEngine {
    // Implementation details
}

impl FileSearchEngine {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl SearchEngine for FileSearchEngine {
    async fn search(
        &self,
        root: &Path,
        criteria: &SearchCriteria,
        progress: mpsc::Sender<SearchProgress>,
    ) -> Result<Vec<SearchResult>> {
        // Implementation to follow
        todo!()
    }
    
    async fn search_stream(
        &self,
        root: &Path,
        criteria: &SearchCriteria,
    ) -> Result<mpsc::Receiver<SearchResult>> {
        // Implementation to follow
        todo!()
    }
    
    fn cancel(&self) {
        // Implementation to follow
    }
}
```

## Step 3: Create Search Dialog UI

### 3.1 Create search dialog (cortex-tui/src/search_dialog.rs)

```rust
use cortex_search::{SearchCriteria, SearchResult, SearchPattern, SearchDepth, FileTypeFilter};
use ratatui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};
use std::path::PathBuf;

pub struct SearchDialog {
    pub criteria: SearchCriteria,
    pub results: Vec<SearchResult>,
    pub selected_result: usize,
    pub search_root: PathBuf,
    pub is_searching: bool,
    pub input_focus: SearchInputFocus,
    pub name_pattern_input: String,
    pub content_pattern_input: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SearchInputFocus {
    NamePattern,
    ContentPattern,
    Results,
}

impl SearchDialog {
    pub fn new(search_root: PathBuf) -> Self {
        Self {
            criteria: SearchCriteria {
                name_pattern: None,
                content_pattern: None,
                content_regex: false,
                case_sensitive: false,
                size_range: None,
                modified_range: None,
                file_types: vec![],
                search_depth: SearchDepth::Unlimited,
                include_hidden: false,
                follow_symlinks: false,
            },
            results: Vec::new(),
            selected_result: 0,
            search_root,
            is_searching: false,
            input_focus: SearchInputFocus::NamePattern,
            name_pattern_input: String::new(),
            content_pattern_input: String::new(),
        }
    }

    pub fn render<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        // Clear background
        f.render_widget(Clear, area);

        // Create layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(10), // Search criteria
                Constraint::Min(5),     // Results
                Constraint::Length(3),  // Status/help
            ])
            .split(area);

        self.render_criteria(f, chunks[0]);
        self.render_results(f, chunks[1]);
        self.render_status(f, chunks[2]);
    }

    fn render_criteria<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let block = Block::default()
            .title(" Search Criteria (Alt+F7) ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(area);
        f.render_widget(block, area);

        // Create input fields layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Name pattern
                Constraint::Length(3), // Content pattern
                Constraint::Length(2), // Options
            ])
            .split(inner);

        // Name pattern input
        let name_style = if self.input_focus == SearchInputFocus::NamePattern {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        
        let name_input = Paragraph::new(format!("Name pattern: {}", self.name_pattern_input))
            .style(name_style);
        f.render_widget(name_input, chunks[0]);

        // Content pattern input
        let content_style = if self.input_focus == SearchInputFocus::ContentPattern {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        
        let content_input = Paragraph::new(format!("Content: {}", self.content_pattern_input))
            .style(content_style);
        f.render_widget(content_input, chunks[1]);

        // Options display
        let options = format!(
            "Depth: {:?} | Hidden: {} | Case: {}",
            self.criteria.search_depth,
            if self.criteria.include_hidden { "Yes" } else { "No" },
            if self.criteria.case_sensitive { "Sensitive" } else { "Ignore" }
        );
        let options_widget = Paragraph::new(options);
        f.render_widget(options_widget, chunks[2]);
    }

    fn render_results<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let block = Block::default()
            .title(format!(" Search Results ({} found) ", self.results.len()))
            .borders(Borders::ALL);

        let items: Vec<ListItem> = self.results
            .iter()
            .enumerate()
            .map(|(i, result)| {
                let style = if i == self.selected_result {
                    Style::default().bg(Color::Blue).fg(Color::White)
                } else {
                    Style::default()
                };
                
                let content = format!(
                    "{} ({} matches)",
                    result.path.display(),
                    result.matches.len()
                );
                
                ListItem::new(content).style(style)
            })
            .collect();

        let list = List::new(items)
            .block(block)
            .highlight_style(Style::default().add_modifier(Modifier::BOLD));

        f.render_widget(list, area);
    }

    fn render_status<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let help_text = if self.is_searching {
            "Searching... Press ESC to cancel"
        } else {
            "Tab: Switch field | F7: Start search | Enter: Open result | ESC: Close"
        };

        let paragraph = Paragraph::new(help_text)
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::TOP));

        f.render_widget(paragraph, area);
    }
}
```

### 3.2 Add search dialog to main dialog enum

Update `cortex-tui/src/dialogs.rs` to include:
```rust
pub enum Dialog {
    // ... existing dialogs
    Search(SearchDialog),
}
```

## Step 4: Wire Up Search Functionality

### 4.1 Add Alt+F7 keybinding in main.rs

In `cortex-cli/src/main.rs`, add to the key handling:

```rust
// In the handle_key method
KeyCode::F7 if modifiers.contains(KeyModifiers::ALT) => {
    let search_root = self.state.active_panel().current_dir.clone();
    self.dialog = Some(Dialog::Search(SearchDialog::new(search_root)));
}
```

### 4.2 Handle search dialog events

Add search dialog handling in the dialog event processing:

```rust
Dialog::Search(ref mut search_dialog) => {
    match key.code {
        KeyCode::Tab => {
            // Switch between input fields
            search_dialog.input_focus = match search_dialog.input_focus {
                SearchInputFocus::NamePattern => SearchInputFocus::ContentPattern,
                SearchInputFocus::ContentPattern => SearchInputFocus::Results,
                SearchInputFocus::Results => SearchInputFocus::NamePattern,
            };
        }
        KeyCode::F7 => {
            // Start search
            self.start_search(search_dialog).await?;
        }
        KeyCode::Enter => {
            if search_dialog.input_focus == SearchInputFocus::Results {
                // Open selected result
                if let Some(result) = search_dialog.results.get(search_dialog.selected_result) {
                    // Navigate to file
                    self.navigate_to_file(&result.path)?;
                    self.dialog = None;
                }
            }
        }
        KeyCode::Esc => {
            self.dialog = None;
        }
        _ => {}
    }
}
```

## Step 5: Implement Basic Search

### 5.1 Create simple name search implementation

```rust
// In cortex-search/src/engine.rs
use walkdir::WalkDir;
use regex::Regex;

impl FileSearchEngine {
    pub async fn search_by_name(
        &self,
        root: &Path,
        pattern: &str,
        max_depth: Option<usize>,
    ) -> Result<Vec<SearchResult>> {
        let regex = Regex::new(pattern)?;
        let mut results = Vec::new();
        
        let walker = if let Some(depth) = max_depth {
            WalkDir::new(root).max_depth(depth)
        } else {
            WalkDir::new(root)
        };
        
        for entry in walker {
            let entry = entry?;
            let path = entry.path();
            
            if let Some(file_name) = path.file_name() {
                if regex.is_match(&file_name.to_string_lossy()) {
                    let metadata = entry.metadata()?;
                    results.push(SearchResult {
                        path: path.to_path_buf(),
                        size: metadata.len(),
                        modified: metadata.modified()?.into(),
                        matches: vec![],
                        score: 1.0,
                    });
                }
            }
        }
        
        Ok(results)
    }
}
```

## Testing the Implementation

### Run and Test
```bash
# Build the project with new search module
cargo build

# Run Cortex
./target/debug/cortex

# Test search:
# 1. Press Alt+F7 to open search dialog
# 2. Type a pattern (e.g., "*.rs" for Rust files)
# 3. Press F7 to start search
# 4. Navigate results with arrow keys
# 5. Press Enter to go to a file
```

## Next Implementation Steps

### Phase 4.1 Completion Checklist
- [ ] Content search with grep
- [ ] Size and date filters
- [ ] Search progress indicator
- [ ] Search result preview pane
- [ ] Export search results
- [ ] Search history
- [ ] Saved search profiles

### Phase 4.2 Archive Support Preparation
- [ ] Create cortex-vfs module structure
- [ ] Design VirtualPath type
- [ ] Implement archive detection
- [ ] Add zip crate integration

## Performance Considerations

1. **Use parallel search with rayon**
```rust
use rayon::prelude::*;

let results: Vec<SearchResult> = entries
    .par_iter()
    .filter_map(|entry| check_criteria(entry, &criteria))
    .collect();
```

2. **Stream results for large searches**
```rust
let (tx, rx) = mpsc::channel(100);
tokio::spawn(async move {
    // Send results as found
    for result in search_iter {
        tx.send(result).await?;
    }
});
```

3. **Cache compiled regexes**
```rust
use once_cell::sync::Lazy;
use lru::LruCache;

static REGEX_CACHE: Lazy<Mutex<LruCache<String, Regex>>> = 
    Lazy::new(|| Mutex::new(LruCache::new(100)));
```

## Common Pitfalls to Avoid

1. **Don't block the UI during search**
   - Always run search in background task
   - Update UI progressively

2. **Handle large result sets**
   - Implement virtual scrolling
   - Limit initial display to 1000 results

3. **Respect system resources**
   - Limit parallel threads
   - Add configurable memory limits

4. **Cross-platform compatibility**
   - Test path handling on Windows
   - Handle case sensitivity properly

## Resources and References

- [ripgrep internals](https://blog.burntsushi.net/ripgrep/) - Learn from ripgrep's architecture
- [tantivy documentation](https://docs.rs/tantivy/) - For future indexed search
- [Far Manager search](https://www.farmanager.com/eng/findfile.html) - UX reference

This implementation guide provides a concrete starting point for Phase 4. Each subsequent phase can follow a similar pattern of incremental, testable additions to the codebase.