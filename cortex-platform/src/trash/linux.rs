use crate::{TrashItem, TrashOperations};
use anyhow::{Context, Result};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

pub struct LinuxTrash {
    trash_dir: PathBuf,
}

impl LinuxTrash {
    pub fn new() -> Self {
        let trash_dir = Self::get_trash_directory();

        // Ensure trash directories exist
        let _ = fs::create_dir_all(trash_dir.join("files"));
        let _ = fs::create_dir_all(trash_dir.join("info"));

        Self { trash_dir }
    }

    fn get_trash_directory() -> PathBuf {
        // Follow XDG trash specification
        if let Ok(xdg_data_home) = env::var("XDG_DATA_HOME") {
            PathBuf::from(xdg_data_home).join("Trash")
        } else if let Ok(home) = env::var("HOME") {
            PathBuf::from(home).join(".local/share/Trash")
        } else {
            PathBuf::from("/tmp/.trash")
        }
    }

    fn generate_trash_name(&self, original_name: &str) -> String {
        let timestamp = chrono::Local::now().timestamp();
        format!("{}.{}", original_name, timestamp)
    }

    fn write_trash_info(&self, original_path: &Path, trash_name: &str) -> Result<()> {
        let info_path = self
            .trash_dir
            .join("info")
            .join(format!("{}.trashinfo", trash_name));
        let deletion_date = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();

        let info_content = format!(
            "[Trash Info]\nPath={}\nDeletionDate={}\n",
            original_path.display(),
            deletion_date
        );

        fs::write(&info_path, info_content).context("Failed to write trash info file")?;

        Ok(())
    }
}

impl TrashOperations for LinuxTrash {
    fn move_to_trash(&self, path: &Path) -> Result<()> {
        if !path.exists() {
            return Err(anyhow::anyhow!("Path does not exist: {}", path.display()));
        }

        let file_name = path
            .file_name()
            .ok_or_else(|| anyhow::anyhow!("Invalid file name"))?
            .to_string_lossy();

        let trash_name = self.generate_trash_name(&file_name);
        let trash_path = self.trash_dir.join("files").join(&trash_name);

        // Write metadata before moving
        self.write_trash_info(path, &trash_name)?;

        // Move file to trash
        fs::rename(path, &trash_path).context("Failed to move file to trash")?;

        Ok(())
    }

    fn restore_from_trash(&self, trash_path: &Path) -> Result<()> {
        let file_name = trash_path
            .file_name()
            .ok_or_else(|| anyhow::anyhow!("Invalid trash file name"))?
            .to_string_lossy();

        let info_path = self
            .trash_dir
            .join("info")
            .join(format!("{}.trashinfo", file_name));

        if !info_path.exists() {
            return Err(anyhow::anyhow!("Trash info file not found"));
        }

        // Read original path from info file
        let info_content = fs::read_to_string(&info_path)?;
        let original_path = info_content
            .lines()
            .find(|line| line.starts_with("Path="))
            .and_then(|line| line.strip_prefix("Path="))
            .ok_or_else(|| anyhow::anyhow!("Invalid trash info file"))?;

        let original_path = PathBuf::from(original_path);
        let trash_file_path = self.trash_dir.join("files").join(file_name.as_ref());

        // Restore file
        fs::rename(&trash_file_path, &original_path)
            .context("Failed to restore file from trash")?;

        // Remove info file
        fs::remove_file(&info_path)?;

        Ok(())
    }

    fn empty_trash(&self) -> Result<()> {
        let files_dir = self.trash_dir.join("files");
        let info_dir = self.trash_dir.join("info");

        // Remove all files
        if files_dir.exists() {
            for entry in fs::read_dir(&files_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    fs::remove_dir_all(&path)?;
                } else {
                    fs::remove_file(&path)?;
                }
            }
        }

        // Remove all info files
        if info_dir.exists() {
            for entry in fs::read_dir(&info_dir)? {
                let entry = entry?;
                fs::remove_file(entry.path())?;
            }
        }

        Ok(())
    }

    fn list_trash_contents(&self) -> Result<Vec<TrashItem>> {
        let mut items = Vec::new();
        let info_dir = self.trash_dir.join("info");

        if !info_dir.exists() {
            return Ok(items);
        }

        for entry in fs::read_dir(&info_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("trashinfo") {
                let info_content = fs::read_to_string(&path)?;

                let original_path = info_content
                    .lines()
                    .find(|line| line.starts_with("Path="))
                    .and_then(|line| line.strip_prefix("Path="))
                    .unwrap_or("")
                    .to_string();

                let deletion_date_str = info_content
                    .lines()
                    .find(|line| line.starts_with("DeletionDate="))
                    .and_then(|line| line.strip_prefix("DeletionDate="))
                    .unwrap_or("");

                let deletion_date =
                    chrono::DateTime::parse_from_rfc3339(&format!("{}+00:00", deletion_date_str))
                        .ok()
                        .map(|dt| dt.with_timezone(&chrono::Local))
                        .unwrap_or_else(chrono::Local::now);

                let trash_name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_string();

                let trash_file_path = self.trash_dir.join("files").join(&trash_name);
                let size = fs::metadata(&trash_file_path).map(|m| m.len()).unwrap_or(0);

                items.push(TrashItem {
                    original_path,
                    trash_path: trash_file_path.to_string_lossy().to_string(),
                    deletion_date,
                    size,
                });
            }
        }

        Ok(items)
    }
}
