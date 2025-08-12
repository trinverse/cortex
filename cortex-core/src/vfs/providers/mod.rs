pub mod archive;
pub mod ftp;
pub mod local;
pub mod ssh;

pub use archive::ArchiveProvider;
pub use local::LocalFileSystemProvider;

#[cfg(feature = "ssh")]
pub use ftp::FtpProvider;
#[cfg(feature = "ssh")]
pub use ssh::{SftpProvider, SshConnectionManager};