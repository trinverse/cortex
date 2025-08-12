// Connection management module
use anyhow::Result;
use cortex_core::{RemoteCredentials, VfsPath};
use cortex_tui::{Dialog, InputDialog};
use crate::app::App;

/// Connect to SFTP server
pub async fn connect_sftp(
    app: &mut App,
    host: String,
    port: u16,
    username: String,
) -> Result<()> {
    // Check if we have cached credentials
    let connection_key = format!("{}:{}@{}", username, port, host);
    
    if let Some(_credentials) = app.state.remote_connections.get(&connection_key) {
        // Use cached credentials
        let vfs_path = VfsPath::Sftp {
            host: host.clone(),
            port,
            username: username.clone(),
            path: "/".to_string(),
        };
        
        match app.state.navigate_into_vfs(vfs_path) {
            Ok(_) => {
                app.state.set_status_message(format!("Connected to {}@{}", username, host));
            }
            Err(e) => {
                app.state.set_status_message(format!("Connection failed: {}", e));
            }
        }
    } else {
        // Need to get password
        app.dialog = Some(Dialog::Input(InputDialog::new(
            &format!("Password for {}@{}", username, host),
            "Enter password:",
        )));
        
        // Store connection info for later use
        app.state.store_connection_credentials(
            &host,
            port,
            &username,
            RemoteCredentials {
                username: username.clone(),
                password: Some(String::new()), // Placeholder - will be populated after dialog
                private_key_path: None,
                passphrase: None,
            },
        );
    }
    
    Ok(())
}

/// Connect to FTP server
pub async fn connect_ftp(
    app: &mut App,
    host: String,
    port: u16,
    username: String,
) -> Result<()> {
    // Similar to SFTP but for FTP
    let connection_key = format!("{}:{}@{}", username, port, host);
    
    if let Some(_credentials) = app.state.remote_connections.get(&connection_key) {
        let vfs_path = VfsPath::Ftp {
            host: host.clone(),
            port,
            username: username.clone(),
            path: "/".to_string(),
        };
        
        match app.state.navigate_into_vfs(vfs_path) {
            Ok(_) => {
                app.state.set_status_message(format!("Connected to ftp://{}@{}", username, host));
            }
            Err(e) => {
                app.state.set_status_message(format!("FTP connection failed: {}", e));
            }
        }
    } else {
        app.dialog = Some(Dialog::Input(InputDialog::new(
            &format!("FTP Password for {}@{}", username, host),
            "Enter password:",
        )));
        
        app.state.store_connection_credentials(
            &host,
            port,
            &username,
            RemoteCredentials {
                username: username.clone(),
                password: Some(String::new()), // Placeholder - will be populated after dialog
                private_key_path: None,
                passphrase: None,
            },
        );
    }
    
    Ok(())
}

/// Disconnect from current remote connection
pub fn _disconnect(app: &mut App) -> Result<()> {
    if app.state.active_panel().is_using_vfs() {
        app.state.navigate_back_from_vfs()?;
        app.state.set_status_message("Disconnected from remote");
    } else {
        app.state.set_status_message("Not connected to any remote");
    }
    Ok(())
}

/// List active connections
pub fn _list_connections(app: &App) -> Vec<String> {
    app.state.remote_connections
        .keys()
        .cloned()
        .collect()
}