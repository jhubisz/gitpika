use crate::models::CommitNode;

const FIELD_SEP: char = '\u{1f}';

/// Parse the output of
/// `git log --pretty=format:"%H%x1f%P%x1f%D%x1f%s%x1f%an%x1f%ar"`.
pub fn parse_log(output: &str) -> Vec<CommitNode> {
    output.lines().filter_map(parse_record).collect()
}

fn parse_record(line: &str) -> Option<CommitNode> {
    let mut fields = line.split(FIELD_SEP);
    let hash = fields.next()?.trim().to_string();
    if hash.is_empty() {
        return None;
    }
    let parents_raw = fields.next().unwrap_or_default();
    let refs_raw = fields.next().unwrap_or_default();
    let subject = fields.next().unwrap_or_default().to_string();
    let author = fields.next().unwrap_or_default().to_string();
    let relative_date = fields.next().unwrap_or_default().to_string();

    let parents = parents_raw
        .split_whitespace()
        .map(|p| p.to_string())
        .collect();
    let refs = refs_raw
        .split(", ")
        .map(str::trim)
        .filter(|r| !r.is_empty())
        .map(|r| r.to_string())
        .collect();

    let short_hash = hash.chars().take(7).collect();

    Some(CommitNode {
        hash,
        short_hash,
        parents,
        refs,
        subject,
        author,
        relative_date,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(fields: &[&str]) -> String {
        fields.join("\u{1f}")
    }

    #[test]
    fn parses_simple_commit() {
        let line = record(&[
            "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2",
            "0011223344556677889900112233445566778899",
            "",
            "Fix the thing",
            "Alice",
            "2 hours ago",
        ]);
        let commits = parse_log(&line);
        assert_eq!(commits.len(), 1);
        let c = &commits[0];
        assert_eq!(c.short_hash, "a1b2c3d");
        assert_eq!(c.parents.len(), 1);
        assert!(c.refs.is_empty());
        assert_eq!(c.subject, "Fix the thing");
        assert_eq!(c.author, "Alice");
        assert_eq!(c.relative_date, "2 hours ago");
    }

    #[test]
    fn parses_merge_commit_with_two_parents() {
        let line = record(&[
            "ffeeddccbbaa99887766554433221100ffeeddcc",
            "1111111111111111111111111111111111111111 2222222222222222222222222222222222222222",
            "",
            "Merge branch 'feature'",
            "Bob",
            "3 days ago",
        ]);
        let c = &parse_log(&line)[0];
        assert_eq!(c.parents.len(), 2);
        assert_eq!(c.parents[0], "1111111111111111111111111111111111111111");
        assert_eq!(c.parents[1], "2222222222222222222222222222222222222222");
    }

    #[test]
    fn parses_refs_decoration() {
        let line = record(&[
            "abcabcabcabcabcabcabcabcabcabcabcabcabca",
            "",
            "HEAD -> main, origin/main, tag: v1.0",
            "Release v1.0",
            "Carol",
            "5 weeks ago",
        ]);
        let c = &parse_log(&line)[0];
        assert_eq!(
            c.refs,
            vec!["HEAD -> main", "origin/main", "tag: v1.0"]
        );
        assert!(c.parents.is_empty());
    }

    #[test]
    fn parses_multiple_records() {
        let out = format!(
            "{}\n{}",
            record(&["aaaaaaa", "", "", "first", "A", "now"]),
            record(&["bbbbbbb", "aaaaaaa", "", "second", "B", "now"]),
        );
        let commits = parse_log(&out);
        assert_eq!(commits.len(), 2);
        assert_eq!(commits[1].parents, vec!["aaaaaaa"]);
    }

    #[test]
    fn ignores_empty_lines() {
        assert!(parse_log("").is_empty());
        assert!(parse_log("\n\n").is_empty());
    }
}
