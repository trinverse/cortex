use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct TextEditor {
    pub path: PathBuf,
    pub title: String,
    pub lines: Vec<String>,
    pub cursor_row: usize,
    pub cursor_col: usize,
    pub offset_row: usize,
    pub offset_col: usize,
    pub modified: bool,
    pub insert_mode: bool,
    pub selection_start: Option<(usize, usize)>,
    pub selection_end: Option<(usize, usize)>,
    pub undo_stack: Vec<EditorState>,
    pub redo_stack: Vec<EditorState>,
    pub search_term: Option<String>,
    pub status_message: Option<String>,
}

#[derive(Debug, Clone)]
pub struct EditorState {
    lines: Vec<String>,
    cursor_row: usize,
    cursor_col: usize,
}

impl TextEditor {
    pub fn new(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let title = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Untitled")
            .to_string();

        let content = if path.exists() {
            fs::read_to_string(&path)?
        } else {
            String::new()
        };

        let lines: Vec<String> = if content.is_empty() {
            vec![String::new()]
        } else {
            content.lines().map(|s| s.to_string()).collect()
        };

        Ok(Self {
            path,
            title,
            lines,
            cursor_row: 0,
            cursor_col: 0,
            offset_row: 0,
            offset_col: 0,
            modified: false,
            insert_mode: true,
            selection_start: None,
            selection_end: None,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            search_term: None,
            status_message: None,
        })
    }

    pub fn save(&mut self) -> Result<()> {
        let content = self.lines.join("\n");
        fs::write(&self.path, content)?;
        self.modified = false;
        self.status_message = Some(format!("Saved {}", self.path.display()));
        Ok(())
    }

    pub fn save_as(&mut self, path: impl AsRef<Path>) -> Result<()> {
        self.path = path.as_ref().to_path_buf();
        self.title = self
            .path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Untitled")
            .to_string();
        self.save()
    }

    fn save_state(&mut self) {
        self.undo_stack.push(EditorState {
            lines: self.lines.clone(),
            cursor_row: self.cursor_row,
            cursor_col: self.cursor_col,
        });
        self.redo_stack.clear();

        // Limit undo stack size
        if self.undo_stack.len() > 100 {
            self.undo_stack.remove(0);
        }
    }

    pub fn undo(&mut self) {
        if let Some(state) = self.undo_stack.pop() {
            self.redo_stack.push(EditorState {
                lines: self.lines.clone(),
                cursor_row: self.cursor_row,
                cursor_col: self.cursor_col,
            });

            self.lines = state.lines;
            self.cursor_row = state.cursor_row;
            self.cursor_col = state.cursor_col;
            self.modified = true;
        }
    }

    pub fn redo(&mut self) {
        if let Some(state) = self.redo_stack.pop() {
            self.undo_stack.push(EditorState {
                lines: self.lines.clone(),
                cursor_row: self.cursor_row,
                cursor_col: self.cursor_col,
            });

            self.lines = state.lines;
            self.cursor_row = state.cursor_row;
            self.cursor_col = state.cursor_col;
            self.modified = true;
        }
    }

    pub fn insert_char(&mut self, ch: char) {
        self.save_state();

        let line = &mut self.lines[self.cursor_row];
        line.insert(self.cursor_col, ch);
        self.cursor_col += 1;
        self.modified = true;
    }

    pub fn insert_newline(&mut self) {
        self.save_state();

        let current_line = self.lines[self.cursor_row].clone();
        let (before, after) = current_line.split_at(self.cursor_col);

        self.lines[self.cursor_row] = before.to_string();
        self.lines.insert(self.cursor_row + 1, after.to_string());

        self.cursor_row += 1;
        self.cursor_col = 0;
        self.modified = true;
    }

    pub fn delete_char(&mut self) {
        if self.cursor_col > 0 {
            self.save_state();
            let line = &mut self.lines[self.cursor_row];
            self.cursor_col -= 1;
            line.remove(self.cursor_col);
            self.modified = true;
        } else if self.cursor_row > 0 {
            self.save_state();
            let current_line = self.lines.remove(self.cursor_row);
            self.cursor_row -= 1;
            self.cursor_col = self.lines[self.cursor_row].len();
            self.lines[self.cursor_row].push_str(&current_line);
            self.modified = true;
        }
    }

    pub fn delete_forward(&mut self) {
        let line_len = self.lines[self.cursor_row].len();
        if self.cursor_col < line_len {
            self.save_state();
            self.lines[self.cursor_row].remove(self.cursor_col);
            self.modified = true;
        } else if self.cursor_row < self.lines.len() - 1 {
            self.save_state();
            let next_line = self.lines.remove(self.cursor_row + 1);
            self.lines[self.cursor_row].push_str(&next_line);
            self.modified = true;
        }
    }

    pub fn move_cursor_up(&mut self) {
        if self.cursor_row > 0 {
            self.cursor_row -= 1;
            let line_len = self.lines[self.cursor_row].len();
            if self.cursor_col > line_len {
                self.cursor_col = line_len;
            }
        }
    }

    pub fn move_cursor_down(&mut self) {
        if self.cursor_row < self.lines.len() - 1 {
            self.cursor_row += 1;
            let line_len = self.lines[self.cursor_row].len();
            if self.cursor_col > line_len {
                self.cursor_col = line_len;
            }
        }
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        } else if self.cursor_row > 0 {
            self.cursor_row -= 1;
            self.cursor_col = self.lines[self.cursor_row].len();
        }
    }

    pub fn move_cursor_right(&mut self) {
        let line_len = self.lines[self.cursor_row].len();
        if self.cursor_col < line_len {
            self.cursor_col += 1;
        } else if self.cursor_row < self.lines.len() - 1 {
            self.cursor_row += 1;
            self.cursor_col = 0;
        }
    }

    pub fn move_cursor_home(&mut self) {
        self.cursor_col = 0;
    }

    pub fn move_cursor_end(&mut self) {
        self.cursor_col = self.lines[self.cursor_row].len();
    }

    pub fn move_cursor_page_up(&mut self, page_size: usize) {
        self.cursor_row = self.cursor_row.saturating_sub(page_size);
        let line_len = self.lines[self.cursor_row].len();
        if self.cursor_col > line_len {
            self.cursor_col = line_len;
        }
    }

    pub fn move_cursor_page_down(&mut self, page_size: usize) {
        let max_row = self.lines.len() - 1;
        self.cursor_row = (self.cursor_row + page_size).min(max_row);
        let line_len = self.lines[self.cursor_row].len();
        if self.cursor_col > line_len {
            self.cursor_col = line_len;
        }
    }

    pub fn search(&mut self, term: &str) -> Option<(usize, usize)> {
        self.search_term = Some(term.to_string());

        for (row, line) in self.lines.iter().enumerate().skip(self.cursor_row) {
            if let Some(col) = line.to_lowercase().find(&term.to_lowercase()) {
                if row > self.cursor_row || col >= self.cursor_col {
                    self.cursor_row = row;
                    self.cursor_col = col;
                    return Some((row, col));
                }
            }
        }

        // Wrap search from beginning
        for (row, line) in self.lines.iter().enumerate().take(self.cursor_row + 1) {
            if let Some(col) = line.to_lowercase().find(&term.to_lowercase()) {
                self.cursor_row = row;
                self.cursor_col = col;
                return Some((row, col));
            }
        }

        None
    }

    pub fn search_next(&mut self) -> Option<(usize, usize)> {
        if let Some(term) = &self.search_term.clone() {
            // Move cursor forward to search for next occurrence
            self.move_cursor_right();
            self.search(&term)
        } else {
            None
        }
    }

    pub fn replace(&mut self, search: &str, replace: &str, all: bool) {
        self.save_state();

        if all {
            for line in &mut self.lines {
                *line = line.replace(search, replace);
            }
            self.modified = true;
        } else {
            if let Some((row, col)) = self.search(search) {
                let line = &mut self.lines[row];
                line.replace_range(col..col + search.len(), replace);
                self.modified = true;
            }
        }
    }

    pub fn update_view_offset(&mut self, window_height: usize, window_width: usize) {
        // Vertical scrolling
        if self.cursor_row < self.offset_row {
            self.offset_row = self.cursor_row;
        } else if self.cursor_row >= self.offset_row + window_height {
            self.offset_row = self.cursor_row - window_height + 1;
        }

        // Horizontal scrolling
        if self.cursor_col < self.offset_col {
            self.offset_col = self.cursor_col;
        } else if self.cursor_col >= self.offset_col + window_width {
            self.offset_col = self.cursor_col - window_width + 1;
        }
    }

    pub fn get_status(&self) -> String {
        let mode = if self.insert_mode { "INSERT" } else { "NORMAL" };
        let modified = if self.modified { "[+]" } else { "" };

        format!(
            "{} {} | Line {}/{} Col {} {}",
            mode,
            self.title,
            self.cursor_row + 1,
            self.lines.len(),
            self.cursor_col + 1,
            modified
        )
    }
}
