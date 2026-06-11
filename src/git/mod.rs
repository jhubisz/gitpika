pub mod commands;
pub mod diff;
pub mod log;
pub mod status;

use std::path::Path;

use anyhow::Result;

use crate::models::{RepoInfo, RepoSnapshot};

const LOG_LIMIT: usize = 200;

/// Load a full snapshot of the repository containing `dir`.
pub fn load_snapshot(dir: &Path) -> Result<RepoSnapshot> {
    let root = commands::repo_root(dir)?;
    let current_branch = commands::current_branch(&root)?;
    let head_short = commands::head_short(&root);

    let files = status::parse_status(&commands::status_porcelain(&root)?);
    let commits = match commands::log_raw(&root, LOG_LIMIT) {
        Ok(raw) => log::parse_log(&raw),
        // An empty repo has no commits; that's not an error worth surfacing.
        Err(_) => Vec::new(),
    };

    Ok(RepoSnapshot {
        repo: RepoInfo {
            root,
            current_branch,
            head_short,
        },
        files,
        commits,
    })
}
