use crate::models::{FileStatus, GitStatusCode};

/// Parse `git status --porcelain=v1` output into a list of file statuses.
pub fn parse_status(output: &str) -> Vec<FileStatus> {
    output.lines().filter_map(parse_line).collect()
}

fn parse_line(line: &str) -> Option<FileStatus> {
    // Format: XY <path> (or "XY <old> -> <new>" for renames/copies).
    let mut chars = line.chars();
    let x = chars.next()?;
    let y = chars.next()?;
    let rest = line.get(3..)?.trim_end();
    if rest.is_empty() {
        return None;
    }

    let index_status = GitStatusCode::from_char(x);
    let worktree_status = GitStatusCode::from_char(y);

    let (old_path, path) = if matches!(index_status, GitStatusCode::Renamed | GitStatusCode::Copied)
        || matches!(
            worktree_status,
            GitStatusCode::Renamed | GitStatusCode::Copied
        ) {
        match rest.split_once(" -> ") {
            Some((old, new)) => (Some(unquote(old)), unquote(new)),
            None => (None, unquote(rest)),
        }
    } else {
        (None, unquote(rest))
    };

    let untracked = x == '?' && y == '?';
    let conflicted = x == 'U' || y == 'U' || (x == 'A' && y == 'A') || (x == 'D' && y == 'D');
    let staged = !untracked && !conflicted && !matches!(x, ' ' | '?' | '!');
    let unstaged = !untracked && !conflicted && !matches!(y, ' ' | '?' | '!');

    Some(FileStatus {
        path,
        old_path,
        index_status,
        worktree_status,
        staged,
        unstaged,
        untracked,
        conflicted,
    })
}

/// Git quotes paths containing special characters; strip the surrounding
/// quotes for display purposes (escape sequences are left as-is).
fn unquote(path: &str) -> String {
    let path = path.trim();
    if path.len() >= 2 && path.starts_with('"') && path.ends_with('"') {
        path[1..path.len() - 1].to_string()
    } else {
        path.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_unstaged_modified() {
        let files = parse_status(" M src/foo.rs\n");
        assert_eq!(files.len(), 1);
        let f = &files[0];
        assert_eq!(f.path, "src/foo.rs");
        assert_eq!(f.index_status, GitStatusCode::None);
        assert_eq!(f.worktree_status, GitStatusCode::Modified);
        assert!(!f.staged);
        assert!(f.unstaged);
        assert!(!f.untracked);
        assert!(!f.conflicted);
    }

    #[test]
    fn parses_staged_added() {
        let files = parse_status("A  src/bar.rs\n");
        let f = &files[0];
        assert_eq!(f.path, "src/bar.rs");
        assert_eq!(f.index_status, GitStatusCode::Added);
        assert!(f.staged);
        assert!(!f.unstaged);
    }

    #[test]
    fn parses_deleted() {
        let files = parse_status(" D old/file.rs\n");
        let f = &files[0];
        assert_eq!(f.path, "old/file.rs");
        assert_eq!(f.worktree_status, GitStatusCode::Deleted);
        assert!(f.unstaged);
    }

    #[test]
    fn parses_untracked() {
        let files = parse_status("?? scratch/test.rs\n");
        let f = &files[0];
        assert_eq!(f.path, "scratch/test.rs");
        assert!(f.untracked);
        assert!(!f.staged);
        assert!(!f.unstaged);
        assert_eq!(f.index_status, GitStatusCode::Untracked);
    }

    #[test]
    fn parses_staged_and_unstaged_modified() {
        let files = parse_status("MM src/both.rs\n");
        let f = &files[0];
        assert!(f.staged);
        assert!(f.unstaged);
        assert_eq!(f.index_status, GitStatusCode::Modified);
        assert_eq!(f.worktree_status, GitStatusCode::Modified);
    }

    #[test]
    fn parses_rename() {
        let files = parse_status("R  old_name.rs -> new_name.rs\n");
        let f = &files[0];
        assert_eq!(f.path, "new_name.rs");
        assert_eq!(f.old_path.as_deref(), Some("old_name.rs"));
        assert_eq!(f.index_status, GitStatusCode::Renamed);
        assert!(f.staged);
    }

    #[test]
    fn parses_conflict() {
        let files = parse_status("UU src/conflict.rs\n");
        let f = &files[0];
        assert!(f.conflicted);
        assert!(!f.staged);
        assert!(!f.unstaged);
    }

    #[test]
    fn parses_quoted_path() {
        let files = parse_status("?? \"with space.rs\"\n");
        assert_eq!(files[0].path, "with space.rs");
    }

    #[test]
    fn skips_empty_output() {
        assert!(parse_status("").is_empty());
        assert!(parse_status("\n").is_empty());
    }

    #[test]
    fn parses_multiple_lines() {
        let out = " M a.rs\nA  b.rs\n?? c.rs\n";
        let files = parse_status(out);
        assert_eq!(files.len(), 3);
        assert_eq!(files[0].path, "a.rs");
        assert_eq!(files[1].path, "b.rs");
        assert_eq!(files[2].path, "c.rs");
    }
}
