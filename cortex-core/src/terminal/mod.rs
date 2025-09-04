pub mod emulator;
pub mod manager;
pub mod shell;

#[cfg(feature = "windowed")]
pub mod renderer;

pub use emulator::{TerminalEmulator, TerminalEvent, TerminalSize};
pub use manager::{TerminalManager, TerminalSession};
pub use shell::{ShellConfig, ShellType};

#[cfg(feature = "windowed")]
pub use renderer::{TerminalRenderer, TerminalCell};