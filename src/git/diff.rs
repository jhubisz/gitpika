use std::fs;
use std::path::Path;

use anyhow::Result;

use crate::git::commands;
use crate::models::{DiffMode, FileStatus};

/// Produce the diff text to display for a file from the changed files list.
pub fn diff_for_file(root: &Path, file: &FileStatus, mode: DiffMode) -> Result<String> {
    let full = mode == DiffMode::FullFile;

    if file.untracked {
        // The synthetic untracked diff is already the full file.
        return Ok(untracked_diff(root, &file.path));
    }

    if file.staged && file.unstaged {
        let staged = commands::diff_staged(root, &file.path, full)?;
        let unstaged = commands::diff_unstaged(root, &file.path, full)?;
        let mut out = String::new();
        if !staged.trim().is_empty() {
            out.push_str("== Staged ==\n");
            out.push_str(&staged);
        }
        if !unstaged.trim().is_empty() {
            if !out.is_empty() {
                out.push('\n');
            }
            out.push_str("== Unstaged ==\n");
            out.push_str(&unstaged);
        }
        return Ok(out);
    }

    let diff = if file.staged {
        commands::diff_staged(root, &file.path, full)?
    } else {
        commands::diff_unstaged(root, &file.path, full)?
    };

    if diff.trim().is_empty() {
        Ok("(no diff to display)".to_string())
    } else {
        Ok(diff)
    }
}

/// Synthesize an "all lines added" diff for an untracked file.
fn untracked_diff(root: &Path, path: &str) -> String {
    let full = root.join(path);
    match fs::read(&full) {
        Ok(bytes) => {
            if bytes.contains(&0) {
                format!("(untracked binary file: {} bytes)", bytes.len())
            } else {
                let content = String::from_utf8_lossy(&bytes);
                let mut out = format!("new file: {path}\n");
                for line in content.lines() {
                    out.push('+');
                    out.push_str(line);
                    out.push('\n');
                }
                out
            }
        }
        Err(err) => format!("(could not read {path}: {err})"),
    }
}
