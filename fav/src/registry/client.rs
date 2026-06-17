//! HTTP client for the Favnir package registry (v17.8.0).
//! Set `REGISTRY_MOCK=1` to return deterministic mock responses (used in tests).

/// Package metadata returned by the registry API.
#[derive(Debug, Clone)]
pub struct PackageInfo {
    pub name: String,
    /// All available versions, ascending order.
    pub versions: Vec<String>,
    /// The latest stable version.
    pub latest: String,
}

/// Client for the Favnir package registry.
pub struct RegistryClient {
    pub base_url: String,
    pub token: Option<String>,
}

impl RegistryClient {
    /// Create a client for the given registry URL.
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            token: None,
        }
    }

    /// Load a client using the default registry URL and any stored credentials.
    pub fn default_client() -> Self {
        let base_url = std::env::var("FAVNIR_REGISTRY_URL")
            .unwrap_or_else(|_| "https://registry.favnir.dev".to_string());
        let token = load_credentials_token();
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            token,
        }
    }

    /// Fetch package info from the registry.
    /// When `REGISTRY_MOCK=1` is set, returns deterministic mock data.
    pub fn fetch_package(&self, name: &str) -> Result<PackageInfo, String> {
        if std::env::var("REGISTRY_MOCK").is_ok() {
            return Ok(PackageInfo {
                name: name.to_string(),
                versions: vec![
                    "1.0.0".to_string(),
                    "2.0.0".to_string(),
                    "2.1.0".to_string(),
                ],
                latest: "2.1.0".to_string(),
            });
        }

        let url = format!("{}/packages/{}", self.base_url, name);
        let resp = ureq::get(&url)
            .call()
            .map_err(|e| format!("registry request failed: {}", e))?;

        let body = resp
            .into_string()
            .map_err(|e| format!("failed to read response: {}", e))?;

        parse_package_info_json(&body, name)
    }

    /// Publish the current package to the registry.
    /// When `dry_run` is true, prints what would be published but does not send.
    pub fn publish(
        &self,
        pkg_name: &str,
        version: &str,
        dry_run: bool,
    ) -> Result<(), String> {
        if dry_run {
            println!("[dry-run] Would publish {}@{} to {}", pkg_name, version, self.base_url);
            return Ok(());
        }

        let token = self
            .token
            .as_deref()
            .ok_or_else(|| "E0330: no auth token — run `fav login` first".to_string())?;

        let url = format!("{}/packages", self.base_url);
        let body = format!(
            "{{\"name\":\"{}\",\"version\":\"{}\"}}",
            pkg_name, version
        );

        ureq::post(&url)
            .set("Authorization", &format!("Bearer {}", token))
            .set("Content-Type", "application/json")
            .send_string(&body)
            .map_err(|e| format!("publish failed: {}", e))?;

        Ok(())
    }
}

// ── helpers ───────────────────────────────────────────────────────────────────

fn parse_package_info_json(body: &str, name_fallback: &str) -> Result<PackageInfo, String> {
    // Minimal hand-rolled JSON extraction (avoids serde_json for a simple structure).
    let extract = |key: &str| -> Option<String> {
        let needle = format!("\"{}\":", key);
        let pos = body.find(&needle)?;
        let rest = &body[pos + needle.len()..].trim_start_matches([' ', '\t']);
        if rest.starts_with('"') {
            let inner = &rest[1..];
            let end = inner.find('"')?;
            Some(inner[..end].to_string())
        } else {
            None
        }
    };

    let name = extract("name").unwrap_or_else(|| name_fallback.to_string());
    let latest = extract("latest").unwrap_or_default();

    // Extract versions array: "versions": ["1.0.0", "2.0.0"]
    let versions = if let Some(pos) = body.find("\"versions\":") {
        let rest = &body[pos + "\"versions\":".len()..].trim_start_matches([' ', '\t']);
        if let Some(start) = rest.find('[') {
            let rest = &rest[start + 1..];
            if let Some(end) = rest.find(']') {
                rest[..end]
                    .split(',')
                    .map(|s| s.trim().trim_matches('"').to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            } else {
                vec![]
            }
        } else {
            vec![]
        }
    } else {
        vec![]
    };

    Ok(PackageInfo { name, versions, latest })
}

/// Load the auth token from `~/.fav/credentials` if it exists.
fn load_credentials_token() -> Option<String> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .ok()?;
    let path = std::path::PathBuf::from(home).join(".fav").join("credentials");
    let content = std::fs::read_to_string(path).ok()?;
    for line in content.lines() {
        if let Some((k, v)) = line.split_once('=') {
            if k.trim() == "token" {
                return Some(v.trim().trim_matches('"').to_string());
            }
        }
    }
    None
}

/// Save an auth token to `~/.fav/credentials`.
pub fn save_credentials_token(token: &str) -> Result<(), String> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map_err(|_| "could not determine home directory".to_string())?;
    let dir = std::path::PathBuf::from(home).join(".fav");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join("credentials");
    std::fs::write(path, format!("token = \"{}\"\n", token)).map_err(|e| e.to_string())
}
