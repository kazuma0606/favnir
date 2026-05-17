// Favnir Local Rune Registry — v4.12.0
// Stores published Rune packages in ~/.fav/registry/<name>/<version>/

use std::path::{Path, PathBuf};

// ── PackageMeta ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct PackageMeta {
    pub name:        String,
    pub version:     String,
    pub description: String,
    pub author:      String,
    pub license:     String,
    pub published:   String,
    pub files:       Vec<String>,
}

// ── PackageEntry ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct PackageEntry {
    pub name:     String,
    pub versions: Vec<String>, // descending semver order
}

// ── Registry ──────────────────────────────────────────────────────────────────

pub struct Registry {
    pub root: PathBuf,
}

impl Registry {
    /// Create a registry using the default path (~/.fav/registry/).
    pub fn new() -> Self {
        Self { root: registry_root() }
    }

    /// Create a registry with an explicit root (for testing).
    pub fn with_root(root: PathBuf) -> Self {
        Self { root }
    }

    /// Return all installed versions of a package, descending semver order.
    pub fn installed_versions(&self, name: &str) -> Vec<String> {
        let pkg_dir = self.root.join(name);
        if !pkg_dir.exists() {
            return vec![];
        }
        let mut versions: Vec<String> = std::fs::read_dir(&pkg_dir)
            .into_iter()
            .flatten()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir())
            .filter_map(|e| e.file_name().into_string().ok())
            .collect();
        versions.sort_by(|a, b| semver_cmp(b, a));
        versions
    }

    /// Resolve a version constraint ("*", "^x.y.z", or exact) for a package.
    pub fn resolve_version(&self, name: &str, constraint: &str) -> Option<String> {
        let available = self.installed_versions(name);
        match constraint.trim() {
            "*" => available.into_iter().next(),
            c if c.starts_with('^') => {
                let base = &c[1..];
                available.into_iter().find(|v| semver_compatible(v, base))
            }
            exact => available.into_iter().find(|v| v == exact),
        }
    }

    /// List all packages in the registry, sorted by name.
    pub fn list(&self) -> Vec<PackageEntry> {
        if !self.root.exists() {
            return vec![];
        }
        let mut entries: Vec<PackageEntry> = std::fs::read_dir(&self.root)
            .into_iter()
            .flatten()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir())
            .filter_map(|e| {
                let name = e.file_name().into_string().ok()?;
                let versions = self.installed_versions(&name);
                if versions.is_empty() { return None; }
                Some(PackageEntry { name, versions })
            })
            .collect();
        entries.sort_by(|a, b| a.name.cmp(&b.name));
        entries
    }

    /// Search packages by name substring.
    pub fn search(&self, query: &str) -> Vec<PackageEntry> {
        self.list().into_iter().filter(|e| e.name.contains(query)).collect()
    }

    /// Get metadata for the latest version of a package.
    pub fn info(&self, name: &str) -> Option<PackageMeta> {
        let versions = self.installed_versions(name);
        let latest = versions.first()?;
        let pkg_dir = self.root.join(name).join(latest);
        let meta_str = std::fs::read_to_string(pkg_dir.join("fav.pkg.toml")).ok()?;
        let mut meta = parse_pkg_toml(&meta_str, name, latest);
        meta.files = collect_rune_files(&pkg_dir.join("runes"));
        Some(meta)
    }

    /// Publish a package to the local registry.
    /// `rune_files` is a list of (relative path, file contents).
    pub fn publish(
        &self,
        meta: &PackageMeta,
        rune_files: &[(String, Vec<u8>)],
    ) -> Result<(), String> {
        let dest = self.root.join(&meta.name).join(&meta.version);
        std::fs::create_dir_all(&dest).map_err(|e| e.to_string())?;

        // Write fav.pkg.toml
        let toml_content = format_pkg_toml(meta);
        std::fs::write(dest.join("fav.pkg.toml"), toml_content)
            .map_err(|e| e.to_string())?;

        // Write rune files
        for (rel_path, content) in rune_files {
            let file_dest = dest.join(rel_path);
            if let Some(parent) = file_dest.parent() {
                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            std::fs::write(&file_dest, content).map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    /// Install a specific version from the registry into `dest_runes_dir/<name>/`.
    pub fn install(&self, name: &str, version: &str, dest_runes_dir: &Path) -> Result<(), String> {
        let src = self.root.join(name).join(version).join("runes").join(name);
        if !src.exists() {
            return Err(format!("{}@{} not found in registry (expected {:?})", name, version, src));
        }
        let dest_pkg = dest_runes_dir.join(name);
        copy_dir_all(&src, &dest_pkg).map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Return the directory path for the latest version of a rune (for resolver fallback).
    pub fn rune_path(&self, name: &str) -> Option<PathBuf> {
        let versions = self.installed_versions(name);
        let latest = versions.first()?;
        let path = self.root.join(name).join(latest).join("runes").join(name);
        if path.exists() { Some(path) } else { None }
    }
}

// ── public helpers ────────────────────────────────────────────────────────────

/// Collect all .fav files under `dir` as (relative_path_string, bytes).
pub fn collect_fav_files_in(dir: &Path) -> Vec<(String, Vec<u8>)> {
    if !dir.exists() {
        return vec![];
    }
    walkdir::WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("fav"))
        .filter_map(|e| {
            let rel = e.path().strip_prefix(dir).ok()?;
            let rel_str = rel.to_string_lossy().replace('\\', "/");
            let content = std::fs::read(e.path()).ok()?;
            Some((rel_str, content))
        })
        .collect()
}

// ── private helpers ───────────────────────────────────────────────────────────

fn registry_root() -> PathBuf {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".fav").join("registry")
}

fn format_pkg_toml(meta: &PackageMeta) -> String {
    format!(
        "name        = \"{}\"\nversion     = \"{}\"\ndescription = \"{}\"\nauthor      = \"{}\"\nlicense     = \"{}\"\npublished   = \"{}\"\n",
        meta.name, meta.version, meta.description, meta.author, meta.license, meta.published
    )
}

fn parse_pkg_toml(content: &str, name_fallback: &str, version_fallback: &str) -> PackageMeta {
    let mut name        = name_fallback.to_string();
    let mut version     = version_fallback.to_string();
    let mut description = String::new();
    let mut author      = String::new();
    let mut license     = String::new();
    let mut published   = String::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some((k, v)) = trimmed.split_once('=') {
            let k = k.trim();
            let v = v.trim().trim_matches('"');
            match k {
                "name"        => name        = v.to_string(),
                "version"     => version     = v.to_string(),
                "description" => description = v.to_string(),
                "author"      => author      = v.to_string(),
                "license"     => license     = v.to_string(),
                "published"   => published   = v.to_string(),
                _ => {}
            }
        }
    }
    PackageMeta { name, version, description, author, license, published, files: vec![] }
}

fn collect_rune_files(runes_dir: &Path) -> Vec<String> {
    if !runes_dir.exists() {
        return vec![];
    }
    walkdir::WalkDir::new(runes_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("fav"))
        .filter_map(|e| {
            let rel = e.path().strip_prefix(runes_dir).ok()?;
            Some(rel.to_string_lossy().replace('\\', "/"))
        })
        .collect()
}

fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let dst_path = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst_path)?;
        } else {
            std::fs::copy(entry.path(), dst_path)?;
        }
    }
    Ok(())
}

// ── semver helpers ────────────────────────────────────────────────────────────

fn parse_semver(v: &str) -> (u32, u32, u32) {
    let parts: Vec<u32> = v.split('.').filter_map(|p| p.parse().ok()).collect();
    (
        parts.first().copied().unwrap_or(0),
        parts.get(1).copied().unwrap_or(0),
        parts.get(2).copied().unwrap_or(0),
    )
}

fn version_ge(a: &str, b: &str) -> bool {
    parse_semver(a) >= parse_semver(b)
}

fn semver_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    parse_semver(a).cmp(&parse_semver(b))
}

fn semver_compatible(version: &str, base: &str) -> bool {
    let (vma, vmi, _) = parse_semver(version);
    let (bma, bmi, _) = parse_semver(base);
    if bma == 0 {
        // ^0.x.y — minor must match, version >= base
        vma == 0 && vmi == bmi && version_ge(version, base)
    } else {
        // ^x.y.z (x >= 1) — major must match, version >= base
        vma == bma && version_ge(version, base)
    }
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_pkg(reg: &Registry, name: &str, version: &str, files: &[(&str, &str)]) {
        let meta = PackageMeta {
            name:        name.to_string(),
            version:     version.to_string(),
            description: format!("{} rune", name),
            author:      "test".to_string(),
            license:     "MIT".to_string(),
            published:   "2026-05-17T00:00:00Z".to_string(),
            files:       vec![],
        };
        let rune_files: Vec<(String, Vec<u8>)> = files
            .iter()
            .map(|(p, c)| (p.to_string(), c.as_bytes().to_vec()))
            .collect();
        reg.publish(&meta, &rune_files).unwrap();
    }

    #[test]
    fn registry_resolve_exact_version() {
        let tmp = TempDir::new().unwrap();
        let reg = Registry::with_root(tmp.path().to_path_buf());
        make_pkg(&reg, "csv", "1.0.0", &[("runes/csv/csv.fav", "// csv rune")]);
        make_pkg(&reg, "csv", "1.1.0", &[("runes/csv/csv.fav", "// csv rune v2")]);
        // Both versions should be installed
        let found = reg.installed_versions("csv");
        assert!(found.contains(&"1.0.0".to_string()));
        assert!(found.contains(&"1.1.0".to_string()));
        // Exact resolve picks the precise version requested
        assert_eq!(reg.resolve_version("csv", "1.0.0"), Some("1.0.0".to_string()));
        assert_eq!(reg.resolve_version("csv", "1.1.0"), Some("1.1.0".to_string()));
        // Non-existent version returns None
        assert_eq!(reg.resolve_version("csv", "9.9.9"), None);
    }

    #[test]
    fn registry_resolve_caret_major() {
        let tmp = TempDir::new().unwrap();
        let reg = Registry::with_root(tmp.path().to_path_buf());
        make_pkg(&reg, "http", "1.0.0", &[("runes/http/http.fav", "")]);
        make_pkg(&reg, "http", "1.2.0", &[("runes/http/http.fav", "")]);
        make_pkg(&reg, "http", "2.0.0", &[("runes/http/http.fav", "")]);
        // ^1.0.0 should pick latest 1.x
        assert_eq!(reg.resolve_version("http", "^1.0.0"), Some("1.2.0".to_string()));
    }

    #[test]
    fn registry_resolve_caret_minor() {
        let tmp = TempDir::new().unwrap();
        let reg = Registry::with_root(tmp.path().to_path_buf());
        make_pkg(&reg, "auth", "0.3.0", &[("runes/auth/auth.fav", "")]);
        make_pkg(&reg, "auth", "0.3.5", &[("runes/auth/auth.fav", "")]);
        make_pkg(&reg, "auth", "0.4.0", &[("runes/auth/auth.fav", "")]);
        // ^0.3.0 must NOT pick 0.4.0
        assert_eq!(reg.resolve_version("auth", "^0.3.0"), Some("0.3.5".to_string()));
    }

    #[test]
    fn registry_resolve_wildcard() {
        let tmp = TempDir::new().unwrap();
        let reg = Registry::with_root(tmp.path().to_path_buf());
        make_pkg(&reg, "util", "0.1.0", &[("runes/util/util.fav", "")]);
        make_pkg(&reg, "util", "2.0.0", &[("runes/util/util.fav", "")]);
        make_pkg(&reg, "util", "1.5.0", &[("runes/util/util.fav", "")]);
        // * picks the latest
        assert_eq!(reg.resolve_version("util", "*"), Some("2.0.0".to_string()));
    }

    #[test]
    fn registry_resolve_not_found() {
        let tmp = TempDir::new().unwrap();
        let reg = Registry::with_root(tmp.path().to_path_buf());
        assert_eq!(reg.resolve_version("nonexistent", "*"), None);
        assert_eq!(reg.resolve_version("nonexistent", "1.0.0"), None);
    }

    #[test]
    fn registry_publish_creates_files() {
        let tmp = TempDir::new().unwrap();
        let reg = Registry::with_root(tmp.path().to_path_buf());
        make_pkg(&reg, "csv", "1.0.0", &[
            ("runes/csv/parse.fav", "// parse"),
            ("runes/csv/csv.fav",   "// barrel"),
        ]);
        assert!(tmp.path().join("csv/1.0.0/fav.pkg.toml").exists());
        assert!(tmp.path().join("csv/1.0.0/runes/csv/parse.fav").exists());
        assert!(tmp.path().join("csv/1.0.0/runes/csv/csv.fav").exists());
    }

    #[test]
    fn registry_install_copies_rune() {
        let tmp = TempDir::new().unwrap();
        let reg = Registry::with_root(tmp.path().to_path_buf());
        make_pkg(&reg, "csv", "1.0.0", &[
            ("runes/csv/csv.fav", "// csv barrel"),
        ]);
        let dest_runes = tmp.path().join("project/runes");
        std::fs::create_dir_all(&dest_runes).unwrap();
        reg.install("csv", "1.0.0", &dest_runes).unwrap();
        assert!(dest_runes.join("csv/csv.fav").exists());
    }

    #[test]
    fn registry_list_returns_all() {
        let tmp = TempDir::new().unwrap();
        let reg = Registry::with_root(tmp.path().to_path_buf());
        make_pkg(&reg, "csv",   "1.0.0", &[("runes/csv/csv.fav",     "")]);
        make_pkg(&reg, "email", "0.3.1", &[("runes/email/email.fav", "")]);
        let list = reg.list();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].name, "csv");
        assert_eq!(list[1].name, "email");
    }
}
