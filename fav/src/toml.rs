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
    pub level: String,   // "debug" | "info" | "warn" | "error"; default "info"
    pub format: String,  // "json" | "text"; default "text"
    pub output: String,  // "stdout" | "stderr"; default "stdout"
    pub service: String, // service name for JSON output
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
        EnvConfig {
            dotenv: None,
            prefix: String::new(),
        }
    }
}

// ── AWS config (v4.11.0) ──────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct AwsTomlConfig {
    pub region: Option<String>,
    pub endpoint_url: Option<String>,
    pub profile: Option<String>,
}

// ── Snowflake config (v10.7.0) ────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct SnowflakeTomlConfig {
    pub account:   Option<String>,
    pub user:      Option<String>,
    pub warehouse: Option<String>,
    pub role:      Option<String>,
    pub database:  Option<String>,
    pub schema:    Option<String>,
}

// ── GCP config (v15.2.0) ─────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct GcpTomlConfig {
    pub project_id:       Option<String>,
    pub credentials_file: Option<String>,
    pub dataset:          Option<String>,
}

// ── Kafka config (v15.4.0) ────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct KafkaTomlConfig {
    pub bootstrap_brokers: Option<String>,
    pub sasl_mechanism:    Option<String>,
    pub sasl_username:     Option<String>,
    pub sasl_password:     Option<String>,
}

// ── Azure config (v14.2.0) ────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct AzureTomlConfig {
    pub postgres_url:    Option<String>,
    pub storage_account: Option<String>,
    pub storage_key:     Option<String>,
    pub container:       Option<String>,
}

// ── Postgres config (v11.5.0) ─────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct PostgresTomlConfig {
    pub host:     Option<String>,
    pub port:     Option<u16>,
    pub dbname:   Option<String>,
    pub user:     Option<String>,
    pub password: Option<String>,
    pub sslmode:  Option<String>,
}

// ── Deploy config (v4.11.0) ───────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct DeployConfig {
    /// Deploy target platform (v15.5.0): "aws-lambda" | "azure-function"
    pub target: String,
    /// Lambda function name override (v15.5.0); defaults to fav.toml [project] name
    pub function_name: Option<String>,
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
            target: "aws-lambda".to_string(),
            function_name: None,
            runtime: "provided.al2023".to_string(),
            handler: "bootstrap".to_string(),
            memory: 256,
            timeout: 30,
            s3_bucket: None,
            role_arn: None,
            region: None,
        }
    }
}

/// `[lint]` section of fav.toml (v12.10.0).
#[derive(Debug, Clone)]
pub struct LintTomlConfig {
    /// Lint codes to treat as errors (exit 1 even with `--warn-only`).
    pub warn_as_error: Option<Vec<String>>,
    /// Lint codes to suppress entirely.
    pub allow: Option<Vec<String>>,
}

/// `[context]` section of fav.toml (v13.5.0).
#[derive(Debug, Clone)]
pub struct ContextConfig {
    /// Database connection URL (e.g. postgres://...). Supports ${ENV_VAR} expansion.
    pub db_url: Option<String>,
    /// Storage backend: "s3" | "local". Defaults to "s3".
    pub storage: Option<String>,
    /// HTTP client backend: "ureq". Defaults to "ureq".
    pub http: Option<String>,
}

/// `[run]` section of fav.toml (v12.5.0).
#[derive(Debug, Clone)]
pub struct RunTomlConfig {
    /// Enable 200-char truncated verbose trace (equivalent to `fav run --verbose`).
    pub verbose: bool,
    /// Enable unlimited verbose trace (equivalent to `fav run --trace`).
    pub trace: bool,
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
    /// Optional Snowflake configuration (v10.7.0).
    pub snowflake: Option<SnowflakeTomlConfig>,
    /// Optional Postgres configuration (v11.5.0).
    pub postgres: Option<PostgresTomlConfig>,
    /// Optional run configuration (v12.5.0).
    pub run: Option<RunTomlConfig>,
    /// Optional lint configuration (v12.10.0).
    pub lint: Option<LintTomlConfig>,
    /// Optional context configuration (v13.5.0).
    pub context: Option<ContextConfig>,
    /// Optional GCP configuration (v15.2.0).
    pub gcp: Option<GcpTomlConfig>,
    /// Optional Kafka / MSK configuration (v15.4.0).
    pub kafka: Option<KafkaTomlConfig>,
    /// Optional Azure configuration (v14.2.0).
    pub azure: Option<AzureTomlConfig>,
    /// Dev-only dependencies declared in `[dev-dependencies]` (v17.8.0).
    pub dev_dependencies: Vec<DependencySpec>,
    /// Registry URL from `[registry]` section (v17.8.0).
    pub registry_url: Option<String>,
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

/// Public test helper — wraps `parse_fav_toml` for cross-module tests.
pub fn parse_fav_toml_pub(content: &str) -> FavToml {
    parse_fav_toml(content)
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
    let mut snowflake_cfg: Option<SnowflakeTomlConfig> = None;
    let mut postgres_cfg: Option<PostgresTomlConfig> = None;
    let mut run_cfg: Option<RunTomlConfig> = None;
    let mut lint_cfg: Option<LintTomlConfig> = None;
    let mut context_cfg: Option<ContextConfig> = None;
    let mut azure_cfg: Option<AzureTomlConfig> = None;
    let mut gcp_cfg: Option<GcpTomlConfig> = None;
    let mut kafka_cfg: Option<KafkaTomlConfig> = None;
    let mut dev_dependencies: Vec<DependencySpec> = Vec::new();
    let mut registry_url: Option<String> = None;
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
        if trimmed == "[snowflake]" {
            section = "snowflake";
            continue;
        }
        if trimmed == "[postgres]" {
            section = "postgres";
            continue;
        }
        if trimmed == "[run]" {
            section = "run";
            continue;
        }
        if trimmed == "[context]" {
            section = "context";
            continue;
        }
        if trimmed == "[azure]" {
            section = "azure";
            continue;
        }
        if trimmed == "[gcp]" {
            section = "gcp";
            continue;
        }
        if trimmed == "[dev-dependencies]" {
            section = "dev-dependencies";
            continue;
        }
        if trimmed == "[registry]" {
            section = "registry";
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
                        "description" => description = Some(val.to_string()),
                        "license" => license = Some(val.to_string()),
                        "authors" => {
                            authors = val.split(',').map(|s| s.trim().to_string()).collect()
                        }
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
                let mut current = auth.take().unwrap_or(AuthConfig { mode: "jwt".into() });
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
                        "level" => current.level = val.to_string(),
                        "format" => current.format = val.to_string(),
                        "output" => current.output = val.to_string(),
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
                        "region" => current.region = Some(val.to_string()),
                        "endpoint_url" => current.endpoint_url = Some(val.to_string()),
                        "profile" => current.profile = Some(val.to_string()),
                        _ => {}
                    }
                }
                aws_cfg = Some(current);
            }
            "deploy" => {
                let mut current: DeployConfig = deploy_cfg.take().unwrap_or_default();
                if let Some((key, val)) = parse_kv(trimmed) {
                    match key {
                        "target"        => current.target        = val.to_string(),
                        "function_name" => current.function_name = Some(val.to_string()),
                        "runtime"       => current.runtime       = val.to_string(),
                        "handler"       => current.handler       = val.to_string(),
                        "memory" | "memory_mb" => current.memory = val.parse().unwrap_or(256),
                        "timeout" | "timeout_sec" => current.timeout = val.parse().unwrap_or(30),
                        "s3_bucket"     => current.s3_bucket     = Some(val.to_string()),
                        "role_arn"      => current.role_arn      = Some(val.to_string()),
                        "region"        => current.region        = Some(val.to_string()),
                        _ => {}
                    }
                }
                deploy_cfg = Some(current);
            }
            "snowflake" => {
                let mut current = snowflake_cfg.take().unwrap_or(SnowflakeTomlConfig {
                    account:   None,
                    user:      None,
                    warehouse: None,
                    role:      None,
                    database:  None,
                    schema:    None,
                });
                if let Some((key, val)) = parse_kv(trimmed) {
                    match key {
                        "account"   => current.account   = Some(val.to_string()),
                        "user"      => current.user      = Some(val.to_string()),
                        "warehouse" => current.warehouse = Some(val.to_string()),
                        "role"      => current.role      = Some(val.to_string()),
                        "database"  => current.database  = Some(val.to_string()),
                        "schema"    => current.schema    = Some(val.to_string()),
                        _ => {}
                    }
                }
                snowflake_cfg = Some(current);
            }
            "postgres" => {
                let mut current = postgres_cfg.take().unwrap_or(PostgresTomlConfig {
                    host:     None,
                    port:     None,
                    dbname:   None,
                    user:     None,
                    password: None,
                    sslmode:  None,
                });
                if let Some((key, val)) = parse_kv(trimmed) {
                    match key {
                        "host"     => current.host     = Some(expand_env_vars(val)),
                        "port"     => current.port     = val.parse::<u16>().ok(),
                        "dbname"   => current.dbname   = Some(expand_env_vars(val)),
                        "user"     => current.user     = Some(expand_env_vars(val)),
                        "password" => current.password = Some(expand_env_vars(val)),
                        "sslmode"  => current.sslmode  = Some(val.to_string()),
                        _ => {}
                    }
                }
                postgres_cfg = Some(current);
            }
            "run" => {
                let mut current = run_cfg.take().unwrap_or(RunTomlConfig {
                    verbose: false,
                    trace:   false,
                });
                if let Some((key, val)) = parse_kv(trimmed) {
                    match key {
                        "verbose" => current.verbose = val == "true",
                        "trace"   => current.trace   = val == "true",
                        _ => {}
                    }
                }
                run_cfg = Some(current);
            }
            "lint" => {
                let mut current = lint_cfg.take().unwrap_or(LintTomlConfig {
                    warn_as_error: None,
                    allow: None,
                });
                if let Some((key, val)) = parse_kv(trimmed) {
                    // Parse comma-separated list: warn_as_error = ["W006", "W007"]
                    let codes: Vec<String> = val
                        .trim_start_matches('[')
                        .trim_end_matches(']')
                        .split(',')
                        .map(|s| s.trim().trim_matches('"').to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                    match key {
                        "warn_as_error" => current.warn_as_error = Some(codes),
                        "allow"         => current.allow         = Some(codes),
                        _ => {}
                    }
                }
                lint_cfg = Some(current);
            }
            "context" => {
                let mut current = context_cfg.take().unwrap_or(ContextConfig {
                    db_url:  None,
                    storage: None,
                    http:    None,
                });
                if let Some((key, val)) = parse_kv(trimmed) {
                    match key {
                        "db_url"  => current.db_url  = Some(expand_env_vars(val)),
                        "storage" => current.storage = Some(val.to_string()),
                        "http"    => current.http    = Some(val.to_string()),
                        _ => {}
                    }
                }
                context_cfg = Some(current);
            }
            "azure" => {
                let mut current = azure_cfg.take().unwrap_or(AzureTomlConfig {
                    postgres_url:    None,
                    storage_account: None,
                    storage_key:     None,
                    container:       None,
                });
                if let Some((key, val)) = parse_kv(trimmed) {
                    match key {
                        "postgres_url"    => current.postgres_url    = Some(expand_env_vars(val)),
                        "storage_account" => current.storage_account = Some(expand_env_vars(val)),
                        "storage_key"     => current.storage_key     = Some(expand_env_vars(val)),
                        "container"       => current.container       = Some(val.to_string()),
                        _ => {}
                    }
                }
                azure_cfg = Some(current);
            }
            "gcp" => {
                let mut current = gcp_cfg.take().unwrap_or(GcpTomlConfig {
                    project_id:       None,
                    credentials_file: None,
                    dataset:          None,
                });
                if let Some((key, val)) = parse_kv(trimmed) {
                    match key {
                        "project_id"       => current.project_id       = Some(expand_env_vars(val)),
                        "credentials_file" => current.credentials_file = Some(expand_env_vars(val)),
                        "dataset"          => current.dataset           = Some(val.to_string()),
                        _ => {}
                    }
                }
                gcp_cfg = Some(current);
            }
            "kafka" => {
                let mut current = kafka_cfg.take().unwrap_or(KafkaTomlConfig {
                    bootstrap_brokers: None,
                    sasl_mechanism:    None,
                    sasl_username:     None,
                    sasl_password:     None,
                });
                if let Some((key, val)) = parse_kv(trimmed) {
                    match key {
                        "bootstrap_brokers" => current.bootstrap_brokers = Some(expand_env_vars(val)),
                        "sasl_mechanism"    => current.sasl_mechanism    = Some(val.to_string()),
                        "sasl_username"     => current.sasl_username     = Some(expand_env_vars(val)),
                        "sasl_password"     => current.sasl_password     = Some(expand_env_vars(val)),
                        _ => {}
                    }
                }
                kafka_cfg = Some(current);
            }
            "dev-dependencies" => {
                if let Some(dep) = parse_dep_line(trimmed) {
                    dev_dependencies.push(dep);
                }
            }
            "registry" => {
                if let Some((key, val)) = parse_kv(trimmed) {
                    if key == "url" {
                        registry_url = Some(val.to_string());
                    }
                }
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
        snowflake: snowflake_cfg,
        postgres: postgres_cfg,
        run: run_cfg,
        lint: lint_cfg,
        context: context_cfg,
        gcp: gcp_cfg,
        kafka: kafka_cfg,
        azure: azure_cfg,
        dev_dependencies,
        registry_url,
    }
}

/// Append a dependency line to `[dependencies]` (or `[dev-dependencies]`) in a `fav.toml` file.
/// Creates the section if it does not exist.
pub fn fav_toml_add_dep(
    toml_path: &std::path::Path,
    dep_name: &str,
    version_req: &str,
    dev: bool,
) -> Result<(), String> {
    let content = std::fs::read_to_string(toml_path).map_err(|e| e.to_string())?;
    let section_header = if dev { "[dev-dependencies]" } else { "[dependencies]" };
    let dep_line = format!("{} = \"{}\"", dep_name, version_req);

    let new_content = if let Some(pos) = content.find(section_header) {
        // Section exists — find where the next section starts (or EOF) and insert before it
        let after_header = &content[pos + section_header.len()..];
        let insert_offset = after_header.find("\n[").unwrap_or(after_header.len());
        let insert_pos = pos + section_header.len() + insert_offset;
        format!("{}\n{}{}", &content[..insert_pos], dep_line, &content[insert_pos..])
    } else {
        // Section does not exist — append it at the end
        format!("{}\n\n{}\n{}\n", content.trim_end(), section_header, dep_line)
    };

    std::fs::write(toml_path, new_content).map_err(|e| e.to_string())
}

/// Parse a dependency line: `name = "^1.0.0"` or `name = { key = "val", ... }`
fn parse_dep_line(line: &str) -> Option<DependencySpec> {
    let (dep_name, rest) = line.split_once('=')?;
    let dep_name = dep_name.trim().to_string();
    let rhs = rest.trim();

    // Plain string form: `name = "1.0.0"` or `name = "^1.0.0"`
    if rhs.starts_with('"') && rhs.ends_with('"') && !rhs.contains('{') {
        let version = rhs.trim_matches('"').to_string();
        return Some(DependencySpec::Semver {
            name: dep_name,
            version,
        });
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

/// Expand `${VAR_NAME}` references using environment variables.
/// Unset variables are replaced with an empty string.
pub fn expand_env_vars(s: &str) -> String {
    let mut result = String::new();
    let mut rest = s;
    while let Some(start) = rest.find("${") {
        result.push_str(&rest[..start]);
        let after = &rest[start + 2..];
        if let Some(end) = after.find('}') {
            let var_name = &after[..end];
            result.push_str(&std::env::var(var_name).unwrap_or_default());
            rest = &after[end + 1..];
        } else {
            result.push_str("${");
            rest = after;
        }
    }
    result.push_str(rest);
    result
}

/// Parse `key = "value"` or `key = value` (unquoted).
fn parse_kv(line: &str) -> Option<(&str, &str)> {
    let (key, rest) = line.split_once('=')?;
    let key = key.trim();
    let val = rest.trim().trim_matches('"');
    Some((key, val))
}

// ── rune_modules helpers ──────────────────────────────────────────────────────

/// Determine the entry `.fav` file for an installed rune in `rune_dir`.
///
/// Reads `<rune_dir>/rune.toml` and extracts the `entry` field from `[rune]`.
/// Falls back to `<rune_dir>/<name>.fav` when the file is absent or entry is empty.
pub fn rune_entry_file(rune_dir: &Path, name: &str) -> PathBuf {
    if let Ok(content) = std::fs::read_to_string(rune_dir.join("rune.toml")) {
        let mut in_rune = false;
        for line in content.lines() {
            let t = line.trim();
            if t == "[rune]" {
                in_rune = true;
                continue;
            }
            if t.starts_with('[') {
                in_rune = false;
                continue;
            }
            if in_rune {
                if let Some((k, v)) = t.split_once('=') {
                    if k.trim() == "entry" {
                        let entry = v.trim().trim_matches('"');
                        if !entry.is_empty() {
                            return rune_dir.join(entry);
                        }
                    }
                }
            }
        }
    }
    rune_dir.join(format!("{}.fav", name))
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

    #[test]
    fn test_rune_entry_file_with_entry_field() {
        let dir = tempfile::tempdir().unwrap();
        let rune_dir = dir.path().join("csv");
        std::fs::create_dir_all(&rune_dir).unwrap();
        std::fs::write(
            rune_dir.join("rune.toml"),
            "[rune]\nname = \"csv\"\nversion = \"0.1.0\"\nentry = \"csv.fav\"\n",
        )
        .unwrap();
        let result = rune_entry_file(&rune_dir, "csv");
        assert_eq!(result, rune_dir.join("csv.fav"));
    }

    #[test]
    fn test_rune_entry_file_fallback_when_no_rune_toml() {
        let dir = tempfile::tempdir().unwrap();
        let rune_dir = dir.path().join("csv");
        std::fs::create_dir_all(&rune_dir).unwrap();
        // No rune.toml
        let result = rune_entry_file(&rune_dir, "csv");
        assert_eq!(result, rune_dir.join("csv.fav"));
    }

    #[test]
    fn toml_snowflake_section_parsed() {
        let t = parse(
            "[rune]\nname = \"app\"\nversion = \"1.0.0\"\n\
             [snowflake]\naccount = \"myaccount\"\nuser = \"myuser\"\nwarehouse = \"WH\"\ndatabase = \"DB\"\n",
        );
        let sf = t.snowflake.expect("snowflake config");
        assert_eq!(sf.account.as_deref(),   Some("myaccount"));
        assert_eq!(sf.user.as_deref(),      Some("myuser"));
        assert_eq!(sf.warehouse.as_deref(), Some("WH"));
        assert_eq!(sf.database.as_deref(),  Some("DB"));
        assert!(sf.role.is_none());
        assert!(sf.schema.is_none());
    }

    #[test]
    fn toml_snowflake_env_var_expanded() {
        unsafe { std::env::set_var("TEST_SF_ACCT_10700", "myaccount"); }
        let expanded = expand_env_vars("${TEST_SF_ACCT_10700}.snowflakecomputing.com");
        assert_eq!(expanded, "myaccount.snowflakecomputing.com");
        unsafe { std::env::remove_var("TEST_SF_ACCT_10700"); }
    }

    #[test]
    fn test_rune_entry_file_fallback_when_entry_empty() {
        let dir = tempfile::tempdir().unwrap();
        let rune_dir = dir.path().join("mylib");
        std::fs::create_dir_all(&rune_dir).unwrap();
        std::fs::write(rune_dir.join("rune.toml"), "[rune]\nname = \"mylib\"\n").unwrap();
        let result = rune_entry_file(&rune_dir, "mylib");
        assert_eq!(result, rune_dir.join("mylib.fav"));
    }
}
