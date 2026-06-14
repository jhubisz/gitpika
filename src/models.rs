use std::path::PathBuf;

/// How the diff panel presents changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffMode {
    /// Only changed hunks with a few lines of context (git default).
    Hunks,
    /// The whole file with inline +/- lines (unlimited context).
    FullFile,
}

impl DiffMode {
    pub fn toggled(self) -> Self {
        match self {
            DiffMode::Hunks => DiffMode::FullFile,
            DiffMode::FullFile => DiffMode::Hunks,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            DiffMode::Hunks => "hunks",
            DiffMode::FullFile => "full file",
        }
    }
}

#[derive(Debug, Clone)]
pub struct RepoInfo {
    pub root: PathBuf,
    pub current_branch: Option<String>,
    pub head_short: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RepoSnapshot {
    pub repo: RepoInfo,
    pub files: Vec<FileStatus>,
    pub commits: Vec<CommitNode>,
}

#[derive(Debug, Clone)]
pub struct FileStatus {
    pub path: String,
    pub old_path: Option<String>,
    pub index_status: GitStatusCode,
    pub worktree_status: GitStatusCode,
    pub staged: bool,
    pub unstaged: bool,
    pub untracked: bool,
    pub conflicted: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GitStatusCode {
    Modified,
    Added,
    Deleted,
    Renamed,
    Copied,
    UpdatedButUnmerged,
    Untracked,
    Ignored,
    None,
    Unknown(char),
}

impl GitStatusCode {
    pub fn from_char(c: char) -> Self {
        match c {
            'M' | 'T' => GitStatusCode::Modified,
            'A' => GitStatusCode::Added,
            'D' => GitStatusCode::Deleted,
            'R' => GitStatusCode::Renamed,
            'C' => GitStatusCode::Copied,
            'U' => GitStatusCode::UpdatedButUnmerged,
            '?' => GitStatusCode::Untracked,
            '!' => GitStatusCode::Ignored,
            ' ' => GitStatusCode::None,
            other => GitStatusCode::Unknown(other),
        }
    }

    pub fn as_char(&self) -> char {
        match self {
            GitStatusCode::Modified => 'M',
            GitStatusCode::Added => 'A',
            GitStatusCode::Deleted => 'D',
            GitStatusCode::Renamed => 'R',
            GitStatusCode::Copied => 'C',
            GitStatusCode::UpdatedButUnmerged => 'U',
            GitStatusCode::Untracked => '?',
            GitStatusCode::Ignored => '!',
            GitStatusCode::None => ' ',
            GitStatusCode::Unknown(c) => *c,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CommitNode {
    // `hash` and `parents` are not rendered yet, but are kept for the
    // future lane-routed graph view.
    #[allow(dead_code)]
    pub hash: String,
    pub short_hash: String,
    #[allow(dead_code)]
    pub parents: Vec<String>,
    pub refs: Vec<String>,
    pub subject: String,
    pub author: String,
    pub relative_date: String,
}
