// Configuration management
use cortex_core::AppState;

/// Apply configuration to the application state
pub fn apply_configuration(state: &mut AppState) {
    let config = state.config_manager.get();
    
    // Apply panel settings
    state.left_panel.show_hidden = false;  // Default to not showing hidden
    state.right_panel.show_hidden = false;
    
    // Apply auto-reload setting
    state.auto_reload_enabled = config.general.auto_reload;
}