#![allow(dead_code)]

// Favnir fav.toml parser (minimal)
// Handles [rune] and [dependencies] sections.

use std::path::{Path, PathBuf};

/// A single dependency entry in `[dependencies]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DependencySpec {
    /// `name = { path = "..." }` — local path dependency.
    Path { name: String, path: String },
    /// `name = { registry = "local", version = "..." }` — local registry dependency.
    Registry {
        name: String,
        registry: String,
        version: String,
    },
}

impl DependencySpec {
    pub fn name(&self) -> &str {
        match self {
            DependencySpec::Path { name, .. } => name,
            DependencySpec::Registry { name, .. } => name,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FavToml {
    pub name: String,
    pub version: String,
    /// Source root directory (relative to fav.toml). Defaults to ".".
    pub src: String,
    /// Optional rune library root directory (relative to fav.toml). Defaults to `runes`.
    pub runes_path: Option<String>,
    /// Dependencies declared in `[dependencies]`.
    pub dependencies: Vec<DependencySpec>,
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

    pub fn runes_dir(&self, root: &Path) -> PathBuf {
        root.join(self.runes_path.as_deref().unwrap_or("runes"))
    }
}

fn parse_fav_toml(content: &str) -> FavToml {
    let mut name = String::new();
    let mut version = String::new();
    let mut src = ".".to_string();
    let mut runes_path = None;
    let mut dependencies = Vec::new();
    let mut section = "";

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if trimmed == "[rune]" {
            section = "rune";
            continue;
        }
        if trimmed == "[dependencies]" {
            section = "dependencies";
            continue;
        }
        if trimmed == "[runes]" {
            section = "runes";
            continue;
        }
        if trimmed.starts_with('[') {
            section = "";
            continue;
        }
        match section {
            "rune" => {
                if let Some((key, val)) = parse_kv(trimmed) {
                    match key {
                        "name" => name = val.to_string(),
                        "version" => version = val.to_string(),
                        "src" => src = val.to_string(),
                        _ => {}
                    }
                }
            }
            "runes" => {
                if let Some((key, val)) = parse_kv(trimmed) {
                    if key == "path" {
                        runes_path = Some(val.to_string());
                    }
                }
            }
            "dependencies" => {
                // name = { path = "..." }  or  name = { registry = "local", version = "..." }
                if let Some(dep) = parse_dep_line(trimmed) {
                    dependencies.push(dep);
                }
            }
            _ => {}
        }
    }

    FavToml {
        name,
        version,
        src,
        runes_path,
        dependencies,
    }
}

/// Parse a dependency line: `name = { key = "val", ... }`
fn parse_dep_line(line: &str) -> Option<DependencySpec> {
    let (dep_name, rest) = line.split_once('=')?;
    let dep_name = dep_name.trim().to_string();
    let inner = rest.trim().trim_start_matches('{').trim_end_matches('}');
    let mut path_val: Option<String> = None;
    let mut registry_val: Option<String> = None;
    let mut version_val: Option<String> = None;

    for part in inner.split(',') {
        let part = part.trim();
        if let Some((k, v)) = part.split_once('=') {
            let k = k.trim();
            let v = v.trim().trim_matches('"').to_string();
            match k {
                "path" => path_val = Some(v),
                "registry" => registry_val = Some(v),
                "version" => version_val = Some(v),
                _ => {}
            }
        }
    }

    if let Some(path) = path_val {
        Some(DependencySpec::Path {
            name: dep_name,
            path,
        })
    } else if let (Some(registry), Some(version)) = (registry_val, version_val) {
        Some(DependencySpec::Registry {
            name: dep_name,
            registry,
            version,
        })
    } else {
        None
    }
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
        let t = parse(
            r#"
[rune]
name    = "myapp"
version = "0.1.0"
src     = "src"
"#,
        );
        assert_eq!(t.name, "myapp");
        assert_eq!(t.version, "0.1.0");
        assert_eq!(t.src, "src");
    }

    #[test]
    fn test_src_default() {
        let t = parse(
            r#"
[rune]
name    = "myapp"
version = "0.1.0"
"#,
        );
        assert_eq!(t.src, ".");
    }

    #[test]
    fn test_comment_lines_skipped() {
        let t = parse(
            r#"
# this is a comment
[rune]
# another comment
name = "hello"
version = "0.2.0"
src = "source"
"#,
        );
        assert_eq!(t.name, "hello");
        assert_eq!(t.src, "source");
    }

    #[test]
    fn test_runes_path_parsed() {
        let t = parse(
            r#"
[rune]
name = "hello"
version = "0.2.0"
[runes]
path = "libs/runes"
"#,
        );
        assert_eq!(t.runes_path.as_deref(), Some("libs/runes"));
    }

    #[test]
    fn test_other_section_ignored() {
        let t = parse(
            r#"
[other]
name = "should-be-ignored"

[rune]
name = "real"
version = "1.0.0"
"#,
        );
        assert_eq!(t.name, "real");
    }

    #[test]
    fn test_path_dependency_parsed() {
        let t = parse(
            r#"
[rune]
name = "myapp"
version = "1.0.0"

[dependencies]
mylib = { path = "../mylib" }
"#,
        );
        assert_eq!(t.dependencies.len(), 1);
        assert_eq!(
            t.dependencies[0],
            DependencySpec::Path {
                name: "mylib".into(),
                path: "../mylib".into()
            }
        );
    }

    #[test]
    fn test_registry_dependency_parsed() {
        let t = parse(
            r#"
[rune]
name = "myapp"
version = "1.0.0"

[dependencies]
utils = { registry = "local", version = "0.1.0" }
"#,
        );
        assert_eq!(t.dependencies.len(), 1);
        assert_eq!(
            t.dependencies[0],
            DependencySpec::Registry {
                name: "utils".into(),
                registry: "local".into(),
                version: "0.1.0".into()
            }
        );
    }

    #[test]
    fn test_multiple_dependencies_parsed() {
        let t = parse(
            r#"
[rune]
name = "myapp"
version = "1.0.0"

[dependencies]
libA = { path = "../libA" }
libB = { registry = "local", version = "2.0.0" }
"#,
        );
        assert_eq!(t.dependencies.len(), 2);
        assert_eq!(t.dependencies[0].name(), "libA");
        assert_eq!(t.dependencies[1].name(), "libB");
    }
}
