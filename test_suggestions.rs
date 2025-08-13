use cortex_core::AppState;

fn main() {
    // Test the suggestion function
    let mut state = AppState::new().unwrap();
    
    // Test with "cd "
    state.command_line = "cd ".to_string();
    state.update_command_suggestions();
    println!("Command: '{}'", state.command_line);
    println!("Suggestions: {:?}", state.command_suggestions);
    
    // Test with "cd"
    state.command_line = "cd".to_string();
    state.update_command_suggestions();
    println!("\nCommand: '{}'", state.command_line);
    println!("Suggestions: {:?}", state.command_suggestions);
    
    // Test with partial path
    state.command_line = "cd c".to_string();
    state.update_command_suggestions();
    println!("\nCommand: '{}'", state.command_line);
    println!("Suggestions: {:?}", state.command_suggestions);
}