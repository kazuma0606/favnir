// Favnir fav.toml parser (minimal)
// Only handles the [rune] section with name / version / src keys.

use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct FavToml {
    pub name: String,
    pub version: String,
    /// Source root directory (relative to fav.toml). Defaults to ".".
    pub src: String,
}

impl FavToml {
    /// Load `fav.toml` from `project_root`. Returns `None` if the file does
    /// not exist or cannot be parsed.
    pub fn load(project_root: &Path) -> Option<Self> {
        let path = project_root.join("fav.toml");
        let content = std::fs::read_to_string(&path).ok()?;
        Some(parse_fav_toml(&content))
    }

    /// Walk up from `start` to find the directory containing `fav.toml`.
    /// Returns `None` if no `fav.toml` is found.
    pub fn find_root(start: &Path) -> Option<PathBuf> {
        let mut dir = start.to_path_buf();
        loop {
            if dir.join("fav.toml").exists() {
                return Some(dir);
            }
            if !dir.pop() {
                return None;
            }
        }
    }

    /// Absolute path to the source root directory.
    pub fn src_dir(&self, root: &Path) -> PathBuf {
        root.join(&self.src)
    }
}

fn parse_fav_toml(content: &str) -> FavToml {
    let mut name = String::new();
    let mut version = String::new();
    let mut src = ".".to_string();
    let mut in_rune = false;

    for line in content.lines() {
        let trimmed = line.trim();
        // skip comments and blank lines
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if trimmed == "[rune]" {
            in_rune = true;
            continue;
        }
        if trimmed.starts_with('[') {
            in_rune = false;
            continue;
        }
        if !in_rune {
            continue;
        }
        // key = "value"
        if let Some((key, val)) = parse_kv(trimmed) {
            match key {
                "name"    => name    = val.to_string(),
                "version" => version = val.to_string(),
                "src"     => src     = val.to_string(),
                _         => {}
            }
        }
    }

    FavToml { name, version, src }
}

/// Parse `key = "value"` or `key = value` (unquoted).
fn parse_kv(line: &str) -> Option<(&str, &str)> {
    let (key, rest) = line.split_once('=')?;
    let key = key.trim();
    let val = rest.trim().trim_matches('"');
    Some((key, val))
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(content: &str) -> FavToml {
        parse_fav_toml(content)
    }

    #[test]
    fn test_all_fields() {
        let t = parse(r#"
[rune]
name    = "myapp"
version = "0.1.0"
src     = "src"
"#);
        assert_eq!(t.name, "myapp");
        assert_eq!(t.version, "0.1.0");
        assert_eq!(t.src, "src");
    }

    #[test]
    fn test_src_default() {
        let t = parse(r#"
[rune]
name    = "myapp"
version = "0.1.0"
"#);
        assert_eq!(t.src, ".");
    }

    #[test]
    fn test_comment_lines_skipped() {
        let t = parse(r#"
# this is a comment
[rune]
# another comment
name = "hello"
version = "0.2.0"
src = "source"
"#);
        assert_eq!(t.name, "hello");
        assert_eq!(t.src, "source");
    }

    #[test]
    fn test_other_section_ignored() {
        let t = parse(r#"
[other]
name = "should-be-ignored"

[rune]
name = "real"
version = "1.0.0"
"#);
        assert_eq!(t.name, "real");
    }
}
