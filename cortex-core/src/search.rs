use std::path::{Path, PathBuf};
use std::time::SystemTime;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use regex::Regex;
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchCriteria {
    pub pattern: String,
    pub search_type: SearchType,
    pub case_sensitive: bool,
    pub search_in_files: bool,
    pub include_hidden: bool,
    pub include_subdirs: bool,
    pub max_depth: Option<usize>,
    pub file_extensions: Vec<String>,
    pub size_filter: Option<SizeFilter>,
    pub date_filter: Option<DateFilter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SearchType {
    Wildcard,   // *.txt, file*.rs
    Regex,      // Full regex support
    Exact,      // Exact match
    Contains,   // Contains substring
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SizeFilter {
    pub min_size: Option<u64>,
    pub max_size: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateFilter {
    pub after: Option<SystemTime>,
    pub before: Option<SystemTime>,
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub path: PathBuf,
    pub size: u64,
    pub modified: SystemTime,
    pub matches: Vec<Match>,
}

#[derive(Debug, Clone)]
pub struct Match {
    pub line_number: Option<usize>,
    pub line_content: Option<String>,
    pub byte_offset: Option<usize>,
}

#[derive(Debug, Clone)]
pub enum SearchProgress {
    Started { total_dirs: usize },
    Searching { current_path: PathBuf, searched: usize, total: usize },
    Found { result: SearchResult },
    Completed { total_found: usize, elapsed_ms: u128 },
    Error { path: PathBuf, error: String },
}

pub struct SearchEngine {
    criteria: SearchCriteria,
    pattern_matcher: Box<dyn PatternMatcher>,
    results: Vec<SearchResult>,
    cancelled: bool,
}

trait PatternMatcher: Send + Sync {
    fn matches(&self, text: &str) -> bool;
}

struct WildcardMatcher {
    pattern: String,
    case_sensitive: bool,
}

impl PatternMatcher for WildcardMatcher {
    fn matches(&self, text: &str) -> bool {
        let pattern = if self.case_sensitive {
            self.pattern.clone()
        } else {
            self.pattern.to_lowercase()
        };
        
        let text = if self.case_sensitive {
            text.to_string()
        } else {
            text.to_lowercase()
        };
        
        // Convert wildcard to regex
        let regex_pattern = pattern
            .replace(".", r"\.")
            .replace("*", ".*")
            .replace("?", ".");
        
        if let Ok(regex) = Regex::new(&format!("^{}$", regex_pattern)) {
            regex.is_match(&text)
        } else {
            false
        }
    }
}

struct RegexMatcher {
    regex: Regex,
}

impl PatternMatcher for RegexMatcher {
    fn matches(&self, text: &str) -> bool {
        self.regex.is_match(text)
    }
}

struct ExactMatcher {
    pattern: String,
    case_sensitive: bool,
}

impl PatternMatcher for ExactMatcher {
    fn matches(&self, text: &str) -> bool {
        if self.case_sensitive {
            text == self.pattern
        } else {
            text.to_lowercase() == self.pattern.to_lowercase()
        }
    }
}

struct ContainsMatcher {
    pattern: String,
    case_sensitive: bool,
}

impl PatternMatcher for ContainsMatcher {
    fn matches(&self, text: &str) -> bool {
        if self.case_sensitive {
            text.contains(&self.pattern)
        } else {
            text.to_lowercase().contains(&self.pattern.to_lowercase())
        }
    }
}

impl SearchEngine {
    pub fn new(criteria: SearchCriteria) -> Result<Self> {
        let pattern_matcher: Box<dyn PatternMatcher> = match &criteria.search_type {
            SearchType::Wildcard => Box::new(WildcardMatcher {
                pattern: criteria.pattern.clone(),
                case_sensitive: criteria.case_sensitive,
            }),
            SearchType::Regex => {
                let pattern = if criteria.case_sensitive {
                    criteria.pattern.clone()
                } else {
                    format!("(?i){}", criteria.pattern)
                };
                Box::new(RegexMatcher {
                    regex: Regex::new(&pattern)?,
                })
            },
            SearchType::Exact => Box::new(ExactMatcher {
                pattern: criteria.pattern.clone(),
                case_sensitive: criteria.case_sensitive,
            }),
            SearchType::Contains => Box::new(ContainsMatcher {
                pattern: criteria.pattern.clone(),
                case_sensitive: criteria.case_sensitive,
            }),
        };
        
        Ok(Self {
            criteria,
            pattern_matcher,
            results: Vec::new(),
            cancelled: false,
        })
    }
    
    pub async fn search(
        &mut self,
        start_path: &Path,
        progress_sender: mpsc::UnboundedSender<SearchProgress>,
    ) -> Result<Vec<SearchResult>> {
        let start_time = std::time::Instant::now();
        self.results.clear();
        self.cancelled = false;
        
        // Count total directories for progress
        let total_dirs = self.count_directories(start_path, 0)?;
        let _ = progress_sender.send(SearchProgress::Started { total_dirs });
        
        // Perform search
        let mut searched = 0;
        self.search_recursive(start_path, &progress_sender, &mut searched, total_dirs, 0).await?;
        
        // Send completion
        let _ = progress_sender.send(SearchProgress::Completed {
            total_found: self.results.len(),
            elapsed_ms: start_time.elapsed().as_millis(),
        });
        
        Ok(self.results.clone())
    }
    
    async fn search_recursive(
        &mut self,
        path: &Path,
        progress_sender: &mpsc::UnboundedSender<SearchProgress>,
        searched: &mut usize,
        total: usize,
        depth: usize,
    ) -> Result<()> {
        if self.cancelled {
            return Ok(());
        }
        
        // Check max depth
        if let Some(max_depth) = self.criteria.max_depth {
            if depth > max_depth {
                return Ok(());
            }
        }
        
        // Send progress
        *searched += 1;
        let _ = progress_sender.send(SearchProgress::Searching {
            current_path: path.to_path_buf(),
            searched: *searched,
            total,
        });
        
        // Read directory
        let entries = match fs::read_dir(path) {
            Ok(entries) => entries,
            Err(e) => {
                let _ = progress_sender.send(SearchProgress::Error {
                    path: path.to_path_buf(),
                    error: e.to_string(),
                });
                return Ok(());
            }
        };
        
        for entry in entries {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };
            
            let entry_path = entry.path();
            let metadata = match entry.metadata() {
                Ok(m) => m,
                Err(_) => continue,
            };
            
            // Skip hidden files if requested
            if !self.criteria.include_hidden {
                if let Some(name) = entry_path.file_name() {
                    if name.to_string_lossy().starts_with('.') {
                        continue;
                    }
                }
            }
            
            if metadata.is_dir() {
                // Recursively search subdirectories
                if self.criteria.include_subdirs {
                    Box::pin(self.search_recursive(
                        &entry_path,
                        progress_sender,
                        searched,
                        total,
                        depth + 1,
                    )).await?;
                }
            } else if metadata.is_file() {
                // Check if file matches criteria
                if self.matches_file(&entry_path, &metadata)? {
                    let mut result = SearchResult {
                        path: entry_path.clone(),
                        size: metadata.len(),
                        modified: metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH),
                        matches: Vec::new(),
                    };
                    
                    // Search inside file if requested
                    if self.criteria.search_in_files {
                        self.search_in_file(&entry_path, &mut result)?;
                    }
                    
                    let _ = progress_sender.send(SearchProgress::Found {
                        result: result.clone(),
                    });
                    
                    self.results.push(result);
                }
            }
        }
        
        Ok(())
    }
    
    fn matches_file(&self, path: &Path, metadata: &fs::Metadata) -> Result<bool> {
        // Check filename pattern
        if let Some(filename) = path.file_name() {
            let filename_str = filename.to_string_lossy();
            if !self.pattern_matcher.matches(&filename_str) && !self.criteria.search_in_files {
                return Ok(false);
            }
        }
        
        // Check extensions filter
        if !self.criteria.file_extensions.is_empty() {
            if let Some(ext) = path.extension() {
                let ext_str = ext.to_string_lossy().to_lowercase();
                if !self.criteria.file_extensions.iter().any(|e| e.to_lowercase() == ext_str) {
                    return Ok(false);
                }
            } else {
                return Ok(false);
            }
        }
        
        // Check size filter
        if let Some(ref size_filter) = self.criteria.size_filter {
            let size = metadata.len();
            if let Some(min) = size_filter.min_size {
                if size < min {
                    return Ok(false);
                }
            }
            if let Some(max) = size_filter.max_size {
                if size > max {
                    return Ok(false);
                }
            }
        }
        
        // Check date filter
        if let Some(ref date_filter) = self.criteria.date_filter {
            if let Ok(modified) = metadata.modified() {
                if let Some(after) = date_filter.after {
                    if modified < after {
                        return Ok(false);
                    }
                }
                if let Some(before) = date_filter.before {
                    if modified > before {
                        return Ok(false);
                    }
                }
            }
        }
        
        Ok(true)
    }
    
    fn search_in_file(&self, path: &Path, result: &mut SearchResult) -> Result<()> {
        // Only search in text files
        if let Ok(content) = fs::read_to_string(path) {
            for (line_num, line) in content.lines().enumerate() {
                if self.pattern_matcher.matches(line) {
                    result.matches.push(Match {
                        line_number: Some(line_num + 1),
                        line_content: Some(line.to_string()),
                        byte_offset: None,
                    });
                }
            }
        }
        Ok(())
    }
    
    fn count_directories(&self, path: &Path, depth: usize) -> Result<usize> {
        if let Some(max_depth) = self.criteria.max_depth {
            if depth > max_depth {
                return Ok(0);
            }
        }
        
        let mut count = 1;
        
        if self.criteria.include_subdirs {
            if let Ok(entries) = fs::read_dir(path) {
                for entry in entries {
                    if let Ok(entry) = entry {
                        if let Ok(metadata) = entry.metadata() {
                            if metadata.is_dir() {
                                count += self.count_directories(&entry.path(), depth + 1)?;
                            }
                        }
                    }
                }
            }
        }
        
        Ok(count)
    }
    
    pub fn cancel(&mut self) {
        self.cancelled = true;
    }
    
    pub fn get_results(&self) -> &[SearchResult] {
        &self.results
    }
}