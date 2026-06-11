use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{bail, Context, Result};

/// Run a git command in `dir` and return trimmed stdout, or an error
/// containing git's stderr.
pub fn run_git(dir: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .with_context(|| format!("failed to spawn `git {}`", args.join(" ")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("`git {}` failed: {}", args.join(" "), stderr.trim());
    }

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// Detect the repository root containing `dir`, if any.
pub fn repo_root(dir: &Path) -> Result<PathBuf> {
    let out = run_git(dir, &["rev-parse", "--show-toplevel"])?;
    Ok(PathBuf::from(out.trim()))
}

/// Current branch name, or `None` when detached.
pub fn current_branch(root: &Path) -> Result<Option<String>> {
    let out = run_git(root, &["branch", "--show-current"])?;
    let name = out.trim();
    Ok(if name.is_empty() {
        None
    } else {
        Some(name.to_string())
    })
}

/// Short HEAD hash, or `None` when the repo has no commits yet.
pub fn head_short(root: &Path) -> Option<String> {
    run_git(root, &["rev-parse", "--short", "HEAD"])
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

pub fn status_porcelain(root: &Path) -> Result<String> {
    run_git(root, &["status", "--porcelain=v1", "--untracked-files=all"])
}

pub fn log_raw(root: &Path, limit: usize) -> Result<String> {
    let n = limit.to_string();
    run_git(
        root,
        &[
            "log",
            "--all",
            "--date-order",
            "--topo-order",
            "--pretty=format:%H%x1f%P%x1f%D%x1f%s%x1f%an%x1f%ar",
            "-n",
            &n,
        ],
    )
}

pub fn diff_unstaged(root: &Path, path: &str) -> Result<String> {
    run_git(root, &["diff", "--", path])
}

pub fn diff_staged(root: &Path, path: &str) -> Result<String> {
    run_git(root, &["diff", "--cached", "--", path])
}
