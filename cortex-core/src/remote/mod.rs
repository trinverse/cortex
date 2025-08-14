#[cfg(feature = "ssh")]
pub mod ssh_manager;
#[cfg(feature = "ssh")]
pub mod sftp_provider;
#[cfg(feature = "ssh")]
pub mod ftp_provider;
#[cfg(all(feature = "ssh", test))]
mod tests;

#[cfg(feature = "ssh")]
pub use ssh_manager::SshConnectionManager;
#[cfg(feature = "ssh")]
pub use sftp_provider::SftpProvider;
#[cfg(feature = "ssh")]
pub use ftp_provider::FtpProvider;