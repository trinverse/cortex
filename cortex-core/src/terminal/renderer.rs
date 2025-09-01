#[cfg(feature = "windowed")]
use anyhow::Result;
#[cfg(feature = "windowed")]
use ratatui::{
    backend::Backend,
    buffer::Cell,
    prelude::*,
};
#[cfg(feature = "windowed")]
use std::sync::Arc;
#[cfg(feature = "windowed")]
use winit::window::Window;

#[cfg(feature = "windowed")]
pub struct TerminalRenderer {
    pub char_width: usize,
    pub char_height: usize,
    pub width_in_chars: usize,
    pub height_in_chars: usize,
    pub buffer: TerminalBuffer,
    pub window_width: u32,
    pub window_height: u32,
}

#[cfg(feature = "windowed")]
pub struct TerminalBuffer {
    cells: Vec<Vec<TerminalCell>>,
    width: usize,
    height: usize,
}

#[cfg(feature = "windowed")]
#[derive(Clone, Debug)]
pub struct TerminalCell {
    pub ch: char,
    pub fg: Color,
    pub bg: Color,
}

#[cfg(feature = "windowed")]
impl Default for TerminalCell {
    fn default() -> Self {
        Self {
            ch: ' ',
            fg: Color::White,
            bg: Color::Black,
        }
    }
}

#[cfg(feature = "windowed")]
impl TerminalRenderer {
    pub fn new(window: Arc<Window>) -> Result<Self> {
        let window_size = window.inner_size();
        
        // Terminal character dimensions (based on standard monospace font)
        let char_width = 9;  // Standard character width for terminal
        let char_height = 18; // Standard character height for terminal
        
        // Calculate terminal size in characters
        let width_in_chars = (window_size.width as usize) / char_width;
        let height_in_chars = (window_size.height as usize) / char_height;

        let buffer = TerminalBuffer::new(width_in_chars, height_in_chars);

        Ok(Self {
            char_width,
            char_height,
            width_in_chars,
            height_in_chars,
            buffer,
            window_width: window_size.width,
            window_height: window_size.height,
        })
    }

    pub fn render_frame(&mut self) -> Result<()> {
        // For now, just output to console - this is a placeholder
        // TODO: Implement actual pixel rendering later
        println!("📺 Rendering terminal frame with {} chars in buffer", 
                 self.buffer.cells.iter().map(|row| row.len()).sum::<usize>());
        
        // Show first few lines of content
        for (i, row) in self.buffer.cells.iter().enumerate().take(5) {
            let line: String = row.iter().map(|cell| cell.ch).collect();
            if !line.trim().is_empty() {
                println!("  Row {}: {}", i, line);
            }
        }
        
        Ok(())
    }

    pub fn resize(&mut self, new_width: u32, new_height: u32) -> Result<()> {
        // Update dimensions
        self.window_width = new_width;
        self.window_height = new_height;
        
        // Update character dimensions
        let width_in_chars = (new_width as usize) / self.char_width;
        let height_in_chars = (new_height as usize) / self.char_height;
        
        self.width_in_chars = width_in_chars;
        self.height_in_chars = height_in_chars;
        
        // Resize buffer
        self.buffer = TerminalBuffer::new(width_in_chars, height_in_chars);
        
        Ok(())
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    pub fn get_backend(&mut self) -> PixelBackend<'_> {
        PixelBackend::new(&mut self.buffer)
    }
}

#[cfg(feature = "windowed")]
impl TerminalBuffer {
    pub fn new(width: usize, height: usize) -> Self {
        let mut cells = Vec::with_capacity(height);
        for _ in 0..height {
            cells.push(vec![TerminalCell::default(); width]);
        }
        
        Self {
            cells,
            width,
            height,
        }
    }

    pub fn clear(&mut self) {
        for row in &mut self.cells {
            for cell in row {
                *cell = TerminalCell::default();
            }
        }
    }

    pub fn set_cell(&mut self, x: usize, y: usize, cell: TerminalCell) {
        if x < self.width && y < self.height {
            self.cells[y][x] = cell;
        }
    }

    pub fn get_cell(&self, x: usize, y: usize) -> Option<&TerminalCell> {
        if x < self.width && y < self.height {
            Some(&self.cells[y][x])
        } else {
            None
        }
    }
}

// Ratatui backend implementation for pixel rendering
#[cfg(feature = "windowed")]
pub struct PixelBackend<'a> {
    buffer: &'a mut TerminalBuffer,
}

#[cfg(feature = "windowed")]
impl<'a> PixelBackend<'a> {
    pub fn new(buffer: &'a mut TerminalBuffer) -> Self {
        Self { buffer }
    }
}

#[cfg(feature = "windowed")]
impl<'a> Backend for PixelBackend<'a> {
    fn draw<'b, I>(&mut self, content: I) -> std::io::Result<()>
    where
        I: Iterator<Item = (u16, u16, &'b Cell)>,
    {
        for (x, y, cell) in content {
            let terminal_cell = TerminalCell {
                ch: cell.symbol().chars().next().unwrap_or(' '),
                fg: cell.fg,
                bg: cell.bg,
            };
            self.buffer.set_cell(x as usize, y as usize, terminal_cell);
        }
        Ok(())
    }

    fn hide_cursor(&mut self) -> std::io::Result<()> {
        // No cursor to hide in pixel mode
        Ok(())
    }

    fn show_cursor(&mut self) -> std::io::Result<()> {
        // No cursor to show in pixel mode  
        Ok(())
    }

    fn get_cursor(&mut self) -> std::io::Result<(u16, u16)> {
        // Return dummy cursor position
        Ok((0, 0))
    }

    fn set_cursor(&mut self, _x: u16, _y: u16) -> std::io::Result<()> {
        // No cursor to set in pixel mode
        Ok(())
    }

    fn clear(&mut self) -> std::io::Result<()> {
        self.buffer.clear();
        Ok(())
    }

    fn size(&self) -> std::io::Result<Rect> {
        Ok(Rect::new(
            0,
            0,
            self.buffer.width as u16,
            self.buffer.height as u16,
        ))
    }

    fn flush(&mut self) -> std::io::Result<()> {
        // Nothing to flush in our case
        Ok(())
    }

    fn window_size(&mut self) -> std::io::Result<ratatui::backend::WindowSize> {
        Ok(ratatui::backend::WindowSize {
            columns_rows: ratatui::layout::Size::new(self.buffer.width as u16, self.buffer.height as u16),
            pixels: ratatui::layout::Size::new(800, 600), // Default pixel size
        })
    }
}

