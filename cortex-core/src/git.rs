use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct GitInfo {
    pub branch: String,
    pub is_dirty: bool,
    pub ahead: usize,
    pub behind: usize,
}

pub fn get_git_info(path: &Path) -> Option<GitInfo> {
    // Check if directory is a git repository
    if !is_git_repository(path) {
        return None;
    }

    // Get current branch
    let branch = get_current_branch(path)?;

    // Check if working directory is dirty
    let is_dirty = is_working_directory_dirty(path);

    // Get ahead/behind counts
    let (ahead, behind) = get_ahead_behind_count(path);

    Some(GitInfo {
        branch,
        is_dirty,
        ahead,
        behind,
    })
}

fn is_git_repository(path: &Path) -> bool {
    Command::new("git")
        .args(["-C", path.to_str().unwrap_or("."), "rev-parse", "--git-dir"])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn get_current_branch(path: &Path) -> Option<String> {
    let output = Command::new("git")
        .args([
            "-C",
            path.to_str().unwrap_or("."),
            "branch",
            "--show-current",
        ])
        .output()
        .ok()?;

    if output.status.success() {
        let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !branch.is_empty() {
            return Some(branch);
        }
    }

    // Fallback for detached HEAD or other states
    let output = Command::new("git")
        .args([
            "-C",
            path.to_str().unwrap_or("."),
            "rev-parse",
            "--abbrev-ref",
            "HEAD",
        ])
        .output()
        .ok()?;

    if output.status.success() {
        let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if branch != "HEAD" {
            return Some(branch);
        }
    }

    // If still no branch, check if we're in detached HEAD state
    let output = Command::new("git")
        .args([
            "-C",
            path.to_str().unwrap_or("."),
            "rev-parse",
            "--short",
            "HEAD",
        ])
        .output()
        .ok()?;

    if output.status.success() {
        let commit = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !commit.is_empty() {
            return Some(format!("(detached: {})", &commit[..7.min(commit.len())]));
        }
    }

    None
}

fn is_working_directory_dirty(path: &Path) -> bool {
    Command::new("git")
        .args(["-C", path.to_str().unwrap_or("."), "status", "--porcelain"])
        .output()
        .map(|output| output.status.success() && !output.stdout.is_empty())
        .unwrap_or(false)
}

fn get_ahead_behind_count(path: &Path) -> (usize, usize) {
    let output = Command::new("git")
        .args([
            "-C",
            path.to_str().unwrap_or("."),
            "rev-list",
            "--left-right",
            "--count",
            "HEAD...@{upstream}",
        ])
        .output();

    if let Ok(output) = output {
        if output.status.success() {
            let result = String::from_utf8_lossy(&output.stdout);
            let parts: Vec<&str> = result.trim().split('\t').collect();
            if parts.len() == 2 {
                let ahead = parts[0].parse().unwrap_or(0);
                let behind = parts[1].parse().unwrap_or(0);
                return (ahead, behind);
            }
        }
    }

    (0, 0)
}
