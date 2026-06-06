// rune_cmd.rs — `fav rune` / `rune` package manager commands (v5.3.0)

use std::collections::BTreeMap;
use std::io::Write as IoWrite;
use std::path::{Path, PathBuf};

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

const REGISTRY_URL: &str = "https://32qp3qwhdh.execute-api.ap-northeast-1.amazonaws.com";
const FAV_CLIENT_TOKEN: &str = "fav-registry-v1-dk9p2mxw4qhz";

// ── Registry HTTP helpers ─────────────────────────────────────────────────────

fn registry_get(path: &str) -> Result<String, String> {
    let url = format!("{}{}", REGISTRY_URL, path);
    match ureq::get(&url).set("X-Fav-Token", FAV_CLIENT_TOKEN).call() {
        Ok(resp) => resp.into_string().map_err(|e| format!("read error: {}", e)),
        Err(ureq::Error::Status(code, resp)) => {
            let text = resp.into_string().unwrap_or_default();
            Err(format!("HTTP {}: {}", code, text))
        }
        Err(e) => Err(format!("network error: {}", e)),
    }
}

fn registry_get_bytes(path: &str) -> Result<Vec<u8>, String> {
    use std::io::Read;
    let url = format!("{}{}", REGISTRY_URL, path);
    match ureq::get(&url).set("X-Fav-Token", FAV_CLIENT_TOKEN).call() {
        Ok(resp) => {
            let mut buf = Vec::new();
            resp.into_reader()
                .read_to_end(&mut buf)
                .map_err(|e| format!("read error: {}", e))?;
            Ok(buf)
        }
        Err(ureq::Error::Status(code, resp)) => {
            let text = resp.into_string().unwrap_or_default();
            Err(format!("HTTP {}: {}", code, text))
        }
        Err(e) => Err(format!("network error: {}", e)),
    }
}

fn registry_post(path: &str, body: &str, auth: &str) -> Result<(u16, String), String> {
    let url = format!("{}{}", REGISTRY_URL, path);
    match ureq::post(&url)
        .set("Content-Type", "application/json")
        .set("Authorization", auth)
        .set("X-Fav-Token", FAV_CLIENT_TOKEN)
        .send_string(body)
    {
        Ok(resp) => {
            let status = resp.status();
            let text = resp.into_string().unwrap_or_default();
            Ok((status, text))
        }
        Err(ureq::Error::Status(code, resp)) => {
            let text = resp.into_string().unwrap_or_default();
            Ok((code, text))
        }
        Err(e) => Err(format!("network error: {}", e)),
    }
}

// ── rune.toml helpers ─────────────────────────────────────────────────────────

#[derive(Default)]
struct RuneMeta {
    name: String,
    version: String,
    description: String,
}

/// Read the [rune] section from a rune.toml file.
fn read_rune_meta(path: &Path) -> Option<RuneMeta> {
    let content = std::fs::read_to_string(path).ok()?;
    let mut meta = RuneMeta::default();
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
                let v = v.trim().trim_matches('"');
                match k.trim() {
                    "name" => meta.name = v.to_string(),
                    "version" => meta.version = v.to_string(),
                    "description" => meta.description = v.to_string(),
                    _ => {}
                }
            }
        }
    }
    if meta.name.is_empty() {
        None
    } else {
        Some(meta)
    }
}

/// Read [dependencies] section: returns BTreeMap<name, version>.
fn read_deps(path: &Path) -> BTreeMap<String, String> {
    let mut deps = BTreeMap::new();
    let Ok(content) = std::fs::read_to_string(path) else {
        return deps;
    };
    let mut in_deps = false;
    for line in content.lines() {
        let t = line.trim();
        if t == "[dependencies]" {
            in_deps = true;
            continue;
        }
        if t.starts_with('[') {
            in_deps = false;
            continue;
        }
        if in_deps {
            if let Some((k, v)) = t.split_once('=') {
                let name = k.trim().to_string();
                let version = v.trim().trim_matches('"').to_string();
                if !name.is_empty() && !version.is_empty() {
                    deps.insert(name, version);
                }
            }
        }
    }
    deps
}

/// Rewrite (or create) the [dependencies] section in rune.toml.
fn write_deps(path: &Path, deps: &BTreeMap<String, String>) {
    let existing = std::fs::read_to_string(path).unwrap_or_default();
    let mut out = String::new();

    if existing.contains("[dependencies]") {
        let mut in_deps = false;
        let mut wrote = false;
        for line in existing.lines() {
            if line.trim() == "[dependencies]" {
                if !wrote {
                    out.push_str("[dependencies]\n");
                    for (k, v) in deps {
                        out.push_str(&format!("{} = \"{}\"\n", k, v));
                    }
                    wrote = true;
                }
                in_deps = true;
                continue;
            }
            if in_deps && line.trim().starts_with('[') {
                in_deps = false;
            }
            if !in_deps {
                out.push_str(line);
                out.push('\n');
            }
        }
    } else {
        out.push_str(&existing);
        if !out.is_empty() && !out.ends_with('\n') {
            out.push('\n');
        }
        out.push_str("[dependencies]\n");
        for (k, v) in deps {
            out.push_str(&format!("{} = \"{}\"\n", k, v));
        }
    }

    let _ = std::fs::write(path, out);
}

fn add_dep(path: &Path, name: &str, version: &str) {
    let mut deps = read_deps(path);
    deps.insert(name.to_string(), version.to_string());
    write_deps(path, &deps);
}

fn remove_dep(path: &Path, name: &str) {
    let mut deps = read_deps(path);
    deps.remove(name);
    write_deps(path, &deps);
}

// ── Zip / unzip helpers ───────────────────────────────────────────────────────

fn unzip_to_dir(bytes: &[u8], dest: &Path) -> Result<(), String> {
    use std::io::Read;
    let cursor = std::io::Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(cursor).map_err(|e| format!("invalid zip: {}", e))?;
    std::fs::create_dir_all(dest).map_err(|e| format!("mkdir error: {}", e))?;
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(|e| e.to_string())?;
        let out_path = dest.join(entry.name());
        if entry.name().ends_with('/') {
            std::fs::create_dir_all(&out_path).map_err(|e| e.to_string())?;
        } else {
            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            let mut buf = Vec::new();
            entry.read_to_end(&mut buf).map_err(|e| e.to_string())?;
            std::fs::write(&out_path, &buf).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

fn zip_fav_files(dir: &Path) -> Result<Vec<u8>, String> {
    use walkdir::WalkDir;
    let buf = std::io::Cursor::new(Vec::new());
    let mut zip = zip::ZipWriter::new(buf);
    let options =
        zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        let is_fav = path.extension().and_then(|e| e.to_str()) == Some("fav");
        let is_rune_toml = path.file_name().and_then(|n| n.to_str()) == Some("rune.toml");
        if path.is_file() && (is_fav || is_rune_toml) {
            let rel = path.strip_prefix(dir).map_err(|e| e.to_string())?;
            let name = rel.to_string_lossy().replace('\\', "/");
            zip.start_file(name, options).map_err(|e| e.to_string())?;
            let content = std::fs::read(path).map_err(|e| e.to_string())?;
            zip.write_all(&content).map_err(|e| e.to_string())?;
        }
    }
    let cursor = zip.finish().map_err(|e| e.to_string())?;
    Ok(cursor.into_inner())
}

// ── JSON helpers ──────────────────────────────────────────────────────────────

fn json_str(obj: &serde_json::Value, key: &str) -> String {
    obj.get(key)
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string()
}

// ── Entry point ───────────────────────────────────────────────────────────────

pub fn cmd_rune(args: &[String]) {
    match args.first().map(|s| s.as_str()) {
        Some("install") => cmd_rune_install(&args[1..]),
        Some("uninstall") => cmd_rune_uninstall(&args[1..]),
        Some("list") => cmd_rune_list(),
        Some("info") => cmd_rune_info(&args[1..]),
        Some("search") => cmd_rune_search(&args[1..]),
        Some("update") => cmd_rune_update(&args[1..]),
        Some("publish") => cmd_rune_publish(),
        Some("help") | Some("-h") | Some("--help") => print_rune_help(),
        _ => {
            eprintln!("Usage: rune <install|uninstall|list|info|search|update|publish>");
            eprintln!("Run 'rune help' for more information.");
            std::process::exit(1);
        }
    }
}

fn print_rune_help() {
    print!(
        "\
rune - Favnir package manager

COMMANDS:
    install [name[@version]]...   Install rune(s) into ./rune_modules/
    uninstall <name>...           Remove rune(s) from ./rune_modules/
    list                          List installed runes
    info <name>                   Show rune info from registry
    search [query]                Search runes in registry
    update [name]                 Update rune(s) to latest version
    publish                       Publish rune from ./rune.toml
    help                          Show this help

EXAMPLES:
    rune install csv
    rune install csv@0.2.0
    rune info parquet
    rune search par
    rune publish
"
    );
}

// ── install ───────────────────────────────────────────────────────────────────

fn cmd_rune_install(args: &[String]) {
    if args.is_empty() {
        // Install all deps from rune.toml
        let toml_path = PathBuf::from("rune.toml");
        let deps = read_deps(&toml_path);
        if deps.is_empty() {
            println!("No dependencies in rune.toml");
            return;
        }
        for (name, version) in deps {
            install_one(&name, Some(&version));
        }
        return;
    }
    for arg in args {
        if let Some(at) = arg.find('@') {
            install_one(&arg[..at], Some(&arg[at + 1..]));
        } else {
            install_one(arg, None);
        }
    }
}

fn install_one(name: &str, version: Option<&str>) {
    let version = match version {
        Some(v) => v.to_string(),
        None => {
            print!("Resolving {}... ", name);
            let _ = std::io::stdout().flush();
            match registry_get(&format!("/runes/{}", name)) {
                Err(e) => {
                    eprintln!("error: {}", e);
                    std::process::exit(1);
                }
                Ok(body) => match serde_json::from_str::<serde_json::Value>(&body) {
                    Err(_) => {
                        eprintln!("error: invalid response from registry");
                        std::process::exit(1);
                    }
                    Ok(obj) => {
                        let v = json_str(&obj, "version");
                        if v.is_empty() {
                            eprintln!("error: rune '{}' not found", name);
                            std::process::exit(1);
                        }
                        println!("{}", v);
                        v
                    }
                },
            }
        }
    };

    print!("Installing {}@{}... ", name, version);
    let _ = std::io::stdout().flush();

    let url = format!("/runes/{}/download?version={}", name, version);
    // API Gateway decodes isBase64Encoded automatically — response is raw zip bytes.
    let bytes = match registry_get_bytes(&url) {
        Err(e) => {
            eprintln!("error: {}", e);
            std::process::exit(1);
        }
        Ok(b) => b,
    };

    let dest = PathBuf::from("rune_modules").join(name);
    if dest.exists() {
        let _ = std::fs::remove_dir_all(&dest);
    }

    if let Err(e) = unzip_to_dir(&bytes, &dest) {
        eprintln!("error: {}", e);
        std::process::exit(1);
    }

    println!("done");

    add_dep(&PathBuf::from("rune.toml"), name, &version);
}

// ── uninstall ─────────────────────────────────────────────────────────────────

fn cmd_rune_uninstall(args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: rune uninstall <name>...");
        std::process::exit(1);
    }
    let toml_path = PathBuf::from("rune.toml");
    for name in args {
        let dest = PathBuf::from("rune_modules").join(name);
        if dest.exists() {
            match std::fs::remove_dir_all(&dest) {
                Ok(_) => println!("Removed {}", name),
                Err(e) => eprintln!("error removing {}: {}", name, e),
            }
        } else {
            println!("{} is not installed", name);
        }
        remove_dep(&toml_path, name);
    }
    if toml_path.exists() {
        println!("Updated rune.toml");
    }
}

// ── list ──────────────────────────────────────────────────────────────────────

fn cmd_rune_list() {
    let modules_dir = PathBuf::from("rune_modules");
    if !modules_dir.exists() {
        println!("No runes installed (rune_modules/ not found)");
        return;
    }
    let mut entries: Vec<_> = std::fs::read_dir(&modules_dir)
        .map(|rd| rd.filter_map(|e| e.ok()).collect())
        .unwrap_or_default();
    entries.sort_by_key(|e| e.file_name());
    if entries.is_empty() {
        println!("No runes installed");
        return;
    }
    println!("{:<20} {:<12} {}", "NAME", "VERSION", "DESCRIPTION");
    println!("{}", "-".repeat(60));
    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            let name = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let toml_path = path.join("rune.toml");
            let (version, description) = if let Some(meta) = read_rune_meta(&toml_path) {
                (meta.version, meta.description)
            } else {
                ("?".to_string(), String::new())
            };
            println!("{:<20} {:<12} {}", name, version, description);
        }
    }
}

// ── info ──────────────────────────────────────────────────────────────────────

fn cmd_rune_info(args: &[String]) {
    let name = match args.first() {
        Some(n) => n,
        None => {
            eprintln!("Usage: rune info <name>");
            std::process::exit(1);
        }
    };
    match registry_get(&format!("/runes/{}", name)) {
        Err(e) => {
            eprintln!("error: {}", e);
            std::process::exit(1);
        }
        Ok(body) => match serde_json::from_str::<serde_json::Value>(&body) {
            Err(_) => eprintln!("error: invalid response"),
            Ok(obj) => {
                println!("name:        {}", json_str(&obj, "name"));
                println!("version:     {}", json_str(&obj, "version"));
                println!("description: {}", json_str(&obj, "description"));
            }
        },
    }
}

// ── search ────────────────────────────────────────────────────────────────────

fn cmd_rune_search(args: &[String]) {
    let query = args.first().map(|s| s.as_str()).unwrap_or("");
    match registry_get("/runes") {
        Err(e) => {
            eprintln!("error: {}", e);
            std::process::exit(1);
        }
        Ok(body) => match serde_json::from_str::<Vec<serde_json::Value>>(&body) {
            Err(_) => eprintln!("error: invalid response"),
            Ok(items) => {
                let filtered: Vec<_> = items
                    .iter()
                    .filter(|item| query.is_empty() || json_str(item, "name").contains(query))
                    .collect();
                if filtered.is_empty() {
                    println!("No runes found matching '{}'", query);
                    return;
                }
                println!("{:<20} {:<12} {}", "NAME", "VERSION", "DESCRIPTION");
                println!("{}", "-".repeat(60));
                for item in filtered {
                    println!(
                        "{:<20} {:<12} {}",
                        json_str(item, "name"),
                        json_str(item, "version"),
                        json_str(item, "description")
                    );
                }
            }
        },
    }
}

// ── update ────────────────────────────────────────────────────────────────────

fn cmd_rune_update(args: &[String]) {
    let toml_path = PathBuf::from("rune.toml");
    let deps = read_deps(&toml_path);

    let names: Vec<String> = if args.is_empty() {
        deps.keys().cloned().collect()
    } else {
        args.to_vec()
    };

    if names.is_empty() {
        println!("No dependencies in rune.toml");
        return;
    }

    for name in &names {
        let current = deps.get(name).map(|s| s.as_str()).unwrap_or("");
        match registry_get(&format!("/runes/{}", name)) {
            Err(e) => eprintln!("error fetching {}: {}", name, e),
            Ok(body) => {
                if let Ok(obj) = serde_json::from_str::<serde_json::Value>(&body) {
                    let latest = json_str(&obj, "version");
                    if latest.is_empty() {
                        eprintln!("{}: not found in registry", name);
                    } else if latest == current {
                        println!("{}: already at latest ({})", name, current);
                    } else {
                        println!("Updating {} {} -> {}", name, current, latest);
                        install_one(name, Some(&latest));
                    }
                }
            }
        }
    }
}

// ── publish ───────────────────────────────────────────────────────────────────

fn cmd_rune_publish() {
    let toml_path = PathBuf::from("rune.toml");
    let meta = match read_rune_meta(&toml_path) {
        Some(m) => m,
        None => {
            eprintln!("error: rune.toml not found or missing [rune] section");
            std::process::exit(1);
        }
    };

    print!("Publishing {}@{}... ", meta.name, meta.version);
    let _ = std::io::stdout().flush();

    let zip_bytes = match zip_fav_files(Path::new(".")) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("error creating zip: {}", e);
            std::process::exit(1);
        }
    };

    let zip_b64 = BASE64.encode(&zip_bytes);

    let body = serde_json::json!({
        "version":     meta.version,
        "description": meta.description,
        "zip":         zip_b64,
    })
    .to_string();

    let token = match std::env::var("FAV_PUBLISH_TOKEN") {
        Ok(t) => t,
        Err(_) => {
            eprintln!("error: FAV_PUBLISH_TOKEN environment variable is not set");
            eprintln!("       Set it before running: export FAV_PUBLISH_TOKEN=<token>");
            std::process::exit(1);
        }
    };
    let auth = format!("Bearer {}", token);

    match registry_post(&format!("/runes/{}", meta.name), &body, &auth) {
        Err(e) => {
            eprintln!("error: {}", e);
            std::process::exit(1);
        }
        Ok((status, resp)) => {
            if status == 200 || status == 201 {
                println!("done");
            } else {
                eprintln!("failed (HTTP {}): {}", status, resp);
                std::process::exit(1);
            }
        }
    }
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_rune_toml(dir: &TempDir, content: &str) -> PathBuf {
        let path = dir.path().join("rune.toml");
        std::fs::write(&path, content).unwrap();
        path
    }

    #[test]
    fn test_read_rune_meta() {
        let dir = TempDir::new().unwrap();
        let path = make_rune_toml(
            &dir,
            r#"
[rune]
name = "csv"
version = "0.2.0"
description = "CSV parsing"
entry = "main.fav"
"#,
        );
        let meta = read_rune_meta(&path).unwrap();
        assert_eq!(meta.name, "csv");
        assert_eq!(meta.version, "0.2.0");
        assert_eq!(meta.description, "CSV parsing");
    }

    #[test]
    fn test_read_rune_meta_missing_returns_none() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("rune.toml");
        assert!(read_rune_meta(&path).is_none());
    }

    #[test]
    fn test_read_deps() {
        let dir = TempDir::new().unwrap();
        let path = make_rune_toml(
            &dir,
            r#"
[dependencies]
csv = "0.2.0"
parquet = "0.1.0"
"#,
        );
        let deps = read_deps(&path);
        assert_eq!(deps.get("csv").map(|s| s.as_str()), Some("0.2.0"));
        assert_eq!(deps.get("parquet").map(|s| s.as_str()), Some("0.1.0"));
    }

    #[test]
    fn test_add_remove_dep_roundtrip() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("rune.toml");

        // Add to non-existent file
        add_dep(&path, "csv", "0.2.0");
        let deps = read_deps(&path);
        assert_eq!(deps.get("csv").map(|s| s.as_str()), Some("0.2.0"));

        // Add second dep
        add_dep(&path, "parquet", "0.1.0");
        let deps = read_deps(&path);
        assert_eq!(deps.len(), 2);

        // Remove one
        remove_dep(&path, "csv");
        let deps = read_deps(&path);
        assert_eq!(deps.len(), 1);
        assert!(deps.get("csv").is_none());
        assert_eq!(deps.get("parquet").map(|s| s.as_str()), Some("0.1.0"));
    }

    #[test]
    fn test_write_deps_preserves_rune_section() {
        let dir = TempDir::new().unwrap();
        let path = make_rune_toml(
            &dir,
            r#"[rune]
name = "myapp"
version = "1.0.0"

[dependencies]
csv = "0.1.0"
"#,
        );
        add_dep(&path, "parquet", "0.1.0");
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("[rune]"));
        assert!(content.contains("name = \"myapp\""));
        assert!(content.contains("csv = \"0.1.0\""));
        assert!(content.contains("parquet = \"0.1.0\""));
    }

    #[test]
    fn test_zip_roundtrip() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("main.fav"), b"fn main() {}").unwrap();
        std::fs::write(dir.path().join("lib.fav"), b"fn helper() {}").unwrap();
        std::fs::write(dir.path().join("readme.txt"), b"not a fav file").unwrap();

        let bytes = zip_fav_files(dir.path()).unwrap();
        assert!(!bytes.is_empty());

        let dest = TempDir::new().unwrap();
        unzip_to_dir(&bytes, dest.path()).unwrap();

        assert!(dest.path().join("main.fav").exists());
        assert!(dest.path().join("lib.fav").exists());
        // Non-.fav files should NOT be included
        assert!(!dest.path().join("readme.txt").exists());
    }
}
