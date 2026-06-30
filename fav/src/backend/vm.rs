/// Bytecode format / VM compatibility version.
/// Independent from the Favnir language version (`fav --version`).
pub const VM_VERSION: &str = "1.0.0";

use arrow::array::{
    Array, ArrayRef, BooleanArray, BooleanBuilder, Float64Array, Float64Builder, Int64Array,
    Int64Builder, StringArray, StringBuilder,
};
use arrow::datatypes::{DataType, Field as ArrowField, Schema as ArrowSchema};
use arrow::record_batch::RecordBatch;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use bytes::Bytes;
use chrono::Utc;
use hmac::{Hmac, Mac};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use parquet::arrow::arrow_writer::ArrowWriter;
use rand::RngCore;
use rusqlite::Connection;
use serde_json::Value as SerdeJsonValue;
use sha2::{Digest, Sha256};
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

type HmacSha256 = Hmac<Sha256>;

use super::artifact::FvcArtifact;
use super::codegen::{Constant, Opcode};
use super::heap_val::HeapVal;
use super::nan_val::{NanVal, RecordMap};
use crate::middle::ir::TypeMeta;
use crate::schemas::ProjectSchemas;
use crate::value::Value;

thread_local! {
    /// When set to `true`, all `IO.println` / `IO.print` output is silently
    /// discarded.  Used by `cmd_test` when `--no-capture` is NOT given so that
    /// test bodies don't pollute the test-runner output.
    static SUPPRESS_IO_OUTPUT: Cell<bool> = const { Cell::new(false) };

    /// Coverage tracking: `Some(set)` when coverage is enabled, `None` otherwise.
    static COVERED_LINES: RefCell<Option<HashSet<u32>>> = RefCell::new(None);

    /// IO capture buffer: when `Some`, IO output is appended here instead of
    /// being printed to stdout.  Used by integration tests to inspect output.
    static IO_CAPTURE: RefCell<Option<String>> = RefCell::new(None);

    /// Test argv override: when `Some`, `IO.argv` returns these values instead
    /// of reading from `std::env::args()`.  Used by bootstrap tests.
    static TEST_ARGV: RefCell<Option<Vec<String>>> = RefCell::new(None);

    /// DB connection store: maps handle ID → connection wrapper.
    /// Transactions are tracked as (conn_id, in_tx flag).
    static DB_CONNECTIONS: RefCell<HashMap<u64, DbConnWrapper>> = RefCell::new(HashMap::new());
    static DB_NEXT_ID: Cell<u64> = const { Cell::new(1) };

    /// Seeded RNG for deterministic generation (v3.5.0).
    /// When `Some`, Random.int / Random.float / Gen.* use this instead of thread_rng.
    static SEEDED_RNG: RefCell<Option<rand::rngs::SmallRng>> = const { RefCell::new(None) };

    /// Profile records (v9.9.0): stage name + elapsed ms, cleared each run.
    static PROFILE_RECORDS: RefCell<Vec<(String, i64)>> = RefCell::new(Vec::new());

    /// Sequential ID counters for hint generation (v4.4.0).
    /// Key = "TypeName.field_name", value = next counter value.
    /// Reset when Random.seed is called.
    static HINT_ID_COUNTER: RefCell<HashMap<String, u64>> = RefCell::new(HashMap::new());

    /// Per-type YAML generation config (v4.4.0).
    static GEN_YAML_CONFIG: RefCell<HashMap<String, GenYamlConfig>> = RefCell::new(HashMap::new());

    static CHECKPOINT_BACKEND: RefCell<CheckpointBackend> = RefCell::new(CheckpointBackend::File {
        dir: PathBuf::from(".fav_checkpoints"),
    });

    /// Type constraint schemas loaded from schemas/*.yaml (v4.1.5).
    static SCHEMA_REGISTRY: RefCell<ProjectSchemas> = RefCell::new(HashMap::new());
}

/// Internal DB connection wrapper.
struct DbConnWrapper {
    conn: rusqlite::Connection,
    in_tx: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckpointBackend {
    File { dir: PathBuf },
    Sqlite { path: PathBuf },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckpointMetaRecord {
    pub name: String,
    pub value: String,
    pub updated_at: String,
}

/// Register project schemas for runtime validation (v4.1.5).
pub fn set_schema_registry(schemas: ProjectSchemas) {
    SCHEMA_REGISTRY.with(|s| *s.borrow_mut() = schemas);
}

thread_local! {
    /// Auth mode from fav.toml [auth] section (v4.5.0).
    static AUTH_MODE: RefCell<String> = RefCell::new("jwt".to_string());
}

/// Set the auth mode (called from cmd_run when fav.toml has [auth] section).
pub fn set_auth_mode(mode: &str) {
    AUTH_MODE.with(|m| *m.borrow_mut() = mode.to_string());
}

// ── AWS config (v4.11.0) ─────────────────────────────────────────────────────

/// AWS configuration from fav.toml [aws] section or environment variables.
#[derive(Debug, Clone)]
pub struct AwsConfig {
    pub region: String,
    pub endpoint_url: Option<String>,
    pub access_key: String,
    pub secret_key: String,
    pub session_token: Option<String>,
}

impl Default for AwsConfig {
    fn default() -> Self {
        AwsConfig {
            region: "us-east-1".to_string(),
            endpoint_url: None,
            access_key: "test".to_string(),
            secret_key: "test".to_string(),
            session_token: None,
        }
    }
}

impl AwsConfig {
    pub fn from_env() -> Self {
        AwsConfig {
            region: std::env::var("AWS_REGION")
                .or_else(|_| std::env::var("AWS_DEFAULT_REGION"))
                .unwrap_or_else(|_| "us-east-1".to_string()),
            endpoint_url: std::env::var("AWS_ENDPOINT_URL").ok(),
            access_key: std::env::var("AWS_ACCESS_KEY_ID").unwrap_or_else(|_| "test".to_string()),
            secret_key: std::env::var("AWS_SECRET_ACCESS_KEY")
                .unwrap_or_else(|_| "test".to_string()),
            session_token: std::env::var("AWS_SESSION_TOKEN").ok(),
        }
    }
}

thread_local! {
    static AWS_CONFIG: std::cell::RefCell<AwsConfig> =
        std::cell::RefCell::new(AwsConfig::from_env());
}

pub fn set_aws_config(cfg: AwsConfig) {
    AWS_CONFIG.with(|c| *c.borrow_mut() = cfg);
}

fn get_aws_config() -> AwsConfig {
    AWS_CONFIG.with(|c| c.borrow().clone())
}

/// Public accessor for use from driver.rs (deploy command).
pub fn get_aws_config_pub() -> AwsConfig {
    get_aws_config()
}

// ── SigV4 helpers (v4.11.0) ──────────────────────────────────────────────────

fn sha256_hex_bytes(data: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(data);
    h.finalize().iter().map(|b| format!("{:02x}", b)).collect()
}

fn hmac_sha256_bytes(key: &[u8], data: &[u8]) -> Vec<u8> {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    type HmacSha256 = Hmac<Sha256>;
    let mut mac = HmacSha256::new_from_slice(key).expect("hmac key");
    mac.update(data);
    mac.finalize().into_bytes().to_vec()
}

fn sigv4_signing_key(secret: &str, date: &str, region: &str, service: &str) -> Vec<u8> {
    let k_secret = format!("AWS4{}", secret);
    let k_date = hmac_sha256_bytes(k_secret.as_bytes(), date.as_bytes());
    let k_region = hmac_sha256_bytes(&k_date, region.as_bytes());
    let k_service = hmac_sha256_bytes(&k_region, service.as_bytes());
    hmac_sha256_bytes(&k_service, b"aws4_request")
}

pub struct SignedHeaders {
    pub authorization: String,
    pub x_amz_date: String,
    pub x_amz_content_sha256: String,
    pub x_amz_security_token: Option<String>,
}

fn sigv4_sign(
    config: &AwsConfig,
    service: &str,
    method: &str,
    url: &str,
    body: &[u8],
) -> SignedHeaders {
    let body_hash = sha256_hex_bytes(body);
    // LocalStack: skip real signing
    if config.endpoint_url.is_some() {
        return SignedHeaders {
            authorization: "AWS4-HMAC-SHA256 Credential=test/20240101/us-east-1/s3/aws4_request, SignedHeaders=host;x-amz-date, Signature=dummy".into(),
            x_amz_date: "20240101T000000Z".into(),
            x_amz_content_sha256: body_hash,
            x_amz_security_token: None,
        };
    }
    let now = chrono::Utc::now();
    let amz_date = now.format("%Y%m%dT%H%M%SZ").to_string();
    let date_stamp = now.format("%Y%m%d").to_string();
    // Parse host from URL (simple extraction)
    let host = url
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .split('/')
        .next()
        .unwrap_or("");
    let path = {
        let after_scheme = if url.contains("://") {
            url.splitn(2, "://").nth(1).unwrap_or(url)
        } else {
            url
        };
        let after_host = after_scheme
            .splitn(2, '/')
            .nth(1)
            .map(|s| format!("/{}", s))
            .unwrap_or_else(|| "/".to_string());
        after_host
    };
    let (path_only, query) = if path.contains('?') {
        let mut it = path.splitn(2, '?');
        (
            it.next().unwrap_or("/").to_string(),
            it.next().unwrap_or("").to_string(),
        )
    } else {
        (path, String::new())
    };
    let (canonical_headers, signed_headers) = if let Some(token) = &config.session_token {
        (
            format!(
                "host:{}\nx-amz-content-sha256:{}\nx-amz-date:{}\nx-amz-security-token:{}\n",
                host, body_hash, amz_date, token
            ),
            "host;x-amz-content-sha256;x-amz-date;x-amz-security-token".to_string(),
        )
    } else {
        (
            format!(
                "host:{}\nx-amz-content-sha256:{}\nx-amz-date:{}\n",
                host, body_hash, amz_date
            ),
            "host;x-amz-content-sha256;x-amz-date".to_string(),
        )
    };
    let canonical_request = format!(
        "{}\n{}\n{}\n{}\n{}\n{}",
        method, path_only, query, canonical_headers, signed_headers, body_hash
    );
    let credential_scope = format!("{}/{}/{}/aws4_request", date_stamp, config.region, service);
    let string_to_sign = format!(
        "AWS4-HMAC-SHA256\n{}\n{}\n{}",
        amz_date,
        credential_scope,
        sha256_hex_bytes(canonical_request.as_bytes())
    );
    let signing_key = sigv4_signing_key(&config.secret_key, &date_stamp, &config.region, service);
    let signature: String = hmac_sha256_bytes(&signing_key, string_to_sign.as_bytes())
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect();
    let authorization = format!(
        "AWS4-HMAC-SHA256 Credential={}/{}, SignedHeaders={}, Signature={}",
        config.access_key, credential_scope, signed_headers, signature
    );
    SignedHeaders {
        authorization,
        x_amz_date: amz_date,
        x_amz_content_sha256: body_hash,
        x_amz_security_token: config.session_token.clone(),
    }
}

/// Public wrapper for driver.rs (deploy command).
pub fn sigv4_sign_pub(
    config: &AwsConfig,
    service: &str,
    method: &str,
    url: &str,
    body: &[u8],
) -> SignedHeaders {
    sigv4_sign(config, service, method, url, body)
}

fn aws_get(config: &AwsConfig, service: &str, url: &str) -> Result<String, String> {
    let h = sigv4_sign(config, service, "GET", url, b"");
    let mut req = ureq::get(url)
        .set("Authorization", &h.authorization)
        .set("x-amz-date", &h.x_amz_date)
        .set("x-amz-content-sha256", &h.x_amz_content_sha256);
    if let Some(token) = &h.x_amz_security_token {
        req = req.set("x-amz-security-token", token);
    }
    req.call()
        .map_err(|e| e.to_string())
        .and_then(|r| r.into_string().map_err(|e| e.to_string()))
}

fn aws_put(config: &AwsConfig, service: &str, url: &str, body: &str) -> Result<(), String> {
    let body_bytes = body.as_bytes();
    let h = sigv4_sign(config, service, "PUT", url, body_bytes);
    let mut req = ureq::put(url)
        .set("Authorization", &h.authorization)
        .set("x-amz-date", &h.x_amz_date)
        .set("x-amz-content-sha256", &h.x_amz_content_sha256);
    if let Some(token) = &h.x_amz_security_token {
        req = req.set("x-amz-security-token", token);
    }
    req.send_string(body).map(|_| ()).map_err(|e| e.to_string())
}

fn aws_get_bytes(config: &AwsConfig, service: &str, url: &str) -> Result<Vec<u8>, String> {
    use std::io::Read;
    let h = sigv4_sign(config, service, "GET", url, b"");
    let mut req = ureq::get(url)
        .set("Authorization", &h.authorization)
        .set("x-amz-date", &h.x_amz_date)
        .set("x-amz-content-sha256", &h.x_amz_content_sha256);
    if let Some(token) = &h.x_amz_security_token {
        req = req.set("x-amz-security-token", token);
    }
    req.call().map_err(|e| e.to_string()).and_then(|r| {
        let mut buf = Vec::new();
        r.into_reader()
            .read_to_end(&mut buf)
            .map_err(|e| e.to_string())?;
        Ok(buf)
    })
}

fn aws_put_bytes(config: &AwsConfig, service: &str, url: &str, body: &[u8]) -> Result<(), String> {
    let h = sigv4_sign(config, service, "PUT", url, body);
    let mut req = ureq::put(url)
        .set("Authorization", &h.authorization)
        .set("x-amz-date", &h.x_amz_date)
        .set("x-amz-content-sha256", &h.x_amz_content_sha256)
        .set("Content-Type", "application/octet-stream");
    if let Some(token) = &h.x_amz_security_token {
        req = req.set("x-amz-security-token", token);
    }
    req.send_bytes(body).map(|_| ()).map_err(|e| e.to_string())
}

fn aws_delete(config: &AwsConfig, service: &str, url: &str) -> Result<(), String> {
    let h = sigv4_sign(config, service, "DELETE", url, b"");
    let mut req = ureq::delete(url)
        .set("Authorization", &h.authorization)
        .set("x-amz-date", &h.x_amz_date)
        .set("x-amz-content-sha256", &h.x_amz_content_sha256);
    if let Some(token) = &h.x_amz_security_token {
        req = req.set("x-amz-security-token", token);
    }
    match req.call() {
        Ok(_) | Err(ureq::Error::Status(204, _)) | Err(ureq::Error::Status(200, _)) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

fn aws_head(config: &AwsConfig, service: &str, url: &str) -> Result<bool, String> {
    let h = sigv4_sign(config, service, "HEAD", url, b"");
    let builder = ureq::request("HEAD", url)
        .set("Authorization", &h.authorization)
        .set("x-amz-date", &h.x_amz_date)
        .set("x-amz-content-sha256", &h.x_amz_content_sha256);
    let builder = if let Some(token) = &h.x_amz_security_token {
        builder.set("x-amz-security-token", token)
    } else {
        builder
    };
    match builder.call() {
        Ok(_) => Ok(true),
        Err(ureq::Error::Status(404, _)) => Ok(false),
        Err(e) => Err(e.to_string()),
    }
}

fn aws_post(
    config: &AwsConfig,
    service: &str,
    url: &str,
    body: &str,
    content_type: &str,
    amz_target: Option<&str>,
) -> Result<String, String> {
    let body_bytes = body.as_bytes();
    let h = sigv4_sign(config, service, "POST", url, body_bytes);
    let mut req = ureq::post(url)
        .set("Authorization", &h.authorization)
        .set("x-amz-date", &h.x_amz_date)
        .set("x-amz-content-sha256", &h.x_amz_content_sha256)
        .set("Content-Type", content_type);
    if let Some(target) = amz_target {
        req = req.set("X-Amz-Target", target);
    }
    if let Some(token) = &h.x_amz_security_token {
        req = req.set("x-amz-security-token", token);
    }
    match req.send_string(body) {
        Ok(r) => r.into_string().map_err(|e| e.to_string()),
        Err(ureq::Error::Status(code, resp)) => {
            let body = resp.into_string().unwrap_or_default();
            Err(format!("HTTP {code}: {body}"))
        }
        Err(e) => Err(e.to_string()),
    }
}

fn extract_xml_tags(xml: &str, tag: &str) -> Vec<String> {
    let open = format!("<{}>", tag);
    let close = format!("</{}>", tag);
    let mut results = Vec::new();
    let mut start = 0;
    while let Some(pos) = xml[start..].find(&open) {
        let abs = start + pos + open.len();
        if let Some(end) = xml[abs..].find(&close) {
            results.push(xml[abs..abs + end].to_string());
            start = abs + end + close.len();
        } else {
            break;
        }
    }
    results
}

fn map_to_dynamo_item(m: &std::collections::HashMap<String, VMValue>) -> String {
    let mut parts = Vec::new();
    for (k, v) in m {
        let s = match v {
            VMValue::Str(s) => s.clone(),
            VMValue::Int(n) => n.to_string(),
            VMValue::Bool(b) => b.to_string(),
            other => format!("{:?}", other),
        };
        let escaped = s.replace('\\', "\\\\").replace('"', "\\\"");
        parts.push(format!(r#""{}":{{"S":"{}"}}"#, k, escaped));
    }
    format!("{{{}}}", parts.join(","))
}

fn dynamo_item_to_map(item: &serde_json::Value) -> std::collections::HashMap<String, VMValue> {
    let mut m = std::collections::HashMap::new();
    if let serde_json::Value::Object(obj) = item {
        for (k, v) in obj {
            let s = v
                .get("S")
                .and_then(|s| s.as_str())
                .unwrap_or("")
                .to_string();
            m.insert(k.clone(), VMValue::Str(s));
        }
    }
    m
}

fn dynamo_list_response(resp: &str) -> VMValue {
    match serde_json::from_str::<serde_json::Value>(resp) {
        Ok(v) => {
            let items = v
                .get("Items")
                .and_then(|i| i.as_array())
                .map(|arr| {
                    arr.iter()
                        .map(|item| VMValue::Record(dynamo_item_to_map(item)))
                        .collect()
                })
                .unwrap_or_default();
            ok_vm(VMValue::List(FavList::new(items)))
        }
        Err(e) => err_vm(VMValue::Str(e.to_string())),
    }
}

fn url_encode(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            _ => {
                out.push_str(&format!("%{:02X}", b));
            }
        }
    }
    out
}

// ── Azure Blob Shared Key signing (v14.5.0) ───────────────────────────────────

/// Generate (x-ms-date, Authorization) headers for Azure Blob Storage Shared Key auth.
///
/// `canonical_resource` format:
///   - blob ops: `/{account}/{container}/{blob_name}`
///   - list ops: `/{account}/{container}\ncomp:list\nprefix:{prefix}\nrestype:container`
fn azure_blob_sign(
    account: &str,
    key_b64: &str,
    method: &str,
    content_type: &str,
    content_length: usize,
    x_ms_blob_type: &str,
    canonical_resource: &str,
) -> Result<(String, String), String> {
    use base64::engine::general_purpose::STANDARD as B64;
    use base64::Engine;

    let now = chrono::Utc::now();
    let date = now.format("%a, %d %b %Y %H:%M:%S GMT").to_string();

    // CanonicalizedHeaders — x-ms-* headers sorted alphabetically
    let mut ms_headers: Vec<(String, String)> = vec![
        ("x-ms-date".to_string(), date.clone()),
        ("x-ms-version".to_string(), "2020-10-02".to_string()),
    ];
    if !x_ms_blob_type.is_empty() {
        ms_headers.push(("x-ms-blob-type".to_string(), x_ms_blob_type.to_string()));
    }
    ms_headers.sort_by(|a, b| a.0.cmp(&b.0));
    let canonical_headers: String = ms_headers
        .iter()
        .map(|(k, v)| format!("{}:{}\n", k, v))
        .collect();

    // Content-Length: empty string when 0 (Azure Shared Key convention)
    let content_length_str = if content_length > 0 {
        content_length.to_string()
    } else {
        String::new()
    };

    // Full Shared Key StringToSign (Blob service):
    // VERB\n Content-Encoding\n Content-Language\n Content-Length\n Content-MD5\n
    // Content-Type\n Date\n If-Modified-Since\n If-Match\n If-None-Match\n
    // If-Unmodified-Since\n Range\n CanonicalizedHeaders CanonicalizedResource
    let string_to_sign = format!(
        "{}\n\n\n{}\n\n{}\n\n\n\n\n\n\n{}{}",
        method,
        content_length_str,
        content_type,
        canonical_headers,
        canonical_resource
    );

    let key_bytes = B64
        .decode(key_b64)
        .map_err(|e| format!("azure_blob_sign: invalid storage key: {}", e))?;
    let sig_bytes = hmac_sha256_bytes(&key_bytes, string_to_sign.as_bytes());
    let sig = B64.encode(&sig_bytes);
    Ok((date, format!("SharedKey {}:{}", account, sig)))
}

// ── Env config (v4.7.0) ───────────────────────────────────────────────────────

/// Env configuration from fav.toml [env] section.
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

thread_local! {
    static ENV_CONFIG: RefCell<EnvConfig> = RefCell::new(EnvConfig::default());
}

/// Set the env config (called from cmd_run).
pub fn set_env_config(cfg: EnvConfig) {
    ENV_CONFIG.with(|c| *c.borrow_mut() = cfg);
}

/// Resolve a key by applying the configured prefix.
fn env_resolve_key(key: &str) -> String {
    ENV_CONFIG.with(|c| {
        let cfg = c.borrow();
        if cfg.prefix.is_empty() {
            key.to_string()
        } else {
            format!("{}{}", cfg.prefix, key)
        }
    })
}

/// Parse a .env file's content into (key, value) pairs.
/// Skips blank lines and `#` comments; strips surrounding quotes from values.
pub(crate) fn parse_dotenv_content(content: &str) -> Vec<(String, String)> {
    content
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                return None;
            }
            let (k, v) = line.split_once('=')?;
            let key = k.trim().to_string();
            if key.is_empty() {
                return None;
            }
            let val = v.trim().trim_matches('"').trim_matches('\'').to_string();
            Some((key, val))
        })
        .collect()
}

// ── Log config (v4.6.0) ───────────────────────────────────────────────────────

/// Log configuration from fav.toml [log] section.
#[derive(Debug, Clone)]
pub struct LogConfig {
    pub level: String,  // "debug" | "info" | "warn" | "error"
    pub format: String, // "json" | "text"
    pub output: String, // "stdout" | "stderr"
    pub service: String,
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

thread_local! {
    static LOG_CONFIG: RefCell<LogConfig> = RefCell::new(LogConfig::default());
    static LOG_CODES: RefCell<std::collections::HashMap<String, String>> =
        RefCell::new(std::collections::HashMap::new());
}

// ── In-process cache store (v7.3.0) ─────────────────────────────────────────
thread_local! {
    static CACHE_STORE: RefCell<std::collections::HashMap<String, String>> =
        RefCell::new(std::collections::HashMap::new());
}

/// Set the log config (called from cmd_run).
pub fn set_log_config(cfg: LogConfig) {
    LOG_CONFIG.with(|c| *c.borrow_mut() = cfg);
}

/// Set custom log codes loaded from logs/*.yaml.
pub fn set_log_codes(codes: std::collections::HashMap<String, String>) {
    LOG_CODES.with(|c| *c.borrow_mut() = codes);
}

/// Returns true if `emit_level` passes the configured level filter.
fn log_level_passes(emit_level: &str) -> bool {
    LOG_CONFIG.with(|c| {
        let cfg = c.borrow();
        match cfg.level.as_str() {
            "error" => emit_level == "ERROR",
            "warn" => matches!(emit_level, "ERROR" | "WARN"),
            "info" => matches!(emit_level, "ERROR" | "WARN" | "INFO" | "SUCCESS"),
            _ => true, // "debug" — all pass
        }
    })
}

/// Format a UTC timestamp as "[2026-05-17 10:30:00]".
fn log_timestamp_text() -> String {
    let now = Utc::now();
    format!(
        "[{:04}-{:02}-{:02} {:02}:{:02}:{:02}]",
        now.format("%Y"),
        now.format("%m"),
        now.format("%d"),
        now.format("%H"),
        now.format("%M"),
        now.format("%S"),
    )
}

/// Format a UTC timestamp as "2026-05-17T10:30:00Z".
fn log_timestamp_iso() -> String {
    Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

/// Unix epoch milliseconds.
fn log_timestamp_millis() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// Parse ctx_json and render as "key=val  key2=val2" for text format.
fn log_ctx_to_text(ctx_json: &str) -> String {
    let ctx_json = ctx_json.trim();
    if ctx_json == "{}" || ctx_json.is_empty() {
        return String::new();
    }
    match serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(ctx_json) {
        Ok(map) => {
            let parts: Vec<String> = map
                .iter()
                .map(|(k, v)| {
                    let val = match v {
                        serde_json::Value::String(s) => s.clone(),
                        other => other.to_string(),
                    };
                    format!("{}={}", k, val)
                })
                .collect();
            if parts.is_empty() {
                String::new()
            } else {
                format!("  {}", parts.join("  "))
            }
        }
        Err(_) => format!("  {}", ctx_json),
    }
}

/// Format a log line in text format.
fn log_format_text(level: &str, code: &str, message: &str, ctx_json: &str) -> String {
    let ts = log_timestamp_text();
    let ctx = log_ctx_to_text(ctx_json);
    format!("{} {:<7} {:<6} {}{}", ts, level, code, message, ctx)
}

/// Format a log line in JSON format.
fn log_format_json(
    level: &str,
    code: &str,
    message: &str,
    ctx_json: &str,
    service: &str,
) -> String {
    let ts = log_timestamp_iso();
    let ctx: serde_json::Value =
        serde_json::from_str(ctx_json).unwrap_or(serde_json::Value::Object(serde_json::Map::new()));
    let mut obj = serde_json::Map::new();
    obj.insert("ts".to_string(), serde_json::Value::String(ts));
    obj.insert(
        "level".to_string(),
        serde_json::Value::String(level.to_string()),
    );
    obj.insert(
        "code".to_string(),
        serde_json::Value::String(code.to_string()),
    );
    obj.insert(
        "msg".to_string(),
        serde_json::Value::String(message.to_string()),
    );
    if !service.is_empty() {
        obj.insert(
            "service".to_string(),
            serde_json::Value::String(service.to_string()),
        );
    }
    obj.insert("ctx".to_string(), ctx);
    serde_json::Value::Object(obj).to_string()
}

/// Format a metric in CloudWatch EMF format.
fn log_metric_emf(name: &str, value: i64, unit: &str) -> String {
    let ts = log_timestamp_millis();
    let mut outer = serde_json::Map::new();
    let mut aws = serde_json::Map::new();
    aws.insert(
        "Timestamp".to_string(),
        serde_json::Value::Number(ts.into()),
    );
    let metric_obj = serde_json::json!([{
        "Namespace": "favnir",
        "Dimensions": [[]],
        "Metrics": [{"Name": name, "Unit": unit}]
    }]);
    aws.insert("CloudWatchMetrics".to_string(), metric_obj);
    outer.insert("_aws".to_string(), serde_json::Value::Object(aws));
    outer.insert(name.to_string(), serde_json::Value::Number(value.into()));
    serde_json::Value::Object(outer).to_string()
}

/// Internal helper: emit a log line regardless of rune context (used by VM error paths).
pub fn log_auto_emit(level: &str, code: &str, message: &str) {
    if !log_level_passes(level) {
        return;
    }
    LOG_CONFIG.with(|c| {
        let cfg = c.borrow();
        let line = if cfg.format == "json" {
            log_format_json(level, code, message, "{}", &cfg.service)
        } else {
            log_format_text(level, code, message, "{}")
        };
        if cfg.output == "stderr" {
            eprintln!("{}", line);
        } else {
            println!("{}", line);
        }
    });
}

/// Enable coverage tracking for the current thread.
pub fn enable_coverage() {
    COVERED_LINES.with(|c| *c.borrow_mut() = Some(HashSet::new()));
}

/// Disable coverage tracking and return the set of covered line numbers.
pub fn take_coverage() -> HashSet<u32> {
    COVERED_LINES.with(|c| c.borrow_mut().take().unwrap_or_default())
}

/// Start capturing IO output to an in-memory buffer.
/// All subsequent `IO.println` / `IO.print` calls append to the buffer.
#[allow(dead_code)]
pub fn start_io_capture() {
    IO_CAPTURE.with(|c| *c.borrow_mut() = Some(String::new()));
}

/// Stop capturing and return the accumulated output.
#[allow(dead_code)]
pub fn take_io_captured() -> String {
    IO_CAPTURE.with(|c| c.borrow_mut().take().unwrap_or_default())
}

/// Set a test-only argv override so `IO.argv` returns these values.
#[allow(dead_code)]
#[cfg(test)]
pub fn set_test_argv(args: Vec<String>) {
    TEST_ARGV.with(|t| *t.borrow_mut() = Some(args));
}

/// Clear the test argv override.
#[allow(dead_code)]
#[cfg(test)]
pub fn clear_test_argv() {
    TEST_ARGV.with(|t| *t.borrow_mut() = None);
}

/// Set whether IO output should be suppressed for the current thread.
/// Call `set_suppress_io(true)` before running tests, `set_suppress_io(false)`
/// after (or in a drop guard).
pub fn set_suppress_io(suppress: bool) {
    SUPPRESS_IO_OUTPUT.with(|c| c.set(suppress));
}

pub fn set_checkpoint_backend(backend: CheckpointBackend) {
    CHECKPOINT_BACKEND.with(|cell| {
        *cell.borrow_mut() = backend;
    });
}

/// Clear profiling records (called before each profiled execution).
pub fn clear_profile_records() {
    PROFILE_RECORDS.with(|r| r.borrow_mut().clear());
}

/// Return profiling records as a JSON string: `[{"name":"…","ms":123}, …]`.
pub fn take_profile_dump_json() -> String {
    PROFILE_RECORDS.with(|r| {
        let records = r.borrow();
        let entries: Vec<String> = records
            .iter()
            .map(|(name, ms)| {
                let name_json = serde_json::to_string(name).unwrap_or_else(|_| format!("\"{}\"", name));
                format!("{{\"name\":{},\"ms\":{}}}", name_json, ms)
            })
            .collect();
        format!("[{}]", entries.join(","))
    })
}

pub fn checkpoint_meta(name: &str) -> Result<CheckpointMetaRecord, String> {
    checkpoint_meta_impl(name)
}

pub fn checkpoint_list() -> Result<Vec<CheckpointMetaRecord>, String> {
    checkpoint_list_impl()
}

pub fn checkpoint_save_direct(name: &str, value: &str) -> Result<(), String> {
    checkpoint_save_impl(name, value)
}

pub fn checkpoint_reset_direct(name: &str) -> Result<(), String> {
    checkpoint_reset_impl(name)
}

fn current_timestamp_string() -> String {
    Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

fn with_checkpoint_backend<T>(
    f: impl FnOnce(&CheckpointBackend) -> Result<T, String>,
) -> Result<T, String> {
    CHECKPOINT_BACKEND.with(|cell| {
        let backend = cell.borrow().clone();
        f(&backend)
    })
}

fn checkpoint_value_path(dir: &Path, name: &str) -> PathBuf {
    dir.join(format!("{name}.txt"))
}

fn checkpoint_meta_path(dir: &Path, name: &str) -> PathBuf {
    dir.join(format!("{name}.meta.txt"))
}

fn checkpoint_meta_default(name: &str) -> CheckpointMetaRecord {
    CheckpointMetaRecord {
        name: name.to_string(),
        value: String::new(),
        updated_at: String::new(),
    }
}

fn ensure_checkpoint_dir(dir: &Path) -> Result<(), String> {
    std::fs::create_dir_all(dir).map_err(|e| {
        format!(
            "checkpoint backend failed to create `{}`: {}",
            dir.display(),
            e
        )
    })
}

fn write_checkpoint_meta_file(dir: &Path, meta: &CheckpointMetaRecord) -> Result<(), String> {
    ensure_checkpoint_dir(dir)?;
    let path = checkpoint_meta_path(dir, &meta.name);
    let body = serde_json::json!({
        "name": meta.name,
        "value": meta.value,
        "updated_at": meta.updated_at,
    })
    .to_string();
    std::fs::write(&path, body)
        .map_err(|e| format!("checkpoint write failed for `{}`: {}", path.display(), e))
}

fn read_checkpoint_meta_file(dir: &Path, name: &str) -> Result<CheckpointMetaRecord, String> {
    let path = checkpoint_meta_path(dir, name);
    if !path.exists() {
        return Ok(checkpoint_meta_default(name));
    }
    let body = std::fs::read_to_string(&path)
        .map_err(|e| format!("checkpoint read failed for `{}`: {}", path.display(), e))?;
    let json: serde_json::Value = serde_json::from_str(&body).map_err(|e| {
        format!(
            "checkpoint meta parse failed for `{}`: {}",
            path.display(),
            e
        )
    })?;
    Ok(CheckpointMetaRecord {
        name: json
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or(name)
            .to_string(),
        value: json
            .get("value")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        updated_at: json
            .get("updated_at")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
    })
}

fn ensure_checkpoint_table(conn: &rusqlite::Connection) -> Result<(), String> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS _fav_checkpoints (
            name TEXT PRIMARY KEY,
            value TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )",
        [],
    )
    .map(|_| ())
    .map_err(|e| format!("checkpoint sqlite setup failed: {}", e))
}

fn open_checkpoint_sqlite(path: &Path) -> Result<rusqlite::Connection, String> {
    let conn = rusqlite::Connection::open(path).map_err(|e| {
        format!(
            "checkpoint sqlite open failed for `{}`: {}",
            path.display(),
            e
        )
    })?;
    ensure_checkpoint_table(&conn)?;
    Ok(conn)
}

fn checkpoint_last_impl(name: &str) -> Result<Option<String>, String> {
    with_checkpoint_backend(|backend| match backend {
        CheckpointBackend::File { dir } => {
            ensure_checkpoint_dir(dir)?;
            let path = checkpoint_value_path(dir, name);
            if !path.exists() {
                return Ok(None);
            }
            let value = std::fs::read_to_string(&path)
                .map_err(|e| format!("checkpoint read failed for `{}`: {}", path.display(), e))?;
            Ok(Some(value))
        }
        CheckpointBackend::Sqlite { path } => {
            let conn = open_checkpoint_sqlite(path)?;
            let mut stmt = conn
                .prepare("SELECT value FROM _fav_checkpoints WHERE name = ?1")
                .map_err(|e| format!("checkpoint sqlite query prepare failed: {}", e))?;
            let mut rows = stmt
                .query([name])
                .map_err(|e| format!("checkpoint sqlite query failed: {}", e))?;
            match rows
                .next()
                .map_err(|e| format!("checkpoint sqlite row fetch failed: {}", e))?
            {
                Some(row) => {
                    let value: String = row
                        .get(0)
                        .map_err(|e| format!("checkpoint sqlite value decode failed: {}", e))?;
                    Ok(Some(value))
                }
                None => Ok(None),
            }
        }
    })
}

fn checkpoint_save_impl(name: &str, value: &str) -> Result<(), String> {
    let now = current_timestamp_string();
    with_checkpoint_backend(|backend| match backend {
        CheckpointBackend::File { dir } => {
            ensure_checkpoint_dir(dir)?;
            let value_path = checkpoint_value_path(dir, name);
            std::fs::write(&value_path, value).map_err(|e| {
                format!(
                    "checkpoint write failed for `{}`: {}",
                    value_path.display(),
                    e
                )
            })?;
            write_checkpoint_meta_file(
                dir,
                &CheckpointMetaRecord {
                    name: name.to_string(),
                    value: value.to_string(),
                    updated_at: now,
                },
            )
        }
        CheckpointBackend::Sqlite { path } => {
            let conn = open_checkpoint_sqlite(path)?;
            conn.execute(
                "INSERT INTO _fav_checkpoints(name, value, updated_at)
                 VALUES(?1, ?2, ?3)
                 ON CONFLICT(name) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
                rusqlite::params![name, value, now],
            )
            .map(|_| ())
            .map_err(|e| format!("checkpoint sqlite save failed: {}", e))
        }
    })
}

fn checkpoint_reset_impl(name: &str) -> Result<(), String> {
    with_checkpoint_backend(|backend| match backend {
        CheckpointBackend::File { dir } => {
            ensure_checkpoint_dir(dir)?;
            let value_path = checkpoint_value_path(dir, name);
            if value_path.exists() {
                std::fs::remove_file(&value_path).map_err(|e| {
                    format!(
                        "checkpoint reset failed for `{}`: {}",
                        value_path.display(),
                        e
                    )
                })?;
            }
            let meta_path = checkpoint_meta_path(dir, name);
            if meta_path.exists() {
                std::fs::remove_file(&meta_path).map_err(|e| {
                    format!(
                        "checkpoint reset failed for `{}`: {}",
                        meta_path.display(),
                        e
                    )
                })?;
            }
            Ok(())
        }
        CheckpointBackend::Sqlite { path } => {
            let conn = open_checkpoint_sqlite(path)?;
            conn.execute("DELETE FROM _fav_checkpoints WHERE name = ?1", [name])
                .map(|_| ())
                .map_err(|e| format!("checkpoint sqlite reset failed: {}", e))
        }
    })
}

fn checkpoint_meta_impl(name: &str) -> Result<CheckpointMetaRecord, String> {
    with_checkpoint_backend(|backend| match backend {
        CheckpointBackend::File { dir } => read_checkpoint_meta_file(dir, name),
        CheckpointBackend::Sqlite { path } => {
            let conn = open_checkpoint_sqlite(path)?;
            let mut stmt = conn
                .prepare("SELECT value, updated_at FROM _fav_checkpoints WHERE name = ?1")
                .map_err(|e| format!("checkpoint sqlite query prepare failed: {}", e))?;
            let mut rows = stmt
                .query([name])
                .map_err(|e| format!("checkpoint sqlite query failed: {}", e))?;
            match rows
                .next()
                .map_err(|e| format!("checkpoint sqlite row fetch failed: {}", e))?
            {
                Some(row) => {
                    let value: String = row
                        .get(0)
                        .map_err(|e| format!("checkpoint sqlite value decode failed: {}", e))?;
                    let updated_at: String = row.get(1).map_err(|e| {
                        format!("checkpoint sqlite updated_at decode failed: {}", e)
                    })?;
                    Ok(CheckpointMetaRecord {
                        name: name.to_string(),
                        value,
                        updated_at,
                    })
                }
                None => Ok(checkpoint_meta_default(name)),
            }
        }
    })
}

fn checkpoint_list_impl() -> Result<Vec<CheckpointMetaRecord>, String> {
    with_checkpoint_backend(|backend| match backend {
        CheckpointBackend::File { dir } => {
            ensure_checkpoint_dir(dir)?;
            let mut metas = Vec::new();
            let rd = std::fs::read_dir(dir)
                .map_err(|e| format!("checkpoint list failed for `{}`: {}", dir.display(), e))?;
            for entry in rd {
                let entry =
                    entry.map_err(|e| format!("checkpoint list entry read failed: {}", e))?;
                let path = entry.path();
                let Some(file_name) = path.file_name().and_then(|s| s.to_str()) else {
                    continue;
                };
                if let Some(name) = file_name.strip_suffix(".meta.txt") {
                    metas.push(read_checkpoint_meta_file(dir, name)?);
                }
            }
            metas.sort_by(|a, b| a.name.cmp(&b.name));
            Ok(metas)
        }
        CheckpointBackend::Sqlite { path } => {
            let conn = open_checkpoint_sqlite(path)?;
            let mut stmt = conn
                .prepare("SELECT name, value, updated_at FROM _fav_checkpoints ORDER BY name")
                .map_err(|e| format!("checkpoint sqlite list prepare failed: {}", e))?;
            let rows = stmt
                .query_map([], |row| {
                    Ok(CheckpointMetaRecord {
                        name: row.get(0)?,
                        value: row.get(1)?,
                        updated_at: row.get(2)?,
                    })
                })
                .map_err(|e| format!("checkpoint sqlite list failed: {}", e))?;
            let mut metas = Vec::new();
            for row in rows {
                metas.push(
                    row.map_err(|e| format!("checkpoint sqlite list row decode failed: {}", e))?,
                );
            }
            Ok(metas)
        }
    })
}

pub struct SuppressIoGuard {
    prev: bool,
}

impl SuppressIoGuard {
    pub fn new(suppress: bool) -> Self {
        let prev = is_io_suppressed();
        set_suppress_io(suppress);
        Self { prev }
    }
}

impl Drop for SuppressIoGuard {
    fn drop(&mut self) {
        set_suppress_io(self.prev);
    }
}

#[inline]
fn is_io_suppressed() -> bool {
    SUPPRESS_IO_OUTPUT.with(|c| c.get())
}

#[cfg(test)]
pub fn io_output_suppressed_for_tests() -> bool {
    is_io_suppressed()
}

#[derive(Debug, Clone, PartialEq)]
pub struct TraceFrame {
    pub fn_name: String,
    pub line: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VMError {
    pub message: String,
    pub fn_name: String,
    pub ip: usize,
    pub stack_trace: Vec<TraceFrame>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CallFrame {
    pub fn_idx: usize,
    pub ip: usize,
    pub base: usize,
    pub n_locals: usize,
    pub line: u32,
}

// Per-thread verbose level for `fav run --verbose` / `--trace` (v12.5.0).
// Thread-local ensures parallel test runs don't interfere with each other.
// Level: 0=off, 1=verbose/200-char truncation, 2=trace/no limit.
thread_local! {
    static VERBOSE_LEVEL: std::cell::Cell<u8> = const { std::cell::Cell::new(0) };
}

// ── v19.5.0: Arrow RecordBatch スレッドローカルストア ─────────────────────────
thread_local! {
    static ARROW_BATCHES: std::cell::RefCell<
        std::collections::HashMap<u64, arrow::record_batch::RecordBatch>
    > = std::cell::RefCell::new(std::collections::HashMap::new());
    static NEXT_ARROW_ID: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

// ── v20.4.0: DuckDB pushdown thread-locals ────────────────────────────────────
thread_local! {
    /// Whether pushdown explain output is enabled (set by --explain-pushdown flag).
    pub static PUSHDOWN_EXPLAIN_ENABLED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    /// Accumulated pushdown log entries for --explain-pushdown output.
    pub static PUSHDOWN_LOG: std::cell::RefCell<Vec<String>> = std::cell::RefCell::new(Vec::new());
}

// ── v23.1.0: Bytes スレッドローカルストア ────────────────────────────────────
// 注意: ArrowBatch / DbHandle と同じ opaque-handle パターン。
// エントリは明示的に削除されない（VMValue::Drop がないため）。
// 同一スレッドで大量の Bytes オブジェクトを生成するプログラムはメモリが増大する既知の制限。
// 将来バージョンで GC / 参照カウント削除を検討（v25.x 予定）。
thread_local! {
    static BYTES_STORE: std::cell::RefCell<
        std::collections::HashMap<u64, std::sync::Arc<Vec<u8>>>
    > = std::cell::RefCell::new(std::collections::HashMap::new());
    static NEXT_BYTES_ID: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

fn bytes_new(data: Vec<u8>) -> u64 {
    NEXT_BYTES_ID.with(|c| {
        let id = c.get();
        c.set(id + 1);
        BYTES_STORE.with(|m| m.borrow_mut().insert(id, std::sync::Arc::new(data)));
        id
    })
}

fn bytes_get_arc(id: u64) -> Option<std::sync::Arc<Vec<u8>>> {
    BYTES_STORE.with(|m| m.borrow().get(&id).cloned())
}

// v23.3.0: Mut コレクションストレージ（GC 未実装のためメモリリークあり、v25.x で対応予定）
// 安全性注記: MUT_LIST_STORE.borrow_mut() は vm_call_builtin の各 Mut アームで 1 回のみ呼ばれる。
// ネスト（borrow_mut 保持中に再度 Mut 操作を呼ぶ）はしないこと。
// ID は 0 始まり（NEXT_BYTES_ID と同パターン）。DB_NEXT_ID の 1 始まりとは異なる。
// MutMap のキー探索は Vec<(VMValue, VMValue)> の線形探索（O(n)）。エントリ数が少ない場合に限り適切。
thread_local! {
    static MUT_LIST_STORE: std::cell::RefCell<
        std::collections::HashMap<u64, Vec<VMValue>>
    > = std::cell::RefCell::new(std::collections::HashMap::new());
    static NEXT_MUT_LIST_ID: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
    static MUT_MAP_STORE: std::cell::RefCell<
        std::collections::HashMap<u64, Vec<(VMValue, VMValue)>>
    > = std::cell::RefCell::new(std::collections::HashMap::new());
    static NEXT_MUT_MAP_ID: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

fn mut_list_new() -> u64 {
    NEXT_MUT_LIST_ID.with(|c| {
        let id = c.get();
        c.set(id + 1);
        MUT_LIST_STORE.with(|m| m.borrow_mut().insert(id, Vec::new()));
        id
    })
}

fn mut_map_new() -> u64 {
    NEXT_MUT_MAP_ID.with(|c| {
        let id = c.get();
        c.set(id + 1);
        MUT_MAP_STORE.with(|m| m.borrow_mut().insert(id, Vec::new()));
        id
    })
}

/// Enable pushdown explain mode for the current thread.
pub fn set_pushdown_explain(enabled: bool) {
    PUSHDOWN_EXPLAIN_ENABLED.with(|c| c.set(enabled));
}

/// Take the accumulated pushdown log entries (clears the log).
pub fn take_pushdown_log() -> Vec<String> {
    PUSHDOWN_LOG.with(|log| std::mem::take(&mut *log.borrow_mut()))
}

// ── v22.1.0: Stage Checkpoint / Resume thread-locals ─────────────────────────
// NOTE: Named with STAGE_ prefix to avoid collision with the existing
// CheckpointBackend (used by `fav checkpoint` incremental processing).
thread_local! {
    static STAGE_CHECKPOINT_DIR: std::cell::RefCell<Option<std::path::PathBuf>>
        = std::cell::RefCell::new(None);
    static STAGE_RESUME_DIR: std::cell::RefCell<Option<std::path::PathBuf>>
        = std::cell::RefCell::new(None);
    static STAGE_CHECKPOINT_NAMES: std::cell::RefCell<std::collections::HashSet<String>>
        = std::cell::RefCell::new(std::collections::HashSet::new());
}

pub fn set_checkpoint_dir(dir: Option<&str>) {
    STAGE_CHECKPOINT_DIR.with(|c| *c.borrow_mut() = dir.map(std::path::PathBuf::from));
}

pub fn set_resume_dir(dir: Option<&str>) {
    STAGE_RESUME_DIR.with(|c| *c.borrow_mut() = dir.map(std::path::PathBuf::from));
}

pub fn set_checkpoint_stages(names: std::collections::HashSet<String>) {
    STAGE_CHECKPOINT_NAMES.with(|c| *c.borrow_mut() = names);
}

// ── v22.2.0: Distributed Worker endpoints thread-local ────────────────────────
thread_local! {
    static WORKER_ENDPOINTS: std::cell::RefCell<Vec<String>>
        = std::cell::RefCell::new(Vec::new());
}

pub fn set_worker_endpoints(endpoints: Vec<String>) {
    WORKER_ENDPOINTS.with(|c| *c.borrow_mut() = endpoints);
}

pub fn get_worker_endpoints() -> Vec<String> {
    WORKER_ENDPOINTS.with(|c| c.borrow().clone())
}

// ── v22.3.0: Pipeline State — in-memory backend (default) ─────────────────────
thread_local! {
    static STATE_STORE: std::cell::RefCell<std::collections::HashMap<String, String>>
        = std::cell::RefCell::new(std::collections::HashMap::new());
    static STATE_BACKEND: std::cell::RefCell<String>
        = std::cell::RefCell::new("memory".to_string());
}

pub fn set_state_backend(backend: &str) {
    if backend != "memory" {
        // v22.3.0: only "memory" backend is implemented.
        // Redis / DynamoDB / PostgreSQL will be added in v22.4+.
        eprintln!(
            "warning [v22.3.0]: state backend \"{}\" is not yet supported. \
             Falling back to in-memory backend. Upgrade to v22.4+ for external backends.",
            backend
        );
    }
    STATE_BACKEND.with(|c| *c.borrow_mut() = backend.to_string());
}

/// テスト用: STATE_STORE からキーを直接読む（クレート内のみ）
pub(crate) fn get_state_value(key: &str) -> Option<String> {
    STATE_STORE.with(|c| c.borrow().get(key).cloned())
}

/// テスト用: STATE_STORE にキーを直接書き込む（クレート内のみ）
pub(crate) fn set_state_value(key: &str, val: &str) {
    STATE_STORE.with(|c| c.borrow_mut().insert(key.to_string(), val.to_string()));
}

#[cfg(not(target_arch = "wasm32"))]
fn write_stage_checkpoint_bytes(dir: &std::path::Path, stage_name: &str, data: &[u8]) -> std::io::Result<()> {
    std::fs::create_dir_all(dir)?;
    let safe_name = stage_name.replace(['/', '\\', ' ', '.'], "_");
    let file_name = format!("{}.ckpt", safe_name);
    std::fs::write(dir.join(file_name), data)
}

#[cfg(not(target_arch = "wasm32"))]
fn read_stage_checkpoint_bytes(dir: &std::path::Path, stage_name: &str) -> Option<Vec<u8>> {
    let safe_name = stage_name.replace(['/', '\\', ' ', '.'], "_");
    let file_name = format!("{}.ckpt", safe_name);
    std::fs::read(dir.join(file_name)).ok()
}

fn arrow_store(batch: arrow::record_batch::RecordBatch) -> u64 {
    NEXT_ARROW_ID.with(|c| {
        let id = c.get();
        c.set(id + 1);
        ARROW_BATCHES.with(|m| m.borrow_mut().insert(id, batch));
        id
    })
}

fn arrow_get(id: u64) -> Option<arrow::record_batch::RecordBatch> {
    ARROW_BATCHES.with(|m| m.borrow().get(&id).cloned())
}

/// Set the thread-local verbose level for `fav run --verbose` / `--trace` (v12.5.0).
pub fn set_verbose_level(level: u8) {
    VERBOSE_LEVEL.with(|v| v.set(level));
}

#[derive(Debug, Clone)]
pub struct VM {
    globals: Vec<NanVal>,
    stack: Vec<NanVal>,
    frames: Vec<CallFrame>,
    collect_frames: Vec<Vec<NanVal>>,
    emit_log: Vec<NanVal>,
    db_path: Option<String>,
    source_file: String,
    type_metas: HashMap<String, TypeMeta>,
    /// Collected trace lines (when verbose > 0). Also written to stderr via eprintln!.
    pub trace_lines: Vec<String>,
    /// v22.7.0: 現在実行中 stage の OTel span ID。SeqStageEnter で設定、SeqStageCheck で消費。
    #[cfg(not(target_arch = "wasm32"))]
    pub current_otel_span_id: Option<crate::otel::SpanId>,
    /// Arena アロケータ — chunk ごとの Vec<VMValue> を pool 再利用（v20.7.0）
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) chunk_arena: crate::arena::ChunkArena,
    /// DAP デバッガー — debug_mode = true 時に dap_adapter フックを呼ぶ（v21.1.0）
    #[cfg(not(target_arch = "wasm32"))]
    pub debug_mode: bool,
    #[cfg(not(target_arch = "wasm32"))]
    pub dap_adapter: Option<crate::dap::DapAdapter>,
}

static SHARED_DBS: Mutex<Vec<(String, Connection)>> = Mutex::new(Vec::new());

// ── DuckDB connection store (v4.3.0) ─────────────────────────────────────────
// Global static (not thread_local): Rust does NOT call Drop on global statics,
// so duckdb::Connection::drop (which hangs on Windows joining worker threads)
// is never invoked automatically. Connections are closed explicitly via close_raw.
static DUCKDB_CONNS: std::sync::OnceLock<Mutex<HashMap<u64, duckdb::Connection>>> =
    std::sync::OnceLock::new();
static DUCKDB_NEXT_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

fn duckdb_store() -> std::sync::MutexGuard<'static, HashMap<u64, duckdb::Connection>> {
    DUCKDB_CONNS
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .unwrap()
}

/// Lazy stream representation for `Stream<T>` (v2.9.0)
#[derive(Debug, Clone)]
pub(crate) enum VMStream {
    /// Infinite: generates next value from current seed using next_fn
    Gen { seed: VMValue, next_fn: VMValue },
    /// Finite: converted from a list
    Of(Vec<VMValue>),
    /// Lazy map: apply map_fn to each element on collect
    Map {
        inner: Box<VMStream>,
        map_fn: VMValue,
    },
    /// Lazy filter: apply pred_fn to each element on collect
    Filter {
        inner: Box<VMStream>,
        pred_fn: VMValue,
    },
    /// Finite prefix of an inner stream
    Take { inner: Box<VMStream>, n: i64 },
    /// v26.4.0: flat_map — apply fn to each element, then flatten 1 level
    FlatMap { inner: Box<VMStream>, map_fn: VMValue },
    /// v26.4.0: window — batch into groups of `size` elements (tumbling window stub)
    Window { inner: Box<VMStream>, size: i64, window_fn: VMValue },
    /// v26.4.0: merge — concatenate multiple streams in order
    Merge { streams: Vec<VMStream> },
    /// v26.4.0: split — partition into [trues, falses] lists by predicate
    Split { inner: Box<VMStream>, pred_fn: VMValue },
}

/// Shared list with start offset enabling O(1) `List.drop` from the front.
/// Cloning is O(1) (Arc refcount bump). Mutation materialises a new Vec.
#[derive(Debug, Clone)]
pub(crate) struct FavList(pub(crate) Arc<Vec<VMValue>>, pub(crate) usize);

impl FavList {
    #[inline]
    pub(crate) fn new(v: Vec<VMValue>) -> Self {
        FavList(Arc::new(v), 0)
    }
    /// O(1) drop from the front — just advances the offset.
    #[inline]
    fn drop_front(&self, n: usize) -> FavList {
        FavList(self.0.clone(), (self.1 + n).min(self.0.len()))
    }
    /// O(n) take — creates a new backing Vec.
    #[inline]
    fn take_front(&self, n: usize) -> FavList {
        FavList::new(self.0[self.1..].iter().take(n).cloned().collect())
    }
    /// Materialise the virtual slice into an owned Vec (O(n)).
    #[inline]
    pub(crate) fn to_vec(&self) -> Vec<VMValue> {
        self.0[self.1..].iter().cloned().collect()
    }
}

impl std::ops::Deref for FavList {
    type Target = [VMValue];
    #[inline]
    fn deref(&self) -> &[VMValue] {
        &self.0[self.1..]
    }
}

impl PartialEq for FavList {
    fn eq(&self, other: &Self) -> bool {
        self.0[self.1..] == other.0[other.1..]
    }
}

impl IntoIterator for FavList {
    type Item = VMValue;
    type IntoIter = std::vec::IntoIter<VMValue>;
    fn into_iter(self) -> Self::IntoIter {
        self.to_vec().into_iter()
    }
}

#[derive(Debug, Clone)]
pub(crate) enum VMValue {
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
    Unit,
    List(FavList),
    Record(HashMap<String, VMValue>),
    Variant(String, Option<Box<VMValue>>),
    VariantCtor(String),
    CompiledFn(usize),
    Closure(usize, Vec<VMValue>),
    Builtin(String),
    /// `Stream<T>` lazy sequence (v2.9.0)
    Stream(Box<VMStream>),
    /// Opaque DB connection handle (v3.3.0)
    DbHandle(u64),
    /// Opaque DB transaction handle (v3.3.0)
    TxHandle(u64),
    /// v19.5.0: Apache Arrow RecordBatch への opaque handle
    ArrowBatch(u64),
    /// v20.8.0: DB コネクションプール opaque handle
    PgPool(u64),
    /// v23.1.0: 生バイト列 opaque handle
    Bytes(u64),
    /// v23.3.0: 可変リスト opaque handle
    MutList(u64),
    /// v23.3.0: 可変マップ opaque handle
    MutMap(u64),
}

impl PartialEq for VMValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (VMValue::Bool(a), VMValue::Bool(b)) => a == b,
            (VMValue::Int(a), VMValue::Int(b)) => a == b,
            (VMValue::Float(a), VMValue::Float(b)) => a == b,
            (VMValue::Str(a), VMValue::Str(b)) => a == b,
            (VMValue::Unit, VMValue::Unit) => true,
            (VMValue::List(a), VMValue::List(b)) => a == b,
            (VMValue::Record(a), VMValue::Record(b)) => a == b,
            (VMValue::Variant(n1, p1), VMValue::Variant(n2, p2)) => n1 == n2 && p1 == p2,
            (VMValue::VariantCtor(a), VMValue::VariantCtor(b)) => a == b,
            (VMValue::CompiledFn(a), VMValue::CompiledFn(b)) => a == b,
            (VMValue::Closure(a, ca), VMValue::Closure(b, cb)) => a == b && ca == cb,
            (VMValue::Builtin(a), VMValue::Builtin(b)) => a == b,
            (VMValue::Stream(_), VMValue::Stream(_)) => false, // streams are not comparable
            (VMValue::DbHandle(a), VMValue::DbHandle(b)) => a == b,
            (VMValue::TxHandle(a), VMValue::TxHandle(b)) => a == b,
            (VMValue::ArrowBatch(a), VMValue::ArrowBatch(b)) => a == b,
            (VMValue::PgPool(a),     VMValue::PgPool(b))     => a == b,
            (VMValue::Bytes(a),      VMValue::Bytes(b))      => a == b,
            (VMValue::MutList(a),    VMValue::MutList(b))    => a == b,
            (VMValue::MutMap(a),     VMValue::MutMap(b))     => a == b,
            _ => false,
        }
    }
}

impl VM {
    #[allow(dead_code)]
    pub fn new(artifact: &FvcArtifact) -> VM {
        Self::new_with_db_path(artifact, None)
    }

    pub fn new_with_db_path(artifact: &FvcArtifact, db_path: Option<String>) -> VM {
        let globals = artifact
            .globals
            .iter()
            .map(|g| match g.kind {
                0 => NanVal::from_heap(HeapVal::CompiledFn(g.fn_idx as usize)),
                1 => {
                    let name = artifact
                        .str_table
                        .get(g.name_idx as usize)
                        .cloned()
                        .unwrap_or_else(|| "<builtin>".to_string());
                    NanVal::from_heap(HeapVal::Builtin(name))
                }
                2 => {
                    let name = artifact
                        .str_table
                        .get(g.name_idx as usize)
                        .cloned()
                        .unwrap_or_else(|| "<variant>".to_string());
                    NanVal::from_heap(HeapVal::VariantCtor(name))
                }
                _ => NanVal::unit(),
            })
            .collect();
        VM {
            globals,
            stack: Vec::new(),
            frames: Vec::new(),
            collect_frames: Vec::new(),
            emit_log: Vec::new(),
            db_path,
            source_file: String::new(),
            type_metas: artifact.type_metas.clone(),
            trace_lines: Vec::new(),
            #[cfg(not(target_arch = "wasm32"))]
            current_otel_span_id: None,
            #[cfg(not(target_arch = "wasm32"))]
            chunk_arena: crate::arena::ChunkArena::new(),
            #[cfg(not(target_arch = "wasm32"))]
            debug_mode: false,
            #[cfg(not(target_arch = "wasm32"))]
            dap_adapter: None,
        }
    }

    pub fn set_source_file(&mut self, source_file: &str) {
        self.source_file = source_file.to_string();
    }

    // ── DAP デバッガー ヘルパー (v21.1.0) ────────────────────────────────────

    /// 現在フレームのローカル変数を DAP 用に収集する。
    /// スタックベース VM のため変数名は local_0, local_1, ... の連番。
    #[cfg(not(target_arch = "wasm32"))]
    fn collect_locals_for_dap(&self) -> Vec<(String, String, String)> {
        let Some(frame) = self.frames.last() else {
            return vec![];
        };
        let end = (frame.base + frame.n_locals).min(self.stack.len());
        self.stack[frame.base..end]
            .iter()
            .enumerate()
            .map(|(i, nanval)| {
                let val: VMValue = nanval.clone().to_vmvalue(); // clone() 必須（to_vmvalue は値渡し）
                (
                    format!("local_{}", i),
                    vmvalue_type_name(&val).to_string(),
                    vmvalue_repr(&val),
                )
            })
            .collect()
    }

    // ── verbose trace helpers (v12.5.0) ──────────────────────────────────────

    /// Return current verbose level from the thread-local.
    #[inline]
    fn verbose_level() -> u8 {
        VERBOSE_LEVEL.with(|v| v.get())
    }

    // ── verbose-aware run API (v12.5.0) ──────────────────────────────────────

    /// Run the given function and return (result, emits, trace_lines).
    /// Call `set_verbose_level` before invoking to enable tracing.
    pub fn run_with_trace(
        artifact: &FvcArtifact,
        fn_idx: usize,
        args: Vec<Value>,
        db_path: Option<&str>,
        source_file: Option<&str>,
    ) -> Result<(Value, Vec<Value>, Vec<String>), VMError> {
        let mut vm = VM::new_with_db_path(artifact, db_path.map(|s| s.to_string()));
        if let Some(sf) = source_file {
            vm.set_source_file(sf);
        }
        let ret = vm.invoke_function(artifact, fn_idx, args.into_iter().map(VMValue::from).collect())?;
        let value = Value::from(ret);
        let emits = vm.emit_log.into_iter().map(|v| Value::from(v.to_vmvalue())).collect();
        Ok((value, emits, vm.trace_lines))
    }

    #[allow(dead_code)]
    pub fn run(artifact: &FvcArtifact, fn_idx: usize, args: Vec<Value>) -> Result<Value, VMError> {
        Self::run_with_db_path(artifact, fn_idx, args, None).map(|(value, _)| value)
    }

    pub fn run_with_db_path(
        artifact: &FvcArtifact,
        fn_idx: usize,
        args: Vec<Value>,
        db_path: Option<&str>,
    ) -> Result<(Value, Vec<Value>), VMError> {
        Self::run_with_emits_and_db_path(artifact, fn_idx, args, db_path)
    }

    #[allow(dead_code)]
    pub fn run_with_emits(
        artifact: &FvcArtifact,
        fn_idx: usize,
        args: Vec<Value>,
    ) -> Result<(Value, Vec<Value>), VMError> {
        Self::run_with_emits_and_db_path(artifact, fn_idx, args, None)
    }

    pub fn run_with_emits_and_db_path(
        artifact: &FvcArtifact,
        fn_idx: usize,
        args: Vec<Value>,
        db_path: Option<&str>,
    ) -> Result<(Value, Vec<Value>), VMError> {
        Self::run_with_emits_db_path_and_source_file(artifact, fn_idx, args, db_path, None)
    }

    pub fn run_with_emits_db_path_and_source_file(
        artifact: &FvcArtifact,
        fn_idx: usize,
        args: Vec<Value>,
        db_path: Option<&str>,
        source_file: Option<&str>,
    ) -> Result<(Value, Vec<Value>), VMError> {
        let (value, emits) = Self::run_with_vmvalues(
            artifact,
            fn_idx,
            args.into_iter().map(VMValue::from).collect(),
            db_path.map(|s| s.to_string()),
            source_file.map(|s| s.to_string()),
        )?;
        Ok((
            Value::from(value),
            emits.into_iter().map(Value::from).collect(),
        ))
    }

    /// DAP デバッグモードで実行する（v21.1.0）。
    /// `self` に `debug_mode = true` と `dap_adapter` をセットしてから呼ぶこと。
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn run_debug(
        &mut self,
        artifact: &FvcArtifact,
        fn_idx: usize,
        args: Vec<VMValue>,
    ) -> Result<VMValue, VMError> {
        self.invoke_function(artifact, fn_idx, args)
    }

    fn run_with_vmvalues(
        artifact: &FvcArtifact,
        fn_idx: usize,
        args: Vec<VMValue>,
        db_path: Option<String>,
        source_file: Option<String>,
    ) -> Result<(VMValue, Vec<VMValue>), VMError> {
        let mut vm = VM::new_with_db_path(artifact, db_path);
        if let Some(source_file) = source_file {
            vm.set_source_file(&source_file);
        }
        let ret = vm.invoke_function(artifact, fn_idx, args)?;
        let emits: Vec<VMValue> = vm.emit_log.into_iter().map(|v| v.to_vmvalue()).collect();
        Ok((ret, emits))
    }

    fn invoke_function(
        &mut self,
        artifact: &FvcArtifact,
        fn_idx: usize,
        args: Vec<VMValue>,
    ) -> Result<VMValue, VMError> {
        let caller_depth = self.frames.len();
        let nan_args: Vec<NanVal> = args.into_iter().map(NanVal::from_vmvalue).collect();
        self.push_compiled_frame(artifact, fn_idx, nan_args)?;
        let ret = self.resume(artifact, caller_depth)?;
        Ok(ret.to_vmvalue())
    }

    /// Push a compiled function frame onto the call stack without running resume.
    /// Used by Call/CallNamed opcodes in the resume loop to avoid Rust recursion.
    fn push_compiled_frame(
        &mut self,
        artifact: &FvcArtifact,
        fn_idx: usize,
        args: Vec<NanVal>,
    ) -> Result<(), VMError> {
        let function = artifact.functions.get(fn_idx).ok_or_else(|| VMError {
            message: format!("unknown function index: {fn_idx}"),
            fn_name: "<invalid>".to_string(),
            ip: 0,
            stack_trace: vec![],
        })?;
        let base = self.stack.len();
        self.stack.extend(args);
        let required = function.local_count as usize;
        while self.stack.len() < base + required {
            self.stack.push(NanVal::unit());
        }
        self.frames.push(CallFrame {
            fn_idx,
            ip: 0,
            base,
            n_locals: required,
            line: function.source_line,
        });

        // ── DAP フック: stage 入口（v21.1.0）──────────────────────────────
        #[cfg(not(target_arch = "wasm32"))]
        if self.debug_mode {
            if let Some(adapter) = &self.dap_adapter {
                let fn_name = artifact
                    .str_table
                    .get(function.name_idx as usize)
                    .cloned()
                    .unwrap_or_else(|| format!("fn_{}", fn_idx));
                // ブレークポイント判定を先に行い、ヒット時のみ locals を収集（MED-7: lazy）
                let will_stop = {
                    let sess = adapter.session.lock().unwrap_or_else(|e| e.into_inner());
                    sess.is_breakpoint(&self.source_file, function.source_line) || adapter.step_mode
                };
                let locals = if will_stop { self.collect_locals_for_dap() } else { vec![] };
                adapter.on_hook(crate::dap::DapHook::StageEnter {
                    name: fn_name,
                    source: self.source_file.clone(),
                    line: function.source_line,
                    locals,
                });
                // ブレークポイント停止中は VM スレッドをここでブロック（HIGH-1）
                adapter.wait_if_stopped();
            }
        }

        Ok(())
    }

    /// Tail-call optimization: when the frame just pushed is a *self-recursive* tail call
    /// (callee == caller, i.e., the parent's effective next instruction is Return, possibly
    /// via Jump chains), replace the parent frame rather than stacking them.
    /// Restricted to self-recursion so that non-recursive call chains preserve stack traces.
    #[inline]
    fn try_apply_tco(&mut self, artifact: &FvcArtifact) {
        let frames_len = self.frames.len();
        if frames_len < 2 {
            return;
        }
        let parent_idx = frames_len - 2;
        // Only TCO self-recursive calls (same function) to preserve stack traces.
        if self.frames[parent_idx].fn_idx != self.frames[frames_len - 1].fn_idx {
            return;
        }
        let parent_next_ip = self.frames[parent_idx].ip;
        let parent_fn_code = &artifact.functions[self.frames[parent_idx].fn_idx].code;

        // Follow unconditional Jumps to find the effective next instruction.
        // Tail calls inside if/else branches emit Call + Jump → end_label, then Return.
        let mut ip = parent_next_ip;
        let is_tail_call = loop {
            if ip >= parent_fn_code.len() {
                break false;
            }
            let byte = parent_fn_code[ip];
            if byte == crate::backend::codegen::Opcode::Return as u8 {
                break true;
            } else if byte == crate::backend::codegen::Opcode::Jump as u8 {
                // Jump encodes a u16 forward offset; target = ip + 3 + offset
                if ip + 2 >= parent_fn_code.len() {
                    break false;
                }
                let lo = parent_fn_code[ip + 1];
                let hi = parent_fn_code[ip + 2];
                let offset = u16::from_le_bytes([lo, hi]) as usize;
                ip = ip + 3 + offset;
            } else {
                break false;
            }
        };
        if !is_tail_call {
            return;
        }

        // Replace parent frame with new frame:
        // stack layout: [0..parent_base] | parent's locals | [new_base..] new frame's locals
        let parent_base = self.frames[parent_idx].base;
        let new_base = self.frames[frames_len - 1].base;

        // Move new frame's stack segment down to parent_base, discarding parent's locals
        let new_locals: Vec<NanVal> = self.stack.drain(new_base..).collect();
        self.stack.truncate(parent_base);
        self.stack.extend(new_locals);

        // Swap parent and new frame so parent is last, then pop it
        self.frames.swap(parent_idx, frames_len - 1);
        self.frames.pop();

        // Update the (formerly new) frame's base to parent_base
        self.frames.last_mut().unwrap().base = parent_base;
    }

    fn resume(&mut self, artifact: &FvcArtifact, caller_depth: usize) -> Result<NanVal, VMError> {
        let vm = self;
        loop {
            let Some(frame) = vm.frames.last_mut() else {
                return Ok(NanVal::unit());
            };
            let function = &artifact.functions[frame.fn_idx];
            if frame.ip >= function.code.len() {
                return Err(vm.error(artifact, "instruction pointer out of bounds"));
            }
            let opcode = function.code[frame.ip];
            frame.ip += 1;

            match opcode {
                x if x == Opcode::Const as u8 => {
                    let idx = Self::read_u16(function, frame)? as usize;
                    let constant = function
                        .constants
                        .get(idx)
                        .ok_or_else(|| vm.error(artifact, "constant index out of bounds"))?;
                    vm.stack.push(constant_to_nan(constant.clone()));
                }
                x if x == Opcode::ConstUnit as u8 => vm.stack.push(NanVal::unit()),
                x if x == Opcode::ConstTrue as u8 => vm.stack.push(NanVal::from_bool(true)),
                x if x == Opcode::ConstFalse as u8 => vm.stack.push(NanVal::from_bool(false)),
                x if x == Opcode::LoadLocal as u8 => {
                    let slot = Self::read_u16(function, frame)? as usize;
                    let idx = frame.base + slot;
                    let value = vm
                        .stack
                        .get(idx)
                        .cloned()
                        .ok_or_else(|| vm.error(artifact, "local slot out of bounds"))?;
                    vm.stack.push(value);
                }
                x if x == Opcode::StoreLocal as u8 => {
                    let slot = Self::read_u16(function, frame)? as usize;
                    let idx = frame.base + slot;
                    let value = vm
                        .stack
                        .pop()
                        .ok_or_else(|| vm.error(artifact, "stack underflow on store"))?;
                    if idx >= vm.stack.len() {
                        vm.stack.resize_with(idx + 1, NanVal::unit);
                    }
                    vm.stack[idx] = value;
                }
                x if x == Opcode::LoadGlobal as u8 => {
                    let idx = Self::read_u16(function, frame)? as usize;
                    let value = match function.constants.get(idx) {
                        Some(crate::backend::codegen::Constant::Name(name)) => {
                            if let Some(fn_idx) = artifact.fn_idx_by_name(name) {
                                NanVal::from_heap(HeapVal::CompiledFn(fn_idx))
                            } else if is_known_builtin_namespace(name) {
                                NanVal::from_heap(HeapVal::Builtin(name.clone()))
                            } else if looks_like_variant_ctor(name) {
                                NanVal::from_heap(HeapVal::VariantCtor(name.clone()))
                            } else {
                                vm.globals.get(idx).cloned().ok_or_else(|| {
                                    vm.error(
                                        artifact,
                                        &format!("unknown global or builtin: {name}"),
                                    )
                                })?
                            }
                        }
                        _ => vm
                            .globals
                            .get(idx)
                            .cloned()
                            .ok_or_else(|| vm.error(artifact, "global index out of bounds"))?,
                    };
                    vm.stack.push(value);
                }
                x if x == Opcode::Pop as u8 => {
                    vm.stack
                        .pop()
                        .ok_or_else(|| vm.error(artifact, "stack underflow on pop"))?;
                }
                x if x == Opcode::Dup as u8 => {
                    let value = vm
                        .stack
                        .last()
                        .cloned()
                        .ok_or_else(|| vm.error(artifact, "stack underflow on dup"))?;
                    vm.stack.push(value);
                }
                x if x == Opcode::Jump as u8 => {
                    let offset = Self::read_u16(function, frame)? as usize;
                    let Some(next_ip) = frame.ip.checked_add(offset) else {
                        return Err(vm.error(artifact, "jump overflow"));
                    };
                    frame.ip = next_ip;
                }
                x if x == Opcode::JumpIfFalse as u8 => {
                    let offset = Self::read_u16(function, frame)? as usize;
                    let Some(cond) = vm.stack.pop() else {
                        return Err(vm.error(artifact, "stack underflow on conditional jump"));
                    };
                    match cond.as_bool() {
                        Some(false) => {
                            let Some(next_ip) = frame.ip.checked_add(offset) else {
                                return Err(vm.error(artifact, "jump overflow"));
                            };
                            frame.ip = next_ip;
                        }
                        Some(true) => {}
                        None => return Err(vm.error(artifact, "conditional jump requires a Bool")),
                    }
                }
                x if x == Opcode::MatchFail as u8 => {
                    return Err(vm.error(artifact, "non-exhaustive match"));
                }
                x if x == Opcode::ChainCheck as u8 => {
                    let offset = Self::read_u16(function, frame)? as usize;
                    let Some(value) = vm.stack.pop() else {
                        return Err(vm.error(artifact, "stack underflow on chain_check"));
                    };
                    match value.as_heap() {
                        Some(HeapVal::Variant(tag, payload))
                            if tag == "ok" || tag == "some" =>
                        {
                            let unwrapped = payload.as_ref().map(|v| v.clone()).ok_or_else(|| {
                                vm.error(artifact, "chain_check expected payload for ok/some")
                            })?;
                            let vlevel = Self::verbose_level();
                            if vlevel > 0 {
                                let uvm = unwrapped.clone().to_vmvalue();
                                let display = truncate_for_trace(&uvm, vlevel);
                                trace_emit(&mut vm.trace_lines, format!("[TRACE]   bind <- \u{2192} Ok({})", display));
                            }
                            vm.stack.push(unwrapped);
                        }
                        Some(HeapVal::Variant(tag, payload)) if tag == "err" => {
                            let vlevel = Self::verbose_level();
                            if vlevel > 0 {
                                let display = match payload {
                                    Some(v) => truncate_for_trace(&v.clone().to_vmvalue(), vlevel),
                                    None => String::new(),
                                };
                                trace_emit(&mut vm.trace_lines, format!("[TRACE]   bind <- \u{2192} Err({})", display));
                            }
                            vm.stack.push(value);
                            let Some(next_ip) = frame.ip.checked_add(offset) else {
                                return Err(vm.error(artifact, "jump overflow"));
                            };
                            frame.ip = next_ip;
                        }
                        Some(HeapVal::Variant(tag, None)) if tag == "none" => {
                            let vlevel = Self::verbose_level();
                            if vlevel > 0 {
                                trace_emit(&mut vm.trace_lines, "[TRACE]   bind <- \u{2192} None".to_string());
                            }
                            vm.stack.push(value);
                            let Some(next_ip) = frame.ip.checked_add(offset) else {
                                return Err(vm.error(artifact, "jump overflow"));
                            };
                            frame.ip = next_ip;
                        }
                        _ => {
                            return Err(vm.error(
                                artifact,
                                &format!(
                                    "chain_check requires ok/some/err/none variant, got {:?}",
                                    value
                                ),
                            ));
                        }
                    }
                }
                x if x == Opcode::LegacyBindCheck as u8 => {
                    let offset = Self::read_u16(function, frame)? as usize;
                    let Some(value) = vm.stack.pop() else {
                        return Err(vm.error(artifact, "stack underflow on legacy_bind_check"));
                    };
                    match value.as_heap() {
                        Some(HeapVal::Variant(tag, payload))
                            if tag == "ok" || tag == "some" =>
                        {
                            let unwrapped = payload.as_ref().map(|v| v.clone()).ok_or_else(|| {
                                vm.error(artifact, "legacy_bind_check expected payload for ok/some")
                            })?;
                            vm.stack.push(unwrapped);
                        }
                        Some(HeapVal::Variant(tag, _)) if tag == "err" || tag == "none" => {
                            vm.stack.push(value);
                            let Some(next_ip) = frame.ip.checked_add(offset) else {
                                return Err(vm.error(artifact, "jump overflow"));
                            };
                            frame.ip = next_ip;
                        }
                        _ => {
                            // Non-Result value: pass through unchanged (simple bind semantics)
                            vm.stack.push(value);
                        }
                    }
                }
                x if x == Opcode::SeqStageCheck as u8 => {
                    // layout: name_str_idx(2) + stage_idx(1) + total(1) + escape_offset(2)
                    let name_idx = Self::read_u16(function, frame)? as usize;
                    if frame.ip + 1 >= function.code.len() {
                        return Err(vm.error(artifact, "unexpected end of bytecode in seq_stage_check"));
                    }
                    let stage_idx = function.code[frame.ip] as usize;
                    frame.ip += 1;
                    let total = function.code[frame.ip] as usize;
                    frame.ip += 1;
                    let offset = Self::read_u16(function, frame)? as usize;
                    // frame.ip is now at StoreLocal
                    let Some(value) = vm.stack.pop() else {
                        return Err(vm.error(artifact, "stack underflow on seq_stage_check"));
                    };
                    let stage_name = artifact
                        .str_table
                        .get(name_idx)
                        .map(|s| s.as_str())
                        .unwrap_or("?");
                    match value.as_heap() {
                        Some(HeapVal::Variant(tag, payload))
                            if tag == "ok" || tag == "some" =>
                        {
                            let unwrapped = payload.as_ref().map(|v| v.clone()).ok_or_else(|| {
                                vm.error(artifact, "seq_stage_check expected payload for ok/some")
                            })?;
                            let vlevel = Self::verbose_level();
                            #[cfg(not(target_arch = "wasm32"))]
                            let needs_otel = vm.current_otel_span_id.is_some();
                            #[cfg(target_arch = "wasm32")]
                            let needs_otel = false;
                            let uvm = if vlevel > 0 || needs_otel {
                                Some(unwrapped.clone().to_vmvalue())
                            } else {
                                None
                            };
                            if vlevel > 0 {
                                let display = truncate_for_trace(uvm.as_ref().unwrap(), vlevel);
                                trace_emit(&mut vm.trace_lines, format!("[TRACE] stage {}: exit Ok({})", stage_name, display));
                            }
                            // v22.7.0: OTel span 終了（Ok）
                            #[cfg(not(target_arch = "wasm32"))]
                            if let Some(ref sid) = vm.current_otel_span_id.take() {
                                let out_items = otel_value_items(uvm.as_ref().unwrap());
                                crate::otel::otel_span_end(sid, 0, out_items, crate::otel::OtelStatus::Ok);
                            }
                            vm.stack.push(unwrapped);
                        }
                        Some(HeapVal::Variant(tag, payload))
                            if tag == "err" || tag == "none" =>
                        {
                            let inner_msg = match payload {
                                Some(v) => match v.as_str() {
                                    Some(s) => s.to_string(),
                                    None => format!("{v:?}"),
                                },
                                None => "none".to_string(),
                            };
                            let vlevel = Self::verbose_level();
                            if vlevel > 0 {
                                trace_emit(&mut vm.trace_lines, format!("[TRACE] stage {}: exit Err({})", stage_name, inner_msg));
                                trace_emit(&mut vm.trace_lines, format!(
                                    "[TRACE] seq: stopped at stage {}/{} ({})",
                                    stage_idx + 1, total, stage_name
                                ));
                            }
                            // v22.7.0: OTel span 終了（Err）
                            #[cfg(not(target_arch = "wasm32"))]
                            if let Some(ref sid) = vm.current_otel_span_id.take() {
                                crate::otel::otel_span_end(
                                    sid, 0, 0,
                                    crate::otel::OtelStatus::Error(inner_msg.clone()),
                                );
                            }
                            let wrapped = format!(
                                "pipeline stopped at stage {}/{} '{}': {}",
                                stage_idx + 1,
                                total,
                                stage_name,
                                inner_msg
                            );
                            vm.stack.push(NanVal::from_heap(HeapVal::Variant(
                                "err".to_string(),
                                Some(NanVal::from_str(wrapped)),
                            )));
                            let Some(next_ip) = frame.ip.checked_add(offset) else {
                                return Err(vm.error(artifact, "jump overflow"));
                            };
                            frame.ip = next_ip;
                        }
                        _ => {
                            // Non-Result value: pass through unchanged
                            // v22.7.0: OTel span 終了（非 Result 値）
                            #[cfg(not(target_arch = "wasm32"))]
                            if let Some(ref sid) = vm.current_otel_span_id.take() {
                                let out_items = otel_value_items(&value.clone().to_vmvalue());
                                crate::otel::otel_span_end(sid, 0, out_items, crate::otel::OtelStatus::Ok);
                            }
                            vm.stack.push(value);
                        }
                    }
                }
                x if x == Opcode::SeqStageEnter as u8 => {
                    // layout: name_str_idx(2)
                    let name_idx = Self::read_u16(function, frame)? as usize;
                    // v22.7.0: stage_name を hoist（OTel + verbose 両方で使う）
                    let stage_name = artifact
                        .str_table
                        .get(name_idx)
                        .map(|s| s.as_str())
                        .unwrap_or("?");
                    if Self::verbose_level() > 0 {
                        trace_emit(&mut vm.trace_lines, format!("[TRACE] stage {}: enter", stage_name));
                    }
                    // v22.7.0: OTel span 開始
                    #[cfg(not(target_arch = "wasm32"))]
                    if crate::otel::otel_is_enabled() {
                        let parent  = crate::otel::otel_current_parent();
                        let span_id = crate::otel::otel_span_start(
                            &format!("stage:{}", stage_name),
                            parent.as_ref(),
                        );
                        vm.current_otel_span_id = Some(span_id);
                    }
                }
                x if x == Opcode::MergeRecord as u8 => {
                    // Layout: n_overrides(2) + names_idx(2)
                    let n_overrides = Self::read_u16(function, frame)? as usize;
                    let names_idx = Self::read_u16(function, frame)? as usize;
                    // Read override field names from str_table
                    let names_str = artifact.str_table.get(names_idx).cloned().unwrap_or_default();
                    let field_names: Vec<String> = if names_str.is_empty() {
                        Vec::new()
                    } else {
                        names_str.split('\u{1f}').map(|s| s.to_string()).collect()
                    };
                    if field_names.len() != n_overrides {
                        return Err(vm.error(artifact, "MergeRecord: field name count mismatch"));
                    }
                    // Pop override values (pushed left-to-right → pop right-to-left)
                    let mut override_vals: Vec<NanVal> = (0..n_overrides)
                        .map(|_| vm.stack.pop().unwrap_or_else(NanVal::unit))
                        .collect();
                    override_vals.reverse();
                    // Pop base record
                    let base_val = vm.stack.pop().ok_or_else(|| {
                        vm.error(artifact, "MergeRecord: stack underflow on base")
                    })?;
                    let mut fields: RecordMap = match base_val.as_record() {
                        Some(map) => map.clone(),
                        None => return Err(vm.error(artifact, "MergeRecord: base is not a record")),
                    };
                    // Apply overrides
                    for (name, val) in field_names.into_iter().zip(override_vals) {
                        fields.insert(name, val);
                    }
                    vm.stack.push(NanVal::from_record(fields));
                }
                // ── list pattern opcodes (v17.2.0) ────────────────────────────
                x if x == Opcode::ListLen as u8 => {
                    let v = vm.stack.pop().ok_or_else(|| {
                        vm.error(artifact, "stack underflow on list_len")
                    })?;
                    match v.as_list() {
                        Some(fl) => vm.stack.push(NanVal::from_int(fl.len() as i64)),
                        None => return Err(vm.error(artifact, "list_len requires a List")),
                    }
                }
                x if x == Opcode::ListGet as u8 => {
                    let idx_val = vm.stack.pop().ok_or_else(|| {
                        vm.error(artifact, "stack underflow on list_get (index)")
                    })?;
                    let list_val = vm.stack.pop().ok_or_else(|| {
                        vm.error(artifact, "stack underflow on list_get (list)")
                    })?;
                    match (list_val.as_list(), idx_val.as_int()) {
                        (Some(fl), Some(i)) => {
                            let elem = fl.get(i as usize).cloned().ok_or_else(|| {
                                vm.error(artifact, "list_get: index out of bounds")
                            })?;
                            vm.stack.push(NanVal::from_vmvalue(elem));
                        }
                        _ => return Err(vm.error(artifact, "list_get requires (List, Int)")),
                    }
                }
                x if x == Opcode::ListDrop as u8 => {
                    let n_val = vm.stack.pop().ok_or_else(|| {
                        vm.error(artifact, "stack underflow on list_drop (n)")
                    })?;
                    let list_val = vm.stack.pop().ok_or_else(|| {
                        vm.error(artifact, "stack underflow on list_drop (list)")
                    })?;
                    match (list_val.as_list(), n_val.as_int()) {
                        (Some(fl), Some(n)) => {
                            vm.stack.push(NanVal::from_list(fl.drop_front(n.max(0) as usize)));
                        }
                        _ => return Err(vm.error(artifact, "list_drop requires (List, Int)")),
                    }
                }
                x if x == Opcode::RefinementAssert as u8 => {
                    let name_idx = Self::read_u16(function, frame)? as usize;
                    let param_name = artifact
                        .str_table
                        .get(name_idx)
                        .cloned()
                        .unwrap_or_else(|| "?".to_string());
                    let cond = vm.stack.pop().ok_or_else(|| {
                        vm.error(artifact, "stack underflow on refinement_assert")
                    })?;
                    match cond.as_bool() {
                        Some(true) => {}
                        Some(false) => {
                            return Err(vm.error(
                                artifact,
                                &format!("refinement violated: argument `{}` does not satisfy its constraint", param_name),
                            ));
                        }
                        None => {
                            return Err(vm.error(
                                artifact,
                                "refinement_assert expects a Bool",
                            ));
                        }
                    }
                }
                // ─── Superinstructions (v20.2.0) ─────────────────────────────────
                x if x == Opcode::AddLL as u8 => {
                    let a = Self::read_u16(function, frame)? as usize;
                    let b = Self::read_u16(function, frame)? as usize;
                    let base = frame.base;
                    let va = vm.stack.get(base + a).cloned()
                        .ok_or_else(|| vm.error(artifact, "AddLL: slot a out of bounds"))?;
                    let vb = vm.stack.get(base + b).cloned()
                        .ok_or_else(|| vm.error(artifact, "AddLL: slot b out of bounds"))?;
                    vm.stack.push(apply_numeric_binop_nan(va, vb, |x, y| x + y, |x, y| x + y, "add", artifact, &vm.frames)?);
                }
                x if x == Opcode::SubLL as u8 => {
                    let a = Self::read_u16(function, frame)? as usize;
                    let b = Self::read_u16(function, frame)? as usize;
                    let base = frame.base;
                    let va = vm.stack.get(base + a).cloned()
                        .ok_or_else(|| vm.error(artifact, "SubLL: slot a out of bounds"))?;
                    let vb = vm.stack.get(base + b).cloned()
                        .ok_or_else(|| vm.error(artifact, "SubLL: slot b out of bounds"))?;
                    vm.stack.push(apply_numeric_binop_nan(va, vb, |x, y| x - y, |x, y| x - y, "sub", artifact, &vm.frames)?);
                }
                x if x == Opcode::MulLL as u8 => {
                    let a = Self::read_u16(function, frame)? as usize;
                    let b = Self::read_u16(function, frame)? as usize;
                    let base = frame.base;
                    let va = vm.stack.get(base + a).cloned()
                        .ok_or_else(|| vm.error(artifact, "MulLL: slot a out of bounds"))?;
                    let vb = vm.stack.get(base + b).cloned()
                        .ok_or_else(|| vm.error(artifact, "MulLL: slot b out of bounds"))?;
                    vm.stack.push(apply_numeric_binop_nan(va, vb, |x, y| x * y, |x, y| x * y, "mul", artifact, &vm.frames)?);
                }
                x if x == Opcode::AddLC as u8 => {
                    let a = Self::read_u16(function, frame)? as usize;
                    let k_idx = Self::read_u16(function, frame)? as usize;
                    let base = frame.base;
                    let va = vm.stack.get(base + a).cloned()
                        .ok_or_else(|| vm.error(artifact, "AddLC: slot out of bounds"))?;
                    let vk = function.constants.get(k_idx).cloned()
                        .map(constant_to_nan)
                        .ok_or_else(|| vm.error(artifact, "AddLC: constant out of bounds"))?;
                    vm.stack.push(apply_numeric_binop_nan(va, vk, |x, y| x + y, |x, y| x + y, "add", artifact, &vm.frames)?);
                }
                x if x == Opcode::SubLC as u8 => {
                    let a = Self::read_u16(function, frame)? as usize;
                    let k_idx = Self::read_u16(function, frame)? as usize;
                    let base = frame.base;
                    let va = vm.stack.get(base + a).cloned()
                        .ok_or_else(|| vm.error(artifact, "SubLC: slot out of bounds"))?;
                    let vk = function.constants.get(k_idx).cloned()
                        .map(constant_to_nan)
                        .ok_or_else(|| vm.error(artifact, "SubLC: constant out of bounds"))?;
                    vm.stack.push(apply_numeric_binop_nan(va, vk, |x, y| x - y, |x, y| x - y, "sub", artifact, &vm.frames)?);
                }
                x if x == Opcode::LeLC as u8 => {
                    let a = Self::read_u16(function, frame)? as usize;
                    let k_idx = Self::read_u16(function, frame)? as usize;
                    let base = frame.base;
                    let va = vm.stack.get(base + a).cloned()
                        .ok_or_else(|| vm.error(artifact, "LeLC: slot out of bounds"))?;
                    let vk = function.constants.get(k_idx).cloned()
                        .map(constant_to_nan)
                        .ok_or_else(|| vm.error(artifact, "LeLC: constant out of bounds"))?;
                    vm.stack.push(compare_pair_nan((va, vk), |a, b| a <= b, artifact, &vm.frames)?);
                }
                x if x == Opcode::LtLC as u8 => {
                    let a = Self::read_u16(function, frame)? as usize;
                    let k_idx = Self::read_u16(function, frame)? as usize;
                    let base = frame.base;
                    let va = vm.stack.get(base + a).cloned()
                        .ok_or_else(|| vm.error(artifact, "LtLC: slot out of bounds"))?;
                    let vk = function.constants.get(k_idx).cloned()
                        .map(constant_to_nan)
                        .ok_or_else(|| vm.error(artifact, "LtLC: constant out of bounds"))?;
                    vm.stack.push(compare_pair_nan((va, vk), |a, b| a < b, artifact, &vm.frames)?);
                }
                x if x == Opcode::EqLC as u8 => {
                    let a = Self::read_u16(function, frame)? as usize;
                    let k_idx = Self::read_u16(function, frame)? as usize;
                    let base = frame.base;
                    let va = vm.stack.get(base + a).cloned()
                        .ok_or_else(|| vm.error(artifact, "EqLC: slot out of bounds"))?;
                    let vk = function.constants.get(k_idx).cloned()
                        .map(constant_to_nan)
                        .ok_or_else(|| vm.error(artifact, "EqLC: constant out of bounds"))?;
                    vm.stack.push(NanVal::from_bool(va == vk));
                }
                x if x == Opcode::GetFieldL as u8 => {
                    let a = Self::read_u16(function, frame)? as usize;
                    let f_idx = Self::read_u16(function, frame)? as usize;
                    let base = frame.base;
                    let field_name = artifact.str_table.get(f_idx).cloned()
                        .ok_or_else(|| vm.error(artifact, "GetFieldL: str_table index out of bounds"))?;
                    let value = vm.stack.get(base + a).cloned()
                        .ok_or_else(|| vm.error(artifact, "GetFieldL: local slot out of bounds"))?;
                    if let Some(map) = value.as_record() {
                        let v = map.get(&field_name).cloned()
                            .ok_or_else(|| vm.error(artifact, &format!("GetFieldL: missing field `{field_name}`")))?;
                        vm.stack.push(v);
                    } else if let Some(heap) = value.as_heap() {
                        let pushed = match heap {
                            HeapVal::Builtin(ns) => {
                                let full = format!("{}.{}", ns, field_name);
                                match full.as_str() {
                                    "Math.pi" => NanVal::from_float(std::f64::consts::PI),
                                    "Math.e"  => NanVal::from_float(std::f64::consts::E),
                                    _         => NanVal::from_heap(HeapVal::Builtin(full)),
                                }
                            }
                            HeapVal::VariantCtor(ns) => {
                                NanVal::from_heap(HeapVal::Builtin(format!("{}.{}", ns, field_name)))
                            }
                            _ => return Err(vm.error(artifact,
                                &format!("GetFieldL: expected Record/Builtin/VariantCtor, got {:?}", value))),
                        };
                        vm.stack.push(pushed);
                    } else {
                        return Err(vm.error(artifact,
                            &format!("GetFieldL: expected Record/Builtin/VariantCtor, got {:?}", value)));
                    }
                }
                x if x == Opcode::MoveLocal as u8 => {
                    let src = Self::read_u16(function, frame)? as usize;
                    let dst = Self::read_u16(function, frame)? as usize;
                    let base = frame.base;
                    let value = vm.stack.get(base + src).cloned()
                        .ok_or_else(|| vm.error(artifact, "MoveLocal: src slot out of bounds"))?;
                    let dst_idx = base + dst;
                    if dst_idx >= vm.stack.len() {
                        vm.stack.resize_with(dst_idx + 1, NanVal::unit);
                    }
                    vm.stack[dst_idx] = value;
                }
                x if x == Opcode::JumpIfNotVariant as u8 => {
                    let name_idx = Self::read_u16(function, frame)? as usize;
                    let offset = Self::read_u16(function, frame)? as usize;
                    let Some(expected) = artifact.str_table.get(name_idx).cloned() else {
                        return Err(vm.error(artifact, "variant name index out of bounds"));
                    };
                    let Some(value) = vm.stack.pop() else {
                        return Err(vm.error(artifact, "stack underflow on variant check"));
                    };
                    let matched = match value.as_heap() {
                        Some(HeapVal::Variant(tag, _)) if tag == &expected => true,
                        Some(HeapVal::VariantCtor(name)) if name == &expected => true,
                        _ => false,
                    };
                    if matched {
                        // Normalize VariantCtor to zero-arg Variant
                        let normalized = match value.as_heap() {
                            Some(HeapVal::VariantCtor(_)) => {
                                NanVal::from_heap(HeapVal::Variant(expected, None))
                            }
                            _ => value,
                        };
                        vm.stack.push(normalized);
                    } else {
                        vm.stack.push(value);
                        let Some(next_ip) = frame.ip.checked_add(offset) else {
                            return Err(vm.error(artifact, "jump overflow"));
                        };
                        frame.ip = next_ip;
                    }
                }
                x if x == Opcode::GetField as u8 => {
                    let idx = Self::read_u16(function, frame)? as usize;
                    let field_name =
                        match function.constants.get(idx) {
                            Some(crate::backend::codegen::Constant::Name(name)) => name.clone(),
                            _ => artifact.str_table.get(idx).cloned().ok_or_else(|| {
                                vm.error(artifact, "field name index out of bounds")
                            })?,
                        };
                    let value = vm
                        .stack
                        .pop()
                        .ok_or_else(|| vm.error(artifact, "stack underflow on get_field"))?;
                    if let Some(map) = value.as_record() {
                        let field = map.get(&field_name).cloned().ok_or_else(|| {
                            vm.error(artifact, &format!("missing record field `{field_name}`"))
                        })?;
                        vm.stack.push(field);
                    } else if let Some(heap) = value.as_heap() {
                        let pushed = match heap {
                            HeapVal::Builtin(ns) => {
                                let full = format!("{}.{}", ns, field_name);
                                match full.as_str() {
                                    "Math.pi" => NanVal::from_float(std::f64::consts::PI),
                                    "Math.e"  => NanVal::from_float(std::f64::consts::E),
                                    _         => NanVal::from_heap(HeapVal::Builtin(full)),
                                }
                            }
                            // TypeName.validate: treat user-defined type names as namespace (v6.6.0)
                            HeapVal::VariantCtor(ns) => {
                                NanVal::from_heap(HeapVal::Builtin(format!("{}.{}", ns, field_name)))
                            }
                            _ => return Err(vm.error(artifact, "get_field requires a record value")),
                        };
                        vm.stack.push(pushed);
                    } else {
                        return Err(vm.error(artifact, "get_field requires a record value"));
                    }
                }
                x if x == Opcode::BuildRecord as u8 => {
                    let field_count = Self::read_u16(function, frame)? as usize;
                    let names_idx = Self::read_u16(function, frame)? as usize;
                    let field_names: Vec<String> =
                        if let Some(crate::backend::codegen::Constant::Name(_)) =
                            function.constants.get(names_idx)
                        {
                            let mut names = Vec::with_capacity(field_count);
                            for i in 0..field_count {
                                let ci = names_idx + i;
                                match function.constants.get(ci) {
                                    Some(crate::backend::codegen::Constant::Name(name)) => {
                                        names.push(name.clone());
                                    }
                                    _ => {
                                        return Err(vm.error(
                                            artifact,
                                            &format!(
                                                "BuildRecord compat: constant[{ci}] is not a Name"
                                            ),
                                        ));
                                    }
                                }
                            }
                            names
                        } else {
                            let names =
                                artifact.str_table.get(names_idx).cloned().ok_or_else(|| {
                                    vm.error(artifact, "record field names index out of bounds")
                                })?;
                            if names.is_empty() {
                                Vec::new()
                            } else {
                                names.split('\u{1f}').map(|s| s.to_string()).collect()
                            }
                        };
                    if field_names.len() != field_count {
                        return Err(vm.error(artifact, "record field name count mismatch"));
                    }
                    let mut values = Vec::with_capacity(field_count);
                    for _ in 0..field_count {
                        values.push(vm.stack.pop().ok_or_else(|| {
                            vm.error(artifact, "stack underflow on build_record")
                        })?);
                    }
                    values.reverse();
                    let mut map: RecordMap = HashMap::with_capacity(field_count);
                    for (name, value) in field_names.into_iter().zip(values.into_iter()) {
                        map.insert(name, value);
                    }
                    vm.stack.push(NanVal::from_record(map));
                }
                x if x == Opcode::MakeClosure as u8 => {
                    let global_idx = Self::read_u16(function, frame)? as usize;
                    let capture_count = Self::read_u16(function, frame)? as usize;
                    let mut captures: Vec<NanVal> = Vec::with_capacity(capture_count);
                    for _ in 0..capture_count {
                        captures.push(vm.stack.pop().ok_or_else(|| {
                            vm.error(artifact, "stack underflow on make_closure")
                        })?);
                    }
                    captures.reverse();
                    let target = vm.globals.get(global_idx).cloned().ok_or_else(|| {
                        vm.error(artifact, "closure global index out of bounds")
                    })?;
                    match target.as_heap() {
                        Some(HeapVal::CompiledFn(fn_idx)) => {
                            vm.stack.push(NanVal::from_heap(HeapVal::Closure(*fn_idx, captures)));
                        }
                        _ => {
                            return Err(vm.error(
                                artifact,
                                "make_closure requires a function global target",
                            ));
                        }
                    }
                }
                x if x == Opcode::GetFieldC as u8 => {
                    let const_idx = Self::read_u16(function, frame)? as usize;
                    let field_name = match function.constants.get(const_idx) {
                        Some(crate::backend::codegen::Constant::Name(name)) => name.clone(),
                        _ => {
                            return Err(vm.error(
                                artifact,
                                &format!("GetFieldC: constant[{const_idx}] is not a Name"),
                            ));
                        }
                    };
                    let value = vm
                        .stack
                        .pop()
                        .ok_or_else(|| vm.error(artifact, "stack underflow on get_field_c"))?;
                    if let Some(map) = value.as_record() {
                        let field = map.get(&field_name).cloned().ok_or_else(|| {
                            vm.error(artifact, &format!("missing record field `{field_name}`"))
                        })?;
                        vm.stack.push(field);
                    } else if let Some(heap) = value.as_heap() {
                        let pushed = match heap {
                            HeapVal::Builtin(ns) => {
                                let full = format!("{}.{}", ns, field_name);
                                match full.as_str() {
                                    "Math.pi" => NanVal::from_float(std::f64::consts::PI),
                                    "Math.e"  => NanVal::from_float(std::f64::consts::E),
                                    _         => NanVal::from_heap(HeapVal::Builtin(full)),
                                }
                            }
                            HeapVal::VariantCtor(ns) => {
                                NanVal::from_heap(HeapVal::Builtin(format!("{}.{}", ns, field_name)))
                            }
                            _ => return Err(vm.error(artifact, "get_field_c requires a record or builtin value")),
                        };
                        vm.stack.push(pushed);
                    } else {
                        return Err(vm.error(artifact, "get_field_c requires a record or builtin value"));
                    }
                }
                x if x == Opcode::BuildRecordC as u8 => {
                    let n = Self::read_u16(function, frame)? as usize;
                    let base_const_idx = Self::read_u16(function, frame)? as usize;
                    let mut field_names = Vec::with_capacity(n);
                    for i in 0..n {
                        let ci = base_const_idx + i;
                        match function.constants.get(ci) {
                            Some(crate::backend::codegen::Constant::Name(name)) => {
                                field_names.push(name.clone());
                            }
                            _ => {
                                return Err(vm.error(
                                    artifact,
                                    &format!("BuildRecordC: constant[{ci}] is not a Name"),
                                ));
                            }
                        }
                    }
                    let mut values = Vec::with_capacity(n);
                    for _ in 0..n {
                        values.push(vm.stack.pop().ok_or_else(|| {
                            vm.error(artifact, "stack underflow on build_record_c")
                        })?);
                    }
                    values.reverse();
                    let mut map: RecordMap = HashMap::with_capacity(n);
                    for (name, value) in field_names.into_iter().zip(values.into_iter()) {
                        map.insert(name, value);
                    }
                    vm.stack.push(NanVal::from_record(map));
                }
                x if x == Opcode::MakeClosureN as u8 => {
                    let name_const_idx = Self::read_u16(function, frame)? as usize;
                    let capture_count = Self::read_u16(function, frame)? as usize;
                    let fn_name = match function.constants.get(name_const_idx) {
                        Some(crate::backend::codegen::Constant::Name(name)) => name.clone(),
                        _ => {
                            return Err(vm.error(
                                artifact,
                                &format!("MakeClosureN: constant[{name_const_idx}] is not a Name"),
                            ));
                        }
                    };
                    let mut captures: Vec<NanVal> = Vec::with_capacity(capture_count);
                    for _ in 0..capture_count {
                        captures.push(vm.stack.pop().ok_or_else(|| {
                            vm.error(artifact, "stack underflow on make_closure_n")
                        })?);
                    }
                    captures.reverse();
                    let fn_idx = artifact.fn_idx_by_name(&fn_name).ok_or_else(|| {
                        vm.error(
                            artifact,
                            &format!("MakeClosureN: function `{fn_name}` not found in globals"),
                        )
                    })?;
                    vm.stack.push(NanVal::from_heap(HeapVal::Closure(fn_idx, captures)));
                }
                x if x == Opcode::GetVariantPayload as u8 => {
                    let value = vm.stack.pop().ok_or_else(|| {
                        vm.error(artifact, "stack underflow on get_variant_payload")
                    })?;
                    match value.as_heap() {
                        Some(HeapVal::Variant(_, Some(payload))) => {
                            vm.stack.push(payload.clone());
                        }
                        Some(HeapVal::Variant(_, None)) => {
                            return Err(vm.error(artifact, "variant has no payload"));
                        }
                        _ => {
                            return Err(
                                vm.error(artifact, "get_variant_payload requires a variant")
                            );
                        }
                    }
                }
                x if x == Opcode::CollectBegin as u8 => {
                    vm.collect_frames.push(Vec::new());
                }
                x if x == Opcode::CollectEnd as u8 => {
                    let nan_values = vm
                        .collect_frames
                        .pop()
                        .ok_or_else(|| vm.error(artifact, "collect_end without collect_begin"))?;
                    let vm_values: Vec<VMValue> = nan_values.into_iter().map(|v| v.to_vmvalue()).collect();
                    vm.stack.push(NanVal::from_list(FavList::new(vm_values)));
                }
                x if x == Opcode::YieldValue as u8 => {
                    let Some(value) = vm.stack.pop() else {
                        return Err(vm.error(artifact, "stack underflow on yield"));
                    };
                    let Some(collect_frame) = vm.collect_frames.last_mut() else {
                        return Err(vm.error(artifact, "yield outside collect"));
                    };
                    collect_frame.push(value);
                }
                x if x == Opcode::EmitEvent as u8 => {
                    let Some(value) = vm.stack.pop() else {
                        return Err(vm.error(artifact, "stack underflow on emit"));
                    };
                    vm.emit_log.push(value);
                    vm.stack.push(NanVal::unit());
                }
                x if x == Opcode::Call as u8 => {
                    let arg_count = Self::read_u16(function, frame)? as usize;
                    let callee_pos = vm
                        .stack
                        .len()
                        .checked_sub(arg_count + 1)
                        .ok_or_else(|| vm.error(artifact, "stack underflow on call"))?;
                    let callee = vm.stack[callee_pos].clone();
                    let mut args: Vec<NanVal> = Vec::with_capacity(arg_count);
                    for _ in 0..arg_count {
                        args.push(
                            vm.stack
                                .pop()
                                .ok_or_else(|| vm.error(artifact, "stack underflow on call"))?,
                        );
                    }
                    args.reverse();
                    vm.stack.remove(callee_pos);
                    // Iterative dispatch: push frame for compiled fns/closures instead of
                    // calling invoke_function recursively (avoids Rust stack overflow on
                    // deeply recursive Favnir programs).
                    match callee.as_heap() {
                        Some(HeapVal::CompiledFn(fn_idx)) => {
                            let fn_idx = *fn_idx;
                            vm.push_compiled_frame(artifact, fn_idx, args)?;
                            vm.try_apply_tco(artifact);
                        }
                        Some(HeapVal::Closure(fn_idx, captures)) => {
                            let fn_idx = *fn_idx;
                            let mut full_args: Vec<NanVal> = captures.clone();
                            full_args.extend(args);
                            vm.push_compiled_frame(artifact, fn_idx, full_args)?;
                            vm.try_apply_tco(artifact);
                        }
                        Some(HeapVal::VariantCtor(name)) => {
                            let name = name.clone();
                            let result = match args.len() {
                                0 => NanVal::from_heap(HeapVal::Variant(name, None)),
                                1 => NanVal::from_heap(HeapVal::Variant(name, Some(args.into_iter().next().unwrap()))),
                                _ => {
                                    let map: RecordMap = args.into_iter().enumerate()
                                        .map(|(i, v)| (format!("_{i}"), v))
                                        .collect();
                                    NanVal::from_heap(HeapVal::Variant(name, Some(NanVal::from_record(map))))
                                }
                            };
                            vm.stack.push(result);
                        }
                        Some(HeapVal::Builtin(name)) => {
                            let name = name.clone();
                            let args_vm: Vec<VMValue> = args.into_iter().map(|v| v.to_vmvalue()).collect();
                            let result = vm.call_builtin(artifact, &name, args_vm)?;
                            vm.stack.push(NanVal::from_vmvalue(result));
                        }
                        _ => {
                            // Bridge for any other heap value
                            let callee_vm = callee.to_vmvalue();
                            let args_vm: Vec<VMValue> = args.into_iter().map(|v| v.to_vmvalue()).collect();
                            let result = vm.call_value(artifact, callee_vm, args_vm)?;
                            vm.stack.push(NanVal::from_vmvalue(result));
                        }
                    }
                }
                x if x == Opcode::CallNamed as u8 => {
                    // Self-hosted compiler output: call function by name stored in constants pool.
                    let name_const_idx = Self::read_u16(function, frame)? as usize;
                    let arg_count = Self::read_u16(function, frame)? as usize;
                    let fn_name = match function.constants.get(name_const_idx) {
                        Some(crate::backend::codegen::Constant::Name(name)) => name.clone(),
                        _ => {
                            return Err(vm.error(
                                artifact,
                                &format!("CallNamed: constant[{name_const_idx}] is not a Name"),
                            ));
                        }
                    };
                    let mut args: Vec<NanVal> = Vec::with_capacity(arg_count);
                    for _ in 0..arg_count {
                        args.push(
                            vm.stack.pop().ok_or_else(|| {
                                vm.error(artifact, "stack underflow on CallNamed")
                            })?,
                        );
                    }
                    args.reverse();
                    // Resolve and dispatch iteratively to avoid Rust recursion.
                    if let Some(fn_idx) = artifact.fn_idx_by_name(&fn_name) {
                        // User-defined function: push frame directly (+ TCO if tail call)
                        vm.push_compiled_frame(artifact, fn_idx, args)?;
                        vm.try_apply_tco(artifact);
                    } else if is_known_builtin_namespace(&fn_name) {
                        // Builtin: handle Result/Option monadic combinators inline to
                        // avoid recursive resume calls on deeply-chained Result.and_then.
                        match fn_name.as_str() {
                            "Result.and_then" => {
                                let func = args.pop().expect("Result.and_then: missing func");
                                let result_val = args.pop().expect("Result.and_then: missing result");
                                match result_val.as_heap() {
                                    Some(HeapVal::Variant(tag, payload)) if tag == "Ok" => {
                                        let inner = payload.as_ref().map(|v| v.clone()).unwrap_or_else(NanVal::unit);
                                        match func.as_heap() {
                                            Some(HeapVal::CompiledFn(fn_idx)) => {
                                                let fn_idx = *fn_idx;
                                                vm.push_compiled_frame(artifact, fn_idx, vec![inner])?;
                                                vm.try_apply_tco(artifact);
                                            }
                                            Some(HeapVal::Closure(fn_idx, captures)) => {
                                                let fn_idx = *fn_idx;
                                                let mut full_args: Vec<NanVal> = captures.clone();
                                                full_args.push(inner);
                                                vm.push_compiled_frame(artifact, fn_idx, full_args)?;
                                                vm.try_apply_tco(artifact);
                                            }
                                            _ => {
                                                let callee_vm = func.to_vmvalue();
                                                let r = vm.call_value(artifact, callee_vm, vec![inner.to_vmvalue()])?;
                                                vm.stack.push(NanVal::from_vmvalue(r));
                                            }
                                        }
                                    }
                                    Some(HeapVal::Variant(tag, _)) if tag == "Err" => {
                                        vm.stack.push(result_val);
                                    }
                                    _ => {
                                        return Err(vm.error(artifact, "Result.and_then: expected a Result value"));
                                    }
                                }
                            }
                            "Option.and_then" => {
                                let func = args.pop().expect("Option.and_then: missing func");
                                let opt_val = args.pop().expect("Option.and_then: missing option");
                                match opt_val.as_heap() {
                                    Some(HeapVal::Variant(tag, payload)) if tag == "Some" => {
                                        let inner = payload.as_ref().map(|v| v.clone()).unwrap_or_else(NanVal::unit);
                                        match func.as_heap() {
                                            Some(HeapVal::CompiledFn(fn_idx)) => {
                                                let fn_idx = *fn_idx;
                                                vm.push_compiled_frame(artifact, fn_idx, vec![inner])?;
                                                vm.try_apply_tco(artifact);
                                            }
                                            Some(HeapVal::Closure(fn_idx, captures)) => {
                                                let fn_idx = *fn_idx;
                                                let mut full_args: Vec<NanVal> = captures.clone();
                                                full_args.push(inner);
                                                vm.push_compiled_frame(artifact, fn_idx, full_args)?;
                                                vm.try_apply_tco(artifact);
                                            }
                                            _ => {
                                                let callee_vm = func.to_vmvalue();
                                                let r = vm.call_value(artifact, callee_vm, vec![inner.to_vmvalue()])?;
                                                vm.stack.push(NanVal::from_vmvalue(r));
                                            }
                                        }
                                    }
                                    Some(HeapVal::Variant(tag, _)) if tag == "None" => {
                                        vm.stack.push(NanVal::from_heap(HeapVal::Variant("None".to_string(), None)));
                                    }
                                    _ => {
                                        return Err(vm.error(artifact, "Option.and_then: expected an Option value"));
                                    }
                                }
                            }
                            _ => {
                                let args_vm: Vec<VMValue> = args.into_iter().map(|v| v.to_vmvalue()).collect();
                                let result = vm.call_builtin(artifact, &fn_name, args_vm)?;
                                vm.stack.push(NanVal::from_vmvalue(result));
                            }
                        }
                    } else if looks_like_variant_ctor(&fn_name) {
                        let result = match args.len() {
                            0 => NanVal::from_heap(HeapVal::Variant(fn_name, None)),
                            1 => NanVal::from_heap(HeapVal::Variant(fn_name, Some(args.into_iter().next().unwrap()))),
                            _ => {
                                let map: RecordMap = args.into_iter().enumerate()
                                    .map(|(i, v)| (format!("_{i}"), v))
                                    .collect();
                                NanVal::from_heap(HeapVal::Variant(fn_name, Some(NanVal::from_record(map))))
                            }
                        };
                        vm.stack.push(result);
                    } else {
                        return Err(vm.error(
                            artifact,
                            &format!("unknown global or builtin: {fn_name}"),
                        ));
                    }
                }
                x if x == Opcode::JumpIfNotVariantC as u8 => {
                    // Self-hosted compiler match codegen: variant name in per-function constants.
                    let const_idx = Self::read_u16(function, frame)? as usize;
                    let offset = Self::read_u16(function, frame)? as usize;
                    let expected = match function.constants.get(const_idx) {
                        Some(crate::backend::codegen::Constant::Name(name)) => name.clone(),
                        _ => {
                            return Err(vm.error(
                                artifact,
                                &format!("JumpIfNotVariantC: constant[{const_idx}] is not a Name"),
                            ));
                        }
                    };
                    let Some(value) = vm.stack.pop() else {
                        return Err(vm.error(artifact, "stack underflow on JumpIfNotVariantC"));
                    };
                    let matched = match value.as_heap() {
                        Some(HeapVal::Variant(tag, _)) if tag == &expected => true,
                        Some(HeapVal::VariantCtor(name)) if name == &expected => true,
                        _ => false,
                    };
                    if matched {
                        let normalized = match value.as_heap() {
                            Some(HeapVal::VariantCtor(_)) => {
                                NanVal::from_heap(HeapVal::Variant(expected, None))
                            }
                            _ => value,
                        };
                        vm.stack.push(normalized);
                    } else {
                        vm.stack.push(value);
                        let Some(next_ip) = frame.ip.checked_add(offset) else {
                            return Err(vm.error(artifact, "JumpIfNotVariantC jump overflow"));
                        };
                        frame.ip = next_ip;
                    }
                }
                x if x == Opcode::Add as u8 => {
                    let (left, right) = vm.pop_pair(artifact)?;
                    vm.stack.push(apply_numeric_binop_nan(
                        left, right, |a, b| a + b, |a, b| a + b, "add", artifact, &vm.frames,
                    )?);
                }
                x if x == Opcode::Sub as u8 => {
                    let (left, right) = vm.pop_pair(artifact)?;
                    vm.stack.push(apply_numeric_binop_nan(
                        left, right, |a, b| a - b, |a, b| a - b, "sub", artifact, &vm.frames,
                    )?);
                }
                x if x == Opcode::Mul as u8 => {
                    let (left, right) = vm.pop_pair(artifact)?;
                    vm.stack.push(apply_numeric_binop_nan(
                        left, right, |a, b| a * b, |a, b| a * b, "mul", artifact, &vm.frames,
                    )?);
                }
                x if x == Opcode::Div as u8 => {
                    let (left, right) = vm.pop_pair(artifact)?;
                    let division_by_zero = match (left.as_int(), left.as_float(), right.as_int(), right.as_float()) {
                        (Some(_), _, Some(0), _) => true,
                        (_, Some(_), _, Some(v)) => v == 0.0,
                        (Some(_), _, _, Some(v)) => v == 0.0,
                        (_, Some(_), Some(0), _) => true,
                        _ => false,
                    };
                    if division_by_zero {
                        return Err(vm_error_from_frames(
                            artifact, &vm.frames, "division by zero".to_string(),
                        ));
                    }
                    vm.stack.push(apply_numeric_binop_nan(
                        left, right, |a, b| a / b, |a, b| a / b, "div", artifact, &vm.frames,
                    )?);
                }
                x if x == Opcode::And as u8 => {
                    let (left, right) = vm.pop_pair(artifact)?;
                    match (left.as_bool(), right.as_bool()) {
                        (Some(a), Some(b)) => vm.stack.push(NanVal::from_bool(a && b)),
                        _ => return Err(vm.error(artifact, "logical and requires Bool operands")),
                    }
                }
                x if x == Opcode::Or as u8 => {
                    let (left, right) = vm.pop_pair(artifact)?;
                    match (left.as_bool(), right.as_bool()) {
                        (Some(a), Some(b)) => vm.stack.push(NanVal::from_bool(a || b)),
                        _ => return Err(vm.error(artifact, "logical or requires Bool operands")),
                    }
                }
                x if x == Opcode::Eq as u8 => {
                    let (left, right) = vm.pop_pair(artifact)?;
                    vm.stack.push(NanVal::from_bool(left == right));
                }
                x if x == Opcode::Ne as u8 => {
                    let (left, right) = vm.pop_pair(artifact)?;
                    vm.stack.push(NanVal::from_bool(left != right));
                }
                x if x == Opcode::Lt as u8 => {
                    let pair = vm.pop_pair(artifact)?;
                    vm.stack.push(compare_pair_nan(pair, |a, b| a < b, artifact, &vm.frames)?);
                }
                x if x == Opcode::Le as u8 => {
                    let pair = vm.pop_pair(artifact)?;
                    vm.stack.push(compare_pair_nan(pair, |a, b| a <= b, artifact, &vm.frames)?);
                }
                x if x == Opcode::Gt as u8 => {
                    let pair = vm.pop_pair(artifact)?;
                    vm.stack.push(compare_pair_nan(pair, |a, b| a > b, artifact, &vm.frames)?);
                }
                x if x == Opcode::Ge as u8 => {
                    let pair = vm.pop_pair(artifact)?;
                    vm.stack.push(compare_pair_nan(pair, |a, b| a >= b, artifact, &vm.frames)?);
                }
                x if x == Opcode::Return as u8 => {
                    let ret = vm.stack.pop().unwrap_or_else(NanVal::unit);
                    let frame = vm.frames.pop().expect("frame exists");
                    vm.stack.truncate(frame.base);
                    if vm.frames.len() == caller_depth {
                        return Ok(ret);
                    }
                    vm.stack.push(ret);
                }
                x if x == Opcode::TrackLine as u8 => {
                    if frame.ip + 3 >= function.code.len() {
                        return Err(vm.error(artifact, "TrackLine: unexpected end of bytecode"));
                    }
                    let b0 = function.code[frame.ip];
                    let b1 = function.code[frame.ip + 1];
                    let b2 = function.code[frame.ip + 2];
                    let b3 = function.code[frame.ip + 3];
                    frame.ip += 4;
                    let line = u32::from_le_bytes([b0, b1, b2, b3]);
                    frame.line = line;
                    COVERED_LINES.with(|c| {
                        if let Some(set) = c.borrow_mut().as_mut() {
                            set.insert(line);
                        }
                    });
                }
                x if x == Opcode::Swap as u8 => {
                    let a = vm
                        .stack
                        .pop()
                        .ok_or_else(|| vm.error(artifact, "stack underflow on swap"))?;
                    let b = vm
                        .stack
                        .pop()
                        .ok_or_else(|| vm.error(artifact, "stack underflow on swap"))?;
                    vm.stack.push(a);
                    vm.stack.push(b);
                }
                other => {
                    return Err(vm.error(artifact, &format!("unsupported opcode: 0x{other:02x}")));
                }
            }
        }
    }

    fn read_u16(
        function: &crate::backend::artifact::FvcFunction,
        frame: &mut CallFrame,
    ) -> Result<u16, VMError> {
        if frame.ip + 1 >= function.code.len() {
            return Err(VMError {
                message: "unexpected end of bytecode".to_string(),
                fn_name: "<decode>".to_string(),
                ip: frame.ip,
                stack_trace: vec![],
            });
        }
        let lo = function.code[frame.ip];
        let hi = function.code[frame.ip + 1];
        frame.ip += 2;
        Ok(u16::from_le_bytes([lo, hi]))
    }

    fn error(&self, artifact: &FvcArtifact, message: &str) -> VMError {
        if let Some(frame) = self.frames.last() {
            let function = &artifact.functions[frame.fn_idx];
            let fn_name = artifact
                .str_table
                .get(function.name_idx as usize)
                .cloned()
                .unwrap_or_else(|| "<unknown>".to_string());
            VMError {
                message: message.to_string(),
                fn_name,
                ip: frame.ip,
                stack_trace: build_stack_trace(artifact, &self.frames),
            }
        } else {
            VMError {
                message: message.to_string(),
                fn_name: "<none>".to_string(),
                ip: 0,
                stack_trace: vec![],
            }
        }
    }

    fn call_value(
        &mut self,
        artifact: &FvcArtifact,
        callee: VMValue,
        args: Vec<VMValue>,
    ) -> Result<VMValue, VMError> {
        match callee {
            VMValue::CompiledFn(target_idx) => self.invoke_function(artifact, target_idx, args),
            VMValue::Closure(target_idx, captures) => {
                let mut full_args = captures;
                full_args.extend(args);
                self.invoke_function(artifact, target_idx, full_args)
            }
            VMValue::VariantCtor(name) => {
                let payload = match args.len() {
                    0 => None,
                    1 => Some(Box::new(args.into_iter().next().expect("single payload"))),
                    _ => {
                        // Multi-arg tuple variant: wrap args into a positional record
                        let map: std::collections::HashMap<String, VMValue> = args
                            .into_iter()
                            .enumerate()
                            .map(|(i, v)| (format!("_{}", i), v))
                            .collect();
                        Some(Box::new(VMValue::Record(map)))
                    }
                };
                Ok(VMValue::Variant(name, payload))
            }
            VMValue::Builtin(name) => self.call_builtin(artifact, &name, args),
            _ => Err(self.error(artifact, "attempted to call a non-function value")),
        }
    }

    fn call_builtin(
        &mut self,
        artifact: &FvcArtifact,
        name: &str,
        args: Vec<VMValue>,
    ) -> Result<VMValue, VMError> {
        match name {
            "List.map" => {
                if args.len() != 2 {
                    return Err(self.error(artifact, "List.map requires 2 arguments"));
                }
                let mut it = args.into_iter();
                let list = it.next().expect("list");
                let func = it.next().expect("func");
                match list {
                    VMValue::List(fl) => {
                        let mut out = Vec::with_capacity(fl.len());
                        for x in fl {
                            out.push(self.call_value(artifact, func.clone(), vec![x])?);
                        }
                        Ok(VMValue::List(FavList::new(out)))
                    }
                    _ => Err(self.error(artifact, "List.map requires a List as first argument")),
                }
            }
            "List.filter" => {
                if args.len() != 2 {
                    return Err(self.error(artifact, "List.filter requires 2 arguments"));
                }
                let mut it = args.into_iter();
                let list = it.next().expect("list");
                let func = it.next().expect("func");
                match list {
                    VMValue::List(fl) => {
                        let mut out = Vec::new();
                        for x in fl {
                            let keep = self.call_value(artifact, func.clone(), vec![x.clone()])?;
                            match keep {
                                VMValue::Bool(true) => out.push(x),
                                VMValue::Bool(false) => {}
                                other => {
                                    return Err(self.error(
                                        artifact,
                                        &format!(
                                            "List.filter predicate must return Bool, got {}",
                                            vmvalue_type_name(&other)
                                        ),
                                    ));
                                }
                            }
                        }
                        Ok(VMValue::List(FavList::new(out)))
                    }
                    _ => Err(self.error(artifact, "List.filter requires a List as first argument")),
                }
            }
            "List.take_while" => {
                if args.len() != 2 {
                    return Err(self.error(artifact, "List.take_while requires 2 arguments"));
                }
                let mut it = args.into_iter();
                let list = it.next().expect("list");
                let func = it.next().expect("func");
                match list {
                    VMValue::List(fl) => {
                        let mut out = Vec::new();
                        for x in fl {
                            match self.call_value(artifact, func.clone(), vec![x.clone()])? {
                                VMValue::Bool(true) => out.push(x),
                                VMValue::Bool(false) => break,
                                other => {
                                    return Err(self.error(
                                        artifact,
                                        &format!(
                                            "List.take_while predicate must return Bool, got {}",
                                            vmvalue_type_name(&other)
                                        ),
                                    ));
                                }
                            }
                        }
                        Ok(VMValue::List(FavList::new(out)))
                    }
                    _ => Err(self.error(
                        artifact,
                        "List.take_while requires a List as first argument",
                    )),
                }
            }
            "List.drop_while" => {
                if args.len() != 2 {
                    return Err(self.error(artifact, "List.drop_while requires 2 arguments"));
                }
                let mut it = args.into_iter();
                let list = it.next().expect("list");
                let func = it.next().expect("func");
                match list {
                    VMValue::List(fl) => {
                        let mut rest = fl.into_iter().peekable();
                        while let Some(x) = rest.peek() {
                            match self.call_value(artifact, func.clone(), vec![x.clone()])? {
                                VMValue::Bool(true) => {
                                    rest.next();
                                }
                                _ => break,
                            }
                        }
                        Ok(VMValue::List(FavList::new(rest.collect())))
                    }
                    _ => Err(self.error(
                        artifact,
                        "List.drop_while requires a List as first argument",
                    )),
                }
            }
            "List.fold" => {
                if args.len() != 3 {
                    return Err(self.error(artifact, "List.fold requires 3 arguments"));
                }
                let mut it = args.into_iter();
                let list = it.next().expect("list");
                let mut acc = it.next().expect("init");
                let func = it.next().expect("func");
                match list {
                    VMValue::List(fl) => {
                        for x in fl {
                            acc = self.call_value(artifact, func.clone(), vec![acc, x])?;
                        }
                        Ok(acc)
                    }
                    _ => Err(self.error(artifact, "List.fold requires a List as first argument")),
                }
            }
            "List.scan" => {
                if args.len() != 3 {
                    return Err(self.error(artifact, "List.scan requires 3 arguments"));
                }
                let mut it = args.into_iter();
                let list = it.next().expect("list");
                let mut acc = it.next().expect("init");
                let func = it.next().expect("func");
                match list {
                    VMValue::List(fl) => {
                        let mut result = vec![acc.clone()];
                        for x in fl {
                            acc = self.call_value(artifact, func.clone(), vec![acc, x])?;
                            result.push(acc.clone());
                        }
                        Ok(VMValue::List(FavList::new(result)))
                    }
                    _ => Err(self.error(artifact, "List.scan requires a List as first argument")),
                }
            }
            "List.flat_map" => {
                if args.len() != 2 {
                    return Err(self.error(artifact, "List.flat_map requires 2 arguments"));
                }
                let mut it = args.into_iter();
                let list = it.next().expect("list");
                let func = it.next().expect("func");
                match list {
                    VMValue::List(fl) => {
                        let mut out: Vec<VMValue> = Vec::new();
                        for x in fl {
                            match self.call_value(artifact, func.clone(), vec![x])? {
                                VMValue::List(inner) => out.extend(inner),
                                other => {
                                    return Err(self.error(
                                        artifact,
                                        &format!(
                                            "List.flat_map: callback must return List, got {}",
                                            vmvalue_type_name(&other)
                                        ),
                                    ));
                                }
                            }
                        }
                        Ok(VMValue::List(FavList::new(out)))
                    }
                    _ => {
                        Err(self.error(artifact, "List.flat_map requires a List as first argument"))
                    }
                }
            }
            // List.collect_result(list, fn) → Result<List<T>, E>  (v17.3.0)
            "List.collect_result" => {
                if args.len() != 2 {
                    return Err(self.error(artifact, "List.collect_result requires 2 arguments"));
                }
                let mut it = args.into_iter();
                let list = it.next().expect("list");
                let func = it.next().expect("func");
                match list {
                    VMValue::List(fl) => {
                        let mut collected: Vec<VMValue> = Vec::new();
                        for x in fl {
                            let res = self.call_value(artifact, func.clone(), vec![x])?;
                            match res {
                                VMValue::Variant(ref tag, ref payload) if tag == "ok" => {
                                    collected.push(*payload.clone().expect("ok payload"));
                                }
                                VMValue::Variant(ref tag, _) if tag == "err" => {
                                    return Ok(res);
                                }
                                other => {
                                    return Err(self.error(
                                        artifact,
                                        &format!(
                                            "List.collect_result: callback must return Result, got {}",
                                            vmvalue_type_name(&other)
                                        ),
                                    ));
                                }
                            }
                        }
                        Ok(VMValue::Variant(
                            "ok".to_string(),
                            Some(Box::new(VMValue::List(FavList::new(collected)))),
                        ))
                    }
                    _ => Err(self.error(artifact, "List.collect_result requires a List as first argument")),
                }
            }
            "List.sort" => {
                if args.len() != 2 {
                    return Err(self.error(artifact, "List.sort requires 2 arguments"));
                }
                let mut it = args.into_iter();
                let list = it.next().expect("list");
                let cmp = it.next().expect("cmp");
                match list {
                    VMValue::List(fl) => {
                        let mut xs = fl.to_vec();
                        let mut sort_err: Option<VMError> = None;
                        xs.sort_by(|a, b| {
                            if sort_err.is_some() {
                                return std::cmp::Ordering::Equal;
                            }
                            match self.call_value(artifact, cmp.clone(), vec![a.clone(), b.clone()])
                            {
                                Ok(VMValue::Int(n)) => {
                                    if n < 0 {
                                        std::cmp::Ordering::Less
                                    } else if n > 0 {
                                        std::cmp::Ordering::Greater
                                    } else {
                                        std::cmp::Ordering::Equal
                                    }
                                }
                                Ok(other) => {
                                    sort_err = Some(self.error(
                                        artifact,
                                        &format!(
                                            "List.sort: comparator must return Int, got {}",
                                            vmvalue_type_name(&other)
                                        ),
                                    ));
                                    std::cmp::Ordering::Equal
                                }
                                Err(e) => {
                                    sort_err = Some(e);
                                    std::cmp::Ordering::Equal
                                }
                            }
                        });
                        if let Some(e) = sort_err {
                            return Err(e);
                        }
                        Ok(VMValue::List(FavList::new(xs)))
                    }
                    _ => Err(self.error(artifact, "List.sort requires a List as first argument")),
                }
            }
            "List.find" => {
                if args.len() != 2 {
                    return Err(self.error(artifact, "List.find requires 2 arguments"));
                }
                let mut it = args.into_iter();
                let list = it.next().expect("list");
                let pred = it.next().expect("pred");
                match list {
                    VMValue::List(fl) => {
                        for x in fl {
                            match self.call_value(artifact, pred.clone(), vec![x.clone()])? {
                                VMValue::Bool(true) => {
                                    return Ok(VMValue::Variant("some".into(), Some(Box::new(x))));
                                }
                                VMValue::Bool(false) => {}
                                other => {
                                    return Err(self.error(
                                        artifact,
                                        &format!(
                                            "List.find predicate must return Bool, got {}",
                                            vmvalue_type_name(&other)
                                        ),
                                    ));
                                }
                            }
                        }
                        Ok(VMValue::Variant("none".into(), None))
                    }
                    _ => Err(self.error(artifact, "List.find requires a List as first argument")),
                }
            }
            "List.any" => {
                if args.len() != 2 {
                    return Err(self.error(artifact, "List.any requires 2 arguments"));
                }
                let mut it = args.into_iter();
                let list = it.next().expect("list");
                let pred = it.next().expect("pred");
                match list {
                    VMValue::List(fl) => {
                        for x in fl {
                            match self.call_value(artifact, pred.clone(), vec![x])? {
                                VMValue::Bool(true) => return Ok(VMValue::Bool(true)),
                                VMValue::Bool(false) => {}
                                other => {
                                    return Err(self.error(
                                        artifact,
                                        &format!(
                                            "List.any predicate must return Bool, got {}",
                                            vmvalue_type_name(&other)
                                        ),
                                    ));
                                }
                            }
                        }
                        Ok(VMValue::Bool(false))
                    }
                    _ => Err(self.error(artifact, "List.any requires a List as first argument")),
                }
            }
            "List.contains" => {
                if args.len() != 2 {
                    return Err(self.error(artifact, "List.contains requires 2 arguments"));
                }
                let mut it = args.into_iter();
                let list = it.next().expect("list");
                let elem = it.next().expect("elem");
                match list {
                    VMValue::List(fl) => {
                        Ok(VMValue::Bool(fl.iter().any(|x| x == &elem)))
                    }
                    _ => Err(self.error(artifact, "List.contains requires a List as first argument")),
                }
            }
            "List.all" => {
                if args.len() != 2 {
                    return Err(self.error(artifact, "List.all requires 2 arguments"));
                }
                let mut it = args.into_iter();
                let list = it.next().expect("list");
                let pred = it.next().expect("pred");
                match list {
                    VMValue::List(fl) => {
                        for x in fl {
                            match self.call_value(artifact, pred.clone(), vec![x])? {
                                VMValue::Bool(false) => return Ok(VMValue::Bool(false)),
                                VMValue::Bool(true) => {}
                                other => {
                                    return Err(self.error(
                                        artifact,
                                        &format!(
                                            "List.all predicate must return Bool, got {}",
                                            vmvalue_type_name(&other)
                                        ),
                                    ));
                                }
                            }
                        }
                        Ok(VMValue::Bool(true))
                    }
                    _ => Err(self.error(artifact, "List.all requires a List as first argument")),
                }
            }
            "List.count" => {
                if args.len() != 2 {
                    return Err(self.error(artifact, "List.count requires 2 arguments"));
                }
                let mut it = args.into_iter();
                let list = it.next().expect("list");
                let pred = it.next().expect("pred");
                match list {
                    VMValue::List(fl) => {
                        let mut count = 0i64;
                        for x in fl {
                            match self.call_value(artifact, pred.clone(), vec![x])? {
                                VMValue::Bool(true) => count += 1,
                                VMValue::Bool(false) => {}
                                other => {
                                    return Err(self.error(
                                        artifact,
                                        &format!(
                                            "List.count predicate must return Bool, got {}",
                                            vmvalue_type_name(&other)
                                        ),
                                    ));
                                }
                            }
                        }
                        Ok(VMValue::Int(count))
                    }
                    _ => Err(self.error(artifact, "List.count requires a List as first argument")),
                }
            }
            "List.partition" => {
                if args.len() != 2 {
                    return Err(self.error(artifact, "List.partition requires 2 arguments"));
                }
                let mut it = args.into_iter();
                let list = it.next().expect("list");
                let pred = it.next().expect("pred");
                match list {
                    VMValue::List(fl) => {
                        let mut matching = Vec::new();
                        let mut non_matching = Vec::new();
                        for x in fl {
                            match self.call_value(artifact, pred.clone(), vec![x.clone()])? {
                                VMValue::Bool(true) => matching.push(x),
                                VMValue::Bool(false) => non_matching.push(x),
                                other => {
                                    return Err(self.error(
                                        artifact,
                                        &format!(
                                            "List.partition predicate must return Bool, got {}",
                                            vmvalue_type_name(&other)
                                        ),
                                    ));
                                }
                            }
                        }
                        Ok(VMValue::List(FavList::new(vec![
                            VMValue::List(FavList::new(matching)),
                            VMValue::List(FavList::new(non_matching)),
                        ])))
                    }
                    _ => Err(self.error(artifact, "List.partition requires a List as first argument")),
                }
            }
            "List.index_of" => {
                if args.len() != 2 {
                    return Err(self.error(artifact, "List.index_of requires 2 arguments"));
                }
                let mut it = args.into_iter();
                let list = it.next().expect("list");
                let pred = it.next().expect("pred");
                match list {
                    VMValue::List(fl) => {
                        for (i, x) in fl.into_iter().enumerate() {
                            match self.call_value(artifact, pred.clone(), vec![x])? {
                                VMValue::Bool(true) => {
                                    return Ok(VMValue::Variant(
                                        "some".into(),
                                        Some(Box::new(VMValue::Int(i as i64))),
                                    ));
                                }
                                VMValue::Bool(false) => {}
                                other => {
                                    return Err(self.error(
                                        artifact,
                                        &format!(
                                            "List.index_of predicate must return Bool, got {}",
                                            vmvalue_type_name(&other)
                                        ),
                                    ));
                                }
                            }
                        }
                        Ok(VMValue::Variant("none".into(), None))
                    }
                    _ => {
                        Err(self.error(artifact, "List.index_of requires a List as first argument"))
                    }
                }
            }
            "Map.map_values" => {
                if args.len() != 2 {
                    return Err(self.error(artifact, "Map.map_values requires 2 arguments"));
                }
                let mut it = args.into_iter();
                let map = it.next().expect("map");
                let func = it.next().expect("func");
                match map {
                    VMValue::Record(m) => {
                        let mut out = HashMap::with_capacity(m.len());
                        for (k, v) in m {
                            let mapped = self.call_value(artifact, func.clone(), vec![v])?;
                            out.insert(k, mapped);
                        }
                        Ok(VMValue::Record(out))
                    }
                    other => Err(self.error(
                        artifact,
                        &format!(
                            "Map.map_values requires a Map as first argument, got {}",
                            vmvalue_type_name(&other)
                        ),
                    )),
                }
            }
            "Map.filter_values" => {
                if args.len() != 2 {
                    return Err(self.error(artifact, "Map.filter_values requires 2 arguments"));
                }
                let mut it = args.into_iter();
                let map = it.next().expect("map");
                let func = it.next().expect("func");
                match map {
                    VMValue::Record(m) => {
                        let mut out = HashMap::new();
                        for (k, v) in m {
                            let keep = self.call_value(artifact, func.clone(), vec![v.clone()])?;
                            match keep {
                                VMValue::Bool(true) => {
                                    out.insert(k, v);
                                }
                                VMValue::Bool(false) => {}
                                other => {
                                    return Err(self.error(
                                        artifact,
                                        &format!(
                                            "Map.filter_values predicate must return Bool, got {}",
                                            vmvalue_type_name(&other)
                                        ),
                                    ));
                                }
                            }
                        }
                        Ok(VMValue::Record(out))
                    }
                    other => Err(self.error(
                        artifact,
                        &format!(
                            "Map.filter_values requires a Map as first argument, got {}",
                            vmvalue_type_name(&other)
                        ),
                    )),
                }
            }
            "List.zip_with" => {
                if args.len() != 3 {
                    return Err(self.error(artifact, "List.zip_with requires 3 arguments: (f, xs, ys)"));
                }
                let mut it = args.into_iter();
                let func = it.next().expect("func");
                let xs = it.next().expect("xs");
                let ys = it.next().expect("ys");
                match (xs, ys) {
                    (VMValue::List(fxs), VMValue::List(fys)) => {
                        let mut out = Vec::new();
                        for (x, y) in fxs.iter().zip(fys.iter()) {
                            // f is curried: |x| |y| body
                            let partial = self.call_value(artifact, func.clone(), vec![x.clone()])?;
                            let result = self.call_value(artifact, partial, vec![y.clone()])?;
                            out.push(result);
                        }
                        Ok(VMValue::List(FavList::new(out)))
                    }
                    _ => Err(self.error(artifact, "List.zip_with requires (f, List, List)")),
                }
            }
            "List.group_by" => {
                if args.len() != 2 {
                    return Err(self.error(artifact, "List.group_by requires 2 arguments: (f, xs)"));
                }
                let mut it = args.into_iter();
                let func = it.next().expect("func");
                let list = it.next().expect("list");
                match list {
                    VMValue::List(fl) => {
                        let mut groups: HashMap<String, Vec<VMValue>> = HashMap::new();
                        let mut order: Vec<String> = Vec::new();
                        for x in fl {
                            let key_val = self.call_value(artifact, func.clone(), vec![x.clone()])?;
                            let key = match key_val {
                                VMValue::Str(s) => s,
                                other => {
                                    return Err(self.error(
                                        artifact,
                                        &format!(
                                            "List.group_by key function must return String, got {}",
                                            vmvalue_type_name(&other)
                                        ),
                                    ));
                                }
                            };
                            if !order.contains(&key) {
                                order.push(key.clone());
                            }
                            groups.entry(key).or_default().push(x);
                        }
                        let mut result = HashMap::new();
                        for key in order {
                            let vals = groups.remove(&key).unwrap_or_default();
                            result.insert(key, VMValue::List(FavList::new(vals)));
                        }
                        Ok(VMValue::Record(result))
                    }
                    _ => Err(self.error(artifact, "List.group_by requires a List as second argument")),
                }
            }
            // ── List high-order additions (v16.4.0) ──────────────────────────────────
            "List.sort_by" => {
                if args.len() != 2 {
                    return Err(self.error(artifact, "List.sort_by requires 2 arguments: (list, key_fn)"));
                }
                let mut it = args.into_iter();
                let list = it.next().expect("list");
                let func = it.next().expect("key_fn");
                match list {
                    VMValue::List(fl) => {
                        let mut keyed: Vec<(VMValue, VMValue)> = Vec::with_capacity(fl.len());
                        for x in fl {
                            let k = self.call_value(artifact, func.clone(), vec![x.clone()])?;
                            keyed.push((k, x));
                        }
                        let mut sort_err: Option<VMError> = None;
                        keyed.sort_by(|(a, _), (b, _)| {
                            if sort_err.is_some() { return std::cmp::Ordering::Equal; }
                            match (a, b) {
                                (VMValue::Int(x), VMValue::Int(y)) => x.cmp(y),
                                (VMValue::Float(x), VMValue::Float(y)) => x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal),
                                (VMValue::Str(x), VMValue::Str(y)) => x.cmp(y),
                                _ => { sort_err = Some(self.error(artifact, "List.sort_by key must return Int, Float, or String")); std::cmp::Ordering::Equal }
                            }
                        });
                        if let Some(e) = sort_err { return Err(e); }
                        Ok(VMValue::List(FavList::new(keyed.into_iter().map(|(_, v)| v).collect())))
                    }
                    _ => Err(self.error(artifact, "List.sort_by requires a List as first argument")),
                }
            }
            "List.sort_by_desc" => {
                if args.len() != 2 {
                    return Err(self.error(artifact, "List.sort_by_desc requires 2 arguments: (list, key_fn)"));
                }
                let mut it = args.into_iter();
                let list = it.next().expect("list");
                let func = it.next().expect("key_fn");
                match list {
                    VMValue::List(fl) => {
                        let mut keyed: Vec<(VMValue, VMValue)> = Vec::with_capacity(fl.len());
                        for x in fl {
                            let k = self.call_value(artifact, func.clone(), vec![x.clone()])?;
                            keyed.push((k, x));
                        }
                        let mut sort_err: Option<VMError> = None;
                        keyed.sort_by(|(a, _), (b, _)| {
                            if sort_err.is_some() { return std::cmp::Ordering::Equal; }
                            match (a, b) {
                                (VMValue::Int(x), VMValue::Int(y)) => y.cmp(x),
                                (VMValue::Float(x), VMValue::Float(y)) => y.partial_cmp(x).unwrap_or(std::cmp::Ordering::Equal),
                                (VMValue::Str(x), VMValue::Str(y)) => y.cmp(x),
                                _ => { sort_err = Some(self.error(artifact, "List.sort_by_desc key must return Int, Float, or String")); std::cmp::Ordering::Equal }
                            }
                        });
                        if let Some(e) = sort_err { return Err(e); }
                        Ok(VMValue::List(FavList::new(keyed.into_iter().map(|(_, v)| v).collect())))
                    }
                    _ => Err(self.error(artifact, "List.sort_by_desc requires a List as first argument")),
                }
            }
            "List.distinct_by" => {
                if args.len() != 2 {
                    return Err(self.error(artifact, "List.distinct_by requires 2 arguments: (list, key_fn)"));
                }
                let mut it = args.into_iter();
                let list = it.next().expect("list");
                let func = it.next().expect("key_fn");
                match list {
                    VMValue::List(fl) => {
                        let mut seen: Vec<String> = Vec::new();
                        let mut out: Vec<VMValue> = Vec::new();
                        for x in fl {
                            let k = self.call_value(artifact, func.clone(), vec![x.clone()])?;
                            let key_str = match &k {
                                VMValue::Str(s) => s.clone(),
                                VMValue::Int(n) => n.to_string(),
                                other => vmvalue_repr(other),
                            };
                            if !seen.contains(&key_str) {
                                seen.push(key_str);
                                out.push(x);
                            }
                        }
                        Ok(VMValue::List(FavList::new(out)))
                    }
                    _ => Err(self.error(artifact, "List.distinct_by requires a List as first argument")),
                }
            }
            "List.count_where" => {
                // alias for List.count
                if args.len() != 2 {
                    return Err(self.error(artifact, "List.count_where requires 2 arguments: (list, pred)"));
                }
                let mut it = args.into_iter();
                let list = it.next().expect("list");
                let pred = it.next().expect("pred");
                match list {
                    VMValue::List(fl) => {
                        let mut count = 0i64;
                        for x in fl {
                            match self.call_value(artifact, pred.clone(), vec![x])? {
                                VMValue::Bool(true) => count += 1,
                                VMValue::Bool(false) => {}
                                other => return Err(self.error(artifact, &format!("List.count_where predicate must return Bool, got {}", vmvalue_type_name(&other)))),
                            }
                        }
                        Ok(VMValue::Int(count))
                    }
                    _ => Err(self.error(artifact, "List.count_where requires a List as first argument")),
                }
            }
            "List.sum_by" => {
                if args.len() != 2 {
                    return Err(self.error(artifact, "List.sum_by requires 2 arguments: (list, fn)"));
                }
                let mut it = args.into_iter();
                let list = it.next().expect("list");
                let func = it.next().expect("fn");
                match list {
                    VMValue::List(fl) => {
                        let mut sum = 0.0f64;
                        for x in fl {
                            match self.call_value(artifact, func.clone(), vec![x])? {
                                VMValue::Float(f) => sum += f,
                                VMValue::Int(n) => sum += n as f64,
                                other => return Err(self.error(artifact, &format!("List.sum_by fn must return Float or Int, got {}", vmvalue_type_name(&other)))),
                            }
                        }
                        Ok(VMValue::Float(sum))
                    }
                    _ => Err(self.error(artifact, "List.sum_by requires a List as first argument")),
                }
            }
            "List.max_by" => {
                if args.len() != 2 {
                    return Err(self.error(artifact, "List.max_by requires 2 arguments: (list, fn)"));
                }
                let mut it = args.into_iter();
                let list = it.next().expect("list");
                let func = it.next().expect("fn");
                match list {
                    VMValue::List(fl) => {
                        let mut best: Option<(f64, VMValue)> = None;
                        for x in fl {
                            let score = match self.call_value(artifact, func.clone(), vec![x.clone()])? {
                                VMValue::Float(f) => f,
                                VMValue::Int(n) => n as f64,
                                other => return Err(self.error(artifact, &format!("List.max_by fn must return Float or Int, got {}", vmvalue_type_name(&other)))),
                            };
                            match &best {
                                None => { best = Some((score, x)); }
                                Some((best_score, _)) if score > *best_score => { best = Some((score, x)); }
                                _ => {}
                            }
                        }
                        match best {
                            Some((_, v)) => Ok(VMValue::Variant("some".into(), Some(Box::new(v)))),
                            None => Ok(VMValue::Variant("none".into(), None)),
                        }
                    }
                    _ => Err(self.error(artifact, "List.max_by requires a List as first argument")),
                }
            }
            "List.min_by" => {
                if args.len() != 2 {
                    return Err(self.error(artifact, "List.min_by requires 2 arguments: (list, fn)"));
                }
                let mut it = args.into_iter();
                let list = it.next().expect("list");
                let func = it.next().expect("fn");
                match list {
                    VMValue::List(fl) => {
                        let mut best: Option<(f64, VMValue)> = None;
                        for x in fl {
                            let score = match self.call_value(artifact, func.clone(), vec![x.clone()])? {
                                VMValue::Float(f) => f,
                                VMValue::Int(n) => n as f64,
                                other => return Err(self.error(artifact, &format!("List.min_by fn must return Float or Int, got {}", vmvalue_type_name(&other)))),
                            };
                            match &best {
                                None => { best = Some((score, x)); }
                                Some((best_score, _)) if score < *best_score => { best = Some((score, x)); }
                                _ => {}
                            }
                        }
                        match best {
                            Some((_, v)) => Ok(VMValue::Variant("some".into(), Some(Box::new(v)))),
                            None => Ok(VMValue::Variant("none".into(), None)),
                        }
                    }
                    _ => Err(self.error(artifact, "List.min_by requires a List as first argument")),
                }
            }

            "Map.merge_with" => {
                if args.len() != 3 {
                    return Err(self.error(artifact, "Map.merge_with requires 3 arguments: (f, m1, m2)"));
                }
                let mut it = args.into_iter();
                let func = it.next().expect("func");
                let m1 = it.next().expect("m1");
                let m2 = it.next().expect("m2");
                let mut base = match m1 {
                    VMValue::Record(m) => m,
                    VMValue::Unit => HashMap::new(),
                    _ => return Err(self.error(artifact, "Map.merge_with: first map must be a Record")),
                };
                let overlay = match m2 {
                    VMValue::Record(m) => m,
                    VMValue::Unit => HashMap::new(),
                    _ => return Err(self.error(artifact, "Map.merge_with: second map must be a Record")),
                };
                for (k, v2) in overlay {
                    if let Some(v1) = base.get(&k).cloned() {
                        let partial = self.call_value(artifact, func.clone(), vec![v1])?;
                        let merged = self.call_value(artifact, partial, vec![v2])?;
                        base.insert(k, merged);
                    } else {
                        base.insert(k, v2);
                    }
                }
                Ok(VMValue::Record(base))
            }
            "Map.filter" => {
                if args.len() != 2 {
                    return Err(self.error(artifact, "Map.filter requires 2 arguments: (pred, m)"));
                }
                let mut it = args.into_iter();
                let func = it.next().expect("func");
                let map = it.next().expect("map");
                let m = match map {
                    VMValue::Record(m) => m,
                    VMValue::Unit => HashMap::new(),
                    _ => return Err(self.error(artifact, "Map.filter: argument must be a Record")),
                };
                let mut result = HashMap::new();
                for (k, v) in m {
                    let partial =
                        self.call_value(artifact, func.clone(), vec![VMValue::Str(k.clone())])?;
                    let keep = self.call_value(artifact, partial, vec![v.clone()])?;
                    match keep {
                        VMValue::Bool(true) => {
                            result.insert(k, v);
                        }
                        VMValue::Bool(false) => {}
                        other => {
                            return Err(self.error(
                                artifact,
                                &format!(
                                    "Map.filter predicate must return Bool, got {}",
                                    vmvalue_type_name(&other)
                                ),
                            ));
                        }
                    }
                }
                Ok(VMValue::Record(result))
            }
            "Option.map" => {
                if args.len() != 2 {
                    return Err(self.error(artifact, "Option.map requires 2 arguments"));
                }
                let mut it = args.into_iter();
                let option = it.next().expect("option");
                let func = it.next().expect("func");
                match option {
                    VMValue::Variant(tag, payload) if tag == "some" => {
                        let inner = payload.map(|value| *value).ok_or_else(|| {
                            self.error(artifact, "Option.map expected payload for some")
                        })?;
                        let mapped = self.call_value(artifact, func, vec![inner])?;
                        Ok(VMValue::Variant("some".to_string(), Some(Box::new(mapped))))
                    }
                    VMValue::Variant(tag, None) if tag == "none" => {
                        Ok(VMValue::Variant("none".to_string(), None))
                    }
                    other => Err(self.error(
                        artifact,
                        &format!(
                            "Option.map requires an Option as first argument, got {}",
                            vmvalue_type_name(&other)
                        ),
                    )),
                }
            }
            "Option.and_then" => {
                if args.len() != 2 {
                    return Err(self.error(artifact, "Option.and_then requires 2 arguments"));
                }
                let mut it = args.into_iter();
                let option = it.next().expect("option");
                let func = it.next().expect("func");
                match option {
                    VMValue::Variant(tag, payload) if tag == "some" => {
                        let inner = payload.map(|value| *value).ok_or_else(|| {
                            self.error(artifact, "Option.and_then expected payload for some")
                        })?;
                        let mapped = self.call_value(artifact, func, vec![inner])?;
                        match mapped {
                            VMValue::Variant(tag, payload) if tag == "some" || tag == "none" => {
                                Ok(VMValue::Variant(tag, payload))
                            }
                            other => Err(self.error(
                                artifact,
                                &format!(
                                    "Option.and_then callback must return Option, got {}",
                                    vmvalue_type_name(&other)
                                ),
                            )),
                        }
                    }
                    VMValue::Variant(tag, None) if tag == "none" => {
                        Ok(VMValue::Variant("none".to_string(), None))
                    }
                    other => Err(self.error(
                        artifact,
                        &format!(
                            "Option.and_then requires an Option as first argument, got {}",
                            vmvalue_type_name(&other)
                        ),
                    )),
                }
            }
            "Option.unwrap_or" => {
                if args.len() != 2 {
                    return Err(self.error(artifact, "Option.unwrap_or requires 2 arguments"));
                }
                let mut it = args.into_iter();
                let option = it.next().expect("option");
                let default = it.next().expect("default");
                match option {
                    VMValue::Variant(tag, payload) if tag == "some" => {
                        payload.map(|value| *value).ok_or_else(|| {
                            self.error(artifact, "Option.unwrap_or expected payload for some")
                        })
                    }
                    VMValue::Variant(tag, None) if tag == "none" => Ok(default),
                    other => Err(self.error(
                        artifact,
                        &format!(
                            "Option.unwrap_or requires an Option as first argument, got {}",
                            vmvalue_type_name(&other)
                        ),
                    )),
                }
            }
            "Option.or_else" => {
                if args.len() != 2 {
                    return Err(self.error(artifact, "Option.or_else requires 2 arguments"));
                }
                let mut it = args.into_iter();
                let option = it.next().expect("option");
                let func = it.next().expect("func");
                match option {
                    VMValue::Variant(tag, payload) if tag == "some" => {
                        Ok(VMValue::Variant(tag, payload))
                    }
                    VMValue::Variant(tag, None) if tag == "none" => {
                        let mapped = self.call_value(artifact, func, vec![])?;
                        match mapped {
                            VMValue::Variant(tag, payload) if tag == "some" || tag == "none" => {
                                Ok(VMValue::Variant(tag, payload))
                            }
                            other => Err(self.error(
                                artifact,
                                &format!(
                                    "Option.or_else callback must return Option, got {}",
                                    vmvalue_type_name(&other)
                                ),
                            )),
                        }
                    }
                    other => Err(self.error(
                        artifact,
                        &format!(
                            "Option.or_else requires an Option as first argument, got {}",
                            vmvalue_type_name(&other)
                        ),
                    )),
                }
            }
            "Option.is_some" => {
                if args.len() != 1 {
                    return Err(self.error(artifact, "Option.is_some requires 1 argument"));
                }
                match args.into_iter().next().expect("option") {
                    VMValue::Variant(tag, payload) if tag == "some" => {
                        Ok(VMValue::Bool(payload.is_some()))
                    }
                    VMValue::Variant(tag, None) if tag == "none" => Ok(VMValue::Bool(false)),
                    other => Err(self.error(
                        artifact,
                        &format!(
                            "Option.is_some requires an Option argument, got {}",
                            vmvalue_type_name(&other)
                        ),
                    )),
                }
            }
            "Option.is_none" => {
                if args.len() != 1 {
                    return Err(self.error(artifact, "Option.is_none requires 1 argument"));
                }
                match args.into_iter().next().expect("option") {
                    VMValue::Variant(tag, payload) if tag == "some" => {
                        Ok(VMValue::Bool(payload.is_none()))
                    }
                    VMValue::Variant(tag, None) if tag == "none" => Ok(VMValue::Bool(true)),
                    other => Err(self.error(
                        artifact,
                        &format!(
                            "Option.is_none requires an Option argument, got {}",
                            vmvalue_type_name(&other)
                        ),
                    )),
                }
            }
            "Option.to_result" => {
                if args.len() != 2 {
                    return Err(self.error(artifact, "Option.to_result requires 2 arguments"));
                }
                let mut it = args.into_iter();
                let option = it.next().expect("option");
                let err = it.next().expect("err");
                match option {
                    VMValue::Variant(tag, payload) if tag == "some" => {
                        let inner = payload.map(|value| *value).ok_or_else(|| {
                            self.error(artifact, "Option.to_result expected payload for some")
                        })?;
                        Ok(VMValue::Variant("ok".to_string(), Some(Box::new(inner))))
                    }
                    VMValue::Variant(tag, None) if tag == "none" => {
                        Ok(VMValue::Variant("err".to_string(), Some(Box::new(err))))
                    }
                    other => Err(self.error(
                        artifact,
                        &format!(
                            "Option.to_result requires an Option as first argument, got {}",
                            vmvalue_type_name(&other)
                        ),
                    )),
                }
            }
            "Result.map" => {
                if args.len() != 2 {
                    return Err(self.error(artifact, "Result.map requires 2 arguments"));
                }
                let mut it = args.into_iter();
                let result = it.next().expect("result");
                let func = it.next().expect("func");
                match result {
                    VMValue::Variant(tag, payload) if tag == "ok" => {
                        let inner = payload.map(|value| *value).ok_or_else(|| {
                            self.error(artifact, "Result.map expected payload for ok")
                        })?;
                        let mapped = self.call_value(artifact, func, vec![inner])?;
                        Ok(VMValue::Variant("ok".to_string(), Some(Box::new(mapped))))
                    }
                    VMValue::Variant(tag, payload) if tag == "err" => {
                        Ok(VMValue::Variant(tag, payload))
                    }
                    other => Err(self.error(
                        artifact,
                        &format!(
                            "Result.map requires a Result as first argument, got {}",
                            vmvalue_type_name(&other)
                        ),
                    )),
                }
            }
            "Result.map_err" => {
                if args.len() != 2 {
                    return Err(self.error(artifact, "Result.map_err requires 2 arguments"));
                }
                let mut it = args.into_iter();
                let result = it.next().expect("result");
                let func = it.next().expect("func");
                match result {
                    VMValue::Variant(tag, payload) if tag == "ok" => {
                        Ok(VMValue::Variant(tag, payload))
                    }
                    VMValue::Variant(tag, payload) if tag == "err" => {
                        let inner = payload.map(|value| *value).ok_or_else(|| {
                            self.error(artifact, "Result.map_err expected payload for err")
                        })?;
                        let mapped = self.call_value(artifact, func, vec![inner])?;
                        Ok(VMValue::Variant("err".to_string(), Some(Box::new(mapped))))
                    }
                    other => Err(self.error(
                        artifact,
                        &format!(
                            "Result.map_err requires a Result as first argument, got {}",
                            vmvalue_type_name(&other)
                        ),
                    )),
                }
            }
            "Result.and_then" => {
                if args.len() != 2 {
                    return Err(self.error(artifact, "Result.and_then requires 2 arguments"));
                }
                let mut it = args.into_iter();
                let result = it.next().expect("result");
                let func = it.next().expect("func");
                match result {
                    VMValue::Variant(tag, payload) if tag == "ok" => {
                        let inner = payload.map(|value| *value).ok_or_else(|| {
                            self.error(artifact, "Result.and_then expected payload for ok")
                        })?;
                        let mapped = self.call_value(artifact, func, vec![inner])?;
                        match mapped {
                            VMValue::Variant(tag, payload) if tag == "ok" || tag == "err" => {
                                Ok(VMValue::Variant(tag, payload))
                            }
                            other => Err(self.error(
                                artifact,
                                &format!(
                                    "Result.and_then callback must return Result, got {}",
                                    vmvalue_type_name(&other)
                                ),
                            )),
                        }
                    }
                    VMValue::Variant(tag, payload) if tag == "err" => {
                        Ok(VMValue::Variant(tag, payload))
                    }
                    other => Err(self.error(
                        artifact,
                        &format!(
                            "Result.and_then requires a Result as first argument, got {}",
                            vmvalue_type_name(&other)
                        ),
                    )),
                }
            }
            "Result.unwrap_or" => {
                if args.len() != 2 {
                    return Err(self.error(artifact, "Result.unwrap_or requires 2 arguments"));
                }
                let mut it = args.into_iter();
                let result = it.next().expect("result");
                let default = it.next().expect("default");
                match result {
                    VMValue::Variant(tag, payload) if tag == "ok" => {
                        payload.map(|value| *value).ok_or_else(|| {
                            self.error(artifact, "Result.unwrap_or expected payload for ok")
                        })
                    }
                    VMValue::Variant(tag, payload) if tag == "err" => {
                        let _ = payload.map(|value| *value).ok_or_else(|| {
                            self.error(artifact, "Result.unwrap_or expected payload for err")
                        })?;
                        Ok(default)
                    }
                    other => Err(self.error(
                        artifact,
                        &format!(
                            "Result.unwrap_or requires a Result as first argument, got {}",
                            vmvalue_type_name(&other)
                        ),
                    )),
                }
            }
            "Result.is_ok" => {
                if args.len() != 1 {
                    return Err(self.error(artifact, "Result.is_ok requires 1 argument"));
                }
                match args.into_iter().next().expect("result") {
                    VMValue::Variant(tag, payload) if tag == "ok" => {
                        Ok(VMValue::Bool(payload.is_some()))
                    }
                    VMValue::Variant(tag, _) if tag == "err" => {
                        Ok(VMValue::Bool(false))
                    }
                    other => Err(self.error(
                        artifact,
                        &format!(
                            "Result.is_ok requires a Result argument, got {}",
                            vmvalue_type_name(&other)
                        ),
                    )),
                }
            }
            "Result.is_err" => {
                if args.len() != 1 {
                    return Err(self.error(artifact, "Result.is_err requires 1 argument"));
                }
                match args.into_iter().next().expect("result") {
                    VMValue::Variant(tag, _) if tag == "ok" => {
                        Ok(VMValue::Bool(false))
                    }
                    VMValue::Variant(tag, payload) if tag == "err" => {
                        Ok(VMValue::Bool(payload.is_some()))
                    }
                    other => Err(self.error(
                        artifact,
                        &format!(
                            "Result.is_err requires a Result argument, got {}",
                            vmvalue_type_name(&other)
                        ),
                    )),
                }
            }
            "Result.all" => {
                if args.len() != 1 {
                    return Err(self.error(artifact, "Result.all requires 1 argument"));
                }
                match args.into_iter().next().expect("list") {
                    VMValue::List(fl) => {
                        let mut oks = Vec::with_capacity(fl.len());
                        for item in fl {
                            match item {
                                VMValue::Variant(tag, payload) if tag == "ok" => {
                                    let v = payload
                                        .map(|b| *b)
                                        .unwrap_or(VMValue::Unit);
                                    oks.push(v);
                                }
                                VMValue::Variant(tag, payload) if tag == "err" => {
                                    // first error short-circuits
                                    return Ok(VMValue::Variant("err".to_string(), payload));
                                }
                                other => {
                                    return Err(self.error(
                                        artifact,
                                        &format!(
                                            "Result.all requires List<Result<A,E>>, got {}",
                                            vmvalue_type_name(&other)
                                        ),
                                    ));
                                }
                            }
                        }
                        Ok(VMValue::Variant(
                            "ok".to_string(),
                            Some(Box::new(VMValue::List(FavList::new(oks)))),
                        ))
                    }
                    other => Err(self.error(
                        artifact,
                        &format!("Result.all requires a List argument, got {}", vmvalue_type_name(&other)),
                    )),
                }
            }
            "Result.to_option" => {
                if args.len() != 1 {
                    return Err(self.error(artifact, "Result.to_option requires 1 argument"));
                }
                match args.into_iter().next().expect("result") {
                    VMValue::Variant(tag, payload) if tag == "ok" => {
                        Ok(VMValue::Variant("some".to_string(), payload))
                    }
                    VMValue::Variant(tag, payload) if tag == "err" => {
                        let _ = payload.map(|value| *value).ok_or_else(|| {
                            self.error(artifact, "Result.to_option expected payload for err")
                        })?;
                        Ok(VMValue::Variant("none".to_string(), None))
                    }
                    other => Err(self.error(
                        artifact,
                        &format!(
                            "Result.to_option requires a Result argument, got {}",
                            vmvalue_type_name(&other)
                        ),
                    )),
                }
            }
            // Stream builtins (v2.9.0)
            "Stream.from" | "Stream.of" => {
                let mut it = args.into_iter();
                let list = it
                    .next()
                    .ok_or_else(|| self.error(artifact, "Stream.from requires 1 argument"))?;
                match list {
                    VMValue::List(fl) => Ok(VMValue::Stream(Box::new(VMStream::Of(fl.to_vec())))),
                    other => Err(self.error(
                        artifact,
                        &format!(
                            "Stream.from requires a List argument, got {}",
                            vmvalue_type_name(&other)
                        ),
                    )),
                }
            }
            "Stream.gen" => {
                if args.len() != 2 {
                    return Err(self.error(artifact, "Stream.gen requires 2 arguments"));
                }
                let mut it = args.into_iter();
                let seed = it.next().expect("seed");
                let next_fn = it.next().expect("next_fn");
                Ok(VMValue::Stream(Box::new(VMStream::Gen { seed, next_fn })))
            }
            "Stream.map" => {
                if args.len() != 2 {
                    return Err(self.error(artifact, "Stream.map requires 2 arguments"));
                }
                let mut it = args.into_iter();
                let stream = it.next().expect("stream");
                let map_fn = it.next().expect("map_fn");
                match stream {
                    VMValue::Stream(inner) => Ok(VMValue::Stream(Box::new(VMStream::Map {
                        inner: inner,
                        map_fn,
                    }))),
                    other => Err(self.error(
                        artifact,
                        &format!(
                            "Stream.map requires a Stream as first argument, got {}",
                            vmvalue_type_name(&other)
                        ),
                    )),
                }
            }
            "Stream.filter" => {
                if args.len() != 2 {
                    return Err(self.error(artifact, "Stream.filter requires 2 arguments"));
                }
                let mut it = args.into_iter();
                let stream = it.next().expect("stream");
                let pred_fn = it.next().expect("pred_fn");
                match stream {
                    VMValue::Stream(inner) => Ok(VMValue::Stream(Box::new(VMStream::Filter {
                        inner: inner,
                        pred_fn,
                    }))),
                    other => Err(self.error(
                        artifact,
                        &format!(
                            "Stream.filter requires a Stream as first argument, got {}",
                            vmvalue_type_name(&other)
                        ),
                    )),
                }
            }
            "Stream.take" => {
                if args.len() != 2 {
                    return Err(self.error(artifact, "Stream.take requires 2 arguments"));
                }
                let mut it = args.into_iter();
                let stream = it.next().expect("stream");
                let n_val = it.next().expect("n");
                match (stream, n_val) {
                    (VMValue::Stream(inner), VMValue::Int(n)) => {
                        Ok(VMValue::Stream(Box::new(VMStream::Take {
                            inner: inner,
                            n,
                        })))
                    }
                    (VMValue::Stream(_), other) => Err(self.error(
                        artifact,
                        &format!(
                            "Stream.take second argument must be Int, got {}",
                            vmvalue_type_name(&other)
                        ),
                    )),
                    (other, _) => Err(self.error(
                        artifact,
                        &format!(
                            "Stream.take requires a Stream as first argument, got {}",
                            vmvalue_type_name(&other)
                        ),
                    )),
                }
            }
            "Stream.to_list" => {
                let mut it = args.into_iter();
                let stream = it
                    .next()
                    .ok_or_else(|| self.error(artifact, "Stream.to_list requires 1 argument"))?;
                match stream {
                    VMValue::Stream(s) => {
                        let items = self.materialize_stream(artifact, *s)?;
                        Ok(VMValue::List(FavList::new(items)))
                    }
                    other => Err(self.error(
                        artifact,
                        &format!(
                            "Stream.to_list requires a Stream argument, got {}",
                            vmvalue_type_name(&other)
                        ),
                    )),
                }
            }
            // ── v26.4.0: Stream.* operations ─────────────────────────────────────────
            "Stream.flat_map" => {
                if args.len() != 2 {
                    return Err(self.error(artifact, "Stream.flat_map requires 2 arguments"));
                }
                let mut it = args.into_iter();
                let stream = it.next().expect("stream");
                let map_fn = it.next().expect("fn");
                match stream {
                    VMValue::Stream(inner) => Ok(VMValue::Stream(Box::new(VMStream::FlatMap {
                        inner,
                        map_fn,
                    }))),
                    other => Err(self.error(
                        artifact,
                        &format!(
                            "Stream.flat_map requires a Stream as first argument, got {}",
                            vmvalue_type_name(&other)
                        ),
                    )),
                }
            }
            "Stream.window" => {
                if args.len() != 3 {
                    return Err(self.error(artifact, "Stream.window requires 3 arguments"));
                }
                let mut it = args.into_iter();
                let stream = it.next().expect("stream");
                let size_val = it.next().expect("size");
                let window_fn = it.next().expect("fn");
                match (stream, size_val) {
                    (VMValue::Stream(inner), VMValue::Int(size)) => {
                        Ok(VMValue::Stream(Box::new(VMStream::Window { inner, size, window_fn })))
                    }
                    (VMValue::Stream(_), other) => Err(self.error(
                        artifact,
                        &format!(
                            "Stream.window second argument must be Int, got {}",
                            vmvalue_type_name(&other)
                        ),
                    )),
                    (other, _) => Err(self.error(
                        artifact,
                        &format!(
                            "Stream.window requires a Stream as first argument, got {}",
                            vmvalue_type_name(&other)
                        ),
                    )),
                }
            }
            "Stream.merge" => {
                if args.len() != 1 {
                    return Err(self.error(artifact, "Stream.merge requires 1 argument (list of streams)"));
                }
                let streams_val = args.into_iter().next().expect("streams");
                match streams_val {
                    VMValue::List(list) => {
                        let mut streams = Vec::new();
                        for item in list.iter() {
                            match item {
                                VMValue::Stream(s) => streams.push(s.as_ref().clone()),
                                other => return Err(self.error(
                                    artifact,
                                    &format!(
                                        "Stream.merge: each element must be a Stream, got {}",
                                        vmvalue_type_name(other)
                                    ),
                                )),
                            }
                        }
                        Ok(VMValue::Stream(Box::new(VMStream::Merge { streams })))
                    }
                    other => Err(self.error(
                        artifact,
                        &format!(
                            "Stream.merge requires a List of Streams, got {}",
                            vmvalue_type_name(&other)
                        ),
                    )),
                }
            }
            "Stream.split" => {
                // Immediately materializes and returns VMValue::List([trues_list, falses_list]).
                // Returning VMValue::Stream would create semantic confusion because the 2-element
                // result does not compose with Stream.map/filter/take in a meaningful way.
                if args.len() != 2 {
                    return Err(self.error(artifact, "Stream.split requires 2 arguments"));
                }
                let mut it = args.into_iter();
                let stream = it.next().expect("stream");
                let pred_fn = it.next().expect("predicate");
                match stream {
                    VMValue::Stream(inner) => {
                        let items = self.materialize_stream(artifact, *inner)?;
                        let mut trues = Vec::new();
                        let mut falses = Vec::new();
                        for item in items {
                            let keep = self.call_value(artifact, pred_fn.clone(), vec![item.clone()])?;
                            match keep {
                                VMValue::Bool(true) => trues.push(item),
                                VMValue::Bool(false) => falses.push(item),
                                other => return Err(self.error(
                                    artifact,
                                    &format!(
                                        "Stream.split predicate must return Bool, got {}",
                                        vmvalue_type_name(&other)
                                    ),
                                )),
                            }
                        }
                        Ok(VMValue::List(FavList::new(vec![
                            VMValue::List(FavList::new(trues)),
                            VMValue::List(FavList::new(falses)),
                        ])))
                    }
                    other => Err(self.error(
                        artifact,
                        &format!(
                            "Stream.split requires a Stream as first argument, got {}",
                            vmvalue_type_name(&other)
                        ),
                    )),
                }
            }
            // ── end v26.4.0 Stream.* ─────────────────────────────────────────────────
            "Http.serve_raw" => {
                if args.len() != 3 {
                    return Err(self.error(artifact, "Http.serve_raw requires 3 arguments"));
                }
                let mut it = args.into_iter();
                let port = match it.next().expect("port") {
                    VMValue::Int(port) => port,
                    other => {
                        return Err(self.error(
                            artifact,
                            &format!(
                                "Http.serve_raw expects Int port, got {}",
                                vmvalue_type_name(&other)
                            ),
                        ));
                    }
                };
                let routes = match it.next().expect("routes") {
                    VMValue::List(fl) => fl,
                    other => {
                        return Err(self.error(
                            artifact,
                            &format!(
                                "Http.serve_raw expects List<Map<String,String>>, got {}",
                                vmvalue_type_name(&other)
                            ),
                        ));
                    }
                };
                let handler_name = match it.next().expect("handler_name") {
                    VMValue::Str(name) => name,
                    other => {
                        return Err(self.error(
                            artifact,
                            &format!(
                                "Http.serve_raw expects String handler_name, got {}",
                                vmvalue_type_name(&other)
                            ),
                        ));
                    }
                };
                let server = tiny_http::Server::http(format!("0.0.0.0:{port}")).map_err(|e| {
                    self.error(artifact, &format!("Http.serve_raw bind failed: {}", e))
                })?;
                let mut request = server.recv().map_err(|e| {
                    self.error(artifact, &format!("Http.serve_raw recv failed: {}", e))
                })?;
                let method = request.method().as_str().to_string();
                let path = request.url().to_string();
                let mut body = String::new();
                let headers_map: HashMap<String, VMValue> = request
                    .headers()
                    .iter()
                    .map(|h| {
                        (
                            h.field.as_str().to_string().to_lowercase(),
                            VMValue::Str(h.value.as_str().to_string()),
                        )
                    })
                    .collect();
                let mut reader = request.as_reader();
                std::io::Read::read_to_string(&mut reader, &mut body).map_err(|e| {
                    self.error(artifact, &format!("Http.serve_raw body read failed: {}", e))
                })?;

                let route_allowed = routes.into_iter().any(|route| match route {
                    VMValue::Record(map) => {
                        let route_method = map.get("method").map(vm_scalar_to_plain_string);
                        let route_path = map.get("path").map(vm_scalar_to_plain_string);
                        route_method.as_deref().unwrap_or("") == method
                            && (route_path.as_deref().unwrap_or("") == path
                                || route_path.as_deref().unwrap_or("") == "*")
                    }
                    _ => false,
                });

                let response_value = if route_allowed {
                    let fn_idx = artifact.fn_idx_by_name(&handler_name).ok_or_else(|| {
                        self.error(
                            artifact,
                            &format!("Http.serve_raw unknown handler `{}`", handler_name),
                        )
                    })?;
                    let function = &artifact.functions[fn_idx];
                    let args = match function.param_count {
                        0 => vec![],
                        1 => {
                            let authorization = headers_map
                                .get("authorization")
                                .and_then(|v| {
                                    if let VMValue::Str(s) = v {
                                        Some(s.clone())
                                    } else {
                                        None
                                    }
                                })
                                .unwrap_or_default();
                            let mut req = HashMap::new();
                            req.insert("method".to_string(), VMValue::Str(method.clone()));
                            req.insert("path".to_string(), VMValue::Str(path.clone()));
                            req.insert("body".to_string(), VMValue::Str(body.clone()));
                            req.insert("authorization".to_string(), VMValue::Str(authorization));
                            vec![VMValue::Record(req)]
                        }
                        3 => vec![
                            VMValue::Str(method.clone()),
                            VMValue::Str(path.clone()),
                            VMValue::Str(body.clone()),
                        ],
                        other => {
                            return Err(self.error(
                                artifact,
                                &format!(
                                    "Http.serve_raw handler `{}` must take 0, 1, or 3 args, got {}",
                                    handler_name, other
                                ),
                            ));
                        }
                    };
                    self.invoke_function(artifact, fn_idx, args)?
                } else {
                    http_response_vm(404, "not found".to_string(), "text/plain".to_string())
                };

                let (status, resp_body, content_type) = match response_value {
                    VMValue::Record(map) => {
                        let status = match map.get("status") {
                            Some(VMValue::Int(n)) => *n as u16,
                            _ => 200,
                        };
                        let body = map
                            .get("body")
                            .map(vm_scalar_to_plain_string)
                            .unwrap_or_default();
                        let content_type = map
                            .get("content_type")
                            .map(vm_scalar_to_plain_string)
                            .unwrap_or_else(|| "text/plain".to_string());
                        (status, body, content_type)
                    }
                    other => {
                        return Err(self.error(
                            artifact,
                            &format!(
                                "Http.serve_raw handler must return HttpResponse record, got {}",
                                vmvalue_type_name(&other)
                            ),
                        ));
                    }
                };
                let response = tiny_http::Response::from_string(resp_body)
                    .with_status_code(status)
                    .with_header(
                        tiny_http::Header::from_bytes(
                            b"Content-Type".as_slice(),
                            content_type.as_bytes(),
                        )
                        .map_err(|_| {
                            self.error(artifact, "Http.serve_raw invalid Content-Type header")
                        })?,
                    );
                request.respond(response).map_err(|e| {
                    self.error(artifact, &format!("Http.serve_raw respond failed: {}", e))
                })?;
                Ok(VMValue::Unit)
            }
            "Grpc.serve_raw" => {
                if args.len() != 2 {
                    return Err(self.error(artifact, "Grpc.serve_raw requires 2 arguments"));
                }
                let mut it = args.into_iter();
                let port = match it.next().expect("port") {
                    VMValue::Int(port) => port,
                    other => {
                        return Err(self.error(
                            artifact,
                            &format!(
                                "Grpc.serve_raw expects Int port, got {}",
                                vmvalue_type_name(&other)
                            ),
                        ));
                    }
                };
                let _service_name = match it.next().expect("service_name") {
                    VMValue::Str(name) => name,
                    other => {
                        return Err(self.error(
                            artifact,
                            &format!(
                                "Grpc.serve_raw expects String service_name, got {}",
                                vmvalue_type_name(&other)
                            ),
                        ));
                    }
                };
                let (req_tx, req_rx) = std::sync::mpsc::channel::<GrpcRequestMsg>();
                grpc_serve_impl(port, req_tx)
                    .map_err(|e| self.error(artifact, &format!("Grpc.serve_raw failed: {}", e)))?;
                loop {
                    let (handler_name, proto_bytes, res_tx) = match req_rx.recv() {
                        Ok(msg) => msg,
                        Err(_) => break,
                    };
                    let fn_idx = match artifact.fn_idx_by_name(&handler_name) {
                        Some(idx) => idx,
                        None => {
                            let _ = res_tx.send(Err(format!(
                                "Grpc.serve_raw: unknown handler `{}`",
                                handler_name
                            )));
                            continue;
                        }
                    };
                    let req_value = match proto_bytes_to_string_map(&proto_bytes) {
                        Ok(row) => VMValue::Record(
                            row.into_iter().map(|(k, v)| (k, VMValue::Str(v))).collect(),
                        ),
                        Err(e) => {
                            let _ = res_tx.send(Err(format!("proto decode failed: {}", e)));
                            continue;
                        }
                    };
                    let result = self.invoke_function(artifact, fn_idx, vec![req_value]);
                    let resp = grpc_vm_value_to_proto_bytes(result.map_err(|e| e.message));
                    let _ = res_tx.send(resp.map(|b| encode_grpc_frame(&b)));
                }
                Ok(VMValue::Unit)
            }
            "Grpc.serve_stream_raw" => {
                if args.len() != 2 {
                    return Err(self.error(artifact, "Grpc.serve_stream_raw requires 2 arguments"));
                }
                let mut it = args.into_iter();
                let port = match it.next().expect("port") {
                    VMValue::Int(port) => port,
                    other => {
                        return Err(self.error(
                            artifact,
                            &format!(
                                "Grpc.serve_stream_raw expects Int port, got {}",
                                vmvalue_type_name(&other)
                            ),
                        ));
                    }
                };
                let _service_name = match it.next().expect("service_name") {
                    VMValue::Str(name) => name,
                    other => {
                        return Err(self.error(
                            artifact,
                            &format!(
                                "Grpc.serve_stream_raw expects String service_name, got {}",
                                vmvalue_type_name(&other)
                            ),
                        ));
                    }
                };
                let (req_tx, req_rx) = std::sync::mpsc::channel::<GrpcRequestMsg>();
                grpc_serve_impl(port, req_tx).map_err(|e| {
                    self.error(artifact, &format!("Grpc.serve_stream_raw failed: {}", e))
                })?;
                loop {
                    let (handler_name, proto_bytes, res_tx) = match req_rx.recv() {
                        Ok(msg) => msg,
                        Err(_) => break,
                    };
                    let fn_idx = match artifact.fn_idx_by_name(&handler_name) {
                        Some(idx) => idx,
                        None => {
                            let _ = res_tx.send(Err(format!(
                                "Grpc.serve_stream_raw: unknown handler `{}`",
                                handler_name
                            )));
                            continue;
                        }
                    };
                    let req_value = match proto_bytes_to_string_map(&proto_bytes) {
                        Ok(row) => VMValue::Record(
                            row.into_iter().map(|(k, v)| (k, VMValue::Str(v))).collect(),
                        ),
                        Err(e) => {
                            let _ = res_tx.send(Err(format!("proto decode failed: {}", e)));
                            continue;
                        }
                    };
                    let result = self.invoke_function(artifact, fn_idx, vec![req_value]);
                    let frames = match result {
                        Ok(VMValue::List(fl)) => {
                            let mut combined: Vec<u8> = Vec::new();
                            let mut ok = true;
                            for item in fl {
                                match grpc_vm_value_to_proto_bytes(Ok(item)) {
                                    Ok(b) => {
                                        combined.extend_from_slice(&encode_grpc_frame(&b));
                                    }
                                    Err(e) => {
                                        let _ = res_tx.send(Err(e));
                                        ok = false;
                                        break;
                                    }
                                }
                            }
                            if !ok {
                                continue;
                            }
                            Ok(combined)
                        }
                        Ok(other) => Err(format!(
                            "Grpc.serve_stream_raw handler must return List, got {}",
                            vmvalue_type_name(&other)
                        )),
                        Err(e) => Err(e.message),
                    };
                    let _ = res_tx.send(frames);
                }
                Ok(VMValue::Unit)
            }
            // par [A, B] parallel stage execution (v9.13.0)
            // Signature: IO.par_execute_raw(names: List<String>, input: Any) -> List<Any>
            "IO.par_execute_raw" => {
                if args.len() != 2 {
                    return Err(self.error(artifact, "IO.par_execute_raw requires 2 arguments"));
                }
                let mut it = args.into_iter();
                let names_val = it.next().unwrap();
                let input = it.next().unwrap();

                let names: Vec<String> = match names_val {
                    VMValue::List(fl) => fl
                        .iter()
                        .map(|v| match v {
                            VMValue::Str(s) => Ok(s.clone()),
                            _ => Err(self.error(
                                artifact,
                                "IO.par_execute_raw: stage names must be strings",
                            )),
                        })
                        .collect::<Result<Vec<_>, _>>()?,
                    _ => {
                        return Err(self.error(
                            artifact,
                            "IO.par_execute_raw: first argument must be a List<String>",
                        ))
                    }
                };

                let artifact_clone = artifact.clone();
                let db_str = self.db_path.clone();

                let handles: Vec<std::thread::JoinHandle<Result<VMValue, String>>> = names
                    .into_iter()
                    .map(|fn_name| {
                        let artifact_c = artifact_clone.clone();
                        let input_c = input.clone();
                        let db_c = db_str.clone();
                        std::thread::spawn(move || {
                            let fn_idx =
                                artifact_c.fn_idx_by_name(&fn_name).ok_or_else(|| {
                                    format!(
                                        "E0017: par ステップ内の stage '{}' が定義されていません",
                                        fn_name
                                    )
                                })?;
                            VM::run_with_vmvalues(
                                &artifact_c,
                                fn_idx,
                                vec![input_c],
                                db_c,
                                None,
                            )
                            .map(|(v, _)| v)
                            .map_err(|e| e.message)
                        })
                    })
                    .collect();

                let mut results = Vec::with_capacity(handles.len());
                for handle in handles {
                    match handle.join() {
                        Ok(Ok(v)) => results.push(v),
                        Ok(Err(e)) => return Err(self.error(artifact, &e)),
                        Err(_) => {
                            return Err(self.error(
                                artifact,
                                "IO.par_execute_raw: a parallel stage panicked",
                            ))
                        }
                    }
                }
                Ok(VMValue::List(FavList::new(results)))
            }

            // v22.2.0: Distributed parallel execution (stub — logs endpoints, falls back to local par)
            // NOTE: Uses std::thread::spawn; not supported on wasm32 (same as IO.par_execute_raw).
            "IO.par_distributed_raw" => {
                if args.len() != 2 {
                    return Err(self.error(artifact, "IO.par_distributed_raw requires 2 arguments"));
                }
                let mut it = args.into_iter();
                let names_val = it.next().unwrap();
                let input = it.next().unwrap();

                let names: Vec<String> = match names_val {
                    VMValue::List(fl) => fl
                        .iter()
                        .map(|v| match v {
                            VMValue::Str(s) => Ok(s.clone()),
                            _ => Err(self.error(
                                artifact,
                                "IO.par_distributed_raw: stage names must be strings",
                            )),
                        })
                        .collect::<Result<Vec<_>, _>>()?,
                    _ => {
                        return Err(self.error(
                            artifact,
                            "IO.par_distributed_raw: first argument must be a List<String>",
                        ))
                    }
                };

                let endpoints = get_worker_endpoints();
                if !endpoints.is_empty() {
                    eprintln!(
                        "[par_distributed] distributing to {} workers (stub: local fallback)",
                        endpoints.len()
                    );
                }

                // v22.2.0: stub — local parallel execution (actual gRPC dispatch in v22.3+)
                let artifact_clone = artifact.clone();
                let db_str = self.db_path.clone();

                let handles: Vec<std::thread::JoinHandle<Result<VMValue, String>>> = names
                    .into_iter()
                    .map(|fn_name| {
                        let artifact_c = artifact_clone.clone();
                        let input_c = input.clone();
                        let db_c = db_str.clone();
                        std::thread::spawn(move || {
                            let fn_idx =
                                artifact_c.fn_idx_by_name(&fn_name).ok_or_else(|| {
                                    format!(
                                        "E0017: par_distributed ステップ内の stage '{}' が定義されていません",
                                        fn_name
                                    )
                                })?;
                            VM::run_with_vmvalues(
                                &artifact_c,
                                fn_idx,
                                vec![input_c],
                                db_c,
                                None,
                            )
                            .map(|(v, _)| v)
                            .map_err(|e| e.message)
                        })
                    })
                    .collect();

                let mut results = Vec::with_capacity(handles.len());
                for handle in handles {
                    match handle.join() {
                        Ok(Ok(v)) => results.push(v),
                        Ok(Err(e)) => return Err(self.error(artifact, &e)),
                        Err(_) => {
                            return Err(self.error(
                                artifact,
                                "IO.par_distributed_raw: a parallel stage panicked",
                            ))
                        }
                    }
                }
                Ok(VMValue::List(FavList::new(results)))
            }

            "__streaming_pipeline" => {
                // args: [source_list: List<T>, stages: List<Fn>, chunk_size: Int]
                let mut args_iter = args.into_iter();
                let source = args_iter.next().unwrap_or(VMValue::Unit);
                let stages = args_iter.next().unwrap_or(VMValue::Unit);
                let chunk_size = match args_iter.next() {
                    Some(VMValue::Int(n)) if n > 0 => n as usize,
                    _ => 512,
                };
                let items = match source {
                    VMValue::List(fl) => fl.to_vec(),
                    other => vec![other],
                };
                let stage_fns: Vec<VMValue> = match stages {
                    VMValue::List(fl) => fl.to_vec(),
                    _ => vec![],
                };
                if stage_fns.is_empty() {
                    return Ok(VMValue::List(FavList::new(items)));
                }
                let mut result: Vec<VMValue> = Vec::new();
                for chunk_items in items.chunks(chunk_size) {
                    #[cfg(not(target_arch = "wasm32"))]
                    self.chunk_arena.start_chunk();

                    #[cfg(not(target_arch = "wasm32"))]
                    let mut buf = self.chunk_arena.acquire(chunk_items.len());
                    #[cfg(target_arch = "wasm32")]
                    let mut buf = Vec::with_capacity(chunk_items.len());

                    buf.extend_from_slice(chunk_items);
                    let mut current = VMValue::List(FavList::new(buf));

                    for stage_fn in &stage_fns {
                        current = self.call_value(artifact, stage_fn.clone(), vec![current])?;
                    }

                    #[cfg(not(target_arch = "wasm32"))]
                    self.chunk_arena.end_chunk(current, &mut result);
                    #[cfg(target_arch = "wasm32")]
                    match current {
                        VMValue::List(fl) => result.extend(fl.to_vec()),
                        other => result.push(other),
                    }
                }
                // end_chunk が各チャンク末に bump.reset() を呼ぶため
                // ループ後の reset_bump() は不要（冗長な二重リセットを回避）

                Ok(VMValue::List(FavList::new(result)))
            }

            // ── v22.1.0: Stage checkpoint lookup ──────────────────────────────────
            // args: [stage_name: Str]
            // v22.1.0 scope: resume lookup only (hit/miss signal).
            //   - Hit: returns the raw checkpoint bytes as Str.
            //   - Miss: returns Bool(false).
            // Full stage_fn wrapping and checkpoint write are deferred to v22.3+.
            "__checkpoint_wrap" => {
                let stage_name = match args.into_iter().next() {
                    Some(VMValue::Str(s)) => s,
                    _ => return Ok(VMValue::Bool(false)),
                };
                #[cfg(not(target_arch = "wasm32"))]
                {
                    let resume_hit = STAGE_RESUME_DIR.with(|c| {
                        c.borrow().as_ref().and_then(|dir| {
                            read_stage_checkpoint_bytes(dir, &stage_name)
                        })
                    });
                    if let Some(data) = resume_hit {
                        return Ok(VMValue::Str(String::from_utf8_lossy(&data).into_owned()));
                    }
                }
                Ok(VMValue::Bool(false))
            }

            // ── v20.4.0: DuckDB pushdown ──────────────────────────────────────────
            "__duckdb_push" => {
                // args: [rows: ArrowBatch | any, sql_template: Str, fallback_fn_idx: Int]
                let mut it = args.into_iter();
                let rows = it.next().unwrap_or(VMValue::Unit);
                let sql_template = match it.next() {
                    Some(VMValue::Str(s)) => s,
                    _ => return Err(self.error(artifact, "__duckdb_push: missing sql template")),
                };
                let fallback_fn_idx = match it.next() {
                    Some(VMValue::Int(n)) => n as usize,
                    _ => return Err(self.error(artifact, "__duckdb_push: missing fallback index")),
                };

                let rows_for_fallback = rows.clone();

                // Attempt DuckDB pushdown if input is ArrowBatch
                let pushdown_result = match &rows {
                    VMValue::ArrowBatch(id) => {
                        execute_duckdb_pushdown(*id, &sql_template)
                    }
                    _ => Err("not_arrow_batch".to_string()),
                };

                match pushdown_result {
                    Ok(result) => {
                        if PUSHDOWN_EXPLAIN_ENABLED.with(|c| c.get()) {
                            PUSHDOWN_LOG.with(|log| {
                                log.borrow_mut().push(format!(
                                    "[pushdown] SQL={} → OK",
                                    sql_template
                                ));
                            });
                        }
                        Ok(result)
                    }
                    Err(_) => {
                        // Fallback: call the original stage function directly
                        self.invoke_function(artifact, fallback_fn_idx, vec![rows_for_fallback])
                    }
                }
            }

            // ── v20.7.0: Arena アロケータ統計 ─────────────────────────────────────
            "Arena.stats" => {
                if !args.is_empty() {
                    return Err(self.error(artifact, "Arena.stats: expected 0 arguments"));
                }
                #[cfg(not(target_arch = "wasm32"))]
                {
                    let s = self.chunk_arena.stats();
                    let mut map = std::collections::HashMap::new();
                    map.insert("acquire_count".to_string(), VMValue::Int(s.acquire_count as i64));
                    map.insert("alloc_count".to_string(), VMValue::Int(s.alloc_count as i64));
                    map.insert("reset_count".to_string(), VMValue::Int(s.reset_count as i64));
                    map.insert("peak_capacity".to_string(), VMValue::Int(s.peak_capacity as i64));
                    Ok(ok_vm(VMValue::Record(map)))
                }
                #[cfg(target_arch = "wasm32")]
                {
                    Ok(err_vm(VMValue::Str(
                        "Arena.stats: not supported on wasm32".to_string(),
                    )))
                }
            }

            _ => {
                if let Some(target_idx) = artifact.globals.iter().position(|g| {
                    g.kind == 0
                        && artifact
                            .str_table
                            .get(g.name_idx as usize)
                            .is_some_and(|n| n == name)
                }) {
                    return self.call_value(
                        artifact,
                        VMValue::CompiledFn(artifact.globals[target_idx].fn_idx as usize),
                        args,
                    );
                }
                // vm_call_builtin works with Vec<VMValue> emit_log; bridge via temp vec
                let mut temp_log: Vec<VMValue> = Vec::new();
                let result = vm_call_builtin(
                    name,
                    args,
                    &mut temp_log,
                    self.db_path.as_deref(),
                    &self.type_metas,
                )
                .map_err(|e| self.error(artifact, &e));
                self.emit_log.extend(temp_log.into_iter().map(NanVal::from_vmvalue));
                result
            }
        }
    }

    /// Materialize a lazy `VMStream` into a `Vec<VMValue>`.
    fn materialize_stream(
        &mut self,
        artifact: &FvcArtifact,
        stream: VMStream,
    ) -> Result<Vec<VMValue>, VMError> {
        match stream {
            VMStream::Of(items) => Ok(items),
            VMStream::Gen { .. } => Err(self.error(
                artifact,
                "cannot collect an infinite stream without Stream.take",
            )),
            VMStream::Map { inner, map_fn } => {
                let items = self.materialize_stream(artifact, *inner)?;
                let mut out = Vec::with_capacity(items.len());
                for item in items {
                    out.push(self.call_value(artifact, map_fn.clone(), vec![item])?);
                }
                Ok(out)
            }
            VMStream::Filter { inner, pred_fn } => {
                let items = self.materialize_stream(artifact, *inner)?;
                let mut out = Vec::new();
                for item in items {
                    let keep = self.call_value(artifact, pred_fn.clone(), vec![item.clone()])?;
                    match keep {
                        VMValue::Bool(true) => out.push(item),
                        VMValue::Bool(false) => {}
                        other => {
                            return Err(self.error(
                                artifact,
                                &format!(
                                    "Stream.filter predicate must return Bool, got {}",
                                    vmvalue_type_name(&other)
                                ),
                            ));
                        }
                    }
                }
                Ok(out)
            }
            VMStream::Take { inner, n } => {
                let n_usize = if n < 0 { 0 } else { n as usize };
                match *inner {
                    VMStream::Gen { seed, next_fn } => {
                        let mut result = Vec::with_capacity(n_usize);
                        let mut current = seed;
                        for _ in 0..n_usize {
                            result.push(current.clone());
                            current = self.call_value(artifact, next_fn.clone(), vec![current])?;
                        }
                        Ok(result)
                    }
                    other => {
                        let items = self.materialize_stream(artifact, other)?;
                        Ok(items.into_iter().take(n_usize).collect())
                    }
                }
            }
            // ── v26.4.0 ────────────────────────────────────────────────────────────
            VMStream::FlatMap { inner, map_fn } => {
                let items = self.materialize_stream(artifact, *inner)?;
                let mut out = Vec::new();
                for item in items {
                    let result = self.call_value(artifact, map_fn.clone(), vec![item])?;
                    match result {
                        VMValue::List(list) => out.extend(list.iter().cloned()),
                        other => out.push(other),
                    }
                }
                Ok(out)
            }
            VMStream::Window { inner, size, window_fn } => {
                let items = self.materialize_stream(artifact, *inner)?;
                let chunk_size = if size <= 0 { 1 } else { size as usize };
                let mut out = Vec::new();
                for chunk in items.chunks(chunk_size) {
                    let batch = VMValue::List(FavList::new(chunk.to_vec()));
                    let result = self.call_value(artifact, window_fn.clone(), vec![batch])?;
                    out.push(result);
                }
                Ok(out)
            }
            VMStream::Merge { streams } => {
                let mut out = Vec::new();
                for s in streams {
                    let items = self.materialize_stream(artifact, s)?;
                    out.extend(items);
                }
                Ok(out)
            }
            // NOTE: VMStream::Split is intentionally absent here.
            // Stream.split is immediately materialized in the primitive itself,
            // so a VMStream::Split value is never passed to materialize_stream.
            VMStream::Split { inner, pred_fn } => {
                // Fallback: should not be reached via normal code paths.
                // Materialize defensively using the same logic as the primitive.
                let items = self.materialize_stream(artifact, *inner)?;
                let mut trues = Vec::new();
                let mut falses = Vec::new();
                for item in items {
                    let keep = self.call_value(artifact, pred_fn.clone(), vec![item.clone()])?;
                    match keep {
                        VMValue::Bool(true) => trues.push(item),
                        VMValue::Bool(false) => falses.push(item),
                        other => {
                            return Err(self.error(
                                artifact,
                                &format!(
                                    "Stream.split predicate must return Bool, got {}",
                                    vmvalue_type_name(&other)
                                ),
                            ));
                        }
                    }
                }
                Ok(vec![
                    VMValue::List(FavList::new(trues)),
                    VMValue::List(FavList::new(falses)),
                ])
            }
        }
    }

    fn pop_pair(&mut self, artifact: &FvcArtifact) -> Result<(NanVal, NanVal), VMError> {
        let right = self
            .stack
            .pop()
            .ok_or_else(|| self.error(artifact, "stack underflow"))?;
        let left = self
            .stack
            .pop()
            .ok_or_else(|| self.error(artifact, "stack underflow"))?;
        Ok((left, right))
    }
}

fn constant_to_value(constant: Constant) -> VMValue {
    match constant {
        Constant::Int(v) => VMValue::Int(v),
        Constant::Float(v) => VMValue::Float(v),
        Constant::Str(v) => VMValue::Str(v),
        Constant::Name(v) => VMValue::Str(v),
    }
}

/// NaN-boxing 版 Constant → NanVal 変換（T5）
fn constant_to_nan(constant: Constant) -> NanVal {
    match constant {
        Constant::Int(v)   => NanVal::from_int(v),
        Constant::Float(v) => NanVal::from_float(v),
        Constant::Str(v)   => NanVal::from_str(v),
        Constant::Name(v)  => NanVal::from_str(v),
    }
}

/// NaN-boxing 版数値二項演算（T5）
fn apply_numeric_binop_nan(
    left: NanVal,
    right: NanVal,
    int_op: impl FnOnce(i64, i64) -> i64,
    float_op: impl FnOnce(f64, f64) -> f64,
    op_name: &str,
    artifact: &FvcArtifact,
    frames: &[CallFrame],
) -> Result<NanVal, VMError> {
    match (left.as_int(), left.as_float(), right.as_int(), right.as_float()) {
        (Some(a), _, Some(b), _) => Ok(NanVal::from_int(int_op(a, b))),
        (_, Some(a), _, Some(b)) => Ok(NanVal::from_float(float_op(a, b))),
        (Some(a), _, _, Some(b)) => Ok(NanVal::from_float(float_op(a as f64, b))),
        (_, Some(a), Some(b), _) => Ok(NanVal::from_float(float_op(a, b as f64))),
        _ => Err(vm_error_from_frames(
            artifact,
            frames,
            format!("type error in {op_name}: numeric operands required"),
        )),
    }
}

/// NaN-boxing 版比較演算（T5）
fn compare_pair_nan(
    pair: (NanVal, NanVal),
    cmp: impl FnOnce(f64, f64) -> bool,
    artifact: &FvcArtifact,
    frames: &[CallFrame],
) -> Result<NanVal, VMError> {
    let (left, right) = pair;
    match (left.as_int(), left.as_float(), right.as_int(), right.as_float()) {
        (Some(a), _, Some(b), _) => Ok(NanVal::from_bool(cmp(a as f64, b as f64))),
        (_, Some(a), _, Some(b)) => Ok(NanVal::from_bool(cmp(a, b))),
        (Some(a), _, _, Some(b)) => Ok(NanVal::from_bool(cmp(a as f64, b))),
        (_, Some(a), Some(b), _) => Ok(NanVal::from_bool(cmp(a, b as f64))),
        _ => Err(vm_error_from_frames(
            artifact,
            frames,
            "type error in comparison: numeric operands required".to_string(),
        )),
    }
}

/// NanVal の型名文字列（T5）
fn nanval_type_name(v: &NanVal) -> &'static str {
    use super::heap_val::HeapVal;
    if v.is_float() { return "Float"; }
    if v.is_int()   { return "Int"; }
    if v.is_bool()  { return "Bool"; }
    if v.is_unit()  { return "Unit"; }
    if v.is_str()   { return "String"; }
    if v.is_list()  { return "List"; }
    if v.is_record() { return "Record"; }
    if v.is_heap() {
        if let Some(h) = v.as_heap() {
            return match h {
                HeapVal::Variant(_, _)  => "Variant",
                HeapVal::VariantCtor(_) => "VariantCtor",
                HeapVal::CompiledFn(_)  => "CompiledFn",
                HeapVal::Closure(_, _)  => "Closure",
                HeapVal::Builtin(_)     => "Builtin",
                HeapVal::Stream(_)      => "Stream",
                HeapVal::DbHandle(_)    => "DbHandle",
                HeapVal::TxHandle(_)    => "TxHandle",
                HeapVal::ArrowBatch(_)  => "ArrowBatch",
                HeapVal::PgPool(_)      => "PgPool",
                HeapVal::Bytes(_)       => "Bytes",
                HeapVal::MutList(_)     => "MutList",
                HeapVal::MutMap(_)      => "MutMap",
                HeapVal::BigInt(_)      => "Int",
            };
        }
    }
    "Unknown"
}

/// NanVal → Value 変換（テスト・CLI 出力用レガシーブリッジ）（T5）
pub fn vm_to_external_value(v: NanVal) -> Value {
    Value::from(v.to_vmvalue())
}

impl From<Value> for VMValue {
    fn from(value: Value) -> Self {
        match value {
            Value::Bool(v) => VMValue::Bool(v),
            Value::Int(v) => VMValue::Int(v),
            Value::Float(v) => VMValue::Float(v),
            Value::Str(v) => VMValue::Str(v),
            Value::Unit => VMValue::Unit,
            Value::List(values) => VMValue::List(FavList::new(
                values.into_iter().map(VMValue::from).collect(),
            )),
            Value::Record(map) => VMValue::Record(
                map.into_iter()
                    .map(|(k, v)| (k, VMValue::from(v)))
                    .collect(),
            ),
            Value::Variant(tag, payload) => {
                VMValue::Variant(tag, payload.map(|inner| Box::new(VMValue::from(*inner))))
            }
            other => panic!("unsupported VM argument value: {other:?}"),
        }
    }
}

impl From<VMValue> for Value {
    fn from(value: VMValue) -> Self {
        match value {
            VMValue::Bool(v) => Value::Bool(v),
            VMValue::Int(v) => Value::Int(v),
            VMValue::Float(v) => Value::Float(v),
            VMValue::Str(v) => Value::Str(v),
            VMValue::Unit => Value::Unit,
            VMValue::List(fl) => Value::List(fl.into_iter().map(Value::from).collect()),
            VMValue::Record(map) => {
                Value::Record(map.into_iter().map(|(k, v)| (k, Value::from(v))).collect())
            }
            VMValue::Variant(tag, payload) => {
                Value::Variant(tag, payload.map(|inner| Box::new(Value::from(*inner))))
            }
            VMValue::VariantCtor(name) => Value::Variant(name, None),
            VMValue::CompiledFn(idx) => Value::Str(format!("<fn:{idx}>")),
            VMValue::Closure(idx, captures) => {
                Value::Str(format!("<closure:{idx};captures={}>", captures.len()))
            }
            VMValue::Builtin(name) => Value::Str(format!("<builtin:{name}>")),
            VMValue::Stream(_) => Value::Str("<stream>".to_string()),
            VMValue::DbHandle(id) => Value::Str(format!("<db:{id}>")),
            VMValue::TxHandle(id) => Value::Str(format!("<tx:{id}>")),
            VMValue::ArrowBatch(id) => Value::Str(format!("<arrow:{id}>")),
            VMValue::PgPool(id) => Value::Str(format!("<pgpool:{id}>")),
            VMValue::Bytes(id)   => Value::Str(format!("<bytes:{id}>")),
            VMValue::MutList(id) => Value::Str(format!("<mut-list:{id}>")),
            VMValue::MutMap(id)  => Value::Str(format!("<mut-map:{id}>")),
        }
    }
}

fn apply_numeric_binop(
    left: VMValue,
    right: VMValue,
    int_op: impl FnOnce(i64, i64) -> i64,
    float_op: impl FnOnce(f64, f64) -> f64,
    op_name: &str,
    artifact: &FvcArtifact,
    frames: &[CallFrame],
) -> Result<VMValue, VMError> {
    match (left, right) {
        (VMValue::Int(a), VMValue::Int(b)) => Ok(VMValue::Int(int_op(a, b))),
        (VMValue::Float(a), VMValue::Float(b)) => Ok(VMValue::Float(float_op(a, b))),
        (VMValue::Int(a), VMValue::Float(b)) => Ok(VMValue::Float(float_op(a as f64, b))),
        (VMValue::Float(a), VMValue::Int(b)) => Ok(VMValue::Float(float_op(a, b as f64))),
        _ => Err(vm_error_from_frames(
            artifact,
            frames,
            format!("type error in {op_name}: numeric operands required"),
        )),
    }
}

fn compare_pair(
    pair: (VMValue, VMValue),
    cmp: impl FnOnce(f64, f64) -> bool,
    artifact: &FvcArtifact,
    frames: &[CallFrame],
) -> Result<VMValue, VMError> {
    match pair {
        (VMValue::Int(a), VMValue::Int(b)) => Ok(VMValue::Bool(cmp(a as f64, b as f64))),
        (VMValue::Float(a), VMValue::Float(b)) => Ok(VMValue::Bool(cmp(a, b))),
        (VMValue::Int(a), VMValue::Float(b)) => Ok(VMValue::Bool(cmp(a as f64, b))),
        (VMValue::Float(a), VMValue::Int(b)) => Ok(VMValue::Bool(cmp(a, b as f64))),
        _ => Err(vm_error_from_frames(
            artifact,
            frames,
            "type error in comparison: numeric operands required".to_string(),
        )),
    }
}

fn build_stack_trace(artifact: &FvcArtifact, frames: &[CallFrame]) -> Vec<TraceFrame> {
    frames
        .iter()
        .rev()
        .map(|frame| {
            let function = &artifact.functions[frame.fn_idx];
            let fn_name = artifact
                .str_table
                .get(function.name_idx as usize)
                .cloned()
                .unwrap_or_else(|| "<unknown>".to_string());
            TraceFrame {
                fn_name,
                line: frame.line,
            }
        })
        .collect()
}

fn vm_error_from_frames(artifact: &FvcArtifact, frames: &[CallFrame], message: String) -> VMError {
    let stack_trace = build_stack_trace(artifact, frames);
    if let Some(frame) = frames.last() {
        let top = stack_trace.first().cloned().unwrap_or(TraceFrame {
            fn_name: "<unknown>".to_string(),
            line: 0,
        });
        VMError {
            message,
            fn_name: top.fn_name,
            ip: frame.ip,
            stack_trace,
        }
    } else {
        VMError {
            message,
            fn_name: "<none>".to_string(),
            ip: 0,
            stack_trace,
        }
    }
}

fn vmvalue_repr(v: &VMValue) -> String {
    match v {
        VMValue::Bool(b) => b.to_string(),
        VMValue::Int(n) => n.to_string(),
        VMValue::Float(f) => {
            if f.fract() == 0.0 {
                format!("{:.1}", f)
            } else {
                f.to_string()
            }
        }
        VMValue::Str(s) => format!("\"{}\"", s),
        VMValue::Unit => "()".to_string(),
        VMValue::List(fl) => {
            let items: Vec<_> = fl.iter().map(vmvalue_repr).collect();
            format!("[{}]", items.join(", "))
        }
        VMValue::Record(m) => {
            let mut pairs: Vec<_> = m
                .iter()
                .map(|(k, v)| format!("{}: {}", k, vmvalue_repr(v)))
                .collect();
            pairs.sort();
            format!("{{ {} }}", pairs.join(", "))
        }
        VMValue::Variant(name, None) => name.clone(),
        VMValue::Variant(name, Some(payload)) => format!("{}({})", name, vmvalue_repr(payload)),
        VMValue::CompiledFn(idx) => format!("<fn:{}>", idx),
        VMValue::Closure(idx, caps) => format!("<closure:{};captures={}>", idx, caps.len()),
        VMValue::VariantCtor(name) => format!("<ctor:{}>", name),
        VMValue::Builtin(name) => format!("<builtin:{}>", name),
        VMValue::Stream(_) => "<stream>".to_string(),
        VMValue::DbHandle(id) => format!("<db:{}>", id),
        VMValue::TxHandle(id) => format!("<tx:{}>", id),
        VMValue::ArrowBatch(id) => format!("<arrow:{}>", id),
        VMValue::PgPool(id) => format!("<pgpool:{}>", id),
        VMValue::Bytes(id) => format!("<bytes:{}>", id),
        VMValue::MutList(id) => format!("<mut-list:{}>", id),
        VMValue::MutMap(id) => format!("<mut-map:{}>", id),
    }
}

fn vmvalue_type_name(v: &VMValue) -> &'static str {
    match v {
        VMValue::Bool(_) => "Bool",
        VMValue::Int(_) => "Int",
        VMValue::Float(_) => "Float",
        VMValue::Str(_) => "String",
        VMValue::Unit => "Unit",
        VMValue::List(_) => "List",
        VMValue::Record(_) => "Record",
        VMValue::Variant(_, _) => "Variant",
        VMValue::VariantCtor(_) => "VariantCtor",
        VMValue::CompiledFn(_) => "CompiledFn",
        VMValue::Closure(_, _) => "Closure",
        VMValue::Builtin(_) => "Builtin",
        VMValue::Stream(_) => "Stream",
        VMValue::DbHandle(_) => "DbHandle",
        VMValue::TxHandle(_) => "TxHandle",
        VMValue::ArrowBatch(_) => "ArrowBatch",
        VMValue::PgPool(_) => "PgPool",
        VMValue::Bytes(_)   => "Bytes",
        VMValue::MutList(_) => "MutList",
        VMValue::MutMap(_)  => "MutMap",
    }
}

fn json_variant_vm(name: &str, payload: Option<VMValue>) -> VMValue {
    VMValue::Variant(name.to_string(), payload.map(Box::new))
}

fn serde_to_vm_json(value: SerdeJsonValue) -> VMValue {
    match value {
        SerdeJsonValue::Null => json_variant_vm("json_null", None),
        SerdeJsonValue::Bool(b) => json_variant_vm("json_bool", Some(VMValue::Bool(b))),
        SerdeJsonValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                json_variant_vm("json_int", Some(VMValue::Int(i)))
            } else {
                json_variant_vm(
                    "json_float",
                    Some(VMValue::Float(n.as_f64().unwrap_or(0.0))),
                )
            }
        }
        SerdeJsonValue::String(s) => json_variant_vm("json_str", Some(VMValue::Str(s))),
        SerdeJsonValue::Array(items) => json_variant_vm(
            "json_array",
            Some(VMValue::List(FavList::new(
                items.into_iter().map(serde_to_vm_json).collect(),
            ))),
        ),
        SerdeJsonValue::Object(map) => {
            let mut fields = HashMap::new();
            for (k, v) in map {
                fields.insert(k, serde_to_vm_json(v));
            }
            json_variant_vm("json_object", Some(VMValue::Record(fields)))
        }
    }
}

fn vm_json_to_serde(value: &VMValue) -> Option<SerdeJsonValue> {
    match value {
        VMValue::Variant(tag, None) if tag == "json_null" => Some(SerdeJsonValue::Null),
        VMValue::Variant(tag, Some(payload)) if tag == "json_bool" => match payload.as_ref() {
            VMValue::Bool(b) => Some(SerdeJsonValue::Bool(*b)),
            _ => None,
        },
        VMValue::Variant(tag, Some(payload)) if tag == "json_int" => match payload.as_ref() {
            VMValue::Int(i) => Some(SerdeJsonValue::Number((*i).into())),
            _ => None,
        },
        VMValue::Variant(tag, Some(payload)) if tag == "json_float" => match payload.as_ref() {
            VMValue::Float(f) => serde_json::Number::from_f64(*f).map(SerdeJsonValue::Number),
            _ => None,
        },
        VMValue::Variant(tag, Some(payload)) if tag == "json_str" => match payload.as_ref() {
            VMValue::Str(s) => Some(SerdeJsonValue::String(s.clone())),
            _ => None,
        },
        VMValue::Variant(tag, Some(payload)) if tag == "json_array" => match payload.as_ref() {
            VMValue::List(fl) => {
                let mut out = Vec::with_capacity(fl.len());
                for item in fl.iter() {
                    out.push(vm_json_to_serde(item)?);
                }
                Some(SerdeJsonValue::Array(out))
            }
            _ => None,
        },
        VMValue::Variant(tag, Some(payload)) if tag == "json_object" => match payload.as_ref() {
            VMValue::Record(map) => {
                let mut out = serde_json::Map::new();
                for (k, v) in map {
                    out.insert(k.clone(), vm_json_to_serde(v)?);
                }
                Some(SerdeJsonValue::Object(out))
            }
            _ => None,
        },
        _ => None,
    }
}

fn vm_string(value: VMValue, context: &str) -> Result<String, String> {
    match value {
        VMValue::Str(s) => Ok(s),
        other => Err(format!(
            "{} expects String, got {}",
            context,
            vmvalue_type_name(&other)
        )),
    }
}

fn vm_int(value: VMValue, context: &str) -> Result<i64, String> {
    match value {
        VMValue::Int(n) => Ok(n),
        other => Err(format!(
            "{} expects Int, got {}",
            context,
            vmvalue_type_name(&other)
        )),
    }
}

fn vm_float(value: VMValue, context: &str) -> Result<f64, String> {
    match value {
        VMValue::Float(f) => Ok(f),
        VMValue::Int(n) => Ok(n as f64),
        other => Err(format!(
            "{} expects Float, got {}",
            context,
            vmvalue_type_name(&other)
        )),
    }
}

fn vm_string_list(value: VMValue, context: &str) -> Result<Vec<String>, String> {
    match value {
        VMValue::List(fl) => {
            let mut out = Vec::with_capacity(fl.len());
            for item in fl {
                out.push(vm_string(item, context)?);
            }
            Ok(out)
        }
        other => Err(format!(
            "{} expects List<String>, got {}",
            context,
            vmvalue_type_name(&other)
        )),
    }
}

fn schema_error_vm(
    field: impl Into<String>,
    expected: impl Into<String>,
    got: impl Into<String>,
) -> VMValue {
    let mut map = HashMap::new();
    map.insert("field".to_string(), VMValue::Str(field.into()));
    map.insert("expected".to_string(), VMValue::Str(expected.into()));
    map.insert("got".to_string(), VMValue::Str(got.into()));
    VMValue::Record(map)
}

fn ok_vm(value: VMValue) -> VMValue {
    VMValue::Variant("ok".to_string(), Some(Box::new(value)))
}

fn err_vm(value: VMValue) -> VMValue {
    VMValue::Variant("err".to_string(), Some(Box::new(value)))
}

// ── Kafka helpers (v15.4.0) ──────────────────────────────────────────────────

/// brokers 引数が空の場合は環境変数 KAFKA_BOOTSTRAP_BROKERS にフォールバック。
fn kafka_resolve_brokers(brokers_arg: &str) -> String {
    let b = brokers_arg.trim().to_string();
    if b.is_empty() {
        std::env::var("KAFKA_BOOTSTRAP_BROKERS").unwrap_or_else(|_| "localhost:9092".to_string())
    } else {
        b
    }
}

/// ブローカーアドレス一覧（カンマ区切り）を Vec<String> に変換。
fn kafka_broker_list(brokers: &str) -> Vec<String> {
    brokers.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()
}

/// Kafka topic に 1 メッセージを produce する（同期ラッパー）。
fn kafka_produce_sync(brokers: &str, topic: &str, key: &str, value: &str) -> Result<(), String> {
    use rskafka::client::ClientBuilder;
    use rskafka::record::Record;
    use rskafka::client::partition::{Compression, UnknownTopicHandling};

    let addrs = kafka_broker_list(brokers);
    if addrs.is_empty() {
        return Err("Kafka.produce_raw: no brokers specified".to_string());
    }
    let username = std::env::var("KAFKA_SASL_USERNAME").ok();
    let password = std::env::var("KAFKA_SASL_PASSWORD").ok();

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| format!("Kafka.produce_raw: tokio: {e}"))?;

    rt.block_on(async {
        let mut builder = ClientBuilder::new(addrs);
        if let (Some(user), Some(pass)) = (username, password) {
            builder = builder.sasl_config(
                rskafka::client::SaslConfig::ScramSha512(
                    rskafka::client::Credentials::new(user, pass)
                )
            );
        }
        let client = builder.build().await.map_err(|e| format!("Kafka.produce_raw: connect: {e}"))?;
        let partition_client = client
            .partition_client(topic, 0, UnknownTopicHandling::Retry)
            .await
            .map_err(|e| format!("Kafka.produce_raw: partition: {e}"))?;

        let record = Record {
            key:       Some(key.as_bytes().to_vec()),
            value:     Some(value.as_bytes().to_vec()),
            timestamp: chrono::Utc::now(),
            headers:   Default::default(),
        };
        partition_client
            .produce(vec![record], Compression::NoCompression)
            .await
            .map_err(|e| format!("Kafka.produce_raw: produce: {e}"))?;
        Ok(())
    })
}

/// Kafka topic から最新オフセットの 1 メッセージを consume する（同期ラッパー）。
fn kafka_consume_one_sync(brokers: &str, topic: &str) -> Result<String, String> {
    use rskafka::client::ClientBuilder;
    use rskafka::client::partition::{OffsetAt, UnknownTopicHandling};

    let addrs = kafka_broker_list(brokers);
    if addrs.is_empty() {
        return Err("Kafka.consume_one_raw: no brokers specified".to_string());
    }
    let username = std::env::var("KAFKA_SASL_USERNAME").ok();
    let password = std::env::var("KAFKA_SASL_PASSWORD").ok();

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| format!("Kafka.consume_one_raw: tokio: {e}"))?;

    rt.block_on(async {
        let mut builder = ClientBuilder::new(addrs);
        if let (Some(user), Some(pass)) = (username, password) {
            builder = builder.sasl_config(
                rskafka::client::SaslConfig::ScramSha512(
                    rskafka::client::Credentials::new(user, pass)
                )
            );
        }
        let client = builder.build().await.map_err(|e| format!("Kafka.consume_one_raw: connect: {e}"))?;
        let partition_client = client
            .partition_client(topic, 0, UnknownTopicHandling::Retry)
            .await
            .map_err(|e| format!("Kafka.consume_one_raw: partition: {e}"))?;

        let offset = partition_client
            .get_offset(OffsetAt::Latest)
            .await
            .map_err(|e| format!("Kafka.consume_one_raw: offset: {e}"))?;

        if offset == 0 {
            return Err("Kafka.consume_one_raw: topic is empty".to_string());
        }
        let fetch_offset = (offset - 1).max(0);

        let (records, _) = partition_client
            .fetch_records(fetch_offset, 1..1_048_576, 5_000)
            .await
            .map_err(|e| format!("Kafka.consume_one_raw: fetch: {e}"))?;

        let record = records.into_iter().next()
            .ok_or_else(|| "Kafka.consume_one_raw: no records returned".to_string())?;

        let payload = record.record.value.unwrap_or_default();
        String::from_utf8(payload).map_err(|e| format!("Kafka.consume_one_raw: utf8: {e}"))
    })
}

// ── Kafka helpers (v25.7.0) ───────────────────────────────────────────────────
// TODO(v26.x): コネクションプール（現在は毎回接続確立）

/// Kafka ブローカーへの接続確認（list_topics ping）。
fn kafka_connect_sync(brokers: &str) -> Result<(), String> {
    use rskafka::client::ClientBuilder;

    let addrs = kafka_broker_list(brokers);
    if addrs.is_empty() {
        return Err("Kafka.connect_raw: no brokers specified".to_string());
    }
    let username = std::env::var("KAFKA_SASL_USERNAME").ok();
    let password = std::env::var("KAFKA_SASL_PASSWORD").ok();
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| format!("Kafka.connect_raw: tokio: {e}"))?;
    rt.block_on(async {
        let mut builder = ClientBuilder::new(addrs);
        if let (Some(user), Some(pass)) = (username, password) {
            builder = builder.sasl_config(
                rskafka::client::SaslConfig::ScramSha512(
                    rskafka::client::Credentials::new(user, pass),
                ),
            );
        }
        let client = builder
            .build()
            .await
            .map_err(|e| format!("Kafka.connect_raw: connect: {e}"))?;
        let _ = client
            .list_topics()
            .await
            .map_err(|e| format!("Kafka.connect_raw: list_topics: {e}"))?;
        Ok(())
    })
}

/// Kafka topic から最大 max_count 件のメッセージを JSON 配列文字列で返す。
fn kafka_consume_batch_sync(brokers: &str, topic: &str, max_count: i64) -> Result<String, String> {
    use rskafka::client::ClientBuilder;
    use rskafka::client::partition::{OffsetAt, UnknownTopicHandling};

    let addrs = kafka_broker_list(brokers);
    if addrs.is_empty() {
        return Err("Kafka.consume_batch_raw: no brokers specified".to_string());
    }
    let username = std::env::var("KAFKA_SASL_USERNAME").ok();
    let password = std::env::var("KAFKA_SASL_PASSWORD").ok();
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| format!("Kafka.consume_batch_raw: tokio: {e}"))?;
    rt.block_on(async {
        let mut builder = ClientBuilder::new(addrs);
        if let (Some(user), Some(pass)) = (username, password) {
            builder = builder.sasl_config(
                rskafka::client::SaslConfig::ScramSha512(
                    rskafka::client::Credentials::new(user, pass),
                ),
            );
        }
        let client = builder
            .build()
            .await
            .map_err(|e| format!("Kafka.consume_batch_raw: connect: {e}"))?;
        let partition_client = client
            .partition_client(topic, 0, UnknownTopicHandling::Retry)
            .await
            .map_err(|e| format!("Kafka.consume_batch_raw: partition: {e}"))?;
        let latest_offset = partition_client
            .get_offset(OffsetAt::Latest)
            .await
            .map_err(|e| format!("Kafka.consume_batch_raw: offset: {e}"))?;
        if latest_offset == 0 {
            return Ok("[]".to_string());
        }
        if max_count <= 0 {
            return Ok("[]".to_string());
        }
        let count = max_count;
        let start_offset = (latest_offset - count).max(0);
        // TODO(v26.x): max_bytes を (count * 64 * 1024).max(1_048_576) に変更してバッチ容量を動的調整する
        let (records, _) = partition_client
            .fetch_records(start_offset, 1..1_048_576, 5_000)
            .await
            .map_err(|e| format!("Kafka.consume_batch_raw: fetch: {e}"))?;
        let payloads: Vec<serde_json::Value> = records
            .into_iter()
            .take(count as usize)
            .map(|r| {
                let bytes = r.record.value.unwrap_or_default();
                serde_json::Value::String(String::from_utf8_lossy(&bytes).to_string())
            })
            .collect();
        serde_json::to_string(&payloads)
            .map_err(|e| format!("Kafka.consume_batch_raw: serialize: {e}"))
    })
}

/// Kafka トピックを作成する（partition 数指定）。
fn kafka_create_topic_sync(brokers: &str, topic: &str, partitions: i32) -> Result<(), String> {
    use rskafka::client::ClientBuilder;

    let addrs = kafka_broker_list(brokers);
    if addrs.is_empty() {
        return Err("Kafka.create_topic_raw: no brokers specified".to_string());
    }
    let username = std::env::var("KAFKA_SASL_USERNAME").ok();
    let password = std::env::var("KAFKA_SASL_PASSWORD").ok();
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| format!("Kafka.create_topic_raw: tokio: {e}"))?;
    rt.block_on(async {
        let mut builder = ClientBuilder::new(addrs);
        if let (Some(user), Some(pass)) = (username, password) {
            builder = builder.sasl_config(
                rskafka::client::SaslConfig::ScramSha512(
                    rskafka::client::Credentials::new(user, pass),
                ),
            );
        }
        let client = builder
            .build()
            .await
            .map_err(|e| format!("Kafka.create_topic_raw: connect: {e}"))?;
        // controller_client() は rskafka v0.6 で同期関数（.await 不要）
        let controller_client = client
            .controller_client()
            .map_err(|e| format!("Kafka.create_topic_raw: controller: {e}"))?;
        controller_client
            .create_topic(topic, partitions, 1_i16, 5_000)
            .await
            .map_err(|e| format!("Kafka.create_topic_raw: topic={}: {e}", topic))?;
        Ok(())
    })
}

// ── GCP helper (v15.2.0) ─────────────────────────────────────────────────────

/// GOOGLE_APPLICATION_CREDENTIALS のサービスアカウント JSON から OAuth2 Bearer token を取得する。
fn gcp_get_access_token() -> Result<String, String> {
    use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
    use std::time::{SystemTime, UNIX_EPOCH};

    let cred_path = std::env::var("GOOGLE_APPLICATION_CREDENTIALS")
        .map_err(|_| "GOOGLE_APPLICATION_CREDENTIALS not set".to_string())?;
    let cred_json = std::fs::read_to_string(&cred_path)
        .map_err(|e| format!("gcp: failed to read credentials file '{cred_path}': {e}"))?;
    let cred: serde_json::Value = serde_json::from_str(&cred_json)
        .map_err(|e| format!("gcp: invalid credentials JSON: {e}"))?;

    let client_email = cred["client_email"].as_str()
        .ok_or_else(|| "gcp: missing client_email in credentials".to_string())?;
    let private_key = cred["private_key"].as_str()
        .ok_or_else(|| "gcp: missing private_key in credentials".to_string())?;

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    #[derive(serde::Serialize)]
    struct GcpClaims<'a> {
        iss: &'a str,
        scope: &'a str,
        aud: &'a str,
        iat: u64,
        exp: u64,
    }

    let claims = GcpClaims {
        iss: client_email,
        scope: "https://www.googleapis.com/auth/bigquery",
        aud: "https://oauth2.googleapis.com/token",
        iat: now,
        exp: now + 3600,
    };

    let header = Header::new(Algorithm::RS256);
    let key = EncodingKey::from_rsa_pem(private_key.as_bytes())
        .map_err(|e| format!("gcp: invalid private key: {e}"))?;
    let jwt = encode(&header, &claims, &key)
        .map_err(|e| format!("gcp: JWT encode failed: {e}"))?;

    let resp = ureq::post("https://oauth2.googleapis.com/token")
        .set("Content-Type", "application/x-www-form-urlencoded")
        .send_string(&format!(
            "grant_type=urn%3Aietf%3Aparams%3Aoauth%3Agrant-type%3Ajwt-bearer&assertion={}",
            jwt
        ))
        .map_err(|e| format!("gcp: token request failed: {e}"))?;

    let resp_body = resp.into_string()
        .map_err(|e| format!("gcp: token response read failed: {e}"))?;
    let token_json: serde_json::Value = serde_json::from_str(&resp_body)
        .map_err(|e| format!("gcp: token response parse failed: {e}"))?;

    token_json["access_token"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| format!("gcp: no access_token in response: {resp_body}"))
}

// ── Debug.show_raw helper (v9.10.0) ──────────────────────────────────────────

fn capitalize_variant_tag(tag: &str) -> String {
    let mut chars = tag.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().to_string() + chars.as_str(),
    }
}

// ── verbose trace helpers (v12.5.0) ──────────────────────────────────────────

/// Emit a trace line to stderr and collect it in `trace_lines`.
/// Standalone (not a method) to avoid borrow conflicts with `frame`.
/// v22.7.0: VMValue の要素数を返す（OTel output_items 用）。
#[cfg(not(target_arch = "wasm32"))]
fn otel_value_items(v: &VMValue) -> u64 {
    match v {
        VMValue::List(fl) => (fl.0.len().saturating_sub(fl.1)) as u64,
        VMValue::Str(s)   => s.len() as u64,
        _                 => 1,
    }
}

fn trace_emit(trace_lines: &mut Vec<String>, msg: String) {
    eprintln!("{}", msg);
    trace_lines.push(msg);
}

/// Format a VMValue for verbose trace, truncating at 200 chars in verbose mode.
fn truncate_for_trace(val: &VMValue, level: u8) -> String {
    let s = display_vmvalue(val);
    let max = if level >= 2 { usize::MAX } else { 200 };
    if s.len() > max {
        format!("{}[{} chars]", &s[..max], s.len())
    } else {
        s
    }
}

fn display_vmvalue(v: &VMValue) -> String {
    match v {
        VMValue::Int(n) => n.to_string(),
        VMValue::Float(f) => {
            if f.fract() == 0.0 && f.abs() < 1e15 {
                format!("{:.1}", f)
            } else {
                format!("{}", f)
            }
        }
        VMValue::Str(s) => format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"")),
        VMValue::Bool(b) => b.to_string(),
        VMValue::Unit => "()".to_string(),
        VMValue::List(xs) => {
            let items: Vec<String> = xs.iter().map(display_vmvalue).collect();
            format!("[{}]", items.join(", "))
        }
        VMValue::Record(fields) => {
            let mut pairs: Vec<(&String, &VMValue)> = fields.iter().collect();
            pairs.sort_by_key(|(k, _)| k.as_str());
            let parts: Vec<String> = pairs.iter()
                .map(|(k, v)| format!("{}: {}", k, display_vmvalue(v)))
                .collect();
            format!("{{{}}}", parts.join(", "))
        }
        VMValue::Variant(tag, None) => capitalize_variant_tag(tag),
        VMValue::Variant(tag, Some(inner)) => {
            format!("{}({})", capitalize_variant_tag(tag), display_vmvalue(inner))
        }
        VMValue::VariantCtor(tag) => capitalize_variant_tag(tag),
        VMValue::CompiledFn(_) | VMValue::Closure(_, _) | VMValue::Builtin(_) => "<fn>".to_string(),
        VMValue::Stream(_) => "<stream>".to_string(),
        VMValue::DbHandle(_) => "<db>".to_string(),
        VMValue::TxHandle(_) => "<tx>".to_string(),
        VMValue::ArrowBatch(_) => "<arrow>".to_string(),
        VMValue::PgPool(_) => "<pgpool>".to_string(),
        VMValue::Bytes(_)   => "<bytes>".to_string(),
        VMValue::MutList(_) => "<mut-list>".to_string(),
        VMValue::MutMap(_)  => "<mut-map>".to_string(),
    }
}

// ── LLM helpers (v9.6.0) ─────────────────────────────────────────────────────

/// Call the LLM provider (Anthropic or OpenAI) with a single user prompt.
/// Reads LLM_PROVIDER (default "anthropic"), LLM_MODEL, and the provider API key
/// from the environment. Returns ok_vm(text) or err_vm(msg).
#[cfg(not(target_arch = "wasm32"))]
fn llm_call_complete(prompt: &str) -> VMValue {
    let provider = std::env::var("LLM_PROVIDER").unwrap_or_else(|_| "anthropic".to_string());
    match provider.as_str() {
        "openai" => {
            let api_key = match std::env::var("OPENAI_API_KEY") {
                Ok(k) => k,
                Err(_) => return err_vm(VMValue::Str("OPENAI_API_KEY is not set".to_string())),
            };
            let model = std::env::var("LLM_MODEL").unwrap_or_else(|_| "gpt-4o".to_string());
            let body = serde_json::json!({
                "model": model,
                "messages": [{"role": "user", "content": prompt}]
            });
            match ureq::post("https://api.openai.com/v1/chat/completions")
                .set("Authorization", &format!("Bearer {}", api_key))
                .set("Content-Type", "application/json")
                .send_string(&body.to_string())
            {
                Ok(resp) => {
                    let text = match resp.into_string() {
                        Ok(t) => t,
                        Err(e) => return err_vm(VMValue::Str(e.to_string())),
                    };
                    match serde_json::from_str::<serde_json::Value>(&text) {
                        Ok(v) => {
                            let content = v["choices"][0]["message"]["content"]
                                .as_str()
                                .unwrap_or("")
                                .to_string();
                            ok_vm(VMValue::Str(content))
                        }
                        Err(e) => err_vm(VMValue::Str(e.to_string())),
                    }
                }
                Err(ureq::Error::Status(_, resp)) => {
                    let msg = resp.into_string().unwrap_or_default();
                    err_vm(VMValue::Str(msg))
                }
                Err(ureq::Error::Transport(e)) => err_vm(VMValue::Str(e.to_string())),
            }
        }
        _ => {
            // Default: anthropic
            let api_key = match std::env::var("ANTHROPIC_API_KEY") {
                Ok(k) => k,
                Err(_) => {
                    return err_vm(VMValue::Str("ANTHROPIC_API_KEY is not set".to_string()))
                }
            };
            let model =
                std::env::var("LLM_MODEL").unwrap_or_else(|_| "claude-opus-4-6".to_string());
            let body = serde_json::json!({
                "model": model,
                "max_tokens": 4096,
                "messages": [{"role": "user", "content": prompt}]
            });
            match ureq::post("https://api.anthropic.com/v1/messages")
                .set("x-api-key", &api_key)
                .set("anthropic-version", "2023-06-01")
                .set("Content-Type", "application/json")
                .send_string(&body.to_string())
            {
                Ok(resp) => {
                    let text = match resp.into_string() {
                        Ok(t) => t,
                        Err(e) => return err_vm(VMValue::Str(e.to_string())),
                    };
                    match serde_json::from_str::<serde_json::Value>(&text) {
                        Ok(v) => {
                            let content = v["content"][0]["text"]
                                .as_str()
                                .unwrap_or("")
                                .to_string();
                            ok_vm(VMValue::Str(content))
                        }
                        Err(e) => err_vm(VMValue::Str(e.to_string())),
                    }
                }
                Err(ureq::Error::Status(_, resp)) => {
                    let msg = resp.into_string().unwrap_or_default();
                    err_vm(VMValue::Str(msg))
                }
                Err(ureq::Error::Transport(e)) => err_vm(VMValue::Str(e.to_string())),
            }
        }
    }
}

/// Call the LLM provider with a JSON-encoded messages array.
#[cfg(not(target_arch = "wasm32"))]
fn llm_call_chat(messages_json: &str) -> VMValue {
    let messages: serde_json::Value = match serde_json::from_str(messages_json) {
        Ok(v) => v,
        Err(e) => {
            return err_vm(VMValue::Str(format!(
                "Llm.chat_raw: invalid messages JSON: {}",
                e
            )))
        }
    };
    let provider = std::env::var("LLM_PROVIDER").unwrap_or_else(|_| "anthropic".to_string());
    match provider.as_str() {
        "openai" => {
            let api_key = match std::env::var("OPENAI_API_KEY") {
                Ok(k) => k,
                Err(_) => return err_vm(VMValue::Str("OPENAI_API_KEY is not set".to_string())),
            };
            let model = std::env::var("LLM_MODEL").unwrap_or_else(|_| "gpt-4o".to_string());
            let body = serde_json::json!({
                "model": model,
                "messages": messages
            });
            match ureq::post("https://api.openai.com/v1/chat/completions")
                .set("Authorization", &format!("Bearer {}", api_key))
                .set("Content-Type", "application/json")
                .send_string(&body.to_string())
            {
                Ok(resp) => {
                    let text = match resp.into_string() {
                        Ok(t) => t,
                        Err(e) => return err_vm(VMValue::Str(e.to_string())),
                    };
                    match serde_json::from_str::<serde_json::Value>(&text) {
                        Ok(v) => {
                            let content = v["choices"][0]["message"]["content"]
                                .as_str()
                                .unwrap_or("")
                                .to_string();
                            ok_vm(VMValue::Str(content))
                        }
                        Err(e) => err_vm(VMValue::Str(e.to_string())),
                    }
                }
                Err(ureq::Error::Status(_, resp)) => {
                    let msg = resp.into_string().unwrap_or_default();
                    err_vm(VMValue::Str(msg))
                }
                Err(ureq::Error::Transport(e)) => err_vm(VMValue::Str(e.to_string())),
            }
        }
        _ => {
            let api_key = match std::env::var("ANTHROPIC_API_KEY") {
                Ok(k) => k,
                Err(_) => {
                    return err_vm(VMValue::Str("ANTHROPIC_API_KEY is not set".to_string()))
                }
            };
            let model =
                std::env::var("LLM_MODEL").unwrap_or_else(|_| "claude-opus-4-6".to_string());
            let body = serde_json::json!({
                "model": model,
                "max_tokens": 4096,
                "messages": messages
            });
            match ureq::post("https://api.anthropic.com/v1/messages")
                .set("x-api-key", &api_key)
                .set("anthropic-version", "2023-06-01")
                .set("Content-Type", "application/json")
                .send_string(&body.to_string())
            {
                Ok(resp) => {
                    let text = match resp.into_string() {
                        Ok(t) => t,
                        Err(e) => return err_vm(VMValue::Str(e.to_string())),
                    };
                    match serde_json::from_str::<serde_json::Value>(&text) {
                        Ok(v) => {
                            let content = v["content"][0]["text"]
                                .as_str()
                                .unwrap_or("")
                                .to_string();
                            ok_vm(VMValue::Str(content))
                        }
                        Err(e) => err_vm(VMValue::Str(e.to_string())),
                    }
                }
                Err(ureq::Error::Status(_, resp)) => {
                    let msg = resp.into_string().unwrap_or_default();
                    err_vm(VMValue::Str(msg))
                }
                Err(ureq::Error::Transport(e)) => err_vm(VMValue::Str(e.to_string())),
            }
        }
    }
}

// ── Snowflake helpers (v10.2.0) ──────────────────────────────────────────────

// ── Postgres helpers (v11.5.0 / v12.6.0) ────────────────────────────────────

// ── MySQL helpers (v25.4.0) ───────────────────────────────────────────────────

// ── MongoDB helpers (v25.5.0) ─────────────────────────────────────────────────

/// URL からデータベース名を抽出する。
/// "mongodb://host:port/dbname"             → "dbname"
/// "mongodb://user:pass@host:port/dbname"   → "dbname"（認証情報をスキップ）
/// "mongodb://host:port"                    → "test"（パスなし時のデフォルト）
#[cfg(not(target_arch = "wasm32"))]
fn extract_mongo_db_name(url: &str) -> String {
    let after_scheme = url
        .strip_prefix("mongodb://")
        .or_else(|| url.strip_prefix("mongodb+srv://"))
        .unwrap_or(url);
    // "user:pass@host:port/dbname" → rfind('@') で認証情報をスキップ
    let after_auth = match after_scheme.rfind('@') {
        Some(pos) => &after_scheme[pos + 1..],
        None => after_scheme,
    };
    if let Some(slash_pos) = after_auth.find('/') {
        let db_part = &after_auth[slash_pos + 1..];
        let db_name = db_part.split('?').next().unwrap_or(db_part);
        if !db_name.is_empty() {
            return db_name.to_string();
        }
    }
    "test".to_string()
}

/// BSON Document → serde_json::Value（ObjectId は {"$oid": "..."} 形式に変換）
#[cfg(not(target_arch = "wasm32"))]
fn mongo_bson_to_json(doc: mongodb::bson::Document) -> serde_json::Value {
    fn bson_val_to_json(b: mongodb::bson::Bson) -> serde_json::Value {
        match b {
            mongodb::bson::Bson::ObjectId(oid) => {
                serde_json::json!({"$oid": oid.to_hex()})
            }
            mongodb::bson::Bson::Document(d) => {
                let map: serde_json::Map<String, serde_json::Value> = d
                    .into_iter()
                    .map(|(k, v)| (k, bson_val_to_json(v)))
                    .collect();
                serde_json::Value::Object(map)
            }
            mongodb::bson::Bson::Array(arr) => {
                serde_json::Value::Array(arr.into_iter().map(bson_val_to_json).collect())
            }
            mongodb::bson::Bson::String(s) => serde_json::Value::String(s),
            mongodb::bson::Bson::Boolean(b) => serde_json::Value::Bool(b),
            mongodb::bson::Bson::Int32(i) => serde_json::Value::Number(i.into()),
            mongodb::bson::Bson::Int64(i) => serde_json::Value::Number(i.into()),
            mongodb::bson::Bson::Double(f) => {
                serde_json::Number::from_f64(f)
                    .map(serde_json::Value::Number)
                    .unwrap_or(serde_json::Value::Null)
            }
            mongodb::bson::Bson::Null => serde_json::Value::Null,
            // DateTime / Timestamp / Binary / Decimal128 等は Extended JSON 形式で返る
            // （例: DateTime → {"$date": ...}）。シリアライズ失敗時は Null を返す。
            other => serde_json::to_value(&other).unwrap_or(serde_json::Value::Null),
        }
    }
    bson_val_to_json(mongodb::bson::Bson::Document(doc))
}

/// JSON 文字列 → BSON Document
#[cfg(not(target_arch = "wasm32"))]
fn mongo_json_to_bson(v: &str) -> Result<mongodb::bson::Document, String> {
    let json: serde_json::Value = serde_json::from_str(v)
        .map_err(|e| format!("MongoDB JSON parse error: {}", e))?;
    mongodb::bson::to_document(&json)
        .map_err(|e| format!("MongoDB BSON conversion error: {}", e))
}

// ── DynamoDB helpers (v25.6.0) ────────────────────────────────────────────────
// TODO(v26.x): コネクションプール（現在は毎回 ListTables ping / HTTP リクエスト）

/// DynamoConn 文字列（endpoint）から実際の DynamoDB endpoint URL を決定する。
/// - 空または "default" → config.endpoint_url → LOCALSTACK_ENDPOINT → AWS 本番
/// - それ以外 → 文字列をそのまま使用
#[cfg(not(target_arch = "wasm32"))]
fn get_dynamo_endpoint(conn_endpoint: &str, config: &AwsConfig) -> String {
    if conn_endpoint.is_empty() || conn_endpoint == "default" {
        config.endpoint_url.as_deref()
            .map(|s| s.to_string())
            .or_else(|| std::env::var("LOCALSTACK_ENDPOINT").ok())
            .unwrap_or_else(|| format!("https://dynamodb.{}.amazonaws.com", config.region))
    } else {
        conn_endpoint.to_string()
    }
}

// ── Elasticsearch helpers (v25.8.0) ─────────────────────────────────────────

/// ESConn url string → 実 ES エンドポイント URL（末尾 `/` なし）
/// 空文字列 → ELASTICSEARCH_URL env → "http://localhost:9200"
#[cfg(not(target_arch = "wasm32"))]
fn get_es_url(conn_url: &str) -> String {
    if conn_url.is_empty() || conn_url == "default" {
        std::env::var("ELASTICSEARCH_URL")
            .unwrap_or_else(|_| "http://localhost:9200".to_string())
    } else {
        conn_url.trim_end_matches('/').to_string()
    }
}

/// ES HTTP リクエスト（ELASTICSEARCH_API_KEY → Basic → no-auth）
#[cfg(not(target_arch = "wasm32"))]
fn es_http(method: &str, url: &str, content_type: Option<&str>, body: Option<&str>) -> Result<String, String> {
    let mut req = ureq::request(method, url);
    if let Ok(key) = std::env::var("ELASTICSEARCH_API_KEY") {
        if !key.is_empty() {
            req = req.set("Authorization", &format!("ApiKey {}", key));
        }
    } else if let (Ok(user), Ok(pass)) = (std::env::var("ELASTICSEARCH_USERNAME"), std::env::var("ELASTICSEARCH_PASSWORD")) {
        if !user.is_empty() {
            let encoded = BASE64.encode(format!("{}:{}", user, pass));
            req = req.set("Authorization", &format!("Basic {}", encoded));
        }
    }
    if let Some(ct) = content_type {
        req = req.set("Content-Type", ct);
    }
    let resp = if let Some(b) = body {
        req.send_string(b)
    } else {
        req.call()
    };
    match resp {
        Ok(r) => r.into_string().map_err(|e| format!("es_http: read body: {}", e)),
        Err(ureq::Error::Status(code, r)) => {
            let msg = r.into_string().unwrap_or_default();
            Err(format!("es_http: HTTP {}: {}", code, msg))
        }
        Err(e) => Err(format!("es_http: {}", e)),
    }
}

/// ES bulk NDJSON リクエスト（Content-Type: application/x-ndjson）
#[cfg(not(target_arch = "wasm32"))]
fn es_http_ndjson(method: &str, url: &str, ndjson: &str) -> Result<String, String> {
    es_http(method, url, Some("application/x-ndjson"), Some(ndjson))
}

/// プレーン JSON Value → DynamoDB 属性型 JSON Value
/// String  → {"S": "val"}
/// Number  → {"N": "1.0"}
/// Boolean → {"BOOL": true}
/// Null    → {"NULL": true}
/// Array   → {"L": [...]}
/// Object  → {"M": {...}}
#[cfg(not(target_arch = "wasm32"))]
fn json_val_to_dynamo_attr(v: &serde_json::Value) -> serde_json::Value {
    match v {
        serde_json::Value::String(s) => serde_json::json!({"S": s}),
        serde_json::Value::Number(n) => serde_json::json!({"N": n.to_string()}),
        serde_json::Value::Bool(b) => serde_json::json!({"BOOL": b}),
        serde_json::Value::Null => serde_json::json!({"NULL": true}),
        serde_json::Value::Array(arr) => {
            let items: Vec<serde_json::Value> = arr.iter().map(json_val_to_dynamo_attr).collect();
            serde_json::json!({"L": items})
        }
        serde_json::Value::Object(obj) => {
            let mut m = serde_json::Map::new();
            for (k, val) in obj {
                m.insert(k.clone(), json_val_to_dynamo_attr(val));
            }
            serde_json::json!({"M": m})
        }
    }
}

/// プレーン JSON Object → DynamoDB Item (属性型 JSON Map)
/// {"pk": "user1", "ttl": 1700} → {"pk":{"S":"user1"},"ttl":{"N":"1700"}}
#[cfg(not(target_arch = "wasm32"))]
fn json_to_dynamo_item(v: &serde_json::Value) -> Result<serde_json::Map<String, serde_json::Value>, String> {
    let obj = v.as_object().ok_or_else(|| "DynamoDB: expected JSON object for item/key".to_string())?;
    let mut item = serde_json::Map::new();
    for (k, val) in obj {
        item.insert(k.clone(), json_val_to_dynamo_attr(val));
    }
    Ok(item)
}

/// DynamoDB 属性値 JSON → プレーン JSON Value
/// {"S": "user1"} → "user1"
/// {"N": "1700"}  → 1700
#[cfg(not(target_arch = "wasm32"))]
fn dynamo_attr_to_json(v: &serde_json::Value) -> serde_json::Value {
    if let Some(obj) = v.as_object() {
        if let Some(s) = obj.get("S").and_then(|x| x.as_str()) {
            return serde_json::Value::String(s.to_string());
        }
        if let Some(n) = obj.get("N").and_then(|x| x.as_str()) {
            if let Ok(i) = n.parse::<i64>() {
                return serde_json::json!(i);
            }
            if let Ok(f) = n.parse::<f64>() {
                return serde_json::Number::from_f64(f)
                    .map(serde_json::Value::Number)
                    .unwrap_or_else(|| serde_json::Value::String(n.to_string()));
            }
        }
        if let Some(b) = obj.get("BOOL").and_then(|x| x.as_bool()) {
            return serde_json::Value::Bool(b);
        }
        if obj.get("NULL").is_some() {
            return serde_json::Value::Null;
        }
        if let Some(arr) = obj.get("L").and_then(|x| x.as_array()) {
            return serde_json::Value::Array(arr.iter().map(dynamo_attr_to_json).collect());
        }
        if let Some(m) = obj.get("M").and_then(|x| x.as_object()) {
            let mut out = serde_json::Map::new();
            for (k, val) in m {
                out.insert(k.clone(), dynamo_attr_to_json(val));
            }
            return serde_json::Value::Object(out);
        }
        // 既知の未対応型: SS（String Set）/ NS（Number Set）/ BS（Binary Set）
        // これらが含まれるアイテムでは属性型 JSON がそのまま返される。
        // TODO(v26.x): SS/NS/BS を Vec<String> / Vec<Number> に変換する
    }
    v.clone()
}

/// DynamoDB Item (属性型 JSON オブジェクト) → プレーン JSON Value
#[cfg(not(target_arch = "wasm32"))]
fn dynamo_item_to_plain_json(item: &serde_json::Value) -> serde_json::Value {
    if let Some(obj) = item.as_object() {
        let mut out = serde_json::Map::new();
        for (k, v) in obj {
            out.insert(k.clone(), dynamo_attr_to_json(v));
        }
        serde_json::Value::Object(out)
    } else {
        item.clone()
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn json_to_mysql_value(v: serde_json::Value) -> mysql::Value {
    match v {
        serde_json::Value::Null => mysql::Value::NULL,
        serde_json::Value::Bool(b) => mysql::Value::Int(if b { 1 } else { 0 }),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                mysql::Value::Int(i)
            } else if let Some(f) = n.as_f64() {
                // Float(f32) ではなく Double(f64) を使用して精度損失を防ぐ
                mysql::Value::Double(f)
            } else {
                mysql::Value::Bytes(n.to_string().into_bytes())
            }
        }
        serde_json::Value::String(s) => mysql::Value::Bytes(s.into_bytes()),
        _ => mysql::Value::Bytes(v.to_string().into_bytes()),
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn mysql_value_to_json(v: mysql::Value) -> serde_json::Value {
    match v {
        mysql::Value::NULL => serde_json::Value::Null,
        mysql::Value::Bytes(b) => {
            serde_json::Value::String(String::from_utf8_lossy(&b).into_owned())
        }
        mysql::Value::Int(i) => serde_json::Value::Number(i.into()),
        mysql::Value::UInt(u) => serde_json::Value::Number(u.into()),
        mysql::Value::Float(f) => {
            serde_json::Number::from_f64(f as f64)
                .map(serde_json::Value::Number)
                .unwrap_or(serde_json::Value::Null)
        }
        mysql::Value::Double(d) => {
            serde_json::Number::from_f64(d)
                .map(serde_json::Value::Number)
                .unwrap_or(serde_json::Value::Null)
        }
        other => serde_json::Value::String(format!("{:?}", other)),
    }
}

// Format a tokio_postgres error with full DbError detail (v12.6.0).
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn format_pg_error_pub(e: &tokio_postgres::Error) -> String {
    format_pg_error(e)
}

#[cfg(not(target_arch = "wasm32"))]
fn format_pg_error(e: &tokio_postgres::Error) -> String {
    if let Some(db_err) = e.as_db_error() {
        let mut msg = format!("db error: {}", db_err.message());
        let code = db_err.code().code();
        if !code.is_empty() {
            msg.push_str(&format!(" (SQLSTATE {})", code));
        }
        if let Some(detail) = db_err.detail() {
            msg.push_str(&format!(", detail: {}", detail));
        }
        msg
    } else {
        format!("db error: {}", e)
    }
}

// Resolve sslmode: URL ?sslmode= > libpq "sslmode=" key > PGSSLMODE env > "prefer" (v15.0.0).
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn resolve_sslmode(conn_str: &str) -> String {
    // URL format: ?sslmode=disable
    if let Some(pos) = conn_str.find("?sslmode=") {
        let rest = &conn_str[pos + 9..];
        let end = rest.find('&').unwrap_or(rest.len());
        return rest[..end].to_string();
    }
    // libpq key=value format: "sslmode=disable" (space-separated tokens)
    for part in conn_str.split_whitespace() {
        if let Some(val) = part.strip_prefix("sslmode=") {
            return val.to_string();
        }
    }
    if let Ok(val) = std::env::var("PGSSLMODE") {
        return val;
    }
    "prefer".to_string()
}

// Connect with TLS routing based on sslmode (v12.6.0).
// "disable" → NoTls; anything else → rustls (webpki-roots CA bundle).
#[cfg(not(target_arch = "wasm32"))]
async fn pg_connect_inner(
    conn_str: &str,
    sslmode: &str,
) -> Result<tokio_postgres::Client, String> {
    match sslmode {
        "disable" => {
            let (client, conn) = tokio_postgres::connect(conn_str, tokio_postgres::NoTls)
                .await
                .map_err(|e| format_pg_error(&e))?;
            tokio::spawn(async move { let _ = conn.await; });
            Ok(client)
        }
        _ => {
            use rustls::{ClientConfig, RootCertStore};
            use tokio_postgres_rustls::MakeRustlsConnect;
            // Ensure ring crypto provider is installed (Rustls 0.23 requires explicit init).
            let _ = rustls::crypto::ring::default_provider().install_default();
            let mut root_store = RootCertStore::empty();
            root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
            let config = ClientConfig::builder()
                .with_root_certificates(root_store)
                .with_no_client_auth();
            let tls = MakeRustlsConnect::new(config);
            let (client, conn) = tokio_postgres::connect(conn_str, tls)
                .await
                .map_err(|e| format_pg_error(&e))?;
            tokio::spawn(async move { let _ = conn.await; });
            Ok(client)
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn pg_conn_str_from_env() -> String {
    if let Ok(url) = std::env::var("DATABASE_URL") {
        return url;
    }
    let host     = std::env::var("PGHOST").unwrap_or_else(|_| "localhost".to_string());
    let port     = std::env::var("PGPORT").unwrap_or_else(|_| "5432".to_string());
    let dbname   = std::env::var("PGDATABASE").unwrap_or_else(|_| "postgres".to_string());
    let user     = std::env::var("PGUSER").unwrap_or_else(|_| "postgres".to_string());
    let password = std::env::var("PGPASSWORD").unwrap_or_default();
    format!("host={} port={} dbname={} user={} password={}", host, port, dbname, user, password)
}

#[cfg(not(target_arch = "wasm32"))]
fn pg_params_from_json(params_json: &str) -> Result<Vec<String>, String> {
    let arr: serde_json::Value = serde_json::from_str(params_json)
        .map_err(|e| format!("invalid params JSON: {}", e))?;
    match arr {
        serde_json::Value::Array(items) => Ok(items.iter().map(|v| match v {
            serde_json::Value::String(s) => s.clone(),
            serde_json::Value::Null      => "NULL".to_string(),
            other                        => other.to_string(),
        }).collect()),
        _ => Err("params must be a JSON array".to_string()),
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn pg_execute(conn_str: &str, sql: &str, params_json: &str) -> Result<(), String> {
    let params = pg_params_from_json(params_json)?;
    let sslmode = resolve_sslmode(conn_str);
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| e.to_string())?;
    rt.block_on(async {
        let client = pg_connect_inner(conn_str, &sslmode).await?;
        let param_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> =
            params.iter().map(|s| s as &(dyn tokio_postgres::types::ToSql + Sync)).collect();
        client.execute(sql, &param_refs).await.map_err(|e| format_pg_error(&e))?;
        Ok(())
    })
}

#[cfg(not(target_arch = "wasm32"))]
pub fn pg_query(conn_str: &str, sql: &str, params_json: &str) -> Result<String, String> {
    let params = pg_params_from_json(params_json)?;
    let sslmode = resolve_sslmode(conn_str);
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| e.to_string())?;
    rt.block_on(async {
        let client = pg_connect_inner(conn_str, &sslmode).await?;
        let param_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> =
            params.iter().map(|s| s as &(dyn tokio_postgres::types::ToSql + Sync)).collect();
        let rows = client.query(sql, &param_refs).await.map_err(|e| format_pg_error(&e))?;
        let json_rows: Vec<serde_json::Value> = rows.iter().map(|row| {
            let mut map = serde_json::Map::new();
            for col in row.columns() {
                let name = col.name().to_string();
                let val: serde_json::Value = {
                    if let Ok(v) = row.try_get::<_, Option<String>>(&name.as_str()) {
                        v.map(serde_json::Value::String).unwrap_or(serde_json::Value::Null)
                    } else if let Ok(v) = row.try_get::<_, Option<i64>>(&name.as_str()) {
                        v.map(|n| serde_json::Value::Number(n.into())).unwrap_or(serde_json::Value::Null)
                    } else if let Ok(v) = row.try_get::<_, Option<i32>>(&name.as_str()) {
                        v.map(|n| serde_json::Value::Number(n.into())).unwrap_or(serde_json::Value::Null)
                    } else if let Ok(v) = row.try_get::<_, Option<f64>>(&name.as_str()) {
                        v.and_then(|f| serde_json::Number::from_f64(f).map(serde_json::Value::Number))
                         .unwrap_or(serde_json::Value::Null)
                    } else if let Ok(v) = row.try_get::<_, Option<bool>>(&name.as_str()) {
                        v.map(serde_json::Value::Bool).unwrap_or(serde_json::Value::Null)
                    } else {
                        serde_json::Value::Null
                    }
                };
                map.insert(name, val);
            }
            serde_json::Value::Object(map)
        }).collect();
        serde_json::to_string(&json_rows).map_err(|e| e.to_string())
    })
}

// ── v20.8.0: PgPool グローバルストア + runtime ────────────────────────────────

#[cfg(not(target_arch = "wasm32"))]
type PgPoolMap = std::collections::HashMap<u64, std::sync::Arc<crate::backend::pg_pool::PgPoolInner>>;

#[cfg(not(target_arch = "wasm32"))]
static PG_POOLS: std::sync::OnceLock<std::sync::Mutex<PgPoolMap>> = std::sync::OnceLock::new();

#[cfg(not(target_arch = "wasm32"))]
static PG_POOL_NEXT_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

#[cfg(not(target_arch = "wasm32"))]
fn pg_pool_store() -> std::sync::MutexGuard<'static, PgPoolMap> {
    PG_POOLS.get_or_init(|| std::sync::Mutex::new(std::collections::HashMap::new()))
        .lock()
        .unwrap_or_else(|e| e.into_inner())
}

#[cfg(not(target_arch = "wasm32"))]
fn pg_pool_alloc(inner: std::sync::Arc<crate::backend::pg_pool::PgPoolInner>) -> u64 {
    let id = PG_POOL_NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    pg_pool_store().insert(id, inner);
    id
}

/// VMValue（List または単一値）を pg_params_from_json が解析できる JSON 配列文字列に変換する。
fn vmvalue_to_params_json(v: &VMValue) -> String {
    fn to_serde(v: &VMValue) -> serde_json::Value {
        match v {
            VMValue::Str(s)   => serde_json::Value::String(s.clone()),
            VMValue::Int(n)   => serde_json::Value::Number((*n).into()),
            VMValue::Float(f) => {
                if f.is_nan() {
                    serde_json::Value::String("NaN".to_string())
                } else if f.is_infinite() && *f > 0.0 {
                    serde_json::Value::String("Infinity".to_string())
                } else if f.is_infinite() {
                    serde_json::Value::String("-Infinity".to_string())
                } else {
                    serde_json::Number::from_f64(*f)
                        .map(serde_json::Value::Number)
                        .unwrap_or(serde_json::Value::Null)
                }
            }
            VMValue::Bool(b)  => serde_json::Value::Bool(*b),
            VMValue::Unit     => serde_json::Value::Null,
            other             => serde_json::Value::String(vmvalue_repr(other)),
        }
    }
    let arr = match v {
        VMValue::List(fl) => serde_json::Value::Array(fl.iter().map(to_serde).collect()),
        other             => serde_json::Value::Array(vec![to_serde(other)]),
    };
    serde_json::to_string(&arr).unwrap_or_else(|_| "[]".to_string())
}

/// PgPool 専用の長寿命 tokio runtime（接続 background task を維持するため）
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn pg_pool_runtime() -> &'static tokio::runtime::Runtime {
    static RT: std::sync::OnceLock<tokio::runtime::Runtime> = std::sync::OnceLock::new();
    RT.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .worker_threads(2)
            .thread_name("fav-pgpool")
            .build()
            .expect("fav: failed to build PgPool runtime")
    })
}

/// PgPool 経由のクエリ実行（async helper）
#[cfg(not(target_arch = "wasm32"))]
async fn pg_query_with_client(
    client: &tokio_postgres::Client,
    sql: &str,
    params: &[String],
) -> Result<Vec<VMValue>, String> {
    let param_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> =
        params.iter().map(|s| s as &(dyn tokio_postgres::types::ToSql + Sync)).collect();
    let rows = client.query(sql, &param_refs).await.map_err(|e| format_pg_error(&e))?;
    let vm_rows: Vec<VMValue> = rows.iter().map(|row| {
        let mut map = std::collections::HashMap::new();
        for col in row.columns() {
            let name = col.name().to_string();
            let val = if let Ok(v) = row.try_get::<_, Option<String>>(&name.as_str()) {
                v.map(VMValue::Str).unwrap_or(VMValue::Unit)
            } else if let Ok(v) = row.try_get::<_, Option<i64>>(&name.as_str()) {
                v.map(VMValue::Int).unwrap_or(VMValue::Unit)
            } else if let Ok(v) = row.try_get::<_, Option<i32>>(&name.as_str()) {
                v.map(|n| VMValue::Int(n as i64)).unwrap_or(VMValue::Unit)
            } else if let Ok(v) = row.try_get::<_, Option<f64>>(&name.as_str()) {
                v.map(VMValue::Float).unwrap_or(VMValue::Unit)
            } else if let Ok(v) = row.try_get::<_, Option<bool>>(&name.as_str()) {
                v.map(VMValue::Bool).unwrap_or(VMValue::Unit)
            } else {
                VMValue::Unit
            };
            map.insert(name, val);
        }
        VMValue::Record(map)
    }).collect();
    Ok(vm_rows)
}

// ── AzurePostgres helpers (v14.1.0) ──────────────────────────────────────────

/// Execute DML on Azure DB for PostgreSQL and return affected row count.
/// Reuses the same pg_connect_inner (rustls TLS) as the Postgres helpers.
/// conn_str must include sslmode=require for Azure's mandatory TLS.
#[cfg(not(target_arch = "wasm32"))]
pub fn azure_pg_execute(conn_str: &str, sql: &str, params_json: &str) -> Result<i64, String> {
    let params = pg_params_from_json(params_json)?;
    let sslmode = resolve_sslmode(conn_str);
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| e.to_string())?;
    rt.block_on(async {
        let client = pg_connect_inner(conn_str, &sslmode).await?;
        let param_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> =
            params.iter().map(|s| s as &(dyn tokio_postgres::types::ToSql + Sync)).collect();
        let n = client.execute(sql, &param_refs).await.map_err(|e| format_pg_error(&e))?;
        Ok(n as i64)
    })
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn snowflake_read_env(key: &str) -> Result<String, String> {
    std::env::var(key).map_err(|_| format!("{} is not set", key))
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(serde::Serialize)]
struct SnowflakeClaims {
    iss: String,
    sub: String,
    iat: i64,
    exp: i64,
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn snowflake_generate_jwt(
    account: &str,
    user: &str,
    private_key_pem: &str,
    public_key_fp: &str,
) -> Result<String, String> {
    let account_up = account.to_uppercase();
    let user_up = user.to_uppercase();
    let now = chrono::Utc::now().timestamp();
    let claims = SnowflakeClaims {
        iss: format!("{}.{}.SHA256:{}", account_up, user_up, public_key_fp),
        sub: format!("{}.{}", account_up, user_up),
        iat: now,
        exp: now + 3600,
    };
    let header = jsonwebtoken::Header::new(jsonwebtoken::Algorithm::RS256);
    let key = jsonwebtoken::EncodingKey::from_rsa_pem(private_key_pem.as_bytes())
        .map_err(|e| format!("Snowflake JWT: invalid private key: {}", e))?;
    jsonwebtoken::encode(&header, &claims, &key)
        .map_err(|e| format!("Snowflake JWT: encode failed: {}", e))
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn snowflake_api_post(
    account: &str,
    jwt: &str,
    body: &serde_json::Value,
) -> Result<serde_json::Value, String> {
    let url = format!(
        "https://{}.snowflakecomputing.com/api/v2/statements",
        account
    );
    let resp = ureq::post(&url)
        .set("Authorization", &format!("Bearer {}", jwt))
        .set("Content-Type", "application/json")
        .set("Accept", "application/json")
        .set("X-Snowflake-Authorization-Token-Type", "KEYPAIR_JWT")
        .send_string(&body.to_string())
        .map_err(|e| match e {
            ureq::Error::Status(_, r) => r.into_string().unwrap_or_default(),
            ureq::Error::Transport(t) => t.to_string(),
        })?;
    let text = resp.into_string().map_err(|e| e.to_string())?;
    serde_json::from_str(&text)
        .map_err(|e| format!("Snowflake API: invalid JSON response: {}", e))
}

fn stringify_json_scalar(value: &SerdeJsonValue) -> Option<String> {
    match value {
        SerdeJsonValue::Null => Some(String::new()),
        SerdeJsonValue::Bool(v) => Some(if *v { "true".into() } else { "false".into() }),
        SerdeJsonValue::Number(v) => Some(v.to_string()),
        SerdeJsonValue::String(v) => Some(v.clone()),
        SerdeJsonValue::Array(_) | SerdeJsonValue::Object(_) => None,
    }
}

fn parse_json_object_raw(text: &str) -> Result<HashMap<String, VMValue>, String> {
    let value: SerdeJsonValue =
        serde_json::from_str(text).map_err(|e| format!("json parse error: {}", e))?;
    let SerdeJsonValue::Object(map) = value else {
        return Err("json parse error: expected object".to_string());
    };
    let mut out = HashMap::new();
    for (key, value) in map {
        let scalar = stringify_json_scalar(&value).ok_or_else(|| {
            "json parse error: nested arrays/objects are not supported".to_string()
        })?;
        out.insert(key, VMValue::Str(scalar));
    }
    Ok(out)
}

fn parse_json_array_raw(text: &str) -> Result<Vec<VMValue>, String> {
    let value: SerdeJsonValue =
        serde_json::from_str(text).map_err(|e| format!("json parse error: {}", e))?;
    let SerdeJsonValue::Array(items) = value else {
        return Err("json parse error: expected array".to_string());
    };
    let mut out = Vec::with_capacity(items.len());
    for item in items {
        let SerdeJsonValue::Object(map) = item else {
            return Err("json parse error: expected array of objects".to_string());
        };
        let mut row = HashMap::new();
        for (key, value) in map {
            let scalar = stringify_json_scalar(&value).ok_or_else(|| {
                "json parse error: nested arrays/objects are not supported".to_string()
            })?;
            row.insert(key, VMValue::Str(scalar));
        }
        out.push(VMValue::Record(row));
    }
    Ok(out)
}

fn parse_bool_like(raw: &str) -> Option<bool> {
    match raw {
        "true" | "1" => Some(true),
        "false" | "0" => Some(false),
        _ => None,
    }
}

fn parse_schema_value(raw: &str, ty: &str, field: &str) -> Result<VMValue, VMValue> {
    if let Some(inner) = ty.strip_prefix("Option<").and_then(|s| s.strip_suffix('>')) {
        if raw.is_empty() {
            return Ok(VMValue::Variant("none".to_string(), None));
        }
        let inner_value = parse_schema_value(raw, inner, field)?;
        return Ok(VMValue::Variant(
            "some".to_string(),
            Some(Box::new(inner_value)),
        ));
    }
    if let Some(inner) = ty.strip_suffix('?') {
        if raw.is_empty() {
            return Ok(VMValue::Variant("none".to_string(), None));
        }
        let inner_value = parse_schema_value(raw, inner, field)?;
        return Ok(VMValue::Variant(
            "some".to_string(),
            Some(Box::new(inner_value)),
        ));
    }

    match ty {
        "Int" => raw
            .parse::<i64>()
            .map(VMValue::Int)
            .map_err(|_| schema_error_vm(field, "Int", raw)),
        "Float" => raw
            .parse::<f64>()
            .map(VMValue::Float)
            .map_err(|_| schema_error_vm(field, "Float", raw)),
        "Bool" => parse_bool_like(raw)
            .map(VMValue::Bool)
            .ok_or_else(|| schema_error_vm(field, "Bool", raw)),
        "String" => Ok(VMValue::Str(raw.to_string())),
        other => Err(schema_error_vm(field, other, raw)),
    }
}

fn schema_rows_from_vm(
    value: VMValue,
    context: &str,
) -> Result<Vec<HashMap<String, VMValue>>, String> {
    match value {
        VMValue::List(fl) => fl
            .into_iter()
            .map(|row| match row {
                VMValue::Record(map) => Ok(map),
                other => Err(format!(
                    "{} expects List<Map<String,String>>, got {}",
                    context,
                    vmvalue_type_name(&other)
                )),
            })
            .collect(),
        other => Err(format!(
            "{} expects List<Map<String,String>>, got {}",
            context,
            vmvalue_type_name(&other)
        )),
    }
}

fn schema_record_to_string_map(record: &HashMap<String, VMValue>) -> HashMap<String, String> {
    record
        .iter()
        .map(|(k, v)| {
            let value = vm_scalar_to_plain_string(v);
            (k.clone(), value)
        })
        .collect()
}

fn vm_scalar_to_plain_string(value: &VMValue) -> String {
    match value {
        VMValue::Str(s) => s.clone(),
        VMValue::Int(n) => n.to_string(),
        VMValue::Float(f) => f.to_string(),
        VMValue::Bool(b) => b.to_string(),
        VMValue::Unit => String::new(),
        VMValue::Variant(tag, None) if tag == "none" => String::new(),
        VMValue::Variant(tag, Some(payload)) if tag == "some" => vm_scalar_to_plain_string(payload),
        other => vmvalue_repr(other),
    }
}

fn schema_adapt_rows(
    rows: Vec<HashMap<String, VMValue>>,
    type_name: &str,
    type_metas: &HashMap<String, TypeMeta>,
) -> VMValue {
    let Some(meta) = type_metas.get(type_name) else {
        return err_vm(schema_error_vm(
            "",
            format!("known type {}", type_name),
            type_name,
        ));
    };
    let positional = meta.fields.iter().any(|field| field.col_index.is_some());
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let mut record = HashMap::new();
        for field in &meta.fields {
            let lookup_key = if positional {
                field
                    .col_index
                    .map(|idx| idx.to_string())
                    .unwrap_or_else(|| field.name.clone())
            } else {
                field.name.clone()
            };
            let raw = match row.get(&lookup_key) {
                Some(VMValue::Str(s)) => s.clone(),
                Some(other) => vmvalue_repr(other),
                None => return err_vm(schema_error_vm(&field.name, &lookup_key, "missing")),
            };
            match parse_schema_value(&raw, &field.ty, &field.name) {
                Ok(value) => {
                    record.insert(field.name.clone(), value);
                }
                Err(err) => return err_vm(err),
            }
        }
        out.push(VMValue::Record(record));
    }
    ok_vm(VMValue::List(FavList::new(out)))
}

fn schema_to_json_value(
    value: &VMValue,
    type_name: &str,
    type_metas: &HashMap<String, TypeMeta>,
) -> Result<SerdeJsonValue, String> {
    let VMValue::Record(record) = value else {
        return Err(format!("Schema expected record for `{}`", type_name));
    };
    let mut out = serde_json::Map::new();
    let ordered_fields: Vec<(String, String)> = if let Some(meta) = type_metas.get(type_name) {
        meta.fields
            .iter()
            .map(|field| (field.name.clone(), field.ty.clone()))
            .collect()
    } else {
        let mut keys: Vec<String> = record.keys().cloned().collect();
        keys.sort();
        keys.into_iter().map(|key| (key, "_".into())).collect()
    };
    for (field_name, field_ty) in ordered_fields {
        let value = record.get(&field_name).ok_or_else(|| {
            format!(
                "record missing field `{}` for schema `{}`",
                field_name, type_name
            )
        })?;
        let json = match value {
            VMValue::Int(v) => SerdeJsonValue::Number((*v).into()),
            VMValue::Float(v) => serde_json::Number::from_f64(*v)
                .map(SerdeJsonValue::Number)
                .ok_or_else(|| format!("invalid float in field `{}`", field_name))?,
            VMValue::Bool(v) => SerdeJsonValue::Bool(*v),
            VMValue::Str(v) => SerdeJsonValue::String(v.clone()),
            VMValue::Variant(tag, None) if tag == "none" => SerdeJsonValue::Null,
            VMValue::Variant(tag, Some(payload)) if tag == "some" => match payload.as_ref() {
                VMValue::Int(v) => SerdeJsonValue::Number((*v).into()),
                VMValue::Float(v) => serde_json::Number::from_f64(*v)
                    .map(SerdeJsonValue::Number)
                    .ok_or_else(|| format!("invalid float in field `{}`", field_name))?,
                VMValue::Bool(v) => SerdeJsonValue::Bool(*v),
                VMValue::Str(v) => SerdeJsonValue::String(v.clone()),
                other => {
                    return Err(format!(
                        "unsupported option payload {} for field `{}`",
                        vmvalue_type_name(other),
                        field_name
                    ));
                }
            },
            other => {
                return Err(format!(
                    "unsupported field value {} for field `{}` ({})",
                    vmvalue_type_name(other),
                    field_name,
                    field_ty
                ));
            }
        };
        out.insert(field_name, json);
    }
    Ok(SerdeJsonValue::Object(out))
}

fn vmvalue_to_sql(value: &VMValue) -> rusqlite::types::Value {
    match value {
        VMValue::Int(n) => rusqlite::types::Value::Integer(*n),
        VMValue::Float(f) => rusqlite::types::Value::Real(*f),
        VMValue::Str(s) => rusqlite::types::Value::Text(s.clone()),
        VMValue::Bool(b) => rusqlite::types::Value::Integer(if *b { 1 } else { 0 }),
        VMValue::Unit => rusqlite::types::Value::Null,
        other => rusqlite::types::Value::Text(vmvalue_repr(other)),
    }
}

fn sqlite_value_to_string(value: rusqlite::types::Value) -> String {
    match value {
        rusqlite::types::Value::Null => "null".to_string(),
        rusqlite::types::Value::Integer(n) => n.to_string(),
        rusqlite::types::Value::Real(f) => f.to_string(),
        rusqlite::types::Value::Text(s) => s,
        rusqlite::types::Value::Blob(bytes) => format!("<blob:{} bytes>", bytes.len()),
    }
}

fn with_db_path<T, F>(db_path: Option<&str>, f: F) -> Result<T, String>
where
    F: FnOnce(&Connection) -> Result<T, String>,
{
    let path =
        db_path.ok_or_else(|| "Db not initialized 窶・run with --db <path> flag".to_string())?;
    let mut dbs = SHARED_DBS
        .lock()
        .map_err(|_| "Db mutex poisoned".to_string())?;
    let entry_idx = if let Some(idx) = dbs.iter().position(|(p, _)| p == path) {
        idx
    } else {
        let conn = if path == ":memory:" {
            Connection::open_in_memory().map_err(|e| format!("Db open failed: {}", e))?
        } else {
            Connection::open(path).map_err(|e| format!("Db open failed for `{}`: {}", path, e))?
        };
        dbs.push((path.to_string(), conn));
        dbs.len() - 1
    };
    let (_, conn) = &dbs[entry_idx];
    f(conn)
}

/// Build a `DbError { code, message }` record.
fn db_error_vm(code: &str, message: &str) -> VMValue {
    let mut m = HashMap::new();
    m.insert("code".to_string(), VMValue::Str(code.to_string()));
    m.insert("message".to_string(), VMValue::Str(message.to_string()));
    VMValue::Record(m)
}

fn http_response_vm(status: i64, body: String, content_type: String) -> VMValue {
    let mut m = HashMap::new();
    m.insert("status".to_string(), VMValue::Int(status));
    m.insert("body".to_string(), VMValue::Str(body));
    m.insert("content_type".to_string(), VMValue::Str(content_type));
    VMValue::Record(m)
}

fn http_error_vm(code: i64, message: String, status: i64) -> VMValue {
    let mut m = HashMap::new();
    m.insert("code".to_string(), VMValue::Int(code));
    m.insert("message".to_string(), VMValue::Str(message));
    m.insert("status".to_string(), VMValue::Int(status));
    VMValue::Record(m)
}

fn parquet_error_vm(message: impl Into<String>) -> VMValue {
    let mut m = HashMap::new();
    m.insert("message".to_string(), VMValue::Str(message.into()));
    VMValue::Record(m)
}

fn rpc_error_vm(code: i64, message: impl Into<String>) -> VMValue {
    let mut m = HashMap::new();
    m.insert("code".to_string(), VMValue::Int(code));
    m.insert("message".to_string(), VMValue::Str(message.into()));
    VMValue::Record(m)
}

fn encode_grpc_frame(payload: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(5 + payload.len());
    out.push(0u8);
    out.extend_from_slice(&(payload.len() as u32).to_be_bytes());
    out.extend_from_slice(payload);
    out
}

fn decode_grpc_frame(data: &[u8]) -> Result<Vec<u8>, String> {
    if data.len() < 5 {
        return Err(format!("gRPC frame too short: {} bytes", data.len()));
    }
    let len = u32::from_be_bytes([data[1], data[2], data[3], data[4]]) as usize;
    if data.len() < 5 + len {
        return Err(format!(
            "gRPC frame body truncated: expected {} bytes, got {}",
            len,
            data.len().saturating_sub(5)
        ));
    }
    Ok(data[5..5 + len].to_vec())
}

fn decode_all_grpc_frames(data: &[u8]) -> Result<Vec<Vec<u8>>, String> {
    let mut frames = Vec::new();
    let mut offset = 0usize;
    while offset < data.len() {
        if data.len() - offset < 5 {
            return Err(format!(
                "gRPC trailing bytes too short for frame header: {}",
                data.len() - offset
            ));
        }
        let len = u32::from_be_bytes([
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
            data[offset + 4],
        ]) as usize;
        let end = offset + 5 + len;
        if end > data.len() {
            return Err(format!(
                "gRPC frame body truncated: expected {} bytes, got {}",
                len,
                data.len().saturating_sub(offset + 5)
            ));
        }
        frames.push(data[offset + 5..end].to_vec());
        offset = end;
    }
    Ok(frames)
}

#[allow(dead_code)]
fn pascal_to_snake(name: &str) -> String {
    let mut out = String::new();
    for (idx, ch) in name.chars().enumerate() {
        if ch.is_ascii_uppercase() {
            if idx > 0 {
                out.push('_');
            }
            out.push(ch.to_ascii_lowercase());
        } else {
            out.push(ch);
        }
    }
    out
}

fn proto_wire_type_for_field(ty: &str) -> u8 {
    match option_inner_type_name(ty) {
        "Int" | "Bool" => 0,
        "Float" => 1,
        _ => 2,
    }
}

fn encode_varint(mut value: u64, out: &mut Vec<u8>) {
    while value >= 0x80 {
        out.push(((value as u8) & 0x7f) | 0x80);
        value >>= 7;
    }
    out.push(value as u8);
}

fn decode_varint(bytes: &[u8], pos: &mut usize) -> Result<u64, String> {
    let mut shift = 0u32;
    let mut value = 0u64;
    while *pos < bytes.len() {
        let byte = bytes[*pos];
        *pos += 1;
        value |= ((byte & 0x7f) as u64) << shift;
        if byte & 0x80 == 0 {
            return Ok(value);
        }
        shift += 7;
        if shift > 63 {
            return Err("protobuf varint too large".to_string());
        }
    }
    Err("unexpected EOF while reading protobuf varint".to_string())
}

fn skip_proto_value(bytes: &[u8], pos: &mut usize, wire_type: u8) -> Result<(), String> {
    match wire_type {
        0 => {
            let _ = decode_varint(bytes, pos)?;
            Ok(())
        }
        1 => {
            if *pos + 8 > bytes.len() {
                return Err("unexpected EOF while reading 64-bit field".to_string());
            }
            *pos += 8;
            Ok(())
        }
        2 => {
            let len = decode_varint(bytes, pos)? as usize;
            if *pos + len > bytes.len() {
                return Err("unexpected EOF while reading length-delimited field".to_string());
            }
            *pos += len;
            Ok(())
        }
        other => Err(format!("unsupported protobuf wire type {}", other)),
    }
}

fn map_to_proto_bytes(
    type_name: &str,
    row: &HashMap<String, String>,
    type_metas: &HashMap<String, TypeMeta>,
) -> Result<Vec<u8>, String> {
    let meta = type_metas
        .get(type_name)
        .ok_or_else(|| format!("Grpc.encode_raw: unknown type `{}`", type_name))?;
    let mut out = Vec::new();
    for (idx, field) in meta.fields.iter().enumerate() {
        let Some(raw) = row.get(&field.name) else {
            continue;
        };
        if raw.is_empty() && is_option_type_name(&field.ty) {
            continue;
        }
        let field_no = (idx + 1) as u64;
        let wire_type = proto_wire_type_for_field(&field.ty) as u64;
        encode_varint((field_no << 3) | wire_type, &mut out);
        match option_inner_type_name(&field.ty) {
            "Int" => {
                let value = raw.parse::<i64>().map_err(|e| {
                    format!(
                        "Grpc.encode_raw invalid Int field `{}` value `{}`: {}",
                        field.name, raw, e
                    )
                })?;
                encode_varint(value as u64, &mut out);
            }
            "Bool" => {
                let value = parse_bool_like(raw).ok_or_else(|| {
                    format!(
                        "Grpc.encode_raw invalid Bool field `{}` value `{}`",
                        field.name, raw
                    )
                })?;
                encode_varint(if value { 1 } else { 0 }, &mut out);
            }
            "Float" => {
                let value = raw.parse::<f64>().map_err(|e| {
                    format!(
                        "Grpc.encode_raw invalid Float field `{}` value `{}`: {}",
                        field.name, raw, e
                    )
                })?;
                out.extend_from_slice(&value.to_le_bytes());
            }
            _ => {
                encode_varint(raw.len() as u64, &mut out);
                out.extend_from_slice(raw.as_bytes());
            }
        }
    }
    Ok(out)
}

fn string_map_to_proto_bytes(row: &HashMap<String, String>) -> Vec<u8> {
    let mut fields: Vec<(&String, &String)> = row.iter().collect();
    fields.sort_by(|a, b| a.0.cmp(b.0));
    let mut out = Vec::new();
    for (idx, (_key, value)) in fields.iter().enumerate() {
        let field_no = (idx + 1) as u64;
        let tag = (field_no << 3) | 2u64;
        encode_varint(tag, &mut out);
        encode_varint(value.len() as u64, &mut out);
        out.extend_from_slice(value.as_bytes());
    }
    out
}

fn proto_bytes_to_map(
    type_name: &str,
    bytes: &[u8],
    type_metas: &HashMap<String, TypeMeta>,
) -> Result<HashMap<String, String>, String> {
    let meta = type_metas
        .get(type_name)
        .ok_or_else(|| format!("Grpc.decode_raw: unknown type `{}`", type_name))?;
    let mut out = HashMap::new();
    let mut pos = 0usize;
    while pos < bytes.len() {
        let key = decode_varint(bytes, &mut pos)?;
        let field_no = (key >> 3) as usize;
        let wire_type = (key & 0x07) as u8;
        let Some(field) = meta.fields.get(field_no.saturating_sub(1)) else {
            skip_proto_value(bytes, &mut pos, wire_type)?;
            continue;
        };
        let value = match (option_inner_type_name(&field.ty), wire_type) {
            ("Int", 0) => decode_varint(bytes, &mut pos)?.to_string(),
            ("Bool", 0) => {
                if decode_varint(bytes, &mut pos)? == 0 {
                    "false".to_string()
                } else {
                    "true".to_string()
                }
            }
            ("Float", 1) => {
                if pos + 8 > bytes.len() {
                    return Err("unexpected EOF while reading double".to_string());
                }
                let mut buf = [0u8; 8];
                buf.copy_from_slice(&bytes[pos..pos + 8]);
                pos += 8;
                f64::from_le_bytes(buf).to_string()
            }
            (_, 2) => {
                let len = decode_varint(bytes, &mut pos)? as usize;
                if pos + len > bytes.len() {
                    return Err("unexpected EOF while reading string field".to_string());
                }
                let value = String::from_utf8(bytes[pos..pos + len].to_vec())
                    .map_err(|e| format!("Grpc.decode_raw invalid UTF-8: {}", e))?;
                pos += len;
                value
            }
            _ => {
                skip_proto_value(bytes, &mut pos, wire_type)?;
                continue;
            }
        };
        out.insert(field.name.clone(), value);
    }
    Ok(out)
}

fn proto_bytes_to_string_map(bytes: &[u8]) -> Result<HashMap<String, String>, String> {
    let mut out = HashMap::new();
    let mut pos = 0usize;
    while pos < bytes.len() {
        let key = decode_varint(bytes, &mut pos)?;
        let field_no = (key >> 3) as usize;
        let wire_type = (key & 0x07) as u8;
        match wire_type {
            0 => {
                let value = decode_varint(bytes, &mut pos)?.to_string();
                out.insert(format!("field{}", field_no), value);
            }
            1 => {
                if pos + 8 > bytes.len() {
                    return Err("unexpected EOF while reading double".to_string());
                }
                let mut buf = [0u8; 8];
                buf.copy_from_slice(&bytes[pos..pos + 8]);
                pos += 8;
                out.insert(
                    format!("field{}", field_no),
                    f64::from_le_bytes(buf).to_string(),
                );
            }
            2 => {
                let len = decode_varint(bytes, &mut pos)? as usize;
                if pos + len > bytes.len() {
                    return Err("unexpected EOF while reading string field".to_string());
                }
                let value = String::from_utf8(bytes[pos..pos + len].to_vec())
                    .map_err(|e| format!("Grpc raw response invalid UTF-8: {}", e))?;
                pos += len;
                out.insert(format!("field{}", field_no), value);
            }
            other => {
                skip_proto_value(bytes, &mut pos, other)?;
            }
        }
    }
    Ok(out)
}

/// Decode proto bytes to a string map, resolving field numbers to names via type_metas.
/// Falls back to "field{n}" when the type or field is not found.
fn proto_bytes_to_named_map(
    bytes: &[u8],
    type_name: &str,
    type_metas: &HashMap<String, TypeMeta>,
) -> Result<HashMap<String, String>, String> {
    let fields: Vec<String> = type_metas
        .get(type_name)
        .map(|tm| tm.fields.iter().map(|f| f.name.clone()).collect())
        .unwrap_or_default();

    let mut out = HashMap::new();
    let mut pos = 0usize;
    while pos < bytes.len() {
        let key = decode_varint(bytes, &mut pos)?;
        let field_no = (key >> 3) as usize;
        let wire_type = (key & 0x07) as u8;
        let field_name = if field_no >= 1 && field_no <= fields.len() {
            fields[field_no - 1].clone()
        } else {
            format!("field{}", field_no)
        };
        match wire_type {
            0 => {
                let value = decode_varint(bytes, &mut pos)?.to_string();
                out.insert(field_name, value);
            }
            1 => {
                if pos + 8 > bytes.len() {
                    return Err("unexpected EOF while reading double".to_string());
                }
                let mut buf = [0u8; 8];
                buf.copy_from_slice(&bytes[pos..pos + 8]);
                pos += 8;
                out.insert(field_name, f64::from_le_bytes(buf).to_string());
            }
            2 => {
                let len = decode_varint(bytes, &mut pos)? as usize;
                if pos + len > bytes.len() {
                    return Err("unexpected EOF while reading string field".to_string());
                }
                let value = String::from_utf8(bytes[pos..pos + len].to_vec())
                    .map_err(|e| format!("Grpc raw response invalid UTF-8: {}", e))?;
                pos += len;
                out.insert(field_name, value);
            }
            other => {
                skip_proto_value(bytes, &mut pos, other)?;
            }
        }
    }
    Ok(out)
}

/// Type alias for messages sent from the h2 server thread to the VM dispatch loop.
/// `(handler_fn_name, proto_bytes, response_sender)`
type GrpcRequestMsg = (
    String,
    Vec<u8>,
    std::sync::mpsc::SyncSender<Result<Vec<u8>, String>>,
);

/// Spawn a background tokio thread running an h2/gRPC server on `port`.
/// Each incoming request is forwarded to `req_tx`; the VM loop replies via the
/// per-request `SyncSender` embedded in the message.
fn grpc_serve_impl(
    port: i64,
    req_tx: std::sync::mpsc::Sender<GrpcRequestMsg>,
) -> Result<(), String> {
    let port_u16 = u16::try_from(port).map_err(|_| format!("invalid gRPC port {}", port))?;
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("tokio runtime build failed");
        rt.block_on(async move {
            let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port_u16))
                .await
                .expect("gRPC bind failed");
            eprintln!("Listening on 0.0.0.0:{port_u16} (gRPC / HTTP2)");
            loop {
                let (socket, _) = match listener.accept().await {
                    Ok(s) => s,
                    Err(_) => continue,
                };
                let req_tx = req_tx.clone();
                tokio::spawn(async move {
                    let mut conn = match h2::server::handshake(socket).await {
                        Ok(c) => c,
                        Err(_) => return,
                    };
                    while let Some(result) = conn.accept().await {
                        let (request, respond) = match result {
                            Ok(r) => r,
                            Err(_) => return,
                        };
                        let req_tx = req_tx.clone();
                        tokio::spawn(async move {
                            grpc_handle_h2_request(request, respond, req_tx).await;
                        });
                    }
                });
            }
        });
    });
    Ok(())
}

/// Async handler for a single h2 gRPC request: reads body, dispatches to VM
/// via channel, sends response back over h2.
async fn grpc_handle_h2_request(
    request: http::Request<h2::RecvStream>,
    mut respond: h2::server::SendResponse<Bytes>,
    req_tx: std::sync::mpsc::Sender<GrpcRequestMsg>,
) {
    // Derive handler name: "/ServiceName/MethodName" -> "handle_method_name"
    let path = request.uri().path().to_string();
    let method_part = path.rsplit('/').next().unwrap_or("unknown").to_string();
    let handler_name = format!("handle_{}", pascal_to_snake(&method_part));

    // Read all DATA frames from the request body
    let mut body = request.into_body();
    let mut body_bytes: Vec<u8> = Vec::new();
    while let Some(chunk) = body.data().await {
        match chunk {
            Ok(data) => {
                let n = data.len();
                body_bytes.extend_from_slice(&data);
                let _ = body.flow_control().release_capacity(n);
            }
            Err(_) => return,
        }
    }

    // Strip gRPC framing (5-byte prefix); fall back to raw bytes if malformed
    let proto_bytes = decode_grpc_frame(&body_bytes).unwrap_or(body_bytes);

    // Send request to VM dispatch loop and wait for response
    let (res_tx, res_rx) = std::sync::mpsc::sync_channel::<Result<Vec<u8>, String>>(1);
    if req_tx.send((handler_name, proto_bytes, res_tx)).is_err() {
        return;
    }
    let resp_data = match tokio::task::spawn_blocking(move || res_rx.recv()).await {
        Ok(Ok(Ok(b))) => b,
        _ => return,
    };

    // Send HTTP/2 response
    let http_resp = http::Response::builder()
        .status(200)
        .header("content-type", "application/grpc")
        .body(())
        .unwrap();
    let mut send = match respond.send_response(http_resp, false) {
        Ok(s) => s,
        Err(_) => return,
    };
    let _ = send.send_data(Bytes::from(resp_data), false);
    let mut trailers = http::HeaderMap::new();
    trailers.insert(
        http::header::HeaderName::from_static("grpc-status"),
        http::HeaderValue::from_static("0"),
    );
    let _ = send.send_trailers(trailers);
}

/// Convert a VM function result into proto bytes for a gRPC response.
fn grpc_vm_value_to_proto_bytes(result: Result<VMValue, String>) -> Result<Vec<u8>, String> {
    match result {
        Ok(VMValue::Record(map)) => {
            let str_map = schema_record_to_string_map(&map);
            Ok(string_map_to_proto_bytes(&str_map))
        }
        Ok(other) => Err(format!(
            "gRPC handler must return Map<String,String>, got {}",
            vmvalue_type_name(&other)
        )),
        Err(e) => Err(e),
    }
}

fn is_known_builtin_namespace(name: &str) -> bool {
    let namespace = name.split('.').next().unwrap_or(name);
    matches!(
        namespace,
        "IO" | "Debug"
            | "Result"
            | "Option"
            | "Math"
            | "Int"
            | "Float"
            | "Bool"
            | "String"
            | "List"
            | "Map"
            | "Trace"
            | "Emit"
            | "File"
            | "Json"
            | "Csv"
            | "Schema"
            | "Checkpoint"
            | "Db"
            | "DB"
            | "Env"
            | "Http"
            | "Grpc"
            | "Parquet"
            | "Task"
            | "Random"
            | "Stream"
            | "Gen"
            | "Validate"
            | "DuckDb"
            | "Crypto"
            | "Auth"
            | "Log"
            | "AWS"
            | "Cache"
            | "Queue"
            | "Email"
            | "Compiler"
            | "Ctx"
            | "AppCtx"
            | "ArrowBatch"
            | "__duckdb_push"
            | "Arena"
            | "State"   // v22.3.0
            | "Bytes"   // v23.1.0
            | "Mut"     // v23.3.0
    )
}

fn looks_like_variant_ctor(name: &str) -> bool {
    name.chars()
        .next()
        .map(|c| c.is_ascii_uppercase())
        .unwrap_or(false)
}

/// Extract the TCP address from a gRPC host string.
/// "http://host:port" -> "host:port", "host:port" -> "host:port"
fn grpc_tcp_addr(host: &str) -> String {
    if let Some(rest) = host.strip_prefix("http://") {
        rest.trim_end_matches('/').to_string()
    } else if let Some(rest) = host.strip_prefix("https://") {
        rest.trim_end_matches('/').to_string()
    } else {
        host.to_string()
    }
}

/// Build a full URI for a gRPC method call.
fn grpc_method_uri(host: &str, method: &str) -> String {
    let base = if host.starts_with("http://") || host.starts_with("https://") {
        host.trim_end_matches('/').to_string()
    } else {
        format!("http://{}", host)
    };
    format!("{}/{}", base, method.trim_start_matches('/'))
}

fn is_option_type_name(ty: &str) -> bool {
    ty.starts_with("Option<") && ty.ends_with('>')
}

fn option_inner_type_name(ty: &str) -> &str {
    if is_option_type_name(ty) {
        &ty[7..ty.len() - 1]
    } else {
        ty
    }
}

fn arrow_type_for_meta(ty: &str) -> DataType {
    match option_inner_type_name(ty) {
        "Int" => DataType::Int64,
        "Float" => DataType::Float64,
        "Bool" => DataType::Boolean,
        _ => DataType::Utf8,
    }
}

// ── v19.5.0: Arrow ヘルパー関数 ───────────────────────────────────────────────

/// `List<Record>` → `RecordBatch` 変換（スキーマを第 1 行目から推論）
fn arrow_from_vm_rows(rows: &[VMValue]) -> Result<arrow::record_batch::RecordBatch, String> {
    use arrow::array::*;
    use arrow::datatypes::*;
    use std::sync::Arc;

    if rows.is_empty() {
        let schema = Arc::new(Schema::empty());
        return Ok(arrow::record_batch::RecordBatch::new_empty(schema));
    }

    let first = match &rows[0] {
        VMValue::Record(m) => m,
        other => {
            return Err(format!(
                "ArrowBatch.from_list: expected Record rows, got {:?}",
                vmvalue_type_name(other)
            ))
        }
    };

    let mut field_names: Vec<String> = first.keys().cloned().collect();
    field_names.sort();

    let fields: Vec<Field> = field_names
        .iter()
        .map(|name| {
            let dtype = match first.get(name).unwrap() {
                VMValue::Int(_) => DataType::Int64,
                VMValue::Float(_) => DataType::Float64,
                VMValue::Bool(_) => DataType::Boolean,
                _ => DataType::Utf8,
            };
            Field::new(name.as_str(), dtype, true)
        })
        .collect();

    let schema = Arc::new(Schema::new(fields.clone()));

    let arrays: Vec<Arc<dyn Array>> = fields
        .iter()
        .map(|field| {
            let name = field.name();
            match field.data_type() {
                DataType::Int64 => {
                    let vals: Vec<Option<i64>> = rows
                        .iter()
                        .map(|r| match r {
                            VMValue::Record(m) => match m.get(name) {
                                Some(VMValue::Int(n)) => Some(*n),
                                _ => None,
                            },
                            _ => None,
                        })
                        .collect();
                    Arc::new(Int64Array::from(vals)) as Arc<dyn Array>
                }
                DataType::Float64 => {
                    let vals: Vec<Option<f64>> = rows
                        .iter()
                        .map(|r| match r {
                            VMValue::Record(m) => match m.get(name) {
                                Some(VMValue::Float(f)) => Some(*f),
                                _ => None,
                            },
                            _ => None,
                        })
                        .collect();
                    Arc::new(Float64Array::from(vals)) as Arc<dyn Array>
                }
                DataType::Boolean => {
                    let vals: Vec<Option<bool>> = rows
                        .iter()
                        .map(|r| match r {
                            VMValue::Record(m) => match m.get(name) {
                                Some(VMValue::Bool(b)) => Some(*b),
                                _ => None,
                            },
                            _ => None,
                        })
                        .collect();
                    Arc::new(BooleanArray::from(vals)) as Arc<dyn Array>
                }
                _ => {
                    let vals: Vec<Option<String>> = rows
                        .iter()
                        .map(|r| match r {
                            VMValue::Record(m) => m.get(name).map(|v| match v {
                                VMValue::Str(s) => s.clone(),
                                other => format!("{:?}", other),
                            }),
                            _ => None,
                        })
                        .collect();
                    let refs: Vec<Option<&str>> =
                        vals.iter().map(|s| s.as_deref()).collect();
                    Arc::new(StringArray::from(refs)) as Arc<dyn Array>
                }
            }
        })
        .collect();

    arrow::record_batch::RecordBatch::try_new(schema, arrays)
        .map_err(|e| format!("arrow RecordBatch::try_new: {e}"))
}

/// `RecordBatch` → `List<Record>` 変換
fn arrow_to_vm_rows(
    batch: &arrow::record_batch::RecordBatch,
) -> Result<Vec<VMValue>, String> {
    use arrow::array::*;
    use arrow::datatypes::DataType;

    let schema = batch.schema();
    let num_rows = batch.num_rows();
    let mut result = Vec::with_capacity(num_rows);

    for row_idx in 0..num_rows {
        let mut record: HashMap<String, VMValue> = HashMap::new();
        for (col_idx, field) in schema.fields().iter().enumerate() {
            let col = batch.column(col_idx);
            let val = match field.data_type() {
                DataType::Int64 => {
                    let arr = col.as_any().downcast_ref::<Int64Array>().unwrap();
                    if arr.is_null(row_idx) {
                        VMValue::Unit
                    } else {
                        VMValue::Int(arr.value(row_idx))
                    }
                }
                DataType::Float64 => {
                    let arr = col.as_any().downcast_ref::<Float64Array>().unwrap();
                    if arr.is_null(row_idx) {
                        VMValue::Unit
                    } else {
                        VMValue::Float(arr.value(row_idx))
                    }
                }
                DataType::Boolean => {
                    let arr = col.as_any().downcast_ref::<BooleanArray>().unwrap();
                    if arr.is_null(row_idx) {
                        VMValue::Unit
                    } else {
                        VMValue::Bool(arr.value(row_idx))
                    }
                }
                _ => {
                    let arr = col.as_any().downcast_ref::<StringArray>().unwrap();
                    if arr.is_null(row_idx) {
                        VMValue::Unit
                    } else {
                        VMValue::Str(arr.value(row_idx).to_string())
                    }
                }
            };
            record.insert(field.name().clone(), val);
        }
        result.push(VMValue::Record(record));
    }

    Ok(result)
}

/// `RecordBatch` → Parquet ファイル書き込み（ゼロコピーに近い）
fn arrow_write_parquet(
    batch: &arrow::record_batch::RecordBatch,
    path: &str,
) -> Result<(), String> {
    use parquet::arrow::arrow_writer::ArrowWriter;
    if let Some(parent) = std::path::Path::new(path).parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("arrow_write_parquet mkdir: {e}"))?;
        }
    }
    let file = std::fs::File::create(path)
        .map_err(|e| format!("arrow_write_parquet create: {e}"))?;
    let mut writer = ArrowWriter::try_new(file, batch.schema(), None)
        .map_err(|e| format!("ArrowWriter::try_new: {e}"))?;
    writer.write(batch).map_err(|e| format!("ArrowWriter::write: {e}"))?;
    writer.close().map_err(|e| format!("ArrowWriter::close: {e}"))?;
    Ok(())
}

/// Parquet ファイル → `RecordBatch` 読み込み
fn arrow_read_parquet(path: &str) -> Result<arrow::record_batch::RecordBatch, String> {
    use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
    let file = std::fs::File::open(path)
        .map_err(|e| format!("arrow_read_parquet open: {e}"))?;
    let mut reader = ParquetRecordBatchReaderBuilder::try_new(file)
        .map_err(|e| format!("ParquetRecordBatchReaderBuilder: {e}"))?
        .build()
        .map_err(|e| format!("build reader: {e}"))?;
    reader
        .next()
        .ok_or_else(|| "arrow_read_parquet: no batches in file".to_string())?
        .map_err(|e| format!("read batch: {e}"))
}

// ── v20.5.0: mmap + arrow-csv CSV reader ─────────────────────────────────────

/// Read a CSV file using mmap (zero-copy) and arrow-csv (columnar SIMD parsing).
/// Schema is inferred from the header + first 1000 rows.
/// Returns a single `RecordBatch` (multiple internal chunks are merged via `concat_batches`).
///
/// Guarded with `#[cfg(not(target_arch = "wasm32"))]` — on WASM always returns Err.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn read_csv_mmap(path: &str) -> Result<arrow::record_batch::RecordBatch, String> {
    use arrow::csv::reader::Format;
    use arrow::csv::ReaderBuilder;
    use memmap2::MmapOptions;
    use std::io::Cursor;

    let file = std::fs::File::open(path)
        .map_err(|e| format!("ArrowBatch.from_csv: cannot open '{}': {e}", path))?;

    // SAFETY: The file is opened read-only and not modified during the mmap lifetime.
    // External file mutation during a pipeline run is outside Favnir's contract.
    let mmap = unsafe {
        MmapOptions::new()
            .map(&file)
            .map_err(|e| format!("ArrowBatch.from_csv: mmap failed: {e}"))?
    };

    // Pass 1: infer schema from first 1000 rows.
    // Format::infer_schema consumes the reader — use a fresh Cursor for the read pass.
    let format = Format::default().with_header(true);
    let (schema, _) = format
        .infer_schema(Cursor::new(&mmap[..]), Some(1000))
        .map_err(|e| format!("ArrowBatch.from_csv: schema inference failed: {e}"))?;

    let schema = std::sync::Arc::new(schema);

    // Pass 2: read all data via mmap slice (zero-copy second reference into the same mapping)
    let mut reader = ReaderBuilder::new(schema.clone())
        .with_header(true)
        .with_batch_size(65536)
        .build(Cursor::new(&mmap[..]))
        .map_err(|e| format!("ArrowBatch.from_csv: reader build failed: {e}"))?;

    // Collect all record batches
    let mut batches: Vec<arrow::record_batch::RecordBatch> = (&mut reader)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("ArrowBatch.from_csv: parse failed: {e}"))?;

    if batches.is_empty() {
        return Err(format!("ArrowBatch.from_csv: no data in '{}'", path));
    }
    if batches.len() == 1 {
        return Ok(batches.swap_remove(0));
    }

    // Merge multiple chunks into a single RecordBatch
    arrow::compute::concat_batches(&schema, &batches)
        .map_err(|e| format!("ArrowBatch.from_csv: concat_batches failed: {e}"))
}

// No wasm32 stub needed: the call site in vm_call_builtin is cfg-guarded below.

// ── v20.6.0: io_uring batch file reader ──────────────────────────────────────

/// Read multiple files concurrently.
/// Linux: tokio-uring (io_uring, near-zero context switches).
/// Windows / macOS: rayon parallel read_to_string (thread-pool fallback).
/// Returns files in the same order as the input paths.
/// Any single failure causes the whole batch to return Err.
#[cfg(all(target_os = "linux", not(target_arch = "wasm32")))]
pub(crate) fn read_files_batch_impl(paths: &[String]) -> Result<Vec<String>, String> {
    tokio_uring::start(async {
        let handles: Vec<_> = paths.iter()
            .map(|p| read_one_uring(p.clone()))
            .collect();
        futures::future::try_join_all(handles).await
    })
    // map_err handles only tokio_uring::start() own failures (e.g. kernel too old).
    // Per-file errors from read_one_uring propagate as-is via try_join_all,
    // so they are not double-wrapped with this prefix.
    .map_err(|e| format!("IO.read_files_batch: io_uring runtime error: {e}"))
}

/// Read a single file using tokio-uring (Linux only).
/// Opens the file, reads via `read_at` with move semantics, and truncates the
/// buffer to the actual bytes read to handle partial reads correctly.
#[cfg(all(target_os = "linux", not(target_arch = "wasm32")))]
async fn read_one_uring(path: String) -> Result<String, String> {
    use tokio_uring::fs::File;

    let size = std::fs::metadata(&path)
        .map_err(|e| format!("IO.read_files_batch: metadata '{}': {e}", path))?
        .len() as usize;

    if size == 0 {
        return Ok(String::new());
    }

    let file = File::open(&path).await
        .map_err(|e| format!("IO.read_files_batch: open '{}': {e}", path))?;

    let buf = vec![0u8; size];
    let (res, mut buf) = file.read_at(buf, 0).await;
    let bytes_read = res
        .map_err(|e| format!("IO.read_files_batch: read '{}': {e}", path))?;
    buf.truncate(bytes_read);

    String::from_utf8(buf)
        .map_err(|e| format!("IO.read_files_batch: utf8 '{}': {e}", path))
}

/// Non-Linux fallback: rayon parallel read (Windows / macOS).
#[cfg(all(not(target_os = "linux"), not(target_arch = "wasm32")))]
pub(crate) fn read_files_batch_impl(paths: &[String]) -> Result<Vec<String>, String> {
    use rayon::prelude::*;
    paths
        .par_iter()
        .map(|p| {
            std::fs::read_to_string(p)
                .map_err(|e| format!("IO.read_files_batch: cannot read '{}': {e}", p))
        })
        .collect()
}

// ── v20.4.0: DuckDB pushdown execution ───────────────────────────────────────

/// Execute a pushdown SQL query against a DuckDB ArrowBatch.
/// Returns `Ok(VMValue::ArrowBatch(new_id))` on success.
/// Returns `Err(...)` on failure (caller falls back to the original stage fn).
///
/// Strategy: write the ArrowBatch to a temp Parquet file, then use DuckDB's
/// native `read_parquet(path)` as the table reference. This avoids arrow crate
/// version incompatibilities between the project and the duckdb crate.
///
/// Guarded with `#[cfg(not(target_arch = "wasm32"))]` — on WASM always returns Err.
#[cfg(not(target_arch = "wasm32"))]
fn execute_duckdb_pushdown(batch_id: u64, sql_template: &str) -> Result<VMValue, String> {
    let batch = match arrow_get(batch_id) {
        Some(b) => b,
        None => return Err(format!("pushdown: invalid ArrowBatch handle {batch_id}")),
    };

    // Write batch to a temp Parquet file.
    // Include PID + thread-local sequence number to prevent collisions during
    // parallel execution (par stages, concurrent tests).
    thread_local! {
        static PUSHDOWN_TMP_SEQ: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
    }
    let seq = PUSHDOWN_TMP_SEQ.with(|c| { let v = c.get(); c.set(v + 1); v });
    let pid = std::process::id();
    let tmp_path = {
        let mut p = std::env::temp_dir();
        p.push(format!("_fav_pushdown_{pid}_{seq}_{batch_id}.parquet"));
        p
    };
    let tmp_str = tmp_path
        .to_str()
        .ok_or_else(|| "pushdown: non-UTF8 temp path".to_string())?
        .to_string();
    arrow_write_parquet(&batch, &tmp_str)?;

    // Replace placeholder with read_parquet reference (using forward slashes for DuckDB)
    let duckdb_path = tmp_str.replace('\\', "/");
    let sql = sql_template.replace(
        "?pushdown_table?",
        &format!("read_parquet('{duckdb_path}')"),
    );
    debug_assert!(!sql.contains("?pushdown_table?"), "SQL placeholder not fully replaced");

    // Open DuckDB in-memory and execute the query
    let conn = duckdb::Connection::open_in_memory()
        .map_err(|e| format!("pushdown: duckdb open: {e}"))?;

    let rows = duckdb_query_typed(&conn, &sql)?;

    // Clean up temp file (best-effort)
    let _ = std::fs::remove_file(&tmp_path);

    // Convert query result rows back to an ArrowBatch
    let result_batch = arrow_from_vm_rows(&rows)?;
    Ok(VMValue::ArrowBatch(arrow_store(result_batch)))
}

#[cfg(target_arch = "wasm32")]
fn execute_duckdb_pushdown(_batch_id: u64, _sql_template: &str) -> Result<VMValue, String> {
    Err("pushdown: not supported on wasm32".to_string())
}

fn parquet_write_rows(
    path: &str,
    type_name: &str,
    rows: Vec<HashMap<String, VMValue>>,
    type_metas: &HashMap<String, TypeMeta>,
) -> Result<(), String> {
    let meta = type_metas
        .get(type_name)
        .ok_or_else(|| format!("Parquet.write_raw: unknown type `{}`", type_name))?;
    if let Some(parent) = Path::new(path).parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Parquet.write_raw failed to create directory: {}", e))?;
        }
    }

    let fields: Vec<ArrowField> = meta
        .fields
        .iter()
        .map(|field| {
            ArrowField::new(
                &field.name,
                arrow_type_for_meta(&field.ty),
                is_option_type_name(&field.ty),
            )
        })
        .collect();
    let schema = std::sync::Arc::new(ArrowSchema::new(fields));
    let mut arrays: Vec<ArrayRef> = Vec::with_capacity(meta.fields.len());

    for field in &meta.fields {
        let base_ty = option_inner_type_name(&field.ty);
        match arrow_type_for_meta(&field.ty) {
            DataType::Int64 => {
                let mut builder = Int64Builder::new();
                for row in &rows {
                    let raw = row
                        .get(&field.name)
                        .map(vm_scalar_to_plain_string)
                        .unwrap_or_default();
                    if raw.is_empty() && is_option_type_name(&field.ty) {
                        builder.append_null();
                    } else {
                        let value = raw.parse::<i64>().map_err(|e| {
                            format!(
                                "Parquet.write_raw invalid {} field `{}` value `{}`: {}",
                                base_ty, field.name, raw, e
                            )
                        })?;
                        builder.append_value(value);
                    }
                }
                arrays.push(std::sync::Arc::new(builder.finish()));
            }
            DataType::Float64 => {
                let mut builder = Float64Builder::new();
                for row in &rows {
                    let raw = row
                        .get(&field.name)
                        .map(vm_scalar_to_plain_string)
                        .unwrap_or_default();
                    if raw.is_empty() && is_option_type_name(&field.ty) {
                        builder.append_null();
                    } else {
                        let value = raw.parse::<f64>().map_err(|e| {
                            format!(
                                "Parquet.write_raw invalid {} field `{}` value `{}`: {}",
                                base_ty, field.name, raw, e
                            )
                        })?;
                        builder.append_value(value);
                    }
                }
                arrays.push(std::sync::Arc::new(builder.finish()));
            }
            DataType::Boolean => {
                let mut builder = BooleanBuilder::new();
                for row in &rows {
                    let raw = row
                        .get(&field.name)
                        .map(vm_scalar_to_plain_string)
                        .unwrap_or_default();
                    if raw.is_empty() && is_option_type_name(&field.ty) {
                        builder.append_null();
                    } else {
                        let value = match raw.as_str() {
                            "true" => true,
                            "false" => false,
                            _ => {
                                return Err(format!(
                                    "Parquet.write_raw invalid Bool field `{}` value `{}`",
                                    field.name, raw
                                ));
                            }
                        };
                        builder.append_value(value);
                    }
                }
                arrays.push(std::sync::Arc::new(builder.finish()));
            }
            DataType::Utf8 => {
                let mut builder = StringBuilder::new();
                for row in &rows {
                    let raw = row
                        .get(&field.name)
                        .map(vm_scalar_to_plain_string)
                        .unwrap_or_default();
                    if raw.is_empty() && is_option_type_name(&field.ty) {
                        builder.append_null();
                    } else {
                        builder.append_value(raw);
                    }
                }
                arrays.push(std::sync::Arc::new(builder.finish()));
            }
            other => {
                return Err(format!(
                    "Parquet.write_raw unsupported Arrow type for `{}`: {:?}",
                    field.name, other
                ));
            }
        }
    }

    let batch = RecordBatch::try_new(schema.clone(), arrays)
        .map_err(|e| format!("Parquet.write_raw record batch failed: {}", e))?;
    let file = File::create(path).map_err(|e| format!("Parquet.write_raw open failed: {}", e))?;
    let mut writer = ArrowWriter::try_new(file, schema, None)
        .map_err(|e| format!("Parquet.write_raw writer failed: {}", e))?;
    writer
        .write(&batch)
        .map_err(|e| format!("Parquet.write_raw write failed: {}", e))?;
    writer
        .close()
        .map_err(|e| format!("Parquet.write_raw close failed: {}", e))?;
    Ok(())
}

fn parquet_read_rows(path: &str) -> Result<Vec<HashMap<String, VMValue>>, String> {
    let file = File::open(path).map_err(|e| format!("Parquet.read_raw open failed: {}", e))?;
    let builder = ParquetRecordBatchReaderBuilder::try_new(file)
        .map_err(|e| format!("Parquet.read_raw reader failed: {}", e))?;
    let reader = builder
        .build()
        .map_err(|e| format!("Parquet.read_raw build failed: {}", e))?;
    let mut rows = Vec::new();
    for batch_result in reader {
        let batch = batch_result.map_err(|e| format!("Parquet.read_raw batch failed: {}", e))?;
        let schema = batch.schema();
        for row_idx in 0..batch.num_rows() {
            let mut row = HashMap::new();
            for (col_idx, field) in schema.fields().iter().enumerate() {
                let column = batch.column(col_idx);
                let value = parquet_cell_to_string(column.as_ref(), row_idx)?;
                row.insert(field.name().clone(), VMValue::Str(value));
            }
            rows.push(row);
        }
    }
    Ok(rows)
}

fn parquet_cell_to_string(array: &dyn Array, row_idx: usize) -> Result<String, String> {
    if array.is_null(row_idx) {
        return Ok(String::new());
    }
    if let Some(arr) = array.as_any().downcast_ref::<StringArray>() {
        return Ok(arr.value(row_idx).to_string());
    }
    if let Some(arr) = array.as_any().downcast_ref::<Int64Array>() {
        return Ok(arr.value(row_idx).to_string());
    }
    if let Some(arr) = array.as_any().downcast_ref::<Float64Array>() {
        return Ok(arr.value(row_idx).to_string());
    }
    if let Some(arr) = array.as_any().downcast_ref::<BooleanArray>() {
        return Ok(arr.value(row_idx).to_string());
    }
    Err(format!(
        "Parquet.read_raw unsupported column type: {:?}",
        array.data_type()
    ))
}

/// Execute a raw SELECT and return rows as `List<Map<String,String>>`.
fn sqlite_query_raw(conn: &rusqlite::Connection, sql: &str) -> Result<Vec<VMValue>, String> {
    let mut stmt = conn
        .prepare(sql)
        .map_err(|e| format!("E0602: db query failed: {}", e))?;
    let col_names: Vec<String> = stmt.column_names().iter().map(|s| s.to_string()).collect();
    let mut rows_out = Vec::new();
    let mut rows = stmt
        .query([])
        .map_err(|e| format!("E0602: db query failed: {}", e))?;
    while let Some(row) = rows
        .next()
        .map_err(|e| format!("E0602: db query failed: {}", e))?
    {
        let mut map = HashMap::new();
        for (i, name) in col_names.iter().enumerate() {
            let val: rusqlite::types::Value = row
                .get(i)
                .map_err(|e| format!("E0602: db query failed: {}", e))?;
            map.insert(name.clone(), VMValue::Str(sqlite_value_to_string(val)));
        }
        rows_out.push(VMValue::Record(map));
    }
    Ok(rows_out)
}

/// Execute a parameterised SELECT and return rows as `List<Map<String,String>>`.
fn sqlite_query_raw_params(
    conn: &rusqlite::Connection,
    sql: &str,
    params: &[String],
) -> Result<Vec<VMValue>, String> {
    let mut stmt = conn
        .prepare(sql)
        .map_err(|e| format!("E0602: db query failed: {}", e))?;
    let col_names: Vec<String> = stmt.column_names().iter().map(|s| s.to_string()).collect();
    let param_refs: Vec<&dyn rusqlite::ToSql> =
        params.iter().map(|s| s as &dyn rusqlite::ToSql).collect();
    let mut rows_out = Vec::new();
    let mut rows = stmt
        .query(param_refs.as_slice())
        .map_err(|e| format!("E0602: db query failed: {}", e))?;
    while let Some(row) = rows
        .next()
        .map_err(|e| format!("E0602: db query failed: {}", e))?
    {
        let mut map = HashMap::new();
        for (i, name) in col_names.iter().enumerate() {
            let val: rusqlite::types::Value = row
                .get(i)
                .map_err(|e| format!("E0602: db query failed: {}", e))?;
            map.insert(name.clone(), VMValue::Str(sqlite_value_to_string(val)));
        }
        rows_out.push(VMValue::Record(map));
    }
    Ok(rows_out)
}

// ── DuckDB helpers (v4.3.0) ──────────────────────────────────────────────────

fn duckdb_value_to_string(val: duckdb::types::Value) -> String {
    use duckdb::types::Value;
    match val {
        Value::Null => String::new(),
        Value::Boolean(b) => b.to_string(),
        Value::TinyInt(i) => i.to_string(),
        Value::SmallInt(i) => i.to_string(),
        Value::Int(i) => i.to_string(),
        Value::BigInt(i) => i.to_string(),
        Value::HugeInt(i) => i.to_string(),
        Value::UTinyInt(i) => i.to_string(),
        Value::USmallInt(i) => i.to_string(),
        Value::UInt(i) => i.to_string(),
        Value::UBigInt(i) => i.to_string(),
        Value::Float(f) => f.to_string(),
        Value::Double(f) => f.to_string(),
        Value::Text(s) => s,
        Value::Blob(b) => format!("<blob:{}>", b.len()),
        _ => String::new(),
    }
}

fn duckdb_value_to_vmvalue(val: duckdb::types::Value) -> VMValue {
    use duckdb::types::Value;
    match val {
        Value::Null => VMValue::Unit,
        Value::Boolean(b) => VMValue::Bool(b),
        Value::TinyInt(i) => VMValue::Int(i as i64),
        Value::SmallInt(i) => VMValue::Int(i as i64),
        Value::Int(i) => VMValue::Int(i as i64),
        Value::BigInt(i) => VMValue::Int(i),
        Value::HugeInt(i) => VMValue::Int(i as i64),
        Value::UTinyInt(i) => VMValue::Int(i as i64),
        Value::USmallInt(i) => VMValue::Int(i as i64),
        Value::UInt(i) => VMValue::Int(i as i64),
        Value::UBigInt(i) => VMValue::Int(i as i64),
        Value::Float(f) => VMValue::Float(f as f64),
        Value::Double(f) => VMValue::Float(f),
        Value::Text(s) => VMValue::Str(s),
        other => VMValue::Str(duckdb_value_to_string(other)),
    }
}

fn duckdb_query_raw(conn: &duckdb::Connection, sql: &str) -> Result<Vec<VMValue>, String> {
    duckdb_query_impl(conn, sql, false)
}

fn duckdb_query_typed(conn: &duckdb::Connection, sql: &str) -> Result<Vec<VMValue>, String> {
    duckdb_query_impl(conn, sql, true)
}

fn duckdb_query_impl(
    conn: &duckdb::Connection,
    sql: &str,
    typed: bool,
) -> Result<Vec<VMValue>, String> {
    let mut stmt = conn
        .prepare(sql)
        .map_err(|e| format!("DuckDB query failed: {}", e))?;
    // Execute the query first; column info is only available after execution.
    let mut rows = stmt
        .query([])
        .map_err(|e| format!("DuckDB query failed: {}", e))?;
    // Collect column names from the executed statement (via Rows::as_ref).
    let col_names: Vec<String> = rows.as_ref().map(|s| s.column_names()).unwrap_or_default();
    let mut rows_out = Vec::new();
    while let Some(row) = rows
        .next()
        .map_err(|e| format!("DuckDB row fetch failed: {}", e))?
    {
        let mut map = HashMap::new();
        for (i, name) in col_names.iter().enumerate() {
            let val: duckdb::types::Value = row
                .get(i)
                .map_err(|e| format!("DuckDB column get failed: {}", e))?;
            let vm_val = if typed {
                duckdb_value_to_vmvalue(val)
            } else {
                VMValue::Str(duckdb_value_to_string(val))
            };
            map.insert(name.clone(), vm_val);
        }
        rows_out.push(VMValue::Record(map));
    }
    Ok(rows_out)
}

// ── Gen 2.0 — YAML config structs (v4.4.0) ────────────────────────────────────

#[derive(Default, Clone)]
struct GenFieldConfig {
    values: Vec<String>,
    min: Option<f64>,
    max: Option<f64>,
    null_rate: f64,
}

#[derive(Default, Clone)]
struct GenYamlConfig {
    fields: HashMap<String, GenFieldConfig>,
}

// ── Gen helpers (v3.5.0) ─────────────────────────────────────────────────────

fn seeded_rand_int(lo: i64, hi: i64) -> i64 {
    use rand::Rng;
    SEEDED_RNG.with(|r| {
        let mut borrowed = r.borrow_mut();
        if let Some(rng) = borrowed.as_mut() {
            rng.gen_range(lo..=hi)
        } else {
            rand::thread_rng().gen_range(lo..=hi)
        }
    })
}

fn seeded_rand_float() -> f64 {
    use rand::Rng;
    SEEDED_RNG.with(|r| {
        let mut borrowed = r.borrow_mut();
        if let Some(rng) = borrowed.as_mut() {
            rng.r#gen::<f64>()
        } else {
            rand::thread_rng().r#gen::<f64>()
        }
    })
}

fn random_alphanumeric_string(len: usize) -> String {
    const CHARS: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    (0..len)
        .map(|_| {
            let idx = seeded_rand_int(0, (CHARS.len() - 1) as i64) as usize;
            CHARS[idx] as char
        })
        .collect()
}

fn gen_value_for_type(ty: &str) -> String {
    if ty.starts_with("Option<") && ty.ends_with('>') {
        // 50% chance of empty (None), 50% of inner type
        if seeded_rand_int(0, 1) == 0 {
            String::new()
        } else {
            let inner = &ty[7..ty.len() - 1];
            gen_value_for_type(inner)
        }
    } else {
        match ty {
            "Int" => seeded_rand_int(-1000, 1000).to_string(),
            "Float" => format!("{:.6}", seeded_rand_float()),
            "Bool" => if seeded_rand_int(0, 1) == 0 {
                "false"
            } else {
                "true"
            }
            .to_string(),
            _ => random_alphanumeric_string(8),
        }
    }
}

fn gen_corrupt_value(ty: &str) -> String {
    // Returns a value that is intentionally invalid for the given type
    if ty.starts_with("Option<") {
        String::new() // Options become empty (None) when corrupted
    } else {
        match ty {
            "Int" | "Float" => "NaN".to_string(),
            "Bool" => "maybe".to_string(),
            _ => String::new(),
        }
    }
}

fn gen_one_row(type_name: &str, type_metas: &HashMap<String, TypeMeta>) -> Result<VMValue, String> {
    let meta = type_metas
        .get(type_name)
        .ok_or_else(|| format!("Gen.one_raw: unknown type '{type_name}'"))?;
    let mut map = HashMap::new();
    for field in &meta.fields {
        let val = gen_value_for_type(&field.ty);
        map.insert(field.name.clone(), VMValue::Str(val));
    }
    Ok(VMValue::Record(map))
}

// ── Gen 2.0 — hint-based generation (v4.4.0) ──────────────────────────────────

const JA_LAST_NAMES: &[&str] = &[
    "田中", "鈴木", "佐藤", "高橋", "伊藤", "渡辺", "山本", "中村", "小林", "加藤",
];
const JA_FIRST_NAMES: &[&str] = &[
    "太郎", "花子", "一郎", "京子", "健二", "恵子", "誠", "裕子", "明", "直子",
];
const STATUSES: &[&str] = &["active", "inactive", "pending"];
const DESCRIPTIONS: &[&str] = &[
    "標準的な商品です。",
    "人気の高いアイテムです。",
    "新商品です。",
    "定番の品です。",
];

fn hint_counter_next(key: &str) -> u64 {
    HINT_ID_COUNTER.with(|c| {
        let mut map = c.borrow_mut();
        let entry = map.entry(key.to_string()).or_insert(0);
        *entry += 1;
        *entry
    })
}

fn hint_reset_counters() {
    HINT_ID_COUNTER.with(|c| c.borrow_mut().clear());
}

/// Convert a serde_json::Value (JWT claims payload) into a VM claims map.
/// All values are stringified: numbers → decimal string, bools → "true"/"false", etc.
fn json_value_to_vm_claims_map(value: &SerdeJsonValue) -> HashMap<String, VMValue> {
    let mut map = HashMap::new();
    if let SerdeJsonValue::Object(obj) = value {
        for (k, v) in obj {
            let s = match v {
                SerdeJsonValue::String(s) => s.clone(),
                SerdeJsonValue::Number(n) => n.to_string(),
                SerdeJsonValue::Bool(b) => {
                    if *b {
                        "true".to_string()
                    } else {
                        "false".to_string()
                    }
                }
                other => other.to_string(),
            };
            map.insert(k.clone(), VMValue::Str(s));
        }
    }
    map
}

fn gen_hint_value_for_field(type_name: &str, field_name: &str, ty: &str) -> String {
    // Check YAML config overrides first
    let yaml_cfg = GEN_YAML_CONFIG.with(|c| {
        c.borrow()
            .get(type_name)
            .and_then(|cfg| cfg.fields.get(field_name).cloned())
    });

    let fname = field_name.to_lowercase();

    if let Some(ref cfg) = yaml_cfg {
        if !cfg.values.is_empty() {
            let idx = seeded_rand_int(0, (cfg.values.len() - 1) as i64) as usize;
            return cfg.values[idx].clone();
        }
        if (ty == "Int" || ty == "Float") && (cfg.min.is_some() || cfg.max.is_some()) {
            let min = cfg.min.unwrap_or(0.0);
            let max = cfg.max.unwrap_or(1000.0);
            return if ty == "Int" {
                seeded_rand_int(min as i64, max as i64).to_string()
            } else {
                format!("{:.2}", min + seeded_rand_float() * (max - min))
            };
        }
    }

    let counter_key = format!("{}.{}", type_name, field_name);

    if fname == "uuid" || fname.ends_with("_uuid") {
        uuid::Uuid::new_v4().to_string()
    } else if fname == "id" || fname.ends_with("_id") {
        hint_counter_next(&counter_key).to_string()
    } else if fname == "email" || fname.ends_with("_email") {
        let n = hint_counter_next(&counter_key);
        format!("user{}@example.com", n)
    } else if fname == "first_name" || fname == "given_name" {
        let i = seeded_rand_int(0, (JA_FIRST_NAMES.len() - 1) as i64) as usize;
        JA_FIRST_NAMES[i].to_string()
    } else if fname == "last_name" || fname == "family_name" {
        let i = seeded_rand_int(0, (JA_LAST_NAMES.len() - 1) as i64) as usize;
        JA_LAST_NAMES[i].to_string()
    } else if fname == "name" || fname.ends_with("_name") || fname == "full_name" {
        let li = seeded_rand_int(0, (JA_LAST_NAMES.len() - 1) as i64) as usize;
        let fi = seeded_rand_int(0, (JA_FIRST_NAMES.len() - 1) as i64) as usize;
        format!("{} {}", JA_LAST_NAMES[li], JA_FIRST_NAMES[fi])
    } else if fname == "phone" || fname.ends_with("_phone") {
        let a = seeded_rand_int(1000, 9999);
        let b = seeded_rand_int(1000, 9999);
        format!("090-{}-{}", a, b)
    } else if fname.ends_with("_at")
        || fname.ends_with("_datetime")
        || fname == "created_at"
        || fname == "updated_at"
    {
        let offset_secs = seeded_rand_int(-(365 * 2 * 24 * 3600), 365 * 2 * 24 * 3600);
        let dt = Utc::now() + chrono::Duration::seconds(offset_secs);
        dt.format("%Y-%m-%dT%H:%M:%SZ").to_string()
    } else if fname.ends_with("_date") || fname == "birth_date" || fname == "date" {
        let offset_days = seeded_rand_int(-(365 * 50), 365 * 50);
        let dt = Utc::now() + chrono::Duration::days(offset_days);
        dt.format("%Y-%m-%d").to_string()
    } else if fname == "price"
        || fname == "amount"
        || fname.ends_with("_fee")
        || fname.ends_with("_price")
        || fname.ends_with("_amount")
    {
        format!("{}.00", seeded_rand_int(100, 99999))
    } else if fname == "age" {
        seeded_rand_int(0, 130).to_string()
    } else if fname == "count" || fname.ends_with("_count") {
        seeded_rand_int(1, 999).to_string()
    } else if fname == "url" || fname.ends_with("_url") {
        let n = hint_counter_next(&counter_key);
        format!("https://example.com/item/{}", n)
    } else if fname == "zip" || fname == "postal_code" {
        format!(
            "{}-{}",
            seeded_rand_int(100, 999),
            seeded_rand_int(1000, 9999)
        )
    } else if fname == "address" {
        format!("東京都千代田区{}丁目", seeded_rand_int(1, 30))
    } else if fname == "description" || fname == "body" || fname == "content" {
        let i = seeded_rand_int(0, (DESCRIPTIONS.len() - 1) as i64) as usize;
        DESCRIPTIONS[i].to_string()
    } else if fname == "status" {
        let i = seeded_rand_int(0, (STATUSES.len() - 1) as i64) as usize;
        STATUSES[i].to_string()
    } else if fname == "flag" || fname.starts_with("is_") || fname.starts_with("has_") {
        if seeded_rand_int(0, 1) == 0 {
            "false"
        } else {
            "true"
        }
        .to_string()
    } else {
        gen_value_for_type(ty)
    }
}

fn gen_hint_one_row(
    type_name: &str,
    type_metas: &HashMap<String, TypeMeta>,
) -> Result<VMValue, String> {
    let meta = type_metas
        .get(type_name)
        .ok_or_else(|| format!("Gen.hint_one_raw: unknown type '{type_name}'"))?;
    let mut map = HashMap::new();
    for field in &meta.fields {
        let val = gen_hint_value_for_field(type_name, &field.name, &field.ty);
        map.insert(field.name.clone(), VMValue::Str(val));
    }
    Ok(VMValue::Record(map))
}

fn is_valid_for_type(val: &str, ty: &str) -> bool {
    if ty.starts_with("Option<") && ty.ends_with('>') {
        if val.is_empty() {
            return true; // None is always valid for Option
        }
        let inner = &ty[7..ty.len() - 1];
        return is_valid_for_type(val, inner);
    }
    match ty {
        "Int" => val.parse::<i64>().is_ok(),
        "Float" => val.parse::<f64>().is_ok(),
        "Bool" => val == "true" || val == "false",
        _ => true, // String and unknown types are always valid
    }
}

/// Core validation logic shared by Validate.run_raw and TypeName.validate (v6.6.0).
/// Checks all registered constraints for the given type and returns Ok(record) or Err(errors).
fn validate_record_inner(
    type_name: &str,
    raw: HashMap<String, VMValue>,
    schemas: &crate::schemas::ProjectSchemas,
) -> Result<VMValue, String> {
    let Some(type_schema) = schemas.get(type_name) else {
        // No schema registered — pass through as Ok
        return Ok(ok_vm(VMValue::Record(raw)));
    };

    let mut errors: Vec<VMValue> = vec![];

    for (field_name, fc) in type_schema {
        let raw_val = raw.get(field_name.as_str()).and_then(|v| {
            if let VMValue::Str(s) = v {
                Some(s.clone())
            } else {
                None
            }
        });

        match raw_val {
            None if !fc.nullable => {
                let mut e = HashMap::new();
                e.insert("field".into(), VMValue::Str(field_name.clone()));
                e.insert("constraint".into(), VMValue::Str("required".into()));
                e.insert("value".into(), VMValue::Str(String::new()));
                errors.push(VMValue::Record(e));
            }
            Some(ref val_str) => {
                // positive / non_negative
                if fc.constraints.iter().any(|c| c == "positive") {
                    if let Ok(n) = val_str.parse::<f64>() {
                        if n <= 0.0 {
                            let mut e = HashMap::new();
                            e.insert("field".into(), VMValue::Str(field_name.clone()));
                            e.insert("constraint".into(), VMValue::Str("positive".into()));
                            e.insert("value".into(), VMValue::Str(val_str.clone()));
                            errors.push(VMValue::Record(e));
                        }
                    }
                }
                if fc.constraints.iter().any(|c| c == "non_negative") {
                    if let Ok(n) = val_str.parse::<f64>() {
                        if n < 0.0 {
                            let mut e = HashMap::new();
                            e.insert("field".into(), VMValue::Str(field_name.clone()));
                            e.insert("constraint".into(), VMValue::Str("non_negative".into()));
                            e.insert("value".into(), VMValue::Str(val_str.clone()));
                            errors.push(VMValue::Record(e));
                        }
                    }
                }
                // min / max
                if let Some(min) = fc.min {
                    if let Ok(n) = val_str.parse::<f64>() {
                        if n < min {
                            let mut e = HashMap::new();
                            e.insert("field".into(), VMValue::Str(field_name.clone()));
                            e.insert("constraint".into(), VMValue::Str(format!("min:{}", min)));
                            e.insert("value".into(), VMValue::Str(val_str.clone()));
                            errors.push(VMValue::Record(e));
                        }
                    }
                }
                if let Some(max) = fc.max {
                    if let Ok(n) = val_str.parse::<f64>() {
                        if n > max {
                            let mut e = HashMap::new();
                            e.insert("field".into(), VMValue::Str(field_name.clone()));
                            e.insert("constraint".into(), VMValue::Str(format!("max:{}", max)));
                            e.insert("value".into(), VMValue::Str(val_str.clone()));
                            errors.push(VMValue::Record(e));
                        }
                    }
                }
                // max_length / min_length
                if let Some(max_len) = fc.max_length {
                    if val_str.len() > max_len {
                        let mut e = HashMap::new();
                        e.insert("field".into(), VMValue::Str(field_name.clone()));
                        e.insert("constraint".into(), VMValue::Str("max_length".into()));
                        e.insert("value".into(), VMValue::Str(val_str.clone()));
                        errors.push(VMValue::Record(e));
                    }
                }
                if let Some(min_len) = fc.min_length {
                    if val_str.len() < min_len {
                        let mut e = HashMap::new();
                        e.insert("field".into(), VMValue::Str(field_name.clone()));
                        e.insert("constraint".into(), VMValue::Str("min_length".into()));
                        e.insert("value".into(), VMValue::Str(val_str.clone()));
                        errors.push(VMValue::Record(e));
                    }
                }
                // pattern
                if let Some(ref pat) = fc.pattern {
                    if let Ok(re) = regex::Regex::new(pat) {
                        if !re.is_match(val_str) {
                            let mut e = HashMap::new();
                            e.insert("field".into(), VMValue::Str(field_name.clone()));
                            e.insert("constraint".into(), VMValue::Str("pattern".into()));
                            e.insert("value".into(), VMValue::Str(val_str.clone()));
                            errors.push(VMValue::Record(e));
                        }
                    }
                }
                // one_of (v6.6.0)
                if let Some(ref allowed) = fc.one_of {
                    if !allowed.contains(val_str) {
                        let mut e = HashMap::new();
                        e.insert("field".into(), VMValue::Str(field_name.clone()));
                        e.insert("constraint".into(), VMValue::Str("one_of".into()));
                        e.insert("value".into(), VMValue::Str(val_str.clone()));
                        errors.push(VMValue::Record(e));
                    }
                }
            }
            _ => {}
        }
    }

    if errors.is_empty() {
        Ok(ok_vm(VMValue::Record(raw)))
    } else {
        Ok(err_vm(VMValue::List(FavList::new(errors))))
    }
}

fn vm_call_builtin(
    name: &str,
    args: Vec<VMValue>,
    emit_log: &mut Vec<VMValue>,
    db_path: Option<&str>,
    type_metas: &HashMap<String, TypeMeta>,
) -> Result<VMValue, String> {
    match name {
        "Math.pi" => {
            if !args.is_empty() {
                return Err("Math.pi requires 0 arguments".to_string());
            }
            Ok(VMValue::Float(std::f64::consts::PI))
        }
        "Math.e" => {
            if !args.is_empty() {
                return Err("Math.e requires 0 arguments".to_string());
            }
            Ok(VMValue::Float(std::f64::consts::E))
        }
        "Math.abs" => match args.as_slice() {
            [VMValue::Int(n)] => Ok(VMValue::Int(n.abs())),
            [_] => Err("Math.abs requires an Int argument".to_string()),
            _ => Err("Math.abs requires 1 argument".to_string()),
        },
        "Math.abs_float" => match args.as_slice() {
            [VMValue::Float(f)] => Ok(VMValue::Float(f.abs())),
            [_] => Err("Math.abs_float requires a Float argument".to_string()),
            _ => Err("Math.abs_float requires 1 argument".to_string()),
        },
        "Math.min" => match args.as_slice() {
            [VMValue::Int(a), VMValue::Int(b)] => Ok(VMValue::Int((*a).min(*b))),
            [_, _] => Err("Math.min requires (Int, Int)".to_string()),
            _ => Err("Math.min requires 2 arguments".to_string()),
        },
        "Math.max" => match args.as_slice() {
            [VMValue::Int(a), VMValue::Int(b)] => Ok(VMValue::Int((*a).max(*b))),
            [_, _] => Err("Math.max requires (Int, Int)".to_string()),
            _ => Err("Math.max requires 2 arguments".to_string()),
        },
        "Math.min_float" => match args.as_slice() {
            [VMValue::Float(a), VMValue::Float(b)] => Ok(VMValue::Float(a.min(*b))),
            [_, _] => Err("Math.min_float requires (Float, Float)".to_string()),
            _ => Err("Math.min_float requires 2 arguments".to_string()),
        },
        "Math.max_float" => match args.as_slice() {
            [VMValue::Float(a), VMValue::Float(b)] => Ok(VMValue::Float(a.max(*b))),
            [_, _] => Err("Math.max_float requires (Float, Float)".to_string()),
            _ => Err("Math.max_float requires 2 arguments".to_string()),
        },
        "Math.clamp" => match args.as_slice() {
            [VMValue::Int(v), VMValue::Int(lo), VMValue::Int(hi)] => {
                Ok(VMValue::Int((*v).max(*lo).min(*hi)))
            }
            [_, _, _] => Err("Math.clamp requires (Int, Int, Int)".to_string()),
            _ => Err("Math.clamp requires 3 arguments".to_string()),
        },
        "Math.pow" => match args.as_slice() {
            [VMValue::Int(base), VMValue::Int(exp)] if *exp >= 0 => {
                Ok(VMValue::Int(base.pow(*exp as u32)))
            }
            [VMValue::Int(_), VMValue::Int(_)] => {
                Err("Math.pow requires a non-negative exponent".to_string())
            }
            [_, _] => Err("Math.pow requires (Int, Int)".to_string()),
            _ => Err("Math.pow requires 2 arguments".to_string()),
        },
        "Math.pow_float" => match args.as_slice() {
            [VMValue::Float(base), VMValue::Float(exp)] => Ok(VMValue::Float(base.powf(*exp))),
            [_, _] => Err("Math.pow_float requires (Float, Float)".to_string()),
            _ => Err("Math.pow_float requires 2 arguments".to_string()),
        },
        "Math.sqrt" => match args.as_slice() {
            [VMValue::Float(v)] => Ok(VMValue::Float(v.sqrt())),
            [_] => Err("Math.sqrt requires a Float argument".to_string()),
            _ => Err("Math.sqrt requires 1 argument".to_string()),
        },
        "Math.floor" => match args.as_slice() {
            [VMValue::Float(v)] => Ok(VMValue::Int(v.floor() as i64)),
            [_] => Err("Math.floor requires a Float argument".to_string()),
            _ => Err("Math.floor requires 1 argument".to_string()),
        },
        "Math.ceil" => match args.as_slice() {
            [VMValue::Float(v)] => Ok(VMValue::Int(v.ceil() as i64)),
            [_] => Err("Math.ceil requires a Float argument".to_string()),
            _ => Err("Math.ceil requires 1 argument".to_string()),
        },
        "Math.round" => match args.as_slice() {
            [VMValue::Float(v)] => Ok(VMValue::Int(v.round() as i64)),
            [_] => Err("Math.round requires a Float argument".to_string()),
            _ => Err("Math.round requires 1 argument".to_string()),
        },
        "Float.to_bits" => match args.as_slice() {
            [VMValue::Float(v)] => Ok(VMValue::Int(v.to_bits() as i64)),
            [_] => Err("Float.to_bits requires a Float argument".to_string()),
            _ => Err("Float.to_bits requires 1 argument".to_string()),
        },
        "IO.println" => {
            let s = match args.into_iter().next() {
                Some(VMValue::Str(s)) => s,
                Some(v) => vmvalue_repr(&v),
                None => return Err("IO.println requires 1 argument".to_string()),
            };
            IO_CAPTURE.with(|c| {
                if let Some(buf) = c.borrow_mut().as_mut() {
                    buf.push_str(&s);
                    buf.push('\n');
                } else if !is_io_suppressed() {
                    println!("{}", s);
                }
            });
            Ok(VMValue::Unit)
        }
        "IO.println_int" => match args.as_slice() {
            [VMValue::Int(n)] => {
                let n = *n;
                IO_CAPTURE.with(|c| {
                    if let Some(buf) = c.borrow_mut().as_mut() {
                        buf.push_str(&n.to_string());
                        buf.push('\n');
                    } else if !is_io_suppressed() {
                        println!("{}", n);
                    }
                });
                Ok(VMValue::Unit)
            }
            [_] => Err("IO.println_int requires an Int argument".to_string()),
            _ => Err("IO.println_int requires 1 argument".to_string()),
        },
        "IO.println_float" => match args.as_slice() {
            [VMValue::Float(n)] => {
                let n = *n;
                IO_CAPTURE.with(|c| {
                    if let Some(buf) = c.borrow_mut().as_mut() {
                        buf.push_str(&n.to_string());
                        buf.push('\n');
                    } else if !is_io_suppressed() {
                        println!("{}", n);
                    }
                });
                Ok(VMValue::Unit)
            }
            [_] => Err("IO.println_float requires a Float argument".to_string()),
            _ => Err("IO.println_float requires 1 argument".to_string()),
        },
        "IO.println_bool" => match args.as_slice() {
            [VMValue::Bool(b)] => {
                let s = if *b { "true" } else { "false" };
                IO_CAPTURE.with(|c| {
                    if let Some(buf) = c.borrow_mut().as_mut() {
                        buf.push_str(s);
                        buf.push('\n');
                    } else if !is_io_suppressed() {
                        println!("{}", s);
                    }
                });
                Ok(VMValue::Unit)
            }
            [_] => Err("IO.println_bool requires a Bool argument".to_string()),
            _ => Err("IO.println_bool requires 1 argument".to_string()),
        },
        "IO.print" => {
            use std::io::Write;
            let s = match args.into_iter().next() {
                Some(VMValue::Str(s)) => s,
                Some(v) => vmvalue_repr(&v),
                None => return Err("IO.print requires 1 argument".to_string()),
            };
            IO_CAPTURE.with(|c| {
                if let Some(buf) = c.borrow_mut().as_mut() {
                    buf.push_str(&s);
                } else if !is_io_suppressed() {
                    print!("{}", s);
                    std::io::stdout().flush().ok();
                }
            });
            Ok(VMValue::Unit)
        }
        "IO.read_line" => {
            if !args.is_empty() {
                return Err("IO.read_line requires 0 arguments".to_string());
            }
            if is_io_suppressed() {
                return Ok(VMValue::Str(String::new()));
            }
            use std::io::BufRead;
            let mut line = String::new();
            std::io::stdin()
                .lock()
                .read_line(&mut line)
                .map_err(|e| format!("IO.read_line failed: {e}"))?;
            if line.ends_with('\n') {
                line.pop();
            }
            if line.ends_with('\r') {
                line.pop();
            }
            Ok(VMValue::Str(line))
        }
        "IO.timestamp" => {
            if !args.is_empty() {
                return Err("IO.timestamp requires 0 arguments".to_string());
            }
            Ok(VMValue::Str(current_timestamp_string()))
        }

        // ── File I/O primitives (v5.1.0) ─────────────────────────────────────
        "IO.read_file_raw" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "IO.read_file_raw requires 1 argument".to_string())?;
            let path = match v {
                VMValue::Str(s) => s,
                _ => return Err("IO.read_file_raw requires a String path".to_string()),
            };
            match std::fs::read_to_string(&path) {
                Ok(content) => Ok(ok_vm(VMValue::Str(content))),
                Err(e) => Ok(err_vm(VMValue::Str(e.to_string()))),
            }
        }
        // ── v20.6.0: io_uring batch read ─────────────────────────────────────
        "IO.read_files_batch" => {
            let paths = match args.into_iter().next() {
                Some(VMValue::List(list)) => list
                    .iter()
                    .map(|v| match v {
                        VMValue::Str(s) => Ok(s.clone()),
                        other => Err(format!(
                            "IO.read_files_batch: path must be String, got {}",
                            vmvalue_type_name(other)
                        )),
                    })
                    .collect::<Result<Vec<_>, _>>()?,
                _ => return Err("IO.read_files_batch: expected List<String>".to_string()),
            };
            #[cfg(not(target_arch = "wasm32"))]
            {
                match read_files_batch_impl(&paths) {
                    Ok(contents) => {
                        let list = FavList::new(
                            contents.into_iter().map(VMValue::Str).collect::<Vec<_>>(),
                        );
                        Ok(ok_vm(VMValue::List(list)))
                    }
                    Err(e) => Ok(err_vm(VMValue::Str(e))),
                }
            }
            #[cfg(target_arch = "wasm32")]
            {
                let _ = paths;
                Ok(err_vm(VMValue::Str(
                    "IO.read_files_batch: not supported on wasm32".to_string(),
                )))
            }
        }
        "IO.write_file_raw" => {
            let mut it = args.into_iter();
            let path = match it
                .next()
                .ok_or_else(|| "IO.write_file_raw requires 2 arguments".to_string())?
            {
                VMValue::Str(s) => s,
                _ => return Err("IO.write_file_raw: path must be a String".to_string()),
            };
            let content = match it
                .next()
                .ok_or_else(|| "IO.write_file_raw requires 2 arguments".to_string())?
            {
                VMValue::Str(s) => s,
                _ => return Err("IO.write_file_raw: content must be a String".to_string()),
            };
            match std::fs::write(&path, content.as_bytes()) {
                Ok(()) => Ok(ok_vm(VMValue::Unit)),
                Err(e) => Ok(err_vm(VMValue::Str(e.to_string()))),
            }
        }
        "IO.make_dir_raw" => {
            let path = match args
                .into_iter()
                .next()
                .ok_or_else(|| "IO.make_dir_raw requires 1 argument".to_string())?
            {
                VMValue::Str(s) => s,
                _ => return Err("IO.make_dir_raw: path must be a String".to_string()),
            };
            match std::fs::create_dir_all(&path) {
                Ok(()) => Ok(ok_vm(VMValue::Unit)),
                Err(e) => Ok(err_vm(VMValue::Str(e.to_string()))),
            }
        }
        "IO.write_bytes_raw" => {
            let mut it = args.into_iter();
            let path = match it
                .next()
                .ok_or_else(|| "IO.write_bytes_raw requires 2 arguments".to_string())?
            {
                VMValue::Str(s) => s,
                _ => return Err("IO.write_bytes_raw: path must be a String".to_string()),
            };
            let bytes_val = it
                .next()
                .ok_or_else(|| "IO.write_bytes_raw requires 2 arguments".to_string())?;
            let bytes: Vec<u8> = match bytes_val {
                VMValue::List(fl) => fl
                    .into_iter()
                    .map(|v| match v {
                        VMValue::Int(n) => Ok((n & 0xFF) as u8),
                        _ => Err("IO.write_bytes_raw: list elements must be Int".to_string()),
                    })
                    .collect::<Result<Vec<u8>, String>>()?,
                _ => return Err("IO.write_bytes_raw: bytes must be a List<Int>".to_string()),
            };
            match std::fs::write(&path, &bytes) {
                Ok(()) => Ok(ok_vm(VMValue::Unit)),
                Err(e) => Ok(err_vm(VMValue::Str(e.to_string()))),
            }
        }
        "IO.file_exists_raw" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "IO.file_exists_raw requires 1 argument".to_string())?;
            let path = match v {
                VMValue::Str(s) => s,
                _ => return Err("IO.file_exists_raw requires a String path".to_string()),
            };
            Ok(VMValue::Bool(std::path::Path::new(&path).is_file()))
        }
        "IO.list_dir_raw" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "IO.list_dir_raw requires 1 argument".to_string())?;
            let path = match v {
                VMValue::Str(s) => s,
                _ => return Err("IO.list_dir_raw requires a String path".to_string()),
            };
            match std::fs::read_dir(&path) {
                Ok(entries) => {
                    let mut names: Vec<VMValue> = Vec::new();
                    for entry in entries.flatten() {
                        if let Some(name) = entry.file_name().to_str() {
                            names.push(VMValue::Str(name.to_string()));
                        }
                    }
                    names.sort_by(|a, b| {
                        let sa = if let VMValue::Str(s) = a { s.as_str() } else { "" };
                        let sb = if let VMValue::Str(s) = b { s.as_str() } else { "" };
                        sa.cmp(sb)
                    });
                    Ok(ok_vm(VMValue::List(FavList::new(names))))
                }
                Err(e) => Ok(err_vm(VMValue::Str(format!("IO.list_dir_raw: {}", e)))),
            }
        }
        "IO.file_stat_raw" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "IO.file_stat_raw requires 1 argument".to_string())?;
            let path = match v {
                VMValue::Str(s) => s,
                _ => return Err("IO.file_stat_raw requires a String path".to_string()),
            };
            let p = std::path::Path::new(&path);
            let mut map = std::collections::HashMap::new();
            let exists = p.exists();
            map.insert("exists".to_string(), VMValue::Str(if exists { "true" } else { "false" }.to_string()));
            if exists {
                let is_dir = p.is_dir();
                map.insert("is_dir".to_string(), VMValue::Str(if is_dir { "true" } else { "false" }.to_string()));
                let size_str = std::fs::metadata(p)
                    .map(|m| m.len().to_string())
                    .unwrap_or_else(|_| "0".to_string());
                map.insert("size".to_string(), VMValue::Str(size_str));
            } else {
                map.insert("is_dir".to_string(), VMValue::Str("false".to_string()));
                map.insert("size".to_string(), VMValue::Str("0".to_string()));
            }
            Ok(VMValue::Record(map))
        }
        "IO.path_join_raw" => {
            let mut it = args.into_iter();
            let base = vm_string(it.next().ok_or("IO.path_join_raw: missing base")?, "IO.path_join_raw")?;
            let seg  = vm_string(it.next().ok_or("IO.path_join_raw: missing segment")?, "IO.path_join_raw")?;
            let joined = std::path::Path::new(&base)
                .join(&seg)
                .to_string_lossy()
                .to_string();
            Ok(VMValue::Str(joined))
        }
        "IO.home_dir_raw" => {
            let home = std::env::var("HOME")
                .or_else(|_| std::env::var("USERPROFILE"))
                .ok();
            Ok(match home {
                Some(p) => VMValue::Variant("some".into(), Some(Box::new(VMValue::Str(p)))),
                None    => VMValue::Variant("none".into(), None),
            })
        }
        "IO.cwd_raw" => {
            let cwd = std::env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| ".".to_string());
            Ok(VMValue::Str(cwd))
        }
        "IO.is_dir_raw" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "IO.is_dir_raw requires 1 argument".to_string())?;
            let path = vm_string(v, "IO.is_dir_raw")?;
            Ok(VMValue::Bool(std::path::Path::new(&path).is_dir()))
        }
        // CLI primitives (v7.6.0)
        "IO.write_stderr_raw" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "IO.write_stderr_raw requires 1 argument".to_string())?;
            let msg = vm_string(v, "IO.write_stderr_raw")?;
            if !is_io_suppressed() {
                eprintln!("{}", msg);
            }
            Ok(VMValue::Unit)
        }
        "IO.exit_raw" => {
            let code = match args.into_iter().next() {
                Some(VMValue::Int(n)) => n as i32,
                _ => 1,
            };
            std::process::exit(code)
        }
        "Compiler.check_raw" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "Compiler.check_raw requires 1 argument".to_string())?;
            let path = vm_string(v, "Compiler.check_raw")?;
            let src = match std::fs::read_to_string(&path) {
                Err(e) => {
                    return Ok(err_vm(VMValue::Str(format!("cannot read {}: {}", path, e))))
                }
                Ok(s) => s,
            };
            let program = match crate::frontend::parser::Parser::parse_str(&src, &path) {
                Err(e) => return Ok(err_vm(VMValue::Str(e.to_string()))),
                Ok(p) => p,
            };
            let prog_vm = crate::middle::ast_lower_checker::lower_program(&program);
            match crate::checker_fav_runner::run_checker_fav(prog_vm) {
                Ok(()) => Ok(ok_vm(VMValue::Str("compiled".to_string()))),
                Err(msgs) => Ok(err_vm(VMValue::Str(msgs.join("\n")))),
            }
        }
        "Compiler.lineage_text_raw" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "Compiler.lineage_text_raw requires 1 argument".to_string())?;
            let path = vm_string(v, "Compiler.lineage_text_raw")?;
            let src = match std::fs::read_to_string(&path) {
                Err(e) => return Ok(VMValue::Str(format!("error: cannot read {}: {}", path, e))),
                Ok(s) => s,
            };
            let program = match crate::frontend::parser::Parser::parse_str(&src, &path) {
                Err(e) => return Ok(VMValue::Str(format!("error: parse failed: {}", e))),
                Ok(p) => p,
            };
            let report = crate::lineage::lineage_analysis(&program);
            let text = crate::lineage::render_lineage_text(&report, &path);
            Ok(VMValue::Str(text))
        }
        "Compiler.fmt_source_raw" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "Compiler.fmt_source_raw requires 1 argument".to_string())?;
            let src = vm_string(v, "Compiler.fmt_source_raw")?;
            match crate::compiler_fav_runner::fmt_source_str(&src) {
                Ok(formatted) => Ok(ok_vm(VMValue::Str(formatted))),
                Err(msg) => Ok(err_vm(VMValue::Str(msg))),
            }
        }
        "Compiler.lint_source_raw" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "Compiler.lint_source_raw requires 1 argument".to_string())?;
            let src = vm_string(v, "Compiler.lint_source_raw")?;
            match crate::compiler_fav_runner::lint_source_str(&src) {
                Ok(warnings) => Ok(ok_vm(VMValue::Str(warnings))),
                Err(msg) => Ok(err_vm(VMValue::Str(msg))),
            }
        }
        "Compiler.doc_source_raw" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "Compiler.doc_source_raw requires 1 argument".to_string())?;
            let src = vm_string(v, "Compiler.doc_source_raw")?;
            match crate::compiler_fav_runner::doc_source_str(&src) {
                Ok(markdown) => Ok(ok_vm(VMValue::Str(markdown))),
                Err(msg) => Ok(err_vm(VMValue::Str(msg))),
            }
        }
        "Compiler.compile_source_profiled_raw" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "Compiler.compile_source_profiled_raw requires 1 argument".to_string())?;
            let src = vm_string(v, "Compiler.compile_source_profiled_raw")?;
            match crate::compiler_fav_runner::compile_profiled_str(&src) {
                Ok(bytes) => {
                    let int_list: Vec<VMValue> = bytes.iter().map(|b| VMValue::Int(*b as i64)).collect();
                    Ok(ok_vm(VMValue::List(FavList::new(int_list))))
                }
                Err(msg) => Ok(err_vm(VMValue::Str(msg))),
            }
        }
        // ── Env.now_ms_raw / profile primitives (v9.9.0) ─────────────────────
        "Env.now_ms_raw" => {
            let ms = chrono::Utc::now().timestamp_millis();
            Ok(VMValue::Int(ms))
        }
        "Env.profile_timed_raw" => {
            // Signature: profile_timed_raw(name: String, start_ms: Int, result: Any) -> Any
            // Args arrive after stack pop + reverse: [name, start_ms, result].
            // Evaluation order: name (literal) → start_ms (now_ms_raw, before stage) → result (stage call).
            let mut it = args.into_iter();
            let name = vm_string(it.next().ok_or_else(|| "Env.profile_timed_raw: missing name".to_string())?, "Env.profile_timed_raw")?;
            let start_ms = vm_int(it.next().ok_or_else(|| "Env.profile_timed_raw: missing start_ms".to_string())?, "Env.profile_timed_raw")?;
            let result = it.next().ok_or_else(|| "Env.profile_timed_raw: missing result".to_string())?;
            let elapsed = chrono::Utc::now().timestamp_millis() - start_ms;
            PROFILE_RECORDS.with(|r| r.borrow_mut().push((name, elapsed)));
            Ok(result)
        }
        "Env.profile_dump_raw" => {
            Ok(VMValue::Str(crate::backend::vm::take_profile_dump_json()))
        }
        // ── IO.file_mtime_raw / IO.sleep_ms_raw (v9.9.0) ──────────────────
        "IO.file_mtime_raw" => {
            let v = args.into_iter().next().ok_or_else(|| "IO.file_mtime_raw requires 1 argument".to_string())?;
            let path = vm_string(v, "IO.file_mtime_raw")?;
            match std::fs::metadata(&path) {
                Ok(meta) => match meta.modified() {
                    Ok(t) => {
                        let ms = t.duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis() as i64).unwrap_or(0);
                        Ok(ok_vm(VMValue::Int(ms)))
                    }
                    Err(e) => Ok(err_vm(VMValue::Str(e.to_string()))),
                },
                Err(e) => Ok(err_vm(VMValue::Str(e.to_string()))),
            }
        }
        "IO.sleep_ms_raw" => {
            let v = args.into_iter().next().ok_or_else(|| "IO.sleep_ms_raw requires 1 argument".to_string())?;
            let ms = vm_int(v, "IO.sleep_ms_raw")?;
            std::thread::sleep(std::time::Duration::from_millis(ms.max(0) as u64));
            Ok(VMValue::Unit)
        }
        // ── Debug.show_raw (v9.10.0) ──────────────────────────────────────────
        "Debug.show_raw" => {
            let v = args.into_iter().next()
                .ok_or_else(|| "Debug.show_raw: missing argument".to_string())?;
            Ok(VMValue::Str(display_vmvalue(&v)))
        }
        "IO.argv" => {
            let argv: Vec<VMValue> = TEST_ARGV.with(|t| {
                if let Some(ref args) = *t.borrow() {
                    args.iter().map(|a| VMValue::Str(a.clone())).collect()
                } else {
                    std::env::args()
                        .skip_while(|a| a != "--")
                        .skip(1)
                        .map(|a| VMValue::Str(a))
                        .collect()
                }
            });
            Ok(VMValue::List(FavList::new(argv)))
        }


        "Debug.show" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "Debug.show requires 1 argument".to_string())?;
            Ok(VMValue::Str(vmvalue_repr(&v)))
        }
        "assert" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "assert requires 1 argument".to_string())?;
            match v {
                VMValue::Bool(true) => Ok(VMValue::Unit),
                VMValue::Bool(false) => Err("assertion failed".to_string()),
                other => Err(format!(
                    "assert requires Bool, got {}",
                    vmvalue_type_name(&other)
                )),
            }
        }
        "assert_ne" => {
            let mut it = args.into_iter();
            let a = it
                .next()
                .ok_or_else(|| "assert_ne requires 2 arguments".to_string())?;
            let b = it
                .next()
                .ok_or_else(|| "assert_ne requires 2 arguments".to_string())?;
            if vmvalue_repr(&a) != vmvalue_repr(&b) {
                Ok(VMValue::Unit)
            } else {
                Err(format!(
                    "assert_ne failed: both equal to {}",
                    vmvalue_repr(&a)
                ))
            }
        }
        // ── v15.3.0 assert primitives ──────────────────────────────────────────
        "assert_ok" => {
            let r = args.into_iter().next()
                .ok_or_else(|| "assert_ok requires 1 argument".to_string())?;
            match r {
                VMValue::Variant(ref tag, ref inner) if tag == "ok" => {
                    Ok(inner.as_deref().cloned().unwrap_or(VMValue::Unit))
                }
                VMValue::Variant(ref tag, ref inner) if tag == "err" => {
                    let msg = inner.as_deref().map(|v| vmvalue_repr(v)).unwrap_or_default();
                    Err(format!("assert_ok failed: got err({msg})"))
                }
                other => Err(format!("assert_ok failed: not a Result: {}", vmvalue_repr(&other))),
            }
        }
        "assert_err" => {
            let r = args.into_iter().next()
                .ok_or_else(|| "assert_err requires 1 argument".to_string())?;
            match r {
                VMValue::Variant(ref tag, ref inner) if tag == "err" => {
                    Ok(inner.as_deref().cloned().unwrap_or(VMValue::Unit))
                }
                VMValue::Variant(ref tag, ref inner) if tag == "ok" => {
                    let msg = inner.as_deref().map(|v| vmvalue_repr(v)).unwrap_or_default();
                    Err(format!("assert_err failed: got ok({msg})"))
                }
                other => Err(format!("assert_err failed: not a Result: {}", vmvalue_repr(&other))),
            }
        }
        "assert_true" => {
            let b = args.into_iter().next()
                .ok_or_else(|| "assert_true requires 1 argument".to_string())?;
            match b {
                VMValue::Bool(true) => Ok(VMValue::Unit),
                VMValue::Bool(false) => Err("assert_true failed: got false".to_string()),
                other => Err(format!("assert_true failed: not a Bool: {}", vmvalue_repr(&other))),
            }
        }
        // ── v16.7.0 assert primitives ──────────────────────────────────────────
        "assert_eq" => {
            let mut it = args.into_iter();
            let actual   = it.next().ok_or_else(|| "assert_eq requires 2 arguments".to_string())?;
            let expected = it.next().ok_or_else(|| "assert_eq requires 2 arguments".to_string())?;
            let a_str = vmvalue_repr(&actual);
            let e_str = vmvalue_repr(&expected);
            if a_str == e_str {
                Ok(VMValue::Unit)
            } else {
                Err(format!(
                    "assert_eq failed:\n  actual:   {}\n  expected: {}",
                    a_str, e_str
                ))
            }
        }
        "assert_approx_eq" => {
            let mut it = args.into_iter();
            let actual   = it.next().ok_or_else(|| "assert_approx_eq requires 3 arguments".to_string())?;
            let expected = it.next().ok_or_else(|| "assert_approx_eq requires 3 arguments".to_string())?;
            let epsilon  = it.next().ok_or_else(|| "assert_approx_eq requires 3 arguments".to_string())?;
            let a   = match &actual   { VMValue::Float(f) => *f, VMValue::Int(i) => *i as f64, _ => return Err(format!("assert_approx_eq: actual must be a number, got {}", vmvalue_repr(&actual))) };
            let e   = match &expected { VMValue::Float(f) => *f, VMValue::Int(i) => *i as f64, _ => return Err(format!("assert_approx_eq: expected must be a number, got {}", vmvalue_repr(&expected))) };
            let eps = match &epsilon  { VMValue::Float(f) => *f, VMValue::Int(i) => *i as f64, _ => return Err(format!("assert_approx_eq: epsilon must be a number, got {}", vmvalue_repr(&epsilon))) };
            if (a - e).abs() <= eps {
                Ok(VMValue::Unit)
            } else {
                Err(format!(
                    "assert_approx_eq failed:\n  actual:   {}\n  expected: {}\n  epsilon:  {}",
                    a, e, eps
                ))
            }
        }
        "assert_contains" => {
            let mut it = args.into_iter();
            let list = it.next().ok_or_else(|| "assert_contains requires 2 arguments".to_string())?;
            let elem = it.next().ok_or_else(|| "assert_contains requires 2 arguments".to_string())?;
            let elem_str = vmvalue_repr(&elem);
            match list {
                VMValue::List(items) => {
                    if items.iter().any(|v| vmvalue_repr(v) == elem_str) {
                        Ok(VMValue::Unit)
                    } else {
                        Err(format!("assert_contains failed: element {} not found in list", elem_str))
                    }
                }
                other => Err(format!("assert_contains: first argument must be a List, got {}", vmvalue_repr(&other))),
            }
        }
        "assert_length" => {
            let mut it = args.into_iter();
            let list = it.next().ok_or_else(|| "assert_length requires 2 arguments".to_string())?;
            let n    = it.next().ok_or_else(|| "assert_length requires 2 arguments".to_string())?;
            let expected_len = match n { VMValue::Int(i) => i as usize, other => return Err(format!("assert_length: second argument must be Int, got {}", vmvalue_repr(&other))) };
            match list {
                VMValue::List(items) => {
                    if items.len() == expected_len {
                        Ok(VMValue::Unit)
                    } else {
                        Err(format!("assert_length failed:\n  actual:   {}\n  expected: {}", items.len(), expected_len))
                    }
                }
                other => Err(format!("assert_length: first argument must be a List, got {}", vmvalue_repr(&other))),
            }
        }
        "assert_str_contains" => {
            let mut it = args.into_iter();
            let s   = it.next().ok_or_else(|| "assert_str_contains requires 2 arguments".to_string())?;
            let sub = it.next().ok_or_else(|| "assert_str_contains requires 2 arguments".to_string())?;
            match (s, sub) {
                (VMValue::Str(s), VMValue::Str(sub)) => {
                    if s.contains(sub.as_str()) {
                        Ok(VMValue::Unit)
                    } else {
                        Err(format!("assert_str_contains failed: {:?} does not contain {:?}", s, sub))
                    }
                }
                (s, sub) => Err(format!("assert_str_contains: both arguments must be String, got {} and {}", vmvalue_repr(&s), vmvalue_repr(&sub))),
            }
        }
        "assert_str_starts_with" => {
            let mut it = args.into_iter();
            let s      = it.next().ok_or_else(|| "assert_str_starts_with requires 2 arguments".to_string())?;
            let prefix = it.next().ok_or_else(|| "assert_str_starts_with requires 2 arguments".to_string())?;
            match (s, prefix) {
                (VMValue::Str(s), VMValue::Str(prefix)) => {
                    if s.starts_with(prefix.as_str()) {
                        Ok(VMValue::Unit)
                    } else {
                        Err(format!("assert_str_starts_with failed: {:?} does not start with {:?}", s, prefix))
                    }
                }
                (s, prefix) => Err(format!("assert_str_starts_with: both arguments must be String, got {} and {}", vmvalue_repr(&s), vmvalue_repr(&prefix))),
            }
        }
        "assert_err_eq" => {
            let mut it = args.into_iter();
            let result   = it.next().ok_or_else(|| "assert_err_eq requires 2 arguments".to_string())?;
            let expected = it.next().ok_or_else(|| "assert_err_eq requires 2 arguments".to_string())?;
            let expected_str = match &expected { VMValue::Str(s) => s.clone(), other => vmvalue_repr(other) };
            match result {
                VMValue::Variant(ref tag, ref inner) if tag == "err" => {
                    let actual_str = inner.as_deref().map(|v| match v { VMValue::Str(s) => s.clone(), other => vmvalue_repr(other) }).unwrap_or_default();
                    if actual_str == expected_str {
                        Ok(VMValue::Unit)
                    } else {
                        Err(format!("assert_err_eq failed:\n  actual:   {:?}\n  expected: {:?}", actual_str, expected_str))
                    }
                }
                VMValue::Variant(ref tag, _) if tag == "ok" => {
                    Err(format!("assert_err_eq failed: got ok, expected err({:?})", expected_str))
                }
                other => Err(format!("assert_err_eq: first argument must be a Result, got {}", vmvalue_repr(&other))),
            }
        }
        "assert_snapshot" => {
            let mut it = args.into_iter();
            let value = it.next().ok_or_else(|| "assert_snapshot requires 2 arguments".to_string())?;
            let name  = it.next().ok_or_else(|| "assert_snapshot requires 2 arguments".to_string())?;
            let snap_name = match name { VMValue::Str(s) => s, other => return Err(format!("assert_snapshot: second argument must be String, got {}", vmvalue_repr(&other))) };
            let snap_content = vmvalue_repr(&value);
            let snap_dir = std::path::Path::new(".snap");
            if !snap_dir.exists() {
                std::fs::create_dir_all(snap_dir)
                    .map_err(|e| format!("assert_snapshot: failed to create .snap/ directory: {}", e))?;
            }
            let snap_path = snap_dir.join(format!("{}.snap", snap_name));
            let update = std::env::var("UPDATE_SNAPSHOTS").unwrap_or_default() == "1";
            if !snap_path.exists() || update {
                std::fs::write(&snap_path, &snap_content)
                    .map_err(|e| format!("assert_snapshot: failed to write snapshot: {}", e))?;
                Ok(VMValue::Unit)
            } else {
                let stored = std::fs::read_to_string(&snap_path)
                    .map_err(|e| format!("assert_snapshot: failed to read snapshot: {}", e))?;
                if stored == snap_content {
                    Ok(VMValue::Unit)
                } else {
                    Err(format!(
                        "assert_snapshot failed for {:?}:\n  stored:  {}\n  current: {}",
                        snap_name, stored, snap_content
                    ))
                }
            }
        }
        "inspect_debug" => {
            let val = args.into_iter().next().ok_or_else(|| "inspect_debug requires 1 argument".to_string())?;
            println!("[inspect] {}", vmvalue_repr(&val));
            Ok(VMValue::Unit)
        }
        "Result.ok" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "Result.ok requires 1 argument".to_string())?;
            Ok(VMValue::Variant("ok".to_string(), Some(Box::new(v))))
        }
        "Result.err" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "Result.err requires 1 argument".to_string())?;
            Ok(VMValue::Variant("err".to_string(), Some(Box::new(v))))
        }
        "Option.some" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "Option.some requires 1 argument".to_string())?;
            Ok(VMValue::Variant("some".to_string(), Some(Box::new(v))))
        }
        "Option.none" => Ok(VMValue::Variant("none".to_string(), None)),
        "Int.to_string" | "Float.to_string" | "Int.show.show" | "Float.show.show" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| format!("{} requires 1 argument", name))?;
            Ok(VMValue::Str(match v {
                VMValue::Int(n) => n.to_string(),
                VMValue::Float(f) => {
                    if f.fract() == 0.0 {
                        format!("{:.1}", f)
                    } else {
                        f.to_string()
                    }
                }
                other => return Err(format!("{} requires Int/Float, got {:?}", name, other)),
            }))
        }
        "Bool.show.show" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "Bool.show.show requires 1 argument".to_string())?;
            Ok(VMValue::Str(match v {
                VMValue::Bool(b) => b.to_string(),
                other => return Err(format!("Bool.show.show requires Bool, got {:?}", other)),
            }))
        }
        "String.show.show" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "String.show.show requires 1 argument".to_string())?;
            Ok(VMValue::Str(match v {
                VMValue::Str(s) => format!("\"{}\"", s),
                other => return Err(format!("String.show.show requires String, got {:?}", other)),
            }))
        }
        "Int.ord.compare" => {
            let mut it = args.into_iter();
            let a = it
                .next()
                .ok_or_else(|| "Int.ord.compare requires 2 arguments".to_string())?;
            let b = it
                .next()
                .ok_or_else(|| "Int.ord.compare requires 2 arguments".to_string())?;
            match (a, b) {
                (VMValue::Int(x), VMValue::Int(y)) => Ok(VMValue::Int(match x.cmp(&y) {
                    std::cmp::Ordering::Less => -1,
                    std::cmp::Ordering::Equal => 0,
                    std::cmp::Ordering::Greater => 1,
                })),
                _ => Err("Int.ord.compare requires two Int arguments".to_string()),
            }
        }
        "Int.eq.equals" => {
            let mut it = args.into_iter();
            let a = it
                .next()
                .ok_or_else(|| "Int.eq.equals requires 2 arguments".to_string())?;
            let b = it
                .next()
                .ok_or_else(|| "Int.eq.equals requires 2 arguments".to_string())?;
            match (a, b) {
                (VMValue::Int(x), VMValue::Int(y)) => Ok(VMValue::Bool(x == y)),
                _ => Err("Int.eq.equals requires two Int arguments".to_string()),
            }
        }
        // ── Bit operations (v5.1.0) ──────────────────────────────────────────
        "Int.shl" => {
            let mut it = args.into_iter();
            let x = it
                .next()
                .ok_or_else(|| "Int.shl requires 2 arguments".to_string())?;
            let n = it
                .next()
                .ok_or_else(|| "Int.shl requires 2 arguments".to_string())?;
            match (x, n) {
                (VMValue::Int(x), VMValue::Int(n)) => Ok(VMValue::Int(x << n)),
                _ => Err("Int.shl requires two Int arguments".to_string()),
            }
        }
        "Int.shr" => {
            let mut it = args.into_iter();
            let x = it
                .next()
                .ok_or_else(|| "Int.shr requires 2 arguments".to_string())?;
            let n = it
                .next()
                .ok_or_else(|| "Int.shr requires 2 arguments".to_string())?;
            match (x, n) {
                (VMValue::Int(x), VMValue::Int(n)) => Ok(VMValue::Int(x >> n)),
                _ => Err("Int.shr requires two Int arguments".to_string()),
            }
        }
        "Int.band" => {
            let mut it = args.into_iter();
            let x = it
                .next()
                .ok_or_else(|| "Int.band requires 2 arguments".to_string())?;
            let y = it
                .next()
                .ok_or_else(|| "Int.band requires 2 arguments".to_string())?;
            match (x, y) {
                (VMValue::Int(x), VMValue::Int(y)) => Ok(VMValue::Int(x & y)),
                _ => Err("Int.band requires two Int arguments".to_string()),
            }
        }
        "Int.bor" => {
            let mut it = args.into_iter();
            let x = it
                .next()
                .ok_or_else(|| "Int.bor requires 2 arguments".to_string())?;
            let y = it
                .next()
                .ok_or_else(|| "Int.bor requires 2 arguments".to_string())?;
            match (x, y) {
                (VMValue::Int(x), VMValue::Int(y)) => Ok(VMValue::Int(x | y)),
                _ => Err("Int.bor requires two Int arguments".to_string()),
            }
        }
        "Int.bxor" => {
            let mut it = args.into_iter();
            let x = it
                .next()
                .ok_or_else(|| "Int.bxor requires 2 arguments".to_string())?;
            let y = it
                .next()
                .ok_or_else(|| "Int.bxor requires 2 arguments".to_string())?;
            match (x, y) {
                (VMValue::Int(x), VMValue::Int(y)) => Ok(VMValue::Int(x ^ y)),
                _ => Err("Int.bxor requires two Int arguments".to_string()),
            }
        }
        "Int.bnot" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "Int.bnot requires 1 argument".to_string())?;
            match v {
                VMValue::Int(x) => Ok(VMValue::Int(!x)),
                _ => Err("Int.bnot requires an Int argument".to_string()),
            }
        }
        // v23.2.0: public API names for bit operations
        "Int.bit_and" => {
            let mut it = args.into_iter();
            let x = it.next().ok_or_else(|| "Int.bit_and requires 2 arguments".to_string())?;
            let y = it.next().ok_or_else(|| "Int.bit_and requires 2 arguments".to_string())?;
            match (x, y) {
                (VMValue::Int(x), VMValue::Int(y)) => Ok(VMValue::Int(x & y)),
                _ => Err("Int.bit_and requires two Int arguments".to_string()),
            }
        }
        "Int.bit_or" => {
            let mut it = args.into_iter();
            let x = it.next().ok_or_else(|| "Int.bit_or requires 2 arguments".to_string())?;
            let y = it.next().ok_or_else(|| "Int.bit_or requires 2 arguments".to_string())?;
            match (x, y) {
                (VMValue::Int(x), VMValue::Int(y)) => Ok(VMValue::Int(x | y)),
                _ => Err("Int.bit_or requires two Int arguments".to_string()),
            }
        }
        "Int.bit_xor" => {
            let mut it = args.into_iter();
            let x = it.next().ok_or_else(|| "Int.bit_xor requires 2 arguments".to_string())?;
            let y = it.next().ok_or_else(|| "Int.bit_xor requires 2 arguments".to_string())?;
            match (x, y) {
                (VMValue::Int(x), VMValue::Int(y)) => Ok(VMValue::Int(x ^ y)),
                _ => Err("Int.bit_xor requires two Int arguments".to_string()),
            }
        }
        "Int.bit_not" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "Int.bit_not requires 1 argument".to_string())?;
            match v {
                VMValue::Int(x) => Ok(VMValue::Int(!x)),
                _ => Err("Int.bit_not requires an Int argument".to_string()),
            }
        }
        "Int.shift_left" => {
            let mut it = args.into_iter();
            let x = it.next().ok_or_else(|| "Int.shift_left requires 2 arguments".to_string())?;
            let n = it.next().ok_or_else(|| "Int.shift_left requires 2 arguments".to_string())?;
            match (x, n) {
                (VMValue::Int(x), VMValue::Int(n)) => {
                    if n < 0 || n >= 64 {
                        return Err(format!(
                            "Int.shift_left: shift amount {} out of range 0..=63",
                            n
                        ));
                    }
                    Ok(VMValue::Int(x << n))
                }
                _ => Err("Int.shift_left requires two Int arguments".to_string()),
            }
        }
        "Int.shift_right" => {
            let mut it = args.into_iter();
            let x = it.next().ok_or_else(|| "Int.shift_right requires 2 arguments".to_string())?;
            let n = it.next().ok_or_else(|| "Int.shift_right requires 2 arguments".to_string())?;
            match (x, n) {
                (VMValue::Int(x), VMValue::Int(n)) => {
                    if n < 0 || n >= 64 {
                        return Err(format!(
                            "Int.shift_right: shift amount {} out of range 0..=63",
                            n
                        ));
                    }
                    Ok(VMValue::Int(x >> n))
                }
                _ => Err("Int.shift_right requires two Int arguments".to_string()),
            }
        }
        "Int.to_byte" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "Int.to_byte requires 1 argument".to_string())?;
            match v {
                VMValue::Int(x) => Ok(VMValue::Int(x & 0xFF)),
                _ => Err("Int.to_byte requires an Int argument".to_string()),
            }
        }

        "String.concat" => {
            let mut it = args.into_iter();
            let a = it
                .next()
                .ok_or_else(|| "String.concat requires 2 arguments".to_string())?;
            let b = it
                .next()
                .ok_or_else(|| "String.concat requires 2 arguments".to_string())?;
            match (a, b) {
                (VMValue::Str(x), VMValue::Str(y)) => Ok(VMValue::Str(x + &y)),
                _ => Err("String.concat requires two String arguments".to_string()),
            }
        }
        "String.length" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "String.length requires 1 argument".to_string())?;
            match v {
                VMValue::Str(s) => Ok(VMValue::Int(s.len() as i64)),
                _ => Err("String.length requires a String argument".to_string()),
            }
        }
        "String.is_empty" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "String.is_empty requires 1 argument".to_string())?;
            match v {
                VMValue::Str(s) => Ok(VMValue::Bool(s.is_empty())),
                _ => Err("String.is_empty requires a String argument".to_string()),
            }
        }
        "String.trim" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "String.trim requires 1 argument".to_string())?;
            match v {
                VMValue::Str(s) => Ok(VMValue::Str(s.trim().to_string())),
                _ => Err("String.trim requires a String argument".to_string()),
            }
        }
        "String.upper" | "String.to_upper" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "String.upper requires 1 argument".to_string())?;
            match v {
                VMValue::Str(s) => Ok(VMValue::Str(s.to_uppercase())),
                _ => Err("String.upper requires a String argument".to_string()),
            }
        }
        "String.lower" | "String.to_lower" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "String.lower requires 1 argument".to_string())?;
            match v {
                VMValue::Str(s) => Ok(VMValue::Str(s.to_lowercase())),
                _ => Err("String.lower requires a String argument".to_string()),
            }
        }
        "String.split" => {
            let mut it = args.into_iter();
            let s = it
                .next()
                .ok_or_else(|| "String.split requires 2 arguments".to_string())?;
            let d = it
                .next()
                .ok_or_else(|| "String.split requires 2 arguments".to_string())?;
            match (s, d) {
                (VMValue::Str(s), VMValue::Str(delim)) => Ok(VMValue::List(FavList::new(
                    s.split(&*delim)
                        .map(|p| VMValue::Str(p.to_string()))
                        .collect(),
                ))),
                _ => Err("String.split requires (String, String)".to_string()),
            }
        }
        "String.join" => {
            let mut it = args.into_iter();
            let xs = it
                .next()
                .ok_or_else(|| "String.join requires 2 arguments".to_string())?;
            let sep = it
                .next()
                .ok_or_else(|| "String.join requires 2 arguments".to_string())?;
            match (xs, sep) {
                (VMValue::List(fl), VMValue::Str(sep)) => {
                    let mut parts = Vec::with_capacity(fl.len());
                    for value in fl {
                        match value {
                            VMValue::Str(s) => parts.push(s),
                            _ => {
                                return Err("String.join requires List<String> as first argument"
                                    .to_string());
                            }
                        }
                    }
                    Ok(VMValue::Str(parts.join(&sep)))
                }
                _ => Err("String.join requires (List<String>, String)".to_string()),
            }
        }
        "String.replace" => {
            let mut it = args.into_iter();
            let s = it
                .next()
                .ok_or_else(|| "String.replace requires 3 arguments".to_string())?;
            let from = it
                .next()
                .ok_or_else(|| "String.replace requires 3 arguments".to_string())?;
            let to = it
                .next()
                .ok_or_else(|| "String.replace requires 3 arguments".to_string())?;
            match (s, from, to) {
                (VMValue::Str(s), VMValue::Str(from), VMValue::Str(to)) => {
                    Ok(VMValue::Str(s.replace(&from, &to)))
                }
                _ => Err("String.replace requires (String, String, String)".to_string()),
            }
        }
        "String.index_of" => {
            let mut it = args.into_iter();
            let s = it
                .next()
                .ok_or_else(|| "String.index_of requires 2 arguments".to_string())?;
            let needle = it
                .next()
                .ok_or_else(|| "String.index_of requires 2 arguments".to_string())?;
            match (s, needle) {
                (VMValue::Str(s), VMValue::Str(needle)) => Ok(match s.find(&needle) {
                    Some(i) => {
                        VMValue::Variant("some".to_string(), Some(Box::new(VMValue::Int(i as i64))))
                    }
                    None => VMValue::Variant("none".to_string(), None),
                }),
                _ => Err("String.index_of requires (String, String)".to_string()),
            }
        }
        "String.pad_left" => {
            let mut it = args.into_iter();
            let s = it
                .next()
                .ok_or_else(|| "String.pad_left requires 3 arguments".to_string())?;
            let width = it
                .next()
                .ok_or_else(|| "String.pad_left requires 3 arguments".to_string())?;
            let fill = it
                .next()
                .ok_or_else(|| "String.pad_left requires 3 arguments".to_string())?;
            match (s, width, fill) {
                (VMValue::Str(s), VMValue::Int(width), VMValue::Str(fill))
                    if width >= 0 && !fill.is_empty() =>
                {
                    let current = s.chars().count();
                    let width = width as usize;
                    if current >= width {
                        Ok(VMValue::Str(s))
                    } else {
                        let needed = width - current;
                        let prefix: String = fill.chars().cycle().take(needed).collect();
                        Ok(VMValue::Str(format!("{prefix}{s}")))
                    }
                }
                (VMValue::Str(_), VMValue::Int(_), VMValue::Str(fill)) if fill.is_empty() => {
                    Err("String.pad_left requires a non-empty fill string".to_string())
                }
                _ => Err("String.pad_left requires (String, Int, String)".to_string()),
            }
        }
        "String.pad_right" => {
            let mut it = args.into_iter();
            let s = it
                .next()
                .ok_or_else(|| "String.pad_right requires 3 arguments".to_string())?;
            let width = it
                .next()
                .ok_or_else(|| "String.pad_right requires 3 arguments".to_string())?;
            let fill = it
                .next()
                .ok_or_else(|| "String.pad_right requires 3 arguments".to_string())?;
            match (s, width, fill) {
                (VMValue::Str(s), VMValue::Int(width), VMValue::Str(fill))
                    if width >= 0 && !fill.is_empty() =>
                {
                    let current = s.chars().count();
                    let width = width as usize;
                    if current >= width {
                        Ok(VMValue::Str(s))
                    } else {
                        let needed = width - current;
                        let suffix: String = fill.chars().cycle().take(needed).collect();
                        Ok(VMValue::Str(format!("{s}{suffix}")))
                    }
                }
                (VMValue::Str(_), VMValue::Int(_), VMValue::Str(fill)) if fill.is_empty() => {
                    Err("String.pad_right requires a non-empty fill string".to_string())
                }
                _ => Err("String.pad_right requires (String, Int, String)".to_string()),
            }
        }
        "String.reverse" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "String.reverse requires 1 argument".to_string())?;
            match v {
                VMValue::Str(s) => Ok(VMValue::Str(s.chars().rev().collect())),
                _ => Err("String.reverse requires a String argument".to_string()),
            }
        }
        "String.lines" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "String.lines requires 1 argument".to_string())?;
            match v {
                VMValue::Str(s) => Ok(VMValue::List(FavList::new(
                    s.lines()
                        .map(|line| VMValue::Str(line.to_string()))
                        .collect(),
                ))),
                _ => Err("String.lines requires a String argument".to_string()),
            }
        }
        "String.words" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "String.words requires 1 argument".to_string())?;
            match v {
                VMValue::Str(s) => Ok(VMValue::List(FavList::new(
                    s.split_whitespace()
                        .map(|word| VMValue::Str(word.to_string()))
                        .collect(),
                ))),
                _ => Err("String.words requires a String argument".to_string()),
            }
        }
        "String.compare" => {
            let mut it = args.into_iter();
            let a = it
                .next()
                .ok_or_else(|| "String.compare requires 2 arguments".to_string())?;
            let b = it
                .next()
                .ok_or_else(|| "String.compare requires 2 arguments".to_string())?;
            match (a, b) {
                (VMValue::Str(a), VMValue::Str(b)) => Ok(VMValue::Int(match a.cmp(&b) {
                    std::cmp::Ordering::Less => -1,
                    std::cmp::Ordering::Equal => 0,
                    std::cmp::Ordering::Greater => 1,
                })),
                _ => Err("String.compare requires 2 String arguments".to_string()),
            }
        }
        "String.starts_with" => {
            let mut it = args.into_iter();
            let s = it
                .next()
                .ok_or_else(|| "String.starts_with requires 2 arguments".to_string())?;
            let prefix = it
                .next()
                .ok_or_else(|| "String.starts_with requires 2 arguments".to_string())?;
            match (s, prefix) {
                (VMValue::Str(s), VMValue::Str(prefix)) => {
                    Ok(VMValue::Bool(s.starts_with(&prefix)))
                }
                _ => Err("String.starts_with requires (String, String)".to_string()),
            }
        }
        "String.is_url" => {
            let value = args
                .into_iter()
                .next()
                .ok_or_else(|| "String.is_url requires 1 argument".to_string())?;
            match value {
                VMValue::Str(s) => Ok(VMValue::Bool(
                    s.starts_with("http://") || s.starts_with("https://"),
                )),
                _ => Err("String.is_url requires a String argument".to_string()),
            }
        }
        "String.is_slug" => {
            let value = args
                .into_iter()
                .next()
                .ok_or_else(|| "String.is_slug requires 1 argument".to_string())?;
            match value {
                VMValue::Str(s) => Ok(VMValue::Bool(
                    !s.is_empty()
                        && s.chars()
                            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-'),
                )),
                _ => Err("String.is_slug requires a String argument".to_string()),
            }
        }
        "String.ends_with" => {
            let mut it = args.into_iter();
            let s = it
                .next()
                .ok_or_else(|| "String.ends_with requires 2 arguments".to_string())?;
            let suffix = it
                .next()
                .ok_or_else(|| "String.ends_with requires 2 arguments".to_string())?;
            match (s, suffix) {
                (VMValue::Str(s), VMValue::Str(suffix)) => Ok(VMValue::Bool(s.ends_with(&suffix))),
                _ => Err("String.ends_with requires (String, String)".to_string()),
            }
        }
        "String.contains" => {
            let mut it = args.into_iter();
            let s = it
                .next()
                .ok_or_else(|| "String.contains requires 2 arguments".to_string())?;
            let sub = it
                .next()
                .ok_or_else(|| "String.contains requires 2 arguments".to_string())?;
            match (s, sub) {
                (VMValue::Str(s), VMValue::Str(sub)) => Ok(VMValue::Bool(s.contains(&sub))),
                _ => Err("String.contains requires (String, String)".to_string()),
            }
        }
        "String.slice" => {
            let mut it = args.into_iter();
            let s = it
                .next()
                .ok_or_else(|| "String.slice requires 3 arguments".to_string())?;
            let start = it
                .next()
                .ok_or_else(|| "String.slice requires 3 arguments".to_string())?;
            let end = it
                .next()
                .ok_or_else(|| "String.slice requires 3 arguments".to_string())?;
            match (s, start, end) {
                (VMValue::Str(s), VMValue::Int(start), VMValue::Int(end)) => {
                    if start < 0 || end < start {
                        return Err("String.slice requires 0 <= start <= end".to_string());
                    }
                    let chars: Vec<char> = s.chars().collect();
                    let start = start as usize;
                    let end = end as usize;
                    if end > chars.len() {
                        return Err("String.slice end is out of bounds".to_string());
                    }
                    Ok(VMValue::Str(chars[start..end].iter().collect()))
                }
                _ => Err("String.slice requires (String, Int, Int)".to_string()),
            }
        }
        "String.repeat" => {
            let mut it = args.into_iter();
            let s = it
                .next()
                .ok_or_else(|| "String.repeat requires 2 arguments".to_string())?;
            let n = it
                .next()
                .ok_or_else(|| "String.repeat requires 2 arguments".to_string())?;
            match (s, n) {
                (VMValue::Str(s), VMValue::Int(n)) if n >= 0 => {
                    Ok(VMValue::Str(s.repeat(n as usize)))
                }
                (VMValue::Str(_), VMValue::Int(_)) => {
                    Err("String.repeat requires a non-negative count".to_string())
                }
                _ => Err("String.repeat requires (String, Int)".to_string()),
            }
        }
        "String.truncate" => {
            let mut it = args.into_iter();
            let s = it
                .next()
                .ok_or_else(|| "String.truncate requires 3 arguments".to_string())?;
            let max_len = it
                .next()
                .ok_or_else(|| "String.truncate requires 3 arguments".to_string())?;
            let suffix = it
                .next()
                .ok_or_else(|| "String.truncate requires 3 arguments".to_string())?;
            match (s, max_len, suffix) {
                (VMValue::Str(s), VMValue::Int(max), VMValue::Str(suf)) => {
                    let max = max as usize;
                    if s.chars().count() <= max {
                        Ok(VMValue::Str(s))
                    } else {
                        let suf_len = suf.chars().count();
                        let take = max.saturating_sub(suf_len);
                        let truncated: String = s.chars().take(take).collect();
                        Ok(VMValue::Str(format!("{}{}", truncated, suf)))
                    }
                }
                _ => Err("String.truncate requires (String, Int, String)".to_string()),
            }
        }
        "String.trim_start" => match args.into_iter().next() {
            Some(VMValue::Str(s)) => Ok(VMValue::Str(s.trim_start().to_string())),
            Some(other) => Err(format!(
                "String.trim_start requires String, got {}",
                vmvalue_type_name(&other)
            )),
            None => Err("String.trim_start requires 1 argument".to_string()),
        },
        "String.trim_end" => match args.into_iter().next() {
            Some(VMValue::Str(s)) => Ok(VMValue::Str(s.trim_end().to_string())),
            Some(other) => Err(format!(
                "String.trim_end requires String, got {}",
                vmvalue_type_name(&other)
            )),
            None => Err("String.trim_end requires 1 argument".to_string()),
        },
        "String.capitalize" => {
            let value_args: Vec<Value> = args.into_iter().map(Value::from).collect();
            crate::stdlib_fav_runner::call_string_stdlib("capitalize", value_args)
                .map(VMValue::from)
                .map_err(|e| e.message)
        }
        "String.indent" => {
            let value_args: Vec<Value> = args.into_iter().map(Value::from).collect();
            crate::stdlib_fav_runner::call_string_stdlib("indent", value_args)
                .map(VMValue::from)
                .map_err(|e| e.message)
        }
        "String.char_at" => {
            let mut it = args.into_iter();
            let s = it
                .next()
                .ok_or_else(|| "String.char_at requires 2 arguments".to_string())?;
            let idx = it
                .next()
                .ok_or_else(|| "String.char_at requires 2 arguments".to_string())?;
            match (s, idx) {
                (VMValue::Str(s), VMValue::Int(idx)) => {
                    if idx < 0 {
                        return Ok(VMValue::Variant("none".to_string(), None));
                    }
                    let ch = s.chars().nth(idx as usize);
                    Ok(match ch {
                        Some(ch) => VMValue::Variant(
                            "some".to_string(),
                            Some(Box::new(VMValue::Str(ch.to_string()))),
                        ),
                        None => VMValue::Variant("none".to_string(), None),
                    })
                }
                _ => Err("String.char_at requires (String, Int)".to_string()),
            }
        }
        "String.to_int" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "String.to_int requires 1 argument".to_string())?;
            match v {
                VMValue::Str(s) => Ok(match s.parse::<i64>() {
                    Ok(n) => VMValue::Variant("some".to_string(), Some(Box::new(VMValue::Int(n)))),
                    Err(_) => VMValue::Variant("none".to_string(), None),
                }),
                _ => Err("String.to_int requires a String argument".to_string()),
            }
        }
        "String.to_float" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "String.to_float requires 1 argument".to_string())?;
            match v {
                VMValue::Str(s) => Ok(match s.parse::<f64>() {
                    Ok(n) => {
                        VMValue::Variant("some".to_string(), Some(Box::new(VMValue::Float(n))))
                    }
                    Err(_) => VMValue::Variant("none".to_string(), None),
                }),
                _ => Err("String.to_float requires a String argument".to_string()),
            }
        }
        "String.from_chars" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "String.from_chars requires 1 argument".to_string())?;
            match v {
                VMValue::List(fl) => {
                    let mut result = String::new();
                    for c in fl {
                        match c {
                            VMValue::Str(s) => result.push_str(&s),
                            other => {
                                return Err(format!(
                                    "String.from_chars: each element must be String, got {}",
                                    vmvalue_type_name(&other)
                                ));
                            }
                        }
                    }
                    Ok(VMValue::Str(result))
                }
                _ => Err("String.from_chars requires a List<String> argument".to_string()),
            }
        }
        "String.from_int" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "String.from_int requires 1 argument".to_string())?;
            match v {
                VMValue::Int(n) => Ok(VMValue::Str(n.to_string())),
                _ => Err("String.from_int requires an Int argument".to_string()),
            }
        }
        "String.from_float" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "String.from_float requires 1 argument".to_string())?;
            match v {
                VMValue::Float(n) => Ok(VMValue::Str(n.to_string())),
                _ => Err("String.from_float requires a Float argument".to_string()),
            }
        }
        "String.chars" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "String.chars requires 1 argument".to_string())?;
            match v {
                VMValue::Str(s) => {
                    let chars: Vec<VMValue> =
                        s.chars().map(|c| VMValue::Str(c.to_string())).collect();
                    Ok(VMValue::List(FavList::new(chars)))
                }
                _ => Err("String.chars requires a String argument".to_string()),
            }
        }
        "String.to_bytes" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "String.to_bytes requires 1 argument".to_string())?;
            match v {
                VMValue::Str(s) => {
                    let bytes: Vec<VMValue> = s
                        .as_bytes()
                        .iter()
                        .map(|&b| VMValue::Int(b as i64))
                        .collect();
                    Ok(VMValue::List(FavList::new(bytes)))
                }
                _ => Err("String.to_bytes requires a String argument".to_string()),
            }
        }
        "List.length" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "List.length requires 1 argument".to_string())?;
            match v {
                VMValue::List(fl) => Ok(VMValue::Int(fl.len() as i64)),
                _ => Err("List.length requires a List argument".to_string()),
            }
        }
        "List.is_empty" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "List.is_empty requires 1 argument".to_string())?;
            match v {
                VMValue::List(fl) => Ok(VMValue::Bool(fl.is_empty())),
                _ => Err("List.is_empty requires a List argument".to_string()),
            }
        }
        "List.first" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "List.first requires 1 argument".to_string())?;
            match v {
                VMValue::List(fl) => Ok(match fl.first().cloned() {
                    Some(first) => VMValue::Variant("some".to_string(), Some(Box::new(first))),
                    None => VMValue::Variant("none".to_string(), None),
                }),
                _ => Err("List.first requires a List argument".to_string()),
            }
        }
        "List.last" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "List.last requires 1 argument".to_string())?;
            match v {
                VMValue::List(fl) => Ok(match fl.last().cloned() {
                    Some(last) => VMValue::Variant("some".to_string(), Some(Box::new(last))),
                    None => VMValue::Variant("none".to_string(), None),
                }),
                _ => Err("List.last requires a List argument".to_string()),
            }
        }
        "List.empty" => Ok(VMValue::List(FavList::new(vec![]))),
        "List.singleton" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "List.singleton requires 1 argument".to_string())?;
            Ok(VMValue::List(FavList::new(vec![v])))
        }
        "List.push" => {
            let mut it = args.into_iter();
            let list = it
                .next()
                .ok_or_else(|| "List.push requires 2 arguments".to_string())?;
            let item = it
                .next()
                .ok_or_else(|| "List.push requires 2 arguments".to_string())?;
            match list {
                VMValue::List(fl) => {
                    let mut xs = fl.to_vec();
                    xs.push(item);
                    Ok(VMValue::List(FavList::new(xs)))
                }
                _ => Err("List.push requires a List as first argument".to_string()),
            }
        }
        "List.zip" => {
            let mut it = args.into_iter();
            let xs = it
                .next()
                .ok_or_else(|| "List.zip requires 2 arguments".to_string())?;
            let ys = it
                .next()
                .ok_or_else(|| "List.zip requires 2 arguments".to_string())?;
            match (xs, ys) {
                (VMValue::List(fla), VMValue::List(flb)) => {
                    let pairs: Vec<VMValue> = fla
                        .into_iter()
                        .zip(flb.into_iter())
                        .map(|(x, y)| {
                            let mut m = HashMap::new();
                            m.insert("first".to_string(), x);
                            m.insert("second".to_string(), y);
                            VMValue::Record(m)
                        })
                        .collect();
                    Ok(VMValue::List(FavList::new(pairs)))
                }
                _ => Err("List.zip expects (List, List)".to_string()),
            }
        }
        "List.range" => {
            let mut it = args.into_iter();
            let start = it
                .next()
                .ok_or_else(|| "List.range requires 2 arguments".to_string())?;
            let end = it
                .next()
                .ok_or_else(|| "List.range requires 2 arguments".to_string())?;
            match (start, end) {
                (VMValue::Int(s), VMValue::Int(e)) => Ok(VMValue::List(FavList::new(
                    (s..e).map(VMValue::Int).collect(),
                ))),
                _ => Err("List.range expects (Int, Int)".to_string()),
            }
        }
        "List.reverse" => match args.into_iter().next() {
            Some(VMValue::List(fl)) => {
                let mut xs = fl.to_vec();
                xs.reverse();
                Ok(VMValue::List(FavList::new(xs)))
            }
            _ => Err("List.reverse expects List".to_string()),
        },
        "List.concat" => {
            let mut it = args.into_iter();
            let xs = it
                .next()
                .ok_or_else(|| "List.concat requires 2 arguments".to_string())?;
            let ys = it
                .next()
                .ok_or_else(|| "List.concat requires 2 arguments".to_string())?;
            match (xs, ys) {
                (VMValue::List(fla), VMValue::List(flb)) => {
                    let mut xs = fla.to_vec();
                    xs.extend(flb.into_iter());
                    Ok(VMValue::List(FavList::new(xs)))
                }
                _ => Err("List.concat expects (List, List)".to_string()),
            }
        }
        "List.take" => {
            let mut it = args.into_iter();
            let list = it
                .next()
                .ok_or_else(|| "List.take requires 2 arguments".to_string())?;
            let n = it
                .next()
                .ok_or_else(|| "List.take requires 2 arguments".to_string())?;
            match (list, n) {
                (VMValue::List(fl), VMValue::Int(n)) => {
                    Ok(VMValue::List(fl.take_front(n.max(0) as usize)))
                }
                _ => Err("List.take expects (List, Int)".to_string()),
            }
        }
        "List.drop" => {
            let mut it = args.into_iter();
            let list = it
                .next()
                .ok_or_else(|| "List.drop requires 2 arguments".to_string())?;
            let n = it
                .next()
                .ok_or_else(|| "List.drop requires 2 arguments".to_string())?;
            match (list, n) {
                // O(1): just advance the offset — no element copies.
                (VMValue::List(fl), VMValue::Int(n)) => {
                    Ok(VMValue::List(fl.drop_front(n.max(0) as usize)))
                }
                _ => Err("List.drop expects (List, Int)".to_string()),
            }
        }
        "List.enumerate" => match args.into_iter().next() {
            Some(VMValue::List(fl)) => {
                let pairs: Vec<VMValue> = fl
                    .into_iter()
                    .enumerate()
                    .map(|(i, v)| {
                        let mut m = HashMap::new();
                        m.insert("first".to_string(), VMValue::Int(i as i64));
                        m.insert("second".to_string(), v);
                        VMValue::Record(m)
                    })
                    .collect();
                Ok(VMValue::List(FavList::new(pairs)))
            }
            _ => Err("List.enumerate expects List".to_string()),
        },
        "List.join" => {
            let mut it = args.into_iter();
            let list = it
                .next()
                .ok_or_else(|| "List.join requires 2 arguments".to_string())?;
            let sep = it
                .next()
                .ok_or_else(|| "List.join requires 2 arguments".to_string())?;
            match (list, sep) {
                (VMValue::List(fl), VMValue::Str(sep)) => {
                    let mut parts = Vec::with_capacity(fl.len());
                    for v in fl {
                        match v {
                            VMValue::Str(s) => parts.push(s),
                            other => {
                                return Err(format!(
                                    "List.join expects List<String>, got {:?}",
                                    other
                                ));
                            }
                        }
                    }
                    Ok(VMValue::Str(parts.join(&sep)))
                }
                _ => Err("List.join expects (List<String>, String)".to_string()),
            }
        }
        "List.unique" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "List.unique requires 1 argument".to_string())?;
            match v {
                VMValue::List(fl) => {
                    let mut seen = HashSet::new();
                    let mut out = Vec::with_capacity(fl.len());
                    for item in fl {
                        let key = vmvalue_repr(&item);
                        if seen.insert(key) {
                            out.push(item);
                        }
                    }
                    Ok(VMValue::List(FavList::new(out)))
                }
                _ => Err("List.unique requires a List argument".to_string()),
            }
        }
        "List.flatten" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "List.flatten requires 1 argument".to_string())?;
            match v {
                VMValue::List(fl) => {
                    let mut out = Vec::new();
                    for inner in fl {
                        match inner {
                            VMValue::List(inner_fl) => out.extend(inner_fl),
                            _ => return Err("List.flatten requires List<List<T>>".to_string()),
                        }
                    }
                    Ok(VMValue::List(FavList::new(out)))
                }
                _ => Err("List.flatten requires a List argument".to_string()),
            }
        }
        "List.chunk" => {
            let mut it = args.into_iter();
            let list = it
                .next()
                .ok_or_else(|| "List.chunk requires 2 arguments".to_string())?;
            let n = it
                .next()
                .ok_or_else(|| "List.chunk requires 2 arguments".to_string())?;
            match (list, n) {
                (VMValue::List(fl), VMValue::Int(n)) if n > 0 => {
                    let size = n as usize;
                    let chunks = fl
                        .chunks(size)
                        .map(|chunk| VMValue::List(FavList::new(chunk.to_vec())))
                        .collect();
                    Ok(VMValue::List(FavList::new(chunks)))
                }
                (VMValue::List(_), VMValue::Int(_)) => {
                    Err("List.chunk requires a positive chunk size".to_string())
                }
                _ => Err("List.chunk expects (List, Int)".to_string()),
            }
        }
        "List.intersperse" => {
            let value_args: Vec<Value> = args.into_iter().map(Value::from).collect();
            crate::stdlib_fav_runner::call_list_stdlib("intersperse", value_args)
                .map(VMValue::from)
                .map_err(|e| e.message)
        }
        "List.sum" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "List.sum requires 1 argument".to_string())?;
            match v {
                VMValue::List(fl) => {
                    let mut sum = 0i64;
                    for item in fl {
                        match item {
                            VMValue::Int(n) => sum += n,
                            _ => return Err("List.sum requires List<Int>".to_string()),
                        }
                    }
                    Ok(VMValue::Int(sum))
                }
                _ => Err("List.sum requires a List argument".to_string()),
            }
        }
        "List.sum_float" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "List.sum_float requires 1 argument".to_string())?;
            match v {
                VMValue::List(fl) => {
                    let mut sum = 0.0f64;
                    for item in fl {
                        match item {
                            VMValue::Float(n) => sum += n,
                            _ => return Err("List.sum_float requires List<Float>".to_string()),
                        }
                    }
                    Ok(VMValue::Float(sum))
                }
                _ => Err("List.sum_float requires a List argument".to_string()),
            }
        }
        "List.min" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "List.min requires 1 argument".to_string())?;
            match v {
                VMValue::List(fl) => {
                    let mut min: Option<i64> = None;
                    for item in fl {
                        match item {
                            VMValue::Int(n) => min = Some(min.map(|m| m.min(n)).unwrap_or(n)),
                            _ => return Err("List.min requires List<Int>".to_string()),
                        }
                    }
                    Ok(match min {
                        Some(n) => {
                            VMValue::Variant("some".to_string(), Some(Box::new(VMValue::Int(n))))
                        }
                        None => VMValue::Variant("none".to_string(), None),
                    })
                }
                _ => Err("List.min requires a List argument".to_string()),
            }
        }
        "List.max" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "List.max requires 1 argument".to_string())?;
            match v {
                VMValue::List(fl) => {
                    let mut max: Option<i64> = None;
                    for item in fl {
                        match item {
                            VMValue::Int(n) => max = Some(max.map(|m| m.max(n)).unwrap_or(n)),
                            _ => return Err("List.max requires List<Int>".to_string()),
                        }
                    }
                    Ok(match max {
                        Some(n) => {
                            VMValue::Variant("some".to_string(), Some(Box::new(VMValue::Int(n))))
                        }
                        None => VMValue::Variant("none".to_string(), None),
                    })
                }
                _ => Err("List.max requires a List argument".to_string()),
            }
        }

        // ── List additions (v16.4.0, no closure needed) ───────────────────────────
        "List.distinct" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "List.distinct requires 1 argument".to_string())?;
            match v {
                VMValue::List(fl) => {
                    let mut seen: Vec<VMValue> = Vec::new();
                    let mut out: Vec<VMValue> = Vec::new();
                    for x in fl {
                        if !seen.contains(&x) {
                            seen.push(x.clone());
                            out.push(x);
                        }
                    }
                    Ok(VMValue::List(FavList::new(out)))
                }
                _ => Err("List.distinct requires a List argument".to_string()),
            }
        }
        "List.unzip" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "List.unzip requires 1 argument".to_string())?;
            match v {
                VMValue::List(fl) => {
                    let mut as_vec: Vec<VMValue> = Vec::new();
                    let mut bs_vec: Vec<VMValue> = Vec::new();
                    for item in fl {
                        match item {
                            VMValue::Variant(ref tag, ref payload) if tag == "Pair" => {
                                match payload.as_deref() {
                                    Some(VMValue::List(pair)) if pair.len() == 2 => {
                                        as_vec.push(pair[0].clone());
                                        bs_vec.push(pair[1].clone());
                                    }
                                    _ => return Err("List.unzip: each element must be a Pair".to_string()),
                                }
                            }
                            _ => return Err("List.unzip: each element must be a Pair".to_string()),
                        }
                    }
                    Ok(VMValue::Variant(
                        "Pair".to_string(),
                        Some(Box::new(VMValue::List(FavList::new(vec![
                            VMValue::List(FavList::new(as_vec)),
                            VMValue::List(FavList::new(bs_vec)),
                        ])))),
                    ))
                }
                _ => Err("List.unzip requires a List argument".to_string()),
            }
        }

        // ── String additions (v16.4.0) ────────────────────────────────────────────
        "String.split_once" => {
            let mut it = args.into_iter();
            let s = match it.next().ok_or_else(|| "String.split_once requires 2 arguments".to_string())? {
                VMValue::Str(s) => s,
                _ => return Err("String.split_once requires a String as first argument".to_string()),
            };
            let sep = match it.next().ok_or_else(|| "String.split_once requires 2 arguments".to_string())? {
                VMValue::Str(s) => s,
                _ => return Err("String.split_once requires a String separator".to_string()),
            };
            match s.split_once(sep.as_str()) {
                Some((a, b)) => Ok(VMValue::Variant(
                    "Pair".to_string(),
                    Some(Box::new(VMValue::List(FavList::new(vec![
                        VMValue::Str(a.to_string()),
                        VMValue::Str(b.to_string()),
                    ])))),
                )),
                None => Ok(VMValue::Variant(
                    "Pair".to_string(),
                    Some(Box::new(VMValue::List(FavList::new(vec![
                        VMValue::Str(s),
                        VMValue::Str(String::new()),
                    ])))),
                )),
            }
        }
        "String.replace_first" => {
            let mut it = args.into_iter();
            let s = match it.next().ok_or_else(|| "String.replace_first requires 3 arguments".to_string())? {
                VMValue::Str(s) => s,
                _ => return Err("String.replace_first: first argument must be String".to_string()),
            };
            let old = match it.next().ok_or_else(|| "String.replace_first requires 3 arguments".to_string())? {
                VMValue::Str(s) => s,
                _ => return Err("String.replace_first: second argument must be String".to_string()),
            };
            let new = match it.next().ok_or_else(|| "String.replace_first requires 3 arguments".to_string())? {
                VMValue::Str(s) => s,
                _ => return Err("String.replace_first: third argument must be String".to_string()),
            };
            Ok(VMValue::Str(s.replacen(old.as_str(), new.as_str(), 1)))
        }
        "String.format_int" => {
            // format_int(n, width, pad_char)
            let mut it = args.into_iter();
            let n = match it.next().ok_or_else(|| "String.format_int requires 3 arguments".to_string())? {
                VMValue::Int(n) => n,
                _ => return Err("String.format_int: first argument must be Int".to_string()),
            };
            let width = match it.next().ok_or_else(|| "String.format_int requires 3 arguments".to_string())? {
                VMValue::Int(w) => w as usize,
                _ => return Err("String.format_int: second argument must be Int (width)".to_string()),
            };
            let pad = match it.next().ok_or_else(|| "String.format_int requires 3 arguments".to_string())? {
                VMValue::Str(s) => s.chars().next().unwrap_or(' '),
                _ => return Err("String.format_int: third argument must be String (pad char)".to_string()),
            };
            let s = n.to_string();
            if s.len() >= width {
                Ok(VMValue::Str(s))
            } else {
                let pad_count = width - s.len();
                Ok(VMValue::Str(format!("{}{}", pad.to_string().repeat(pad_count), s)))
            }
        }
        "String.format_float" => {
            // format_float(f, digits)
            let mut it = args.into_iter();
            let f = match it.next().ok_or_else(|| "String.format_float requires 2 arguments".to_string())? {
                VMValue::Float(f) => f,
                VMValue::Int(n) => n as f64,
                _ => return Err("String.format_float: first argument must be Float or Int".to_string()),
            };
            let digits = match it.next().ok_or_else(|| "String.format_float requires 2 arguments".to_string())? {
                VMValue::Int(d) => d as usize,
                _ => return Err("String.format_float: second argument must be Int (digits)".to_string()),
            };
            Ok(VMValue::Str(format!("{:.prec$}", f, prec = digits)))
        }

        // ── Math additions (v16.4.0) ──────────────────────────────────────────────
        "Math.round_to" => {
            let mut it = args.into_iter();
            let f = match it.next().ok_or_else(|| "Math.round_to requires 2 arguments".to_string())? {
                VMValue::Float(f) => f,
                VMValue::Int(n) => n as f64,
                _ => return Err("Math.round_to: first argument must be Float or Int".to_string()),
            };
            let digits = match it.next().ok_or_else(|| "Math.round_to requires 2 arguments".to_string())? {
                VMValue::Int(d) => d,
                _ => return Err("Math.round_to: second argument must be Int (digits)".to_string()),
            };
            let factor = 10f64.powi(digits as i32);
            Ok(VMValue::Float((f * factor).round() / factor))
        }
        "Math.log" => match args.as_slice() {
            [VMValue::Float(v)] => Ok(VMValue::Float(v.ln())),
            [VMValue::Int(n)] => Ok(VMValue::Float((*n as f64).ln())),
            [_] => Err("Math.log requires a Float argument".to_string()),
            _ => Err("Math.log requires 1 argument".to_string()),
        },
        "Math.log2" => match args.as_slice() {
            [VMValue::Float(v)] => Ok(VMValue::Float(v.log2())),
            [VMValue::Int(n)] => Ok(VMValue::Float((*n as f64).log2())),
            [_] => Err("Math.log2 requires a Float argument".to_string()),
            _ => Err("Math.log2 requires 1 argument".to_string()),
        },
        "Math.log10" => match args.as_slice() {
            [VMValue::Float(v)] => Ok(VMValue::Float(v.log10())),
            [VMValue::Int(n)] => Ok(VMValue::Float((*n as f64).log10())),
            [_] => Err("Math.log10 requires a Float argument".to_string()),
            _ => Err("Math.log10 requires 1 argument".to_string()),
        },

        // ── DateTime (v16.4.0) ────────────────────────────────────────────────────
        "DateTime.now" | "DateTime.now_unix" => {
            if !args.is_empty() {
                return Err("DateTime.now requires 0 arguments".to_string());
            }
            use chrono::Utc;
            Ok(VMValue::Int(Utc::now().timestamp()))
        }
        "DateTime.parse" => {
            let s = match args.into_iter().next().ok_or_else(|| "DateTime.parse requires 1 argument".to_string())? {
                VMValue::Str(s) => s,
                _ => return Err("DateTime.parse requires a String argument".to_string()),
            };
            use chrono::DateTime;
            match DateTime::parse_from_rfc3339(&s) {
                Ok(dt) => Ok(VMValue::Variant(
                    "ok".to_string(),
                    Some(Box::new(VMValue::Int(dt.timestamp()))),
                )),
                Err(e) => Ok(VMValue::Variant(
                    "err".to_string(),
                    Some(Box::new(VMValue::Str(e.to_string()))),
                )),
            }
        }
        "DateTime.format" => {
            let mut it = args.into_iter();
            let ts = match it.next().ok_or_else(|| "DateTime.format requires 2 arguments".to_string())? {
                VMValue::Int(n) => n,
                _ => return Err("DateTime.format: first argument must be DateTime (Int)".to_string()),
            };
            let fmt = match it.next().ok_or_else(|| "DateTime.format requires 2 arguments".to_string())? {
                VMValue::Str(s) => s,
                _ => return Err("DateTime.format: second argument must be a format String".to_string()),
            };
            use chrono::{TimeZone, Utc};
            let dt = Utc.timestamp_opt(ts, 0).single()
                .ok_or_else(|| "DateTime.format: invalid timestamp".to_string())?;
            // Convert simple YYYY-MM-DD tokens to chrono format
            let chrono_fmt = fmt
                .replace("YYYY", "%Y")
                .replace("MM", "%m")
                .replace("DD", "%d")
                .replace("HH", "%H")
                .replace("mm", "%M")
                .replace("ss", "%S");
            Ok(VMValue::Str(dt.format(&chrono_fmt).to_string()))
        }
        "DateTime.format_iso" => {
            let ts = match args.into_iter().next().ok_or_else(|| "DateTime.format_iso requires 1 argument".to_string())? {
                VMValue::Int(n) => n,
                _ => return Err("DateTime.format_iso: argument must be DateTime (Int)".to_string()),
            };
            use chrono::{TimeZone, Utc};
            let dt = Utc.timestamp_opt(ts, 0).single()
                .ok_or_else(|| "DateTime.format_iso: invalid timestamp".to_string())?;
            Ok(VMValue::Str(dt.to_rfc3339()))
        }
        "DateTime.add_days" => {
            let mut it = args.into_iter();
            let ts = match it.next().ok_or_else(|| "DateTime.add_days requires 2 arguments".to_string())? {
                VMValue::Int(n) => n,
                _ => return Err("DateTime.add_days: first argument must be DateTime (Int)".to_string()),
            };
            let days = match it.next().ok_or_else(|| "DateTime.add_days requires 2 arguments".to_string())? {
                VMValue::Int(n) => n,
                _ => return Err("DateTime.add_days: second argument must be Int".to_string()),
            };
            use chrono::{Duration, TimeZone, Utc};
            let dt = Utc.timestamp_opt(ts, 0).single()
                .ok_or_else(|| "DateTime.add_days: invalid timestamp".to_string())?;
            Ok(VMValue::Int((dt + Duration::days(days)).timestamp()))
        }
        "DateTime.add_hours" => {
            let mut it = args.into_iter();
            let ts = match it.next().ok_or_else(|| "DateTime.add_hours requires 2 arguments".to_string())? {
                VMValue::Int(n) => n,
                _ => return Err("DateTime.add_hours: first argument must be DateTime (Int)".to_string()),
            };
            let hours = match it.next().ok_or_else(|| "DateTime.add_hours requires 2 arguments".to_string())? {
                VMValue::Int(n) => n,
                _ => return Err("DateTime.add_hours: second argument must be Int".to_string()),
            };
            use chrono::{Duration, TimeZone, Utc};
            let dt = Utc.timestamp_opt(ts, 0).single()
                .ok_or_else(|| "DateTime.add_hours: invalid timestamp".to_string())?;
            Ok(VMValue::Int((dt + Duration::hours(hours)).timestamp()))
        }
        "DateTime.diff_days" => {
            let mut it = args.into_iter();
            let from = match it.next().ok_or_else(|| "DateTime.diff_days requires 2 arguments".to_string())? {
                VMValue::Int(n) => n,
                _ => return Err("DateTime.diff_days: first argument must be DateTime (Int)".to_string()),
            };
            let to = match it.next().ok_or_else(|| "DateTime.diff_days requires 2 arguments".to_string())? {
                VMValue::Int(n) => n,
                _ => return Err("DateTime.diff_days: second argument must be DateTime (Int)".to_string()),
            };
            use chrono::{TimeZone, Utc};
            let dt_from = Utc.timestamp_opt(from, 0).single()
                .ok_or_else(|| "DateTime.diff_days: invalid from timestamp".to_string())?;
            let dt_to = Utc.timestamp_opt(to, 0).single()
                .ok_or_else(|| "DateTime.diff_days: invalid to timestamp".to_string())?;
            Ok(VMValue::Int((dt_to - dt_from).num_days()))
        }
        "DateTime.diff_seconds" => {
            let mut it = args.into_iter();
            let from = match it.next().ok_or_else(|| "DateTime.diff_seconds requires 2 arguments".to_string())? {
                VMValue::Int(n) => n,
                _ => return Err("DateTime.diff_seconds: first argument must be DateTime (Int)".to_string()),
            };
            let to = match it.next().ok_or_else(|| "DateTime.diff_seconds requires 2 arguments".to_string())? {
                VMValue::Int(n) => n,
                _ => return Err("DateTime.diff_seconds: second argument must be DateTime (Int)".to_string()),
            };
            Ok(VMValue::Int(to - from))
        }
        "DateTime.before" => {
            let mut it = args.into_iter();
            let a = match it.next().ok_or_else(|| "DateTime.before requires 2 arguments".to_string())? {
                VMValue::Int(n) => n,
                _ => return Err("DateTime.before: first argument must be DateTime (Int)".to_string()),
            };
            let b = match it.next().ok_or_else(|| "DateTime.before requires 2 arguments".to_string())? {
                VMValue::Int(n) => n,
                _ => return Err("DateTime.before: second argument must be DateTime (Int)".to_string()),
            };
            Ok(VMValue::Bool(a < b))
        }
        "DateTime.after" => {
            let mut it = args.into_iter();
            let a = match it.next().ok_or_else(|| "DateTime.after requires 2 arguments".to_string())? {
                VMValue::Int(n) => n,
                _ => return Err("DateTime.after: first argument must be DateTime (Int)".to_string()),
            };
            let b = match it.next().ok_or_else(|| "DateTime.after requires 2 arguments".to_string())? {
                VMValue::Int(n) => n,
                _ => return Err("DateTime.after: second argument must be DateTime (Int)".to_string()),
            };
            Ok(VMValue::Bool(a > b))
        }
        "DateTime.between" => {
            let mut it = args.into_iter();
            let dt = match it.next().ok_or_else(|| "DateTime.between requires 3 arguments".to_string())? {
                VMValue::Int(n) => n,
                _ => return Err("DateTime.between: first argument must be DateTime (Int)".to_string()),
            };
            let from = match it.next().ok_or_else(|| "DateTime.between requires 3 arguments".to_string())? {
                VMValue::Int(n) => n,
                _ => return Err("DateTime.between: second argument must be DateTime (Int)".to_string()),
            };
            let to = match it.next().ok_or_else(|| "DateTime.between requires 3 arguments".to_string())? {
                VMValue::Int(n) => n,
                _ => return Err("DateTime.between: third argument must be DateTime (Int)".to_string()),
            };
            Ok(VMValue::Bool(from <= dt && dt <= to))
        }

        "Map.empty" => {
            Ok(VMValue::Record(std::collections::HashMap::new()))
        }
        "Map.set" => {
            let mut it = args.into_iter();
            let map = it
                .next()
                .ok_or_else(|| "Map.set requires 3 arguments".to_string())?;
            let key = it
                .next()
                .ok_or_else(|| "Map.set requires 3 arguments".to_string())?;
            let val = it
                .next()
                .ok_or_else(|| "Map.set requires 3 arguments".to_string())?;
            let mut m = match map {
                VMValue::Record(m) => m,
                VMValue::Unit => HashMap::new(),
                _ => return Err("Map.set requires a Record or Unit as first argument".to_string()),
            };
            let k = match key {
                VMValue::Str(s) => s,
                _ => return Err("Map.set requires a String key".to_string()),
            };
            m.insert(k, val);
            Ok(VMValue::Record(m))
        }
        "Map.get" => {
            let mut it = args.into_iter();
            let map = it
                .next()
                .ok_or_else(|| "Map.get requires 2 arguments".to_string())?;
            let key = it
                .next()
                .ok_or_else(|| "Map.get requires 2 arguments".to_string())?;
            let m = match map {
                VMValue::Record(m) => m,
                _ => return Err("Map.get requires a Record as first argument".to_string()),
            };
            let k = match key {
                VMValue::Str(s) => s,
                _ => return Err("Map.get requires a String key".to_string()),
            };
            Ok(match m.get(&k) {
                Some(v) => VMValue::Variant("some".to_string(), Some(Box::new(v.clone()))),
                None => VMValue::Variant("none".to_string(), None),
            })
        }
        "Map.delete" => {
            let mut it = args.into_iter();
            let map = it
                .next()
                .ok_or_else(|| "Map.delete requires 2 arguments".to_string())?;
            let key = it
                .next()
                .ok_or_else(|| "Map.delete requires 2 arguments".to_string())?;
            let mut m = match map {
                VMValue::Record(m) => m,
                _ => return Err("Map.delete requires a Record as first argument".to_string()),
            };
            let k = match key {
                VMValue::Str(s) => s,
                _ => return Err("Map.delete requires a String key".to_string()),
            };
            m.remove(&k);
            Ok(VMValue::Record(m))
        }
        "Map.keys" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "Map.keys requires 1 argument".to_string())?;
            match v {
                VMValue::Record(m) => {
                    let mut keys: Vec<VMValue> =
                        m.keys().map(|k| VMValue::Str(k.clone())).collect();
                    keys.sort_by(|a, b| match (a, b) {
                        (VMValue::Str(x), VMValue::Str(y)) => x.cmp(y),
                        _ => std::cmp::Ordering::Equal,
                    });
                    Ok(VMValue::List(FavList::new(keys)))
                }
                _ => Err("Map.keys requires a Record (map) argument".to_string()),
            }
        }
        "Map.values" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "Map.values requires 1 argument".to_string())?;
            match v {
                VMValue::Record(m) => {
                    let mut pairs: Vec<_> = m.iter().collect();
                    pairs.sort_by(|a, b| a.0.cmp(b.0));
                    Ok(VMValue::List(FavList::new(
                        pairs.into_iter().map(|(_, v)| v.clone()).collect(),
                    )))
                }
                _ => Err("Map.values requires a Record (map) argument".to_string()),
            }
        }
        "Map.has_key" => {
            let mut it = args.into_iter();
            let map = it
                .next()
                .ok_or_else(|| "Map.has_key requires 2 arguments".to_string())?;
            let key = it
                .next()
                .ok_or_else(|| "Map.has_key requires 2 arguments".to_string())?;
            match (map, key) {
                (VMValue::Record(m), VMValue::Str(k)) => Ok(VMValue::Bool(m.contains_key(&k))),
                _ => Err("Map.has_key requires (Map, String)".to_string()),
            }
        }
        "Map.size" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "Map.size requires 1 argument".to_string())?;
            match v {
                VMValue::Record(m) => Ok(VMValue::Int(m.len() as i64)),
                _ => Err("Map.size requires a Map argument".to_string()),
            }
        }
        "Map.is_empty" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "Map.is_empty requires 1 argument".to_string())?;
            match v {
                VMValue::Record(m) => Ok(VMValue::Bool(m.is_empty())),
                _ => Err("Map.is_empty requires a Map argument".to_string()),
            }
        }
        "Map.remove" => {
            let mut it = args.into_iter();
            let map = it
                .next()
                .ok_or_else(|| "Map.remove requires 2 arguments".to_string())?;
            let key = it
                .next()
                .ok_or_else(|| "Map.remove requires 2 arguments".to_string())?;
            match (map, key) {
                (VMValue::Record(mut m), VMValue::Str(k)) => {
                    m.remove(&k);
                    Ok(VMValue::Record(m))
                }
                _ => Err("Map.remove requires (Map, String)".to_string()),
            }
        }
        "Map.contains_key" => {
            let mut it = args.into_iter();
            let map = it
                .next()
                .ok_or_else(|| "Map.contains_key requires 2 arguments".to_string())?;
            let key = it
                .next()
                .ok_or_else(|| "Map.contains_key requires 2 arguments".to_string())?;
            match (map, key) {
                (VMValue::Record(m), VMValue::Str(k)) => Ok(VMValue::Bool(m.contains_key(&k))),
                _ => Err("Map.contains_key requires (Map, String)".to_string()),
            }
        }
        "Map.merge" => {
            let mut it = args.into_iter();
            let base = it
                .next()
                .ok_or_else(|| "Map.merge requires 2 arguments".to_string())?;
            let overrides = it
                .next()
                .ok_or_else(|| "Map.merge requires 2 arguments".to_string())?;
            match (base, overrides) {
                (VMValue::Record(mut base), VMValue::Record(overrides)) => {
                    for (k, v) in overrides {
                        base.insert(k, v);
                    }
                    Ok(VMValue::Record(base))
                }
                _ => Err("Map.merge requires (Map, Map)".to_string()),
            }
        }
        "Map.from_list" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "Map.from_list requires 1 argument".to_string())?;
            match v {
                VMValue::List(fl) => {
                    let mut out = HashMap::with_capacity(fl.len());
                    for pair in fl {
                        match pair {
                            VMValue::Record(mut fields) => {
                                let first = fields.remove("first");
                                let second = fields.remove("second");
                                match (first, second) {
                                    (Some(VMValue::Str(k)), Some(v)) => {
                                        out.insert(k, v);
                                    }
                                    _ => {
                                        return Err(
                                            "Map.from_list requires Pair-like records with { first: String second: V }"
                                                .to_string(),
                                        )
                                    }
                                }
                            }
                            _ => {
                                return Err(
                                    "Map.from_list requires List<Pair<String, V>>".to_string()
                                );
                            }
                        }
                    }
                    Ok(VMValue::Record(out))
                }
                _ => Err("Map.from_list requires a List argument".to_string()),
            }
        }
        "Map.to_list" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "Map.to_list requires 1 argument".to_string())?;
            match v {
                VMValue::Record(m) => {
                    let mut pairs: Vec<_> = m.into_iter().collect();
                    pairs.sort_by(|a, b| a.0.cmp(&b.0));
                    Ok(VMValue::List(FavList::new(
                        pairs
                            .into_iter()
                            .map(|(k, v)| {
                                let mut fields = HashMap::new();
                                fields.insert("first".to_string(), VMValue::Str(k));
                                fields.insert("second".to_string(), v);
                                VMValue::Record(fields)
                            })
                            .collect(),
                    )))
                }
                _ => Err("Map.to_list requires a Map argument".to_string()),
            }
        }
        "Json.null" => Ok(json_variant_vm("json_null", None)),
        "Json.bool" => match args.into_iter().next() {
            Some(VMValue::Bool(b)) => Ok(json_variant_vm("json_bool", Some(VMValue::Bool(b)))),
            Some(other) => Err(format!(
                "Json.bool expects Bool, got {}",
                vmvalue_type_name(&other)
            )),
            None => Err("Json.bool requires 1 argument".to_string()),
        },
        "Json.int" => match args.into_iter().next() {
            Some(VMValue::Int(i)) => Ok(json_variant_vm("json_int", Some(VMValue::Int(i)))),
            Some(other) => Err(format!(
                "Json.int expects Int, got {}",
                vmvalue_type_name(&other)
            )),
            None => Err("Json.int requires 1 argument".to_string()),
        },
        "Json.float" => match args.into_iter().next() {
            Some(VMValue::Float(f)) => Ok(json_variant_vm("json_float", Some(VMValue::Float(f)))),
            Some(other) => Err(format!(
                "Json.float expects Float, got {}",
                vmvalue_type_name(&other)
            )),
            None => Err("Json.float requires 1 argument".to_string()),
        },
        "Json.str" => match args.into_iter().next() {
            Some(VMValue::Str(s)) => Ok(json_variant_vm("json_str", Some(VMValue::Str(s)))),
            Some(other) => Err(format!(
                "Json.str expects String, got {}",
                vmvalue_type_name(&other)
            )),
            None => Err("Json.str requires 1 argument".to_string()),
        },
        "Json.array" => match args.into_iter().next() {
            Some(VMValue::List(fl)) => Ok(json_variant_vm("json_array", Some(VMValue::List(fl)))),
            Some(other) => Err(format!(
                "Json.array expects List<Json>, got {}",
                vmvalue_type_name(&other)
            )),
            None => Err("Json.array requires 1 argument".to_string()),
        },
        "Json.object" => match args.into_iter().next() {
            Some(VMValue::List(fields)) => {
                let mut obj = HashMap::new();
                for field in fields {
                    let rec = match field {
                        VMValue::Record(rec) => rec,
                        other => {
                            return Err(format!(
                                "Json.object expects List<JsonField>, got {}",
                                vmvalue_type_name(&other)
                            ));
                        }
                    };
                    let key = match rec.get("key") {
                        Some(VMValue::Str(s)) => s.clone(),
                        Some(other) => {
                            return Err(format!(
                                "JsonField.key must be String, got {}",
                                vmvalue_type_name(other)
                            ));
                        }
                        None => return Err("JsonField missing `key`".to_string()),
                    };
                    let value = rec
                        .get("value")
                        .cloned()
                        .ok_or_else(|| "JsonField missing `value`".to_string())?;
                    obj.insert(key, value);
                }
                Ok(json_variant_vm("json_object", Some(VMValue::Record(obj))))
            }
            Some(other) => Err(format!(
                "Json.object expects List<JsonField>, got {}",
                vmvalue_type_name(&other)
            )),
            None => Err("Json.object requires 1 argument".to_string()),
        },
        "Json.parse_raw" => match args.into_iter().next() {
            Some(VMValue::Str(text)) => match parse_json_object_raw(&text) {
                Ok(map) => Ok(ok_vm(VMValue::Record(map))),
                Err(message) => Ok(err_vm(schema_error_vm("", "valid json object", message))),
            },
            Some(other) => Err(format!(
                "Json.parse_raw expects String, got {}",
                vmvalue_type_name(&other)
            )),
            None => Err("Json.parse_raw requires 1 argument".to_string()),
        },
        "Json.parse_array_raw" => match args.into_iter().next() {
            Some(VMValue::Str(text)) => match parse_json_array_raw(&text) {
                Ok(rows) => Ok(ok_vm(VMValue::List(FavList::new(rows)))),
                Err(message) => Ok(err_vm(schema_error_vm("", "valid json array", message))),
            },
            Some(other) => Err(format!(
                "Json.parse_array_raw expects String, got {}",
                vmvalue_type_name(&other)
            )),
            None => Err("Json.parse_array_raw requires 1 argument".to_string()),
        },
        "Json.write_raw" => match args.into_iter().next() {
            Some(VMValue::Record(map)) => serde_json::to_string(&schema_record_to_string_map(&map))
                .map(VMValue::Str)
                .map_err(|e| format!("Json.write_raw failed: {}", e)),
            Some(other) => Err(format!(
                "Json.write_raw expects Map<String,String>, got {}",
                vmvalue_type_name(&other)
            )),
            None => Err("Json.write_raw requires 1 argument".to_string()),
        },
        "Json.write_array_raw" => match args.into_iter().next() {
            Some(VMValue::List(rows)) => {
                let objects: Result<Vec<_>, _> = rows
                    .into_iter()
                    .map(|row| match row {
                        VMValue::Record(map) => Ok(schema_record_to_string_map(&map)),
                        other => Err(format!(
                            "Json.write_array_raw expects List<Map<String,String>>, got {}",
                            vmvalue_type_name(&other)
                        )),
                    })
                    .collect();
                serde_json::to_string(&objects?)
                    .map(VMValue::Str)
                    .map_err(|e| format!("Json.write_array_raw failed: {}", e))
            }
            Some(other) => Err(format!(
                "Json.write_array_raw expects List<Map<String,String>>, got {}",
                vmvalue_type_name(&other)
            )),
            None => Err("Json.write_array_raw requires 1 argument".to_string()),
        },
        "Json.pretty_raw" => match args.into_iter().next() {
            Some(VMValue::Str(s)) => match serde_json::from_str::<SerdeJsonValue>(&s) {
                Ok(v) => serde_json::to_string_pretty(&v)
                    .map(VMValue::Str)
                    .map_err(|e| format!("Json.pretty_raw failed: {}", e)),
                Err(e) => Err(format!("Json.pretty_raw: invalid JSON: {}", e)),
            },
            Some(other) => Err(format!(
                "Json.pretty_raw expects String, got {}",
                vmvalue_type_name(&other)
            )),
            None => Err("Json.pretty_raw requires 1 argument".to_string()),
        },
        "Json.parse" => match args.into_iter().next() {
            Some(VMValue::Str(s)) => match serde_json::from_str::<SerdeJsonValue>(&s) {
                Ok(v) => Ok(VMValue::Variant(
                    "some".to_string(),
                    Some(Box::new(serde_to_vm_json(v))),
                )),
                Err(_) => Ok(VMValue::Variant("none".to_string(), None)),
            },
            Some(other) => Err(format!(
                "Json.parse expects String, got {}",
                vmvalue_type_name(&other)
            )),
            None => Err("Json.parse requires 1 argument".to_string()),
        },
        "Json.encode" | "Json.encode_pretty" => {
            let json = args
                .into_iter()
                .next()
                .ok_or_else(|| format!("{} requires 1 argument", name))?;
            let serde = vm_json_to_serde(&json).ok_or_else(|| format!("{} expects Json", name))?;
            let out = if name == "Json.encode_pretty" {
                serde_json::to_string_pretty(&serde)
            } else {
                serde_json::to_string(&serde)
            }
            .map_err(|e| format!("{} failed: {}", name, e))?;
            Ok(VMValue::Str(out))
        }
        "Json.get" => {
            if args.len() != 2 {
                return Err("Json.get requires 2 arguments".to_string());
            }
            let mut it = args.into_iter();
            let json = it.next().unwrap();
            let key = match it.next().unwrap() {
                VMValue::Str(s) => s,
                other => {
                    return Err(format!(
                        "Json.get expects String key, got {}",
                        vmvalue_type_name(&other)
                    ));
                }
            };
            match json {
                VMValue::Variant(tag, Some(payload)) if tag == "json_object" => match *payload {
                    VMValue::Record(map) => Ok(map
                        .get(&key)
                        .cloned()
                        .map(|v| VMValue::Variant("some".to_string(), Some(Box::new(v))))
                        .unwrap_or(VMValue::Variant("none".to_string(), None))),
                    _ => Err("Json.get received malformed json_object payload".to_string()),
                },
                _ => Ok(VMValue::Variant("none".to_string(), None)),
            }
        }
        "Json.at" => {
            if args.len() != 2 {
                return Err("Json.at requires 2 arguments".to_string());
            }
            let mut it = args.into_iter();
            let json = it.next().unwrap();
            let idx = match it.next().unwrap() {
                VMValue::Int(i) => i,
                other => {
                    return Err(format!(
                        "Json.at expects Int index, got {}",
                        vmvalue_type_name(&other)
                    ));
                }
            };
            match json {
                VMValue::Variant(tag, Some(payload)) if tag == "json_array" => match *payload {
                    VMValue::List(items) if idx >= 0 => Ok(items
                        .get(idx as usize)
                        .cloned()
                        .map(|v| VMValue::Variant("some".to_string(), Some(Box::new(v))))
                        .unwrap_or(VMValue::Variant("none".to_string(), None))),
                    VMValue::List(_) => Ok(VMValue::Variant("none".to_string(), None)),
                    _ => Err("Json.at received malformed json_array payload".to_string()),
                },
                _ => Ok(VMValue::Variant("none".to_string(), None)),
            }
        }
        "Json.as_str" => match args.into_iter().next() {
            Some(VMValue::Variant(tag, Some(payload))) if tag == "json_str" => {
                Ok(VMValue::Variant("some".to_string(), Some(payload)))
            }
            Some(_) => Ok(VMValue::Variant("none".to_string(), None)),
            None => Err("Json.as_str requires 1 argument".to_string()),
        },
        "Json.as_int" => match args.into_iter().next() {
            Some(VMValue::Variant(tag, Some(payload))) if tag == "json_int" => {
                Ok(VMValue::Variant("some".to_string(), Some(payload)))
            }
            Some(_) => Ok(VMValue::Variant("none".to_string(), None)),
            None => Err("Json.as_int requires 1 argument".to_string()),
        },
        "Json.as_float" => match args.into_iter().next() {
            Some(VMValue::Variant(tag, Some(payload))) if tag == "json_float" => {
                Ok(VMValue::Variant("some".to_string(), Some(payload)))
            }
            Some(_) => Ok(VMValue::Variant("none".to_string(), None)),
            None => Err("Json.as_float requires 1 argument".to_string()),
        },
        "Json.as_bool" => match args.into_iter().next() {
            Some(VMValue::Variant(tag, Some(payload))) if tag == "json_bool" => {
                Ok(VMValue::Variant("some".to_string(), Some(payload)))
            }
            Some(_) => Ok(VMValue::Variant("none".to_string(), None)),
            None => Err("Json.as_bool requires 1 argument".to_string()),
        },
        "Json.as_array" => match args.into_iter().next() {
            Some(VMValue::Variant(tag, Some(payload))) if tag == "json_array" => {
                Ok(VMValue::Variant("some".to_string(), Some(payload)))
            }
            Some(_) => Ok(VMValue::Variant("none".to_string(), None)),
            None => Err("Json.as_array requires 1 argument".to_string()),
        },
        "Json.is_null" => match args.into_iter().next() {
            Some(VMValue::Variant(tag, None)) if tag == "json_null" => Ok(VMValue::Bool(true)),
            Some(_) => Ok(VMValue::Bool(false)),
            None => Err("Json.is_null requires 1 argument".to_string()),
        },
        "Json.keys" => match args.into_iter().next() {
            Some(VMValue::Variant(tag, Some(payload))) if tag == "json_object" => match *payload {
                VMValue::Record(map) => {
                    let mut keys: Vec<VMValue> = map.into_keys().map(VMValue::Str).collect();
                    keys.sort_by(|a, b| vmvalue_repr(a).cmp(&vmvalue_repr(b)));
                    Ok(VMValue::Variant(
                        "some".to_string(),
                        Some(Box::new(VMValue::List(FavList::new(keys)))),
                    ))
                }
                _ => Err("Json.keys received malformed json_object payload".to_string()),
            },
            Some(_) => Ok(VMValue::Variant("none".to_string(), None)),
            None => Err("Json.keys requires 1 argument".to_string()),
        },
        "Json.length" => match args.into_iter().next() {
            Some(VMValue::Variant(tag, Some(payload))) if tag == "json_array" => match *payload {
                VMValue::List(items) => Ok(VMValue::Variant(
                    "some".to_string(),
                    Some(Box::new(VMValue::Int(items.len() as i64))),
                )),
                _ => Err("Json.length received malformed json_array payload".to_string()),
            },
            Some(VMValue::Variant(tag, Some(payload))) if tag == "json_object" => match *payload {
                VMValue::Record(map) => Ok(VMValue::Variant(
                    "some".to_string(),
                    Some(Box::new(VMValue::Int(map.len() as i64))),
                )),
                _ => Err("Json.length received malformed json_object payload".to_string()),
            },
            Some(_) => Ok(VMValue::Variant("none".to_string(), None)),
            None => Err("Json.length requires 1 argument".to_string()),
        },
        "Csv.parse" => {
            let input = vm_string(
                args.into_iter()
                    .next()
                    .ok_or_else(|| "Csv.parse requires 1 argument".to_string())?,
                "Csv.parse",
            )?;
            let mut rdr = csv::ReaderBuilder::new()
                .has_headers(false)
                .from_reader(input.as_bytes());
            let mut rows = Vec::new();
            for record in rdr.records() {
                let record = record.map_err(|e| format!("Csv.parse failed: {}", e))?;
                rows.push(VMValue::List(FavList::new(
                    record
                        .iter()
                        .map(|cell| VMValue::Str(cell.to_string()))
                        .collect(),
                )));
            }
            Ok(VMValue::List(FavList::new(rows)))
        }
        "Csv.parse_raw" => {
            if args.len() != 3 {
                return Err("Csv.parse_raw requires 3 arguments".to_string());
            }
            let mut it = args.into_iter();
            let text = vm_string(it.next().unwrap(), "Csv.parse_raw")?;
            let delimiter = vm_string(it.next().unwrap(), "Csv.parse_raw")?;
            let has_header = match it.next().unwrap() {
                VMValue::Bool(v) => v,
                other => {
                    return Err(format!(
                        "Csv.parse_raw expects Bool has_header, got {}",
                        vmvalue_type_name(&other)
                    ));
                }
            };
            let delimiter_char = delimiter
                .chars()
                .next()
                .ok_or_else(|| "Csv.parse_raw delimiter must not be empty".to_string())?;
            let mut rdr = csv::ReaderBuilder::new()
                .has_headers(has_header)
                .delimiter(delimiter_char as u8)
                .from_reader(text.as_bytes());
            let headers = if has_header {
                Some(
                    rdr.headers()
                        .map_err(|e| format!("csv parse error: {}", e))?
                        .iter()
                        .map(|s| s.to_string())
                        .collect::<Vec<_>>(),
                )
            } else {
                None
            };
            let mut rows = Vec::new();
            for record in rdr.records() {
                let record = match record {
                    Ok(record) => record,
                    Err(e) => return Ok(err_vm(schema_error_vm("", "valid csv", e.to_string()))),
                };
                let mut row = HashMap::new();
                for (idx, value) in record.iter().enumerate() {
                    let key = headers
                        .as_ref()
                        .and_then(|h| h.get(idx).cloned())
                        .unwrap_or_else(|| idx.to_string());
                    row.insert(key, VMValue::Str(value.to_string()));
                }
                rows.push(VMValue::Record(row));
            }
            Ok(ok_vm(VMValue::List(FavList::new(rows))))
        }
        "Csv.parse_with_header" => {
            let input = vm_string(
                args.into_iter()
                    .next()
                    .ok_or_else(|| "Csv.parse_with_header requires 1 argument".to_string())?,
                "Csv.parse_with_header",
            )?;
            let mut rdr = csv::ReaderBuilder::new()
                .has_headers(true)
                .from_reader(input.as_bytes());
            let headers = rdr
                .headers()
                .map_err(|e| format!("Csv.parse_with_header failed: {}", e))?
                .clone();
            let mut rows = Vec::new();
            for record in rdr.records() {
                let record = record.map_err(|e| format!("Csv.parse_with_header failed: {}", e))?;
                let mut row = HashMap::new();
                for (key, value) in headers.iter().zip(record.iter()) {
                    row.insert(key.to_string(), VMValue::Str(value.to_string()));
                }
                rows.push(VMValue::Record(row));
            }
            Ok(VMValue::List(FavList::new(rows)))
        }
        "Csv.encode" => {
            let rows = match args.into_iter().next() {
                Some(VMValue::List(rows)) => rows,
                Some(other) => {
                    return Err(format!(
                        "Csv.encode expects List<List<String>>, got {}",
                        vmvalue_type_name(&other)
                    ));
                }
                None => return Err("Csv.encode requires 1 argument".to_string()),
            };
            let mut writer = csv::WriterBuilder::new().from_writer(vec![]);
            for row in rows {
                let fields = vm_string_list(row, "Csv.encode")?;
                writer
                    .write_record(fields)
                    .map_err(|e| format!("Csv.encode failed: {}", e))?;
            }
            let bytes = writer
                .into_inner()
                .map_err(|e| format!("Csv.encode failed: {}", e.into_error()))?;
            let out = String::from_utf8(bytes)
                .map_err(|e| format!("Csv.encode produced invalid UTF-8: {}", e))?;
            Ok(VMValue::Str(out))
        }
        "Csv.encode_with_header" => {
            if args.len() != 2 {
                return Err("Csv.encode_with_header requires 2 arguments".to_string());
            }
            let mut it = args.into_iter();
            let header = vm_string_list(it.next().unwrap(), "Csv.encode_with_header")?;
            let rows = match it.next().unwrap() {
                VMValue::List(rows) => rows,
                other => {
                    return Err(format!(
                        "Csv.encode_with_header expects List<List<String>>, got {}",
                        vmvalue_type_name(&other)
                    ));
                }
            };
            let mut writer = csv::WriterBuilder::new().from_writer(vec![]);
            writer
                .write_record(&header)
                .map_err(|e| format!("Csv.encode_with_header failed: {}", e))?;
            for row in rows {
                let fields = vm_string_list(row, "Csv.encode_with_header")?;
                writer
                    .write_record(fields)
                    .map_err(|e| format!("Csv.encode_with_header failed: {}", e))?;
            }
            let bytes = writer
                .into_inner()
                .map_err(|e| format!("Csv.encode_with_header failed: {}", e.into_error()))?;
            let out = String::from_utf8(bytes)
                .map_err(|e| format!("Csv.encode_with_header produced invalid UTF-8: {}", e))?;
            Ok(VMValue::Str(out))
        }
        "Csv.from_records" => {
            let records = match args.into_iter().next() {
                Some(VMValue::List(records)) => records,
                Some(other) => {
                    return Err(format!(
                        "Csv.from_records expects List<Map<String>>, got {}",
                        vmvalue_type_name(&other)
                    ));
                }
                None => return Err("Csv.from_records requires 1 argument".to_string()),
            };
            let mut headers = std::collections::BTreeSet::new();
            let mut rows = Vec::new();
            for record in records {
                match record {
                    VMValue::Record(map) => {
                        for key in map.keys() {
                            headers.insert(key.clone());
                        }
                        rows.push(map);
                    }
                    other => {
                        return Err(format!(
                            "Csv.from_records expects record rows, got {}",
                            vmvalue_type_name(&other)
                        ));
                    }
                }
            }
            let header: Vec<String> = headers.into_iter().collect();
            let mut writer = csv::WriterBuilder::new().from_writer(vec![]);
            writer
                .write_record(&header)
                .map_err(|e| format!("Csv.from_records failed: {}", e))?;
            for row in rows {
                let mut values = Vec::with_capacity(header.len());
                for key in &header {
                    let value = row.get(key).cloned().unwrap_or(VMValue::Str(String::new()));
                    values.push(vm_string(value, "Csv.from_records")?);
                }
                writer
                    .write_record(values)
                    .map_err(|e| format!("Csv.from_records failed: {}", e))?;
            }
            let bytes = writer
                .into_inner()
                .map_err(|e| format!("Csv.from_records failed: {}", e.into_error()))?;
            let out = String::from_utf8(bytes)
                .map_err(|e| format!("Csv.from_records produced invalid UTF-8: {}", e))?;
            Ok(VMValue::Str(out))
        }
        "Csv.write_raw" => {
            if args.len() != 2 {
                return Err("Csv.write_raw requires 2 arguments".to_string());
            }
            let mut it = args.into_iter();
            let rows = schema_rows_from_vm(it.next().unwrap(), "Csv.write_raw")?;
            let delimiter = vm_string(it.next().unwrap(), "Csv.write_raw")?;
            let delimiter_char = delimiter
                .chars()
                .next()
                .ok_or_else(|| "Csv.write_raw delimiter must not be empty".to_string())?;
            let mut writer = csv::WriterBuilder::new()
                .delimiter(delimiter_char as u8)
                .from_writer(vec![]);
            if let Some(first) = rows.first() {
                let mut header: Vec<String> = first.keys().cloned().collect();
                header.sort();
                writer
                    .write_record(&header)
                    .map_err(|e| format!("Csv.write_raw failed: {}", e))?;
                for row in rows {
                    let values: Vec<String> = header
                        .iter()
                        .map(|key| {
                            row.get(key)
                                .map(vm_scalar_to_plain_string)
                                .unwrap_or_default()
                        })
                        .collect();
                    writer
                        .write_record(values)
                        .map_err(|e| format!("Csv.write_raw failed: {}", e))?;
                }
            }
            let bytes = writer
                .into_inner()
                .map_err(|e| format!("Csv.write_raw failed: {}", e.into_error()))?;
            String::from_utf8(bytes)
                .map(VMValue::Str)
                .map_err(|e| format!("Csv.write_raw produced invalid UTF-8: {}", e))
        }
        "Schema.adapt" => {
            if args.len() != 2 {
                return Err("Schema.adapt requires 2 arguments".to_string());
            }
            let mut it = args.into_iter();
            let rows = schema_rows_from_vm(it.next().unwrap(), "Schema.adapt")?;
            let type_name = vm_string(it.next().unwrap(), "Schema.adapt")?;
            Ok(schema_adapt_rows(rows, &type_name, type_metas))
        }
        "Schema.adapt_one" => {
            if args.len() != 2 {
                return Err("Schema.adapt_one requires 2 arguments".to_string());
            }
            let mut it = args.into_iter();
            let row = match it.next().unwrap() {
                VMValue::Record(map) => map,
                other => {
                    return Err(format!(
                        "Schema.adapt_one expects Map<String,String>, got {}",
                        vmvalue_type_name(&other)
                    ));
                }
            };
            let type_name = vm_string(it.next().unwrap(), "Schema.adapt_one")?;
            let adapted = schema_adapt_rows(vec![row], &type_name, type_metas);
            match &adapted {
                VMValue::Variant(tag, Some(payload)) if tag == "ok" => match payload.as_ref() {
                    VMValue::List(rows) => {
                        Ok(ok_vm(rows.first().cloned().unwrap_or(VMValue::Unit)))
                    }
                    _ => Ok(adapted),
                },
                _ => Ok(adapted),
            }
        }
        "Schema.to_csv" => {
            if args.len() != 2 {
                return Err("Schema.to_csv requires 2 arguments".to_string());
            }
            let mut it = args.into_iter();
            let rows = match it.next().unwrap() {
                VMValue::List(rows) => rows,
                other => {
                    return Err(format!(
                        "Schema.to_csv expects List<Record>, got {}",
                        vmvalue_type_name(&other)
                    ));
                }
            };
            let type_name = vm_string(it.next().unwrap(), "Schema.to_csv")?;
            let mut writer = csv::WriterBuilder::new().from_writer(vec![]);
            let header: Vec<String> = if let Some(meta) = type_metas.get(&type_name) {
                meta.fields.iter().map(|field| field.name.clone()).collect()
            } else if let Some(VMValue::Record(first)) = rows.first() {
                let mut keys: Vec<String> = first.keys().cloned().collect();
                keys.sort();
                keys
            } else {
                Vec::new()
            };
            writer
                .write_record(&header)
                .map_err(|e| format!("Schema.to_csv failed: {}", e))?;
            for row in rows {
                let VMValue::Record(record) = row else {
                    return Err("Schema.to_csv expects record rows".to_string());
                };
                let values: Vec<String> = header
                    .iter()
                    .map(|field_name| {
                        record
                            .get(field_name)
                            .map(vm_scalar_to_plain_string)
                            .unwrap_or_default()
                    })
                    .collect();
                writer
                    .write_record(values)
                    .map_err(|e| format!("Schema.to_csv failed: {}", e))?;
            }
            let bytes = writer
                .into_inner()
                .map_err(|e| format!("Schema.to_csv failed: {}", e.into_error()))?;
            String::from_utf8(bytes)
                .map(VMValue::Str)
                .map_err(|e| format!("Schema.to_csv produced invalid UTF-8: {}", e))
        }
        "Schema.to_json" => {
            if args.len() != 2 {
                return Err("Schema.to_json requires 2 arguments".to_string());
            }
            let mut it = args.into_iter();
            let value = it.next().unwrap();
            let type_name = vm_string(it.next().unwrap(), "Schema.to_json")?;
            let json = schema_to_json_value(&value, &type_name, type_metas)?;
            serde_json::to_string(&json)
                .map(VMValue::Str)
                .map_err(|e| format!("Schema.to_json failed: {}", e))
        }
        "Schema.to_json_array" => {
            if args.len() != 2 {
                return Err("Schema.to_json_array requires 2 arguments".to_string());
            }
            let mut it = args.into_iter();
            let rows = match it.next().unwrap() {
                VMValue::List(rows) => rows,
                other => {
                    return Err(format!(
                        "Schema.to_json_array expects List<Record>, got {}",
                        vmvalue_type_name(&other)
                    ));
                }
            };
            let type_name = vm_string(it.next().unwrap(), "Schema.to_json_array")?;
            let mut json_rows = Vec::with_capacity(rows.len());
            for row in rows {
                json_rows.push(schema_to_json_value(&row, &type_name, type_metas)?);
            }
            serde_json::to_string(&json_rows)
                .map(VMValue::Str)
                .map_err(|e| format!("Schema.to_json_array failed: {}", e))
        }
        "Trace.print" => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| "Trace.print requires 1 argument".to_string())?;
            let s = match v {
                VMValue::Str(s) => s,
                other => vmvalue_repr(&other),
            };
            eprintln!("[trace] {}", s);
            Ok(VMValue::Unit)
        }
        "Trace.log" => {
            let mut it = args.into_iter();
            let label = it
                .next()
                .ok_or_else(|| "Trace.log requires 2 arguments".to_string())?;
            let val = it
                .next()
                .ok_or_else(|| "Trace.log requires 2 arguments".to_string())?;
            let label_s = match label {
                VMValue::Str(s) => s,
                other => vmvalue_repr(&other),
            };
            eprintln!("[trace] {}: {}", label_s, vmvalue_repr(&val));
            Ok(VMValue::Unit)
        }
        "Emit.log" => {
            let log: Vec<VMValue> = emit_log
                .iter()
                .map(|v| VMValue::Str(vmvalue_repr(v)))
                .collect();
            Ok(VMValue::List(FavList::new(log)))
        }
        "Db.execute" => {
            if args.is_empty() {
                return Err("Db.execute requires a SQL string".to_string());
            }
            let mut it = args.into_iter();
            let sql = vm_string(it.next().expect("sql"), "Db.execute")?;
            let params: Vec<VMValue> = it.collect();
            with_db_path(db_path, |conn| {
                let mut stmt = conn.prepare(&sql).map_err(|e| format!("Db error: {}", e))?;
                let bound: Vec<rusqlite::types::Value> =
                    params.iter().map(vmvalue_to_sql).collect();
                let refs: Vec<&dyn rusqlite::ToSql> =
                    bound.iter().map(|b| b as &dyn rusqlite::ToSql).collect();
                let rows = stmt
                    .execute(refs.as_slice())
                    .map_err(|e| format!("Db error: {}", e))?;
                Ok(VMValue::Int(rows as i64))
            })
        }
        "Db.query" => {
            if args.is_empty() {
                return Err("Db.query requires a SQL string".to_string());
            }
            let mut it = args.into_iter();
            let sql = vm_string(it.next().expect("sql"), "Db.query")?;
            let params: Vec<VMValue> = it.collect();
            with_db_path(db_path, |conn| {
                let mut stmt = conn.prepare(&sql).map_err(|e| format!("Db error: {}", e))?;
                let bound: Vec<rusqlite::types::Value> =
                    params.iter().map(vmvalue_to_sql).collect();
                let refs: Vec<&dyn rusqlite::ToSql> =
                    bound.iter().map(|b| b as &dyn rusqlite::ToSql).collect();
                let col_names: Vec<String> =
                    stmt.column_names().iter().map(|s| s.to_string()).collect();
                let rows = stmt
                    .query_map(refs.as_slice(), |row| {
                        let mut map = HashMap::new();
                        for (i, name) in col_names.iter().enumerate() {
                            let value: rusqlite::types::Value = row.get(i)?;
                            map.insert(name.clone(), VMValue::Str(sqlite_value_to_string(value)));
                        }
                        Ok(VMValue::Record(map))
                    })
                    .map_err(|e| format!("Db error: {}", e))?
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|e| format!("Db error: {}", e))?;
                Ok(VMValue::List(FavList::new(rows)))
            })
        }
        "Db.query_one" => {
            if args.is_empty() {
                return Err("Db.query_one requires a SQL string".to_string());
            }
            let mut it = args.into_iter();
            let sql = vm_string(it.next().expect("sql"), "Db.query_one")?;
            let params: Vec<VMValue> = it.collect();
            with_db_path(db_path, |conn| {
                let mut stmt = conn.prepare(&sql).map_err(|e| format!("Db error: {}", e))?;
                let bound: Vec<rusqlite::types::Value> =
                    params.iter().map(vmvalue_to_sql).collect();
                let refs: Vec<&dyn rusqlite::ToSql> =
                    bound.iter().map(|b| b as &dyn rusqlite::ToSql).collect();
                let col_names: Vec<String> =
                    stmt.column_names().iter().map(|s| s.to_string()).collect();
                let mut rows = stmt
                    .query(refs.as_slice())
                    .map_err(|e| format!("Db error: {}", e))?;
                match rows.next().map_err(|e| format!("Db error: {}", e))? {
                    None => Ok(VMValue::Variant("none".to_string(), None)),
                    Some(row) => {
                        let mut map = HashMap::new();
                        for (i, name) in col_names.iter().enumerate() {
                            let value: rusqlite::types::Value =
                                row.get(i).map_err(|e| format!("Db error: {}", e))?;
                            map.insert(name.clone(), VMValue::Str(sqlite_value_to_string(value)));
                        }
                        Ok(VMValue::Variant(
                            "some".to_string(),
                            Some(Box::new(VMValue::Record(map))),
                        ))
                    }
                }
            })
        }
        "Http.get" => {
            let url = vm_string(
                args.into_iter()
                    .next()
                    .ok_or_else(|| "Http.get requires a URL argument".to_string())?,
                "Http.get",
            )?;
            match ureq::get(&url).call() {
                Ok(resp) => {
                    let body = resp
                        .into_string()
                        .map_err(|e| format!("Http.get read error: {}", e))?;
                    Ok(VMValue::Variant(
                        "ok".to_string(),
                        Some(Box::new(VMValue::Str(body))),
                    ))
                }
                Err(e) => Ok(VMValue::Variant(
                    "err".to_string(),
                    Some(Box::new(VMValue::Str(e.to_string()))),
                )),
            }
        }
        "Http.get_raw" => {
            let url = vm_string(
                args.into_iter()
                    .next()
                    .ok_or_else(|| "Http.get_raw requires a URL argument".to_string())?,
                "Http.get_raw",
            )?;
            match ureq::get(&url).call() {
                Ok(resp) => {
                    let status = resp.status() as i64;
                    let content_type = resp
                        .header("Content-Type")
                        .unwrap_or("application/octet-stream")
                        .to_string();
                    let body = resp
                        .into_string()
                        .map_err(|e| format!("Http.get_raw read error: {}", e))?;
                    Ok(ok_vm(http_response_vm(status, body, content_type)))
                }
                Err(ureq::Error::Status(status, resp)) => {
                    let body = resp.into_string().unwrap_or_default();
                    Ok(err_vm(http_error_vm(2, body, status as i64)))
                }
                Err(ureq::Error::Transport(err)) => {
                    let msg = err.to_string();
                    let code = if msg.to_ascii_lowercase().contains("timed out") {
                        1
                    } else {
                        0
                    };
                    Ok(err_vm(http_error_vm(code, msg, 0)))
                }
            }
        }
        "Http.post" => {
            if args.len() < 2 {
                return Err("Http.post requires 2 arguments (url, body)".to_string());
            }
            let mut it = args.into_iter();
            let url = vm_string(it.next().expect("url"), "Http.post")?;
            let body = match it.next().expect("body") {
                VMValue::Str(s) => s,
                other => vmvalue_repr(&other),
            };
            match ureq::post(&url).send_string(&body) {
                Ok(resp) => {
                    let body = resp
                        .into_string()
                        .map_err(|e| format!("Http.post read error: {}", e))?;
                    Ok(VMValue::Variant(
                        "ok".to_string(),
                        Some(Box::new(VMValue::Str(body))),
                    ))
                }
                Err(e) => Ok(VMValue::Variant(
                    "err".to_string(),
                    Some(Box::new(VMValue::Str(e.to_string()))),
                )),
            }
        }
        "Http.post_raw" => {
            if args.len() != 3 {
                return Err("Http.post_raw requires 3 arguments".to_string());
            }
            let mut it = args.into_iter();
            let url = vm_string(it.next().unwrap(), "Http.post_raw url")?;
            let body = vm_string(it.next().unwrap(), "Http.post_raw body")?;
            let content_type = vm_string(it.next().unwrap(), "Http.post_raw content_type")?;
            match ureq::post(&url)
                .set("Content-Type", &content_type)
                .send_string(&body)
            {
                Ok(resp) => {
                    let status = resp.status() as i64;
                    let response_content_type = resp
                        .header("Content-Type")
                        .unwrap_or("application/octet-stream")
                        .to_string();
                    let response_body = resp
                        .into_string()
                        .map_err(|e| format!("Http.post_raw read error: {}", e))?;
                    Ok(ok_vm(http_response_vm(
                        status,
                        response_body,
                        response_content_type,
                    )))
                }
                Err(ureq::Error::Status(status, resp)) => {
                    let body = resp.into_string().unwrap_or_default();
                    Ok(err_vm(http_error_vm(2, body, status as i64)))
                }
                Err(ureq::Error::Transport(err)) => {
                    let msg = err.to_string();
                    let code = if msg.to_ascii_lowercase().contains("timed out") {
                        1
                    } else {
                        0
                    };
                    Ok(err_vm(http_error_vm(code, msg, 0)))
                }
            }
        }
        // ── Http.get_body_raw / Http.post_body_raw (v9.5.0) ────────────────────
        "Http.get_body_raw" => {
            let url = vm_string(
                args.into_iter()
                    .next()
                    .ok_or_else(|| "Http.get_body_raw requires a URL argument".to_string())?,
                "Http.get_body_raw",
            )?;
            match ureq::get(&url).call() {
                Ok(resp) => {
                    let body = resp
                        .into_string()
                        .map_err(|e| format!("Http.get_body_raw read error: {}", e))?;
                    Ok(ok_vm(VMValue::Str(body)))
                }
                Err(ureq::Error::Status(_, resp)) => {
                    let msg = resp.into_string().unwrap_or_default();
                    Ok(err_vm(VMValue::Str(msg)))
                }
                Err(ureq::Error::Transport(err)) => Ok(err_vm(VMValue::Str(err.to_string()))),
            }
        }
        "Http.post_body_raw" => {
            if args.len() != 3 {
                return Err("Http.post_body_raw requires 3 arguments".to_string());
            }
            let mut it = args.into_iter();
            let url = vm_string(it.next().unwrap(), "Http.post_body_raw url")?;
            let body = vm_string(it.next().unwrap(), "Http.post_body_raw body")?;
            let content_type = vm_string(it.next().unwrap(), "Http.post_body_raw content_type")?;
            match ureq::post(&url)
                .set("Content-Type", &content_type)
                .send_string(&body)
            {
                Ok(resp) => {
                    let resp_body = resp
                        .into_string()
                        .map_err(|e| format!("Http.post_body_raw read error: {}", e))?;
                    Ok(ok_vm(VMValue::Str(resp_body)))
                }
                Err(ureq::Error::Status(_, resp)) => {
                    let msg = resp.into_string().unwrap_or_default();
                    Ok(err_vm(VMValue::Str(msg)))
                }
                Err(ureq::Error::Transport(err)) => Ok(err_vm(VMValue::Str(err.to_string()))),
            }
        }

        // ── Llm.complete_raw / Llm.chat_raw / Llm.extract_raw (v9.6.0) ─────────
        "Llm.complete_raw" => {
            let prompt = vm_string(
                args.into_iter()
                    .next()
                    .ok_or_else(|| "Llm.complete_raw requires a prompt argument".to_string())?,
                "Llm.complete_raw",
            )?;
            Ok(llm_call_complete(&prompt))
        }
        "Llm.chat_raw" => {
            let messages_json = vm_string(
                args.into_iter()
                    .next()
                    .ok_or_else(|| "Llm.chat_raw requires a messages argument".to_string())?,
                "Llm.chat_raw",
            )?;
            Ok(llm_call_chat(&messages_json))
        }
        "Llm.extract_raw" => {
            if args.len() != 3 {
                return Err("Llm.extract_raw requires 3 arguments".to_string());
            }
            let mut it = args.into_iter();
            let schema_name = vm_string(it.next().unwrap(), "Llm.extract_raw schema_name")?;
            let prompt = vm_string(it.next().unwrap(), "Llm.extract_raw prompt")?;
            let data = vm_string(it.next().unwrap(), "Llm.extract_raw data")?;
            let full_prompt = format!(
                "Extract data as JSON matching the schema '{}'. {}\n\nData:\n{}",
                schema_name, prompt, data
            );
            Ok(llm_call_complete(&full_prompt))
        }

        // ── Snowflake.execute_raw / Snowflake.query_raw (v10.2.0) ────────────────
        "Snowflake.execute_raw" => {
            let sql = vm_string(
                args.into_iter()
                    .next()
                    .ok_or_else(|| "Snowflake.execute_raw requires a sql argument".to_string())?,
                "Snowflake.execute_raw",
            )?;
            let account   = match snowflake_read_env("SNOWFLAKE_ACCOUNT")      { Ok(v) => v, Err(e) => return Ok(err_vm(VMValue::Str(e))) };
            let user      = match snowflake_read_env("SNOWFLAKE_USER")         { Ok(v) => v, Err(e) => return Ok(err_vm(VMValue::Str(e))) };
            let privkey   = match snowflake_read_env("SNOWFLAKE_PRIVATE_KEY")  { Ok(v) => v, Err(e) => return Ok(err_vm(VMValue::Str(e))) };
            let pubkey_fp = match snowflake_read_env("SNOWFLAKE_PUBLIC_KEY_FP") { Ok(v) => v, Err(e) => return Ok(err_vm(VMValue::Str(e))) };
            let jwt = match snowflake_generate_jwt(&account, &user, &privkey, &pubkey_fp) {
                Ok(t) => t,
                Err(e) => return Ok(err_vm(VMValue::Str(e))),
            };
            let mut body = serde_json::json!({ "statement": sql, "timeout": 60 });
            if let Ok(wh) = std::env::var("SNOWFLAKE_WAREHOUSE") { body["warehouse"] = serde_json::Value::String(wh); }
            if let Ok(rl) = std::env::var("SNOWFLAKE_ROLE")      { body["role"]      = serde_json::Value::String(rl); }
            if let Ok(db) = std::env::var("SNOWFLAKE_DATABASE")  { body["database"]  = serde_json::Value::String(db); }
            if let Ok(sc) = std::env::var("SNOWFLAKE_SCHEMA")    { body["schema"]    = serde_json::Value::String(sc); }
            match snowflake_api_post(&account, &jwt, &body) {
                Ok(_)  => Ok(ok_vm(VMValue::Str("ok".to_string()))),
                Err(e) => Ok(err_vm(VMValue::Str(e))),
            }
        }
        "Snowflake.query_raw" => {
            let sql = vm_string(
                args.into_iter()
                    .next()
                    .ok_or_else(|| "Snowflake.query_raw requires a sql argument".to_string())?,
                "Snowflake.query_raw",
            )?;
            let account   = match snowflake_read_env("SNOWFLAKE_ACCOUNT")      { Ok(v) => v, Err(e) => return Ok(err_vm(VMValue::Str(e))) };
            let user      = match snowflake_read_env("SNOWFLAKE_USER")         { Ok(v) => v, Err(e) => return Ok(err_vm(VMValue::Str(e))) };
            let privkey   = match snowflake_read_env("SNOWFLAKE_PRIVATE_KEY")  { Ok(v) => v, Err(e) => return Ok(err_vm(VMValue::Str(e))) };
            let pubkey_fp = match snowflake_read_env("SNOWFLAKE_PUBLIC_KEY_FP") { Ok(v) => v, Err(e) => return Ok(err_vm(VMValue::Str(e))) };
            let jwt = match snowflake_generate_jwt(&account, &user, &privkey, &pubkey_fp) {
                Ok(t) => t,
                Err(e) => return Ok(err_vm(VMValue::Str(e))),
            };
            let mut body = serde_json::json!({ "statement": sql, "timeout": 60 });
            if let Ok(wh) = std::env::var("SNOWFLAKE_WAREHOUSE") { body["warehouse"] = serde_json::Value::String(wh); }
            if let Ok(rl) = std::env::var("SNOWFLAKE_ROLE")      { body["role"]      = serde_json::Value::String(rl); }
            if let Ok(db) = std::env::var("SNOWFLAKE_DATABASE")  { body["database"]  = serde_json::Value::String(db); }
            if let Ok(sc) = std::env::var("SNOWFLAKE_SCHEMA")    { body["schema"]    = serde_json::Value::String(sc); }
            match snowflake_api_post(&account, &jwt, &body) {
                Ok(resp) => {
                    let cols: Vec<String> = resp["resultSetMetaData"]["rowType"]
                        .as_array()
                        .unwrap_or(&vec![])
                        .iter()
                        .map(|c| c["name"].as_str().unwrap_or("").to_string())
                        .collect();
                    let rows: Vec<serde_json::Value> = resp["data"]
                        .as_array()
                        .unwrap_or(&vec![])
                        .iter()
                        .map(|row| {
                            let mut obj = serde_json::Map::new();
                            for (i, col) in cols.iter().enumerate() {
                                obj.insert(col.clone(), row[i].clone());
                            }
                            serde_json::Value::Object(obj)
                        })
                        .collect();
                    let json_str = serde_json::to_string(&rows).unwrap_or_default();
                    Ok(ok_vm(VMValue::Str(json_str)))
                }
                Err(e) => Ok(err_vm(VMValue::Str(e))),
            }
        }

        // ── Snowflake.infer_table_raw (v10.8.0) ──────────────────────────────────
        "Snowflake.infer_table_raw" => {
            let table = vm_string(
                args.into_iter()
                    .next()
                    .ok_or_else(|| "Snowflake.infer_table_raw requires a table argument".to_string())?,
                "Snowflake.infer_table_raw",
            )?;
            let account   = match snowflake_read_env("SNOWFLAKE_ACCOUNT")      { Ok(v) => v, Err(e) => return Ok(err_vm(VMValue::Str(e))) };
            let user      = match snowflake_read_env("SNOWFLAKE_USER")         { Ok(v) => v, Err(e) => return Ok(err_vm(VMValue::Str(e))) };
            let privkey   = match snowflake_read_env("SNOWFLAKE_PRIVATE_KEY")  { Ok(v) => v, Err(e) => return Ok(err_vm(VMValue::Str(e))) };
            let pubkey_fp = match snowflake_read_env("SNOWFLAKE_PUBLIC_KEY_FP") { Ok(v) => v, Err(e) => return Ok(err_vm(VMValue::Str(e))) };
            let jwt = match snowflake_generate_jwt(&account, &user, &privkey, &pubkey_fp) {
                Ok(t) => t,
                Err(e) => return Ok(err_vm(VMValue::Str(e))),
            };
            let sql = format!(
                "SELECT COLUMN_NAME, DATA_TYPE, IS_NULLABLE \
                 FROM INFORMATION_SCHEMA.COLUMNS \
                 WHERE TABLE_NAME = '{}' \
                 ORDER BY ORDINAL_POSITION",
                table.to_uppercase()
            );
            let body = serde_json::json!({ "statement": sql, "timeout": 60 });
            match snowflake_api_post(&account, &jwt, &body) {
                Err(e) => Ok(err_vm(VMValue::Str(e))),
                Ok(resp) => {
                    let empty_vec = vec![];
                    let cols: Vec<String> = resp["resultSetMetaData"]["rowType"]
                        .as_array().unwrap_or(&empty_vec)
                        .iter().map(|c| c["name"].as_str().unwrap_or("").to_string()).collect();
                    let col_idx = |name: &str| cols.iter().position(|c| c == name).unwrap_or(usize::MAX);
                    let name_idx     = col_idx("COLUMN_NAME");
                    let type_idx     = col_idx("DATA_TYPE");
                    let nullable_idx = col_idx("IS_NULLABLE");
                    let rows_data: Vec<Vec<String>> = resp["data"]
                        .as_array().unwrap_or(&empty_vec)
                        .iter()
                        .map(|row| row.as_array().unwrap_or(&empty_vec)
                            .iter().map(|v| v.as_str().unwrap_or("").to_string()).collect())
                        .collect();
                    // Build a simple type def string
                    let type_name_str = {
                        let t = table.to_uppercase();
                        let stripped = if t.ends_with('S') && t.len() > 1 { &t[..t.len()-1] } else { &t };
                        stripped.split('_').map(|seg| {
                            let mut c = seg.chars();
                            match c.next() {
                                None => String::new(),
                                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                            }
                        }).collect::<String>()
                    };
                    let source = format!("--from snowflake --table {}", table.to_uppercase());
                    let max_len = rows_data.iter()
                        .map(|r| r.get(name_idx).map(|s| s.len()).unwrap_or(0))
                        .max().unwrap_or(0);
                    let mut out = format!(
                        "// auto-generated by `fav infer {}`\n// Review and adjust before use.\ntype {} = {{\n",
                        source, type_name_str
                    );
                    for row in &rows_data {
                        let col_name   = row.get(name_idx).cloned().unwrap_or_default().to_lowercase();
                        let col_type   = row.get(type_idx).cloned().unwrap_or_default();
                        let nullable_s = row.get(nullable_idx).cloned().unwrap_or_default();
                        let nullable   = matches!(nullable_s.to_uppercase().as_str(), "YES" | "Y");
                        let fav_type = match col_type.to_uppercase().as_str() {
                            "NUMBER" | "DECIMAL" | "NUMERIC"
                            | "INT" | "INTEGER" | "BIGINT" | "SMALLINT" | "TINYINT" | "BYTEINT" => "Int",
                            "FLOAT" | "FLOAT4" | "FLOAT8" | "DOUBLE" | "REAL" => "Float",
                            "BOOLEAN" => "Bool",
                            _ => "String",
                        };
                        let fav_type_str = if nullable { format!("Option<{}>", fav_type) } else { fav_type.to_string() };
                        let padding = " ".repeat(max_len.saturating_sub(col_name.len()));
                        out.push_str(&format!("    {}:{} {}\n", col_name, padding, fav_type_str));
                    }
                    out.push_str("}\n");
                    Ok(ok_vm(VMValue::Str(out)))
                }
            }
        }

        // ── AzurePostgres.execute_raw / AzurePostgres.query_raw (v14.1.0) ──
        "AzurePostgres.execute_raw" => {
            let mut it = args.into_iter();
            let conn_str = vm_string(
                it.next().ok_or_else(|| "AzurePostgres.execute_raw requires conn_str".to_string())?,
                "AzurePostgres.execute_raw conn_str",
            )?;
            let sql = vm_string(
                it.next().ok_or_else(|| "AzurePostgres.execute_raw requires sql".to_string())?,
                "AzurePostgres.execute_raw sql",
            )?;
            let params_json = vm_string(
                it.next().ok_or_else(|| "AzurePostgres.execute_raw requires params".to_string())?,
                "AzurePostgres.execute_raw params",
            )?;
            match azure_pg_execute(&conn_str, &sql, &params_json) {
                Ok(n)  => Ok(ok_vm(VMValue::Int(n))),
                Err(e) => Ok(err_vm(VMValue::Str(e))),
            }
        }
        "AzurePostgres.query_raw" => {
            let mut it = args.into_iter();
            let conn_str = vm_string(
                it.next().ok_or_else(|| "AzurePostgres.query_raw requires conn_str".to_string())?,
                "AzurePostgres.query_raw conn_str",
            )?;
            let sql = vm_string(
                it.next().ok_or_else(|| "AzurePostgres.query_raw requires sql".to_string())?,
                "AzurePostgres.query_raw sql",
            )?;
            let params_json = vm_string(
                it.next().ok_or_else(|| "AzurePostgres.query_raw requires params".to_string())?,
                "AzurePostgres.query_raw params",
            )?;
            match pg_query(&conn_str, &sql, &params_json) {
                Ok(json) => Ok(ok_vm(VMValue::Str(json))),
                Err(e)   => Ok(err_vm(VMValue::Str(e))),
            }
        }

        // ── Postgres.execute_raw / Postgres.query_raw / Postgres.infer_table_raw (v11.5.0) ──
        "Postgres.execute_raw" => {
            let mut it = args.into_iter();
            let sql = vm_string(
                it.next().ok_or_else(|| "Postgres.execute_raw requires sql".to_string())?,
                "Postgres.execute_raw sql",
            )?;
            let params_json = vm_string(
                it.next().ok_or_else(|| "Postgres.execute_raw requires params".to_string())?,
                "Postgres.execute_raw params",
            )?;
            let conn_str = pg_conn_str_from_env();
            match pg_execute(&conn_str, &sql, &params_json) {
                Ok(())  => Ok(ok_vm(VMValue::Unit)),
                Err(e)  => Ok(err_vm(VMValue::Str(e))),
            }
        }
        "Postgres.query_raw" => {
            let mut it = args.into_iter();
            let sql = vm_string(
                it.next().ok_or_else(|| "Postgres.query_raw requires sql".to_string())?,
                "Postgres.query_raw sql",
            )?;
            let params_json = vm_string(
                it.next().ok_or_else(|| "Postgres.query_raw requires params".to_string())?,
                "Postgres.query_raw params",
            )?;
            let conn_str = pg_conn_str_from_env();
            match pg_query(&conn_str, &sql, &params_json) {
                Ok(json) => Ok(ok_vm(VMValue::Str(json))),
                Err(e)   => Ok(err_vm(VMValue::Str(e))),
            }
        }
        "Postgres.infer_table_raw" => {
            let table = vm_string(
                args.into_iter().next().ok_or_else(|| "Postgres.infer_table_raw requires table".to_string())?,
                "Postgres.infer_table_raw",
            )?;
            let conn_str = pg_conn_str_from_env();
            let sql = "SELECT column_name, data_type, is_nullable \
                       FROM information_schema.columns \
                       WHERE table_name = $1 \
                       ORDER BY ordinal_position";
            match pg_query(&conn_str, sql, &format!("[\"{}\"]", table)) {
                Err(e) => Ok(err_vm(VMValue::Str(e))),
                Ok(json_str) => {
                    let rows: Vec<serde_json::Value> = match serde_json::from_str(&json_str) {
                        Ok(v) => v,
                        Err(e) => return Ok(err_vm(VMValue::Str(e.to_string()))),
                    };
                    let type_name_str = {
                        let t = table.to_uppercase();
                        let stripped = if t.ends_with('S') && t.len() > 1 { &t[..t.len()-1] } else { &t };
                        stripped.split('_').map(|seg| {
                            let mut c = seg.chars();
                            match c.next() {
                                None => String::new(),
                                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                            }
                        }).collect::<String>()
                    };
                    let max_len = rows.iter()
                        .filter_map(|r| r["column_name"].as_str())
                        .map(|s| s.len()).max().unwrap_or(0);
                    let mut out = format!(
                        "// auto-generated by `fav infer --from postgres --table {}`\n// Review and adjust before use.\ntype {} = {{\n",
                        table, type_name_str
                    );
                    for row in &rows {
                        let col_name   = row["column_name"].as_str().unwrap_or("").to_lowercase();
                        let col_type   = row["data_type"].as_str().unwrap_or("");
                        let nullable_s = row["is_nullable"].as_str().unwrap_or("NO");
                        let nullable   = nullable_s.to_uppercase() == "YES";
                        let fav_type = match col_type.to_lowercase().as_str() {
                            "integer" | "int" | "int2" | "int4" | "int8" | "bigint" | "smallint" | "serial" | "bigserial" => "Int",
                            "real" | "double precision" | "float4" | "float8" | "numeric" | "decimal" => "Float",
                            "boolean" | "bool" => "Bool",
                            _ => "String",
                        };
                        let fav_type_str = if nullable { format!("Option<{}>", fav_type) } else { fav_type.to_string() };
                        let padding = " ".repeat(max_len.saturating_sub(col_name.len()));
                        out.push_str(&format!("    {}:{} {}\n", col_name, padding, fav_type_str));
                    }
                    out.push_str("}\n");
                    Ok(ok_vm(VMValue::Str(out)))
                }
            }
        }

        // ── v20.8.0: Postgres.Pool primitives ────────────────────────────────────────

        "Postgres.Pool.create" => {
            let pool_size = match args.into_iter().next() {
                Some(VMValue::Int(n)) if n > 0 => n as usize,
                _ => 5,
            };
            let conn_str = pg_conn_str_from_env();
            let inner = crate::backend::pg_pool::PgPoolInner::new(&conn_str, pool_size);
            let id = pg_pool_alloc(inner);
            Ok(ok_vm(VMValue::PgPool(id)))
        }

        "Postgres.Pool.query" => {
            let mut it = args.into_iter();
            let id = match it.next() {
                Some(VMValue::PgPool(id)) => id,
                _ => return Ok(err_vm(VMValue::Str("Postgres.Pool.query: expected PgPool as first argument".into()))),
            };
            let sql = match it.next() {
                Some(VMValue::Str(s)) => s,
                _ => return Ok(err_vm(VMValue::Str("Postgres.Pool.query: expected sql Str".into()))),
            };
            let params_json = match it.next() {
                Some(v) => vmvalue_to_params_json(&v),
                None => "[]".to_string(),
            };
            let inner = pg_pool_store().get(&id).cloned();
            match inner {
                None => Ok(err_vm(VMValue::Str(format!("Postgres.Pool.query: invalid pool id {id}")))),
                Some(pool) => {
                    match pool.acquire() {
                        Err(e) => Ok(err_vm(VMValue::Str(e))),
                        Ok(client) => {
                            let params = match pg_params_from_json(&params_json) {
                                Ok(p) => p,
                                Err(e) => { pool.release(client); return Ok(err_vm(VMValue::Str(e))); }
                            };
                            let result = pg_pool_runtime().block_on(async {
                                pg_query_with_client(&client, &sql, &params).await
                            });
                            pool.release(client);
                            match result {
                                Ok(rows) => Ok(ok_vm(VMValue::List(FavList::new(rows)))),
                                Err(e)   => Ok(err_vm(VMValue::Str(e))),
                            }
                        }
                    }
                }
            }
        }

        "Postgres.Pool.execute" => {
            let mut it = args.into_iter();
            let id = match it.next() {
                Some(VMValue::PgPool(id)) => id,
                _ => return Ok(err_vm(VMValue::Str("Postgres.Pool.execute: expected PgPool as first argument".into()))),
            };
            let sql = match it.next() {
                Some(VMValue::Str(s)) => s,
                _ => return Ok(err_vm(VMValue::Str("Postgres.Pool.execute: expected sql Str".into()))),
            };
            let params_json = match it.next() {
                Some(v) => vmvalue_to_params_json(&v),
                None => "[]".to_string(),
            };
            let inner = pg_pool_store().get(&id).cloned();
            match inner {
                None => Ok(err_vm(VMValue::Str(format!("Postgres.Pool.execute: invalid pool id {id}")))),
                Some(pool) => {
                    match pool.acquire() {
                        Err(e) => Ok(err_vm(VMValue::Str(e))),
                        Ok(client) => {
                            let params = match pg_params_from_json(&params_json) {
                                Ok(p) => p,
                                Err(e) => { pool.release(client); return Ok(err_vm(VMValue::Str(e))); }
                            };
                            let result = pg_pool_runtime().block_on(async {
                                let param_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> =
                                    params.iter().map(|s| s as &(dyn tokio_postgres::types::ToSql + Sync)).collect();
                                client.execute(sql.as_str(), &param_refs).await
                                    .map(|n| n as i64)
                                    .map_err(|e| format_pg_error(&e))
                            });
                            pool.release(client);
                            match result {
                                Ok(n)  => Ok(ok_vm(VMValue::Int(n))),
                                Err(e) => Ok(err_vm(VMValue::Str(e))),
                            }
                        }
                    }
                }
            }
        }

        "Postgres.Pool.stats" => {
            let id = match args.into_iter().next() {
                Some(VMValue::PgPool(id)) => id,
                _ => return Ok(err_vm(VMValue::Str("Postgres.Pool.stats: expected PgPool".into()))),
            };
            match pg_pool_store().get(&id).cloned() {
                None => Ok(err_vm(VMValue::Str(format!("Postgres.Pool.stats: invalid pool id {id}")))),
                Some(pool) => {
                    let s = pool.stats_snapshot();
                    let mut map = std::collections::HashMap::new();
                    map.insert("borrow_count".to_string(), VMValue::Int(s.borrow_count as i64));
                    map.insert("miss_count".to_string(),   VMValue::Int(s.miss_count   as i64));
                    map.insert("return_count".to_string(), VMValue::Int(s.return_count as i64));
                    map.insert("error_count".to_string(),  VMValue::Int(s.error_count  as i64));
                    map.insert("idle_count".to_string(),   VMValue::Int(s.idle_count   as i64));
                    Ok(ok_vm(VMValue::Record(map)))
                }
            }
        }

        "Postgres.Pool.close" => {
            let id = match args.into_iter().next() {
                Some(VMValue::PgPool(id)) => id,
                _ => return Ok(err_vm(VMValue::Str("Postgres.Pool.close: expected PgPool".into()))),
            };
            pg_pool_store().remove(&id);
            Ok(ok_vm(VMValue::Unit))
        }

        // ── v25.1.0: 接続オブジェクト API ─────────────────────────────────────────

        // Postgres.connect_raw — PgConfig（接続文字列）を受け取り接続確認後 PgConn を返す
        "Postgres.connect_raw" => {
            let conn_str = vm_string(
                args.into_iter().next().ok_or_else(|| "Postgres.connect_raw requires config".to_string())?,
                "Postgres.connect_raw config",
            )?;
            // 接続文字列を PgConn（String）としてそのまま返す
            // 実環境では接続確認 ping を行う
            Ok(ok_vm(VMValue::Str(conn_str)))
        }

        // Postgres.execute_many_raw — 同一 SQL を複数行に対してバッチ実行する
        // 注意: 各行は autocommit で実行される。部分失敗を防ぐには transaction_begin_raw / commit_raw で囲むこと。
        "Postgres.execute_many_raw" => {
            let mut it = args.into_iter();
            let conn_str = vm_string(
                it.next().ok_or_else(|| "Postgres.execute_many_raw requires conn".to_string())?,
                "Postgres.execute_many_raw conn",
            )?;
            let sql = vm_string(
                it.next().ok_or_else(|| "Postgres.execute_many_raw requires sql".to_string())?,
                "Postgres.execute_many_raw sql",
            )?;
            let rows_val = it.next().ok_or_else(|| "Postgres.execute_many_raw requires rows".to_string())?;
            let rows = match rows_val {
                VMValue::List(fl) => fl,
                _ => return Ok(err_vm(VMValue::Str("Postgres.execute_many_raw: expected List for rows".into()))),
            };
            let mut total: i64 = 0;
            for row in rows.iter() {
                let params_json = vmvalue_to_params_json(&row);
                match pg_execute(&conn_str, &sql, &params_json) {
                    Ok(()) => total += 1,
                    Err(e) => return Ok(err_vm(VMValue::Str(e))),
                }
            }
            Ok(ok_vm(VMValue::Int(total)))
        }

        // Postgres.transaction_begin_raw — BEGIN を発行する（client.fav の transaction<T> が呼ぶ）
        "Postgres.transaction_begin_raw" => {
            let conn_str = vm_string(
                args.into_iter().next().ok_or_else(|| "Postgres.transaction_begin_raw requires conn".to_string())?,
                "Postgres.transaction_begin_raw conn",
            )?;
            match pg_execute(&conn_str, "BEGIN", "[]") {
                Ok(()) => Ok(ok_vm(VMValue::Unit)),
                Err(e) => Ok(err_vm(VMValue::Str(format!("transaction BEGIN failed: {e}")))),
            }
        }

        // Postgres.transaction_commit_raw — COMMIT を発行する
        "Postgres.transaction_commit_raw" => {
            let conn_str = vm_string(
                args.into_iter().next().ok_or_else(|| "Postgres.transaction_commit_raw requires conn".to_string())?,
                "Postgres.transaction_commit_raw conn",
            )?;
            match pg_execute(&conn_str, "COMMIT", "[]") {
                Ok(()) => Ok(ok_vm(VMValue::Unit)),
                Err(e) => Ok(err_vm(VMValue::Str(format!("transaction COMMIT failed: {e}")))),
            }
        }

        // Postgres.transaction_rollback_raw — ROLLBACK を発行する（エラーはベストエフォートで無視）
        "Postgres.transaction_rollback_raw" => {
            let conn_str = vm_string(
                args.into_iter().next().ok_or_else(|| "Postgres.transaction_rollback_raw requires conn".to_string())?,
                "Postgres.transaction_rollback_raw conn",
            )?;
            // ROLLBACK 失敗は無視（接続断の場合はサーバー側でタイムアウト後にロールバック）
            let _ = pg_execute(&conn_str, "ROLLBACK", "[]");
            Ok(ok_vm(VMValue::Unit))
        }

        // Postgres.execute_with_conn_raw — PgConn（接続文字列）経由で DML を実行する（HIGH-4 対応）
        "Postgres.execute_with_conn_raw" => {
            let mut it = args.into_iter();
            let conn_str = vm_string(
                it.next().ok_or_else(|| "Postgres.execute_with_conn_raw requires conn".to_string())?,
                "Postgres.execute_with_conn_raw conn",
            )?;
            let sql = vm_string(
                it.next().ok_or_else(|| "Postgres.execute_with_conn_raw requires sql".to_string())?,
                "Postgres.execute_with_conn_raw sql",
            )?;
            let params_json = vm_string(
                it.next().ok_or_else(|| "Postgres.execute_with_conn_raw requires params".to_string())?,
                "Postgres.execute_with_conn_raw params",
            )?;
            match pg_execute(&conn_str, &sql, &params_json) {
                Ok(()) => Ok(ok_vm(VMValue::Unit)),
                Err(e)  => Ok(err_vm(VMValue::Str(e))),
            }
        }

        // Postgres.query_with_conn_raw — PgConn（接続文字列）経由でクエリを実行する（HIGH-3 対応）
        "Postgres.query_with_conn_raw" => {
            let mut it = args.into_iter();
            let conn_str = vm_string(
                it.next().ok_or_else(|| "Postgres.query_with_conn_raw requires conn".to_string())?,
                "Postgres.query_with_conn_raw conn",
            )?;
            let sql = vm_string(
                it.next().ok_or_else(|| "Postgres.query_with_conn_raw requires sql".to_string())?,
                "Postgres.query_with_conn_raw sql",
            )?;
            let params_json = vm_string(
                it.next().ok_or_else(|| "Postgres.query_with_conn_raw requires params".to_string())?,
                "Postgres.query_with_conn_raw params",
            )?;
            match pg_query(&conn_str, &sql, &params_json) {
                Ok(json) => Ok(ok_vm(VMValue::Str(json))),
                Err(e)   => Ok(err_vm(VMValue::Str(e))),
            }
        }

        // Postgres.pool_create_with_config_raw — PoolConfig（接続文字列）からプールを作成する
        "Postgres.pool_create_with_config_raw" => {
            let config_str = vm_string(
                args.into_iter().next().ok_or_else(|| "Postgres.pool_create_with_config_raw requires config".to_string())?,
                "Postgres.pool_create_with_config_raw config",
            )?;
            // max_size を config 文字列から抽出（"max_size=N" を探す）
            let pool_size: usize = config_str.split_whitespace()
                .find_map(|kv| {
                    let mut parts = kv.splitn(2, '=');
                    if parts.next()? == "max_size" {
                        parts.next()?.parse().ok()
                    } else {
                        None
                    }
                })
                .unwrap_or(5);
            // max_size= を除いた残りを接続文字列として使う
            let conn_str: String = config_str.split_whitespace()
                .filter(|kv| !kv.starts_with("max_size="))
                .collect::<Vec<_>>()
                .join(" ");
            let effective_conn = if conn_str.is_empty() { pg_conn_str_from_env() } else { conn_str };
            let inner = crate::backend::pg_pool::PgPoolInner::new(&effective_conn, pool_size);
            let id = pg_pool_alloc(inner);
            Ok(ok_vm(VMValue::PgPool(id)))
        }

        // Postgres.pool_get_raw — プールから接続を取得し PgConn（接続文字列）を返す
        // .cloned() で MutexGuard を即座に解放（既存 Pool.query/Pool.execute と同パターン）
        "Postgres.pool_get_raw" => {
            let id = match args.into_iter().next() {
                Some(VMValue::PgPool(id)) => id,
                _ => return Ok(err_vm(VMValue::Str("Postgres.pool_get_raw: expected PgPool".into()))),
            };
            let inner = pg_pool_store().get(&id).cloned();
            match inner {
                None => Ok(err_vm(VMValue::Str(format!("Postgres.pool_get_raw: invalid pool id {id}")))),
                Some(pool) => Ok(ok_vm(VMValue::Str(pool.conn_str.clone()))),
            }
        }

        // Postgres.pool_release_raw — PgConn をプールに返却（現実装では no-op）
        "Postgres.pool_release_raw" => {
            // PgConn は String（接続文字列）のため返却リソースなし
            Ok(ok_vm(VMValue::Unit))
        }

        "Http.put_raw" => {
            if args.len() != 3 {
                return Err("Http.put_raw requires 3 arguments".to_string());
            }
            let mut it = args.into_iter();
            let url = vm_string(it.next().unwrap(), "Http.put_raw url")?;
            let body = vm_string(it.next().unwrap(), "Http.put_raw body")?;
            let content_type = vm_string(it.next().unwrap(), "Http.put_raw content_type")?;
            match ureq::put(&url)
                .set("Content-Type", &content_type)
                .send_string(&body)
            {
                Ok(resp) => {
                    let status = resp.status() as i64;
                    let response_content_type = resp
                        .header("Content-Type")
                        .unwrap_or("application/octet-stream")
                        .to_string();
                    let response_body = resp
                        .into_string()
                        .map_err(|e| format!("Http.put_raw read error: {}", e))?;
                    Ok(ok_vm(http_response_vm(
                        status,
                        response_body,
                        response_content_type,
                    )))
                }
                Err(ureq::Error::Status(status, resp)) => {
                    let body = resp.into_string().unwrap_or_default();
                    Ok(err_vm(http_error_vm(2, body, status as i64)))
                }
                Err(ureq::Error::Transport(err)) => {
                    let msg = err.to_string();
                    let code = if msg.to_ascii_lowercase().contains("timed out") {
                        1
                    } else {
                        0
                    };
                    Ok(err_vm(http_error_vm(code, msg, 0)))
                }
            }
        }
        "Http.delete_raw" => {
            if args.len() != 1 {
                return Err("Http.delete_raw requires 1 argument".to_string());
            }
            let url = vm_string(args.into_iter().next().unwrap(), "Http.delete_raw url")?;
            match ureq::delete(&url).call() {
                Ok(resp) => {
                    let status = resp.status() as i64;
                    let response_content_type = resp
                        .header("Content-Type")
                        .unwrap_or("application/octet-stream")
                        .to_string();
                    let response_body = resp
                        .into_string()
                        .map_err(|e| format!("Http.delete_raw read error: {}", e))?;
                    Ok(ok_vm(http_response_vm(
                        status,
                        response_body,
                        response_content_type,
                    )))
                }
                Err(ureq::Error::Status(status, resp)) => {
                    let body = resp.into_string().unwrap_or_default();
                    Ok(err_vm(http_error_vm(2, body, status as i64)))
                }
                Err(ureq::Error::Transport(err)) => {
                    let msg = err.to_string();
                    let code = if msg.to_ascii_lowercase().contains("timed out") {
                        1
                    } else {
                        0
                    };
                    Ok(err_vm(http_error_vm(code, msg, 0)))
                }
            }
        }
        "Http.patch_raw" => {
            if args.len() != 3 {
                return Err("Http.patch_raw requires 3 arguments".to_string());
            }
            let mut it = args.into_iter();
            let url = vm_string(it.next().unwrap(), "Http.patch_raw url")?;
            let body = vm_string(it.next().unwrap(), "Http.patch_raw body")?;
            let content_type = vm_string(it.next().unwrap(), "Http.patch_raw content_type")?;
            match ureq::request("PATCH", &url)
                .set("Content-Type", &content_type)
                .send_string(&body)
            {
                Ok(resp) => {
                    let status = resp.status() as i64;
                    let response_content_type = resp
                        .header("Content-Type")
                        .unwrap_or("application/octet-stream")
                        .to_string();
                    let response_body = resp
                        .into_string()
                        .map_err(|e| format!("Http.patch_raw read error: {}", e))?;
                    Ok(ok_vm(http_response_vm(
                        status,
                        response_body,
                        response_content_type,
                    )))
                }
                Err(ureq::Error::Status(status, resp)) => {
                    let body = resp.into_string().unwrap_or_default();
                    Ok(err_vm(http_error_vm(2, body, status as i64)))
                }
                Err(ureq::Error::Transport(err)) => {
                    let msg = err.to_string();
                    let code = if msg.to_ascii_lowercase().contains("timed out") {
                        1
                    } else {
                        0
                    };
                    Ok(err_vm(http_error_vm(code, msg, 0)))
                }
            }
        }
        "Http.check_basic_auth" => {
            if args.len() != 3 {
                return Err("Http.check_basic_auth requires 3 arguments".to_string());
            }
            let mut it = args.into_iter();
            let auth_header = vm_string(it.next().unwrap(), "Http.check_basic_auth auth_header")?;
            let expected_user = vm_string(it.next().unwrap(), "Http.check_basic_auth username")?;
            let expected_pass = vm_string(it.next().unwrap(), "Http.check_basic_auth password")?;
            let ok = if let Some(encoded) = auth_header.strip_prefix("Basic ") {
                use base64::Engine as _;
                match base64::engine::general_purpose::STANDARD.decode(encoded.trim()) {
                    Ok(decoded) => {
                        if let Ok(s) = std::str::from_utf8(&decoded) {
                            let expected = format!("{}:{}", expected_user, expected_pass);
                            s == expected
                        } else {
                            false
                        }
                    }
                    Err(_) => false,
                }
            } else {
                false
            };
            Ok(VMValue::Bool(ok))
        }
        "Http.get_raw_headers" => {
            if args.len() != 2 {
                return Err("Http.get_raw_headers requires 2 arguments".to_string());
            }
            let mut it = args.into_iter();
            let url = vm_string(it.next().unwrap(), "Http.get_raw_headers url")?;
            let headers = match it.next().unwrap() {
                VMValue::Record(m) => schema_record_to_string_map(&m),
                other => {
                    return Err(format!(
                        "Http.get_raw_headers expects Map<String,String> headers, got {}",
                        vmvalue_type_name(&other)
                    ));
                }
            };
            let mut req = ureq::get(&url);
            for (k, v) in &headers {
                req = req.set(k, v);
            }
            match req.call() {
                Ok(resp) => {
                    let status = resp.status() as i64;
                    let response_content_type = resp
                        .header("Content-Type")
                        .unwrap_or("application/octet-stream")
                        .to_string();
                    let response_body = resp
                        .into_string()
                        .map_err(|e| format!("Http.get_raw_headers read error: {}", e))?;
                    Ok(ok_vm(http_response_vm(
                        status,
                        response_body,
                        response_content_type,
                    )))
                }
                Err(ureq::Error::Status(status, resp)) => {
                    let body = resp.into_string().unwrap_or_default();
                    Ok(err_vm(http_error_vm(2, body, status as i64)))
                }
                Err(ureq::Error::Transport(err)) => {
                    let msg = err.to_string();
                    let code = if msg.to_ascii_lowercase().contains("timed out") {
                        1
                    } else {
                        0
                    };
                    Ok(err_vm(http_error_vm(code, msg, 0)))
                }
            }
        }
        "Http.post_raw_headers" => {
            if args.len() != 4 {
                return Err("Http.post_raw_headers requires 4 arguments".to_string());
            }
            let mut it = args.into_iter();
            let url = vm_string(it.next().unwrap(), "Http.post_raw_headers url")?;
            let body = vm_string(it.next().unwrap(), "Http.post_raw_headers body")?;
            let content_type = vm_string(it.next().unwrap(), "Http.post_raw_headers content_type")?;
            let headers = match it.next().unwrap() {
                VMValue::Record(m) => schema_record_to_string_map(&m),
                other => {
                    return Err(format!(
                        "Http.post_raw_headers expects Map<String,String> headers, got {}",
                        vmvalue_type_name(&other)
                    ));
                }
            };
            let mut req = ureq::post(&url).set("Content-Type", &content_type);
            for (k, v) in &headers {
                req = req.set(k, v);
            }
            match req.send_string(&body) {
                Ok(resp) => {
                    let status = resp.status() as i64;
                    let response_content_type = resp
                        .header("Content-Type")
                        .unwrap_or("application/octet-stream")
                        .to_string();
                    let response_body = resp
                        .into_string()
                        .map_err(|e| format!("Http.post_raw_headers read error: {}", e))?;
                    Ok(ok_vm(http_response_vm(
                        status,
                        response_body,
                        response_content_type,
                    )))
                }
                Err(ureq::Error::Status(status, resp)) => {
                    let body = resp.into_string().unwrap_or_default();
                    Ok(err_vm(http_error_vm(2, body, status as i64)))
                }
                Err(ureq::Error::Transport(err)) => {
                    let msg = err.to_string();
                    let code = if msg.to_ascii_lowercase().contains("timed out") {
                        1
                    } else {
                        0
                    };
                    Ok(err_vm(http_error_vm(code, msg, 0)))
                }
            }
        }
        "String.base64_encode" => {
            if args.len() != 1 {
                return Err("String.base64_encode requires 1 argument".to_string());
            }
            let s = vm_string(args.into_iter().next().unwrap(), "String.base64_encode")?;
            use base64::Engine;
            Ok(VMValue::Str(
                base64::engine::general_purpose::STANDARD.encode(s.as_bytes()),
            ))
        }
        "String.base64_decode" => {
            if args.len() != 1 {
                return Err("String.base64_decode requires 1 argument".to_string());
            }
            let s = vm_string(args.into_iter().next().unwrap(), "String.base64_decode")?;
            use base64::Engine;
            match base64::engine::general_purpose::STANDARD.decode(s.as_bytes()) {
                Ok(bytes) => {
                    let list: Vec<VMValue> =
                        bytes.into_iter().map(|b| VMValue::Int(b as i64)).collect();
                    Ok(ok_vm(VMValue::List(FavList::new(list))))
                }
                Err(e) => Ok(err_vm(VMValue::Str(e.to_string()))),
            }
        }
        "Grpc.encode_raw" => {
            if args.len() != 2 {
                return Err("Grpc.encode_raw requires 2 arguments".to_string());
            }
            let mut it = args.into_iter();
            let type_name = vm_string(it.next().unwrap(), "Grpc.encode_raw type_name")?;
            let row = match it.next().unwrap() {
                VMValue::Record(map) => schema_record_to_string_map(&map),
                other => {
                    return Err(format!(
                        "Grpc.encode_raw expects Map<String,String>, got {}",
                        vmvalue_type_name(&other)
                    ));
                }
            };
            let bytes = map_to_proto_bytes(&type_name, &row, type_metas)?;
            Ok(VMValue::Str(BASE64.encode(bytes)))
        }
        "Grpc.decode_raw" => {
            if args.len() != 2 {
                return Err("Grpc.decode_raw requires 2 arguments".to_string());
            }
            let mut it = args.into_iter();
            let type_name = vm_string(it.next().unwrap(), "Grpc.decode_raw type_name")?;
            let encoded = vm_string(it.next().unwrap(), "Grpc.decode_raw encoded")?;
            let bytes = BASE64
                .decode(encoded)
                .map_err(|e| format!("Grpc.decode_raw base64 decode failed: {}", e))?;
            let row = proto_bytes_to_map(&type_name, &bytes, type_metas)?;
            Ok(VMValue::Record(
                row.into_iter().map(|(k, v)| (k, VMValue::Str(v))).collect(),
            ))
        }
        "Grpc.call_raw" => {
            if args.len() != 3 {
                return Err("Grpc.call_raw requires 3 arguments".to_string());
            }
            let mut it = args.into_iter();
            let host = vm_string(it.next().unwrap(), "Grpc.call_raw host")?;
            let method = vm_string(it.next().unwrap(), "Grpc.call_raw method")?;
            let payload = match it.next().unwrap() {
                VMValue::Record(map) => schema_record_to_string_map(&map),
                other => {
                    return Err(format!(
                        "Grpc.call_raw expects Map<String,String>, got {}",
                        vmvalue_type_name(&other)
                    ));
                }
            };
            let proto_bytes = string_map_to_proto_bytes(&payload);
            let frame = encode_grpc_frame(&proto_bytes);
            let tcp_addr = grpc_tcp_addr(&host);
            let uri_str = grpc_method_uri(&host, &method);
            let result = std::thread::spawn(move || -> Result<VMValue, String> {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .map_err(|e| format!("Grpc.call_raw tokio build failed: {}", e))?;
                rt.block_on(async move {
                    let tcp = match tokio::net::TcpStream::connect(&tcp_addr).await {
                        Ok(s) => s,
                        Err(e) => {
                            return Ok(err_vm(rpc_error_vm(
                                14,
                                format!("connection failed: {}", e),
                            )));
                        }
                    };
                    let (mut h2_client, h2_conn) = match h2::client::handshake(tcp).await {
                        Ok(r) => r,
                        Err(e) => {
                            return Ok(err_vm(rpc_error_vm(
                                14,
                                format!("h2 handshake failed: {}", e),
                            )));
                        }
                    };
                    tokio::spawn(async move {
                        let _ = h2_conn.await;
                    });
                    let request = match http::Request::builder()
                        .method("POST")
                        .uri(uri_str.as_str())
                        .header("content-type", "application/grpc")
                        .header("te", "trailers")
                        .body(())
                    {
                        Ok(r) => r,
                        Err(e) => return Err(format!("request build failed: {}", e)),
                    };
                    let (response_future, mut send_stream) =
                        match h2_client.send_request(request, false) {
                            Ok(r) => r,
                            Err(e) => {
                                return Ok(err_vm(rpc_error_vm(
                                    14,
                                    format!("send_request failed: {}", e),
                                )));
                            }
                        };
                    if let Err(e) = send_stream.send_data(Bytes::from(frame), true) {
                        return Ok(err_vm(rpc_error_vm(14, format!("send_data failed: {}", e))));
                    }
                    let response = match response_future.await {
                        Ok(r) => r,
                        Err(e) => {
                            return Ok(err_vm(rpc_error_vm(14, format!("response failed: {}", e))));
                        }
                    };
                    if !response.status().is_success() {
                        return Ok(err_vm(rpc_error_vm(
                            14,
                            format!("HTTP {}", response.status()),
                        )));
                    }
                    let mut body = response.into_body();
                    let mut resp_bytes: Vec<u8> = Vec::new();
                    while let Some(chunk) = body.data().await {
                        match chunk {
                            Ok(data) => {
                                let n = data.len();
                                resp_bytes.extend_from_slice(&data);
                                let _ = body.flow_control().release_capacity(n);
                            }
                            Err(e) => {
                                return Ok(err_vm(rpc_error_vm(
                                    14,
                                    format!("body read failed: {}", e),
                                )));
                            }
                        }
                    }
                    // Check gRPC status from trailers
                    if let Ok(Some(trailers)) = body.trailers().await {
                        if let Some(grpc_status) = trailers.get("grpc-status") {
                            if grpc_status.as_bytes() != b"0" {
                                let msg = trailers
                                    .get("grpc-message")
                                    .and_then(|v| v.to_str().ok())
                                    .unwrap_or("gRPC error")
                                    .to_string();
                                let code: i64 = grpc_status
                                    .to_str()
                                    .ok()
                                    .and_then(|s| s.parse().ok())
                                    .unwrap_or(2);
                                return Ok(err_vm(rpc_error_vm(code, msg)));
                            }
                        }
                    }
                    let proto = match decode_grpc_frame(&resp_bytes) {
                        Ok(b) => b,
                        Err(e) => return Err(format!("decode_grpc_frame failed: {}", e)),
                    };
                    let row = match proto_bytes_to_string_map(&proto) {
                        Ok(m) => m,
                        Err(e) => return Err(format!("proto_bytes_to_string_map failed: {}", e)),
                    };
                    Ok(ok_vm(VMValue::Record(
                        row.into_iter().map(|(k, v)| (k, VMValue::Str(v))).collect(),
                    )))
                })
            })
            .join()
            .map_err(|_| "Grpc.call_raw thread panicked".to_string())??;
            Ok(result)
        }
        "Grpc.call_stream_raw" => {
            if args.len() != 3 {
                return Err("Grpc.call_stream_raw requires 3 arguments".to_string());
            }
            let mut it = args.into_iter();
            let host = vm_string(it.next().unwrap(), "Grpc.call_stream_raw host")?;
            let method = vm_string(it.next().unwrap(), "Grpc.call_stream_raw method")?;
            let payload = match it.next().unwrap() {
                VMValue::Record(map) => schema_record_to_string_map(&map),
                other => {
                    return Err(format!(
                        "Grpc.call_stream_raw expects Map<String,String>, got {}",
                        vmvalue_type_name(&other)
                    ));
                }
            };
            let proto_bytes = string_map_to_proto_bytes(&payload);
            let frame = encode_grpc_frame(&proto_bytes);
            let tcp_addr = grpc_tcp_addr(&host);
            let uri_str = grpc_method_uri(&host, &method);
            let result = std::thread::spawn(move || -> Result<VMValue, String> {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .map_err(|e| format!("Grpc.call_stream_raw tokio build failed: {}", e))?;
                rt.block_on(async move {
                    let tcp = match tokio::net::TcpStream::connect(&tcp_addr).await {
                        Ok(s) => s,
                        Err(_) => return Ok(VMValue::List(FavList::new(vec![]))),
                    };
                    let (mut h2_client, h2_conn) = match h2::client::handshake(tcp).await {
                        Ok(r) => r,
                        Err(_) => return Ok(VMValue::List(FavList::new(vec![]))),
                    };
                    tokio::spawn(async move {
                        let _ = h2_conn.await;
                    });
                    let request = match http::Request::builder()
                        .method("POST")
                        .uri(uri_str.as_str())
                        .header("content-type", "application/grpc")
                        .header("te", "trailers")
                        .body(())
                    {
                        Ok(r) => r,
                        Err(_) => return Ok(VMValue::List(FavList::new(vec![]))),
                    };
                    let (response_future, mut send_stream) =
                        match h2_client.send_request(request, false) {
                            Ok(r) => r,
                            Err(_) => return Ok(VMValue::List(FavList::new(vec![]))),
                        };
                    if send_stream.send_data(Bytes::from(frame), true).is_err() {
                        return Ok(VMValue::List(FavList::new(vec![])));
                    }
                    let response = match response_future.await {
                        Ok(r) => r,
                        Err(_) => return Ok(VMValue::List(FavList::new(vec![]))),
                    };
                    let mut body = response.into_body();
                    let mut resp_bytes: Vec<u8> = Vec::new();
                    while let Some(chunk) = body.data().await {
                        match chunk {
                            Ok(data) => {
                                let n = data.len();
                                resp_bytes.extend_from_slice(&data);
                                let _ = body.flow_control().release_capacity(n);
                            }
                            Err(_) => return Ok(VMValue::List(FavList::new(vec![]))),
                        }
                    }
                    let rows = decode_all_grpc_frames(&resp_bytes)?
                        .into_iter()
                        .map(|bytes| {
                            proto_bytes_to_string_map(&bytes).map(|row| {
                                VMValue::Record(
                                    row.into_iter().map(|(k, v)| (k, VMValue::Str(v))).collect(),
                                )
                            })
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    Ok(VMValue::List(FavList::new(rows)))
                })
            })
            .join()
            .map_err(|_| "Grpc.call_stream_raw thread panicked".to_string())??;
            Ok(result)
        }
        "Grpc.call_typed_raw" => {
            if args.len() != 4 {
                return Err("Grpc.call_typed_raw requires 4 arguments".to_string());
            }
            let mut it = args.into_iter();
            let response_type = vm_string(it.next().unwrap(), "Grpc.call_typed_raw response_type")?;
            let host = vm_string(it.next().unwrap(), "Grpc.call_typed_raw host")?;
            let method = vm_string(it.next().unwrap(), "Grpc.call_typed_raw method")?;
            let payload = match it.next().unwrap() {
                VMValue::Record(map) => schema_record_to_string_map(&map),
                other => {
                    return Err(format!(
                        "Grpc.call_typed_raw expects Map<String,String> payload, got {}",
                        vmvalue_type_name(&other)
                    ));
                }
            };
            let proto_bytes = string_map_to_proto_bytes(&payload);
            let frame = encode_grpc_frame(&proto_bytes);
            let tcp_addr = grpc_tcp_addr(&host);
            let uri_str = grpc_method_uri(&host, &method);
            let type_metas_clone = type_metas.clone();
            let result = std::thread::spawn(move || -> Result<VMValue, String> {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .map_err(|e| format!("Grpc.call_typed_raw tokio build failed: {}", e))?;
                rt.block_on(async move {
                    let tcp = match tokio::net::TcpStream::connect(&tcp_addr).await {
                        Ok(s) => s,
                        Err(e) => {
                            return Ok(err_vm(rpc_error_vm(
                                14,
                                format!("connection failed: {}", e),
                            )));
                        }
                    };
                    let (mut h2_client, h2_conn) = match h2::client::handshake(tcp).await {
                        Ok(r) => r,
                        Err(e) => {
                            return Ok(err_vm(rpc_error_vm(
                                14,
                                format!("h2 handshake failed: {}", e),
                            )));
                        }
                    };
                    tokio::spawn(async move {
                        let _ = h2_conn.await;
                    });
                    let request = match http::Request::builder()
                        .method("POST")
                        .uri(uri_str.as_str())
                        .header("content-type", "application/grpc")
                        .header("te", "trailers")
                        .body(())
                    {
                        Ok(r) => r,
                        Err(e) => return Err(format!("request build failed: {}", e)),
                    };
                    let (response_future, mut send_stream) =
                        match h2_client.send_request(request, false) {
                            Ok(r) => r,
                            Err(e) => {
                                return Ok(err_vm(rpc_error_vm(
                                    14,
                                    format!("send_request failed: {}", e),
                                )));
                            }
                        };
                    if let Err(e) = send_stream.send_data(Bytes::from(frame), true) {
                        return Ok(err_vm(rpc_error_vm(14, format!("send_data failed: {}", e))));
                    }
                    let response = match response_future.await {
                        Ok(r) => r,
                        Err(e) => {
                            return Ok(err_vm(rpc_error_vm(14, format!("response failed: {}", e))));
                        }
                    };
                    if !response.status().is_success() {
                        return Ok(err_vm(rpc_error_vm(
                            14,
                            format!("HTTP {}", response.status()),
                        )));
                    }
                    let mut body = response.into_body();
                    let mut resp_bytes: Vec<u8> = Vec::new();
                    while let Some(chunk) = body.data().await {
                        match chunk {
                            Ok(data) => {
                                let n = data.len();
                                resp_bytes.extend_from_slice(&data);
                                let _ = body.flow_control().release_capacity(n);
                            }
                            Err(e) => {
                                return Ok(err_vm(rpc_error_vm(
                                    14,
                                    format!("body read failed: {}", e),
                                )));
                            }
                        }
                    }
                    if let Ok(Some(trailers)) = body.trailers().await {
                        if let Some(grpc_status) = trailers.get("grpc-status") {
                            if grpc_status.as_bytes() != b"0" {
                                let msg = trailers
                                    .get("grpc-message")
                                    .and_then(|v| v.to_str().ok())
                                    .unwrap_or("gRPC error")
                                    .to_string();
                                let code: i64 = grpc_status
                                    .to_str()
                                    .ok()
                                    .and_then(|s| s.parse().ok())
                                    .unwrap_or(2);
                                return Ok(err_vm(rpc_error_vm(code, msg)));
                            }
                        }
                    }
                    let proto = match decode_grpc_frame(&resp_bytes) {
                        Ok(b) => b,
                        Err(e) => return Err(format!("decode_grpc_frame failed: {}", e)),
                    };
                    let row =
                        match proto_bytes_to_named_map(&proto, &response_type, &type_metas_clone) {
                            Ok(m) => m,
                            Err(e) => {
                                return Err(format!("proto_bytes_to_named_map failed: {}", e));
                            }
                        };
                    Ok(ok_vm(VMValue::Record(
                        row.into_iter().map(|(k, v)| (k, VMValue::Str(v))).collect(),
                    )))
                })
            })
            .join()
            .map_err(|_| "Grpc.call_typed_raw thread panicked".to_string())??;
            Ok(result)
        }
        "File.read" => {
            let path = match args.into_iter().next() {
                Some(VMValue::Str(s)) => s,
                Some(other) => {
                    return Err(format!(
                        "File.read expects String path, got {}",
                        vmvalue_type_name(&other)
                    ));
                }
                None => return Err("File.read requires 1 argument".to_string()),
            };
            let content = std::fs::read_to_string(&path)
                .map_err(|e| format!("File.read failed for `{}`: {}", path, e))?;
            Ok(VMValue::Str(content))
        }
        "File.read_lines" => {
            let path = match args.into_iter().next() {
                Some(VMValue::Str(s)) => s,
                Some(other) => {
                    return Err(format!(
                        "File.read_lines expects String path, got {}",
                        vmvalue_type_name(&other)
                    ));
                }
                None => return Err("File.read_lines requires 1 argument".to_string()),
            };
            let content = std::fs::read_to_string(&path)
                .map_err(|e| format!("File.read_lines failed for `{}`: {}", path, e))?;
            Ok(VMValue::List(FavList::new(
                content
                    .lines()
                    .map(|line| VMValue::Str(line.to_string()))
                    .collect(),
            )))
        }
        "File.write" => {
            if args.len() != 2 {
                return Err("File.write requires 2 arguments".to_string());
            }
            let mut it = args.into_iter();
            let path = match it.next().unwrap() {
                VMValue::Str(s) => s,
                other => {
                    return Err(format!(
                        "File.write expects String path, got {}",
                        vmvalue_type_name(&other)
                    ));
                }
            };
            let content = match it.next().unwrap() {
                VMValue::Str(s) => s,
                other => {
                    return Err(format!(
                        "File.write expects String content, got {}",
                        vmvalue_type_name(&other)
                    ));
                }
            };
            std::fs::write(&path, content)
                .map_err(|e| format!("File.write failed for `{}`: {}", path, e))?;
            Ok(VMValue::Unit)
        }
        "File.write_lines" => {
            if args.len() != 2 {
                return Err("File.write_lines requires 2 arguments".to_string());
            }
            let mut it = args.into_iter();
            let path = match it.next().unwrap() {
                VMValue::Str(s) => s,
                other => {
                    return Err(format!(
                        "File.write_lines expects String path, got {}",
                        vmvalue_type_name(&other)
                    ));
                }
            };
            let lines = match it.next().unwrap() {
                VMValue::List(items) => {
                    let mut parts = Vec::with_capacity(items.len());
                    for item in items {
                        match item {
                            VMValue::Str(s) => parts.push(s),
                            other => {
                                return Err(format!(
                                    "File.write_lines expects List<String>, got List<{}>",
                                    vmvalue_type_name(&other)
                                ));
                            }
                        }
                    }
                    parts
                }
                other => {
                    return Err(format!(
                        "File.write_lines expects List<String>, got {}",
                        vmvalue_type_name(&other)
                    ));
                }
            };
            std::fs::write(&path, lines.join("\n"))
                .map_err(|e| format!("File.write_lines failed for `{}`: {}", path, e))?;
            Ok(VMValue::Unit)
        }
        "File.append" => {
            use std::io::Write;
            if args.len() != 2 {
                return Err("File.append requires 2 arguments".to_string());
            }
            let mut it = args.into_iter();
            let path = match it.next().unwrap() {
                VMValue::Str(s) => s,
                other => {
                    return Err(format!(
                        "File.append expects String path, got {}",
                        vmvalue_type_name(&other)
                    ));
                }
            };
            let content = match it.next().unwrap() {
                VMValue::Str(s) => s,
                other => {
                    return Err(format!(
                        "File.append expects String content, got {}",
                        vmvalue_type_name(&other)
                    ));
                }
            };
            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path)
                .map_err(|e| format!("File.append failed for `{}`: {}", path, e))?;
            file.write_all(content.as_bytes())
                .map_err(|e| format!("File.append failed for `{}`: {}", path, e))?;
            Ok(VMValue::Unit)
        }
        "File.exists" => {
            let path = match args.into_iter().next() {
                Some(VMValue::Str(s)) => s,
                Some(other) => {
                    return Err(format!(
                        "File.exists expects String path, got {}",
                        vmvalue_type_name(&other)
                    ));
                }
                None => return Err("File.exists requires 1 argument".to_string()),
            };
            Ok(VMValue::Bool(std::path::Path::new(&path).exists()))
        }
        "File.delete" => {
            let path = match args.into_iter().next() {
                Some(VMValue::Str(s)) => s,
                Some(other) => {
                    return Err(format!(
                        "File.delete expects String path, got {}",
                        vmvalue_type_name(&other)
                    ));
                }
                None => return Err("File.delete requires 1 argument".to_string()),
            };
            match std::fs::remove_file(&path) {
                Ok(_) => Ok(VMValue::Unit),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(VMValue::Unit),
                Err(e) => Err(format!("File.delete failed for `{}`: {}", path, e)),
            }
        }
        // Task builtins (v1.7.0) — synchronous-only implementation
        // Task<T> is transparent at runtime: the value IS the T.
        "Task.run" => {
            // Task.run(t) — returns the task value immediately
            match args.into_iter().next() {
                Some(v) => Ok(v),
                None => Err("Task.run requires 1 argument".to_string()),
            }
        }
        "Task.map" => {
            // Task.map(task_val, f) — f(task_val)
            let mut it = args.into_iter();
            match (it.next(), it.next()) {
                (Some(val), Some(f)) => {
                    match f {
                        VMValue::CompiledFn(_) | VMValue::Closure(_, _) => Err(
                            "Task.map: function calling not supported in builtin context"
                                .to_string(),
                        ),
                        _ => Ok(val), // identity if f is not callable here
                    }
                }
                _ => Err("Task.map requires 2 arguments".to_string()),
            }
        }
        "Task.and_then" => {
            // Task.and_then(task_val, f) — same as Task.map for synchronous tasks
            match args.into_iter().next() {
                Some(v) => Ok(v),
                None => Err("Task.and_then requires 2 arguments".to_string()),
            }
        }
        // Task parallel API (v1.8.0) — synchronous transparent implementation
        "Task.all" => {
            // Task.all(list_of_tasks) — runs all, returns List of results
            // v1.8.0: Tasks are transparent values, so this is identity on the list.
            match args.into_iter().next() {
                Some(VMValue::List(items)) => {
                    if items.is_empty() {
                        return Err("E061: Task.all requires a non-empty list".to_string());
                    }
                    Ok(VMValue::List(items))
                }
                Some(other) => Err(format!("Task.all: expected List, got {:?}", other)),
                None => Err("Task.all requires 1 argument (a List of tasks)".to_string()),
            }
        }
        "Task.race" => {
            // Task.race(list_of_tasks) — returns the first task's result
            // v1.8.0: returns head element (no true parallelism).
            match args.into_iter().next() {
                Some(VMValue::List(items)) => {
                    if items.is_empty() {
                        return Err("E061: Task.race requires a non-empty list".to_string());
                    }
                    Ok(items.first().cloned().unwrap())
                }
                Some(other) => Err(format!("Task.race: expected List, got {:?}", other)),
                None => Err("Task.race requires 1 argument (a List of tasks)".to_string()),
            }
        }
        "Task.timeout" => {
            // Task.timeout(task, ms) — v1.8.0: always Some(value), no real timeout.
            let mut it = args.into_iter();
            match (it.next(), it.next()) {
                (Some(val), Some(VMValue::Int(_ms))) => {
                    Ok(VMValue::Variant("some".into(), Some(Box::new(val))))
                }
                (Some(val), None) => Ok(VMValue::Variant("some".into(), Some(Box::new(val)))),
                _ => {
                    Err("Task.timeout requires 2 arguments: task and timeout_ms (Int)".to_string())
                }
            }
        }
        // Random builtins (v2.8.0) — updated v3.5.0 to support seeded RNG
        "Random.int" => {
            let mut it = args.into_iter();
            let min_val = it
                .next()
                .ok_or_else(|| "Random.int requires 2 arguments".to_string())?;
            let max_val = it
                .next()
                .ok_or_else(|| "Random.int requires 2 arguments".to_string())?;
            match (min_val, max_val) {
                (VMValue::Int(lo), VMValue::Int(hi)) => Ok(VMValue::Int(seeded_rand_int(lo, hi))),
                _ => Err("Random.int requires (Int, Int)".to_string()),
            }
        }
        "Random.float" => Ok(VMValue::Float(seeded_rand_float())),
        // Random.seed (v3.5.0)
        "Random.seed" => {
            let n = vm_int(
                args.into_iter()
                    .next()
                    .ok_or_else(|| "Random.seed requires 1 argument".to_string())?,
                "Random.seed",
            )?;
            use rand::SeedableRng;
            SEEDED_RNG.with(|r| {
                *r.borrow_mut() = Some(rand::rngs::SmallRng::seed_from_u64(n as u64));
            });
            hint_reset_counters();
            Ok(VMValue::Unit)
        }

        // ── Gen.* (v3.5.0) ─────────────────────────────────────────────────
        "Gen.string_val" => {
            let len = vm_int(
                args.into_iter()
                    .next()
                    .ok_or_else(|| "Gen.string_val requires 1 argument".to_string())?,
                "Gen.string_val",
            )? as usize;
            Ok(VMValue::Str(random_alphanumeric_string(len)))
        }
        "Gen.one_raw" => {
            let type_name = vm_string(
                args.into_iter()
                    .next()
                    .ok_or_else(|| "Gen.one_raw requires 1 argument".to_string())?,
                "Gen.one_raw",
            )?;
            gen_one_row(&type_name, type_metas)
        }
        "Gen.list_raw" => {
            let mut it = args.into_iter();
            let type_name = vm_string(
                it.next()
                    .ok_or_else(|| "Gen.list_raw requires 2 arguments".to_string())?,
                "Gen.list_raw",
            )?;
            let n = vm_int(
                it.next()
                    .ok_or_else(|| "Gen.list_raw requires 2 arguments".to_string())?,
                "Gen.list_raw",
            )? as usize;
            let rows: Result<Vec<VMValue>, String> = (0..n)
                .map(|_| gen_one_row(&type_name, type_metas))
                .collect();
            Ok(VMValue::List(FavList::new(rows?)))
        }
        "Gen.simulate_raw" => {
            let mut it = args.into_iter();
            let type_name = vm_string(
                it.next()
                    .ok_or_else(|| "Gen.simulate_raw requires 3 arguments".to_string())?,
                "Gen.simulate_raw",
            )?;
            let n = vm_int(
                it.next()
                    .ok_or_else(|| "Gen.simulate_raw requires 3 arguments".to_string())?,
                "Gen.simulate_raw",
            )? as usize;
            let noise = vm_float(
                it.next()
                    .ok_or_else(|| "Gen.simulate_raw requires 3 arguments".to_string())?,
                "Gen.simulate_raw",
            )?;
            let meta = type_metas
                .get(&type_name)
                .ok_or_else(|| format!("Gen.simulate_raw: unknown type '{type_name}'"))?;
            let noise_thresh = (noise * 1000.0) as i64;
            let rows: Result<Vec<VMValue>, String> = (0..n)
                .map(|_| {
                    let mut map = HashMap::new();
                    for field in &meta.fields {
                        let corrupt = seeded_rand_int(0, 999) < noise_thresh;
                        let val = if corrupt {
                            gen_corrupt_value(&field.ty)
                        } else {
                            gen_value_for_type(&field.ty)
                        };
                        map.insert(field.name.clone(), VMValue::Str(val));
                    }
                    Ok(VMValue::Record(map))
                })
                .collect();
            Ok(VMValue::List(FavList::new(rows?)))
        }
        "Gen.profile_raw" => {
            let mut it = args.into_iter();
            let type_name = vm_string(
                it.next()
                    .ok_or_else(|| "Gen.profile_raw requires 2 arguments".to_string())?,
                "Gen.profile_raw",
            )?;
            let data_val = it
                .next()
                .ok_or_else(|| "Gen.profile_raw requires 2 arguments".to_string())?;
            let rows = match data_val {
                VMValue::List(rows) => rows,
                _ => return Err("Gen.profile_raw: second argument must be a list".to_string()),
            };
            let meta = type_metas
                .get(&type_name)
                .ok_or_else(|| format!("Gen.profile_raw: unknown type '{type_name}'"))?;
            let total = rows.len();
            let valid = rows
                .iter()
                .filter(|row| {
                    if let VMValue::Record(map) = row {
                        meta.fields.iter().all(|field| {
                            let val = map
                                .get(&field.name)
                                .and_then(|v| {
                                    if let VMValue::Str(s) = v {
                                        Some(s.as_str())
                                    } else {
                                        None
                                    }
                                })
                                .unwrap_or("");
                            is_valid_for_type(val, &field.ty)
                        })
                    } else {
                        false
                    }
                })
                .count();
            let invalid = total - valid;
            let rate = if total > 0 {
                valid as f64 / total as f64
            } else {
                0.0
            };
            let mut profile_map = HashMap::new();
            profile_map.insert("total".to_string(), VMValue::Int(total as i64));
            profile_map.insert("valid".to_string(), VMValue::Int(valid as i64));
            profile_map.insert("invalid".to_string(), VMValue::Int(invalid as i64));
            profile_map.insert("rate".to_string(), VMValue::Float(rate));
            Ok(VMValue::Record(profile_map))
        }

        // ── Gen.* v4.4.0 additions ──────────────────────────────────────────
        "Gen.hint_one_raw" => {
            let type_name = vm_string(
                args.into_iter()
                    .next()
                    .ok_or_else(|| "Gen.hint_one_raw requires 1 argument".to_string())?,
                "Gen.hint_one_raw",
            )?;
            match gen_hint_one_row(&type_name, type_metas) {
                Ok(row) => Ok(ok_vm(row)),
                Err(e) => Ok(err_vm(VMValue::Str(e))),
            }
        }
        "Gen.hint_list_raw" => {
            let mut it = args.into_iter();
            let type_name = vm_string(
                it.next()
                    .ok_or_else(|| "Gen.hint_list_raw requires 2 arguments".to_string())?,
                "Gen.hint_list_raw",
            )?;
            let n = vm_int(
                it.next()
                    .ok_or_else(|| "Gen.hint_list_raw requires 2 arguments".to_string())?,
                "Gen.hint_list_raw",
            )? as usize;
            let rows: Result<Vec<VMValue>, String> = (0..n)
                .map(|_| gen_hint_one_row(&type_name, type_metas))
                .collect();
            match rows {
                Ok(list) => Ok(ok_vm(VMValue::List(FavList::new(list)))),
                Err(e) => Ok(err_vm(VMValue::Str(e))),
            }
        }
        "Gen.set_yaml_config_raw" => {
            let mut it = args.into_iter();
            let type_name = vm_string(
                it.next()
                    .ok_or_else(|| "Gen.set_yaml_config_raw requires 2 arguments".to_string())?,
                "Gen.set_yaml_config_raw",
            )?;
            let path_val = vm_string(
                it.next()
                    .ok_or_else(|| "Gen.set_yaml_config_raw requires 2 arguments".to_string())?,
                "Gen.set_yaml_config_raw",
            )?;
            let content = std::fs::read_to_string(&path_val).map_err(|e| {
                format!("Gen.set_yaml_config_raw: cannot read '{}': {}", path_val, e)
            })?;
            // Parse YAML: expect top-level map of field_name -> config
            let yaml: serde_yaml::Value = serde_yaml::from_str(&content)
                .map_err(|e| format!("Gen.set_yaml_config_raw: invalid YAML: {}", e))?;
            let mut cfg = GenYamlConfig::default();
            if let serde_yaml::Value::Mapping(top) = &yaml {
                if let Some(fields_val) = top.get("fields") {
                    if let serde_yaml::Value::Mapping(fields) = fields_val {
                        for (k, v) in fields {
                            let field_name = k.as_str().unwrap_or("").to_string();
                            let mut fc = GenFieldConfig::default();
                            if let serde_yaml::Value::Mapping(m) = v {
                                if let Some(serde_yaml::Value::Sequence(vals)) = m.get("values") {
                                    fc.values = vals
                                        .iter()
                                        .filter_map(|x| x.as_str().map(|s| s.to_string()))
                                        .collect();
                                }
                                if let Some(mn) = m.get("min") {
                                    fc.min = mn.as_f64();
                                }
                                if let Some(mx) = m.get("max") {
                                    fc.max = mx.as_f64();
                                }
                                if let Some(nr) = m.get("null_rate") {
                                    fc.null_rate = nr.as_f64().unwrap_or(0.0);
                                }
                            }
                            cfg.fields.insert(field_name, fc);
                        }
                    }
                }
            }
            GEN_YAML_CONFIG.with(|c| c.borrow_mut().insert(type_name, cfg));
            Ok(VMValue::Unit)
        }
        "Gen.to_csv_raw" => {
            let mut it = args.into_iter();
            let path_val = vm_string(
                it.next()
                    .ok_or_else(|| "Gen.to_csv_raw requires 2 arguments".to_string())?,
                "Gen.to_csv_raw",
            )?;
            let data_val = it
                .next()
                .ok_or_else(|| "Gen.to_csv_raw requires 2 arguments".to_string())?;
            let rows = match data_val {
                VMValue::List(rows) => rows,
                _ => {
                    return Ok(err_vm(VMValue::Str(
                        "Gen.to_csv_raw: second argument must be a list".to_string(),
                    )));
                }
            };
            // Collect headers from first row
            let headers: Vec<String> = if let Some(VMValue::Record(first)) = rows.first() {
                let mut keys: Vec<String> = first.keys().cloned().collect();
                keys.sort();
                keys
            } else {
                vec![]
            };
            let mut wtr = csv::Writer::from_path(&path_val)
                .map_err(|e| format!("Gen.to_csv_raw: cannot open '{}': {}", path_val, e))?;
            wtr.write_record(&headers)
                .map_err(|e| format!("Gen.to_csv_raw: write error: {}", e))?;
            for row in rows.iter() {
                if let VMValue::Record(map) = row {
                    let record: Vec<String> = headers
                        .iter()
                        .map(|h| match map.get(h) {
                            Some(VMValue::Str(s)) => s.clone(),
                            Some(v) => format!("{:?}", v),
                            None => String::new(),
                        })
                        .collect();
                    wtr.write_record(&record)
                        .map_err(|e| format!("Gen.to_csv_raw: write error: {}", e))?;
                }
            }
            wtr.flush()
                .map_err(|e| format!("Gen.to_csv_raw: flush error: {}", e))?;
            Ok(ok_vm(VMValue::Unit))
        }
        // ── v19.5.0: ArrowBatch primitives ───────────────────────────────────────
        // ── v19.5.0: ArrowBatch primitives ───────────────────────────────────────
        "ArrowBatch.from_list" => {
            let rows = match args.into_iter().next() {
                Some(VMValue::List(list)) => list.to_vec(),
                other => return Err(format!(
                    "ArrowBatch.from_list expects List, got {:?}",
                    other.as_ref().map(vmvalue_type_name)
                )),
            };
            match arrow_from_vm_rows(&rows) {
                Ok(batch) => Ok(ok_vm(VMValue::ArrowBatch(arrow_store(batch)))),
                Err(e) => Ok(err_vm(VMValue::Str(e))),
            }
        }

        "ArrowBatch.to_list" => {
            let id = match args.into_iter().next() {
                Some(VMValue::ArrowBatch(id)) => id,
                other => return Err(format!(
                    "ArrowBatch.to_list expects ArrowBatch, got {:?}",
                    other.as_ref().map(vmvalue_type_name)
                )),
            };
            match arrow_get(id) {
                Some(batch) => {
                    let rows = arrow_to_vm_rows(&batch)?;
                    Ok(ok_vm(VMValue::List(FavList::new(rows))))
                }
                None => Ok(err_vm(VMValue::Str(format!("ArrowBatch: invalid handle {id}")))),
            }
        }

        "ArrowBatch.write_parquet" => {
            let mut it = args.into_iter();
            let id = match it.next() {
                Some(VMValue::ArrowBatch(id)) => id,
                other => return Err(format!(
                    "ArrowBatch.write_parquet: expected ArrowBatch, got {:?}",
                    other.as_ref().map(vmvalue_type_name)
                )),
            };
            let path = match it.next() {
                Some(VMValue::Str(s)) => s,
                other => return Err(format!(
                    "ArrowBatch.write_parquet: expected String path, got {:?}",
                    other.as_ref().map(vmvalue_type_name)
                )),
            };
            match arrow_get(id) {
                Some(batch) => match arrow_write_parquet(&batch, &path) {
                    Ok(_) => Ok(ok_vm(VMValue::Unit)),
                    Err(e) => Ok(err_vm(VMValue::Str(e))),
                },
                None => Ok(err_vm(VMValue::Str(format!("ArrowBatch.write_parquet: invalid handle {id}")))),
            }
        }

        "ArrowBatch.read_parquet" => {
            let path = match args.into_iter().next() {
                Some(VMValue::Str(s)) => s,
                other => return Err(format!(
                    "ArrowBatch.read_parquet expects String, got {:?}",
                    other.as_ref().map(vmvalue_type_name)
                )),
            };
            match arrow_read_parquet(&path) {
                Ok(batch) => Ok(ok_vm(VMValue::ArrowBatch(arrow_store(batch)))),
                Err(e) => Ok(err_vm(VMValue::Str(e))),
            }
        }

        // ── v20.5.0: mmap + SIMD CSV ──────────────────────────────────────────
        "ArrowBatch.from_csv" => {
            let path = match args.into_iter().next() {
                Some(VMValue::Str(s)) => s,
                _ => return Err("ArrowBatch.from_csv: expected String path".to_string()),
            };
            // read_csv_mmap uses memmap2 and arrow-csv which are native-only dependencies.
            // On wasm32 we return a typed error without calling native code.
            #[cfg(not(target_arch = "wasm32"))]
            {
                match read_csv_mmap(&path) {
                    Ok(batch) => Ok(ok_vm(VMValue::ArrowBatch(arrow_store(batch)))),
                    Err(e) => Ok(err_vm(VMValue::Str(e))),
                }
            }
            #[cfg(target_arch = "wasm32")]
            {
                let _ = path;
                Ok(err_vm(VMValue::Str(
                    "ArrowBatch.from_csv: not supported on wasm32".to_string(),
                )))
            }
        }

        "Gen.to_parquet_raw" => {
            let mut it = args.into_iter();
            let path_val = vm_string(
                it.next()
                    .ok_or_else(|| "Gen.to_parquet_raw requires 2 arguments".to_string())?,
                "Gen.to_parquet_raw",
            )?;
            let data_val = it
                .next()
                .ok_or_else(|| "Gen.to_parquet_raw requires 2 arguments".to_string())?;
            let rows = match data_val {
                VMValue::List(rows) => rows,
                _ => {
                    return Ok(err_vm(VMValue::Str(
                        "Gen.to_parquet_raw: second argument must be a list".to_string(),
                    )));
                }
            };
            // Delegate to the existing Parquet.write_raw logic by collecting string columns
            use arrow::array::{ArrayRef, StringArray};
            use arrow::datatypes::{DataType, Field, Schema};
            use arrow::record_batch::RecordBatch;
            use parquet::arrow::ArrowWriter;
            use std::sync::Arc;
            let headers: Vec<String> = if let Some(VMValue::Record(first)) = rows.first() {
                let mut keys: Vec<String> = first.keys().cloned().collect();
                keys.sort();
                keys
            } else {
                vec![]
            };
            let schema = Arc::new(Schema::new(
                headers
                    .iter()
                    .map(|h| Field::new(h.as_str(), DataType::Utf8, true))
                    .collect::<Vec<_>>(),
            ));
            let columns: Vec<ArrayRef> = headers
                .iter()
                .map(|h| {
                    let vals: Vec<Option<&str>> = rows
                        .iter()
                        .map(|row| {
                            if let VMValue::Record(map) = row {
                                map.get(h).and_then(|v| {
                                    if let VMValue::Str(s) = v {
                                        Some(s.as_str())
                                    } else {
                                        None
                                    }
                                })
                            } else {
                                None
                            }
                        })
                        .collect();
                    Arc::new(StringArray::from(vals)) as ArrayRef
                })
                .collect();
            let batch = RecordBatch::try_new(schema.clone(), columns)
                .map_err(|e| format!("Gen.to_parquet_raw: arrow error: {}", e))?;
            let file = std::fs::File::create(&path_val)
                .map_err(|e| format!("Gen.to_parquet_raw: cannot create '{}': {}", path_val, e))?;
            let mut writer = ArrowWriter::try_new(file, schema, None)
                .map_err(|e| format!("Gen.to_parquet_raw: parquet writer error: {}", e))?;
            writer
                .write(&batch)
                .map_err(|e| format!("Gen.to_parquet_raw: write error: {}", e))?;
            writer
                .close()
                .map_err(|e| format!("Gen.to_parquet_raw: close error: {}", e))?;
            Ok(ok_vm(VMValue::Unit))
        }
        "Gen.load_into_raw" => {
            // Gen.load_into_raw(handle, table_name, rows) — load generated data into DuckDB
            let mut it = args.into_iter();
            let handle = match it
                .next()
                .ok_or_else(|| "Gen.load_into_raw requires 3 arguments".to_string())?
            {
                VMValue::DbHandle(id) => id,
                VMValue::Int(n) => n as u64,
                other => {
                    return Err(format!(
                        "Gen.load_into_raw: first argument must be DbHandle or Int, got {}",
                        vmvalue_type_name(&other)
                    ));
                }
            };
            let table_name = vm_string(
                it.next()
                    .ok_or_else(|| "Gen.load_into_raw requires 3 arguments".to_string())?,
                "Gen.load_into_raw",
            )?;
            let data_val = it
                .next()
                .ok_or_else(|| "Gen.load_into_raw requires 3 arguments".to_string())?;
            let rows = match data_val {
                VMValue::List(rows) => rows,
                _ => {
                    return Ok(err_vm(VMValue::Str(
                        "Gen.load_into_raw: third argument must be a list".to_string(),
                    )));
                }
            };
            if rows.is_empty() {
                return Ok(ok_vm(VMValue::Int(0)));
            }
            let headers: Vec<String> = if let Some(VMValue::Record(first)) = rows.first() {
                let mut keys: Vec<String> = first.keys().cloned().collect();
                keys.sort();
                keys
            } else {
                vec![]
            };
            let inserted: Result<i64, String> = (|| {
                let mut store = duckdb_store();
                let conn = store
                    .get_mut(&handle)
                    .ok_or_else(|| format!("Gen.load_into_raw: invalid handle {handle}"))?;
                // Create table if not exists (all TEXT columns)
                let col_defs: String = headers
                    .iter()
                    .map(|h| format!("{h} TEXT"))
                    .collect::<Vec<_>>()
                    .join(", ");
                conn.execute_batch(&format!(
                    "CREATE TABLE IF NOT EXISTS {table_name} ({col_defs})"
                ))
                .map_err(|e| format!("Gen.load_into_raw: create table error: {}", e))?;
                let placeholders: String =
                    headers.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
                let sql = format!("INSERT INTO {table_name} VALUES ({placeholders})");
                let mut stmt = conn
                    .prepare(&sql)
                    .map_err(|e| format!("Gen.load_into_raw: prepare error: {}", e))?;
                let mut count = 0i64;
                for row in rows.iter() {
                    if let VMValue::Record(record_map) = row {
                        let vals: Vec<duckdb::types::Value> = headers
                            .iter()
                            .map(|h| {
                                let s = match record_map.get(h) {
                                    Some(VMValue::Str(s)) => s.clone(),
                                    Some(v) => format!("{:?}", v),
                                    None => String::new(),
                                };
                                duckdb::types::Value::Text(s)
                            })
                            .collect();
                        let params = duckdb::params_from_iter(vals.iter());
                        stmt.execute(params)
                            .map_err(|e| format!("Gen.load_into_raw: insert error: {}", e))?;
                        count += 1;
                    }
                }
                Ok(count)
            })();
            match inserted {
                Ok(n) => Ok(ok_vm(VMValue::Int(n))),
                Err(e) => Ok(err_vm(VMValue::Str(e))),
            }
        }
        "Gen.edge_cases_raw" => {
            let type_name = vm_string(
                args.into_iter()
                    .next()
                    .ok_or_else(|| "Gen.edge_cases_raw requires 1 argument".to_string())?,
                "Gen.edge_cases_raw",
            )?;
            let meta = match type_metas.get(&type_name) {
                Some(m) => m,
                None => {
                    return Ok(err_vm(VMValue::Str(format!(
                        "Gen.edge_cases_raw: unknown type '{type_name}'"
                    ))));
                }
            };
            // Generate boundary rows: all-min, all-max, all-empty, all-null-like
            let mut results: Vec<VMValue> = Vec::new();
            // Row 1: minimum values
            {
                let mut map = HashMap::new();
                for field in &meta.fields {
                    let val = match field.ty.as_str() {
                        "Int" => "0".to_string(),
                        "Float" => "0.0".to_string(),
                        "Bool" => "false".to_string(),
                        _ => String::new(),
                    };
                    map.insert(field.name.clone(), VMValue::Str(val));
                }
                results.push(VMValue::Record(map));
            }
            // Row 2: maximum values
            {
                let mut map = HashMap::new();
                for field in &meta.fields {
                    let val = match field.ty.as_str() {
                        "Int" => i64::MAX.to_string(),
                        "Float" => f64::MAX.to_string(),
                        "Bool" => "true".to_string(),
                        _ => "z".repeat(255),
                    };
                    map.insert(field.name.clone(), VMValue::Str(val));
                }
                results.push(VMValue::Record(map));
            }
            // Row 3: empty string values
            {
                let mut map = HashMap::new();
                for field in &meta.fields {
                    map.insert(field.name.clone(), VMValue::Str(String::new()));
                }
                results.push(VMValue::Record(map));
            }
            // Row 4: whitespace values
            {
                let mut map = HashMap::new();
                for field in &meta.fields {
                    map.insert(field.name.clone(), VMValue::Str("   ".to_string()));
                }
                results.push(VMValue::Record(map));
            }
            Ok(ok_vm(VMValue::List(FavList::new(results))))
        }

        // ── Gen.* v9.4.0 additions — UUID / nano_id ──────────────────────────
        "Gen.uuid_raw" => Ok(VMValue::Str(uuid::Uuid::new_v4().to_string())),
        "Gen.uuid_v7_raw" => Ok(VMValue::Str(uuid::Uuid::now_v7().to_string())),
        "Gen.nano_id_raw" => {
            let n = vm_int(
                args.into_iter()
                    .next()
                    .ok_or_else(|| "Gen.nano_id_raw requires 1 argument".to_string())?,
                "Gen.nano_id_raw",
            )? as usize;
            const ALPHABET: &[u8] =
                b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789_-";
            use rand::Rng;
            let mut rng = rand::thread_rng();
            let id: String = (0..n)
                .map(|_| ALPHABET[rng.gen_range(0..ALPHABET.len())] as char)
                .collect();
            Ok(VMValue::Str(id))
        }

        // ── Crypto.* (v4.5.0) — cryptographic primitives ──────────────────
        "Crypto.jwt_verify_raw" => {
            let mut it = args.into_iter();
            let token = vm_string(
                it.next()
                    .ok_or_else(|| "Crypto.jwt_verify_raw requires 3 arguments".to_string())?,
                "Crypto.jwt_verify_raw",
            )?;
            let secret = vm_string(
                it.next()
                    .ok_or_else(|| "Crypto.jwt_verify_raw requires 3 arguments".to_string())?,
                "Crypto.jwt_verify_raw",
            )?;
            let alg_str = vm_string(
                it.next()
                    .ok_or_else(|| "Crypto.jwt_verify_raw requires 3 arguments".to_string())?,
                "Crypto.jwt_verify_raw",
            )?;
            let result: Result<HashMap<String, VMValue>, String> = (|| match alg_str.as_str() {
                "HS256" => {
                    let key = DecodingKey::from_secret(secret.as_bytes());
                    let validation = Validation::new(Algorithm::HS256);
                    let data = decode::<SerdeJsonValue>(&token, &key, &validation)
                        .map_err(|e| format!("jwt verify failed: {}", e))?;
                    Ok(json_value_to_vm_claims_map(&data.claims))
                }
                "RS256" => {
                    let key = DecodingKey::from_rsa_pem(secret.as_bytes())
                        .map_err(|e| format!("invalid RSA PEM: {}", e))?;
                    let validation = Validation::new(Algorithm::RS256);
                    let data = decode::<SerdeJsonValue>(&token, &key, &validation)
                        .map_err(|e| format!("jwt verify failed: {}", e))?;
                    Ok(json_value_to_vm_claims_map(&data.claims))
                }
                other => Err(format!("unsupported algorithm: {}", other)),
            })();
            match result {
                Ok(map) => Ok(ok_vm(VMValue::Record(map))),
                Err(e) => Ok(err_vm(VMValue::Str(e))),
            }
        }

        "Crypto.jwt_decode_raw" => {
            let token = vm_string(
                args.into_iter()
                    .next()
                    .ok_or_else(|| "Crypto.jwt_decode_raw requires 1 argument".to_string())?,
                "Crypto.jwt_decode_raw",
            )?;
            // Decode without signature verification (v9 API)
            let mut validation = Validation::new(Algorithm::HS256);
            validation.insecure_disable_signature_validation();
            validation.validate_exp = false;
            let dummy_key = DecodingKey::from_secret(b"");
            match decode::<SerdeJsonValue>(&token, &dummy_key, &validation) {
                Ok(data) => Ok(ok_vm(VMValue::Record(json_value_to_vm_claims_map(
                    &data.claims,
                )))),
                Err(e) => Ok(err_vm(VMValue::Str(format!("jwt decode failed: {}", e)))),
            }
        }

        "Crypto.jwt_sign_raw" => {
            let mut it = args.into_iter();
            let claims_json = vm_string(
                it.next()
                    .ok_or_else(|| "Crypto.jwt_sign_raw requires 3 arguments".to_string())?,
                "Crypto.jwt_sign_raw",
            )?;
            let secret = vm_string(
                it.next()
                    .ok_or_else(|| "Crypto.jwt_sign_raw requires 3 arguments".to_string())?,
                "Crypto.jwt_sign_raw",
            )?;
            let _alg = vm_string(
                it.next()
                    .ok_or_else(|| "Crypto.jwt_sign_raw requires 3 arguments".to_string())?,
                "Crypto.jwt_sign_raw",
            )?;
            let claims: SerdeJsonValue = match serde_json::from_str(&claims_json) {
                Ok(v) => v,
                Err(e) => return Ok(err_vm(VMValue::Str(format!("invalid claims JSON: {}", e)))),
            };
            let header = Header::new(Algorithm::HS256);
            let key = EncodingKey::from_secret(secret.as_bytes());
            match encode(&header, &claims, &key) {
                Ok(token) => Ok(ok_vm(VMValue::Str(token))),
                Err(e) => Ok(err_vm(VMValue::Str(format!("jwt sign failed: {}", e)))),
            }
        }

        "Crypto.hmac_sha256_raw" => {
            let mut it = args.into_iter();
            let key = vm_string(
                it.next()
                    .ok_or_else(|| "Crypto.hmac_sha256_raw requires 2 arguments".to_string())?,
                "Crypto.hmac_sha256_raw",
            )?;
            let data = vm_string(
                it.next()
                    .ok_or_else(|| "Crypto.hmac_sha256_raw requires 2 arguments".to_string())?,
                "Crypto.hmac_sha256_raw",
            )?;
            let mut mac = HmacSha256::new_from_slice(key.as_bytes())
                .map_err(|e| format!("Crypto.hmac_sha256_raw: {}", e))?;
            mac.update(data.as_bytes());
            let result = mac.finalize().into_bytes();
            let hex: String = result.iter().map(|b| format!("{:02x}", b)).collect();
            Ok(VMValue::Str(hex))
        }

        "Crypto.sha256_raw" => {
            let data = vm_string(
                args.into_iter()
                    .next()
                    .ok_or_else(|| "Crypto.sha256_raw requires 1 argument".to_string())?,
                "Crypto.sha256_raw",
            )?;
            let mut hasher = Sha256::new();
            hasher.update(data.as_bytes());
            let result = hasher.finalize();
            let hex: String = result.iter().map(|b| format!("{:02x}", b)).collect();
            Ok(VMValue::Str(hex))
        }

        "Crypto.random_hex_raw" => {
            let n = match args.into_iter().next() {
                Some(VMValue::Int(n)) => n as usize,
                Some(other) => {
                    return Err(format!(
                        "Crypto.random_hex_raw: expected Int, got {}",
                        vmvalue_type_name(&other)
                    ));
                }
                None => return Err("Crypto.random_hex_raw requires 1 argument".to_string()),
            };
            let mut bytes = vec![0u8; n];
            rand::rngs::OsRng.fill_bytes(&mut bytes);
            let hex: String = bytes.iter().map(|b| format!("{:02x}", b)).collect();
            Ok(VMValue::Str(hex))
        }

        // ── Crypto.ecdsa_verify_raw (v15.1.5) ───────────────────────────────
        // Args: (pub_key_pem: String, message: String, sig_der_b64: String)
        // Returns: Result<Unit, String>  (!Auth effect)
        "Crypto.ecdsa_verify_raw" => {
            use base64::Engine;
            use p256::ecdsa::{Signature, VerifyingKey};
            use p256::ecdsa::signature::Verifier as _;
            use p256::pkcs8::DecodePublicKey as _;

            let mut it = args.into_iter();
            let pem = vm_string(
                it.next().ok_or("Crypto.ecdsa_verify_raw: missing pub_key_pem")?,
                "Crypto.ecdsa_verify_raw",
            )?;
            let message = vm_string(
                it.next().ok_or("Crypto.ecdsa_verify_raw: missing message")?,
                "Crypto.ecdsa_verify_raw",
            )?;
            let sig_b64 = vm_string(
                it.next().ok_or("Crypto.ecdsa_verify_raw: missing sig_der_b64")?,
                "Crypto.ecdsa_verify_raw",
            )?;

            let sig_bytes = BASE64.decode(sig_b64.trim())
                .map_err(|e| format!("Crypto.ecdsa_verify_raw: base64 decode: {e}"))?;

            let verifying_key = match VerifyingKey::from_public_key_pem(pem.trim()) {
                Ok(k) => k,
                Err(_) => return Ok(VMValue::Variant(
                    "err".into(),
                    Some(Box::new(VMValue::Str("ecdsa_verify_failed".into()))),
                )),
            };

            let sig = match Signature::from_der(&sig_bytes) {
                Ok(s) => s,
                Err(_) => return Ok(VMValue::Variant(
                    "err".into(),
                    Some(Box::new(VMValue::Str("ecdsa_verify_failed".into()))),
                )),
            };

            match verifying_key.verify(message.as_bytes(), &sig) {
                Ok(()) => Ok(VMValue::Variant("ok".into(), Some(Box::new(VMValue::Unit)))),
                Err(_) => Ok(VMValue::Variant(
                    "err".into(),
                    Some(Box::new(VMValue::Str("ecdsa_verify_failed".into()))),
                )),
            }
        }

        // ── Auth.* (v4.5.0) — auth config helpers ──────────────────────────
        "Auth.get_mode_raw" => Ok(VMValue::Str(AUTH_MODE.with(|m| m.borrow().clone()))),

        // ── Log.* (v4.6.0) — structured logging + metrics ──────────────────
        "Log.emit_raw" => {
            // args: (level, code, message, ctx_json)
            let mut it = args.into_iter();
            let level = vm_string(
                it.next().ok_or("Log.emit_raw: missing level")?,
                "Log.emit_raw",
            )?;
            let code = vm_string(
                it.next().ok_or("Log.emit_raw: missing code")?,
                "Log.emit_raw",
            )?;
            let message = vm_string(
                it.next().ok_or("Log.emit_raw: missing message")?,
                "Log.emit_raw",
            )?;
            let ctx_json = vm_string(
                it.next().ok_or("Log.emit_raw: missing ctx")?,
                "Log.emit_raw",
            )?;
            if log_level_passes(&level) {
                LOG_CONFIG.with(|c| {
                    let cfg = c.borrow();
                    let line = if cfg.format == "json" {
                        log_format_json(&level, &code, &message, &ctx_json, &cfg.service)
                    } else {
                        log_format_text(&level, &code, &message, &ctx_json)
                    };
                    if cfg.output == "stderr" {
                        eprintln!("{}", line);
                    } else {
                        println!("{}", line);
                    }
                });
            }
            Ok(VMValue::Unit)
        }

        "Log.metric_raw" => {
            // args: (name, value: Int, unit)
            let mut it = args.into_iter();
            let name = vm_string(
                it.next().ok_or("Log.metric_raw: missing name")?,
                "Log.metric_raw",
            )?;
            let value = match it.next().ok_or("Log.metric_raw: missing value")? {
                VMValue::Int(n) => n,
                other => return Err(format!("Log.metric_raw: expected Int, got {:?}", other)),
            };
            let unit = vm_string(
                it.next().ok_or("Log.metric_raw: missing unit")?,
                "Log.metric_raw",
            )?;
            LOG_CONFIG.with(|c| {
                let cfg = c.borrow();
                let line = if cfg.format == "json" {
                    log_metric_emf(&name, value, &unit)
                } else {
                    let ts = log_timestamp_text();
                    format!("{} {:<7} {}={} {}", ts, "METRIC", name, value, unit)
                };
                if cfg.output == "stderr" {
                    eprintln!("{}", line);
                } else {
                    println!("{}", line);
                }
            });
            Ok(VMValue::Unit)
        }

        "Log.map_to_json_raw" => {
            // args: (ctx: Map<String,String> as VMValue::Record)
            let ctx = match args.into_iter().next() {
                Some(VMValue::Record(map)) => map,
                Some(VMValue::Unit) => std::collections::HashMap::new(),
                _ => std::collections::HashMap::new(),
            };
            let mut obj = serde_json::Map::new();
            for (k, v) in &ctx {
                let val = match v {
                    VMValue::Str(s) => serde_json::Value::String(s.clone()),
                    VMValue::Int(n) => serde_json::Value::Number((*n).into()),
                    VMValue::Bool(b) => serde_json::Value::Bool(*b),
                    other => serde_json::Value::String(format!("{:?}", other)),
                };
                obj.insert(k.clone(), val);
            }
            Ok(VMValue::Str(serde_json::Value::Object(obj).to_string()))
        }

        // ── Env.* (v4.7.0) ─────────────────────────────────────────────────
        "Env.get_raw" => {
            let key = vm_string(
                args.into_iter().next().ok_or("Env.get_raw: missing key")?,
                "Env.get_raw",
            )?;
            let resolved = env_resolve_key(&key);
            match std::env::var(&resolved) {
                Ok(val) => Ok(VMValue::Variant(
                    "some".to_string(),
                    Some(Box::new(VMValue::Str(val))),
                )),
                Err(_) => Ok(VMValue::Variant("none".to_string(), None)),
            }
        }

        "Env.require_raw" => {
            let key = vm_string(
                args.into_iter()
                    .next()
                    .ok_or("Env.require_raw: missing key")?,
                "Env.require_raw",
            )?;
            let resolved = env_resolve_key(&key);
            match std::env::var(&resolved) {
                Ok(val) => Ok(ok_vm(VMValue::Str(val))),
                Err(_) => Ok(err_vm(VMValue::Str(format!("ENV_MISSING: {}", resolved)))),
            }
        }

        "Env.get_int_raw" => {
            let key = vm_string(
                args.into_iter()
                    .next()
                    .ok_or("Env.get_int_raw: missing key")?,
                "Env.get_int_raw",
            )?;
            let resolved = env_resolve_key(&key);
            match std::env::var(&resolved) {
                Err(_) => Ok(err_vm(VMValue::Str(format!("ENV_MISSING: {}", resolved)))),
                Ok(val) => match val.trim().parse::<i64>() {
                    Ok(n) => Ok(ok_vm(VMValue::Int(n))),
                    Err(_) => Ok(err_vm(VMValue::Str(format!(
                        "ENV_PARSE_INT: {}={}",
                        resolved, val
                    )))),
                },
            }
        }

        "Env.get_bool_raw" => {
            let key = vm_string(
                args.into_iter()
                    .next()
                    .ok_or("Env.get_bool_raw: missing key")?,
                "Env.get_bool_raw",
            )?;
            let resolved = env_resolve_key(&key);
            match std::env::var(&resolved) {
                Err(_) => Ok(err_vm(VMValue::Str(format!("ENV_MISSING: {}", resolved)))),
                Ok(val) => match val.trim().to_lowercase().as_str() {
                    "true" | "1" | "yes" | "on" => Ok(ok_vm(VMValue::Bool(true))),
                    "false" | "0" | "no" | "off" => Ok(ok_vm(VMValue::Bool(false))),
                    _ => Ok(err_vm(VMValue::Str(format!(
                        "ENV_PARSE_BOOL: {}={}",
                        resolved, val
                    )))),
                },
            }
        }

        "Env.load_dotenv_raw" => {
            let path = vm_string(
                args.into_iter()
                    .next()
                    .ok_or("Env.load_dotenv_raw: missing path")?,
                "Env.load_dotenv_raw",
            )?;
            match std::fs::read_to_string(&path) {
                Err(_) => Ok(err_vm(VMValue::Str(format!(
                    "ENV_DOTENV_NOT_FOUND: {}",
                    path
                )))),
                Ok(content) => {
                    for (key, val) in parse_dotenv_content(&content) {
                        if std::env::var(&key).is_err() {
                            // SAFETY: single-threaded VM context; no other threads read env at this point
                            unsafe {
                                std::env::set_var(&key, &val);
                            }
                        }
                    }
                    Ok(ok_vm(VMValue::Unit))
                }
            }
        }

        "Env.all_raw" => {
            let map: std::collections::BTreeMap<String, VMValue> = std::env::vars()
                .map(|(k, v)| (k, VMValue::Str(v)))
                .collect();
            let hash: std::collections::HashMap<String, VMValue> = map.into_iter().collect();
            Ok(VMValue::Record(hash))
        }

        // ── DB.* (v3.3.0) ──────────────────────────────────────────────────
        "DB.connect" => {
            if args.len() != 1 {
                return Err("DB.connect requires 1 argument".to_string());
            }
            let conn_str = vm_string(args.into_iter().next().unwrap(), "DB.connect")?;
            let conn = if conn_str == "sqlite::memory:" {
                rusqlite::Connection::open_in_memory()
                    .map_err(|e| format!("E0601: db connection failed: {}", e))?
            } else if let Some(path) = conn_str.strip_prefix("sqlite:") {
                rusqlite::Connection::open(path)
                    .map_err(|e| format!("E0601: db connection failed: {}", e))?
            } else if conn_str.starts_with("postgres://") {
                return Ok(err_vm(db_error_vm(
                    "E0605",
                    "db driver unsupported: postgres not compiled in (enable feature 'postgres_integration')",
                )));
            } else {
                return Ok(err_vm(db_error_vm(
                    "E0605",
                    &format!("db driver unsupported: unknown scheme in '{}'", conn_str),
                )));
            };
            let id = DB_NEXT_ID.with(|c| {
                let id = c.get();
                c.set(id + 1);
                id
            });
            DB_CONNECTIONS.with(|store| {
                store
                    .borrow_mut()
                    .insert(id, DbConnWrapper { conn, in_tx: false });
            });
            Ok(ok_vm(VMValue::DbHandle(id)))
        }

        "DB.close" => {
            if args.len() != 1 {
                return Err("DB.close requires 1 argument".to_string());
            }
            match args.into_iter().next().unwrap() {
                VMValue::DbHandle(id) => {
                    DB_CONNECTIONS.with(|store| {
                        store.borrow_mut().remove(&id);
                    });
                    Ok(VMValue::Unit)
                }
                other => Err(format!(
                    "DB.close expects DbHandle, got {}",
                    vmvalue_type_name(&other)
                )),
            }
        }

        "DB.query_raw" => {
            if args.len() != 2 {
                return Err("DB.query_raw requires 2 arguments".to_string());
            }
            let mut it = args.into_iter();
            let handle_id = match it.next().unwrap() {
                VMValue::DbHandle(id) => id,
                other => {
                    return Err(format!(
                        "DB.query_raw expects DbHandle, got {}",
                        vmvalue_type_name(&other)
                    ));
                }
            };
            let sql = vm_string(it.next().unwrap(), "DB.query_raw")?;
            let rows = DB_CONNECTIONS.with(|store| -> Result<Vec<VMValue>, String> {
                let store = store.borrow();
                let wrapper = store
                    .get(&handle_id)
                    .ok_or_else(|| "DB.query_raw: invalid DbHandle".to_string())?;
                sqlite_query_raw(&wrapper.conn, &sql)
            })?;
            Ok(ok_vm(VMValue::List(FavList::new(rows))))
        }

        "DB.execute_raw" => {
            if args.len() != 2 {
                return Err("DB.execute_raw requires 2 arguments".to_string());
            }
            let mut it = args.into_iter();
            let handle_id = match it.next().unwrap() {
                VMValue::DbHandle(id) => id,
                other => {
                    return Err(format!(
                        "DB.execute_raw expects DbHandle, got {}",
                        vmvalue_type_name(&other)
                    ));
                }
            };
            let sql = vm_string(it.next().unwrap(), "DB.execute_raw")?;
            let n = DB_CONNECTIONS.with(|store| -> Result<i64, String> {
                let store = store.borrow();
                let wrapper = store
                    .get(&handle_id)
                    .ok_or_else(|| "DB.execute_raw: invalid DbHandle".to_string())?;
                wrapper
                    .conn
                    .execute(&sql, [])
                    .map(|n| n as i64)
                    .map_err(|e| format!("E0602: db query failed: {}", e))
            })?;
            Ok(ok_vm(VMValue::Int(n)))
        }

        "DB.query_raw_params" => {
            if args.len() != 3 {
                return Err("DB.query_raw_params requires 3 arguments".to_string());
            }
            let mut it = args.into_iter();
            let handle_id = match it.next().unwrap() {
                VMValue::DbHandle(id) => id,
                other => {
                    return Err(format!(
                        "DB.query_raw_params expects DbHandle, got {}",
                        vmvalue_type_name(&other)
                    ));
                }
            };
            let sql = vm_string(it.next().unwrap(), "DB.query_raw_params")?;
            let params = match it.next().unwrap() {
                VMValue::List(v) => v
                    .into_iter()
                    .map(|p| vm_string(p, "DB.query_raw_params param"))
                    .collect::<Result<Vec<_>, _>>()?,
                other => {
                    return Err(format!(
                        "DB.query_raw_params: params must be List<String>, got {}",
                        vmvalue_type_name(&other)
                    ));
                }
            };
            let rows = DB_CONNECTIONS.with(|store| -> Result<Vec<VMValue>, String> {
                let store = store.borrow();
                let wrapper = store
                    .get(&handle_id)
                    .ok_or_else(|| "DB.query_raw_params: invalid DbHandle".to_string())?;
                sqlite_query_raw_params(&wrapper.conn, &sql, &params)
            })?;
            Ok(ok_vm(VMValue::List(FavList::new(rows))))
        }

        "DB.execute_raw_params" => {
            if args.len() != 3 {
                return Err("DB.execute_raw_params requires 3 arguments".to_string());
            }
            let mut it = args.into_iter();
            let handle_id = match it.next().unwrap() {
                VMValue::DbHandle(id) => id,
                other => {
                    return Err(format!(
                        "DB.execute_raw_params expects DbHandle, got {}",
                        vmvalue_type_name(&other)
                    ));
                }
            };
            let sql = vm_string(it.next().unwrap(), "DB.execute_raw_params")?;
            let params = match it.next().unwrap() {
                VMValue::List(v) => v
                    .into_iter()
                    .map(|p| vm_string(p, "DB.execute_raw_params param"))
                    .collect::<Result<Vec<_>, _>>()?,
                other => {
                    return Err(format!(
                        "DB.execute_raw_params: params must be List<String>, got {}",
                        vmvalue_type_name(&other)
                    ));
                }
            };
            let n = DB_CONNECTIONS.with(|store| -> Result<i64, String> {
                let store = store.borrow();
                let wrapper = store
                    .get(&handle_id)
                    .ok_or_else(|| "DB.execute_raw_params: invalid DbHandle".to_string())?;
                let param_refs: Vec<&dyn rusqlite::ToSql> =
                    params.iter().map(|s| s as &dyn rusqlite::ToSql).collect();
                wrapper
                    .conn
                    .execute(&sql, param_refs.as_slice())
                    .map(|n| n as i64)
                    .map_err(|e| format!("E0602: db query failed: {}", e))
            })?;
            Ok(ok_vm(VMValue::Int(n)))
        }

        "DB.upsert_raw" => {
            if args.len() != 4 {
                return Err("DB.upsert_raw requires 4 arguments".to_string());
            }
            let mut it = args.into_iter();
            let handle_id = match it.next().unwrap() {
                VMValue::DbHandle(id) => id,
                other => {
                    return Err(format!(
                        "DB.upsert_raw expects DbHandle, got {}",
                        vmvalue_type_name(&other)
                    ));
                }
            };
            let table_name = vm_string(it.next().unwrap(), "DB.upsert_raw type_name")?;
            let row = match it.next().unwrap() {
                VMValue::Record(map) => map
                    .into_iter()
                    .map(|(k, v)| Ok((k, vm_string(v, "DB.upsert_raw row value")?)))
                    .collect::<Result<HashMap<_, _>, String>>()?,
                other => {
                    return Err(format!(
                        "DB.upsert_raw expects Map<String,String>, got {}",
                        vmvalue_type_name(&other)
                    ));
                }
            };
            let key_field = vm_string(it.next().unwrap(), "DB.upsert_raw key_field")?;
            if row.is_empty() {
                return Ok(VMValue::Unit);
            }
            let mut columns: Vec<String> = row.keys().cloned().collect();
            columns.sort();
            if !columns.iter().any(|c| c == &key_field) {
                return Err(format!(
                    "DB.upsert_raw key field `{}` is missing from row",
                    key_field
                ));
            }
            let placeholders = (1..=columns.len())
                .map(|idx| format!("?{}", idx))
                .collect::<Vec<_>>()
                .join(", ");
            let assignments = columns
                .iter()
                .filter(|c| *c != &key_field)
                .map(|c| format!("{c} = excluded.{c}"))
                .collect::<Vec<_>>()
                .join(", ");
            let sql = if assignments.is_empty() {
                format!(
                    "INSERT OR IGNORE INTO {table_name} ({}) VALUES ({})",
                    columns.join(", "),
                    placeholders
                )
            } else {
                format!(
                    "INSERT INTO {table_name} ({}) VALUES ({}) ON CONFLICT({key_field}) DO UPDATE SET {}",
                    columns.join(", "),
                    placeholders,
                    assignments
                )
            };
            let values: Vec<String> = columns
                .iter()
                .map(|c| row.get(c).cloned().unwrap_or_default())
                .collect();
            Ok(DB_CONNECTIONS.with(|store| -> Result<VMValue, String> {
                let store = store.borrow();
                let wrapper = store
                    .get(&handle_id)
                    .ok_or_else(|| "DB.upsert_raw: invalid DbHandle".to_string())?;
                let param_refs: Vec<&dyn rusqlite::ToSql> =
                    values.iter().map(|s| s as &dyn rusqlite::ToSql).collect();
                wrapper
                    .conn
                    .execute(&sql, param_refs.as_slice())
                    .map_err(|e| format!("E0602: db query failed: {}", e))?;
                Ok(VMValue::Unit)
            })?)
        }

        "DB.begin_tx" => {
            if args.len() != 1 {
                return Err("DB.begin_tx requires 1 argument".to_string());
            }
            let handle_id = match args.into_iter().next().unwrap() {
                VMValue::DbHandle(id) => id,
                other => {
                    return Err(format!(
                        "DB.begin_tx expects DbHandle, got {}",
                        vmvalue_type_name(&other)
                    ));
                }
            };
            DB_CONNECTIONS.with(|store| -> Result<VMValue, String> {
                let mut store = store.borrow_mut();
                let wrapper = store
                    .get_mut(&handle_id)
                    .ok_or_else(|| "DB.begin_tx: invalid DbHandle".to_string())?;
                if wrapper.in_tx {
                    return Ok(err_vm(db_error_vm(
                        "E0603",
                        "db transaction failed: already in transaction",
                    )));
                }
                wrapper
                    .conn
                    .execute_batch("BEGIN")
                    .map_err(|e| format!("E0603: db transaction failed: {}", e))?;
                wrapper.in_tx = true;
                Ok(ok_vm(VMValue::TxHandle(handle_id)))
            })
        }

        "DB.commit_tx" => {
            if args.len() != 1 {
                return Err("DB.commit_tx requires 1 argument".to_string());
            }
            let tx_id = match args.into_iter().next().unwrap() {
                VMValue::TxHandle(id) => id,
                other => {
                    return Err(format!(
                        "DB.commit_tx expects TxHandle, got {}",
                        vmvalue_type_name(&other)
                    ));
                }
            };
            DB_CONNECTIONS.with(|store| -> Result<VMValue, String> {
                let mut store = store.borrow_mut();
                let wrapper = store
                    .get_mut(&tx_id)
                    .ok_or_else(|| "DB.commit_tx: invalid TxHandle".to_string())?;
                wrapper
                    .conn
                    .execute_batch("COMMIT")
                    .map_err(|e| format!("E0603: db transaction failed: {}", e))?;
                wrapper.in_tx = false;
                Ok(ok_vm(VMValue::Unit))
            })
        }

        "DB.rollback_tx" => {
            if args.len() != 1 {
                return Err("DB.rollback_tx requires 1 argument".to_string());
            }
            let tx_id = match args.into_iter().next().unwrap() {
                VMValue::TxHandle(id) => id,
                other => {
                    return Err(format!(
                        "DB.rollback_tx expects TxHandle, got {}",
                        vmvalue_type_name(&other)
                    ));
                }
            };
            DB_CONNECTIONS.with(|store| -> Result<VMValue, String> {
                let mut store = store.borrow_mut();
                let wrapper = store
                    .get_mut(&tx_id)
                    .ok_or_else(|| "DB.rollback_tx: invalid TxHandle".to_string())?;
                wrapper
                    .conn
                    .execute_batch("ROLLBACK")
                    .map_err(|e| format!("E0603: db transaction failed: {}", e))?;
                wrapper.in_tx = false;
                Ok(ok_vm(VMValue::Unit))
            })
        }

        "DB.query_in_tx" => {
            if args.len() != 2 {
                return Err("DB.query_in_tx requires 2 arguments".to_string());
            }
            let mut it = args.into_iter();
            let tx_id = match it.next().unwrap() {
                VMValue::TxHandle(id) => id,
                other => {
                    return Err(format!(
                        "DB.query_in_tx expects TxHandle, got {}",
                        vmvalue_type_name(&other)
                    ));
                }
            };
            let sql = vm_string(it.next().unwrap(), "DB.query_in_tx")?;
            let rows = DB_CONNECTIONS.with(|store| -> Result<Vec<VMValue>, String> {
                let store = store.borrow();
                let wrapper = store
                    .get(&tx_id)
                    .ok_or_else(|| "DB.query_in_tx: invalid TxHandle".to_string())?;
                sqlite_query_raw(&wrapper.conn, &sql)
            })?;
            Ok(ok_vm(VMValue::List(FavList::new(rows))))
        }

        "DB.execute_in_tx" => {
            if args.len() != 2 {
                return Err("DB.execute_in_tx requires 2 arguments".to_string());
            }
            let mut it = args.into_iter();
            let tx_id = match it.next().unwrap() {
                VMValue::TxHandle(id) => id,
                other => {
                    return Err(format!(
                        "DB.execute_in_tx expects TxHandle, got {}",
                        vmvalue_type_name(&other)
                    ));
                }
            };
            let sql = vm_string(it.next().unwrap(), "DB.execute_in_tx")?;
            let n = DB_CONNECTIONS.with(|store| -> Result<i64, String> {
                let store = store.borrow();
                let wrapper = store
                    .get(&tx_id)
                    .ok_or_else(|| "DB.execute_in_tx: invalid TxHandle".to_string())?;
                wrapper
                    .conn
                    .execute(&sql, [])
                    .map(|n| n as i64)
                    .map_err(|e| format!("E0602: db query failed: {}", e))
            })?;
            Ok(ok_vm(VMValue::Int(n)))
        }

        // ── Env.* (v3.3.0) ─────────────────────────────────────────────────
        "Env.get" => {
            if args.len() != 1 {
                return Err("Env.get requires 1 argument".to_string());
            }
            let name = vm_string(args.into_iter().next().unwrap(), "Env.get")?;
            match std::env::var(&name) {
                Ok(val) => Ok(ok_vm(VMValue::Str(val))),
                Err(_) => Ok(err_vm(db_error_vm(
                    "E0001",
                    &format!("environment variable '{}' not found", name),
                ))),
            }
        }

        "Env.get_or" => {
            if args.len() != 2 {
                return Err("Env.get_or requires 2 arguments".to_string());
            }
            let mut it = args.into_iter();
            let name = vm_string(it.next().unwrap(), "Env.get_or")?;
            let default = vm_string(it.next().unwrap(), "Env.get_or default")?;
            Ok(VMValue::Str(std::env::var(&name).unwrap_or(default)))
        }

        "Checkpoint.last" => {
            if args.len() != 1 {
                return Err("Checkpoint.last requires 1 argument".to_string());
            }
            let name = vm_string(args.into_iter().next().unwrap(), "Checkpoint.last")?;
            match checkpoint_last_impl(&name)? {
                Some(value) => Ok(VMValue::Variant(
                    "some".to_string(),
                    Some(Box::new(VMValue::Str(value))),
                )),
                None => Ok(VMValue::Variant("none".to_string(), None)),
            }
        }

        "Checkpoint.save" => {
            if args.len() != 2 {
                return Err("Checkpoint.save requires 2 arguments".to_string());
            }
            let mut it = args.into_iter();
            let name = vm_string(it.next().unwrap(), "Checkpoint.save")?;
            let value = vm_string(it.next().unwrap(), "Checkpoint.save value")?;
            checkpoint_save_impl(&name, &value)?;
            Ok(VMValue::Unit)
        }

        "Checkpoint.reset" => {
            if args.len() != 1 {
                return Err("Checkpoint.reset requires 1 argument".to_string());
            }
            let name = vm_string(args.into_iter().next().unwrap(), "Checkpoint.reset")?;
            checkpoint_reset_impl(&name)?;
            Ok(VMValue::Unit)
        }

        "Checkpoint.meta" => {
            if args.len() != 1 {
                return Err("Checkpoint.meta requires 1 argument".to_string());
            }
            let name = vm_string(args.into_iter().next().unwrap(), "Checkpoint.meta")?;
            let meta = checkpoint_meta_impl(&name)?;
            let mut map = HashMap::new();
            map.insert("name".to_string(), VMValue::Str(meta.name));
            map.insert("value".to_string(), VMValue::Str(meta.value));
            map.insert("updated_at".to_string(), VMValue::Str(meta.updated_at));
            Ok(VMValue::Record(map))
        }

        "Parquet.write_raw" => {
            if args.len() != 3 {
                return Err("Parquet.write_raw requires 3 arguments".to_string());
            }
            let mut it = args.into_iter();
            let path = vm_string(it.next().unwrap(), "Parquet.write_raw path")?;
            let type_name = vm_string(it.next().unwrap(), "Parquet.write_raw type_name")?;
            let rows = schema_rows_from_vm(it.next().unwrap(), "Parquet.write_raw")?;
            match parquet_write_rows(&path, &type_name, rows, type_metas) {
                Ok(()) => Ok(ok_vm(VMValue::Unit)),
                Err(err) => Ok(err_vm(parquet_error_vm(err))),
            }
        }

        "Parquet.read_raw" => {
            if args.len() != 1 {
                return Err("Parquet.read_raw requires 1 argument".to_string());
            }
            let path = vm_string(args.into_iter().next().unwrap(), "Parquet.read_raw path")?;
            match parquet_read_rows(&path) {
                Ok(rows) => Ok(ok_vm(VMValue::List(FavList::new(
                    rows.into_iter().map(VMValue::Record).collect(),
                )))),
                Err(err) => Ok(err_vm(parquet_error_vm(err))),
            }
        }

        // ── Validate.* (v4.1.5) ───────────────────────────────────────────────
        "Validate.run_raw" => {
            if args.len() != 2 {
                return Err(
                    "Validate.run_raw requires 2 arguments (type_name, raw_map)".to_string()
                );
            }
            let mut it = args.into_iter();
            let type_name = vm_string(it.next().unwrap(), "Validate.run_raw type_name")?;
            let raw = match it.next().unwrap() {
                VMValue::Record(m) => m,
                other => {
                    return Err(format!(
                        "Validate.run_raw: second argument must be a Map/Record, got {}",
                        vmvalue_type_name(&other)
                    ));
                }
            };

            let schemas = SCHEMA_REGISTRY.with(|s| s.borrow().clone());
            validate_record_inner(&type_name, raw, &schemas)
        }

        // Validate.rows_raw(type_name, rows) — validate a list of records (v6.6.0)
        // Returns Ok(rows) if all pass, Err(first_error_list) on first violation.
        "Validate.rows_raw" => {
            if args.len() != 2 {
                return Err(
                    "Validate.rows_raw requires 2 arguments (type_name, rows)".to_string()
                );
            }
            let mut it = args.into_iter();
            let type_name = vm_string(it.next().unwrap(), "Validate.rows_raw type_name")?;
            let rows = match it.next().unwrap() {
                VMValue::List(lst) => lst.to_vec(),
                other => {
                    return Err(format!(
                        "Validate.rows_raw: second argument must be List, got {}",
                        vmvalue_type_name(&other)
                    ));
                }
            };
            let schemas = SCHEMA_REGISTRY.with(|s| s.borrow().clone());
            for row in &rows {
                if let VMValue::Record(m) = row {
                    let result = validate_record_inner(&type_name, m.clone(), &schemas)?;
                    // If any row fails, return the Err immediately
                    if let VMValue::Variant(ref tag, _) = result {
                        if tag == "err" {
                            return Ok(result);
                        }
                    }
                }
            }
            Ok(ok_vm(VMValue::List(FavList::new(rows))))
        }

        // Dynamic TypeName.validate(record) dispatch (v6.6.0)
        // Handles calls like Order.validate(raw_order) where Order has a schema entry.
        name if name.ends_with(".validate") => {
            let type_name = &name[..name.len() - ".validate".len()];
            if args.len() != 1 {
                return Err(format!(
                    "{}.validate requires 1 argument (record)",
                    type_name
                ));
            }
            let raw = match args.into_iter().next().unwrap() {
                VMValue::Record(m) => m,
                other => {
                    return Err(format!(
                        "{}.validate: argument must be a Record, got {}",
                        type_name,
                        vmvalue_type_name(&other)
                    ));
                }
            };
            let schemas = SCHEMA_REGISTRY.with(|s| s.borrow().clone());
            validate_record_inner(type_name, raw, &schemas)
        }

        // ── DuckDb.* (v4.3.0) — embedded OLAP engine ───────────────────────
        "DuckDb.open_raw" => {
            if args.len() != 1 {
                return Err("DuckDb.open_raw requires 1 argument".to_string());
            }
            let path = vm_string(args.into_iter().next().unwrap(), "DuckDb.open_raw")?;
            let config = duckdb::Config::default()
                .enable_autoload_extension(false)
                .map_err(|e| format!("DuckDB config error: {}", e))?;
            match duckdb::Connection::open_with_flags(&path, config) {
                Ok(conn) => {
                    let id = DUCKDB_NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    duckdb_store().insert(id, conn);
                    Ok(ok_vm(VMValue::DbHandle(id)))
                }
                Err(e) => Ok(err_vm(db_error_vm(
                    "OPEN_ERROR",
                    &format!("DuckDB open error: {}", e),
                ))),
            }
        }

        "DuckDb.query_raw" => {
            if args.len() != 2 {
                return Err("DuckDb.query_raw requires 2 arguments".to_string());
            }
            let mut it = args.into_iter();
            let handle_id = match it.next().unwrap() {
                VMValue::DbHandle(id) => id,
                other => {
                    return Err(format!(
                        "DuckDb.query_raw expects DbHandle, got {}",
                        vmvalue_type_name(&other)
                    ));
                }
            };
            let sql = vm_string(it.next().unwrap(), "DuckDb.query_raw")?;
            let result = {
                let store = duckdb_store();
                let conn = store
                    .get(&handle_id)
                    .ok_or_else(|| "DuckDb.query_raw: invalid DbHandle".to_string())?;
                duckdb_query_raw(conn, &sql)
            };
            match result {
                Ok(rows) => Ok(ok_vm(VMValue::List(FavList::new(rows)))),
                Err(e) => Ok(err_vm(db_error_vm("QUERY_ERROR", &e))),
            }
        }

        "DuckDb.execute_raw" => {
            if args.len() != 2 {
                return Err("DuckDb.execute_raw requires 2 arguments".to_string());
            }
            let mut it = args.into_iter();
            let handle_id = match it.next().unwrap() {
                VMValue::DbHandle(id) => id,
                other => {
                    return Err(format!(
                        "DuckDb.execute_raw expects DbHandle, got {}",
                        vmvalue_type_name(&other)
                    ));
                }
            };
            let sql = vm_string(it.next().unwrap(), "DuckDb.execute_raw")?;
            let exec_result = {
                let store = duckdb_store();
                let conn = store
                    .get(&handle_id)
                    .ok_or_else(|| "DuckDb.execute_raw: invalid DbHandle".to_string())?;
                conn.execute(&sql, [])
                    .map(|n| n as i64)
                    .map_err(|e| format!("DuckDB execute failed: {}", e))
            };
            match exec_result {
                Ok(n) => Ok(ok_vm(VMValue::Int(n))),
                Err(e) => Ok(err_vm(db_error_vm("EXECUTE_ERROR", &e))),
            }
        }

        "DuckDb.close_raw" => {
            if args.len() != 1 {
                return Err("DuckDb.close_raw requires 1 argument".to_string());
            }
            match args.into_iter().next().unwrap() {
                VMValue::DbHandle(id) => {
                    duckdb_store().remove(&id);
                    Ok(VMValue::Unit)
                }
                other => Err(format!(
                    "DuckDb.close_raw expects DbHandle, got {}",
                    vmvalue_type_name(&other)
                )),
            }
        }

        // ── AWS S3 (v4.11.0) ─────────────────────────────────────────────
        "AWS.s3_get_object_raw" => {
            let mut it = args.into_iter();
            let bucket = vm_string(
                it.next().ok_or("s3_get_object_raw: missing bucket")?,
                "AWS.s3_get_object_raw",
            )?;
            let key = vm_string(
                it.next().ok_or("s3_get_object_raw: missing key")?,
                "AWS.s3_get_object_raw",
            )?;
            let config = get_aws_config();
            let base = if let Some(ep) = &config.endpoint_url {
                format!("{}/{}", ep.trim_end_matches('/'), bucket)
            } else {
                format!("https://{}.s3.{}.amazonaws.com", bucket, config.region)
            };
            let url = format!("{}/{}", base, key);
            Ok(match aws_get(&config, "s3", &url) {
                Ok(body) => ok_vm(VMValue::Str(body)),
                Err(e) => err_vm(VMValue::Str(e)),
            })
        }

        "AWS.s3_put_object_raw" => {
            let mut it = args.into_iter();
            let bucket = vm_string(
                it.next().ok_or("s3_put_object_raw: missing bucket")?,
                "AWS.s3_put_object_raw",
            )?;
            let key = vm_string(
                it.next().ok_or("s3_put_object_raw: missing key")?,
                "AWS.s3_put_object_raw",
            )?;
            let body = vm_string(
                it.next().ok_or("s3_put_object_raw: missing body")?,
                "AWS.s3_put_object_raw",
            )?;
            let config = get_aws_config();
            let base = if let Some(ep) = &config.endpoint_url {
                format!("{}/{}", ep.trim_end_matches('/'), bucket)
            } else {
                format!("https://{}.s3.{}.amazonaws.com", bucket, config.region)
            };
            let url = format!("{}/{}", base, key);
            Ok(match aws_put(&config, "s3", &url, &body) {
                Ok(()) => ok_vm(VMValue::Unit),
                Err(e) => err_vm(VMValue::Str(e)),
            })
        }

        "AWS.s3_get_object_base64_raw" => {
            let mut it = args.into_iter();
            let bucket = vm_string(
                it.next()
                    .ok_or("s3_get_object_base64_raw: missing bucket")?,
                "AWS.s3_get_object_base64_raw",
            )?;
            let key = vm_string(
                it.next().ok_or("s3_get_object_base64_raw: missing key")?,
                "AWS.s3_get_object_base64_raw",
            )?;
            let config = get_aws_config();
            let base = if let Some(ep) = &config.endpoint_url {
                format!("{}/{}", ep.trim_end_matches('/'), bucket)
            } else {
                format!("https://{}.s3.{}.amazonaws.com", bucket, config.region)
            };
            let url = format!("{}/{}", base, key);
            use base64::Engine;
            Ok(match aws_get_bytes(&config, "s3", &url) {
                Ok(bytes) => ok_vm(VMValue::Str(
                    base64::engine::general_purpose::STANDARD.encode(&bytes),
                )),
                Err(e) => err_vm(VMValue::Str(e)),
            })
        }

        "AWS.s3_put_bytes_raw" => {
            let mut it = args.into_iter();
            let bucket = vm_string(
                it.next().ok_or("s3_put_bytes_raw: missing bucket")?,
                "AWS.s3_put_bytes_raw",
            )?;
            let key = vm_string(
                it.next().ok_or("s3_put_bytes_raw: missing key")?,
                "AWS.s3_put_bytes_raw",
            )?;
            let bytes_val = it.next().ok_or("s3_put_bytes_raw: missing bytes")?;
            let bytes: Vec<u8> = match &bytes_val {
                VMValue::List(lst) => lst
                    .iter()
                    .map(|v| match v {
                        VMValue::Int(n) => (n & 0xFF) as u8,
                        _ => 0u8,
                    })
                    .collect(),
                _ => {
                    return Err(format!(
                        "s3_put_bytes_raw: bytes must be List<Int>, got {}",
                        vmvalue_type_name(&bytes_val)
                    ));
                }
            };
            let config = get_aws_config();
            let base = if let Some(ep) = &config.endpoint_url {
                format!("{}/{}", ep.trim_end_matches('/'), bucket)
            } else {
                format!("https://{}.s3.{}.amazonaws.com", bucket, config.region)
            };
            let url = format!("{}/{}", base, key);
            Ok(match aws_put_bytes(&config, "s3", &url, &bytes) {
                Ok(()) => ok_vm(VMValue::Unit),
                Err(e) => err_vm(VMValue::Str(e)),
            })
        }

        "AWS.s3_delete_object_raw" => {
            let mut it = args.into_iter();
            let bucket = vm_string(
                it.next().ok_or("s3_delete_object_raw: missing bucket")?,
                "AWS.s3_delete_object_raw",
            )?;
            let key = vm_string(
                it.next().ok_or("s3_delete_object_raw: missing key")?,
                "AWS.s3_delete_object_raw",
            )?;
            let config = get_aws_config();
            let base = if let Some(ep) = &config.endpoint_url {
                format!("{}/{}", ep.trim_end_matches('/'), bucket)
            } else {
                format!("https://{}.s3.{}.amazonaws.com", bucket, config.region)
            };
            let url = format!("{}/{}", base, key);
            Ok(match aws_delete(&config, "s3", &url) {
                Ok(()) => ok_vm(VMValue::Unit),
                Err(e) => err_vm(VMValue::Str(e)),
            })
        }

        "AWS.s3_list_objects_raw" => {
            let mut it = args.into_iter();
            let bucket = vm_string(
                it.next().ok_or("s3_list_objects_raw: missing bucket")?,
                "AWS.s3_list_objects_raw",
            )?;
            let prefix = vm_string(
                it.next().ok_or("s3_list_objects_raw: missing prefix")?,
                "AWS.s3_list_objects_raw",
            )?;
            let config = get_aws_config();
            let base = if let Some(ep) = &config.endpoint_url {
                format!("{}/{}", ep.trim_end_matches('/'), bucket)
            } else {
                format!("https://{}.s3.{}.amazonaws.com", bucket, config.region)
            };
            let url = format!("{}/?list-type=2&prefix={}", base, url_encode(&prefix));
            Ok(match aws_get(&config, "s3", &url) {
                Ok(xml) => {
                    let keys: Vec<VMValue> = extract_xml_tags(&xml, "Key")
                        .into_iter()
                        .map(VMValue::Str)
                        .collect();
                    ok_vm(VMValue::List(FavList::new(keys)))
                }
                Err(e) => err_vm(VMValue::Str(e)),
            })
        }

        "AWS.s3_head_bucket_raw" => {
            let mut it = args.into_iter();
            let bucket = vm_string(
                it.next().ok_or("s3_head_bucket_raw: missing bucket")?,
                "AWS.s3_head_bucket_raw",
            )?;
            let config = get_aws_config();
            let url = if let Some(ep) = &config.endpoint_url {
                format!("{}/{}", ep.trim_end_matches('/'), bucket)
            } else {
                format!("https://{}.s3.{}.amazonaws.com", bucket, config.region)
            };
            Ok(match aws_head(&config, "s3", &url) {
                Ok(b) => ok_vm(VMValue::Bool(b)),
                Err(e) => err_vm(VMValue::Str(e)),
            })
        }

        // ── AWS S3 extended (v25.2.0) ────────────────────────────────────
        "AWS.s3_presign_url_raw" => {
            let mut it = args.into_iter();
            let bucket = vm_string(
                it.next().ok_or("s3_presign_url_raw: missing bucket")?,
                "AWS.s3_presign_url_raw",
            )?;
            let key = vm_string(
                it.next().ok_or("s3_presign_url_raw: missing key")?,
                "AWS.s3_presign_url_raw",
            )?;
            let ttl: i64 = match it.next() {
                Some(VMValue::Int(n)) => n,
                _ => 3600,
            };
            let config = get_aws_config();
            let base = if let Some(ep) = &config.endpoint_url {
                format!("{}/{}", ep.trim_end_matches('/'), bucket)
            } else {
                format!("https://{}.s3.{}.amazonaws.com", bucket, config.region)
            };
            let object_url = format!("{}/{}", base, key);
            // Build presigned URL using SigV4 query parameters.
            // For LocalStack (endpoint_url set) use a simplified unsigned URL.
            let presigned = if config.endpoint_url.is_some() {
                format!("{}?X-Amz-Algorithm=AWS4-HMAC-SHA256&X-Amz-Expires={}&X-Amz-Signature=dummy", object_url, ttl)
            } else {
                let now = chrono::Utc::now();
                let amz_date = now.format("%Y%m%dT%H%M%SZ").to_string();
                let date_stamp = now.format("%Y%m%d").to_string();
                let credential_scope = format!("{}/{}/s3/aws4_request", date_stamp, config.region);
                let credential = format!("{}/{}", config.access_key, credential_scope);
                let canonical_qs = format!(
                    "X-Amz-Algorithm=AWS4-HMAC-SHA256&X-Amz-Credential={}&X-Amz-Date={}&X-Amz-Expires={}&X-Amz-SignedHeaders=host",
                    url_encode(&credential), amz_date, ttl
                );
                let host = object_url
                    .trim_start_matches("https://")
                    .split('/')
                    .next()
                    .unwrap_or("");
                let path = {
                    let after = object_url.splitn(2, "://").nth(1).unwrap_or(&object_url);
                    after.splitn(2, '/').nth(1).map(|s| format!("/{}", s)).unwrap_or_else(|| "/".to_string())
                };
                let canonical_req = format!(
                    "GET\n{}\n{}\nhost:{}\n\nhost\nUNSIGNED-PAYLOAD",
                    path, canonical_qs, host
                );
                let string_to_sign = format!(
                    "AWS4-HMAC-SHA256\n{}\n{}\n{}",
                    amz_date,
                    credential_scope,
                    sha256_hex_bytes(canonical_req.as_bytes())
                );
                let signing_key = sigv4_signing_key(&config.secret_key, &date_stamp, &config.region, "s3");
                let sig_bytes = hmac_sha256_bytes(&signing_key, string_to_sign.as_bytes());
                let signature: String = sig_bytes.iter().map(|b| format!("{:02x}", b)).collect();
                format!("{}?{}&X-Amz-Signature={}", object_url, canonical_qs, signature)
            };
            Ok(ok_vm(VMValue::Str(presigned)))
        }

        "AWS.s3_stream_get_raw" => {
            // TODO(v25.x): Stream<Bytes> 対応 — 現バージョンは get_object と同一ロジック
            let mut it = args.into_iter();
            let bucket = vm_string(
                it.next().ok_or("s3_stream_get_raw: missing bucket")?,
                "AWS.s3_stream_get_raw",
            )?;
            let key = vm_string(
                it.next().ok_or("s3_stream_get_raw: missing key")?,
                "AWS.s3_stream_get_raw",
            )?;
            let config = get_aws_config();
            let base = if let Some(ep) = &config.endpoint_url {
                format!("{}/{}", ep.trim_end_matches('/'), bucket)
            } else {
                format!("https://{}.s3.{}.amazonaws.com", bucket, config.region)
            };
            let url = format!("{}/{}", base, key);
            Ok(match aws_get(&config, "s3", &url) {
                Ok(body) => ok_vm(VMValue::Str(body)),
                Err(e) => err_vm(VMValue::Str(e)),
            })
        }

        // ── AWS SQS (v4.11.0) ─────────────────────────────────────────────
        "AWS.sqs_send_message_raw" => {
            let mut it = args.into_iter();
            let queue_url = vm_string(
                it.next().ok_or("sqs_send_message_raw: missing queue_url")?,
                "AWS.sqs_send_message_raw",
            )?;
            let body = vm_string(
                it.next().ok_or("sqs_send_message_raw: missing body")?,
                "AWS.sqs_send_message_raw",
            )?;
            let config = get_aws_config();
            let form = format!(
                "Action=SendMessage&MessageBody={}&Version=2012-11-05",
                url_encode(&body)
            );
            Ok(
                match aws_post(
                    &config,
                    "sqs",
                    &queue_url,
                    &form,
                    "application/x-www-form-urlencoded",
                    None,
                ) {
                    Ok(xml) => {
                        let ids = extract_xml_tags(&xml, "MessageId");
                        let id = ids.into_iter().next().unwrap_or_default();
                        ok_vm(VMValue::Str(id))
                    }
                    Err(e) => err_vm(VMValue::Str(e)),
                },
            )
        }

        "AWS.sqs_receive_messages_raw" => {
            let mut it = args.into_iter();
            let queue_url = vm_string(
                it.next()
                    .ok_or("sqs_receive_messages_raw: missing queue_url")?,
                "AWS.sqs_receive_messages_raw",
            )?;
            let max = match it.next().ok_or("sqs_receive_messages_raw: missing max")? {
                VMValue::Int(n) => n,
                _ => 1,
            };
            let config = get_aws_config();
            let form = format!(
                "Action=ReceiveMessage&MaxNumberOfMessages={}&Version=2012-11-05",
                max
            );
            Ok(
                match aws_post(
                    &config,
                    "sqs",
                    &queue_url,
                    &form,
                    "application/x-www-form-urlencoded",
                    None,
                ) {
                    Ok(xml) => {
                        let messages = extract_xml_tags(&xml, "Message");
                        let items: Vec<VMValue> = messages
                            .into_iter()
                            .map(|msg| {
                                let mut map = std::collections::HashMap::new();
                                let ids = extract_xml_tags(&msg, "MessageId");
                                let bodies = extract_xml_tags(&msg, "Body");
                                let handles = extract_xml_tags(&msg, "ReceiptHandle");
                                map.insert(
                                    "message_id".to_string(),
                                    VMValue::Str(ids.into_iter().next().unwrap_or_default()),
                                );
                                map.insert(
                                    "body".to_string(),
                                    VMValue::Str(bodies.into_iter().next().unwrap_or_default()),
                                );
                                map.insert(
                                    "receipt_handle".to_string(),
                                    VMValue::Str(handles.into_iter().next().unwrap_or_default()),
                                );
                                VMValue::Record(map)
                            })
                            .collect();
                        ok_vm(VMValue::List(FavList::new(items)))
                    }
                    Err(e) => err_vm(VMValue::Str(e)),
                },
            )
        }

        "AWS.sqs_delete_message_raw" => {
            let mut it = args.into_iter();
            let queue_url = vm_string(
                it.next()
                    .ok_or("sqs_delete_message_raw: missing queue_url")?,
                "AWS.sqs_delete_message_raw",
            )?;
            let receipt_handle = vm_string(
                it.next()
                    .ok_or("sqs_delete_message_raw: missing receipt_handle")?,
                "AWS.sqs_delete_message_raw",
            )?;
            let config = get_aws_config();
            let form = format!(
                "Action=DeleteMessage&ReceiptHandle={}&Version=2012-11-05",
                url_encode(&receipt_handle)
            );
            Ok(
                match aws_post(
                    &config,
                    "sqs",
                    &queue_url,
                    &form,
                    "application/x-www-form-urlencoded",
                    None,
                ) {
                    Ok(_) => ok_vm(VMValue::Unit),
                    Err(e) => err_vm(VMValue::Str(e)),
                },
            )
        }

        "AWS.sqs_get_queue_url_raw" => {
            let mut it = args.into_iter();
            let queue_name = vm_string(
                it.next()
                    .ok_or("sqs_get_queue_url_raw: missing queue_name")?,
                "AWS.sqs_get_queue_url_raw",
            )?;
            let config = get_aws_config();
            let base = if let Some(ep) = &config.endpoint_url {
                ep.trim_end_matches('/').to_string()
            } else {
                format!("https://sqs.{}.amazonaws.com", config.region)
            };
            let form = format!(
                "Action=GetQueueUrl&QueueName={}&Version=2012-11-05",
                url_encode(&queue_name)
            );
            Ok(
                match aws_post(
                    &config,
                    "sqs",
                    &base,
                    &form,
                    "application/x-www-form-urlencoded",
                    None,
                ) {
                    Ok(xml) => {
                        let urls = extract_xml_tags(&xml, "QueueUrl");
                        let url = urls.into_iter().next().unwrap_or_default();
                        ok_vm(VMValue::Str(url))
                    }
                    Err(e) => err_vm(VMValue::Str(e)),
                },
            )
        }

        // ── SQS Rune primitives (v26.8.0) ────────────────────────────────
        "SQS.send_message_batch_raw" => {
            let mut it = args.into_iter();
            let queue_url = vm_string(
                it.next().ok_or("SQS.send_message_batch_raw: missing queue_url")?,
                "SQS.send_message_batch_raw",
            )?;
            let messages = match it.next().ok_or("SQS.send_message_batch_raw: missing messages")? {
                VMValue::List(list) => list,
                _ => return Err("SQS.send_message_batch_raw: messages must be a List".to_string()),
            };
            if messages.len() > 10 {
                return Ok(err_vm(VMValue::Str(format!(
                    "SQS.send_message_batch: batch size {} exceeds SQS limit of 10",
                    messages.len()
                ))));
            }
            let config = get_aws_config();
            let mut form = "Action=SendMessageBatch&Version=2012-11-05".to_string();
            for (i, msg) in messages.iter().enumerate() {
                let body = match msg {
                    VMValue::Str(s) => s.clone(),
                    _ => format!("{:?}", msg),
                };
                let n = i + 1; // SQS バッチエントリは 1-indexed（SQS API 仕様）
                form.push_str(&format!(
                    "&SendMessageBatchRequestEntry.{n}.Id=msg{n}&SendMessageBatchRequestEntry.{n}.MessageBody={}",
                    url_encode(&body),
                ));
            }
            Ok(match aws_post(&config, "sqs", &queue_url, &form, "application/x-www-form-urlencoded", None) {
                Ok(xml) => {
                    let ids = extract_xml_tags(&xml, "MessageId");
                    ok_vm(VMValue::Str(format!("{{\"sent\":{}}}", ids.len())))
                }
                Err(e) => err_vm(VMValue::Str(e)),
            })
        }

        "SQS.receive_messages_raw" => {
            // AttributeName.1=All: LocalStack 動作確認済み。本番 AWS の MessageAttribute 取得は v27.x で対応。
            let mut it = args.into_iter();
            let queue_url = vm_string(
                it.next().ok_or("SQS.receive_messages_raw: missing queue_url")?,
                "SQS.receive_messages_raw",
            )?;
            let max = match it.next().ok_or("SQS.receive_messages_raw: missing max")? {
                VMValue::Int(n) => n,
                _ => return Err("SQS.receive_messages_raw: max must be an Int".to_string()),
            };
            if max < 1 || max > 10 {
                return Ok(err_vm(VMValue::Str(format!(
                    "SQS.receive_messages: max must be between 1 and 10, got {}",
                    max
                ))));
            }
            let config = get_aws_config();
            let form = format!(
                "Action=ReceiveMessage&MaxNumberOfMessages={}&AttributeName.1=All&Version=2012-11-05",
                max
            );
            Ok(match aws_post(&config, "sqs", &queue_url, &form, "application/x-www-form-urlencoded", None) {
                Ok(xml) => {
                    let messages = extract_xml_tags(&xml, "Message");
                    let items: Vec<String> = messages.into_iter().map(|msg| {
                        let id = extract_xml_tags(&msg, "MessageId").into_iter().next().unwrap_or_default();
                        let body = extract_xml_tags(&msg, "Body").into_iter().next().unwrap_or_default();
                        let handle = extract_xml_tags(&msg, "ReceiptHandle").into_iter().next().unwrap_or_default();
                        let id_j = serde_json::to_string(&id).unwrap_or_else(|_| "\"\"".to_string());
                        let body_j = serde_json::to_string(&body).unwrap_or_else(|_| "\"\"".to_string());
                        let handle_j = serde_json::to_string(&handle).unwrap_or_else(|_| "\"\"".to_string());
                        format!("{{\"message_id\":{},\"body\":{},\"receipt_handle\":{}}}", id_j, body_j, handle_j)
                    }).collect();
                    ok_vm(VMValue::Str(format!("[{}]", items.join(","))))
                }
                Err(e) => err_vm(VMValue::Str(e)),
            })
        }

        "SQS.purge_raw" => {
            let mut it = args.into_iter();
            let queue_url = vm_string(
                it.next().ok_or("SQS.purge_raw: missing queue_url")?,
                "SQS.purge_raw",
            )?;
            let config = get_aws_config();
            let form = "Action=PurgeQueue&Version=2012-11-05".to_string();
            Ok(match aws_post(&config, "sqs", &queue_url, &form, "application/x-www-form-urlencoded", None) {
                Ok(_) => ok_vm(VMValue::Unit),
                Err(e) => err_vm(VMValue::Str(e)),
            })
        }

        "SQS.consume_raw" => {
            // Stub: 1 回ポーリングして JSON 文字列で返す（継続ループは v27.x）
            let mut it = args.into_iter();
            let queue_url = vm_string(
                it.next().ok_or("SQS.consume_raw: missing queue_url")?,
                "SQS.consume_raw",
            )?;
            let config = get_aws_config();
            let form = "Action=ReceiveMessage&MaxNumberOfMessages=10&AttributeName.1=All&Version=2012-11-05".to_string();
            Ok(match aws_post(&config, "sqs", &queue_url, &form, "application/x-www-form-urlencoded", None) {
                Ok(xml) => {
                    let messages = extract_xml_tags(&xml, "Message");
                    let items: Vec<String> = messages.into_iter().map(|msg| {
                        let id = extract_xml_tags(&msg, "MessageId").into_iter().next().unwrap_or_default();
                        let body = extract_xml_tags(&msg, "Body").into_iter().next().unwrap_or_default();
                        let handle = extract_xml_tags(&msg, "ReceiptHandle").into_iter().next().unwrap_or_default();
                        let id_j = serde_json::to_string(&id).unwrap_or_else(|_| "\"\"".to_string());
                        let body_j = serde_json::to_string(&body).unwrap_or_else(|_| "\"\"".to_string());
                        let handle_j = serde_json::to_string(&handle).unwrap_or_else(|_| "\"\"".to_string());
                        format!("{{\"message_id\":{},\"body\":{},\"receipt_handle\":{}}}", id_j, body_j, handle_j)
                    }).collect();
                    ok_vm(VMValue::Str(format!("[{}]", items.join(","))))
                }
                Err(e) => err_vm(VMValue::Str(e)),
            })
        }

        // ── AWS DynamoDB (v4.11.0) ────────────────────────────────────────
        "AWS.dynamo_get_item_raw" => {
            let mut it = args.into_iter();
            let table = vm_string(
                it.next().ok_or("dynamo_get_item_raw: missing table")?,
                "AWS.dynamo_get_item_raw",
            )?;
            let key_map = match it.next().ok_or("dynamo_get_item_raw: missing key")? {
                VMValue::Record(m) => m,
                _ => return Err("dynamo_get_item_raw: key must be a Map".to_string()),
            };
            let config = get_aws_config();
            let url = if let Some(ep) = &config.endpoint_url {
                ep.trim_end_matches('/').to_string()
            } else {
                format!("https://dynamodb.{}.amazonaws.com", config.region)
            };
            let key_json = map_to_dynamo_item(&key_map);
            let body = format!(r#"{{"TableName":"{}","Key":{}}}"#, table, key_json);
            Ok(
                match aws_post(
                    &config,
                    "dynamodb",
                    &url,
                    &body,
                    "application/x-amz-json-1.0",
                    Some("DynamoDB_20120810.GetItem"),
                ) {
                    Ok(resp) => match serde_json::from_str::<serde_json::Value>(&resp) {
                        Ok(v) => {
                            if let Some(item) = v.get("Item") {
                                let m = dynamo_item_to_map(item);
                                ok_vm(VMValue::Variant(
                                    "some".into(),
                                    Some(Box::new(VMValue::Record(m))),
                                ))
                            } else {
                                ok_vm(VMValue::Variant("none".into(), None))
                            }
                        }
                        Err(e) => err_vm(VMValue::Str(e.to_string())),
                    },
                    Err(e) => err_vm(VMValue::Str(e)),
                },
            )
        }

        "AWS.dynamo_put_item_raw" => {
            let mut it = args.into_iter();
            let table = vm_string(
                it.next().ok_or("dynamo_put_item_raw: missing table")?,
                "AWS.dynamo_put_item_raw",
            )?;
            let item_map = match it.next().ok_or("dynamo_put_item_raw: missing item")? {
                VMValue::Record(m) => m,
                _ => return Err("dynamo_put_item_raw: item must be a Map".to_string()),
            };
            let config = get_aws_config();
            let url = if let Some(ep) = &config.endpoint_url {
                ep.trim_end_matches('/').to_string()
            } else {
                format!("https://dynamodb.{}.amazonaws.com", config.region)
            };
            let item_json = map_to_dynamo_item(&item_map);
            let body = format!(r#"{{"TableName":"{}","Item":{}}}"#, table, item_json);
            Ok(
                match aws_post(
                    &config,
                    "dynamodb",
                    &url,
                    &body,
                    "application/x-amz-json-1.0",
                    Some("DynamoDB_20120810.PutItem"),
                ) {
                    Ok(_) => ok_vm(VMValue::Unit),
                    Err(e) => err_vm(VMValue::Str(e)),
                },
            )
        }

        "AWS.dynamo_delete_item_raw" => {
            let mut it = args.into_iter();
            let table = vm_string(
                it.next().ok_or("dynamo_delete_item_raw: missing table")?,
                "AWS.dynamo_delete_item_raw",
            )?;
            let key_map = match it.next().ok_or("dynamo_delete_item_raw: missing key")? {
                VMValue::Record(m) => m,
                _ => return Err("dynamo_delete_item_raw: key must be a Map".to_string()),
            };
            let config = get_aws_config();
            let url = if let Some(ep) = &config.endpoint_url {
                ep.trim_end_matches('/').to_string()
            } else {
                format!("https://dynamodb.{}.amazonaws.com", config.region)
            };
            let key_json = map_to_dynamo_item(&key_map);
            let body = format!(r#"{{"TableName":"{}","Key":{}}}"#, table, key_json);
            Ok(
                match aws_post(
                    &config,
                    "dynamodb",
                    &url,
                    &body,
                    "application/x-amz-json-1.0",
                    Some("DynamoDB_20120810.DeleteItem"),
                ) {
                    Ok(_) => ok_vm(VMValue::Unit),
                    Err(e) => err_vm(VMValue::Str(e)),
                },
            )
        }

        "AWS.dynamo_query_raw" => {
            let mut it = args.into_iter();
            let table = vm_string(
                it.next().ok_or("dynamo_query_raw: missing table")?,
                "AWS.dynamo_query_raw",
            )?;
            let condition = vm_string(
                it.next().ok_or("dynamo_query_raw: missing condition")?,
                "AWS.dynamo_query_raw",
            )?;
            let vals_map = match it.next().ok_or("dynamo_query_raw: missing values")? {
                VMValue::Record(m) => m,
                _ => return Err("dynamo_query_raw: values must be a Map".to_string()),
            };
            let config = get_aws_config();
            let url = if let Some(ep) = &config.endpoint_url {
                ep.trim_end_matches('/').to_string()
            } else {
                format!("https://dynamodb.{}.amazonaws.com", config.region)
            };
            let expr_vals = map_to_dynamo_item(&vals_map);
            let body = format!(
                r#"{{"TableName":"{}","KeyConditionExpression":"{}","ExpressionAttributeValues":{}}}"#,
                table, condition, expr_vals
            );
            Ok(
                match aws_post(
                    &config,
                    "dynamodb",
                    &url,
                    &body,
                    "application/x-amz-json-1.0",
                    Some("DynamoDB_20120810.Query"),
                ) {
                    Ok(resp) => dynamo_list_response(&resp),
                    Err(e) => err_vm(VMValue::Str(e)),
                },
            )
        }

        "AWS.dynamo_scan_raw" => {
            let mut it = args.into_iter();
            let table = vm_string(
                it.next().ok_or("dynamo_scan_raw: missing table")?,
                "AWS.dynamo_scan_raw",
            )?;
            let config = get_aws_config();
            let url = if let Some(ep) = &config.endpoint_url {
                ep.trim_end_matches('/').to_string()
            } else {
                format!("https://dynamodb.{}.amazonaws.com", config.region)
            };
            let body = format!(r#"{{"TableName":"{}"}}"#, table);
            Ok(
                match aws_post(
                    &config,
                    "dynamodb",
                    &url,
                    &body,
                    "application/x-amz-json-1.0",
                    Some("DynamoDB_20120810.Scan"),
                ) {
                    Ok(resp) => dynamo_list_response(&resp),
                    Err(e) => err_vm(VMValue::Str(e)),
                },
            )
        }

        // ── AWS Secrets Manager (v14.4.0) ────────────────────────────────
        "AWS.secrets_get_raw" => {
            // AWS.secrets_get_raw(region: String, secret_name: String) -> Result<String, String>
            let mut it = args.into_iter();
            let region = vm_string(
                it.next().ok_or("secrets_get_raw: missing region")?,
                "AWS.secrets_get_raw",
            )?;
            let secret_name = vm_string(
                it.next().ok_or("secrets_get_raw: missing secret_name")?,
                "AWS.secrets_get_raw",
            )?;
            let config = get_aws_config();
            let url = if let Some(ep) = &config.endpoint_url {
                format!("{}/", ep.trim_end_matches('/'))
            } else {
                format!("https://secretsmanager.{}.amazonaws.com/", region)
            };
            let body = format!(
                r#"{{"SecretId":"{}"}}"#,
                secret_name.replace('"', "\\\"")
            );
            Ok(
                match aws_post(
                    &config,
                    "secretsmanager",
                    &url,
                    &body,
                    "application/x-amz-json-1.1",
                    Some("secretsmanager.GetSecretValue"),
                ) {
                    Ok(resp) => {
                        let parsed: serde_json::Value =
                            serde_json::from_str(&resp).unwrap_or(serde_json::Value::Null);
                        let secret = parsed
                            .get("SecretString")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        ok_vm(VMValue::Str(secret))
                    }
                    Err(e) => err_vm(VMValue::Str(e)),
                },
            )
        }

        // ── AWS DynamoDB PutItem with ConditionExpression (v15.1.0) ──────
        "AWS.dynamo_put_item_cond_raw" => {
            // AWS.dynamo_put_item_cond_raw(table, key_attr, key_val, ttl_attr, ttl_epoch, condition_expr)
            //   -> Result<Unit, String>
            // Returns err("nonce_already_used") on ConditionalCheckFailedException.
            let mut it = args.into_iter();
            let table = vm_string(
                it.next().ok_or("dynamo_put_item_cond_raw: missing table")?,
                "AWS.dynamo_put_item_cond_raw",
            )?;
            let key_attr = vm_string(
                it.next().ok_or("dynamo_put_item_cond_raw: missing key_attr")?,
                "AWS.dynamo_put_item_cond_raw",
            )?;
            let key_val = vm_string(
                it.next().ok_or("dynamo_put_item_cond_raw: missing key_val")?,
                "AWS.dynamo_put_item_cond_raw",
            )?;
            let ttl_attr = vm_string(
                it.next().ok_or("dynamo_put_item_cond_raw: missing ttl_attr")?,
                "AWS.dynamo_put_item_cond_raw",
            )?;
            let ttl_epoch: i64 = match it
                .next()
                .ok_or("dynamo_put_item_cond_raw: missing ttl_epoch")?
            {
                VMValue::Int(n) => n,
                VMValue::Str(s) => s.parse::<i64>().map_err(|_| {
                    "dynamo_put_item_cond_raw: ttl_epoch must be integer".to_string()
                })?,
                other => {
                    return Err(format!(
                        "dynamo_put_item_cond_raw: ttl_epoch must be Int or String, got {}",
                        vmvalue_type_name(&other)
                    ))
                }
            };
            let condition_expr = vm_string(
                it.next().ok_or("dynamo_put_item_cond_raw: missing condition_expr")?,
                "AWS.dynamo_put_item_cond_raw",
            )?;
            let config = get_aws_config();
            let url = if let Some(ep) = &config.endpoint_url {
                ep.trim_end_matches('/').to_string()
            } else {
                format!("https://dynamodb.{}.amazonaws.com", config.region)
            };
            let key_esc = key_val.replace('\\', "\\\\").replace('"', "\\\"");
            let cond_esc = condition_expr.replace('"', "\\\"");
            let body = format!(
                r#"{{"TableName":"{}","Item":{{"{}":{{"S":"{}"}},"{}":{{"N":"{}"}}}},"ConditionExpression":"{}"}}"#,
                table, key_attr, key_esc, ttl_attr, ttl_epoch, cond_esc
            );
            Ok(
                match aws_post(
                    &config,
                    "dynamodb",
                    &url,
                    &body,
                    "application/x-amz-json-1.0",
                    Some("DynamoDB_20120810.PutItem"),
                ) {
                    Ok(_) => ok_vm(VMValue::Unit),
                    Err(e) => {
                        if e.contains("ConditionalCheckFailedException") {
                            err_vm(VMValue::Str("nonce_already_used".to_string()))
                        } else {
                            err_vm(VMValue::Str(e))
                        }
                    }
                },
            )
        }

        // ── AWS ECS RunTask (v15.1.0) ──────────────────────────────────────
        "AWS.ecs_run_task_raw" => {
            // AWS.ecs_run_task_raw(cluster_arn, task_def_arn, subnets_csv, security_group, overrides_json)
            //   -> Result<String, String>  (returns task ARN on success)
            let mut it = args.into_iter();
            let cluster_arn = vm_string(
                it.next().ok_or("ecs_run_task_raw: missing cluster_arn")?,
                "AWS.ecs_run_task_raw",
            )?;
            let task_def_arn = vm_string(
                it.next().ok_or("ecs_run_task_raw: missing task_def_arn")?,
                "AWS.ecs_run_task_raw",
            )?;
            let subnets_csv = vm_string(
                it.next().ok_or("ecs_run_task_raw: missing subnets_csv")?,
                "AWS.ecs_run_task_raw",
            )?;
            let security_group = vm_string(
                it.next().ok_or("ecs_run_task_raw: missing security_group")?,
                "AWS.ecs_run_task_raw",
            )?;
            let overrides_json = vm_string(
                it.next().ok_or("ecs_run_task_raw: missing overrides_json")?,
                "AWS.ecs_run_task_raw",
            )?;
            let config = get_aws_config();
            let url = if let Some(ep) = &config.endpoint_url {
                format!("{}/", ep.trim_end_matches('/'))
            } else {
                format!("https://ecs.{}.amazonaws.com/", config.region)
            };
            let subnets_arr: String = {
                let parts: Vec<String> = subnets_csv
                    .split(',')
                    .map(|s| format!(r#""{}""#, s.trim().replace('"', "\\\"")))
                    .collect();
                format!("[{}]", parts.join(","))
            };
            let cluster_esc = cluster_arn.replace('"', "\\\"");
            let taskdef_esc = task_def_arn.replace('"', "\\\"");
            let sg_esc = security_group.replace('"', "\\\"");
            let body = format!(
                r#"{{"cluster":"{}","taskDefinition":"{}","launchType":"FARGATE","networkConfiguration":{{"awsvpcConfiguration":{{"subnets":{},"securityGroups":["{}"],"assignPublicIp":"ENABLED"}}}},"overrides":{}}}"#,
                cluster_esc, taskdef_esc, subnets_arr, sg_esc, overrides_json
            );
            Ok(
                match aws_post(
                    &config,
                    "ecs",
                    &url,
                    &body,
                    "application/x-amz-json-1.1",
                    Some("AmazonEC2ContainerServiceV20141113.RunTask"),
                ) {
                    Ok(resp) => {
                        let parsed: serde_json::Value =
                            serde_json::from_str(&resp).unwrap_or(serde_json::Value::Null);
                        let task_arn = parsed
                            .get("tasks")
                            .and_then(|t| t.as_array())
                            .and_then(|arr| arr.first())
                            .and_then(|task| task.get("taskArn"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        if task_arn.is_empty() {
                            let failures = parsed
                                .get("failures")
                                .and_then(|f| f.as_array())
                                .map(|arr| {
                                    arr.iter()
                                        .map(|f| {
                                            f.get("reason")
                                                .and_then(|r| r.as_str())
                                                .unwrap_or("unknown")
                                        })
                                        .collect::<Vec<_>>()
                                        .join(", ")
                                })
                                .filter(|s| !s.is_empty())
                                .unwrap_or(resp);
                            err_vm(VMValue::Str(failures))
                        } else {
                            ok_vm(VMValue::Str(task_arn))
                        }
                    }
                    Err(e) => err_vm(VMValue::Str(e)),
                },
            )
        }

        // ── AWS KMS GetPublicKey (v15.1.5) ────────────────────────────────
        // AWS.kms_get_public_key_raw(region: String, key_id: String) -> Result<String, String>
        // Calls KMS GetPublicKey, converts DER → PEM and returns the PEM string.
        "AWS.kms_get_public_key_raw" => {
            let mut it = args.into_iter();
            let region = vm_string(
                it.next().ok_or("kms_get_public_key_raw: missing region")?,
                "AWS.kms_get_public_key_raw",
            )?;
            let key_id = vm_string(
                it.next().ok_or("kms_get_public_key_raw: missing key_id")?,
                "AWS.kms_get_public_key_raw",
            )?;

            let mut config = get_aws_config();
            // Override region with the explicitly passed region argument
            config.region = region.clone();

            let url = if let Some(ep) = &config.endpoint_url {
                format!("{}/", ep.trim_end_matches('/'))
            } else {
                format!("https://kms.{}.amazonaws.com/", region)
            };

            let key_id_esc = key_id.replace('"', "\\\"");
            let body = format!(r#"{{"KeyId":"{}"}}"#, key_id_esc);

            match aws_post(
                &config,
                "kms",
                &url,
                &body,
                "application/x-amz-json-1.1",
                Some("TrentService.GetPublicKey"),
            ) {
                Ok(resp) => {
                    let parsed: serde_json::Value = serde_json::from_str(&resp)
                        .map_err(|e| format!("kms_get_public_key_raw: parse JSON: {e}"))?;
                    let der_b64 = parsed
                        .get("PublicKey")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| format!("kms_get_public_key_raw: missing PublicKey in response: {resp}"))?;
                    // KMS returns base64-encoded DER.
                    // PEM requires base64 wrapped at 64 chars per line.
                    let b64_clean: String = der_b64
                        .trim()
                        .chars()
                        .filter(|c| !c.is_whitespace())
                        .collect();
                    let b64_wrapped: String = b64_clean
                        .as_bytes()
                        .chunks(64)
                        .map(|c| std::str::from_utf8(c).unwrap_or(""))
                        .collect::<Vec<_>>()
                        .join("\n");
                    let pem = format!(
                        "-----BEGIN PUBLIC KEY-----\n{}\n-----END PUBLIC KEY-----\n",
                        b64_wrapped
                    );
                    Ok(ok_vm(VMValue::Str(pem)))
                }
                Err(e) => Ok(err_vm(VMValue::Str(e))),
            }
        }

        // ── GCP BigQuery primitives (v15.2.0) ────────────────────────────
        // 共通: GOOGLE_APPLICATION_CREDENTIALS → RS256 JWT → Bearer token
        // TODO(v28.x): query_raw / execute_raw / infer_table_raw に wasm32 ガードを追加する。
        //              v27.4.0 追加の connect_raw 等（下方）はガード済みのため非対称になっている。
        //              v28.x の google-cloud-bigquery クレート統合時にまとめて対処すること。

        "BigQuery.query_raw" => {
            // (project_id, dataset, sql, params_json) -> Result<String, String>
            let mut it = args.into_iter();
            let project_id = vm_string(it.next().ok_or("BigQuery.query_raw: missing project_id")?, "BigQuery.query_raw")?;
            let _dataset   = vm_string(it.next().ok_or("BigQuery.query_raw: missing dataset")?,    "BigQuery.query_raw")?;
            let sql        = vm_string(it.next().ok_or("BigQuery.query_raw: missing sql")?,        "BigQuery.query_raw")?;
            let _params    = vm_string(it.next().ok_or("BigQuery.query_raw: missing params_json")?, "BigQuery.query_raw")?;

            let token = match gcp_get_access_token() {
                Ok(t) => t,
                Err(e) => return Ok(err_vm(VMValue::Str(e))),
            };

            let url = format!(
                "https://bigquery.googleapis.com/bigquery/v2/projects/{}/queries",
                project_id
            );
            let body = serde_json::json!({
                "query": sql,
                "useLegacySql": false,
                "maxResults": 10000,
                "timeoutMs": 30000,
            });

            let resp_text = match ureq::post(&url)
                .set("Authorization", &format!("Bearer {}", token))
                .set("Content-Type", "application/json")
                .send_string(&body.to_string())
            {
                Ok(r) => match r.into_string() {
                    Ok(s) => s,
                    Err(e) => return Ok(err_vm(VMValue::Str(format!("BigQuery.query_raw: read body: {e}")))),
                },
                Err(ureq::Error::Status(code, resp)) => {
                    let body = resp.into_string().unwrap_or_default();
                    return Ok(err_vm(VMValue::Str(format!("BigQuery.query_raw: HTTP {code}: {body}"))));
                }
                Err(e) => return Ok(err_vm(VMValue::Str(format!("BigQuery.query_raw: {e}")))),
            };

            let json: serde_json::Value = match serde_json::from_str(&resp_text) {
                Ok(j) => j,
                Err(e) => return Ok(err_vm(VMValue::Str(format!("BigQuery.query_raw: parse: {e}")))),
            };

            if let Some(err) = json.get("error") {
                return Ok(err_vm(VMValue::Str(format!("BigQuery.query_raw: {err}"))));
            }

            let rows   = json.get("rows").cloned().unwrap_or(serde_json::Value::Array(vec![]));
            let schema = json.get("schema").cloned().unwrap_or(serde_json::Value::Null);
            let result = serde_json::json!({"schema": schema, "rows": rows}).to_string();

            Ok(ok_vm(VMValue::Str(result)))
        }

        "BigQuery.execute_raw" => {
            // (project_id, dataset, sql, params_json) -> Result<Int, String>
            // DML via Jobs API → poll → numDmlAffectedRows
            let mut it = args.into_iter();
            let project_id = vm_string(it.next().ok_or("BigQuery.execute_raw: missing project_id")?, "BigQuery.execute_raw")?;
            let _dataset   = vm_string(it.next().ok_or("BigQuery.execute_raw: missing dataset")?,    "BigQuery.execute_raw")?;
            let sql        = vm_string(it.next().ok_or("BigQuery.execute_raw: missing sql")?,        "BigQuery.execute_raw")?;
            let _params    = vm_string(it.next().ok_or("BigQuery.execute_raw: missing params_json")?, "BigQuery.execute_raw")?;

            let token = match gcp_get_access_token() {
                Ok(t) => t,
                Err(e) => return Ok(err_vm(VMValue::Str(e))),
            };

            // ジョブ作成
            let jobs_url = format!(
                "https://bigquery.googleapis.com/bigquery/v2/projects/{}/jobs",
                project_id
            );
            let bq_location = std::env::var("BQ_LOCATION").unwrap_or_else(|_| "asia-northeast1".to_string());
            let job_body = serde_json::json!({
                "configuration": {
                    "query": {
                        "query": sql,
                        "useLegacySql": false
                    }
                },
                "jobReference": {
                    "projectId": &project_id,
                    "location": &bq_location
                }
            });

            let resp_text = match ureq::post(&jobs_url)
                .set("Authorization", &format!("Bearer {}", token))
                .set("Content-Type", "application/json")
                .send_string(&job_body.to_string())
            {
                Ok(r) => match r.into_string() {
                    Ok(s) => s,
                    Err(e) => return Ok(err_vm(VMValue::Str(format!("BigQuery.execute_raw: read body: {e}")))),
                },
                Err(ureq::Error::Status(code, resp)) => {
                    let body = resp.into_string().unwrap_or_default();
                    return Ok(err_vm(VMValue::Str(format!("BigQuery.execute_raw: HTTP {code}: {body}"))));
                }
                Err(e) => return Ok(err_vm(VMValue::Str(format!("BigQuery.execute_raw: {e}")))),
            };

            let json: serde_json::Value = match serde_json::from_str(&resp_text) {
                Ok(j) => j,
                Err(e) => return Ok(err_vm(VMValue::Str(format!("BigQuery.execute_raw: parse job: {e}")))),
            };

            if let Some(err) = json.get("error") {
                return Ok(err_vm(VMValue::Str(format!("BigQuery.execute_raw: {err}"))));
            }

            let job_id = match json.pointer("/jobReference/jobId").and_then(|v| v.as_str()) {
                Some(id) => id.to_string(),
                None => return Ok(err_vm(VMValue::Str("BigQuery.execute_raw: missing jobId".into()))),
            };
            let job_location = json.pointer("/jobReference/location")
                .or_else(|| json.pointer("/configuration/query/defaultDataset/datasetId"))
                .and_then(|v| v.as_str())
                .unwrap_or("US")
                .to_string();

            // 完了まで polling（最大 60 回 × 1 秒 = 60 秒）
            let get_url = format!(
                "https://bigquery.googleapis.com/bigquery/v2/projects/{}/jobs/{}?location={}",
                project_id, job_id, job_location
            );
            for _ in 0..60 {
                std::thread::sleep(std::time::Duration::from_secs(1));
                let poll_text = match ureq::get(&get_url)
                    .set("Authorization", &format!("Bearer {}", token))
                    .call()
                {
                    Ok(r) => r.into_string().unwrap_or_default(),
                    Err(e) => return Ok(err_vm(VMValue::Str(format!("BigQuery.execute_raw: poll: {e}")))),
                };
                let poll: serde_json::Value = serde_json::from_str(&poll_text)
                    .unwrap_or(serde_json::Value::Null);

                if let Some(err) = poll.get("status").and_then(|s| s.get("errorResult")) {
                    return Ok(err_vm(VMValue::Str(format!("BigQuery.execute_raw: job error: {err}"))));
                }

                let state = poll.pointer("/status/state").and_then(|v| v.as_str()).unwrap_or("");
                if state == "DONE" {
                    let affected = poll
                        .pointer("/statistics/query/numDmlAffectedRows")
                        .and_then(|v| v.as_str().and_then(|s| s.parse::<i64>().ok())
                            .or_else(|| v.as_i64()))
                        .unwrap_or(0);
                    return Ok(ok_vm(VMValue::Int(affected)));
                }
            }

            Ok(err_vm(VMValue::Str("BigQuery.execute_raw: job timed out".into())))
        }

        "BigQuery.infer_table_raw" => {
            // (project_id, dataset, table) -> Result<String, String>
            let mut it = args.into_iter();
            let project_id = vm_string(it.next().ok_or("BigQuery.infer_table_raw: missing project_id")?, "BigQuery.infer_table_raw")?;
            let dataset    = vm_string(it.next().ok_or("BigQuery.infer_table_raw: missing dataset")?,    "BigQuery.infer_table_raw")?;
            let table      = vm_string(it.next().ok_or("BigQuery.infer_table_raw: missing table")?,      "BigQuery.infer_table_raw")?;

            let token = match gcp_get_access_token() {
                Ok(t) => t,
                Err(e) => return Ok(err_vm(VMValue::Str(e))),
            };

            let sql = format!(
                "SELECT column_name, data_type, is_nullable \
                 FROM `{project_id}.{dataset}.INFORMATION_SCHEMA.COLUMNS` \
                 WHERE table_name = '{table}' \
                 ORDER BY ordinal_position"
            );
            let url = format!(
                "https://bigquery.googleapis.com/bigquery/v2/projects/{}/queries",
                project_id
            );
            let body = serde_json::json!({
                "query": sql,
                "useLegacySql": false,
                "maxResults": 1000,
                "timeoutMs": 30000,
            });

            let resp_text = match ureq::post(&url)
                .set("Authorization", &format!("Bearer {}", token))
                .set("Content-Type", "application/json")
                .send_string(&body.to_string())
            {
                Ok(r) => match r.into_string() {
                    Ok(s) => s,
                    Err(e) => return Ok(err_vm(VMValue::Str(format!("BigQuery.infer_table_raw: read: {e}")))),
                },
                Err(ureq::Error::Status(code, resp)) => {
                    let body = resp.into_string().unwrap_or_default();
                    return Ok(err_vm(VMValue::Str(format!("BigQuery.infer_table_raw: HTTP {code}: {body}"))));
                }
                Err(e) => return Ok(err_vm(VMValue::Str(format!("BigQuery.infer_table_raw: {e}")))),
            };

            let json: serde_json::Value = match serde_json::from_str(&resp_text) {
                Ok(j) => j,
                Err(e) => return Ok(err_vm(VMValue::Str(format!("BigQuery.infer_table_raw: parse: {e}")))),
            };

            if let Some(err) = json.get("error") {
                return Ok(err_vm(VMValue::Str(format!("BigQuery.infer_table_raw: {err}"))));
            }

            Ok(ok_vm(VMValue::Str(resp_text)))
        }

        // ── BigQuery connect-based primitives (v27.4.0) ───────────────────
        // connect-based API（!Db エフェクト、DWH 統一）。v15.2.0 の query_raw / execute_raw は残す。
        // TODO(v28.x): google-cloud-bigquery クレートを使った実接続に移行予定。
        //              _config のフォーマットは現在 "project:X,dataset:Y" のカンマ区切り文字列。
        //              v28.x で fav.toml [bigquery] セクション統合に変更予定（フォーマット破壊的変更あり）。
        //              _config を BigQuery クライアントの初期化（project_id / credentials）に渡す。

        #[cfg(not(target_arch = "wasm32"))]
        "BigQuery.connect_raw" => {
            // (config: String) -> Result<String, String> !Db
            let mut it = args.into_iter();
            let _config = vm_string(it.next().ok_or("BigQuery.connect_raw: missing config")?, "BigQuery.connect_raw")?;
            Ok(ok_vm(VMValue::Str("bigquery-stub-conn".into())))
        }
        #[cfg(target_arch = "wasm32")]
        "BigQuery.connect_raw" => Ok(err_vm(VMValue::Str("BigQuery not supported on wasm32".into()))),

        #[cfg(not(target_arch = "wasm32"))]
        "BigQuery.conn_query_raw" => {
            // (conn: String, sql: String) -> Result<String, String> !Db
            let mut it = args.into_iter();
            let _conn = vm_string(it.next().ok_or("BigQuery.conn_query_raw: missing conn")?, "BigQuery.conn_query_raw")?;
            let _sql  = vm_string(it.next().ok_or("BigQuery.conn_query_raw: missing sql")?,  "BigQuery.conn_query_raw")?;
            Ok(ok_vm(VMValue::Str("[]".into())))
        }
        #[cfg(target_arch = "wasm32")]
        "BigQuery.conn_query_raw" => Ok(err_vm(VMValue::Str("BigQuery not supported on wasm32".into()))),

        #[cfg(not(target_arch = "wasm32"))]
        "BigQuery.insert_raw" => {
            // (conn: String, table: String, rows: String) -> Result<Unit, String> !Db
            let mut it = args.into_iter();
            let _conn  = vm_string(it.next().ok_or("BigQuery.insert_raw: missing conn")?,  "BigQuery.insert_raw")?;
            let _table = vm_string(it.next().ok_or("BigQuery.insert_raw: missing table")?, "BigQuery.insert_raw")?;
            let _rows  = vm_string(it.next().ok_or("BigQuery.insert_raw: missing rows")?,  "BigQuery.insert_raw")?;
            Ok(ok_vm(VMValue::Unit))
        }
        #[cfg(target_arch = "wasm32")]
        "BigQuery.insert_raw" => Ok(err_vm(VMValue::Str("BigQuery not supported on wasm32".into()))),

        #[cfg(not(target_arch = "wasm32"))]
        "BigQuery.load_from_gcs_raw" => {
            // (conn: String, table: String, gcs_uri: String, format: String) -> Result<Unit, String> !Db
            let mut it = args.into_iter();
            let _conn    = vm_string(it.next().ok_or("BigQuery.load_from_gcs_raw: missing conn")?,    "BigQuery.load_from_gcs_raw")?;
            let _table   = vm_string(it.next().ok_or("BigQuery.load_from_gcs_raw: missing table")?,   "BigQuery.load_from_gcs_raw")?;
            let _gcs_uri = vm_string(it.next().ok_or("BigQuery.load_from_gcs_raw: missing gcs_uri")?, "BigQuery.load_from_gcs_raw")?;
            let _format  = vm_string(it.next().ok_or("BigQuery.load_from_gcs_raw: missing format")?,  "BigQuery.load_from_gcs_raw")?;
            Ok(ok_vm(VMValue::Unit))
        }
        #[cfg(target_arch = "wasm32")]
        "BigQuery.load_from_gcs_raw" => Ok(err_vm(VMValue::Str("BigQuery not supported on wasm32".into()))),

        #[cfg(not(target_arch = "wasm32"))]
        "BigQuery.create_table_raw" => {
            // (conn: String, table: String, schema: String) -> Result<Unit, String> !Db
            let mut it = args.into_iter();
            let _conn   = vm_string(it.next().ok_or("BigQuery.create_table_raw: missing conn")?,   "BigQuery.create_table_raw")?;
            let _table  = vm_string(it.next().ok_or("BigQuery.create_table_raw: missing table")?,  "BigQuery.create_table_raw")?;
            let _schema = vm_string(it.next().ok_or("BigQuery.create_table_raw: missing schema")?, "BigQuery.create_table_raw")?;
            Ok(ok_vm(VMValue::Unit))
        }
        #[cfg(target_arch = "wasm32")]
        "BigQuery.create_table_raw" => Ok(err_vm(VMValue::Str("BigQuery not supported on wasm32".into()))),

        // ── Kafka / MSK primitives (v15.4.0) ──────────────────────────────
        // TODO(v25.7.0): wasm32 ガードを既存 2 primitive にも追加（新規 3 件と対称化）

        #[cfg(not(target_arch = "wasm32"))]
        "Kafka.produce_raw" => {
            // (conn_str: String [= brokers addr from KafkaConn], topic: String, key: String, value: String) -> Result<Unit, String>
            // Rune 側から呼ばれる際は conn: KafkaConn の内部 String（ブローカーアドレス）が渡される（名目型ラッパー）。
            let mut it = args.into_iter();
            let brokers_arg = vm_string(it.next().ok_or("Kafka.produce_raw: missing brokers")?,  "Kafka.produce_raw")?;
            let topic       = vm_string(it.next().ok_or("Kafka.produce_raw: missing topic")?,    "Kafka.produce_raw")?;
            let key_str     = vm_string(it.next().ok_or("Kafka.produce_raw: missing key")?,      "Kafka.produce_raw")?;
            let value_str   = vm_string(it.next().ok_or("Kafka.produce_raw: missing value")?,    "Kafka.produce_raw")?;

            let brokers = kafka_resolve_brokers(&brokers_arg);
            match kafka_produce_sync(&brokers, &topic, &key_str, &value_str) {
                Ok(()) => Ok(ok_vm(VMValue::Unit)),
                Err(e) => Ok(err_vm(VMValue::Str(e))),
            }
        }
        #[cfg(target_arch = "wasm32")]
        "Kafka.produce_raw" => Ok(err_vm(VMValue::Str("Kafka not supported on wasm32".into()))),

        #[cfg(not(target_arch = "wasm32"))]
        "Kafka.consume_one_raw" => {
            // (conn_str: String [= brokers addr from KafkaConn], topic: String, group_id: String) -> Result<String, String>
            // Rune 側から呼ばれる際は conn: KafkaConn の内部 String（ブローカーアドレス）が渡される（名目型ラッパー）。
            let mut it = args.into_iter();
            let brokers_arg = vm_string(it.next().ok_or("Kafka.consume_one_raw: missing brokers")?,  "Kafka.consume_one_raw")?;
            let topic       = vm_string(it.next().ok_or("Kafka.consume_one_raw: missing topic")?,    "Kafka.consume_one_raw")?;
            let _group_id   = vm_string(it.next().ok_or("Kafka.consume_one_raw: missing group_id")?, "Kafka.consume_one_raw")?;

            let brokers = kafka_resolve_brokers(&brokers_arg);
            match kafka_consume_one_sync(&brokers, &topic) {
                Ok(msg) => Ok(ok_vm(VMValue::Str(msg))),
                Err(e)  => Ok(err_vm(VMValue::Str(e))),
            }
        }
        #[cfg(target_arch = "wasm32")]
        "Kafka.consume_one_raw" => Ok(err_vm(VMValue::Str("Kafka not supported on wasm32".into()))),

        // ── Kafka 新規 primitives (v25.7.0) ──────────────────────────────
        #[cfg(not(target_arch = "wasm32"))]
        "Kafka.connect_raw" => {
            // (brokers: String) -> Result<String, String>
            let mut it = args.into_iter();
            let brokers_arg = vm_string(it.next().ok_or("Kafka.connect_raw: missing brokers")?, "Kafka.connect_raw")?;
            let brokers = kafka_resolve_brokers(&brokers_arg);
            match kafka_connect_sync(&brokers) {
                Ok(()) => Ok(ok_vm(VMValue::Str(brokers))),
                Err(e)  => Ok(err_vm(VMValue::Str(e))),
            }
        }
        #[cfg(target_arch = "wasm32")]
        "Kafka.connect_raw" => Ok(err_vm(VMValue::Str("Kafka not supported on wasm32".into()))),

        #[cfg(not(target_arch = "wasm32"))]
        "Kafka.consume_batch_raw" => {
            // (brokers: String, topic: String, max_count: Int) -> Result<String, String>
            let mut it = args.into_iter();
            let brokers_arg = vm_string(it.next().ok_or("Kafka.consume_batch_raw: missing brokers")?, "Kafka.consume_batch_raw")?;
            let topic       = vm_string(it.next().ok_or("Kafka.consume_batch_raw: missing topic")?,   "Kafka.consume_batch_raw")?;
            let max_count   = match it.next().ok_or("Kafka.consume_batch_raw: missing max_count")? {
                VMValue::Int(n) => n,
                _ => return Err("Kafka.consume_batch_raw: max_count must be Int".to_string()),
            };
            let brokers = kafka_resolve_brokers(&brokers_arg);
            match kafka_consume_batch_sync(&brokers, &topic, max_count) {
                Ok(s)  => Ok(ok_vm(VMValue::Str(s))),
                Err(e) => Ok(err_vm(VMValue::Str(e))),
            }
        }
        #[cfg(target_arch = "wasm32")]
        "Kafka.consume_batch_raw" => Ok(err_vm(VMValue::Str("Kafka not supported on wasm32".into()))),

        #[cfg(not(target_arch = "wasm32"))]
        "Kafka.create_topic_raw" => {
            // (brokers: String, topic: String, partitions: Int) -> Result<Unit, String>
            let mut it = args.into_iter();
            let brokers_arg = vm_string(it.next().ok_or("Kafka.create_topic_raw: missing brokers")?,  "Kafka.create_topic_raw")?;
            let topic       = vm_string(it.next().ok_or("Kafka.create_topic_raw: missing topic")?,    "Kafka.create_topic_raw")?;
            let partitions  = match it.next().ok_or("Kafka.create_topic_raw: missing partitions")? {
                VMValue::Int(n) => n as i32,
                _ => return Err("Kafka.create_topic_raw: partitions must be Int".to_string()),
            };
            let brokers = kafka_resolve_brokers(&brokers_arg);
            match kafka_create_topic_sync(&brokers, &topic, partitions) {
                Ok(())  => Ok(ok_vm(VMValue::Unit)),
                Err(e)  => Ok(err_vm(VMValue::Str(e))),
            }
        }
        #[cfg(target_arch = "wasm32")]
        "Kafka.create_topic_raw" => Ok(err_vm(VMValue::Str("Kafka not supported on wasm32".into()))),

        // ── Kinesis primitives (v26.1.0) ─────────────────────────────────
        #[cfg(not(target_arch = "wasm32"))]
        "Kinesis.connect_raw" => {
            // (endpoint: String) -> Result<String, String>
            // "" → KINESIS_ENDPOINT env var → "http://localhost:4566"
            let mut it = args.into_iter();
            let endpoint = vm_string(it.next().ok_or("Kinesis.connect_raw: missing endpoint")?, "Kinesis.connect_raw")?;
            let ep = if endpoint.is_empty() {
                std::env::var("KINESIS_ENDPOINT").unwrap_or_else(|_| "http://localhost:4566".to_string())
            } else {
                endpoint
            };
            Ok(ok_vm(VMValue::Str(ep)))
        }
        #[cfg(target_arch = "wasm32")]
        "Kinesis.connect_raw" => Ok(err_vm(VMValue::Str("Kinesis not supported on wasm32".into()))),

        #[cfg(not(target_arch = "wasm32"))]
        "Kinesis.put_record_raw" => {
            // (conn: String, stream: String, key: String, data: String) -> Result<String, String>
            // Returns sequence number stub "seq-0001"
            let mut it = args.into_iter();
            let _conn   = vm_string(it.next().ok_or("Kinesis.put_record_raw: missing conn")?,   "Kinesis.put_record_raw")?;
            let _stream = vm_string(it.next().ok_or("Kinesis.put_record_raw: missing stream")?, "Kinesis.put_record_raw")?;
            let _key    = vm_string(it.next().ok_or("Kinesis.put_record_raw: missing key")?,    "Kinesis.put_record_raw")?;
            let _data   = vm_string(it.next().ok_or("Kinesis.put_record_raw: missing data")?,   "Kinesis.put_record_raw")?;
            Ok(ok_vm(VMValue::Str("seq-0001".to_string())))
        }
        #[cfg(target_arch = "wasm32")]
        "Kinesis.put_record_raw" => Ok(err_vm(VMValue::Str("Kinesis not supported on wasm32".into()))),

        #[cfg(not(target_arch = "wasm32"))]
        "Kinesis.put_records_raw" => {
            // (conn: String, stream: String, records: List) -> Result<Int, String>
            // Returns count of records successfully sent (stub: all succeed)
            let mut it = args.into_iter();
            let _conn    = vm_string(it.next().ok_or("Kinesis.put_records_raw: missing conn")?,   "Kinesis.put_records_raw")?;
            let _stream  = vm_string(it.next().ok_or("Kinesis.put_records_raw: missing stream")?, "Kinesis.put_records_raw")?;
            let records  = it.next().ok_or("Kinesis.put_records_raw: missing records")?;
            // TODO: when replacing stub with real AWS SDK call, validate each element
            // is a VMValue::Record with { partition_key, data, sequence_num } fields
            let count = match records {
                VMValue::List(v) => v.len() as i64,
                _ => return Err("Kinesis.put_records_raw: records must be a List".to_string()),
            };
            Ok(ok_vm(VMValue::Int(count)))
        }
        #[cfg(target_arch = "wasm32")]
        "Kinesis.put_records_raw" => Ok(err_vm(VMValue::Str("Kinesis not supported on wasm32".into()))),

        #[cfg(not(target_arch = "wasm32"))]
        "Kinesis.get_shard_iterator_raw" => {
            // (conn: String, stream: String, shard_id: String, iter_type: String) -> Result<String, String>
            // Returns a stub iterator string
            let mut it = args.into_iter();
            let _conn      = vm_string(it.next().ok_or("Kinesis.get_shard_iterator_raw: missing conn")?,      "Kinesis.get_shard_iterator_raw")?;
            let stream     = vm_string(it.next().ok_or("Kinesis.get_shard_iterator_raw: missing stream")?,    "Kinesis.get_shard_iterator_raw")?;
            let _shard_id  = vm_string(it.next().ok_or("Kinesis.get_shard_iterator_raw: missing shard_id")?,  "Kinesis.get_shard_iterator_raw")?;
            let iter_type  = vm_string(it.next().ok_or("Kinesis.get_shard_iterator_raw: missing iter_type")?, "Kinesis.get_shard_iterator_raw")?;
            let iterator = format!("shard-iter-{stream}-{iter_type}");
            Ok(ok_vm(VMValue::Str(iterator)))
        }
        #[cfg(target_arch = "wasm32")]
        "Kinesis.get_shard_iterator_raw" => Ok(err_vm(VMValue::Str("Kinesis not supported on wasm32".into()))),

        #[cfg(not(target_arch = "wasm32"))]
        "Kinesis.get_records_raw" => {
            // (conn: String, iterator: String, limit: Int) -> Result<String, String>
            // Returns JSON array string of KinesisRecord objects (stub: empty array)
            let mut it = args.into_iter();
            let _conn     = vm_string(it.next().ok_or("Kinesis.get_records_raw: missing conn")?,     "Kinesis.get_records_raw")?;
            let _iterator = vm_string(it.next().ok_or("Kinesis.get_records_raw: missing iterator")?, "Kinesis.get_records_raw")?;
            let limit     = match it.next().ok_or("Kinesis.get_records_raw: missing limit")? {
                VMValue::Int(n) => n,
                _ => return Err("Kinesis.get_records_raw: limit must be Int".to_string()),
            };
            if limit <= 0 {
                return Ok(err_vm(VMValue::Str("Kinesis.get_records_raw: limit must be > 0".to_string())));
            }
            Ok(ok_vm(VMValue::Str("[]".to_string())))
        }
        #[cfg(target_arch = "wasm32")]
        "Kinesis.get_records_raw" => Ok(err_vm(VMValue::Str("Kinesis not supported on wasm32".into()))),

        // ── NATS primitives (v26.2.0) ─────────────────────────────────────
        #[cfg(not(target_arch = "wasm32"))]
        "NATS.connect_raw" => {
            // (url: String) -> Result<NatsConn, String>
            // Stub: returns the URL string as VMValue::Str (NatsConn nominal type wrapper).
            // TODO: when replacing stub with real nats crate call, replace VMValue::Str
            // with a connection handle (e.g., store in a global registry keyed by ID string).
            let mut it = args.into_iter();
            let url = vm_string(it.next().ok_or("NATS.connect_raw: missing url")?, "NATS.connect_raw")?;
            let u = if url.is_empty() {
                std::env::var("NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".to_string())
            } else { url };
            Ok(ok_vm(VMValue::Str(u)))
        }
        #[cfg(target_arch = "wasm32")]
        "NATS.connect_raw" => Ok(err_vm(VMValue::Str("NATS not supported on wasm32".into()))),

        #[cfg(not(target_arch = "wasm32"))]
        "NATS.publish_raw" => {
            // (conn: NatsConn, subject: String, payload: String) -> Result<Unit, String>
            let mut it = args.into_iter();
            let _conn    = vm_string(it.next().ok_or("NATS.publish_raw: missing conn")?,    "NATS.publish_raw")?;
            let _subject = vm_string(it.next().ok_or("NATS.publish_raw: missing subject")?, "NATS.publish_raw")?;
            let _payload = vm_string(it.next().ok_or("NATS.publish_raw: missing payload")?, "NATS.publish_raw")?;
            Ok(ok_vm(VMValue::Unit))
        }
        #[cfg(target_arch = "wasm32")]
        "NATS.publish_raw" => Ok(err_vm(VMValue::Str("NATS not supported on wasm32".into()))),

        #[cfg(not(target_arch = "wasm32"))]
        "NATS.subscribe_raw" => {
            // (conn: NatsConn, subject: String) -> Result<String, String>
            // Stub: returns empty JSON object (no live server connection)
            let mut it = args.into_iter();
            let _conn    = vm_string(it.next().ok_or("NATS.subscribe_raw: missing conn")?,    "NATS.subscribe_raw")?;
            let _subject = vm_string(it.next().ok_or("NATS.subscribe_raw: missing subject")?, "NATS.subscribe_raw")?;
            Ok(ok_vm(VMValue::Str("{}".to_string())))
        }
        #[cfg(target_arch = "wasm32")]
        "NATS.subscribe_raw" => Ok(err_vm(VMValue::Str("NATS not supported on wasm32".into()))),

        #[cfg(not(target_arch = "wasm32"))]
        "NATS.jetstream_publish_raw" => {
            // (conn: NatsConn, stream: String, payload: String) -> Result<String, String>
            // Stub: returns sequence number string
            let mut it = args.into_iter();
            let _conn    = vm_string(it.next().ok_or("NATS.jetstream_publish_raw: missing conn")?,    "NATS.jetstream_publish_raw")?;
            let _stream  = vm_string(it.next().ok_or("NATS.jetstream_publish_raw: missing stream")?,  "NATS.jetstream_publish_raw")?;
            let _payload = vm_string(it.next().ok_or("NATS.jetstream_publish_raw: missing payload")?, "NATS.jetstream_publish_raw")?;
            Ok(ok_vm(VMValue::Str("seq-js-0001".to_string())))
        }
        #[cfg(target_arch = "wasm32")]
        "NATS.jetstream_publish_raw" => Ok(err_vm(VMValue::Str("NATS not supported on wasm32".into()))),

        #[cfg(not(target_arch = "wasm32"))]
        "NATS.jetstream_consume_raw" => {
            // (conn: NatsConn, stream: String, consumer: String) -> Result<String, String>
            // Stub: returns empty JSON array
            let mut it = args.into_iter();
            let _conn     = vm_string(it.next().ok_or("NATS.jetstream_consume_raw: missing conn")?,     "NATS.jetstream_consume_raw")?;
            let _stream   = vm_string(it.next().ok_or("NATS.jetstream_consume_raw: missing stream")?,   "NATS.jetstream_consume_raw")?;
            let _consumer = vm_string(it.next().ok_or("NATS.jetstream_consume_raw: missing consumer")?, "NATS.jetstream_consume_raw")?;
            Ok(ok_vm(VMValue::Str("[]".to_string())))
        }
        #[cfg(target_arch = "wasm32")]
        "NATS.jetstream_consume_raw" => Ok(err_vm(VMValue::Str("NATS not supported on wasm32".into()))),

        // ── RabbitMQ primitives (v26.3.0) ─────────────────────────────────
        #[cfg(not(target_arch = "wasm32"))]
        "RabbitMQ.connect_raw" => {
            // (url: String) -> Result<RabbitConn, String>
            // Stub: returns URL string as VMValue::Str (RabbitConn nominal type wrapper).
            // TODO: when replacing stub with real AMQP connection, replace VMValue::Str
            // with a connection handle stored in a global registry keyed by ID string.
            let mut it = args.into_iter();
            let url = vm_string(it.next().ok_or("RabbitMQ.connect_raw: missing url")?, "RabbitMQ.connect_raw")?;
            let u = if url.is_empty() {
                std::env::var("RABBITMQ_URL").unwrap_or_else(|_| "amqp://guest:guest@localhost:5672".to_string())
            } else { url };
            Ok(ok_vm(VMValue::Str(u)))
        }
        #[cfg(target_arch = "wasm32")]
        "RabbitMQ.connect_raw" => Ok(err_vm(VMValue::Str("RabbitMQ not supported on wasm32".into()))),

        #[cfg(not(target_arch = "wasm32"))]
        "RabbitMQ.declare_exchange_raw" => {
            // (conn: RabbitConn, name: String, ex_type: String) -> Result<Unit, String>
            // Stub: validates args and returns Unit
            let mut it = args.into_iter();
            let _conn    = vm_string(it.next().ok_or("RabbitMQ.declare_exchange_raw: missing conn")?,    "RabbitMQ.declare_exchange_raw")?;
            let _name    = vm_string(it.next().ok_or("RabbitMQ.declare_exchange_raw: missing name")?,    "RabbitMQ.declare_exchange_raw")?;
            let _ex_type = vm_string(it.next().ok_or("RabbitMQ.declare_exchange_raw: missing ex_type")?, "RabbitMQ.declare_exchange_raw")?;
            Ok(ok_vm(VMValue::Unit))
        }
        #[cfg(target_arch = "wasm32")]
        "RabbitMQ.declare_exchange_raw" => Ok(err_vm(VMValue::Str("RabbitMQ not supported on wasm32".into()))),

        #[cfg(not(target_arch = "wasm32"))]
        "RabbitMQ.declare_queue_raw" => {
            // (conn: RabbitConn, name: String) -> Result<Unit, String>
            let mut it = args.into_iter();
            let _conn = vm_string(it.next().ok_or("RabbitMQ.declare_queue_raw: missing conn")?, "RabbitMQ.declare_queue_raw")?;
            let _name = vm_string(it.next().ok_or("RabbitMQ.declare_queue_raw: missing name")?, "RabbitMQ.declare_queue_raw")?;
            Ok(ok_vm(VMValue::Unit))
        }
        #[cfg(target_arch = "wasm32")]
        "RabbitMQ.declare_queue_raw" => Ok(err_vm(VMValue::Str("RabbitMQ not supported on wasm32".into()))),

        #[cfg(not(target_arch = "wasm32"))]
        "RabbitMQ.bind_queue_raw" => {
            // (conn: RabbitConn, queue: String, exchange: String, routing_key: String) -> Result<Unit, String>
            let mut it = args.into_iter();
            let _conn        = vm_string(it.next().ok_or("RabbitMQ.bind_queue_raw: missing conn")?,        "RabbitMQ.bind_queue_raw")?;
            let _queue       = vm_string(it.next().ok_or("RabbitMQ.bind_queue_raw: missing queue")?,       "RabbitMQ.bind_queue_raw")?;
            let _exchange    = vm_string(it.next().ok_or("RabbitMQ.bind_queue_raw: missing exchange")?,    "RabbitMQ.bind_queue_raw")?;
            let _routing_key = vm_string(it.next().ok_or("RabbitMQ.bind_queue_raw: missing routing_key")?, "RabbitMQ.bind_queue_raw")?;
            Ok(ok_vm(VMValue::Unit))
        }
        #[cfg(target_arch = "wasm32")]
        "RabbitMQ.bind_queue_raw" => Ok(err_vm(VMValue::Str("RabbitMQ not supported on wasm32".into()))),

        #[cfg(not(target_arch = "wasm32"))]
        "RabbitMQ.publish_raw" => {
            // (conn: RabbitConn, exchange: String, routing_key: String, msg: String) -> Result<Unit, String>
            let mut it = args.into_iter();
            let _conn        = vm_string(it.next().ok_or("RabbitMQ.publish_raw: missing conn")?,        "RabbitMQ.publish_raw")?;
            let _exchange    = vm_string(it.next().ok_or("RabbitMQ.publish_raw: missing exchange")?,    "RabbitMQ.publish_raw")?;
            let _routing_key = vm_string(it.next().ok_or("RabbitMQ.publish_raw: missing routing_key")?, "RabbitMQ.publish_raw")?;
            let _msg         = vm_string(it.next().ok_or("RabbitMQ.publish_raw: missing msg")?,         "RabbitMQ.publish_raw")?;
            Ok(ok_vm(VMValue::Unit))
        }
        #[cfg(target_arch = "wasm32")]
        "RabbitMQ.publish_raw" => Ok(err_vm(VMValue::Str("RabbitMQ not supported on wasm32".into()))),

        #[cfg(not(target_arch = "wasm32"))]
        "RabbitMQ.consume_raw" => {
            // (conn: RabbitConn, queue: String) -> Result<String, String>
            // Stub: returns empty JSON object (no live AMQP connection)
            let mut it = args.into_iter();
            let _conn  = vm_string(it.next().ok_or("RabbitMQ.consume_raw: missing conn")?,  "RabbitMQ.consume_raw")?;
            let _queue = vm_string(it.next().ok_or("RabbitMQ.consume_raw: missing queue")?, "RabbitMQ.consume_raw")?;
            Ok(ok_vm(VMValue::Str("{}".to_string())))
        }
        #[cfg(target_arch = "wasm32")]
        "RabbitMQ.consume_raw" => Ok(err_vm(VMValue::Str("RabbitMQ not supported on wasm32".into()))),

        // ── Pulsar primitives (v26.9.0) ────────────────────────────────────
        #[cfg(not(target_arch = "wasm32"))]
        "Pulsar.produce_raw" => {
            // (topic: String, key: String, value: String) -> Result<String, String>
            // Stub: Pulsar Binary Protocol (port 6650) は v27.x 以降。引数検証のみ実施。
            let mut it = args.into_iter();
            let _topic = vm_string(it.next().ok_or("Pulsar.produce_raw: missing topic")?, "Pulsar.produce_raw")?;
            let _key   = vm_string(it.next().ok_or("Pulsar.produce_raw: missing key")?,   "Pulsar.produce_raw")?;
            let _value = vm_string(it.next().ok_or("Pulsar.produce_raw: missing value")?, "Pulsar.produce_raw")?;
            Ok(ok_vm(VMValue::Str("stub-message-id".to_string())))
        }
        #[cfg(target_arch = "wasm32")]
        "Pulsar.produce_raw" => Ok(err_vm(VMValue::Str("Pulsar not supported on wasm32".into()))),

        #[cfg(not(target_arch = "wasm32"))]
        "Pulsar.consume_raw" => {
            // (topic: String, subscription: String) -> Result<String, String>
            // Stub: 1 回ポーリング返却。継続消費ループは v27.x 以降。
            let mut it = args.into_iter();
            let _topic        = vm_string(it.next().ok_or("Pulsar.consume_raw: missing topic")?,        "Pulsar.consume_raw")?;
            let _subscription = vm_string(it.next().ok_or("Pulsar.consume_raw: missing subscription")?, "Pulsar.consume_raw")?;
            Ok(ok_vm(VMValue::Str("[]".to_string())))
        }
        #[cfg(target_arch = "wasm32")]
        "Pulsar.consume_raw" => Ok(err_vm(VMValue::Str("Pulsar not supported on wasm32".into()))),

        #[cfg(not(target_arch = "wasm32"))]
        "Pulsar.ack_raw" => {
            // (message_id: String) -> Result<Unit, String>
            let mut it = args.into_iter();
            let _message_id = vm_string(it.next().ok_or("Pulsar.ack_raw: missing message_id")?, "Pulsar.ack_raw")?;
            Ok(ok_vm(VMValue::Unit))
        }
        #[cfg(target_arch = "wasm32")]
        "Pulsar.ack_raw" => Ok(err_vm(VMValue::Str("Pulsar not supported on wasm32".into()))),

        #[cfg(not(target_arch = "wasm32"))]
        "Pulsar.nack_raw" => {
            // (message_id: String) -> Result<Unit, String>
            let mut it = args.into_iter();
            let _message_id = vm_string(it.next().ok_or("Pulsar.nack_raw: missing message_id")?, "Pulsar.nack_raw")?;
            Ok(ok_vm(VMValue::Unit))
        }
        #[cfg(target_arch = "wasm32")]
        "Pulsar.nack_raw" => Ok(err_vm(VMValue::Str("Pulsar not supported on wasm32".into()))),

        // ── Delta Lake primitives (v27.1.0) ───────────────────────────────
        #[cfg(not(target_arch = "wasm32"))]
        "DeltaLake.read_raw" => {
            // (path: String) -> Result<String, String>
            // Stub: delta-rs 統合は v28.x 以降
            let mut it = args.into_iter();
            let _path = vm_string(it.next().ok_or("DeltaLake.read_raw: missing path")?, "DeltaLake.read_raw")?;
            Ok(ok_vm(VMValue::Str("[]".to_string())))
        }
        #[cfg(target_arch = "wasm32")]
        "DeltaLake.read_raw" => Ok(err_vm(VMValue::Str("DeltaLake not supported on wasm32".into()))),

        #[cfg(not(target_arch = "wasm32"))]
        "DeltaLake.read_with_filter_raw" => {
            // (path: String, predicate: String) -> Result<String, String>
            let mut it = args.into_iter();
            let _path      = vm_string(it.next().ok_or("DeltaLake.read_with_filter_raw: missing path")?,      "DeltaLake.read_with_filter_raw")?;
            let _predicate = vm_string(it.next().ok_or("DeltaLake.read_with_filter_raw: missing predicate")?, "DeltaLake.read_with_filter_raw")?;
            Ok(ok_vm(VMValue::Str("[]".to_string())))
        }
        #[cfg(target_arch = "wasm32")]
        "DeltaLake.read_with_filter_raw" => Ok(err_vm(VMValue::Str("DeltaLake not supported on wasm32".into()))),

        #[cfg(not(target_arch = "wasm32"))]
        "DeltaLake.write_raw" => {
            // (path: String, data: String, mode: String) -> Result<Unit, String>
            let mut it = args.into_iter();
            let _path = vm_string(it.next().ok_or("DeltaLake.write_raw: missing path")?, "DeltaLake.write_raw")?;
            let _data = vm_string(it.next().ok_or("DeltaLake.write_raw: missing data")?, "DeltaLake.write_raw")?;
            let mode  = vm_string(it.next().ok_or("DeltaLake.write_raw: missing mode")?, "DeltaLake.write_raw")?;
            if mode != "append" && mode != "overwrite" {
                return Ok(err_vm(VMValue::Str(format!("DeltaLake.write_raw: invalid mode '{}', must be 'append' or 'overwrite'", mode))));
            }
            Ok(ok_vm(VMValue::Unit))
        }
        #[cfg(target_arch = "wasm32")]
        "DeltaLake.write_raw" => Ok(err_vm(VMValue::Str("DeltaLake not supported on wasm32".into()))),

        #[cfg(not(target_arch = "wasm32"))]
        "DeltaLake.merge_raw" => {
            // (path: String, data: String, condition: String) -> Result<String, String>
            let mut it = args.into_iter();
            let _path      = vm_string(it.next().ok_or("DeltaLake.merge_raw: missing path")?,      "DeltaLake.merge_raw")?;
            let _data      = vm_string(it.next().ok_or("DeltaLake.merge_raw: missing data")?,      "DeltaLake.merge_raw")?;
            let _condition = vm_string(it.next().ok_or("DeltaLake.merge_raw: missing condition")?, "DeltaLake.merge_raw")?;
            Ok(ok_vm(VMValue::Str("{\"merged\":0}".to_string())))
        }
        #[cfg(target_arch = "wasm32")]
        "DeltaLake.merge_raw" => Ok(err_vm(VMValue::Str("DeltaLake not supported on wasm32".into()))),

        #[cfg(not(target_arch = "wasm32"))]
        "DeltaLake.history_raw" => {
            // (path: String) -> Result<String, String>
            let mut it = args.into_iter();
            let _path = vm_string(it.next().ok_or("DeltaLake.history_raw: missing path")?, "DeltaLake.history_raw")?;
            Ok(ok_vm(VMValue::Str("[]".to_string())))
        }
        #[cfg(target_arch = "wasm32")]
        "DeltaLake.history_raw" => Ok(err_vm(VMValue::Str("DeltaLake not supported on wasm32".into()))),

        #[cfg(not(target_arch = "wasm32"))]
        "DeltaLake.vacuum_raw" => {
            // (path: String, retention_hours: Int) -> Result<Unit, String>
            let mut it = args.into_iter();
            let _path = vm_string(it.next().ok_or("DeltaLake.vacuum_raw: missing path")?, "DeltaLake.vacuum_raw")?;
            let retention = match it.next().ok_or("DeltaLake.vacuum_raw: missing retention_hours")? {
                VMValue::Int(n) => n,
                _ => return Err("DeltaLake.vacuum_raw: retention_hours must be an Int".to_string()),
            };
            if retention < 168 {
                return Ok(err_vm(VMValue::Str(format!(
                    "DeltaLake.vacuum_raw: retention_hours {} is below minimum 168 (7 days)", retention
                ))));
            }
            Ok(ok_vm(VMValue::Unit))
        }
        #[cfg(target_arch = "wasm32")]
        "DeltaLake.vacuum_raw" => Ok(err_vm(VMValue::Str("DeltaLake not supported on wasm32".into()))),

        #[cfg(not(target_arch = "wasm32"))]
        "DeltaLake.optimize_raw" => {
            // (path: String) -> Result<String, String>
            let mut it = args.into_iter();
            let _path = vm_string(it.next().ok_or("DeltaLake.optimize_raw: missing path")?, "DeltaLake.optimize_raw")?;
            Ok(ok_vm(VMValue::Str("{\"optimized\":0}".to_string())))
        }
        #[cfg(target_arch = "wasm32")]
        "DeltaLake.optimize_raw" => Ok(err_vm(VMValue::Str("DeltaLake not supported on wasm32".into()))),

        #[cfg(not(target_arch = "wasm32"))]
        "DeltaLake.infer_schema_raw" => {
            // (path: String) -> Result<String, String>
            // Stub: delta-rs 統合は v28.x 以降。固定スキーマ JSON を返す
            let mut it = args.into_iter();
            let _path = vm_string(it.next().ok_or("DeltaLake.infer_schema_raw: missing path")?, "DeltaLake.infer_schema_raw")?;
            Ok(ok_vm(VMValue::Str("[]".to_string())))
        }
        #[cfg(target_arch = "wasm32")]
        "DeltaLake.infer_schema_raw" => Ok(err_vm(VMValue::Str("DeltaLake not supported on wasm32".into()))),

        // ── Apache Iceberg primitives (v27.2.0) ───────────────────────────
        #[cfg(not(target_arch = "wasm32"))]
        "Iceberg.read_raw" => {
            // (catalog: String, table: String) -> Result<String, String>
            // Stub: iceberg-rust 統合は v28.x 以降
            let mut it = args.into_iter();
            let _catalog = vm_string(it.next().ok_or("Iceberg.read_raw: missing catalog")?, "Iceberg.read_raw")?;
            let _table   = vm_string(it.next().ok_or("Iceberg.read_raw: missing table")?,   "Iceberg.read_raw")?;
            Ok(ok_vm(VMValue::Str("[]".to_string())))
        }
        #[cfg(target_arch = "wasm32")]
        "Iceberg.read_raw" => Ok(err_vm(VMValue::Str("Iceberg not supported on wasm32".into()))),

        #[cfg(not(target_arch = "wasm32"))]
        "Iceberg.append_raw" => {
            // (catalog: String, table: String, data: String) -> Result<Unit, String>
            let mut it = args.into_iter();
            let _catalog = vm_string(it.next().ok_or("Iceberg.append_raw: missing catalog")?, "Iceberg.append_raw")?;
            let _table   = vm_string(it.next().ok_or("Iceberg.append_raw: missing table")?,   "Iceberg.append_raw")?;
            let _data    = vm_string(it.next().ok_or("Iceberg.append_raw: missing data")?,    "Iceberg.append_raw")?;
            Ok(ok_vm(VMValue::Unit))
        }
        #[cfg(target_arch = "wasm32")]
        "Iceberg.append_raw" => Ok(err_vm(VMValue::Str("Iceberg not supported on wasm32".into()))),

        #[cfg(not(target_arch = "wasm32"))]
        "Iceberg.overwrite_raw" => {
            // (catalog: String, table: String, data: String, filter: String) -> Result<Unit, String>
            let mut it = args.into_iter();
            let _catalog = vm_string(it.next().ok_or("Iceberg.overwrite_raw: missing catalog")?, "Iceberg.overwrite_raw")?;
            let _table   = vm_string(it.next().ok_or("Iceberg.overwrite_raw: missing table")?,   "Iceberg.overwrite_raw")?;
            let _data    = vm_string(it.next().ok_or("Iceberg.overwrite_raw: missing data")?,    "Iceberg.overwrite_raw")?;
            let _filter  = vm_string(it.next().ok_or("Iceberg.overwrite_raw: missing filter")?,  "Iceberg.overwrite_raw")?;
            Ok(ok_vm(VMValue::Unit))
        }
        #[cfg(target_arch = "wasm32")]
        "Iceberg.overwrite_raw" => Ok(err_vm(VMValue::Str("Iceberg not supported on wasm32".into()))),

        #[cfg(not(target_arch = "wasm32"))]
        "Iceberg.time_travel_raw" => {
            // (catalog: String, table: String, snapshot_id: Int) -> Result<String, String>
            // snapshot_id: i64 (Apache Iceberg 仕様の 64-bit long と一致)
            let mut it = args.into_iter();
            let _catalog = vm_string(it.next().ok_or("Iceberg.time_travel_raw: missing catalog")?, "Iceberg.time_travel_raw")?;
            let _table   = vm_string(it.next().ok_or("Iceberg.time_travel_raw: missing table")?,   "Iceberg.time_travel_raw")?;
            let _snapshot_id = match it.next().ok_or("Iceberg.time_travel_raw: missing snapshot_id")? {
                VMValue::Int(n) => n,
                _ => return Err("Iceberg.time_travel_raw: snapshot_id must be an Int".to_string()),
            };
            Ok(ok_vm(VMValue::Str("[]".to_string())))
        }
        #[cfg(target_arch = "wasm32")]
        "Iceberg.time_travel_raw" => Ok(err_vm(VMValue::Str("Iceberg not supported on wasm32".into()))),

        #[cfg(not(target_arch = "wasm32"))]
        "Iceberg.schema_evolution_raw" => {
            // (catalog: String, table: String, new_schema: String) -> Result<Unit, String>
            let mut it = args.into_iter();
            let _catalog    = vm_string(it.next().ok_or("Iceberg.schema_evolution_raw: missing catalog")?,    "Iceberg.schema_evolution_raw")?;
            let _table      = vm_string(it.next().ok_or("Iceberg.schema_evolution_raw: missing table")?,      "Iceberg.schema_evolution_raw")?;
            let _new_schema = vm_string(it.next().ok_or("Iceberg.schema_evolution_raw: missing new_schema")?, "Iceberg.schema_evolution_raw")?;
            Ok(ok_vm(VMValue::Unit))
        }
        #[cfg(target_arch = "wasm32")]
        "Iceberg.schema_evolution_raw" => Ok(err_vm(VMValue::Str("Iceberg not supported on wasm32".into()))),

        #[cfg(not(target_arch = "wasm32"))]
        "Iceberg.list_snapshots_raw" => {
            // (catalog: String, table: String) -> Result<String, String>
            let mut it = args.into_iter();
            let _catalog = vm_string(it.next().ok_or("Iceberg.list_snapshots_raw: missing catalog")?, "Iceberg.list_snapshots_raw")?;
            let _table   = vm_string(it.next().ok_or("Iceberg.list_snapshots_raw: missing table")?,   "Iceberg.list_snapshots_raw")?;
            Ok(ok_vm(VMValue::Str("[]".to_string())))
        }
        #[cfg(target_arch = "wasm32")]
        "Iceberg.list_snapshots_raw" => Ok(err_vm(VMValue::Str("Iceberg not supported on wasm32".into()))),

        #[cfg(not(target_arch = "wasm32"))]
        "Iceberg.infer_schema_raw" => {
            // (catalog: String, table: String) -> Result<String, String>
            // Stub: iceberg-rust 統合は v28.x 以降。固定スキーマ JSON を返す
            let mut it = args.into_iter();
            let _catalog = vm_string(it.next().ok_or("Iceberg.infer_schema_raw: missing catalog")?, "Iceberg.infer_schema_raw")?;
            let _table   = vm_string(it.next().ok_or("Iceberg.infer_schema_raw: missing table")?,   "Iceberg.infer_schema_raw")?;
            Ok(ok_vm(VMValue::Str("[]".to_string())))
        }
        #[cfg(target_arch = "wasm32")]
        "Iceberg.infer_schema_raw" => Ok(err_vm(VMValue::Str("Iceberg not supported on wasm32".into()))),

        // ── ClickHouse primitives (v27.3.0) ───────────────────────────────
        #[cfg(not(target_arch = "wasm32"))]
        "ClickHouse.connect_raw" => {
            // (config: String) -> Result<String, String>
            // Stub: clickhouse-rs 統合は v28.x 以降。接続ハンドル識別子を返す（postgres Rune と同一パターン）
            // TODO(v28.x): _config（接続 URL / DSN）を clickhouse-rs の Client 初期化に渡し、
            //              実際のコネクションハンドルを返すように移行すること
            let mut it = args.into_iter();
            let _config = vm_string(it.next().ok_or("ClickHouse.connect_raw: missing config")?, "ClickHouse.connect_raw")?;
            Ok(ok_vm(VMValue::Str("clickhouse-stub-conn".to_string())))
        }
        #[cfg(target_arch = "wasm32")]
        "ClickHouse.connect_raw" => Ok(err_vm(VMValue::Str("ClickHouse not supported on wasm32".into()))),

        #[cfg(not(target_arch = "wasm32"))]
        "ClickHouse.query_raw" => {
            // (conn: String, sql: String) -> Result<String, String>
            // TODO(v28.x): ClickHouse.query[T] ジェネリック API に移行予定
            let mut it = args.into_iter();
            let _conn = vm_string(it.next().ok_or("ClickHouse.query_raw: missing conn")?, "ClickHouse.query_raw")?;
            let _sql  = vm_string(it.next().ok_or("ClickHouse.query_raw: missing sql")?,  "ClickHouse.query_raw")?;
            Ok(ok_vm(VMValue::Str("[]".to_string())))
        }
        #[cfg(target_arch = "wasm32")]
        "ClickHouse.query_raw" => Ok(err_vm(VMValue::Str("ClickHouse not supported on wasm32".into()))),

        #[cfg(not(target_arch = "wasm32"))]
        "ClickHouse.insert_raw" => {
            // (conn: String, table: String, rows: String) -> Result<Unit, String>
            let mut it = args.into_iter();
            let _conn  = vm_string(it.next().ok_or("ClickHouse.insert_raw: missing conn")?,  "ClickHouse.insert_raw")?;
            let _table = vm_string(it.next().ok_or("ClickHouse.insert_raw: missing table")?, "ClickHouse.insert_raw")?;
            let _rows  = vm_string(it.next().ok_or("ClickHouse.insert_raw: missing rows")?,  "ClickHouse.insert_raw")?;
            Ok(ok_vm(VMValue::Unit))
        }
        #[cfg(target_arch = "wasm32")]
        "ClickHouse.insert_raw" => Ok(err_vm(VMValue::Str("ClickHouse not supported on wasm32".into()))),

        #[cfg(not(target_arch = "wasm32"))]
        "ClickHouse.async_insert_raw" => {
            // (conn: String, table: String, rows: String) -> Result<Unit, String>
            let mut it = args.into_iter();
            let _conn  = vm_string(it.next().ok_or("ClickHouse.async_insert_raw: missing conn")?,  "ClickHouse.async_insert_raw")?;
            let _table = vm_string(it.next().ok_or("ClickHouse.async_insert_raw: missing table")?, "ClickHouse.async_insert_raw")?;
            let _rows  = vm_string(it.next().ok_or("ClickHouse.async_insert_raw: missing rows")?,  "ClickHouse.async_insert_raw")?;
            Ok(ok_vm(VMValue::Unit))
        }
        #[cfg(target_arch = "wasm32")]
        "ClickHouse.async_insert_raw" => Ok(err_vm(VMValue::Str("ClickHouse not supported on wasm32".into()))),

        // ── Redshift primitives (v27.5.0) ─────────────────────────────────
        // connect-based API（!Db エフェクト、postgres 互換）。
        // TODO(v28.x): postgres クレートを使った実接続に移行予定。
        //              _config（DSN / 接続文字列）を postgres クライアントの初期化に渡す。

        #[cfg(not(target_arch = "wasm32"))]
        "Redshift.connect_raw" => {
            // (config: String) -> Result<String, String> !Db
            let mut it = args.into_iter();
            let _config = vm_string(it.next().ok_or("Redshift.connect_raw: missing config")?, "Redshift.connect_raw")?;
            Ok(ok_vm(VMValue::Str("redshift-stub-conn".into())))
        }
        #[cfg(target_arch = "wasm32")]
        "Redshift.connect_raw" => Ok(err_vm(VMValue::Str("Redshift not supported on wasm32".into()))),

        #[cfg(not(target_arch = "wasm32"))]
        "Redshift.query_raw" => {
            // (conn: String, sql: String) -> Result<String, String> !Db
            let mut it = args.into_iter();
            let _conn = vm_string(it.next().ok_or("Redshift.query_raw: missing conn")?, "Redshift.query_raw")?;
            let _sql  = vm_string(it.next().ok_or("Redshift.query_raw: missing sql")?,  "Redshift.query_raw")?;
            Ok(ok_vm(VMValue::Str("[]".into())))
        }
        #[cfg(target_arch = "wasm32")]
        "Redshift.query_raw" => Ok(err_vm(VMValue::Str("Redshift not supported on wasm32".into()))),

        #[cfg(not(target_arch = "wasm32"))]
        "Redshift.execute_raw" => {
            // (conn: String, sql: String, params: String) -> Result<Int, String> !Db
            let mut it = args.into_iter();
            let _conn   = vm_string(it.next().ok_or("Redshift.execute_raw: missing conn")?,   "Redshift.execute_raw")?;
            let _sql    = vm_string(it.next().ok_or("Redshift.execute_raw: missing sql")?,    "Redshift.execute_raw")?;
            let _params = vm_string(it.next().ok_or("Redshift.execute_raw: missing params")?, "Redshift.execute_raw")?;
            Ok(ok_vm(VMValue::Int(0)))
        }
        #[cfg(target_arch = "wasm32")]
        "Redshift.execute_raw" => Ok(err_vm(VMValue::Str("Redshift not supported on wasm32".into()))),

        #[cfg(not(target_arch = "wasm32"))]
        "Redshift.copy_from_s3_raw" => {
            // (conn: String, table: String, s3_uri: String, opts: String) -> Result<Unit, String> !Db
            let mut it = args.into_iter();
            let _conn    = vm_string(it.next().ok_or("Redshift.copy_from_s3_raw: missing conn")?,    "Redshift.copy_from_s3_raw")?;
            let _table   = vm_string(it.next().ok_or("Redshift.copy_from_s3_raw: missing table")?,   "Redshift.copy_from_s3_raw")?;
            let _s3_uri  = vm_string(it.next().ok_or("Redshift.copy_from_s3_raw: missing s3_uri")?,  "Redshift.copy_from_s3_raw")?;
            let _opts    = vm_string(it.next().ok_or("Redshift.copy_from_s3_raw: missing opts")?,    "Redshift.copy_from_s3_raw")?;
            Ok(ok_vm(VMValue::Unit))
        }
        #[cfg(target_arch = "wasm32")]
        "Redshift.copy_from_s3_raw" => Ok(err_vm(VMValue::Str("Redshift not supported on wasm32".into()))),

        #[cfg(not(target_arch = "wasm32"))]
        "Redshift.unload_to_s3_raw" => {
            // (conn: String, query: String, s3_uri: String, opts: String) -> Result<Unit, String> !Db
            let mut it = args.into_iter();
            let _conn   = vm_string(it.next().ok_or("Redshift.unload_to_s3_raw: missing conn")?,   "Redshift.unload_to_s3_raw")?;
            let _query  = vm_string(it.next().ok_or("Redshift.unload_to_s3_raw: missing query")?,  "Redshift.unload_to_s3_raw")?;
            let _s3_uri = vm_string(it.next().ok_or("Redshift.unload_to_s3_raw: missing s3_uri")?, "Redshift.unload_to_s3_raw")?;
            let _opts   = vm_string(it.next().ok_or("Redshift.unload_to_s3_raw: missing opts")?,   "Redshift.unload_to_s3_raw")?;
            Ok(ok_vm(VMValue::Unit))
        }
        #[cfg(target_arch = "wasm32")]
        "Redshift.unload_to_s3_raw" => Ok(err_vm(VMValue::Str("Redshift not supported on wasm32".into()))),

        // ── JSONL primitives (v27.6.0) ────────────────────────────────────
        // JSON Lines（1 行 1 JSON オブジェクト）の読み書き・ストリーミング処理。
        // TODO(v28.x): JSONL.read[T] / JSONL.stream[T] ジェネリック API に移行予定。
        //              stream_raw はコールバック機構（高階関数）なし。v28.x で追加する。

        #[cfg(not(target_arch = "wasm32"))]
        "JSONL.read_raw" => {
            // (path: String) -> Result<String, String> !Io
            let mut it = args.into_iter();
            let _path = vm_string(it.next().ok_or("JSONL.read_raw: missing path")?, "JSONL.read_raw")?;
            Ok(ok_vm(VMValue::Str("[]".into())))
        }
        #[cfg(target_arch = "wasm32")]
        "JSONL.read_raw" => Ok(err_vm(VMValue::Str("JSONL not supported on wasm32".into()))),

        #[cfg(not(target_arch = "wasm32"))]
        "JSONL.write_raw" => {
            // (path: String, rows: String) -> Result<Unit, String> !Io
            let mut it = args.into_iter();
            let _path = vm_string(it.next().ok_or("JSONL.write_raw: missing path")?, "JSONL.write_raw")?;
            let _rows = vm_string(it.next().ok_or("JSONL.write_raw: missing rows")?, "JSONL.write_raw")?;
            Ok(ok_vm(VMValue::Unit))
        }
        #[cfg(target_arch = "wasm32")]
        "JSONL.write_raw" => Ok(err_vm(VMValue::Str("JSONL not supported on wasm32".into()))),

        #[cfg(not(target_arch = "wasm32"))]
        "JSONL.stream_raw" => {
            // (path: String) -> Result<String, String> !Io
            // stub: コールバック機構は v28.x。全件読み込みと同等の結果を返す。
            let mut it = args.into_iter();
            let _path = vm_string(it.next().ok_or("JSONL.stream_raw: missing path")?, "JSONL.stream_raw")?;
            Ok(ok_vm(VMValue::Str("[]".into())))
        }
        #[cfg(target_arch = "wasm32")]
        "JSONL.stream_raw" => Ok(err_vm(VMValue::Str("JSONL not supported on wasm32".into()))),

        #[cfg(not(target_arch = "wasm32"))]
        "JSONL.append_raw" => {
            // (path: String, row: String) -> Result<Unit, String> !Io
            let mut it = args.into_iter();
            let _path = vm_string(it.next().ok_or("JSONL.append_raw: missing path")?, "JSONL.append_raw")?;
            let _row  = vm_string(it.next().ok_or("JSONL.append_raw: missing row")?,  "JSONL.append_raw")?;
            Ok(ok_vm(VMValue::Unit))
        }
        #[cfg(target_arch = "wasm32")]
        "JSONL.append_raw" => Ok(err_vm(VMValue::Str("JSONL not supported on wasm32".into()))),

        // ── Dbt primitives (v27.8.0) ──────────────────────────────────────
        // TODO(v28.x): manifest.json の実解析・compiled SQL 実行に移行予定
        #[cfg(not(target_arch = "wasm32"))]
        "Dbt.ref_raw" => {
            // (config: String, model_name: String) -> Result<String, String>
            // Stub: manifest.json 解析は v28.x 以降。"[]" = 空行セット（JSON 配列）を返す
            let mut it = args.into_iter();
            let _config     = vm_string(it.next().ok_or("Dbt.ref_raw: missing config")?,      "Dbt.ref_raw")?;
            let _model_name = vm_string(it.next().ok_or("Dbt.ref_raw: missing model_name")?,  "Dbt.ref_raw")?;
            Ok(ok_vm(VMValue::Str("[]".to_string())))
        }
        #[cfg(target_arch = "wasm32")]
        "Dbt.ref_raw" => Ok(err_vm(VMValue::Str("Dbt not supported on wasm32".into()))),

        #[cfg(not(target_arch = "wasm32"))]
        "Dbt.source_raw" => {
            // (config: String, source_name: String, table_name: String) -> Result<String, String>
            // Stub: dbt source 定義解析は v28.x 以降。"[]" = 空行セット（JSON 配列）を返す
            let mut it = args.into_iter();
            let _config      = vm_string(it.next().ok_or("Dbt.source_raw: missing config")?,       "Dbt.source_raw")?;
            let _source_name = vm_string(it.next().ok_or("Dbt.source_raw: missing source_name")?,  "Dbt.source_raw")?;
            let _table_name  = vm_string(it.next().ok_or("Dbt.source_raw: missing table_name")?,   "Dbt.source_raw")?;
            Ok(ok_vm(VMValue::Str("[]".to_string())))
        }
        #[cfg(target_arch = "wasm32")]
        "Dbt.source_raw" => Ok(err_vm(VMValue::Str("Dbt not supported on wasm32".into()))),

        // ── SQLite primitives (v27.9.0) ───────────────────────────────────
        // TODO(v28.x): rusqlite --features bundled を使った実 SQLite 操作に移行予定
        #[cfg(not(target_arch = "wasm32"))]
        "SQLite.open_raw" => {
            // (path: String) -> Result<String, String>
            // Stub: DB ファイルを開く（v28.x で rusqlite 統合）。接続ハンドル識別子を返す
            let mut it = args.into_iter();
            let _path = vm_string(it.next().ok_or("SQLite.open_raw: missing path")?, "SQLite.open_raw")?;
            Ok(ok_vm(VMValue::Str("sqlite-stub-conn".to_string())))
        }
        #[cfg(target_arch = "wasm32")]
        "SQLite.open_raw" => Ok(err_vm(VMValue::Str("SQLite not supported on wasm32".into()))),

        #[cfg(not(target_arch = "wasm32"))]
        "SQLite.open_memory_raw" => {
            // () -> Result<String, String>
            // Stub: インメモリ DB（引数なし）。テスト用途を想定
            Ok(ok_vm(VMValue::Str("sqlite-memory-stub-conn".to_string())))
        }
        #[cfg(target_arch = "wasm32")]
        "SQLite.open_memory_raw" => Ok(err_vm(VMValue::Str("SQLite not supported on wasm32".into()))),

        #[cfg(not(target_arch = "wasm32"))]
        "SQLite.query_raw" => {
            // (db: String, sql: String, params: String) -> Result<String, String>
            // Stub: 型付きクエリ。"[]" = 空行セット（JSON 配列）を返す
            let mut it = args.into_iter();
            let _db     = vm_string(it.next().ok_or("SQLite.query_raw: missing db")?,     "SQLite.query_raw")?;
            let _sql    = vm_string(it.next().ok_or("SQLite.query_raw: missing sql")?,    "SQLite.query_raw")?;
            let _params = vm_string(it.next().ok_or("SQLite.query_raw: missing params")?, "SQLite.query_raw")?;
            Ok(ok_vm(VMValue::Str("[]".to_string())))
        }
        #[cfg(target_arch = "wasm32")]
        "SQLite.query_raw" => Ok(err_vm(VMValue::Str("SQLite not supported on wasm32".into()))),

        #[cfg(not(target_arch = "wasm32"))]
        "SQLite.execute_raw" => {
            // (db: String, sql: String, params: String) -> Result<Int, String>
            // Stub: DDL / DML 実行。影響行数 0 を返す（Redshift.execute_raw と同パターン）
            let mut it = args.into_iter();
            let _db     = vm_string(it.next().ok_or("SQLite.execute_raw: missing db")?,     "SQLite.execute_raw")?;
            let _sql    = vm_string(it.next().ok_or("SQLite.execute_raw: missing sql")?,    "SQLite.execute_raw")?;
            let _params = vm_string(it.next().ok_or("SQLite.execute_raw: missing params")?, "SQLite.execute_raw")?;
            Ok(ok_vm(VMValue::Int(0)))
        }
        #[cfg(target_arch = "wasm32")]
        "SQLite.execute_raw" => Ok(err_vm(VMValue::Str("SQLite not supported on wasm32".into()))),

        #[cfg(not(target_arch = "wasm32"))]
        "SQLite.execute_many_raw" => {
            // (db: String, sql: String, rows: String) -> Result<Int, String>
            // Stub: バッチ実行。影響行数 0 を返す
            let mut it = args.into_iter();
            let _db   = vm_string(it.next().ok_or("SQLite.execute_many_raw: missing db")?,   "SQLite.execute_many_raw")?;
            let _sql  = vm_string(it.next().ok_or("SQLite.execute_many_raw: missing sql")?,  "SQLite.execute_many_raw")?;
            let _rows = vm_string(it.next().ok_or("SQLite.execute_many_raw: missing rows")?, "SQLite.execute_many_raw")?;
            Ok(ok_vm(VMValue::Int(0)))
        }
        #[cfg(target_arch = "wasm32")]
        "SQLite.execute_many_raw" => Ok(err_vm(VMValue::Str("SQLite not supported on wasm32".into()))),

        #[cfg(not(target_arch = "wasm32"))]
        "SQLite.close_raw" => {
            // (db: String) -> Result<Unit, String>
            // Stub: DB クローズ
            let mut it = args.into_iter();
            let _db = vm_string(it.next().ok_or("SQLite.close_raw: missing db")?, "SQLite.close_raw")?;
            Ok(ok_vm(VMValue::Unit))
        }
        #[cfg(target_arch = "wasm32")]
        "SQLite.close_raw" => Ok(err_vm(VMValue::Str("SQLite not supported on wasm32".into()))),

        // ── Prometheus primitives (v28.1.0) ───────────────────────────────
        #[cfg(not(target_arch = "wasm32"))]
        "Prometheus.counter_raw" => {
            // (name: String, value: Float, labels: String) -> Result<Unit, String>
            // Stub: Pushgateway への HTTP 送信は v28.x 以降
            let mut it = args.into_iter();
            let _name   = vm_string(it.next().ok_or("Prometheus.counter_raw: missing name")?,   "Prometheus.counter_raw")?;
            let _value  = vm_float( it.next().ok_or("Prometheus.counter_raw: missing value")?,  "Prometheus.counter_raw")?;
            let _labels = vm_string(it.next().ok_or("Prometheus.counter_raw: missing labels")?, "Prometheus.counter_raw")?;
            Ok(ok_vm(VMValue::Unit))
        }
        #[cfg(target_arch = "wasm32")]
        "Prometheus.counter_raw" => Ok(err_vm(VMValue::Str("Prometheus not supported on wasm32".into()))),
        #[cfg(not(target_arch = "wasm32"))]
        "Prometheus.gauge_raw" => {
            // (name: String, value: Float) -> Result<Unit, String>
            let mut it = args.into_iter();
            let _name  = vm_string(it.next().ok_or("Prometheus.gauge_raw: missing name")?,  "Prometheus.gauge_raw")?;
            let _value = vm_float( it.next().ok_or("Prometheus.gauge_raw: missing value")?, "Prometheus.gauge_raw")?;
            Ok(ok_vm(VMValue::Unit))
        }
        #[cfg(target_arch = "wasm32")]
        "Prometheus.gauge_raw" => Ok(err_vm(VMValue::Str("Prometheus not supported on wasm32".into()))),
        #[cfg(not(target_arch = "wasm32"))]
        "Prometheus.histogram_raw" => {
            // (name: String, value: Float) -> Result<Unit, String>
            let mut it = args.into_iter();
            let _name  = vm_string(it.next().ok_or("Prometheus.histogram_raw: missing name")?,  "Prometheus.histogram_raw")?;
            let _value = vm_float( it.next().ok_or("Prometheus.histogram_raw: missing value")?, "Prometheus.histogram_raw")?;
            Ok(ok_vm(VMValue::Unit))
        }
        #[cfg(target_arch = "wasm32")]
        "Prometheus.histogram_raw" => Ok(err_vm(VMValue::Str("Prometheus not supported on wasm32".into()))),
        #[cfg(not(target_arch = "wasm32"))]
        "Prometheus.push_raw" => {
            // (gateway_url: String) -> Result<Unit, String>
            let mut it = args.into_iter();
            let _gateway_url = vm_string(it.next().ok_or("Prometheus.push_raw: missing gateway_url")?, "Prometheus.push_raw")?;
            Ok(ok_vm(VMValue::Unit))
        }
        #[cfg(target_arch = "wasm32")]
        "Prometheus.push_raw" => Ok(err_vm(VMValue::Str("Prometheus not supported on wasm32".into()))),

        // ── Datadog primitives (v28.2.0) ──────────────────────────────────
        #[cfg(not(target_arch = "wasm32"))]
        "Datadog.metric_raw" => {
            // (name: String, value: Float, tags: String) -> Result<Unit, String>
            // Stub: DogStatsD / Datadog API 送信は v28.x 以降
            let mut it = args.into_iter();
            let _name  = vm_string(it.next().ok_or("Datadog.metric_raw: missing name")?,  "Datadog.metric_raw")?;
            let _value = vm_float( it.next().ok_or("Datadog.metric_raw: missing value")?, "Datadog.metric_raw")?;
            let _tags  = vm_string(it.next().ok_or("Datadog.metric_raw: missing tags")?,  "Datadog.metric_raw")?;
            Ok(ok_vm(VMValue::Unit))
        }
        #[cfg(target_arch = "wasm32")]
        "Datadog.metric_raw" => Ok(err_vm(VMValue::Str("Datadog not supported on wasm32".into()))),
        #[cfg(not(target_arch = "wasm32"))]
        "Datadog.log_raw" => {
            // (level: String, message: String, attrs: String) -> Result<Unit, String>
            let mut it = args.into_iter();
            let _level   = vm_string(it.next().ok_or("Datadog.log_raw: missing level")?,   "Datadog.log_raw")?;
            let _message = vm_string(it.next().ok_or("Datadog.log_raw: missing message")?, "Datadog.log_raw")?;
            let _attrs   = vm_string(it.next().ok_or("Datadog.log_raw: missing attrs")?,   "Datadog.log_raw")?;
            Ok(ok_vm(VMValue::Unit))
        }
        #[cfg(target_arch = "wasm32")]
        "Datadog.log_raw" => Ok(err_vm(VMValue::Str("Datadog not supported on wasm32".into()))),
        #[cfg(not(target_arch = "wasm32"))]
        "Datadog.trace_raw" => {
            // (name: String, fn_body: String) -> Result<Unit, String>
            let mut it = args.into_iter();
            let _name    = vm_string(it.next().ok_or("Datadog.trace_raw: missing name")?,    "Datadog.trace_raw")?;
            let _fn_body = vm_string(it.next().ok_or("Datadog.trace_raw: missing fn_body")?, "Datadog.trace_raw")?;
            Ok(ok_vm(VMValue::Unit))
        }
        #[cfg(target_arch = "wasm32")]
        "Datadog.trace_raw" => Ok(err_vm(VMValue::Str("Datadog not supported on wasm32".into()))),
        #[cfg(not(target_arch = "wasm32"))]
        "Datadog.event_raw" => {
            // (title: String, text: String, tags: String) -> Result<Unit, String>
            let mut it = args.into_iter();
            let _title = vm_string(it.next().ok_or("Datadog.event_raw: missing title")?, "Datadog.event_raw")?;
            let _text  = vm_string(it.next().ok_or("Datadog.event_raw: missing text")?,  "Datadog.event_raw")?;
            let _tags  = vm_string(it.next().ok_or("Datadog.event_raw: missing tags")?,  "Datadog.event_raw")?;
            Ok(ok_vm(VMValue::Unit))
        }
        #[cfg(target_arch = "wasm32")]
        "Datadog.event_raw" => Ok(err_vm(VMValue::Str("Datadog not supported on wasm32".into()))),
        #[cfg(not(target_arch = "wasm32"))]
        "Datadog.service_check_raw" => {
            // (name: String, status: String) -> Result<Unit, String>
            let mut it = args.into_iter();
            let _name   = vm_string(it.next().ok_or("Datadog.service_check_raw: missing name")?,   "Datadog.service_check_raw")?;
            let _status = vm_string(it.next().ok_or("Datadog.service_check_raw: missing status")?, "Datadog.service_check_raw")?;
            Ok(ok_vm(VMValue::Unit))
        }
        #[cfg(target_arch = "wasm32")]
        "Datadog.service_check_raw" => Ok(err_vm(VMValue::Str("Datadog not supported on wasm32".into()))),

        // ── OTel primitives (v28.3.0) ─────────────────────────────────────
        // Stub: OTLP HTTP エクスポートは fav/src/otel.rs 経由（v28.x 以降）
        #[cfg(not(target_arch = "wasm32"))]
        "OTel.start_span_raw" => {
            // (name: String, service: String) -> Result<Unit, String>
            let mut it = args.into_iter();
            let _name    = vm_string(it.next().ok_or("OTel.start_span_raw: missing name")?,    "OTel.start_span_raw")?;
            let _service = vm_string(it.next().ok_or("OTel.start_span_raw: missing service")?, "OTel.start_span_raw")?;
            Ok(ok_vm(VMValue::Unit))
        }
        #[cfg(target_arch = "wasm32")]
        "OTel.start_span_raw" => Ok(err_vm(VMValue::Str("OTel not supported on wasm32".into()))),

        #[cfg(not(target_arch = "wasm32"))]
        "OTel.set_attribute_raw" => {
            // (key: String, value: String) -> Result<Unit, String>
            let mut it = args.into_iter();
            let _key   = vm_string(it.next().ok_or("OTel.set_attribute_raw: missing key")?,   "OTel.set_attribute_raw")?;
            let _value = vm_string(it.next().ok_or("OTel.set_attribute_raw: missing value")?, "OTel.set_attribute_raw")?;
            Ok(ok_vm(VMValue::Unit))
        }
        #[cfg(target_arch = "wasm32")]
        "OTel.set_attribute_raw" => Ok(err_vm(VMValue::Str("OTel not supported on wasm32".into()))),

        #[cfg(not(target_arch = "wasm32"))]
        "OTel.add_event_raw" => {
            // (name: String, attrs: String) -> Result<Unit, String>
            let mut it = args.into_iter();
            let _name  = vm_string(it.next().ok_or("OTel.add_event_raw: missing name")?,  "OTel.add_event_raw")?;
            let _attrs = vm_string(it.next().ok_or("OTel.add_event_raw: missing attrs")?, "OTel.add_event_raw")?;
            Ok(ok_vm(VMValue::Unit))
        }
        #[cfg(target_arch = "wasm32")]
        "OTel.add_event_raw" => Ok(err_vm(VMValue::Str("OTel not supported on wasm32".into()))),

        #[cfg(not(target_arch = "wasm32"))]
        "OTel.end_span_raw" => {
            // (status: String) -> Result<Unit, String>
            let mut it = args.into_iter();
            let _status = vm_string(it.next().ok_or("OTel.end_span_raw: missing status")?, "OTel.end_span_raw")?;
            Ok(ok_vm(VMValue::Unit))
        }
        #[cfg(target_arch = "wasm32")]
        "OTel.end_span_raw" => Ok(err_vm(VMValue::Str("OTel not supported on wasm32".into()))),

        // ── Sentry primitives (v28.5.0) ───────────────────────────────────
        // Stub: Sentry SDK / Relay HTTP 送信は v28.x 以降
        #[cfg(not(target_arch = "wasm32"))]
        "Sentry.capture_error_raw" => {
            let mut it = args.into_iter();
            let _err = vm_string(it.next().ok_or("Sentry.capture_error_raw: missing err")?, "Sentry.capture_error_raw")?;
            Ok(ok_vm(VMValue::Unit))
        }
        #[cfg(target_arch = "wasm32")]
        "Sentry.capture_error_raw" => Ok(err_vm(VMValue::Str("Sentry not supported on wasm32".into()))),

        #[cfg(not(target_arch = "wasm32"))]
        "Sentry.capture_message_raw" => {
            let mut it = args.into_iter();
            let _level = vm_string(it.next().ok_or("Sentry.capture_message_raw: missing level")?, "Sentry.capture_message_raw")?;
            let _msg   = vm_string(it.next().ok_or("Sentry.capture_message_raw: missing msg")?,   "Sentry.capture_message_raw")?;
            Ok(ok_vm(VMValue::Unit))
        }
        #[cfg(target_arch = "wasm32")]
        "Sentry.capture_message_raw" => Ok(err_vm(VMValue::Str("Sentry not supported on wasm32".into()))),

        #[cfg(not(target_arch = "wasm32"))]
        "Sentry.set_user_raw" => {
            let mut it = args.into_iter();
            let _id    = vm_string(it.next().ok_or("Sentry.set_user_raw: missing id")?,    "Sentry.set_user_raw")?;
            let _email = vm_string(it.next().ok_or("Sentry.set_user_raw: missing email")?, "Sentry.set_user_raw")?;
            Ok(ok_vm(VMValue::Unit))
        }
        #[cfg(target_arch = "wasm32")]
        "Sentry.set_user_raw" => Ok(err_vm(VMValue::Str("Sentry not supported on wasm32".into()))),

        #[cfg(not(target_arch = "wasm32"))]
        "Sentry.set_tag_raw" => {
            let mut it = args.into_iter();
            let _key   = vm_string(it.next().ok_or("Sentry.set_tag_raw: missing key")?,   "Sentry.set_tag_raw")?;
            let _value = vm_string(it.next().ok_or("Sentry.set_tag_raw: missing value")?, "Sentry.set_tag_raw")?;
            Ok(ok_vm(VMValue::Unit))
        }
        #[cfg(target_arch = "wasm32")]
        "Sentry.set_tag_raw" => Ok(err_vm(VMValue::Str("Sentry not supported on wasm32".into()))),

        #[cfg(not(target_arch = "wasm32"))]
        "Sentry.set_extra_raw" => {
            let mut it = args.into_iter();
            let _key   = vm_string(it.next().ok_or("Sentry.set_extra_raw: missing key")?,   "Sentry.set_extra_raw")?;
            let _value = vm_string(it.next().ok_or("Sentry.set_extra_raw: missing value")?, "Sentry.set_extra_raw")?;
            Ok(ok_vm(VMValue::Unit))
        }
        #[cfg(target_arch = "wasm32")]
        "Sentry.set_extra_raw" => Ok(err_vm(VMValue::Str("Sentry not supported on wasm32".into()))),

        // ── Grafana primitives (v28.6.0) ──────────────────────────────────
        // Stub: Grafana HTTP API リクエストは v28.7 以降
        #[cfg(not(target_arch = "wasm32"))]
        "Grafana.create_annotation_raw" => {
            let mut it = args.into_iter();
            let _dashboard_id = vm_string(it.next().ok_or("Grafana.create_annotation_raw: missing dashboard_id")?, "Grafana.create_annotation_raw")?;
            let _text         = vm_string(it.next().ok_or("Grafana.create_annotation_raw: missing text")?,         "Grafana.create_annotation_raw")?;
            let _tags         = vm_string(it.next().ok_or("Grafana.create_annotation_raw: missing tags")?,         "Grafana.create_annotation_raw")?;
            Ok(ok_vm(VMValue::Unit))
        }
        #[cfg(target_arch = "wasm32")]
        "Grafana.create_annotation_raw" => Ok(err_vm(VMValue::Str("Grafana not supported on wasm32".into()))),

        #[cfg(not(target_arch = "wasm32"))]
        "Grafana.push_dashboard_raw" => {
            let mut it = args.into_iter();
            let _json = vm_string(it.next().ok_or("Grafana.push_dashboard_raw: missing json")?, "Grafana.push_dashboard_raw")?;
            Ok(ok_vm(VMValue::Unit))
        }
        #[cfg(target_arch = "wasm32")]
        "Grafana.push_dashboard_raw" => Ok(err_vm(VMValue::Str("Grafana not supported on wasm32".into()))),

        #[cfg(not(target_arch = "wasm32"))]
        "Grafana.snapshot_raw" => {
            let mut it = args.into_iter();
            let _dashboard_id = vm_string(it.next().ok_or("Grafana.snapshot_raw: missing dashboard_id")?, "Grafana.snapshot_raw")?;
            // 注意: snapshot_raw のみ VMValue::Str を返す（スナップショット URL）。
            // create_annotation_raw / push_dashboard_raw は VMValue::Unit を返す点と非対称。
            Ok(ok_vm(VMValue::Str("https://grafana.example.com/dashboard/snapshot/stub".into())))
        }
        #[cfg(target_arch = "wasm32")]
        "Grafana.snapshot_raw" => Ok(err_vm(VMValue::Str("Grafana not supported on wasm32".into()))),

        // ── Azure Blob Storage primitives (v14.5.0) ───────────────────────
        "AzureBlob.put_raw" => {
            // AzureBlob.put_raw(account, key, container, blob_name, body) -> Result<Unit, String>
            let mut it = args.into_iter();
            let account   = vm_string(it.next().ok_or("put_raw: missing account")?,   "AzureBlob.put_raw")?;
            let key       = vm_string(it.next().ok_or("put_raw: missing key")?,       "AzureBlob.put_raw")?;
            let container = vm_string(it.next().ok_or("put_raw: missing container")?, "AzureBlob.put_raw")?;
            let blob_name = vm_string(it.next().ok_or("put_raw: missing blob_name")?, "AzureBlob.put_raw")?;
            let body      = vm_string(it.next().ok_or("put_raw: missing body")?,      "AzureBlob.put_raw")?;

            let canonical_resource = format!("/{}/{}/{}", account, container, blob_name);
            let (date, auth) = match azure_blob_sign(
                &account, &key, "PUT",
                "application/octet-stream", body.len(),
                "BlockBlob", &canonical_resource,
            ) {
                Ok(h) => h,
                Err(e) => return Ok(err_vm(VMValue::Str(e))),
            };
            let url = format!(
                "https://{}.blob.core.windows.net/{}/{}",
                account, container, blob_name
            );
            Ok(match ureq::put(&url)
                .set("x-ms-date", &date)
                .set("x-ms-version", "2020-10-02")
                .set("x-ms-blob-type", "BlockBlob")
                .set("Content-Type", "application/octet-stream")
                .set("Authorization", &auth)
                .send_bytes(body.as_bytes())
            {
                Ok(_)  => ok_vm(VMValue::Unit),
                Err(e) => err_vm(VMValue::Str(e.to_string())),
            })
        }

        "AzureBlob.get_raw" => {
            // AzureBlob.get_raw(account, key, container, blob_name) -> Result<String, String>
            let mut it = args.into_iter();
            let account   = vm_string(it.next().ok_or("get_raw: missing account")?,   "AzureBlob.get_raw")?;
            let key       = vm_string(it.next().ok_or("get_raw: missing key")?,       "AzureBlob.get_raw")?;
            let container = vm_string(it.next().ok_or("get_raw: missing container")?, "AzureBlob.get_raw")?;
            let blob_name = vm_string(it.next().ok_or("get_raw: missing blob_name")?, "AzureBlob.get_raw")?;

            let canonical_resource = format!("/{}/{}/{}", account, container, blob_name);
            let (date, auth) = match azure_blob_sign(
                &account, &key, "GET", "", 0, "", &canonical_resource,
            ) {
                Ok(h) => h,
                Err(e) => return Ok(err_vm(VMValue::Str(e))),
            };
            let url = format!(
                "https://{}.blob.core.windows.net/{}/{}",
                account, container, blob_name
            );
            Ok(match ureq::get(&url)
                .set("x-ms-date", &date)
                .set("x-ms-version", "2020-10-02")
                .set("Authorization", &auth)
                .call()
            {
                Ok(resp)  => match resp.into_string() {
                    Ok(body) => ok_vm(VMValue::Str(body)),
                    Err(e)   => err_vm(VMValue::Str(e.to_string())),
                },
                Err(e) => err_vm(VMValue::Str(e.to_string())),
            })
        }

        "AzureBlob.list_raw" => {
            // AzureBlob.list_raw(account, key, container, prefix) -> Result<String, String>
            // Returns JSON array of blob names
            let mut it = args.into_iter();
            let account   = vm_string(it.next().ok_or("list_raw: missing account")?,   "AzureBlob.list_raw")?;
            let key       = vm_string(it.next().ok_or("list_raw: missing key")?,       "AzureBlob.list_raw")?;
            let container = vm_string(it.next().ok_or("list_raw: missing container")?, "AzureBlob.list_raw")?;
            let prefix    = vm_string(it.next().ok_or("list_raw: missing prefix")?,    "AzureBlob.list_raw")?;

            // CanonicalizedResource with sorted query params
            let canonical_resource = format!(
                "/{}/{}\ncomp:list\nprefix:{}\nrestype:container",
                account, container, prefix
            );
            let (date, auth) = match azure_blob_sign(
                &account, &key, "GET", "", 0, "", &canonical_resource,
            ) {
                Ok(h) => h,
                Err(e) => return Ok(err_vm(VMValue::Str(e))),
            };
            let url = format!(
                "https://{}.blob.core.windows.net/{}?restype=container&comp=list&prefix={}",
                account, container, url_encode(&prefix)
            );
            Ok(match ureq::get(&url)
                .set("x-ms-date", &date)
                .set("x-ms-version", "2020-10-02")
                .set("Authorization", &auth)
                .call()
            {
                Ok(resp) => match resp.into_string() {
                    Ok(xml) => {
                        let names = extract_xml_tags(&xml, "Name");
                        let json = serde_json::to_string(&names)
                            .unwrap_or_else(|_| "[]".to_string());
                        ok_vm(VMValue::Str(json))
                    }
                    Err(e) => err_vm(VMValue::Str(e.to_string())),
                },
                Err(e) => err_vm(VMValue::Str(e.to_string())),
            })
        }

        "AzureBlob.delete_raw" => {
            // AzureBlob.delete_raw(account, key, container, blob_name) -> Result<Unit, String>
            let mut it = args.into_iter();
            let account   = vm_string(it.next().ok_or("delete_raw: missing account")?,   "AzureBlob.delete_raw")?;
            let key       = vm_string(it.next().ok_or("delete_raw: missing key")?,       "AzureBlob.delete_raw")?;
            let container = vm_string(it.next().ok_or("delete_raw: missing container")?, "AzureBlob.delete_raw")?;
            let blob_name = vm_string(it.next().ok_or("delete_raw: missing blob_name")?, "AzureBlob.delete_raw")?;

            let canonical_resource = format!("/{}/{}/{}", account, container, blob_name);
            let (date, auth) = match azure_blob_sign(
                &account, &key, "DELETE", "", 0, "", &canonical_resource,
            ) {
                Ok(h) => h,
                Err(e) => return Ok(err_vm(VMValue::Str(e))),
            };
            let url = format!(
                "https://{}.blob.core.windows.net/{}/{}",
                account, container, blob_name
            );
            Ok(match ureq::request("DELETE", &url)
                .set("x-ms-date", &date)
                .set("x-ms-version", "2020-10-02")
                .set("Authorization", &auth)
                .call()
            {
                Ok(_)  => ok_vm(VMValue::Unit),
                Err(e) => err_vm(VMValue::Str(e.to_string())),
            })
        }

        // ── Email primitives (v7.4.0) — SES thin wrapper ──────────────────
        "Email.send_raw" => {
            let mut it = args.into_iter();
            let from = vm_string(
                it.next().ok_or("Email.send_raw: missing from")?,
                "Email.send_raw",
            )?;
            let to = vm_string(
                it.next().ok_or("Email.send_raw: missing to")?,
                "Email.send_raw",
            )?;
            let subject = vm_string(
                it.next().ok_or("Email.send_raw: missing subject")?,
                "Email.send_raw",
            )?;
            let body = vm_string(
                it.next().ok_or("Email.send_raw: missing body")?,
                "Email.send_raw",
            )?;
            let config = get_aws_config();
            let base = if let Some(ep) = &config.endpoint_url {
                ep.trim_end_matches('/').to_string()
            } else {
                format!("https://email.{}.amazonaws.com/", config.region)
            };
            let form = format!(
                "Action=SendEmail&Source={}&Destination.ToAddresses.member.1={}&Message.Subject.Data={}&Message.Body.Text.Data={}&Version=2010-12-01",
                url_encode(&from),
                url_encode(&to),
                url_encode(&subject),
                url_encode(&body),
            );
            Ok(match aws_post(&config, "ses", &base, &form, "application/x-www-form-urlencoded", None) {
                Ok(_) => ok_vm(VMValue::Unit),
                Err(e) => err_vm(VMValue::Str(e)),
            })
        }

        // ── Cache primitives (v7.3.0) ─────────────────────────────────────
        "Cache.get_raw" => {
            let key = match args.into_iter().next() {
                Some(VMValue::Str(s)) => s,
                _ => return Err("Cache.get_raw requires a String key".to_string()),
            };
            let val = CACHE_STORE.with(|c| c.borrow().get(&key).cloned());
            Ok(match val {
                Some(v) => VMValue::Variant("some".to_string(), Some(Box::new(VMValue::Str(v)))),
                None => VMValue::Variant("none".to_string(), None),
            })
        }
        "Cache.set_raw" => {
            let mut it = args.into_iter();
            let key = match it.next() {
                Some(VMValue::Str(s)) => s,
                _ => return Err("Cache.set_raw: key must be a String".to_string()),
            };
            let value = match it.next() {
                Some(VMValue::Str(s)) => s,
                _ => return Err("Cache.set_raw: value must be a String".to_string()),
            };
            // ttl_secs is accepted but not enforced (in-process store)
            CACHE_STORE.with(|c| c.borrow_mut().insert(key, value));
            Ok(VMValue::Unit)
        }
        "Cache.del_raw" => {
            let key = match args.into_iter().next() {
                Some(VMValue::Str(s)) => s,
                _ => return Err("Cache.del_raw requires a String key".to_string()),
            };
            CACHE_STORE.with(|c| c.borrow_mut().remove(&key));
            Ok(VMValue::Unit)
        }
        "Cache.exists_raw" => {
            let key = match args.into_iter().next() {
                Some(VMValue::Str(s)) => s,
                _ => return Err("Cache.exists_raw requires a String key".to_string()),
            };
            let exists = CACHE_STORE.with(|c| c.borrow().contains_key(&key));
            Ok(VMValue::Bool(exists))
        }
        "Cache.del_prefix_raw" => {
            let prefix = match args.into_iter().next() {
                Some(VMValue::Str(s)) => s,
                _ => return Err("Cache.del_prefix_raw requires a String prefix".to_string()),
            };
            let count = CACHE_STORE.with(|c| {
                let mut store = c.borrow_mut();
                let keys: Vec<String> = store.keys()
                    .filter(|k| k.starts_with(&prefix))
                    .cloned()
                    .collect();
                let n = keys.len();
                for k in keys { store.remove(&k); }
                n
            });
            Ok(VMValue::Int(count as i64))
        }

        // ── Redis (v25.3.0) ───────────────────────────────────────────────
        // NOTE: !Cache primitives（上記）はインメモリ用。Redis は外部サービス専用。
        // 接続モデル: connect_raw は URL を RedisConn（String ラッパー）として返す。
        // 各 primitive は毎回 Client::open + get_connection() で接続を確立する。
        // これは PgConn パターンと同様の設計（接続プールは v26.x 以降で対応予定）。
        "Redis.connect_raw" => {
            let mut it = args.into_iter();
            let url = vm_string(
                it.next().ok_or("Redis.connect_raw: missing url")?,
                "Redis.connect_raw",
            )?;
            let client = redis::Client::open(url.as_str())
                .map_err(|e| format!("Redis connect error: {}", e))?;
            let mut conn = client
                .get_connection()
                .map_err(|e| format!("Redis connection error: {}", e))?;
            // PING 確認
            let _: String = redis::cmd("PING")
                .query(&mut conn)
                .map_err(|e| format!("Redis PING error: {}", e))?;
            Ok(ok_vm(VMValue::Str(url)))
        }

        "Redis.get_raw" => {
            let mut it = args.into_iter();
            let url = vm_string(
                it.next().ok_or("Redis.get_raw: missing conn")?,
                "Redis.get_raw",
            )?;
            let key = vm_string(
                it.next().ok_or("Redis.get_raw: missing key")?,
                "Redis.get_raw",
            )?;
            let client = redis::Client::open(url.as_str())
                .map_err(|e| format!("Redis.get_raw connect error: {}", e))?;
            let mut conn = client
                .get_connection()
                .map_err(|e| format!("Redis.get_raw connection error: {}", e))?;
            let result: Option<String> = redis::cmd("GET")
                .arg(&key)
                .query(&mut conn)
                .map_err(|e| format!("Redis GET error: {}", e))?;
            match result {
                Some(v) => Ok(ok_vm(VMValue::Str(v))),
                None => Ok(err_vm(VMValue::Str("nil".to_string()))),
            }
        }

        "Redis.set_raw" => {
            let mut it = args.into_iter();
            let url = vm_string(
                it.next().ok_or("Redis.set_raw: missing conn")?,
                "Redis.set_raw",
            )?;
            let key = vm_string(
                it.next().ok_or("Redis.set_raw: missing key")?,
                "Redis.set_raw",
            )?;
            let value = vm_string(
                it.next().ok_or("Redis.set_raw: missing value")?,
                "Redis.set_raw",
            )?;
            let ttl: i64 = match it.next() {
                Some(VMValue::Int(n)) => n,
                _ => 0,
            };
            let client = redis::Client::open(url.as_str())
                .map_err(|e| format!("Redis.set_raw connect error: {}", e))?;
            let mut conn = client
                .get_connection()
                .map_err(|e| format!("Redis.set_raw connection error: {}", e))?;
            if ttl > 0 {
                redis::cmd("SET")
                    .arg(&key)
                    .arg(&value)
                    .arg("EX")
                    .arg(ttl)
                    .query::<()>(&mut conn)
                    .map_err(|e| format!("Redis SET EX error: {}", e))?;
            } else {
                redis::cmd("SET")
                    .arg(&key)
                    .arg(&value)
                    .query::<()>(&mut conn)
                    .map_err(|e| format!("Redis SET error: {}", e))?;
            }
            Ok(ok_vm(VMValue::Unit))
        }

        "Redis.del_raw" => {
            let mut it = args.into_iter();
            let url = vm_string(
                it.next().ok_or("Redis.del_raw: missing conn")?,
                "Redis.del_raw",
            )?;
            let key = vm_string(
                it.next().ok_or("Redis.del_raw: missing key")?,
                "Redis.del_raw",
            )?;
            let client = redis::Client::open(url.as_str())
                .map_err(|e| format!("Redis.del_raw connect error: {}", e))?;
            let mut conn = client
                .get_connection()
                .map_err(|e| format!("Redis.del_raw connection error: {}", e))?;
            let count: i64 = redis::cmd("DEL")
                .arg(&key)
                .query(&mut conn)
                .map_err(|e| format!("Redis DEL error: {}", e))?;
            Ok(ok_vm(VMValue::Int(count)))
        }

        "Redis.incr_raw" => {
            let mut it = args.into_iter();
            let url = vm_string(
                it.next().ok_or("Redis.incr_raw: missing conn")?,
                "Redis.incr_raw",
            )?;
            let key = vm_string(
                it.next().ok_or("Redis.incr_raw: missing key")?,
                "Redis.incr_raw",
            )?;
            let client = redis::Client::open(url.as_str())
                .map_err(|e| format!("Redis.incr_raw connect error: {}", e))?;
            let mut conn = client
                .get_connection()
                .map_err(|e| format!("Redis.incr_raw connection error: {}", e))?;
            let new_val: i64 = redis::cmd("INCR")
                .arg(&key)
                .query(&mut conn)
                .map_err(|e| format!("Redis INCR error: {}", e))?;
            Ok(ok_vm(VMValue::Int(new_val)))
        }

        "Redis.lpush_raw" => {
            let mut it = args.into_iter();
            let url = vm_string(
                it.next().ok_or("Redis.lpush_raw: missing conn")?,
                "Redis.lpush_raw",
            )?;
            let key = vm_string(
                it.next().ok_or("Redis.lpush_raw: missing key")?,
                "Redis.lpush_raw",
            )?;
            let value = vm_string(
                it.next().ok_or("Redis.lpush_raw: missing value")?,
                "Redis.lpush_raw",
            )?;
            let client = redis::Client::open(url.as_str())
                .map_err(|e| format!("Redis.lpush_raw connect error: {}", e))?;
            let mut conn = client
                .get_connection()
                .map_err(|e| format!("Redis.lpush_raw connection error: {}", e))?;
            let list_len: i64 = redis::cmd("LPUSH")
                .arg(&key)
                .arg(&value)
                .query(&mut conn)
                .map_err(|e| format!("Redis LPUSH error: {}", e))?;
            Ok(ok_vm(VMValue::Int(list_len)))
        }

        "Redis.rpop_raw" => {
            let mut it = args.into_iter();
            let url = vm_string(
                it.next().ok_or("Redis.rpop_raw: missing conn")?,
                "Redis.rpop_raw",
            )?;
            let key = vm_string(
                it.next().ok_or("Redis.rpop_raw: missing key")?,
                "Redis.rpop_raw",
            )?;
            let client = redis::Client::open(url.as_str())
                .map_err(|e| format!("Redis.rpop_raw connect error: {}", e))?;
            let mut conn = client
                .get_connection()
                .map_err(|e| format!("Redis.rpop_raw connection error: {}", e))?;
            let result: Option<String> = redis::cmd("RPOP")
                .arg(&key)
                .query(&mut conn)
                .map_err(|e| format!("Redis RPOP error: {}", e))?;
            match result {
                Some(v) => Ok(ok_vm(VMValue::Str(v))),
                None => Ok(err_vm(VMValue::Str("nil".to_string()))),
            }
        }

        "Redis.publish_raw" => {
            let mut it = args.into_iter();
            let url = vm_string(
                it.next().ok_or("Redis.publish_raw: missing conn")?,
                "Redis.publish_raw",
            )?;
            let channel = vm_string(
                it.next().ok_or("Redis.publish_raw: missing channel")?,
                "Redis.publish_raw",
            )?;
            let msg = vm_string(
                it.next().ok_or("Redis.publish_raw: missing msg")?,
                "Redis.publish_raw",
            )?;
            let client = redis::Client::open(url.as_str())
                .map_err(|e| format!("Redis.publish_raw connect error: {}", e))?;
            let mut conn = client
                .get_connection()
                .map_err(|e| format!("Redis.publish_raw connection error: {}", e))?;
            let receivers: i64 = redis::cmd("PUBLISH")
                .arg(&channel)
                .arg(&msg)
                .query(&mut conn)
                .map_err(|e| format!("Redis PUBLISH error: {}", e))?;
            Ok(ok_vm(VMValue::Int(receivers)))
        }

        "Redis.subscribe_once_raw" => {
            // ブロッキング受信（タイムアウト REDIS_SUBSCRIBE_TIMEOUT_SECS 秒）
            // タイムアウト時は Result.err("timeout: no message received within 30s")
            const REDIS_SUBSCRIBE_TIMEOUT_SECS: u64 = 30;
            let mut it = args.into_iter();
            let url = vm_string(
                it.next().ok_or("Redis.subscribe_once_raw: missing conn")?,
                "Redis.subscribe_once_raw",
            )?;
            let channel = vm_string(
                it.next().ok_or("Redis.subscribe_once_raw: missing channel")?,
                "Redis.subscribe_once_raw",
            )?;
            let client = redis::Client::open(url.as_str())
                .map_err(|e| format!("Redis.subscribe_once_raw connect error: {}", e))?;
            let mut conn = client
                .get_connection()
                .map_err(|e| format!("Redis.subscribe_once_raw connection error: {}", e))?;
            conn.set_read_timeout(Some(std::time::Duration::from_secs(REDIS_SUBSCRIBE_TIMEOUT_SECS)))
                .map_err(|e| format!("Redis subscribe_once set_read_timeout error: {}", e))?;
            // PubSubCommands::subscribe を使って 1 件だけ受信
            use redis::PubSubCommands;
            let mut payload_result: Result<String, String> =
                Err(format!("timeout: no message received within {}s", REDIS_SUBSCRIBE_TIMEOUT_SECS));
            if let Err(e) = conn.subscribe(&[channel.as_str()], |msg| {
                payload_result = msg
                    .get_payload::<String>()
                    .map_err(|e| format!("Redis get_payload error: {}", e));
                redis::ControlFlow::Break(())
            }) {
                // subscribe 自体が失敗した場合（接続切断、購読エラー等）は実際のエラーを返す
                payload_result = Err(format!("Redis subscribe error: {}", e));
            }
            match payload_result {
                Ok(v) => Ok(ok_vm(VMValue::Str(v))),
                Err(e) => Ok(err_vm(VMValue::Str(e))),
            }
        }

        // ── MySQL (v25.4.0) ───────────────────────────────────────────────
        // NOTE: 接続モデルは RedisConn / PgConn パターンと同様。
        // 各 primitive は毎回 mysql::Conn::new(url) で接続を確立する（接続プールは v26.x 以降）。
        // transaction_begin/commit/rollback は各呼び出しで独立接続を使用するため、
        // 同一接続上のトランザクションとしては動作しない（擬似実装）。原子性は保証されない。
        #[cfg(not(target_arch = "wasm32"))]
        "MySQL.connect_raw" => {
            use mysql::prelude::Queryable;
            let url = vm_string(
                args.into_iter().next().ok_or("MySQL.connect_raw: missing url")?,
                "MySQL.connect_raw",
            )?;
            let mut conn = mysql::Conn::new(
                mysql::Opts::from_url(&url)
                    .map_err(|e| format!("MySQL.connect_raw invalid URL: {}", e))?,
            )
            .map_err(|e| format!("MySQL.connect_raw connection error: {}", e))?;
            // PING 確認
            conn.query_drop("SELECT 1")
                .map_err(|e| format!("MySQL.connect_raw PING error: {}", e))?;
            Ok(ok_vm(VMValue::Str(url)))
        }
        #[cfg(target_arch = "wasm32")]
        "MySQL.connect_raw" => Ok(err_vm(VMValue::Str("MySQL not supported on wasm32".into()))),

        #[cfg(not(target_arch = "wasm32"))]
        "MySQL.query_raw" => {
            use mysql::prelude::Queryable;
            let mut it = args.into_iter();
            let url = vm_string(
                it.next().ok_or("MySQL.query_raw: missing conn")?,
                "MySQL.query_raw",
            )?;
            let sql = vm_string(
                it.next().ok_or("MySQL.query_raw: missing sql")?,
                "MySQL.query_raw",
            )?;
            let params_json = vm_string(
                it.next().ok_or("MySQL.query_raw: missing params")?,
                "MySQL.query_raw",
            )?;
            let mut conn = mysql::Conn::new(
                mysql::Opts::from_url(&url)
                    .map_err(|e| format!("MySQL.query_raw invalid URL: {}", e))?,
            )
            .map_err(|e| format!("MySQL.query_raw connection error: {}", e))?;
            // params_json を mysql::Params に変換
            let params_val: serde_json::Value = serde_json::from_str(&params_json)
                .map_err(|e| format!("MySQL.query_raw params JSON error: {}", e))?;
            let positional: Vec<mysql::Value> = if let serde_json::Value::Array(arr) = params_val {
                arr.into_iter().map(|v| json_to_mysql_value(v)).collect()
            } else {
                vec![]
            };
            let rows: Vec<mysql::Row> = conn
                .exec(sql.as_str(), mysql::Params::Positional(positional))
                .map_err(|e| format!("MySQL.query_raw exec error: {}", e))?;
            // 各 Row を JSON オブジェクトに変換
            let json_rows: Vec<serde_json::Value> = rows
                .into_iter()
                .map(|row| {
                    let cols = row.columns_ref().to_vec();
                    let mut map = serde_json::Map::new();
                    for (i, col) in cols.iter().enumerate() {
                        let key = col.name_str().to_string();
                        let val = row.get::<mysql::Value, usize>(i)
                            .unwrap_or(mysql::Value::NULL);
                        map.insert(key, mysql_value_to_json(val));
                    }
                    serde_json::Value::Object(map)
                })
                .collect();
            let json_str = serde_json::to_string(&json_rows)
                .map_err(|e| format!("MySQL.query_raw serialize error: {}", e))?;
            Ok(ok_vm(VMValue::Str(json_str)))
        }
        #[cfg(target_arch = "wasm32")]
        "MySQL.query_raw" => Ok(err_vm(VMValue::Str("MySQL not supported on wasm32".into()))),

        #[cfg(not(target_arch = "wasm32"))]
        "MySQL.execute_raw" => {
            use mysql::prelude::Queryable;
            let mut it = args.into_iter();
            let url = vm_string(
                it.next().ok_or("MySQL.execute_raw: missing conn")?,
                "MySQL.execute_raw",
            )?;
            let sql = vm_string(
                it.next().ok_or("MySQL.execute_raw: missing sql")?,
                "MySQL.execute_raw",
            )?;
            let params_json = vm_string(
                it.next().ok_or("MySQL.execute_raw: missing params")?,
                "MySQL.execute_raw",
            )?;
            let mut conn = mysql::Conn::new(
                mysql::Opts::from_url(&url)
                    .map_err(|e| format!("MySQL.execute_raw invalid URL: {}", e))?,
            )
            .map_err(|e| format!("MySQL.execute_raw connection error: {}", e))?;
            let params_val: serde_json::Value = serde_json::from_str(&params_json)
                .map_err(|e| format!("MySQL.execute_raw params JSON error: {}", e))?;
            let positional: Vec<mysql::Value> = if let serde_json::Value::Array(arr) = params_val {
                arr.into_iter().map(|v| json_to_mysql_value(v)).collect()
            } else {
                vec![]
            };
            conn.exec_drop(sql.as_str(), mysql::Params::Positional(positional))
                .map_err(|e| format!("MySQL.execute_raw exec error: {e} (sql: {sql})"))?;
            let affected = conn.affected_rows();
            Ok(ok_vm(VMValue::Int(affected as i64)))
        }
        #[cfg(target_arch = "wasm32")]
        "MySQL.execute_raw" => Ok(err_vm(VMValue::Str("MySQL not supported on wasm32".into()))),

        #[cfg(not(target_arch = "wasm32"))]
        "MySQL.transaction_begin_raw" => {
            use mysql::prelude::Queryable;
            let url = vm_string(
                args.into_iter().next().ok_or("MySQL.transaction_begin_raw: missing conn")?,
                "MySQL.transaction_begin_raw",
            )?;
            let mut conn = mysql::Conn::new(
                mysql::Opts::from_url(&url)
                    .map_err(|e| format!("MySQL.transaction_begin_raw invalid URL: {}", e))?,
            )
            .map_err(|e| format!("MySQL.transaction_begin_raw connection error: {}", e))?;
            conn.exec_drop("BEGIN", mysql::Params::Empty)
                .map_err(|e| format!("MySQL transaction BEGIN failed: {}", e))?;
            Ok(ok_vm(VMValue::Unit))
        }
        #[cfg(target_arch = "wasm32")]
        "MySQL.transaction_begin_raw" => Ok(err_vm(VMValue::Str("MySQL not supported on wasm32".into()))),

        #[cfg(not(target_arch = "wasm32"))]
        "MySQL.transaction_commit_raw" => {
            use mysql::prelude::Queryable;
            let url = vm_string(
                args.into_iter().next().ok_or("MySQL.transaction_commit_raw: missing conn")?,
                "MySQL.transaction_commit_raw",
            )?;
            let mut conn = mysql::Conn::new(
                mysql::Opts::from_url(&url)
                    .map_err(|e| format!("MySQL.transaction_commit_raw invalid URL: {}", e))?,
            )
            .map_err(|e| format!("MySQL.transaction_commit_raw connection error: {}", e))?;
            conn.exec_drop("COMMIT", mysql::Params::Empty)
                .map_err(|e| format!("MySQL transaction COMMIT failed: {}", e))?;
            Ok(ok_vm(VMValue::Unit))
        }
        #[cfg(target_arch = "wasm32")]
        "MySQL.transaction_commit_raw" => Ok(err_vm(VMValue::Str("MySQL not supported on wasm32".into()))),

        #[cfg(not(target_arch = "wasm32"))]
        "MySQL.transaction_rollback_raw" => {
            use mysql::prelude::Queryable;
            let url = vm_string(
                args.into_iter().next().ok_or("MySQL.transaction_rollback_raw: missing conn")?,
                "MySQL.transaction_rollback_raw",
            )?;
            let mut conn = mysql::Conn::new(
                mysql::Opts::from_url(&url)
                    .map_err(|e| format!("MySQL.transaction_rollback_raw invalid URL: {}", e))?,
            )
            .map_err(|e| format!("MySQL.transaction_rollback_raw connection error: {}", e))?;
            conn.exec_drop("ROLLBACK", mysql::Params::Empty)
                .map_err(|e| format!("MySQL transaction ROLLBACK failed: {}", e))?;
            Ok(ok_vm(VMValue::Unit))
        }
        #[cfg(target_arch = "wasm32")]
        "MySQL.transaction_rollback_raw" => Ok(err_vm(VMValue::Str("MySQL not supported on wasm32".into()))),

        // ── MongoDB primitives (v25.5.0) — tokio + mongodb async API ──────
        // TODO(v26.x): 各 primitive で毎回 tokio runtime + mongodb::Client を生成している。
        // コネクションプール（static OnceLock<Client> 等）で改善予定。
        #[cfg(not(target_arch = "wasm32"))]
        "Mongo.connect_raw" => {
            let mut it = args.into_iter();
            let url = vm_string(
                it.next().ok_or("Mongo.connect_raw: missing url")?,
                "Mongo.connect_raw",
            )?;
            let db_name = extract_mongo_db_name(&url);
            let url2 = url.clone();
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|e| format!("Mongo.connect_raw: tokio error: {}", e))?
                .block_on(async move {
                    let client = mongodb::Client::with_uri_str(&url2)
                        .await
                        .map_err(|e| format!("Mongo.connect_raw: invalid URL: {}", e))?;
                    client
                        .database(&db_name)
                        .run_command(mongodb::bson::doc! { "ping": 1 })
                        .await
                        .map_err(|e| format!("Mongo.connect_raw: ping error: {}", e))?;
                    Ok::<_, String>(())
                })?;
            Ok(ok_vm(VMValue::Str(url)))
        }
        #[cfg(target_arch = "wasm32")]
        "Mongo.connect_raw" => Ok(err_vm(VMValue::Str("MongoDB not supported on wasm32".into()))),

        #[cfg(not(target_arch = "wasm32"))]
        "Mongo.find_raw" => {
            let mut it = args.into_iter();
            let url = vm_string(it.next().ok_or("Mongo.find_raw: missing conn")?, "Mongo.find_raw")?;
            let coll = vm_string(it.next().ok_or("Mongo.find_raw: missing coll")?, "Mongo.find_raw")?;
            let filter_json = vm_string(it.next().ok_or("Mongo.find_raw: missing filter")?, "Mongo.find_raw")?;
            let db_name = extract_mongo_db_name(&url);
            let result = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|e| format!("Mongo.find_raw: tokio error: {}", e))?
                .block_on(async move {
                    use futures::TryStreamExt;
                    let filter_doc = mongo_json_to_bson(&filter_json)?;
                    let client = mongodb::Client::with_uri_str(&url).await
                        .map_err(|e| format!("Mongo.find_raw connection error on '{}': {}", coll, e))?;
                    let collection = client.database(&db_name)
                        .collection::<mongodb::bson::Document>(&coll);
                    let cursor = collection.find(filter_doc).await
                        .map_err(|e| format!("Mongo.find_raw error on '{}': {}", coll, e))?;
                    let docs: Vec<mongodb::bson::Document> = cursor.try_collect().await
                        .map_err(|e| format!("Mongo.find_raw cursor error on '{}': {}", coll, e))?;
                    let json_arr: Vec<serde_json::Value> = docs.into_iter().map(mongo_bson_to_json).collect();
                    serde_json::to_string(&json_arr)
                        .map_err(|e| format!("Mongo.find_raw serialize error: {}", e))
                })?;
            Ok(ok_vm(VMValue::Str(result)))
        }
        #[cfg(target_arch = "wasm32")]
        "Mongo.find_raw" => Ok(err_vm(VMValue::Str("MongoDB not supported on wasm32".into()))),

        #[cfg(not(target_arch = "wasm32"))]
        "Mongo.find_one_raw" => {
            let mut it = args.into_iter();
            let url = vm_string(it.next().ok_or("Mongo.find_one_raw: missing conn")?, "Mongo.find_one_raw")?;
            let coll = vm_string(it.next().ok_or("Mongo.find_one_raw: missing coll")?, "Mongo.find_one_raw")?;
            let filter_json = vm_string(it.next().ok_or("Mongo.find_one_raw: missing filter")?, "Mongo.find_one_raw")?;
            let db_name = extract_mongo_db_name(&url);
            // connection/query エラーは ? で Rust Err として伝播（MySQL/Redis と同パターン）
            // None（未発見）のみ Favnir Result.err("not_found") として返す
            let opt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|e| format!("Mongo.find_one_raw: tokio error: {}", e))?
                .block_on(async move {
                    let filter_doc = mongo_json_to_bson(&filter_json)?;
                    let client = mongodb::Client::with_uri_str(&url).await
                        .map_err(|e| format!("Mongo.find_one_raw connection error on '{}': {}", coll, e))?;
                    let collection = client.database(&db_name)
                        .collection::<mongodb::bson::Document>(&coll);
                    collection.find_one(filter_doc).await
                        .map_err(|e| format!("Mongo.find_one_raw error on '{}': {}", coll, e))
                })?;
            match opt {
                Some(doc) => {
                    let json = mongo_bson_to_json(doc);
                    let s = serde_json::to_string(&json)
                        .map_err(|e| format!("Mongo.find_one_raw serialize error: {}", e))?;
                    Ok(ok_vm(VMValue::Str(s)))
                }
                None => Ok(err_vm(VMValue::Str("not_found".into()))),
            }
        }
        #[cfg(target_arch = "wasm32")]
        "Mongo.find_one_raw" => Ok(err_vm(VMValue::Str("MongoDB not supported on wasm32".into()))),

        #[cfg(not(target_arch = "wasm32"))]
        "Mongo.insert_one_raw" => {
            let mut it = args.into_iter();
            let url = vm_string(it.next().ok_or("Mongo.insert_one_raw: missing conn")?, "Mongo.insert_one_raw")?;
            let coll = vm_string(it.next().ok_or("Mongo.insert_one_raw: missing coll")?, "Mongo.insert_one_raw")?;
            let doc_json = vm_string(it.next().ok_or("Mongo.insert_one_raw: missing doc")?, "Mongo.insert_one_raw")?;
            let db_name = extract_mongo_db_name(&url);
            let result = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|e| format!("Mongo.insert_one_raw: tokio error: {}", e))?
                .block_on(async move {
                    let doc = mongo_json_to_bson(&doc_json)?;
                    let client = mongodb::Client::with_uri_str(&url).await
                        .map_err(|e| format!("Mongo.insert_one_raw connection error on '{}': {}", coll, e))?;
                    let collection = client.database(&db_name)
                        .collection::<mongodb::bson::Document>(&coll);
                    let res = collection.insert_one(doc).await
                        .map_err(|e| format!("Mongo.insert_one_raw error on '{}': {}", coll, e))?;
                    let id_str = match &res.inserted_id {
                        mongodb::bson::Bson::ObjectId(oid) => oid.to_hex(),
                        other => other.to_string(),
                    };
                    Ok::<_, String>(id_str)
                })?;
            Ok(ok_vm(VMValue::Str(result)))
        }
        #[cfg(target_arch = "wasm32")]
        "Mongo.insert_one_raw" => Ok(err_vm(VMValue::Str("MongoDB not supported on wasm32".into()))),

        #[cfg(not(target_arch = "wasm32"))]
        "Mongo.insert_many_raw" => {
            let mut it = args.into_iter();
            let url = vm_string(it.next().ok_or("Mongo.insert_many_raw: missing conn")?, "Mongo.insert_many_raw")?;
            let coll = vm_string(it.next().ok_or("Mongo.insert_many_raw: missing coll")?, "Mongo.insert_many_raw")?;
            let docs_json = vm_string(it.next().ok_or("Mongo.insert_many_raw: missing docs")?, "Mongo.insert_many_raw")?;
            let db_name = extract_mongo_db_name(&url);
            let count = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|e| format!("Mongo.insert_many_raw: tokio error: {}", e))?
                .block_on(async move {
                    let json_val: serde_json::Value = serde_json::from_str(&docs_json)
                        .map_err(|e| format!("Mongo.insert_many_raw JSON error: {}", e))?;
                    let arr = match json_val {
                        serde_json::Value::Array(a) => a,
                        _ => return Err("Mongo.insert_many_raw: docs must be a JSON array".to_string()),
                    };
                    let docs: Vec<mongodb::bson::Document> = arr.into_iter()
                        .map(|v| mongodb::bson::to_document(&v)
                            .map_err(|e| format!("Mongo.insert_many_raw BSON error: {}", e)))
                        .collect::<Result<_, _>>()?;
                    let client = mongodb::Client::with_uri_str(&url).await
                        .map_err(|e| format!("Mongo.insert_many_raw connection error on '{}': {}", coll, e))?;
                    let collection = client.database(&db_name)
                        .collection::<mongodb::bson::Document>(&coll);
                    let res = collection.insert_many(docs).await
                        .map_err(|e| format!("Mongo.insert_many_raw error on '{}': {}", coll, e))?;
                    Ok::<_, String>(res.inserted_ids.len() as i64)
                })?;
            Ok(ok_vm(VMValue::Int(count)))
        }
        #[cfg(target_arch = "wasm32")]
        "Mongo.insert_many_raw" => Ok(err_vm(VMValue::Str("MongoDB not supported on wasm32".into()))),

        #[cfg(not(target_arch = "wasm32"))]
        "Mongo.update_one_raw" => {
            let mut it = args.into_iter();
            let url = vm_string(it.next().ok_or("Mongo.update_one_raw: missing conn")?, "Mongo.update_one_raw")?;
            let coll = vm_string(it.next().ok_or("Mongo.update_one_raw: missing coll")?, "Mongo.update_one_raw")?;
            let filter_json = vm_string(it.next().ok_or("Mongo.update_one_raw: missing filter")?, "Mongo.update_one_raw")?;
            let update_json = vm_string(it.next().ok_or("Mongo.update_one_raw: missing update")?, "Mongo.update_one_raw")?;
            let db_name = extract_mongo_db_name(&url);
            let modified = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|e| format!("Mongo.update_one_raw: tokio error: {}", e))?
                .block_on(async move {
                    let filter_doc = mongo_json_to_bson(&filter_json)?;
                    let update_doc = mongo_json_to_bson(&update_json)?;
                    // update ドキュメントは $set / $inc 等の演算子を含む必要がある
                    if update_doc.keys().next().map(|k| !k.starts_with('$')).unwrap_or(false) {
                        return Err(format!(
                            "Mongo.update_one_raw: update document on '{}' must use operators ($set, $inc, etc.), not a plain document",
                            coll
                        ));
                    }
                    let client = mongodb::Client::with_uri_str(&url).await
                        .map_err(|e| format!("Mongo.update_one_raw connection error on '{}': {}", coll, e))?;
                    let collection = client.database(&db_name)
                        .collection::<mongodb::bson::Document>(&coll);
                    let res = collection.update_one(filter_doc, update_doc).await
                        .map_err(|e| format!("Mongo.update_one_raw error on '{}': {}", coll, e))?;
                    Ok::<_, String>(res.modified_count as i64)
                })?;
            Ok(ok_vm(VMValue::Int(modified)))
        }
        #[cfg(target_arch = "wasm32")]
        "Mongo.update_one_raw" => Ok(err_vm(VMValue::Str("MongoDB not supported on wasm32".into()))),

        #[cfg(not(target_arch = "wasm32"))]
        "Mongo.delete_one_raw" => {
            let mut it = args.into_iter();
            let url = vm_string(it.next().ok_or("Mongo.delete_one_raw: missing conn")?, "Mongo.delete_one_raw")?;
            let coll = vm_string(it.next().ok_or("Mongo.delete_one_raw: missing coll")?, "Mongo.delete_one_raw")?;
            let filter_json = vm_string(it.next().ok_or("Mongo.delete_one_raw: missing filter")?, "Mongo.delete_one_raw")?;
            let db_name = extract_mongo_db_name(&url);
            let deleted = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|e| format!("Mongo.delete_one_raw: tokio error: {}", e))?
                .block_on(async move {
                    let filter_doc = mongo_json_to_bson(&filter_json)?;
                    let client = mongodb::Client::with_uri_str(&url).await
                        .map_err(|e| format!("Mongo.delete_one_raw connection error on '{}': {}", coll, e))?;
                    let collection = client.database(&db_name)
                        .collection::<mongodb::bson::Document>(&coll);
                    let res = collection.delete_one(filter_doc).await
                        .map_err(|e| format!("Mongo.delete_one_raw error on '{}': {}", coll, e))?;
                    Ok::<_, String>(res.deleted_count as i64)
                })?;
            Ok(ok_vm(VMValue::Int(deleted)))
        }
        #[cfg(target_arch = "wasm32")]
        "Mongo.delete_one_raw" => Ok(err_vm(VMValue::Str("MongoDB not supported on wasm32".into()))),

        #[cfg(not(target_arch = "wasm32"))]
        "Mongo.aggregate_raw" => {
            let mut it = args.into_iter();
            let url = vm_string(it.next().ok_or("Mongo.aggregate_raw: missing conn")?, "Mongo.aggregate_raw")?;
            let coll = vm_string(it.next().ok_or("Mongo.aggregate_raw: missing coll")?, "Mongo.aggregate_raw")?;
            let pipeline_json = vm_string(it.next().ok_or("Mongo.aggregate_raw: missing pipeline")?, "Mongo.aggregate_raw")?;
            let db_name = extract_mongo_db_name(&url);
            let result = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|e| format!("Mongo.aggregate_raw: tokio error: {}", e))?
                .block_on(async move {
                    use futures::TryStreamExt;
                    let json_val: serde_json::Value = serde_json::from_str(&pipeline_json)
                        .map_err(|e| format!("Mongo.aggregate_raw JSON error: {}", e))?;
                    let pipeline_arr = match json_val {
                        serde_json::Value::Array(a) => a,
                        _ => return Err("Mongo.aggregate_raw: pipeline must be a JSON array".to_string()),
                    };
                    let pipeline: Vec<mongodb::bson::Document> = pipeline_arr.into_iter()
                        .map(|v| mongodb::bson::to_document(&v)
                            .map_err(|e| format!("Mongo.aggregate_raw BSON error: {}", e)))
                        .collect::<Result<_, _>>()?;
                    let client = mongodb::Client::with_uri_str(&url).await
                        .map_err(|e| format!("Mongo.aggregate_raw connection error on '{}': {}", coll, e))?;
                    let collection = client.database(&db_name)
                        .collection::<mongodb::bson::Document>(&coll);
                    let cursor = collection.aggregate(pipeline).await
                        .map_err(|e| format!("Mongo.aggregate_raw error on '{}': {}", coll, e))?;
                    let docs: Vec<mongodb::bson::Document> = cursor.try_collect().await
                        .map_err(|e| format!("Mongo.aggregate_raw cursor error on '{}': {}", coll, e))?;
                    let json_arr: Vec<serde_json::Value> = docs.into_iter().map(mongo_bson_to_json).collect();
                    serde_json::to_string(&json_arr)
                        .map_err(|e| format!("Mongo.aggregate_raw serialize error: {}", e))
                })?;
            Ok(ok_vm(VMValue::Str(result)))
        }
        #[cfg(target_arch = "wasm32")]
        "Mongo.aggregate_raw" => Ok(err_vm(VMValue::Str("MongoDB not supported on wasm32".into()))),

        // ── DynamoDB primitives (v25.6.0) — JSON string I/O ──────────────
        #[cfg(not(target_arch = "wasm32"))]
        "DynamoDB.connect_raw" => {
            let mut it = args.into_iter();
            let endpoint_str = vm_string(it.next().ok_or("DynamoDB.connect_raw: missing endpoint")?, "DynamoDB.connect_raw")?;
            let config = get_aws_config();
            let url = get_dynamo_endpoint(&endpoint_str, &config);
            // ListTables ping（接続確認）
            match aws_post(&config, "dynamodb", &url, "{}", "application/x-amz-json-1.0", Some("DynamoDB_20120810.ListTables")) {
                Ok(_) => Ok(ok_vm(VMValue::Str(endpoint_str))),
                Err(e) => Ok(err_vm(VMValue::Str(format!("DynamoDB.connect_raw: ping failed: {}", e)))),
            }
        }
        #[cfg(target_arch = "wasm32")]
        "DynamoDB.connect_raw" => Ok(err_vm(VMValue::Str("DynamoDB not supported on wasm32".into()))),

        #[cfg(not(target_arch = "wasm32"))]
        "DynamoDB.get_item_raw" => {
            let mut it = args.into_iter();
            let endpoint_str = vm_string(it.next().ok_or("DynamoDB.get_item_raw: missing endpoint")?, "DynamoDB.get_item_raw")?;
            let table = vm_string(it.next().ok_or("DynamoDB.get_item_raw: missing table")?, "DynamoDB.get_item_raw")?;
            let key_json = vm_string(it.next().ok_or("DynamoDB.get_item_raw: missing key_json")?, "DynamoDB.get_item_raw")?;
            let config = get_aws_config();
            let url = get_dynamo_endpoint(&endpoint_str, &config);
            let key_val: serde_json::Value = serde_json::from_str(&key_json)
                .map_err(|e| format!("DynamoDB.get_item_raw: key JSON parse: {}", e))?;
            let key_item = json_to_dynamo_item(&key_val)
                .map_err(|e| format!("DynamoDB.get_item_raw: {}", e))?;
            let body = serde_json::json!({"TableName": table, "Key": key_item}).to_string();
            let resp_str = match aws_post(&config, "dynamodb", &url, &body, "application/x-amz-json-1.0", Some("DynamoDB_20120810.GetItem")) {
                Ok(s) => s,
                Err(e) => return Ok(err_vm(VMValue::Str(format!("DynamoDB.get_item_raw: table={}: {}", table, e)))),
            };
            let resp_json: serde_json::Value = serde_json::from_str(&resp_str)
                .map_err(|e| format!("DynamoDB.get_item_raw: JSON parse: {}", e))?;
            if let Some(item) = resp_json.get("Item") {
                let plain = dynamo_item_to_plain_json(item);
                let s = serde_json::to_string(&plain)
                    .map_err(|e| format!("DynamoDB.get_item_raw: serialize: {}", e))?;
                Ok(ok_vm(VMValue::Str(s)))
            } else {
                Ok(err_vm(VMValue::Str("not_found".into())))
            }
        }
        #[cfg(target_arch = "wasm32")]
        "DynamoDB.get_item_raw" => Ok(err_vm(VMValue::Str("DynamoDB not supported on wasm32".into()))),

        #[cfg(not(target_arch = "wasm32"))]
        "DynamoDB.put_item_raw" => {
            let mut it = args.into_iter();
            let endpoint_str = vm_string(it.next().ok_or("DynamoDB.put_item_raw: missing endpoint")?, "DynamoDB.put_item_raw")?;
            let table = vm_string(it.next().ok_or("DynamoDB.put_item_raw: missing table")?, "DynamoDB.put_item_raw")?;
            let item_json = vm_string(it.next().ok_or("DynamoDB.put_item_raw: missing item_json")?, "DynamoDB.put_item_raw")?;
            let config = get_aws_config();
            let url = get_dynamo_endpoint(&endpoint_str, &config);
            let item_val: serde_json::Value = serde_json::from_str(&item_json)
                .map_err(|e| format!("DynamoDB.put_item_raw: item JSON parse: {}", e))?;
            let item = json_to_dynamo_item(&item_val)
                .map_err(|e| format!("DynamoDB.put_item_raw: {}", e))?;
            let body = serde_json::json!({"TableName": table, "Item": item}).to_string();
            match aws_post(&config, "dynamodb", &url, &body, "application/x-amz-json-1.0", Some("DynamoDB_20120810.PutItem")) {
                Ok(_) => Ok(ok_vm(VMValue::Unit)),
                Err(e) => Ok(err_vm(VMValue::Str(format!("DynamoDB.put_item_raw: table={}: {}", table, e)))),
            }
        }
        #[cfg(target_arch = "wasm32")]
        "DynamoDB.put_item_raw" => Ok(err_vm(VMValue::Str("DynamoDB not supported on wasm32".into()))),

        #[cfg(not(target_arch = "wasm32"))]
        "DynamoDB.delete_item_raw" => {
            let mut it = args.into_iter();
            let endpoint_str = vm_string(it.next().ok_or("DynamoDB.delete_item_raw: missing endpoint")?, "DynamoDB.delete_item_raw")?;
            let table = vm_string(it.next().ok_or("DynamoDB.delete_item_raw: missing table")?, "DynamoDB.delete_item_raw")?;
            let key_json = vm_string(it.next().ok_or("DynamoDB.delete_item_raw: missing key_json")?, "DynamoDB.delete_item_raw")?;
            let config = get_aws_config();
            let url = get_dynamo_endpoint(&endpoint_str, &config);
            let key_val: serde_json::Value = serde_json::from_str(&key_json)
                .map_err(|e| format!("DynamoDB.delete_item_raw: key JSON parse: {}", e))?;
            let key_item = json_to_dynamo_item(&key_val)
                .map_err(|e| format!("DynamoDB.delete_item_raw: {}", e))?;
            let body = serde_json::json!({"TableName": table, "Key": key_item}).to_string();
            match aws_post(&config, "dynamodb", &url, &body, "application/x-amz-json-1.0", Some("DynamoDB_20120810.DeleteItem")) {
                Ok(_) => Ok(ok_vm(VMValue::Unit)),
                Err(e) => Ok(err_vm(VMValue::Str(format!("DynamoDB.delete_item_raw: table={}: {}", table, e)))),
            }
        }
        #[cfg(target_arch = "wasm32")]
        "DynamoDB.delete_item_raw" => Ok(err_vm(VMValue::Str("DynamoDB not supported on wasm32".into()))),

        #[cfg(not(target_arch = "wasm32"))]
        "DynamoDB.query_raw" => {
            let mut it = args.into_iter();
            let endpoint_str = vm_string(it.next().ok_or("DynamoDB.query_raw: missing endpoint")?, "DynamoDB.query_raw")?;
            let table = vm_string(it.next().ok_or("DynamoDB.query_raw: missing table")?, "DynamoDB.query_raw")?;
            let key_cond = vm_string(it.next().ok_or("DynamoDB.query_raw: missing key_cond")?, "DynamoDB.query_raw")?;
            let attr_vals_json = vm_string(it.next().ok_or("DynamoDB.query_raw: missing attr_vals_json")?, "DynamoDB.query_raw")?;
            let config = get_aws_config();
            let url = get_dynamo_endpoint(&endpoint_str, &config);
            let attr_vals: serde_json::Value = serde_json::from_str(&attr_vals_json)
                .map_err(|e| format!("DynamoDB.query_raw: attr_vals JSON parse: {}", e))?;
            let attr_map = json_to_dynamo_item(&attr_vals)
                .map_err(|e| format!("DynamoDB.query_raw: {}", e))?;
            let body = serde_json::json!({
                "TableName": table,
                "KeyConditionExpression": key_cond,
                "ExpressionAttributeValues": attr_map
            }).to_string();
            let resp_str = match aws_post(&config, "dynamodb", &url, &body, "application/x-amz-json-1.0", Some("DynamoDB_20120810.Query")) {
                Ok(s) => s,
                Err(e) => return Ok(err_vm(VMValue::Str(format!("DynamoDB.query_raw: table={}: {}", table, e)))),
            };
            let resp_json: serde_json::Value = serde_json::from_str(&resp_str)
                .map_err(|e| format!("DynamoDB.query_raw: JSON parse: {}", e))?;
            let items = resp_json.get("Items").and_then(|x| x.as_array()).cloned().unwrap_or_default();
            let plain: Vec<serde_json::Value> = items.iter().map(dynamo_item_to_plain_json).collect();
            let s = serde_json::to_string(&plain)
                .map_err(|e| format!("DynamoDB.query_raw: serialize: {}", e))?;
            Ok(ok_vm(VMValue::Str(s)))
        }
        #[cfg(target_arch = "wasm32")]
        "DynamoDB.query_raw" => Ok(err_vm(VMValue::Str("DynamoDB not supported on wasm32".into()))),

        #[cfg(not(target_arch = "wasm32"))]
        "DynamoDB.scan_raw" => {
            let mut it = args.into_iter();
            let endpoint_str = vm_string(it.next().ok_or("DynamoDB.scan_raw: missing endpoint")?, "DynamoDB.scan_raw")?;
            let table = vm_string(it.next().ok_or("DynamoDB.scan_raw: missing table")?, "DynamoDB.scan_raw")?;
            let filter_json = vm_string(it.next().ok_or("DynamoDB.scan_raw: missing filter_json")?, "DynamoDB.scan_raw")?;
            let config = get_aws_config();
            let url = get_dynamo_endpoint(&endpoint_str, &config);
            // filter_json が空 → FilterExpression を含めない（全件スキャン）
            // filter_json は DynamoDB 式文字列（例: "attribute_exists(pk)"）を直接渡す。
            // JSON オブジェクトではなく文字列として FilterExpression フィールドに設定する。
            let body = if filter_json.is_empty() {
                serde_json::json!({"TableName": table}).to_string()
            } else {
                serde_json::json!({"TableName": table, "FilterExpression": filter_json}).to_string()
            };
            let resp_str = match aws_post(&config, "dynamodb", &url, &body, "application/x-amz-json-1.0", Some("DynamoDB_20120810.Scan")) {
                Ok(s) => s,
                Err(e) => return Ok(err_vm(VMValue::Str(format!("DynamoDB.scan_raw: table={}: {}", table, e)))),
            };
            let resp_json: serde_json::Value = serde_json::from_str(&resp_str)
                .map_err(|e| format!("DynamoDB.scan_raw: JSON parse: {}", e))?;
            let items = resp_json.get("Items").and_then(|x| x.as_array()).cloned().unwrap_or_default();
            let plain: Vec<serde_json::Value> = items.iter().map(dynamo_item_to_plain_json).collect();
            let s = serde_json::to_string(&plain)
                .map_err(|e| format!("DynamoDB.scan_raw: serialize: {}", e))?;
            Ok(ok_vm(VMValue::Str(s)))
        }
        #[cfg(target_arch = "wasm32")]
        "DynamoDB.scan_raw" => Ok(err_vm(VMValue::Str("DynamoDB not supported on wasm32".into()))),

        #[cfg(not(target_arch = "wasm32"))]
        "DynamoDB.batch_write_raw" => {
            let mut it = args.into_iter();
            let endpoint_str = vm_string(it.next().ok_or("DynamoDB.batch_write_raw: missing endpoint")?, "DynamoDB.batch_write_raw")?;
            let table = vm_string(it.next().ok_or("DynamoDB.batch_write_raw: missing table")?, "DynamoDB.batch_write_raw")?;
            let puts_json = vm_string(it.next().ok_or("DynamoDB.batch_write_raw: missing puts_json")?, "DynamoDB.batch_write_raw")?;
            let puts: Vec<serde_json::Value> = serde_json::from_str(&puts_json)
                .map_err(|e| format!("DynamoDB.batch_write_raw: JSON parse: {}", e))?;
            if puts.len() > 25 {
                return Ok(err_vm(VMValue::Str(format!("DynamoDB.batch_write_raw: max 25 items per batch, got {}", puts.len()))));
            }
            let count = puts.len();
            // collect::<Result<_,_>>() の Err を ? で伝播させると VM クラッシュになるため
            // 他プリミティブと同様に Ok(err_vm(...)) パターンに変換する
            let requests: Vec<serde_json::Value> = match puts.iter()
                .map(|item| json_to_dynamo_item(item).map(|m| serde_json::json!({"PutRequest": {"Item": m}})))
                .collect::<Result<Vec<_>, _>>()
            {
                Ok(v) => v,
                Err(e) => return Ok(err_vm(VMValue::Str(format!("DynamoDB.batch_write_raw: item conversion: {}", e)))),
            };
            let config = get_aws_config();
            let url = get_dynamo_endpoint(&endpoint_str, &config);
            let body = serde_json::json!({"RequestItems": {&table: requests}}).to_string();
            match aws_post(&config, "dynamodb", &url, &body, "application/x-amz-json-1.0", Some("DynamoDB_20120810.BatchWriteItem")) {
                Ok(_) => Ok(ok_vm(VMValue::Int(count as i64))),
                Err(e) => Ok(err_vm(VMValue::Str(format!("DynamoDB.batch_write_raw: table={}: {}", table, e)))),
            }
        }
        #[cfg(target_arch = "wasm32")]
        "DynamoDB.batch_write_raw" => Ok(err_vm(VMValue::Str("DynamoDB not supported on wasm32".into()))),

        #[cfg(not(target_arch = "wasm32"))]
        "DynamoDB.transact_write_raw" => {
            let mut it = args.into_iter();
            let endpoint_str = vm_string(it.next().ok_or("DynamoDB.transact_write_raw: missing endpoint")?, "DynamoDB.transact_write_raw")?;
            let ops_json = vm_string(it.next().ok_or("DynamoDB.transact_write_raw: missing ops_json")?, "DynamoDB.transact_write_raw")?;
            // ops_json は DynamoDB TransactItems 形式（属性型 JSON）をそのまま渡す
            let ops: serde_json::Value = serde_json::from_str(&ops_json)
                .map_err(|e| format!("DynamoDB.transact_write_raw: JSON parse: {}", e))?;
            let config = get_aws_config();
            let url = get_dynamo_endpoint(&endpoint_str, &config);
            let body = serde_json::json!({"TransactItems": ops}).to_string();
            match aws_post(&config, "dynamodb", &url, &body, "application/x-amz-json-1.0", Some("DynamoDB_20120810.TransactWriteItems")) {
                Ok(_) => Ok(ok_vm(VMValue::Unit)),
                Err(e) => Ok(err_vm(VMValue::Str(format!("DynamoDB.transact_write_raw: {}", e)))),
            }
        }
        #[cfg(target_arch = "wasm32")]
        "DynamoDB.transact_write_raw" => Ok(err_vm(VMValue::Str("DynamoDB not supported on wasm32".into()))),

        // ── Elasticsearch primitives (v25.8.0) — JSON string I/O ─────────
        #[cfg(not(target_arch = "wasm32"))]
        "ES.connect_raw" => {
            let mut it = args.into_iter();
            let url_str = vm_string(it.next().ok_or("ES.connect_raw: missing url")?, "ES.connect_raw")?;
            let url = get_es_url(&url_str);
            match es_http("GET", &format!("{}/", url), None, None) {
                Ok(_) => Ok(ok_vm(VMValue::Str(url))),
                Err(e) => Ok(err_vm(VMValue::Str(format!("ES.connect_raw: ping failed: {}", e)))),
            }
        }
        #[cfg(target_arch = "wasm32")]
        "ES.connect_raw" => Ok(err_vm(VMValue::Str("Elasticsearch not supported on wasm32".into()))),

        #[cfg(not(target_arch = "wasm32"))]
        "ES.index_raw" => {
            let mut it = args.into_iter();
            let url_str = vm_string(it.next().ok_or("ES.index_raw: missing url")?, "ES.index_raw")?;
            let index = vm_string(it.next().ok_or("ES.index_raw: missing index")?, "ES.index_raw")?;
            let doc_json = vm_string(it.next().ok_or("ES.index_raw: missing doc_json")?, "ES.index_raw")?;
            let ep = format!("{}/{}/_doc", get_es_url(&url_str), index);
            match es_http("POST", &ep, Some("application/json"), Some(&doc_json)) {
                Ok(_) => Ok(ok_vm(VMValue::Unit)),
                Err(e) => Ok(err_vm(VMValue::Str(format!("ES.index_raw: {}", e)))),
            }
        }
        #[cfg(target_arch = "wasm32")]
        "ES.index_raw" => Ok(err_vm(VMValue::Str("Elasticsearch not supported on wasm32".into()))),

        #[cfg(not(target_arch = "wasm32"))]
        "ES.index_with_id_raw" => {
            let mut it = args.into_iter();
            let url_str = vm_string(it.next().ok_or("ES.index_with_id_raw: missing url")?, "ES.index_with_id_raw")?;
            let index = vm_string(it.next().ok_or("ES.index_with_id_raw: missing index")?, "ES.index_with_id_raw")?;
            let id = vm_string(it.next().ok_or("ES.index_with_id_raw: missing id")?, "ES.index_with_id_raw")?;
            let doc_json = vm_string(it.next().ok_or("ES.index_with_id_raw: missing doc_json")?, "ES.index_with_id_raw")?;
            let ep = format!("{}/{}/_doc/{}", get_es_url(&url_str), index, id);
            match es_http("PUT", &ep, Some("application/json"), Some(&doc_json)) {
                Ok(_) => Ok(ok_vm(VMValue::Unit)),
                Err(e) => Ok(err_vm(VMValue::Str(format!("ES.index_with_id_raw: {}", e)))),
            }
        }
        #[cfg(target_arch = "wasm32")]
        "ES.index_with_id_raw" => Ok(err_vm(VMValue::Str("Elasticsearch not supported on wasm32".into()))),

        #[cfg(not(target_arch = "wasm32"))]
        "ES.search_raw" => {
            let mut it = args.into_iter();
            let url_str = vm_string(it.next().ok_or("ES.search_raw: missing url")?, "ES.search_raw")?;
            let index = vm_string(it.next().ok_or("ES.search_raw: missing index")?, "ES.search_raw")?;
            let query_json = vm_string(it.next().ok_or("ES.search_raw: missing query_json")?, "ES.search_raw")?;
            let ep = format!("{}/{}/_search", get_es_url(&url_str), index);
            match es_http("POST", &ep, Some("application/json"), Some(&query_json)) {
                Ok(body) => {
                    let v: serde_json::Value = serde_json::from_str(&body)
                        .map_err(|e| format!("ES.search_raw: parse: {}", e))?;
                    let hits = v["hits"]["hits"].as_array().cloned().unwrap_or_default();
                    let sources: Vec<serde_json::Value> = hits.iter()
                        .filter_map(|h| {
                            let src = &h["_source"];
                            if src.is_null() { None } else { Some(src.clone()) }
                        })
                        .collect();
                    let s = serde_json::to_string(&sources)
                        .map_err(|e| format!("ES.search_raw: serialize: {}", e))?;
                    Ok(ok_vm(VMValue::Str(s)))
                }
                Err(e) => Ok(err_vm(VMValue::Str(format!("ES.search_raw: {}", e)))),
            }
        }
        #[cfg(target_arch = "wasm32")]
        "ES.search_raw" => Ok(err_vm(VMValue::Str("Elasticsearch not supported on wasm32".into()))),

        #[cfg(not(target_arch = "wasm32"))]
        "ES.knn_search_raw" => {
            let mut it = args.into_iter();
            let url_str = vm_string(it.next().ok_or("ES.knn_search_raw: missing url")?, "ES.knn_search_raw")?;
            let index = vm_string(it.next().ok_or("ES.knn_search_raw: missing index")?, "ES.knn_search_raw")?;
            let knn_json = vm_string(it.next().ok_or("ES.knn_search_raw: missing knn_json")?, "ES.knn_search_raw")?;
            let ep = format!("{}/{}/_search", get_es_url(&url_str), index);
            match es_http("POST", &ep, Some("application/json"), Some(&knn_json)) {
                Ok(body) => {
                    let v: serde_json::Value = serde_json::from_str(&body)
                        .map_err(|e| format!("ES.knn_search_raw: parse: {}", e))?;
                    let hits = v["hits"]["hits"].as_array().cloned().unwrap_or_default();
                    let sources: Vec<serde_json::Value> = hits.iter()
                        .filter_map(|h| {
                            let src = &h["_source"];
                            if src.is_null() { None } else { Some(src.clone()) }
                        })
                        .collect();
                    let s = serde_json::to_string(&sources)
                        .map_err(|e| format!("ES.knn_search_raw: serialize: {}", e))?;
                    Ok(ok_vm(VMValue::Str(s)))
                }
                Err(e) => Ok(err_vm(VMValue::Str(format!("ES.knn_search_raw: {}", e)))),
            }
        }
        #[cfg(target_arch = "wasm32")]
        "ES.knn_search_raw" => Ok(err_vm(VMValue::Str("Elasticsearch not supported on wasm32".into()))),

        #[cfg(not(target_arch = "wasm32"))]
        "ES.bulk_raw" => {
            let mut it = args.into_iter();
            let url_str = vm_string(it.next().ok_or("ES.bulk_raw: missing url")?, "ES.bulk_raw")?;
            let index = vm_string(it.next().ok_or("ES.bulk_raw: missing index")?, "ES.bulk_raw")?;
            let docs_json = vm_string(it.next().ok_or("ES.bulk_raw: missing docs_json")?, "ES.bulk_raw")?;
            let docs: Vec<serde_json::Value> = serde_json::from_str(&docs_json)
                .map_err(|e| format!("ES.bulk_raw: JSON parse: {}", e))?;
            let mut ndjson = String::new();
            for doc in &docs {
                ndjson.push_str(&format!("{{\"index\":{{\"_index\":\"{}\"}}}}\n", index));
                ndjson.push_str(&serde_json::to_string(doc)
                    .map_err(|e| format!("ES.bulk_raw: serialize doc: {}", e))?);
                ndjson.push('\n');
            }
            let ep = format!("{}/_bulk", get_es_url(&url_str));
            match es_http_ndjson("POST", &ep, &ndjson) {
                Ok(_) => Ok(ok_vm(VMValue::Unit)),
                Err(e) => Ok(err_vm(VMValue::Str(format!("ES.bulk_raw: {}", e)))),
            }
        }
        #[cfg(target_arch = "wasm32")]
        "ES.bulk_raw" => Ok(err_vm(VMValue::Str("Elasticsearch not supported on wasm32".into()))),

        #[cfg(not(target_arch = "wasm32"))]
        "ES.delete_raw" => {
            let mut it = args.into_iter();
            let url_str = vm_string(it.next().ok_or("ES.delete_raw: missing url")?, "ES.delete_raw")?;
            let index = vm_string(it.next().ok_or("ES.delete_raw: missing index")?, "ES.delete_raw")?;
            let id = vm_string(it.next().ok_or("ES.delete_raw: missing id")?, "ES.delete_raw")?;
            let ep = format!("{}/{}/_doc/{}", get_es_url(&url_str), index, id);
            match es_http("DELETE", &ep, None, None) {
                Ok(_) => Ok(ok_vm(VMValue::Unit)),
                Err(e) => Ok(err_vm(VMValue::Str(format!("ES.delete_raw: {}", e)))),
            }
        }
        #[cfg(target_arch = "wasm32")]
        "ES.delete_raw" => Ok(err_vm(VMValue::Str("Elasticsearch not supported on wasm32".into()))),

        #[cfg(not(target_arch = "wasm32"))]
        "ES.create_index_raw" => {
            let mut it = args.into_iter();
            let url_str = vm_string(it.next().ok_or("ES.create_index_raw: missing url")?, "ES.create_index_raw")?;
            let index = vm_string(it.next().ok_or("ES.create_index_raw: missing index")?, "ES.create_index_raw")?;
            let mapping_json = vm_string(it.next().ok_or("ES.create_index_raw: missing mapping_json")?, "ES.create_index_raw")?;
            let body = if mapping_json.trim().is_empty() { "{}".to_string() } else { mapping_json };
            let ep = format!("{}/{}", get_es_url(&url_str), index);
            match es_http("PUT", &ep, Some("application/json"), Some(&body)) {
                Ok(_) => Ok(ok_vm(VMValue::Unit)),
                Err(e) => Ok(err_vm(VMValue::Str(format!("ES.create_index_raw: {}", e)))),
            }
        }
        #[cfg(target_arch = "wasm32")]
        "ES.create_index_raw" => Ok(err_vm(VMValue::Str("Elasticsearch not supported on wasm32".into()))),

        // ── State primitives (v22.3.0) — in-memory backend stub ───────────
        "State.get_raw" => {
            let key = match args.into_iter().next() {
                Some(VMValue::Str(s)) => s,
                _ => return Err("State.get_raw requires a String key".to_string()),
            };
            let val = STATE_STORE.with(|c| c.borrow().get(&key).cloned());
            Ok(match val {
                Some(v) => VMValue::Variant("some".to_string(), Some(Box::new(VMValue::Str(v)))),
                None    => VMValue::Variant("none".to_string(), None),
            })
        }
        "State.set_raw" => {
            let mut it = args.into_iter();
            let key = match it.next() {
                Some(VMValue::Str(s)) => s,
                _ => return Err("State.set_raw: key must be a String".to_string()),
            };
            let val = match it.next() {
                Some(VMValue::Str(s)) => s,
                _ => return Err("State.set_raw: value must be a String".to_string()),
            };
            STATE_STORE.with(|c| c.borrow_mut().insert(key, val));
            Ok(VMValue::Unit)
        }
        "State.has_raw" => {
            let key = match args.into_iter().next() {
                Some(VMValue::Str(s)) => s,
                _ => return Err("State.has_raw: key must be a String".to_string()),
            };
            let exists = STATE_STORE.with(|c| c.borrow().contains_key(&key));
            Ok(VMValue::Bool(exists))
        }
        "State.delete_raw" => {
            let key = match args.into_iter().next() {
                Some(VMValue::Str(s)) => s,
                _ => return Err("State.delete_raw: key must be a String".to_string()),
            };
            STATE_STORE.with(|c| c.borrow_mut().remove(&key));
            Ok(VMValue::Unit)
        }

        // ── v23.1.0: Bytes 型 ─────────────────────────────────────────────
        "Bytes.from_hex" => {
            let s = match args.into_iter().next() {
                Some(VMValue::Str(s)) => s,
                _ => return Err("Bytes.from_hex: expected String".to_string()),
            };
            let s = s.trim().to_string();
            if s.len() % 2 != 0 {
                return Ok(err_vm(VMValue::Str("Bytes.from_hex: odd length".into())));
            }
            let bytes: Result<Vec<u8>, _> = (0..s.len() / 2)
                .map(|i| u8::from_str_radix(&s[i * 2..i * 2 + 2], 16))
                .collect();
            match bytes {
                Ok(b)  => Ok(ok_vm(VMValue::Bytes(bytes_new(b)))),
                Err(e) => Ok(err_vm(VMValue::Str(format!("Bytes.from_hex: {}", e)))),
            }
        }
        "Bytes.from_str" => {
            let s = match args.into_iter().next() {
                Some(VMValue::Str(s)) => s,
                _ => return Err("Bytes.from_str: expected String".to_string()),
            };
            Ok(VMValue::Bytes(bytes_new(s.into_bytes())))
        }
        "Bytes.len" => {
            let id = match args.into_iter().next() {
                Some(VMValue::Bytes(id)) => id,
                _ => return Err("Bytes.len: expected Bytes".to_string()),
            };
            let len = bytes_get_arc(id).map(|a| a.len()).unwrap_or(0) as i64;
            Ok(VMValue::Int(len))
        }
        "Bytes.get" => {
            let mut it = args.into_iter();
            let id  = match it.next() { Some(VMValue::Bytes(id)) => id, _ => return Err("Bytes.get: arg0 not Bytes".to_string()) };
            let idx = match it.next() { Some(VMValue::Int(n))    => n,  _ => return Err("Bytes.get: arg1 not Int".to_string()) };
            // [v23.1.0 fix] reject negative indices before as-usize cast
            if idx < 0 {
                return Ok(err_vm(VMValue::Str(format!("Bytes.get: negative index {}", idx))));
            }
            match bytes_get_arc(id) {
                Some(arc) => {
                    let i = idx as usize;
                    if i < arc.len() {
                        Ok(ok_vm(VMValue::Int(arc[i] as i64)))
                    } else {
                        Ok(err_vm(VMValue::Str(format!("Bytes.get: index {} out of bounds (len={})", idx, arc.len()))))
                    }
                }
                None => Ok(err_vm(VMValue::Str("Bytes.get: invalid Bytes handle".into()))),
            }
        }
        "Bytes.slice" => {
            let mut it = args.into_iter();
            let id    = match it.next() { Some(VMValue::Bytes(id)) => id, _ => return Err("Bytes.slice: arg0 not Bytes".to_string()) };
            let start = match it.next() { Some(VMValue::Int(n))    => n,  _ => return Err("Bytes.slice: arg1 not Int".to_string()) };
            let end   = match it.next() { Some(VMValue::Int(n))    => n,  _ => return Err("Bytes.slice: arg2 not Int".to_string()) };
            match bytes_get_arc(id) {
                Some(arc) => {
                    let len = arc.len();
                    // [v23.1.0 fix] clamp negative values to 0 before as-usize cast
                    let s = if start < 0 { 0usize } else { (start as usize).min(len) };
                    let e = if end < 0 { 0usize } else { (end as usize).min(len) }.max(s);
                    Ok(VMValue::Bytes(bytes_new(arc[s..e].to_vec())))
                }
                None => Ok(err_vm(VMValue::Str("Bytes.slice: invalid Bytes handle".into()))),
            }
        }
        "Bytes.concat" => {
            let mut it = args.into_iter();
            let id_a = match it.next() { Some(VMValue::Bytes(id)) => id, _ => return Err("Bytes.concat: arg0 not Bytes".to_string()) };
            let id_b = match it.next() { Some(VMValue::Bytes(id)) => id, _ => return Err("Bytes.concat: arg1 not Bytes".to_string()) };
            // [v23.1.0 fix] propagate runtime error on invalid handles instead of silent empty fallback
            let a = match bytes_get_arc(id_a) {
                Some(arc) => arc,
                None => return Err("Bytes.concat: invalid Bytes handle (arg0)".to_string()),
            };
            let b = match bytes_get_arc(id_b) {
                Some(arc) => arc,
                None => return Err("Bytes.concat: invalid Bytes handle (arg1)".to_string()),
            };
            let mut v = (*a).clone();
            v.extend_from_slice(&b);
            Ok(VMValue::Bytes(bytes_new(v)))
        }
        "Bytes.to_utf8" => {
            let id = match args.into_iter().next() { Some(VMValue::Bytes(id)) => id, _ => return Err("Bytes.to_utf8: expected Bytes".to_string()) };
            match bytes_get_arc(id) {
                Some(arc) => match std::str::from_utf8(&arc) {
                    Ok(s)  => Ok(ok_vm(VMValue::Str(s.to_string()))),
                    Err(e) => Ok(err_vm(VMValue::Str(format!("Bytes.to_utf8: {}", e)))),
                },
                None => Ok(err_vm(VMValue::Str("Bytes.to_utf8: invalid handle".into()))),
            }
        }
        "Bytes.to_hex" => {
            let id = match args.into_iter().next() { Some(VMValue::Bytes(id)) => id, _ => return Err("Bytes.to_hex: expected Bytes".to_string()) };
            // [v23.1.0 fix] propagate runtime error on invalid handle instead of silent empty string
            match bytes_get_arc(id) {
                Some(arc) => {
                    let hex = arc.iter().map(|b| format!("{:02x}", b)).collect::<String>();
                    Ok(VMValue::Str(hex))
                }
                None => Err("Bytes.to_hex: invalid Bytes handle".to_string()),
            }
        }
        "Bytes.read_u16" => {
            let mut it = args.into_iter();
            let id  = match it.next() { Some(VMValue::Bytes(id)) => id, _ => return Err("Bytes.read_u16: arg0 not Bytes".to_string()) };
            let off = match it.next() { Some(VMValue::Int(n))    => n as usize, _ => return Err("Bytes.read_u16: arg1 not Int".to_string()) };
            match bytes_get_arc(id) {
                Some(arc) if off + 2 <= arc.len() => {
                    let v = (arc[off] as i64) << 8 | arc[off + 1] as i64;
                    Ok(ok_vm(VMValue::Int(v)))
                }
                _ => Ok(err_vm(VMValue::Str("Bytes.read_u16: out of bounds".into()))),
            }
        }
        "Bytes.read_u24" => {
            let mut it = args.into_iter();
            let id  = match it.next() { Some(VMValue::Bytes(id)) => id, _ => return Err("Bytes.read_u24: arg0 not Bytes".to_string()) };
            let off = match it.next() { Some(VMValue::Int(n))    => n as usize, _ => return Err("Bytes.read_u24: arg1 not Int".to_string()) };
            match bytes_get_arc(id) {
                Some(arc) if off + 3 <= arc.len() => {
                    let v = (arc[off] as i64) << 16 | (arc[off + 1] as i64) << 8 | arc[off + 2] as i64;
                    Ok(ok_vm(VMValue::Int(v)))
                }
                _ => Ok(err_vm(VMValue::Str("Bytes.read_u24: out of bounds".into()))),
            }
        }
        "Bytes.read_u32" => {
            let mut it = args.into_iter();
            let id  = match it.next() { Some(VMValue::Bytes(id)) => id, _ => return Err("Bytes.read_u32: arg0 not Bytes".to_string()) };
            let off = match it.next() { Some(VMValue::Int(n))    => n as usize, _ => return Err("Bytes.read_u32: arg1 not Int".to_string()) };
            match bytes_get_arc(id) {
                Some(arc) if off + 4 <= arc.len() => {
                    let v = (arc[off] as i64) << 24
                          | (arc[off + 1] as i64) << 16
                          | (arc[off + 2] as i64) << 8
                          |  arc[off + 3] as i64;
                    Ok(ok_vm(VMValue::Int(v)))
                }
                _ => Ok(err_vm(VMValue::Str("Bytes.read_u32: out of bounds".into()))),
            }
        }
        // v23.4.0: LE バリアント（codegen.rs は u16 LE でバイトコードを生成するため）
        "Bytes.read_u16_le" => {
            let mut it = args.into_iter();
            let id  = match it.next() { Some(VMValue::Bytes(id)) => id, _ => return Err("Bytes.read_u16_le: arg0 not Bytes".to_string()) };
            let off = match it.next() { Some(VMValue::Int(n))    => n as usize, _ => return Err("Bytes.read_u16_le: arg1 not Int".to_string()) };
            match bytes_get_arc(id) {
                Some(arc) if off + 2 <= arc.len() => {
                    let v = arc[off] as i64 | (arc[off + 1] as i64) << 8;
                    Ok(ok_vm(VMValue::Int(v)))
                }
                _ => Ok(err_vm(VMValue::Str("Bytes.read_u16_le: out of bounds".into()))),
            }
        }
        "Bytes.read_u24_le" => {
            let mut it = args.into_iter();
            let id  = match it.next() { Some(VMValue::Bytes(id)) => id, _ => return Err("Bytes.read_u24_le: arg0 not Bytes".to_string()) };
            let off = match it.next() { Some(VMValue::Int(n))    => n as usize, _ => return Err("Bytes.read_u24_le: arg1 not Int".to_string()) };
            match bytes_get_arc(id) {
                Some(arc) if off + 3 <= arc.len() => {
                    let v = arc[off] as i64 | (arc[off + 1] as i64) << 8 | (arc[off + 2] as i64) << 16;
                    Ok(ok_vm(VMValue::Int(v)))
                }
                _ => Ok(err_vm(VMValue::Str("Bytes.read_u24_le: out of bounds".into()))),
            }
        }
        "Bytes.read_file" => {
            #[cfg(not(target_arch = "wasm32"))]
            {
                let path = match args.into_iter().next() {
                    Some(VMValue::Str(s)) => s,
                    _ => return Err("Bytes.read_file: expected String".to_string()),
                };
                // [v23.1.0 fix] reject path traversal: paths containing ".." components
                if std::path::Path::new(&path).components().any(|c| c == std::path::Component::ParentDir) {
                    return Ok(err_vm(VMValue::Str("Bytes.read_file: path traversal not allowed".into())));
                }
                return match std::fs::read(&path) {
                    Ok(data) => Ok(ok_vm(VMValue::Bytes(bytes_new(data)))),
                    Err(e)   => Ok(err_vm(VMValue::Str(format!("Bytes.read_file: {}", e)))),
                };
            }
            #[allow(unreachable_code)]
            Ok(err_vm(VMValue::Str("Bytes.read_file: not available on wasm32".into())))
        }
        "Bytes.write_file" => {
            #[cfg(not(target_arch = "wasm32"))]
            {
                let mut it = args.into_iter();
                let path = match it.next() {
                    Some(VMValue::Str(s)) => s,
                    _ => return Err("Bytes.write_file: arg0 not String".to_string()),
                };
                // [v23.1.0 fix] reject path traversal: paths containing ".." components
                if std::path::Path::new(&path).components().any(|c| c == std::path::Component::ParentDir) {
                    return Ok(err_vm(VMValue::Str("Bytes.write_file: path traversal not allowed".into())));
                }
                let id = match it.next() {
                    Some(VMValue::Bytes(id)) => id,
                    _ => return Err("Bytes.write_file: arg1 not Bytes".to_string()),
                };
                return match bytes_get_arc(id) {
                    Some(arc) => match std::fs::write(&path, arc.as_ref()) {
                        Ok(())  => Ok(ok_vm(VMValue::Unit)),
                        Err(e)  => Ok(err_vm(VMValue::Str(format!("Bytes.write_file: {}", e)))),
                    },
                    None => Ok(err_vm(VMValue::Str("Bytes.write_file: invalid Bytes handle".into()))),
                };
            }
            #[allow(unreachable_code)]
            Ok(err_vm(VMValue::Str("Bytes.write_file: not available on wasm32".into())))
        }

        // ── Mut コレクション primitives (v23.3.0) ──────────────────────────
        "Mut.list" => {
            Ok(VMValue::MutList(mut_list_new()))
        }
        "Mut.push" => {
            // ok_vm(VMValue::Unit) を返す設計: checker 戻り型は Result<Unit, String>。
            // Favnir の bind は Result をアンラップしない（単純代入）。
            // `bind _p <- Mut.push(lst, val)` で戻り値を捨てるのが標準パターン。
            let mut it = args.into_iter();
            let handle = it.next().ok_or_else(|| "Mut.push requires 2 arguments".to_string())?;
            let val    = it.next().ok_or_else(|| "Mut.push requires 2 arguments".to_string())?;
            let id = match handle {
                VMValue::MutList(id) => id,
                _ => return Err("Mut.push: first argument must be a MutList".to_string()),
            };
            MUT_LIST_STORE.with(|s| {
                s.borrow_mut()
                    .get_mut(&id)
                    .ok_or_else(|| format!("Mut.push: invalid MutList handle {}", id))
                    .map(|vec| vec.push(val))
            })?;
            Ok(ok_vm(VMValue::Unit))
        }
        "Mut.pop" => {
            let handle = args.into_iter().next()
                .ok_or_else(|| "Mut.pop requires 1 argument".to_string())?;
            let id = match handle {
                VMValue::MutList(id) => id,
                _ => return Err("Mut.pop: argument must be a MutList".to_string()),
            };
            MUT_LIST_STORE.with(|s| {
                s.borrow_mut()
                    .get_mut(&id)
                    .ok_or_else(|| format!("Mut.pop: invalid MutList handle {}", id))
                    .map(|vec| {
                        vec.pop()
                            .map(ok_vm)
                            .unwrap_or_else(|| err_vm(VMValue::Str("Mut.pop: list is empty".to_string())))
                    })
            })
        }
        "Mut.peek" => {
            let handle = args.into_iter().next()
                .ok_or_else(|| "Mut.peek requires 1 argument".to_string())?;
            let id = match handle {
                VMValue::MutList(id) => id,
                _ => return Err("Mut.peek: argument must be a MutList".to_string()),
            };
            MUT_LIST_STORE.with(|s| {
                s.borrow()
                    .get(&id)
                    .ok_or_else(|| format!("Mut.peek: invalid MutList handle {}", id))
                    .map(|vec| {
                        vec.last()
                            .cloned()
                            .map(ok_vm)
                            .unwrap_or_else(|| err_vm(VMValue::Str("Mut.peek: list is empty".to_string())))
                    })
            })
        }
        "Mut.len" => {
            let handle = args.into_iter().next()
                .ok_or_else(|| "Mut.len requires 1 argument".to_string())?;
            match handle {
                VMValue::MutList(id) => {
                    MUT_LIST_STORE.with(|s| {
                        s.borrow()
                            .get(&id)
                            .ok_or_else(|| format!("Mut.len: invalid MutList handle {}", id))
                            .map(|vec| VMValue::Int(vec.len() as i64))
                    })
                }
                VMValue::MutMap(id) => {
                    MUT_MAP_STORE.with(|s| {
                        s.borrow()
                            .get(&id)
                            .ok_or_else(|| format!("Mut.len: invalid MutMap handle {}", id))
                            .map(|vec| VMValue::Int(vec.len() as i64))
                    })
                }
                _ => Err("Mut.len: argument must be a MutList or MutMap".to_string()),
            }
        }
        "Mut.map" => {
            Ok(VMValue::MutMap(mut_map_new()))
        }
        "Mut.set" => {
            let mut it = args.into_iter();
            let handle = it.next().ok_or_else(|| "Mut.set requires 3 arguments".to_string())?;
            let key    = it.next().ok_or_else(|| "Mut.set requires 3 arguments".to_string())?;
            let val    = it.next().ok_or_else(|| "Mut.set requires 3 arguments".to_string())?;
            let id = match handle {
                VMValue::MutMap(id) => id,
                _ => return Err("Mut.set: first argument must be a MutMap".to_string()),
            };
            MUT_MAP_STORE.with(|s| {
                s.borrow_mut()
                    .get_mut(&id)
                    .ok_or_else(|| format!("Mut.set: invalid MutMap handle {}", id))
                    .map(|vec| {
                        if let Some(entry) = vec.iter_mut().find(|(k, _)| k == &key) {
                            entry.1 = val;
                        } else {
                            vec.push((key, val));
                        }
                    })
            })?;
            Ok(ok_vm(VMValue::Unit))
        }
        "Mut.get" => {
            let mut it = args.into_iter();
            let handle = it.next().ok_or_else(|| "Mut.get requires 2 arguments".to_string())?;
            let key    = it.next().ok_or_else(|| "Mut.get requires 2 arguments".to_string())?;
            let id = match handle {
                VMValue::MutMap(id) => id,
                _ => return Err("Mut.get: first argument must be a MutMap".to_string()),
            };
            MUT_MAP_STORE.with(|s| {
                s.borrow()
                    .get(&id)
                    .ok_or_else(|| format!("Mut.get: invalid MutMap handle {}", id))
                    .map(|vec| {
                        vec.iter()
                            .find(|(k, _)| k == &key)
                            .map(|(_, v)| ok_vm(v.clone()))
                            .unwrap_or_else(|| err_vm(VMValue::Str("Mut.get: key not found".to_string())))
                    })
            })
        }
        "Mut.delete" => {
            // キーが存在しない場合も ok(unit) を返す（冪等削除）。
            // エラーが必要な場合は Mut.has で事前確認すること。
            let mut it = args.into_iter();
            let handle = it.next().ok_or_else(|| "Mut.delete requires 2 arguments".to_string())?;
            let key    = it.next().ok_or_else(|| "Mut.delete requires 2 arguments".to_string())?;
            let id = match handle {
                VMValue::MutMap(id) => id,
                _ => return Err("Mut.delete: first argument must be a MutMap".to_string()),
            };
            MUT_MAP_STORE.with(|s| {
                s.borrow_mut()
                    .get_mut(&id)
                    .ok_or_else(|| format!("Mut.delete: invalid MutMap handle {}", id))
                    .map(|vec| vec.retain(|(k, _)| k != &key))
            })?;
            Ok(ok_vm(VMValue::Unit))
        }
        "Mut.has" => {
            let mut it = args.into_iter();
            let handle = it.next().ok_or_else(|| "Mut.has requires 2 arguments".to_string())?;
            let key    = it.next().ok_or_else(|| "Mut.has requires 2 arguments".to_string())?;
            let id = match handle {
                VMValue::MutMap(id) => id,
                _ => return Err("Mut.has: first argument must be a MutMap".to_string()),
            };
            MUT_MAP_STORE.with(|s| {
                s.borrow()
                    .get(&id)
                    .ok_or_else(|| format!("Mut.has: invalid MutMap handle {}", id))
                    .map(|vec| VMValue::Bool(vec.iter().any(|(k, _)| k == &key)))
            })
        }

        // ── Queue primitives (v7.3.0) — thin SQS wrappers ─────────────────
        "Queue.send_raw" => {
            let mut it = args.into_iter();
            let queue_url = vm_string(
                it.next().ok_or("Queue.send_raw: missing queue_url")?,
                "Queue.send_raw",
            )?;
            let body = vm_string(
                it.next().ok_or("Queue.send_raw: missing body")?,
                "Queue.send_raw",
            )?;
            let config = get_aws_config();
            let form = format!(
                "Action=SendMessage&MessageBody={}&Version=2012-11-05",
                url_encode(&body)
            );
            Ok(match aws_post(&config, "sqs", &queue_url, &form, "application/x-www-form-urlencoded", None) {
                Ok(_) => ok_vm(VMValue::Unit),
                Err(e) => err_vm(VMValue::Str(e)),
            })
        }
        "Queue.recv_raw" => {
            let mut it = args.into_iter();
            let queue_url = vm_string(
                it.next().ok_or("Queue.recv_raw: missing queue_url")?,
                "Queue.recv_raw",
            )?;
            let max = match it.next() {
                Some(VMValue::Int(n)) => n,
                _ => 1,
            };
            let config = get_aws_config();
            let form = format!(
                "Action=ReceiveMessage&MaxNumberOfMessages={}&AttributeName=All&Version=2012-11-05",
                max
            );
            Ok(match aws_post(&config, "sqs", &queue_url, &form, "application/x-www-form-urlencoded", None) {
                Ok(xml) => {
                    let bodies = extract_xml_tags(&xml, "Body");
                    let items: Vec<VMValue> = bodies.into_iter().map(VMValue::Str).collect();
                    ok_vm(VMValue::List(FavList::new(items)))
                }
                Err(e) => err_vm(VMValue::Str(e)),
            })
        }
        "Queue.ack_raw" | "Queue.delete_raw" => {
            let mut it = args.into_iter();
            let queue_url = vm_string(
                it.next().ok_or("Queue.ack_raw: missing queue_url")?,
                "Queue.ack_raw",
            )?;
            let receipt = vm_string(
                it.next().ok_or("Queue.ack_raw: missing receipt_handle")?,
                "Queue.ack_raw",
            )?;
            let config = get_aws_config();
            let form = format!(
                "Action=DeleteMessage&ReceiptHandle={}&Version=2012-11-05",
                url_encode(&receipt)
            );
            Ok(match aws_post(&config, "sqs", &queue_url, &form, "application/x-www-form-urlencoded", None) {
                Ok(_) => ok_vm(VMValue::Unit),
                Err(e) => err_vm(VMValue::Str(e)),
            })
        }

        // v13.5.0: Ctx primitives ────────────────────────────────────────────
        "Ctx.build_raw" => {
            // Ctx.build_raw(db_url: String, aws_region: String, s3_bucket: String)
            //   -> Result<String, String>
            // Returns Err if db_url is empty; otherwise Ok with a JSON context descriptor.
            if args.len() < 3 {
                return Err("Ctx.build_raw requires 3 arguments".to_string());
            }
            let db_url = match &args[0] {
                VMValue::Str(s) => s.clone(),
                _ => return Err("Ctx.build_raw: db_url must be a String".to_string()),
            };
            let aws_region = match &args[1] {
                VMValue::Str(s) => s.clone(),
                _ => return Err("Ctx.build_raw: aws_region must be a String".to_string()),
            };
            let s3_bucket = match &args[2] {
                VMValue::Str(s) => s.clone(),
                _ => return Err("Ctx.build_raw: s3_bucket must be a String".to_string()),
            };
            if db_url.is_empty() {
                return Ok(err_vm(VMValue::Str("missing env: DATABASE_URL".to_string())));
            }
            let ctx_json = format!(
                r#"{{"type":"AppCtx","db_url":"{}","aws_region":"{}","s3_bucket":"{}"}}"#,
                db_url.replace('"', "\\\""),
                aws_region.replace('"', "\\\""),
                s3_bucket.replace('"', "\\\""),
            );
            Ok(ok_vm(VMValue::Str(ctx_json)))
        }
        "Ctx.mock_raw" => {
            // Ctx.mock_raw(seed_rows: List<String>) -> String
            // Returns a JSON descriptor for a mock context.
            if args.len() < 1 {
                return Err("Ctx.mock_raw requires 1 argument".to_string());
            }
            let rows_json = match &args[0] {
                VMValue::List(fl) => {
                    let items: Vec<String> = fl
                        .iter()
                        .map(|v| match v {
                            VMValue::Str(s) => format!("\"{}\"", s.replace('"', "\\\"")),
                            other => format!("\"{}\"", vmvalue_repr(other).replace('"', "\\\"")),
                        })
                        .collect();
                    format!("[{}]", items.join(","))
                }
                _ => "[]".to_string(),
            };
            let mock_json = format!(r#"{{"type":"MockAppCtx","seed_rows":{}}}"#, rows_json);
            Ok(VMValue::Str(mock_json))
        }

        // v14.2.0: CrossCloud Ctx primitives ─────────────────────────────────
        "Ctx.build_aws_raw" => {
            // Ctx.build_aws_raw(region: String, s3_bucket: String, db_url: String)
            //   -> Result<AwsCtx, String>
            if args.len() < 3 {
                return Err("Ctx.build_aws_raw requires 3 arguments".to_string());
            }
            let region = match &args[0] {
                VMValue::Str(s) => s.clone(),
                _ => return Err("Ctx.build_aws_raw: region must be a String".to_string()),
            };
            let s3_bucket = match &args[1] {
                VMValue::Str(s) => s.clone(),
                _ => return Err("Ctx.build_aws_raw: s3_bucket must be a String".to_string()),
            };
            let db_url = match &args[2] {
                VMValue::Str(s) => s.clone(),
                _ => return Err("Ctx.build_aws_raw: db_url must be a String".to_string()),
            };
            let json = format!(
                r#"{{"type":"AwsCtx","region":"{}","s3_bucket":"{}","db_url":"{}"}}"#,
                region.replace('"', "\\\""),
                s3_bucket.replace('"', "\\\""),
                db_url.replace('"', "\\\""),
            );
            Ok(ok_vm(VMValue::Str(json)))
        }
        "Ctx.build_azure_raw" => {
            // Ctx.build_azure_raw(postgres_url, storage_account, storage_key, container)
            //   -> Result<AzureCtx, String>
            if args.len() < 4 {
                return Err("Ctx.build_azure_raw requires 4 arguments".to_string());
            }
            let postgres_url = match &args[0] {
                VMValue::Str(s) => s.clone(),
                _ => return Err("Ctx.build_azure_raw: postgres_url must be a String".to_string()),
            };
            let storage_account = match &args[1] {
                VMValue::Str(s) => s.clone(),
                _ => return Err("Ctx.build_azure_raw: storage_account must be a String".to_string()),
            };
            let storage_key = match &args[2] {
                VMValue::Str(s) => s.clone(),
                _ => return Err("Ctx.build_azure_raw: storage_key must be a String".to_string()),
            };
            let container = match &args[3] {
                VMValue::Str(s) => s.clone(),
                _ => return Err("Ctx.build_azure_raw: container must be a String".to_string()),
            };
            let json = format!(
                r#"{{"type":"AzureCtx","postgres_url":"{}","storage_account":"{}","storage_key":"{}","container":"{}"}}"#,
                postgres_url.replace('"', "\\\""),
                storage_account.replace('"', "\\\""),
                storage_key.replace('"', "\\\""),
                container.replace('"', "\\\""),
            );
            Ok(ok_vm(VMValue::Str(json)))
        }
        "Ctx.azure_get_field_raw" => {
            // Ctx.azure_get_field_raw(ctx: AzureCtx, field: String) -> String
            if args.len() < 2 {
                return Err("Ctx.azure_get_field_raw requires 2 arguments".to_string());
            }
            let ctx_str = match &args[0] {
                VMValue::Str(s) => s.clone(),
                _ => return Err("Ctx.azure_get_field_raw: ctx must be a String".to_string()),
            };
            let field = match &args[1] {
                VMValue::Str(s) => s.clone(),
                _ => return Err("Ctx.azure_get_field_raw: field must be a String".to_string()),
            };
            // ctx_str may be "ok({...})" or raw JSON
            let json_str = if ctx_str.starts_with("ok(") && ctx_str.ends_with(')') {
                &ctx_str[3..ctx_str.len() - 1]
            } else {
                &ctx_str
            };
            let val: String = serde_json::from_str::<serde_json::Value>(json_str)
                .ok()
                .and_then(|v| v.get(&field).and_then(|f| f.as_str()).map(|s| s.to_string()))
                .unwrap_or_default();
            Ok(VMValue::Str(val))
        }

        "Ctx.aws_get_field_raw" => {
            // Ctx.aws_get_field_raw(ctx: AwsCtx, field: String) -> String (v14.4.0)
            if args.len() < 2 {
                return Err("Ctx.aws_get_field_raw requires 2 arguments".to_string());
            }
            let ctx_str = match &args[0] {
                VMValue::Str(s) => s.clone(),
                _ => return Err("Ctx.aws_get_field_raw: ctx must be a String".to_string()),
            };
            let field = match &args[1] {
                VMValue::Str(s) => s.clone(),
                _ => return Err("Ctx.aws_get_field_raw: field must be a String".to_string()),
            };
            let json_str = if ctx_str.starts_with("ok(") && ctx_str.ends_with(')') {
                &ctx_str[3..ctx_str.len() - 1]
            } else {
                &ctx_str
            };
            let val: String = serde_json::from_str::<serde_json::Value>(json_str)
                .ok()
                .and_then(|v| v.get(&field).and_then(|f| f.as_str()).map(|s| s.to_string()))
                .unwrap_or_default();
            Ok(VMValue::Str(val))
        }

        // ── IO.getenv_raw (v13.6.0) ──────────────────────────────────────────
        "IO.getenv_raw" => {
            // IO.getenv_raw(key: String) -> Option<String>
            let key = vm_string(
                args.into_iter().next().ok_or_else(|| "IO.getenv_raw requires 1 argument".to_string())?,
                "IO.getenv_raw",
            )?;
            match std::env::var(&key) {
                Ok(val) => Ok(VMValue::Variant("some".to_string(), Some(Box::new(VMValue::Str(val))))),
                Err(_)  => Ok(VMValue::Variant("none".to_string(), None)),
            }
        }

        // ── AppCtx.* primitives (v13.6.0) ────────────────────────────────────
        "AppCtx.db_execute" => {
            // AppCtx.db_execute(ctx_json: String, sql: String, params: String) -> Result<Int, String>
            let mut it = args.into_iter();
            let ctx_json = vm_string(it.next().ok_or_else(|| "AppCtx.db_execute: missing ctx".to_string())?,   "AppCtx.db_execute ctx")?;
            let sql      = vm_string(it.next().ok_or_else(|| "AppCtx.db_execute: missing sql".to_string())?,   "AppCtx.db_execute sql")?;
            let params   = vm_string(it.next().ok_or_else(|| "AppCtx.db_execute: missing params".to_string())?, "AppCtx.db_execute params")?;
            let conn_str = appctx_db_url(&ctx_json);
            match pg_execute(&conn_str, &sql, &params) {
                Ok(()) => Ok(ok_vm(VMValue::Int(0))),
                Err(e) => Ok(err_vm(VMValue::Str(e))),
            }
        }

        "AppCtx.db_query" => {
            // AppCtx.db_query(ctx_json: String, sql: String, params: String) -> Result<String, String>
            let mut it = args.into_iter();
            let ctx_json = vm_string(it.next().ok_or_else(|| "AppCtx.db_query: missing ctx".to_string())?,   "AppCtx.db_query ctx")?;
            let sql      = vm_string(it.next().ok_or_else(|| "AppCtx.db_query: missing sql".to_string())?,   "AppCtx.db_query sql")?;
            let params   = vm_string(it.next().ok_or_else(|| "AppCtx.db_query: missing params".to_string())?, "AppCtx.db_query params")?;
            let conn_str = appctx_db_url(&ctx_json);
            match pg_query(&conn_str, &sql, &params) {
                Ok(json) => Ok(ok_vm(VMValue::Str(json))),
                Err(e)   => Ok(err_vm(VMValue::Str(e))),
            }
        }

        "AppCtx.storage_put" => {
            // AppCtx.storage_put(ctx_json: String, bucket: String, key: String, body: String) -> Result<Unit, String>
            let mut it = args.into_iter();
            let ctx_json = vm_string(it.next().ok_or_else(|| "AppCtx.storage_put: missing ctx".to_string())?,    "AppCtx.storage_put ctx")?;
            let bucket   = vm_string(it.next().ok_or_else(|| "AppCtx.storage_put: missing bucket".to_string())?, "AppCtx.storage_put bucket")?;
            let key      = vm_string(it.next().ok_or_else(|| "AppCtx.storage_put: missing key".to_string())?,    "AppCtx.storage_put key")?;
            let body     = vm_string(it.next().ok_or_else(|| "AppCtx.storage_put: missing body".to_string())?,   "AppCtx.storage_put body")?;
            let config = appctx_aws_config(&ctx_json);
            let base = if let Some(ep) = &config.endpoint_url {
                format!("{}/{}", ep.trim_end_matches('/'), bucket)
            } else {
                format!("https://{}.s3.{}.amazonaws.com", bucket, config.region)
            };
            let url = format!("{}/{}", base, key);
            Ok(match aws_put(&config, "s3", &url, &body) {
                Ok(())  => ok_vm(VMValue::Unit),
                Err(e)  => err_vm(VMValue::Str(e)),
            })
        }

        "AppCtx.io_println" => {
            // AppCtx.io_println(ctx_json: String, msg: String) -> Unit
            let mut it = args.into_iter();
            let _ctx = it.next(); // ctx is not used at runtime
            let msg = vm_string(
                it.next().ok_or_else(|| "AppCtx.io_println: missing msg".to_string())?,
                "AppCtx.io_println msg",
            )?;
            println!("{}", msg);
            Ok(VMValue::Unit)
        }

        // ── forall generators (v17.7.0) ───────────────────────────────────────
        "__forall_gen_int" => {
            let n = match args.into_iter().next() {
                Some(VMValue::Int(n)) => n.max(0) as usize,
                _ => 100,
            };
            let seed_fixed = [0i64, 1, -1, i32::MAX as i64, i32::MIN as i64];
            let mut vals: Vec<VMValue> = seed_fixed.iter().take(n).map(|&v| VMValue::Int(v)).collect();
            if n > seed_fixed.len() {
                let mut state: u64 = 12345;
                for _ in 0..(n - seed_fixed.len()) {
                    state ^= state << 13;
                    state ^= state >> 7;
                    state ^= state << 17;
                    vals.push(VMValue::Int(state as i64));
                }
            }
            Ok(VMValue::List(FavList::new(vals)))
        }
        "__forall_gen_str" => {
            let n = match args.into_iter().next() {
                Some(VMValue::Int(n)) => n.max(0) as usize,
                _ => 100,
            };
            let seed_fixed = ["", " ", "a", "\n", "hello world"];
            let mut vals: Vec<VMValue> = seed_fixed.iter().take(n).map(|&v| VMValue::Str(v.to_string())).collect();
            if n > seed_fixed.len() {
                let mut state: u64 = 12345;
                for _ in 0..(n - seed_fixed.len()) {
                    state ^= state << 13;
                    state ^= state >> 7;
                    state ^= state << 17;
                    let len = (state % 21) as usize; // 0..=20
                    let mut s = String::with_capacity(len);
                    for _ in 0..len {
                        state ^= state << 13;
                        state ^= state >> 7;
                        state ^= state << 17;
                        let ch = (32u8 + (state % 95) as u8) as char; // ASCII 32-126
                        s.push(ch);
                    }
                    vals.push(VMValue::Str(s));
                }
            }
            Ok(VMValue::List(FavList::new(vals)))
        }
        "__forall_gen_bool" => {
            let n = match args.into_iter().next() {
                Some(VMValue::Int(n)) => n.max(0) as usize,
                _ => 100,
            };
            let vals: Vec<VMValue> = (0..n).map(|i| VMValue::Bool(i % 2 == 0)).collect();
            Ok(VMValue::List(FavList::new(vals)))
        }
        "__forall_gen_float" => {
            let n = match args.into_iter().next() {
                Some(VMValue::Int(n)) => n.max(0) as usize,
                _ => 100,
            };
            let seed_fixed: &[f64] = &[0.0, 1.0, -1.0, 0.5, -0.5];
            let mut vals: Vec<VMValue> = seed_fixed.iter().take(n).map(|&v| VMValue::Float(v)).collect();
            if n > seed_fixed.len() {
                let mut state: u64 = 12345;
                for _ in 0..(n - seed_fixed.len()) {
                    state ^= state << 13;
                    state ^= state >> 7;
                    state ^= state << 17;
                    // Map to [-1e6, 1e6] range, exclude NaN/Inf
                    let f = (state as i64 as f64) / (i64::MAX as f64) * 1_000_000.0;
                    if f.is_finite() {
                        vals.push(VMValue::Float(f));
                    } else {
                        vals.push(VMValue::Float(0.0));
                    }
                }
            }
            Ok(VMValue::List(FavList::new(vals)))
        }


        other => Err(format!("unknown builtin: {}", other)),
    }
}

/// Extract the db_url from an AppCtx JSON string.
/// Falls back to the environment variable if parsing fails.
fn appctx_db_url(ctx_json: &str) -> String {
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(ctx_json) {
        if let Some(s) = v.get("db_url").and_then(|x| x.as_str()) {
            if !s.is_empty() {
                return s.to_string();
            }
        }
    }
    pg_conn_str_from_env()
}

/// Build an AwsConfig from an AppCtx JSON string.
/// Merges env-based values (endpoint_url, credentials) with ctx's region/bucket.
fn appctx_aws_config(ctx_json: &str) -> AwsConfig {
    let mut cfg = AwsConfig::from_env();
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(ctx_json) {
        if let Some(region) = v.get("aws_region").and_then(|x| x.as_str()) {
            if !region.is_empty() {
                cfg.region = region.to_string();
            }
        }
    }
    cfg
}

#[cfg(test)]
#[path = "vm_legacy_coverage_tests.rs"]
mod vm_legacy_coverage_tests;

#[cfg(test)]
#[path = "vm_stdlib_tests.rs"]
mod vm_stdlib_tests;

#[cfg(test)]
mod wasm_phase0_builtin_tests {
    use super::{
        SuppressIoGuard, VMValue, io_output_suppressed_for_tests, set_suppress_io, vm_call_builtin,
    };

    #[test]
    fn vm_builtin_io_print_variants_return_unit() {
        let mut emit_log = Vec::new();
        assert_eq!(
            vm_call_builtin(
                "IO.print",
                vec![VMValue::Str("hello".into())],
                &mut emit_log,
                None,
                &std::collections::HashMap::new(),
            )
            .unwrap(),
            VMValue::Unit
        );
        assert_eq!(
            vm_call_builtin(
                "IO.println_int",
                vec![VMValue::Int(42)],
                &mut emit_log,
                None,
                &std::collections::HashMap::new(),
            )
            .unwrap(),
            VMValue::Unit
        );
        assert_eq!(
            vm_call_builtin(
                "IO.println_float",
                vec![VMValue::Float(3.5)],
                &mut emit_log,
                None,
                &std::collections::HashMap::new(),
            )
            .unwrap(),
            VMValue::Unit
        );
        assert_eq!(
            vm_call_builtin(
                "IO.println_bool",
                vec![VMValue::Bool(true)],
                &mut emit_log,
                None,
                &std::collections::HashMap::new(),
            )
            .unwrap(),
            VMValue::Unit
        );
    }

    #[test]
    fn vm_builtin_string_state_helpers() {
        let mut emit_log = Vec::new();
        assert_eq!(
            vm_call_builtin(
                "String.is_url",
                vec![VMValue::Str("https://example.com".into())],
                &mut emit_log,
                None,
                &std::collections::HashMap::new(),
            )
            .unwrap(),
            VMValue::Bool(true)
        );
        assert_eq!(
            vm_call_builtin(
                "String.is_url",
                vec![VMValue::Str("ftp://example.com".into())],
                &mut emit_log,
                None,
                &std::collections::HashMap::new(),
            )
            .unwrap(),
            VMValue::Bool(false)
        );
        assert_eq!(
            vm_call_builtin(
                "String.is_slug",
                vec![VMValue::Str("hello-world-2026".into())],
                &mut emit_log,
                None,
                &std::collections::HashMap::new(),
            )
            .unwrap(),
            VMValue::Bool(true)
        );
        assert_eq!(
            vm_call_builtin(
                "String.is_slug",
                vec![VMValue::Str("Hello world".into())],
                &mut emit_log,
                None,
                &std::collections::HashMap::new(),
            )
            .unwrap(),
            VMValue::Bool(false)
        );
    }

    #[test]
    fn suppress_io_guard_restores_previous_state() {
        set_suppress_io(false);
        assert!(!io_output_suppressed_for_tests());
        {
            let _guard = SuppressIoGuard::new(true);
            assert!(io_output_suppressed_for_tests());
        }
        assert!(!io_output_suppressed_for_tests());
    }
}

// ── v10200_tests (v10.2.0) — Snowflake VM primitives ─────────────────────────
#[cfg(test)]
#[cfg(not(target_arch = "wasm32"))]
mod v10200_tests {
    use super::{VMValue, vm_call_builtin, snowflake_generate_jwt};
    use std::collections::HashMap;

    /// 環境変数未設定時に Snowflake.execute_raw が Err("SNOWFLAKE_ACCOUNT is not set") を返す
    #[test]
    fn snowflake_execute_raw_missing_env_returns_err() {
        // Ensure the env var is absent for this test
        // SAFETY: test-only, no other threads read this env var during this test
        unsafe { std::env::remove_var("SNOWFLAKE_ACCOUNT") };
        let mut emit_log = Vec::new();
        let result = vm_call_builtin(
            "Snowflake.execute_raw",
            vec![VMValue::Str("SELECT 1".to_string())],
            &mut emit_log,
            None,
            &HashMap::new(),
        )
        .expect("call_builtin should not return Err");
        match result {
            VMValue::Variant(tag, inner) => {
                assert_eq!(tag, "err", "expected err variant, got {}", tag);
                match inner.as_deref() {
                    Some(VMValue::Str(msg)) => assert_eq!(
                        msg, "SNOWFLAKE_ACCOUNT is not set",
                        "unexpected error message: {}",
                        msg
                    ),
                    other => panic!("unexpected inner value: {:?}", other),
                }
            }
            other => panic!("expected Variant, got {:?}", other),
        }
    }

    /// 環境変数未設定時に Snowflake.query_raw が Err("SNOWFLAKE_ACCOUNT is not set") を返す
    #[test]
    fn snowflake_query_raw_missing_env_returns_err() {
        // SAFETY: test-only, no other threads read this env var during this test
        unsafe { std::env::remove_var("SNOWFLAKE_ACCOUNT") };
        let mut emit_log = Vec::new();
        let result = vm_call_builtin(
            "Snowflake.query_raw",
            vec![VMValue::Str("SELECT 1".to_string())],
            &mut emit_log,
            None,
            &HashMap::new(),
        )
        .expect("call_builtin should not return Err");
        match result {
            VMValue::Variant(tag, inner) => {
                assert_eq!(tag, "err", "expected err variant, got {}", tag);
                match inner.as_deref() {
                    Some(VMValue::Str(msg)) => assert_eq!(
                        msg, "SNOWFLAKE_ACCOUNT is not set",
                        "unexpected error message: {}",
                        msg
                    ),
                    other => panic!("unexpected inner value: {:?}", other),
                }
            }
            other => panic!("expected Variant, got {:?}", other),
        }
    }

    /// snowflake_generate_jwt が well-formed な JWT（3 パート xxx.yyy.zzz）を返す
    #[test]
    fn snowflake_jwt_well_formed() {
        // Test-only RSA PKCS#8 key (not used in production)
        let test_private_key = "\
-----BEGIN PRIVATE KEY-----\n\
MIIEvAIBADANBgkqhkiG9w0BAQEFAASCBKYwggSiAgEAAoIBAQCp7XR5mt2/jst5\n\
Iv92iejdSHWSpWVlfNvav0Jks5lJluqPMN3RQtOwW/QGpahJ1ogof4/eWIWx36jM\n\
gOZKm5Sy0hKLEChS2yf4wYbcxk2xqR1baBy9cHOWAxkkukznqm2qP24dhwUaFumz\n\
Npnrf35XgKvuqF4eGWk2HhgvXmMOb/i/aBMrQ+sBnHJkiVJMsGyioeEhz9ZxyTK2\n\
4R+lnlboWWkpx8iG9oDKjrhgoKf2KQIwG3vJ5DP9tpMChWo0wltue+oIOeUOdgrB\n\
jWmn+Tmv0YOFD93FH/SNlobWyl/XT7zVaPfPU5HoSedmMFVckr3TUj1TrGaNlEk6\n\
H+lAh9W3AgMBAAECggEAAaQqgNYvGB+P9Y6R+xof5qtBf3YbgZxwHy/Du2dErsJH\n\
Z7SoH9JOayCoPbwx4OyyifmZcSNXvz0Sy07fao8QI54F0dQJH6vAOhXccJt1uqaQ\n\
gwaBaI8Cfstu3bzy6zXpM0DDloNsWDEqdrGrUOv9U2kJwBdeOVozevNVdnep60TD\n\
kJWqww7VX1RiqI7jb3QSlbMQxZxSmRYeKF73xLuOyzxH/i1QxOrAQ24gtwL1D/TB\n\
fSIUrHyb0XrtCdlzjhIBmfc5ySfPwbHFC8r3cejSWnR3dF3PhNg40LU0+3/gmfXJ\n\
dAG7Zn46ZHfM+iWm0MGrFDCAt6hNybgF6ZEZwhbiaQKBgQDbM5gHZxvMuOWiPKIp\n\
93WBE5P24fMZUp8Flw9ecA6a9D4M892yeS1o3l3TcqW2ttf4JxLJ4vzlO2KaeXQI\n\
gsEQOs/7jtXX+XSvwpe2cMqGD+S42nH6rAIrGuTAKd7lxpY1/gccFHbEpshs8zEQ\n\
WDvtUFqOOoNk5ps0qQZtkrbLRQKBgQDGdEMqnRdc51iWWJ31pEqtXqo8enck5KNn\n\
kWlbamiyVLvS7P5UPbOWB4iYb+7HoHOjwtaf4LkUfHdZsBlOefe4aFEJapcLQn4F\n\
1uqvOLE5YIDY44h/v9f1jv695ZiHeh9GN49kvQjuIxiNem1JtHjaRnkjGl1mv7ge\n\
CM25epluywKBgHXY3SlNs9JyrXJ1qrFpSxEkF26pt2qr0rbMqgSZtiB0o0+PZGdp\n\
YpJ4ynS9tH3w+1d8mktT76bGMJLgLRPOSEGTfPG/rxQ4FxXPRoVdSmSc8ti3CIQ+\n\
KcRG5yiw2hcqluNcOTJNhjTffe2lKYGiDkXd53GD39RFbrf3D2+lawUJAoGALN+v\n\
LFyXIs/BDUX+ecPriuZD8iby9+mnNU0BGMWn5OMaEWi7XYsSJ5OOhIGS6ZrTay0s\n\
YLxsvUAjsKkMH92ecRlNcaajfs1LN8DQEkzsbf/vQpu4isJzb7gkzAW1hrTLi5IW\n\
n33LHiXbcGpFegwP47NZwuE8S3aAiHIPKqiZNx8CgYBaAvgw8X3qGUAsJXdRPhss\n\
VOCZsatM+TkHTQpW0cB1WBFuze7HuqkFpQx/3FfPgYAy1+8pQNQc3pLMfNYYgNPO\n\
fp/s2Pd9AIZbqesNpT+3klKnED+oxyq7zT9zzfiK1sHvHytnIxQKWAOdnQTfxblw\n\
/6V76JjLOJAao9hnPCFyZA==\n\
-----END PRIVATE KEY-----";
        let test_fp = "h96et+XrQBbK5r4IuPy+81/5pXTVSjZBBX8aW2910GE=";

        let token = snowflake_generate_jwt("myorg-myaccount", "testuser", test_private_key, test_fp)
            .expect("JWT generation failed");

        // Must be 3-part dot-separated
        let parts: Vec<&str> = token.split('.').collect();
        assert_eq!(parts.len(), 3, "JWT must have 3 parts, got: {}", token);

        // Decode and verify claims (skip signature validation for unit test)
        let mut validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::RS256);
        validation.insecure_disable_signature_validation();
        validation.set_required_spec_claims(&["iss", "sub", "iat", "exp"]);
        let pub_key_pem = openssl_pubkey_from_pem(test_private_key);
        let decoding_key = jsonwebtoken::DecodingKey::from_rsa_pem(pub_key_pem.as_bytes())
            .expect("decode key");
        let data = jsonwebtoken::decode::<serde_json::Value>(&token, &decoding_key, &validation)
            .expect("decode failed");

        let payload = &data.claims;
        let iss = payload["iss"].as_str().expect("iss missing");
        let sub = payload["sub"].as_str().expect("sub missing");
        assert!(payload["iat"].is_number(), "iat must be a number");
        assert!(payload["exp"].is_number(), "exp must be a number");
        assert!(
            iss.contains("SHA256:"),
            "iss must contain SHA256: prefix, got: {}",
            iss
        );
        assert_eq!(
            iss, "MYORG-MYACCOUNT.TESTUSER.SHA256:h96et+XrQBbK5r4IuPy+81/5pXTVSjZBBX8aW2910GE=",
            "iss mismatch"
        );
        assert_eq!(sub, "MYORG-MYACCOUNT.TESTUSER", "sub mismatch");
        let iat = payload["iat"].as_i64().unwrap();
        let exp = payload["exp"].as_i64().unwrap();
        assert_eq!(exp - iat, 3600, "exp should be iat + 3600");
    }

    /// Extract the RSA public key PEM from a PKCS#8 private key PEM using openssl subprocess
    fn openssl_pubkey_from_pem(private_key_pem: &str) -> String {
        use std::io::Write;
        use std::process::{Command, Stdio};
        let mut child = Command::new("openssl")
            .args(["rsa", "-pubout"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("openssl spawn");
        child
            .stdin
            .as_mut()
            .unwrap()
            .write_all(private_key_pem.as_bytes())
            .unwrap();
        let output = child.wait_with_output().expect("openssl wait");
        String::from_utf8(output.stdout).expect("openssl output utf8")
    }
}
