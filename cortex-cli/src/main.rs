use anyhow::Result;
use clap::Parser;
use cortex_core::{AppState, FileSystem};
use cortex_tui::{Event, EventHandler, KeyBinding, UI};
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{io, path::PathBuf, time::Duration};

#[derive(Parser, Debug)]
#[command(name = "cortex")]
#[command(about = "A modern orthodox file manager", long_about = None)]
struct Args {
    #[arg(help = "Directory to open")]
    path: Option<PathBuf>,

    #[arg(short, long, help = "Show version information")]
    version: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    
    let args = Args::parse();
    
    if args.version {
        println!("Cortex v{}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    let mut app = App::new(args.path)?;
    app.run().await
}

struct App {
    state: AppState,
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    events: EventHandler,
}

impl App {
    fn new(initial_path: Option<PathBuf>) -> Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        
        let mut state = AppState::new()?;
        
        if let Some(path) = initial_path {
            if path.is_dir() {
                state.left_panel.current_dir = path.clone();
                state.right_panel.current_dir = path;
            }
        }
        
        Self::refresh_panel(&mut state.left_panel)?;
        Self::refresh_panel(&mut state.right_panel)?;
        
        let events = EventHandler::new(Duration::from_millis(100));
        
        Ok(Self {
            state,
            terminal,
            events,
        })
    }

    async fn run(&mut self) -> Result<()> {
        loop {
            self.terminal.draw(|frame| UI::draw(frame, &self.state))?;
            
            match self.events.next().await? {
                Event::Key(key_event) => {
                    if let Some(binding) = KeyBinding::from_key_event(key_event) {
                        if !self.handle_input(binding).await? {
                            break;
                        }
                    }
                }
                Event::Resize(_, _) => {
                    self.terminal.autoresize()?;
                }
                Event::Tick => {}
            }
        }
        
        Ok(())
    }

    async fn handle_input(&mut self, binding: KeyBinding) -> Result<bool> {
        use cortex_core::FileType;
        
        match binding {
            KeyBinding::Quit => return Ok(false),
            
            KeyBinding::Up => {
                let panel = self.state.active_panel_mut();
                panel.move_selection_up();
                panel.update_view_offset(self.terminal.size()?.height as usize - 5);
            }
            
            KeyBinding::Down => {
                let panel = self.state.active_panel_mut();
                panel.move_selection_down();
                panel.update_view_offset(self.terminal.size()?.height as usize - 5);
            }
            
            KeyBinding::PageUp => {
                let height = self.terminal.size()?.height as usize - 5;
                let panel = self.state.active_panel_mut();
                panel.move_selection_page_up(height);
                panel.update_view_offset(height);
            }
            
            KeyBinding::PageDown => {
                let height = self.terminal.size()?.height as usize - 5;
                let panel = self.state.active_panel_mut();
                panel.move_selection_page_down(height);
                panel.update_view_offset(height);
            }
            
            KeyBinding::Home => {
                let panel = self.state.active_panel_mut();
                panel.move_selection_home();
                panel.update_view_offset(self.terminal.size()?.height as usize - 5);
            }
            
            KeyBinding::End => {
                let panel = self.state.active_panel_mut();
                panel.move_selection_end();
                panel.update_view_offset(self.terminal.size()?.height as usize - 5);
            }
            
            KeyBinding::Enter | KeyBinding::Right => {
                let panel = self.state.active_panel_mut();
                if let Some(entry) = panel.current_entry() {
                    if entry.file_type == FileType::Directory {
                        let new_dir = if entry.name == ".." {
                            panel.current_dir.parent().map(|p| p.to_path_buf())
                        } else {
                            Some(entry.path.clone())
                        };
                        
                        if let Some(dir) = new_dir {
                            panel.current_dir = dir;
                            panel.selected_index = 0;
                            panel.view_offset = 0;
                            Self::refresh_panel(panel)?;
                        }
                    }
                }
            }
            
            KeyBinding::Back | KeyBinding::Left => {
                let panel = self.state.active_panel_mut();
                if let Some(parent) = panel.current_dir.parent() {
                    panel.current_dir = parent.to_path_buf();
                    panel.selected_index = 0;
                    panel.view_offset = 0;
                    Self::refresh_panel(panel)?;
                }
            }
            
            KeyBinding::Tab => {
                self.state.toggle_panel();
            }
            
            KeyBinding::ToggleHidden => {
                let panel = self.state.active_panel_mut();
                panel.show_hidden = !panel.show_hidden;
                Self::refresh_panel(panel)?;
            }
            
            KeyBinding::ToggleMark => {
                let panel = self.state.active_panel_mut();
                panel.toggle_mark_current();
                panel.move_selection_down();
                panel.update_view_offset(self.terminal.size()?.height as usize - 5);
            }
            
            KeyBinding::Refresh => {
                Self::refresh_panel(self.state.active_panel_mut())?;
                let inactive = match self.state.active_panel {
                    cortex_core::ActivePanel::Left => &mut self.state.right_panel,
                    cortex_core::ActivePanel::Right => &mut self.state.left_panel,
                };
                Self::refresh_panel(inactive)?;
            }
            
            _ => {}
        }
        
        Ok(true)
    }

    fn refresh_panel(panel: &mut cortex_core::PanelState) -> Result<()> {
        panel.entries = FileSystem::list_directory(&panel.current_dir, panel.show_hidden)?;
        panel.sort_entries();
        
        if panel.selected_index >= panel.entries.len() && !panel.entries.is_empty() {
            panel.selected_index = panel.entries.len() - 1;
        }
        
        Ok(())
    }
}

impl Drop for App {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(self.terminal.backend_mut(), LeaveAlternateScreen);
    }
}