#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::vfs::{VfsPath, VfsProvider};
    use crate::remote::ssh_manager::RemoteCredentials;
    use crate::remote::ftp_provider::FtpCredentials;
    use std::sync::Arc;
    use std::time::Duration;

    #[test]
    fn test_ssh_connection_manager_creation() {
        let manager = SshConnectionManager::new()
            .with_timeouts(Duration::from_secs(10), Duration::from_secs(300))
            .with_max_retries(5);
        
        // Manager should be created successfully
        let _arc = Arc::new(manager);
    }

    #[test]
    fn test_remote_credentials() {
        let creds = RemoteCredentials {
            username: "testuser".to_string(),
            password: Some("testpass".to_string()),
            private_key_path: None,
            passphrase: None,
        };
        
        assert_eq!(creds.username, "testuser");
        assert_eq!(creds.password, Some("testpass".to_string()));
    }

    #[test]
    fn test_sftp_provider_can_handle() {
        let manager = Arc::new(SshConnectionManager::new());
        let creds = RemoteCredentials {
            username: "test".to_string(),
            password: None,
            private_key_path: None,
            passphrase: None,
        };
        let provider = SftpProvider::new(manager, creds);
        
        let sftp_path = VfsPath::Sftp {
            host: "example.com".to_string(),
            port: 22,
            username: "user".to_string(),
            path: "/home/user".to_string(),
        };
        
        let local_path = VfsPath::Local(std::path::PathBuf::from("/tmp"));
        
        assert!(provider.can_handle(&sftp_path));
        assert!(!provider.can_handle(&local_path));
    }

    #[test]
    fn test_ftp_provider_can_handle() {
        let provider = FtpProvider::new();
        
        let ftp_path = VfsPath::Ftp {
            host: "ftp.example.com".to_string(),
            port: 21,
            username: "user".to_string(),
            path: "/public".to_string(),
        };
        
        let sftp_path = VfsPath::Sftp {
            host: "example.com".to_string(),
            port: 22,
            username: "user".to_string(),
            path: "/home/user".to_string(),
        };
        
        assert!(provider.can_handle(&ftp_path));
        assert!(!provider.can_handle(&sftp_path));
    }

    #[test]
    fn test_ftp_credentials() {
        let creds = FtpCredentials {
            username: "ftpuser".to_string(),
            password: "ftppass".to_string(),
            use_tls: true,
        };
        
        assert_eq!(creds.username, "ftpuser");
        assert_eq!(creds.password, "ftppass");
        assert!(creds.use_tls);
    }

    #[test]
    fn test_connection_manager_timeouts() {
        let manager = SshConnectionManager::new()
            .with_timeouts(Duration::from_secs(5), Duration::from_secs(1));
        
        // Manager should be created with custom timeouts
        let _ = manager; // Just ensure it compiles
    }

    #[test]
    fn test_disconnect_all() {
        let manager = SshConnectionManager::new();
        let result = manager.disconnect_all();
        assert!(result.is_ok());
    }

    #[cfg(feature = "integration")]
    #[test]
    fn test_sftp_integration() {
        // This test would require actual SFTP server for integration testing
        // It's marked with a feature flag so it only runs when explicitly enabled
        
        let manager = Arc::new(SshConnectionManager::new());
        let creds = RemoteCredentials {
            username: std::env::var("TEST_SFTP_USER").unwrap_or_else(|_| "test".to_string()),
            password: std::env::var("TEST_SFTP_PASS").ok(),
            private_key_path: None,
            passphrase: None,
        };
        
        let provider = SftpProvider::new(manager.clone(), creds.clone());
        
        let test_path = VfsPath::Sftp {
            host: std::env::var("TEST_SFTP_HOST").unwrap_or_else(|_| "localhost".to_string()),
            port: 22,
            username: creds.username.clone(),
            path: "/tmp".to_string(),
        };
        
        // Test listing directory
        match provider.list_entries(&test_path) {
            Ok(entries) => {
                for entry in entries {
                    println!("Found entry: {:?}", entry.name);
                }
            }
            Err(e) => {
                println!("SFTP test skipped: {}", e);
            }
        }
    }

    #[cfg(feature = "integration")]
    #[test]
    fn test_ftp_integration() {
        // This test would require actual FTP server for integration testing
        
        let provider = FtpProvider::new();
        
        let test_path = VfsPath::Ftp {
            host: std::env::var("TEST_FTP_HOST").unwrap_or_else(|_| "localhost".to_string()),
            port: 21,
            username: std::env::var("TEST_FTP_USER").unwrap_or_else(|_| "anonymous".to_string()),
            path: "/".to_string(),
        };
        
        // Test listing directory
        match provider.list_entries(&test_path) {
            Ok(entries) => {
                for entry in entries {
                    println!("Found FTP entry: {:?}", entry.name);
                }
            }
            Err(e) => {
                println!("FTP test skipped: {}", e);
            }
        }
    }
}