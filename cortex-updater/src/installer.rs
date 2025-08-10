use anyhow::{Context, Result};
use semver::Version;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

/// Update installer
pub struct Installer {
    install_dir: PathBuf,
}

impl Installer {
    pub fn new() -> Self {
        Self {
            install_dir: Self::get_install_directory(),
        }
    }

    /// Install an update from a downloaded package
    pub async fn install(&self, package_path: &Path, version: &Version) -> Result<()> {
        // Verify package exists
        if !package_path.exists() {
            anyhow::bail!("Update package not found: {:?}", package_path);
        }

        // Create temp extraction directory
        let temp_dir = TempDir::new()?;
        let extract_dir = temp_dir.path();

        // Extract based on package type
        self.extract_package(package_path, extract_dir)?;

        // Verify extracted files
        self.verify_installation(extract_dir, version)?;

        // Stop current instance if running
        self.stop_current_instance()?;

        // Perform atomic installation
        self.atomic_install(extract_dir)?;

        // Update version file
        self.update_version_file(version)?;

        // Clean up
        temp_dir.close()?;

        Ok(())
    }

    /// Extract update package
    fn extract_package(&self, package_path: &Path, extract_dir: &Path) -> Result<()> {
        let extension = package_path
            .extension()
            .and_then(|e| e.to_str())
            .context("Invalid package extension")?;

        match extension {
            "gz" | "tar" => self.extract_tar_gz(package_path, extract_dir),
            "zip" => self.extract_zip(package_path, extract_dir),
            _ => anyhow::bail!("Unsupported package format: {}", extension),
        }
    }

    /// Extract tar.gz archive
    fn extract_tar_gz(&self, package_path: &Path, extract_dir: &Path) -> Result<()> {
        use flate2::read::GzDecoder;
        use tar::Archive;

        let file = fs::File::open(package_path)?;
        let decoder = GzDecoder::new(file);
        let mut archive = Archive::new(decoder);

        archive.unpack(extract_dir)?;

        Ok(())
    }

    /// Extract zip archive (Windows)
    #[cfg(target_os = "windows")]
    fn extract_zip(&self, package_path: &Path, extract_dir: &Path) -> Result<()> {
        use zip::ZipArchive;

        let file = fs::File::open(package_path)?;
        let mut archive = ZipArchive::new(file)?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let path = extract_dir.join(file.name());

            if file.is_dir() {
                fs::create_dir_all(&path)?;
            } else {
                if let Some(parent) = path.parent() {
                    fs::create_dir_all(parent)?;
                }
                let mut outfile = fs::File::create(&path)?;
                std::io::copy(&mut file, &mut outfile)?;
            }
        }

        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    fn extract_zip(&self, _package_path: &Path, _extract_dir: &Path) -> Result<()> {
        anyhow::bail!("ZIP extraction not supported on this platform")
    }

    /// Verify extracted installation files
    fn verify_installation(&self, extract_dir: &Path, version: &Version) -> Result<()> {
        // Check for main executable
        let executable = if cfg!(target_os = "windows") {
            extract_dir.join("cortex.exe")
        } else {
            extract_dir.join("cortex")
        };

        if !executable.exists() {
            anyhow::bail!("Main executable not found in update package");
        }

        // Verify version
        let output = Command::new(&executable).arg("--version").output()?;

        let version_str = String::from_utf8_lossy(&output.stdout);
        if !version_str.contains(&version.to_string()) {
            anyhow::bail!(
                "Version mismatch. Expected {}, got {}",
                version,
                version_str
            );
        }

        Ok(())
    }

    /// Stop the current running instance
    fn stop_current_instance(&self) -> Result<()> {
        // Try to stop gracefully first
        #[cfg(unix)]
        {
            use std::fs;
            let pid_file = dirs::runtime_dir()
                .unwrap_or_else(|| PathBuf::from("/tmp"))
                .join("cortex.pid");

            if pid_file.exists() {
                let pid_str = fs::read_to_string(&pid_file)?;
                if let Ok(pid) = pid_str.trim().parse::<i32>() {
                    unsafe {
                        libc::kill(pid, libc::SIGTERM);
                    }
                    std::thread::sleep(std::time::Duration::from_secs(2));
                }
                fs::remove_file(pid_file)?;
            }
        }

        #[cfg(windows)]
        {
            // On Windows, use taskkill
            let _ = Command::new("taskkill")
                .args(&["/IM", "cortex.exe", "/F"])
                .output();
        }

        Ok(())
    }

    /// Perform atomic installation
    fn atomic_install(&self, extract_dir: &Path) -> Result<()> {
        let backup_dir = self.install_dir.with_extension("backup");

        // Backup current installation
        if self.install_dir.exists() {
            if backup_dir.exists() {
                fs::remove_dir_all(&backup_dir)?;
            }
            fs::rename(&self.install_dir, &backup_dir)?;
        }

        // Move new installation
        match fs::rename(extract_dir, &self.install_dir) {
            Ok(_) => {
                // Success - remove backup
                if backup_dir.exists() {
                    fs::remove_dir_all(&backup_dir)?;
                }
                Ok(())
            }
            Err(e) => {
                // Failed - restore backup
                if backup_dir.exists() {
                    fs::rename(&backup_dir, &self.install_dir)?;
                }
                Err(e.into())
            }
        }
    }

    /// Update version file
    fn update_version_file(&self, version: &Version) -> Result<()> {
        let version_file = self.install_dir.join("VERSION");
        fs::write(version_file, version.to_string())?;
        Ok(())
    }

    /// Get installation directory
    fn get_install_directory() -> PathBuf {
        if let Ok(exe_path) = std::env::current_exe() {
            exe_path.parent().unwrap().to_path_buf()
        } else {
            #[cfg(unix)]
            {
                PathBuf::from("/usr/local/bin")
            }
            #[cfg(windows)]
            {
                PathBuf::from("C:\\Program Files\\Cortex")
            }
        }
    }

    /// Restart the application after update
    pub fn restart_application(&self) -> Result<()> {
        let executable = if cfg!(target_os = "windows") {
            self.install_dir.join("cortex.exe")
        } else {
            self.install_dir.join("cortex")
        };

        #[cfg(unix)]
        {
            Command::new(&executable).spawn()?;
        }

        #[cfg(windows)]
        {
            Command::new("cmd")
                .args(&["/C", "start", "", &executable.to_string_lossy()])
                .spawn()?;
        }

        // Exit current process
        std::process::exit(0);
    }
}
