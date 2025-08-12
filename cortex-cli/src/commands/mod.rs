// Command processing module
mod executor;
mod special;
mod plugin;

pub use executor::process_command;
pub use special::handle_special_command;
pub use plugin::handle_plugin_command;

/// Parse and identify command type
pub fn parse_command(command: &str) -> CommandType {
    if command.starts_with('/') {
        CommandType::Special(command[1..].to_string())
    } else if command.starts_with(':') {
        CommandType::Plugin(command[1..].to_string())
    } else {
        CommandType::Shell(command.to_string())
    }
}

#[derive(Debug)]
pub enum CommandType {
    Shell(String),
    Special(String),
    Plugin(String),
}