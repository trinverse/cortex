use anyhow::Result;
use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct FileViewer {
    pub path: PathBuf,
    pub title: String,
    pub lines: Vec<String>,
    pub offset: usize,
    pub selected_line: usize,
    pub search_term: Option<String>,
    pub hex_mode: bool,
    pub wrap_lines: bool,
    pub file_size: u64,
    pub encoding: String,
}

impl FileViewer {
    pub fn new(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let title = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown")
            .to_string();

        let file = File::open(&path)?;
        let metadata = file.metadata()?;
        let file_size = metadata.len();

        Ok(Self {
            path,
            title,
            lines: Vec::new(),
            offset: 0,
            selected_line: 0,
            search_term: None,
            hex_mode: false,
            wrap_lines: true,
            file_size,
            encoding: "UTF-8".to_string(),
        })
    }

    pub fn load_content(&mut self, max_lines: usize) -> Result<()> {
        if self.hex_mode {
            self.load_hex_content(max_lines)?;
        } else {
            self.load_text_content(max_lines)?;
        }
        Ok(())
    }

    fn load_text_content(&mut self, max_lines: usize) -> Result<()> {
        let file = File::open(&self.path)?;
        let reader = BufReader::new(file);

        self.lines.clear();

        for (i, line) in reader.lines().enumerate() {
            if i < self.offset {
                continue;
            }
            if self.lines.len() >= max_lines {
                break;
            }

            match line {
                Ok(text) => {
                    if self.wrap_lines {
                        self.lines.push(text);
                    } else {
                        self.lines.push(text);
                    }
                }
                Err(_) => {
                    self.lines.push("<Error reading line>".to_string());
                }
            }
        }

        Ok(())
    }

    fn load_hex_content(&mut self, max_lines: usize) -> Result<()> {
        use std::io::Read;

        let mut file = File::open(&self.path)?;
        let start_pos = self.offset * 16; // 16 bytes per line in hex view
        file.seek(SeekFrom::Start(start_pos as u64))?;

        self.lines.clear();
        let mut buffer = [0u8; 16];

        for _ in 0..max_lines {
            match file.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => {
                    let hex_part: String =
                        buffer[..n].iter().map(|b| format!("{:02X} ", b)).collect();

                    let ascii_part: String = buffer[..n]
                        .iter()
                        .map(|&b| {
                            if b.is_ascii_graphic() || b == b' ' {
                                b as char
                            } else {
                                '.'
                            }
                        })
                        .collect();

                    let address = start_pos + (self.lines.len() * 16);
                    self.lines
                        .push(format!("{:08X}  {:<48}  {}", address, hex_part, ascii_part));
                }
                Err(_) => break,
            }
        }

        Ok(())
    }

    pub fn scroll_up(&mut self, amount: usize) {
        self.offset = self.offset.saturating_sub(amount);
    }

    pub fn scroll_down(&mut self, amount: usize) {
        self.offset += amount;
    }

    pub fn page_up(&mut self, page_size: usize) {
        self.scroll_up(page_size);
    }

    pub fn page_down(&mut self, page_size: usize) {
        self.scroll_down(page_size);
    }

    pub fn toggle_hex_mode(&mut self) {
        self.hex_mode = !self.hex_mode;
        self.offset = 0;
    }

    pub fn toggle_wrap(&mut self) {
        self.wrap_lines = !self.wrap_lines;
    }

    pub fn search(&mut self, term: &str) -> Option<usize> {
        self.search_term = Some(term.to_string());

        for (i, line) in self.lines.iter().enumerate() {
            if line.to_lowercase().contains(&term.to_lowercase()) {
                self.selected_line = i;
                return Some(i);
            }
        }

        None
    }

    pub fn search_next(&mut self) -> Option<usize> {
        if let Some(term) = &self.search_term.clone() {
            for (i, line) in self.lines.iter().enumerate().skip(self.selected_line + 1) {
                if line.to_lowercase().contains(&term.to_lowercase()) {
                    self.selected_line = i;
                    return Some(i);
                }
            }
        }
        None
    }

    pub fn get_status(&self) -> String {
        let mode = if self.hex_mode { "HEX" } else { "TEXT" };
        let wrap = if self.wrap_lines { "WRAP" } else { "NOWRAP" };

        format!(
            "{} | {} | Line: {} | Size: {} | {}",
            mode,
            self.encoding,
            self.offset + self.selected_line + 1,
            humansize::format_size(self.file_size, humansize::BINARY),
            wrap
        )
    }
}
