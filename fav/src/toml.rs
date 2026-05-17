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
    /// `name = "^1.0.0"` — semver constraint resolved against the local registry.
    Semver { name: String, version: String },
}

impl DependencySpec {
    pub fn name(&self) -> &str {
        match self {
            DependencySpec::Path { name, .. } => name,
            DependencySpec::Registry { name, .. } => name,
            DependencySpec::Semver { name, .. } => name,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CheckpointConfig {
    pub backend: String,
    pub path: String,
}

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub migrations: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub mode: String, // "jwt" | "cognito" | "none"; default "jwt"
}

#[derive(Debug, Clone)]
pub struct LogConfig {
    pub level: String,    // "debug" | "info" | "warn" | "error"; default "info"
    pub format: String,   // "json" | "text"; default "text"
    pub output: String,   // "stdout" | "stderr"; default "stdout"
    pub service: String,  // service name for JSON output
}

impl Default for LogConfig {
    fn default() -> Self {
        LogConfig {
            level: "info".to_string(),
            format: "text".to_string(),
            output: "stdout".to_string(),
            service: String::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct EnvConfig {
    pub dotenv: Option<String>,
    pub prefix: String,
}

impl Default for EnvConfig {
    fn default() -> Self {
        EnvConfig { dotenv: None, prefix: String::new() }
    }
}

// ── AWS config (v4.11.0) ──────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct AwsTomlConfig {
    pub region: Option<String>,
    pub endpoint_url: Option<String>,
    pub profile: Option<String>,
}

// ── Deploy config (v4.11.0) ───────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct DeployConfig {
    pub runtime: String,
    pub handler: String,
    pub memory: u32,
    pub timeout: u32,
    pub s3_bucket: Option<String>,
    pub role_arn: Option<String>,
    pub region: Option<String>,
}

impl Default for DeployConfig {
    fn default() -> Self {
        DeployConfig {
            runtime: "provided.al2".to_string(),
            handler: "bootstrap".to_string(),
            memory: 256,
            timeout: 30,
            s3_bucket: None,
            role_arn: None,
            region: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FavToml {
    pub name: String,
    pub version: String,
    /// Package description (for fav publish).
    pub description: Option<String>,
    /// Package authors (for fav publish).
    pub authors: Vec<String>,
    /// Package license (for fav publish).
    pub license: Option<String>,
    /// Source root directory (relative to fav.toml). Defaults to ".".
    pub src: String,
    /// Optional rune library root directory (relative to fav.toml). Defaults to `runes`.
    pub runes_path: Option<String>,
    /// Dependencies declared in `[dependencies]`.
    pub dependencies: Vec<DependencySpec>,
    /// Optional checkpoint backend configuration.
    pub checkpoint: Option<CheckpointConfig>,
    /// Optional database configuration.
    pub database: Option<DatabaseConfig>,
    /// Optional auth configuration.
    pub auth: Option<AuthConfig>,
    /// Optional log configuration.
    pub log: Option<LogConfig>,
    /// Optional env configuration.
    pub env: Option<EnvConfig>,
    /// Optional AWS configuration (v4.11.0).
    pub aws: Option<AwsTomlConfig>,
    /// Optional deploy configuration (v4.11.0).
    pub deploy: Option<DeployConfig>,
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
    let mut description: Option<String> = None;
    let mut authors: Vec<String> = Vec::new();
    let mut license: Option<String> = None;
    let mut src = ".".to_string();
    let mut runes_path = None;
    let mut dependencies = Vec::new();
    let mut checkpoint = None;
    let mut database = None;
    let mut auth = None;
    let mut log = None;
    let mut env_cfg: Option<EnvConfig> = None;
    let mut aws_cfg: Option<AwsTomlConfig> = None;
    let mut deploy_cfg: Option<DeployConfig> = None;
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
        if trimmed == "[checkpoint]" {
            section = "checkpoint";
            continue;
        }
        if trimmed == "[database]" {
            section = "database";
            continue;
        }
        if trimmed == "[runes]" {
            section = "runes";
            continue;
        }
        if trimmed == "[auth]" {
            section = "auth";
            continue;
        }
        if trimmed == "[log]" {
            section = "log";
            continue;
        }
        if trimmed == "[env]" {
            section = "env";
            continue;
        }
        if trimmed == "[aws]" {
            section = "aws";
            continue;
        }
        if trimmed == "[deploy]" || trimmed == "[[deploy.env]]" {
            section = "deploy";
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
                        "name"        => name        = val.to_string(),
                        "version"     => version     = val.to_string(),
                        "description" => description = Some(val.to_string()),
                        "license"     => license     = Some(val.to_string()),
                        "authors"     => authors     = val.split(',').map(|s| s.trim().to_string()).collect(),
                        "src"         => src         = val.to_string(),
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
            "checkpoint" => {
                let mut current = checkpoint.take().unwrap_or(CheckpointConfig {
                    backend: "file".into(),
                    path: ".fav_checkpoints".into(),
                });
                if let Some((key, val)) = parse_kv(trimmed) {
                    match key {
                        "backend" => current.backend = val.to_string(),
                        "path" => current.path = val.to_string(),
                        _ => {}
                    }
                }
                checkpoint = Some(current);
            }
            "database" => {
                let mut current = database.take().unwrap_or(DatabaseConfig {
                    url: String::new(),
                    migrations: None,
                });
                if let Some((key, val)) = parse_kv(trimmed) {
                    match key {
                        "url" => current.url = val.to_string(),
                        "migrations" => current.migrations = Some(val.to_string()),
                        _ => {}
                    }
                }
                database = Some(current);
            }
            "auth" => {
                let mut current = auth.take().unwrap_or(AuthConfig {
                    mode: "jwt".into(),
                });
                if let Some((key, val)) = parse_kv(trimmed) {
                    if key == "mode" {
                        current.mode = val.to_string();
                    }
                }
                auth = Some(current);
            }
            "log" => {
                let mut current: LogConfig = log.take().unwrap_or_default();
                if let Some((key, val)) = parse_kv(trimmed) {
                    match key {
                        "level"   => current.level   = val.to_string(),
                        "format"  => current.format  = val.to_string(),
                        "output"  => current.output  = val.to_string(),
                        "service" => current.service = val.to_string(),
                        _ => {}
                    }
                }
                log = Some(current);
            }
            "env" => {
                let mut current: EnvConfig = env_cfg.take().unwrap_or_default();
                if let Some((key, val)) = parse_kv(trimmed) {
                    match key {
                        "dotenv" => current.dotenv = Some(val.to_string()),
                        "prefix" => current.prefix = val.to_string(),
                        _ => {}
                    }
                }
                env_cfg = Some(current);
            }
            "aws" => {
                let mut current = aws_cfg.take().unwrap_or(AwsTomlConfig {
                    region: None,
                    endpoint_url: None,
                    profile: None,
                });
                if let Some((key, val)) = parse_kv(trimmed) {
                    match key {
                        "region"       => current.region       = Some(val.to_string()),
                        "endpoint_url" => current.endpoint_url = Some(val.to_string()),
                        "profile"      => current.profile      = Some(val.to_string()),
                        _ => {}
                    }
                }
                aws_cfg = Some(current);
            }
            "deploy" => {
                let mut current: DeployConfig = deploy_cfg.take().unwrap_or_default();
                if let Some((key, val)) = parse_kv(trimmed) {
                    match key {
                        "runtime"   => current.runtime   = val.to_string(),
                        "handler"   => current.handler   = val.to_string(),
                        "memory"    => current.memory    = val.parse().unwrap_or(256),
                        "timeout"   => current.timeout   = val.parse().unwrap_or(30),
                        "s3_bucket" => current.s3_bucket = Some(val.to_string()),
                        "role_arn"  => current.role_arn  = Some(val.to_string()),
                        "region"    => current.region    = Some(val.to_string()),
                        _ => {}
                    }
                }
                deploy_cfg = Some(current);
            }
            _ => {}
        }
    }

    FavToml {
        name,
        version,
        description,
        authors,
        license,
        src,
        runes_path,
        dependencies,
        checkpoint,
        database,
        auth,
        log,
        env: env_cfg,
        aws: aws_cfg,
        deploy: deploy_cfg,
    }
}

/// Parse a dependency line: `name = "^1.0.0"` or `name = { key = "val", ... }`
fn parse_dep_line(line: &str) -> Option<DependencySpec> {
    let (dep_name, rest) = line.split_once('=')?;
    let dep_name = dep_name.trim().to_string();
    let rhs = rest.trim();

    // Plain string form: `name = "1.0.0"` or `name = "^1.0.0"`
    if rhs.starts_with('"') && rhs.ends_with('"') && !rhs.contains('{') {
        let version = rhs.trim_matches('"').to_string();
        return Some(DependencySpec::Semver { name: dep_name, version });
    }

    let inner = rhs.trim_start_matches('{').trim_end_matches('}');
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
