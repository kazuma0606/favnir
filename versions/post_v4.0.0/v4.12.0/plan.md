# Favnir v4.12.0 実装計画 — Rune Registry

作成日: 2026-05-17

---

## Phase 0: バージョン更新

- `fav/Cargo.toml` の version を `"4.12.0"` に変更
- `fav/src/main.rs` のヘルプ文字列・バージョン表示を `4.12.0` に更新

---

## Phase 1: `fav.toml` — `[dependencies]` セクション

### `RuneDepSpec` 構造体

```rust
// fav/src/toml.rs
#[derive(Debug, Clone)]
pub struct RuneDepSpec {
    pub version:  String,         // "1.0.0" / "^1.0.0" / "*"
    pub registry: Option<String>, // None = "local"
}
```

### パース

`fav.toml` の `[dependencies]` は 2 形式を受け付ける：

```toml
[dependencies]
csv   = "1.0.0"                              # 文字列形式
email = { version = "^0.3.0" }              # テーブル形式
http  = { version = "2.1.0", registry = "local" }
```

```rust
// FavToml に追加
pub dependencies: HashMap<String, RuneDepSpec>,
```

パース処理:

```rust
fn parse_dependencies(table: &TomlTable) -> HashMap<String, RuneDepSpec> {
    let mut deps = HashMap::new();
    for (name, val) in table {
        let spec = match val {
            TomlValue::Str(v) => RuneDepSpec { version: v.clone(), registry: None },
            TomlValue::Table(t) => RuneDepSpec {
                version:  t.get("version").and_then(|v| v.as_str()).unwrap_or("*").to_string(),
                registry: t.get("registry").and_then(|v| v.as_str()).map(|s| s.to_string()),
            },
            _ => continue,
        };
        deps.insert(name.clone(), spec);
    }
    deps
}
```

### `FavToml` リテラルへの追加

`checker.rs` × 2、`resolver.rs` × 2、`driver.rs` × 1 の `FavToml { ... }` リテラルに
`dependencies: HashMap::new()` を追加する。

---

## Phase 2: `fav/src/registry/mod.rs` — Registry コアロジック

新規ファイル。`pub(crate)` で `driver.rs` と `resolver.rs` から使う。

### ホームディレクトリ解決

```rust
fn registry_root() -> PathBuf {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".fav").join("registry")
}
```

### `Registry` 構造体

```rust
pub struct Registry {
    pub root: PathBuf,
}

impl Registry {
    pub fn new() -> Self {
        Self { root: registry_root() }
    }

    pub fn with_root(root: PathBuf) -> Self {
        Self { root }
    }
}
```

### `installed_versions(name) -> Vec<String>`

```rust
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
    // semver ソート（降順）
    versions.sort_by(|a, b| semver_cmp(b, a));
    versions
}
```

### `resolve_version(name, constraint) -> Option<String>`

```rust
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
```

### `semver_compatible(version, base) -> bool`

```rust
fn semver_compatible(version: &str, base: &str) -> bool {
    let (vma, vmi, _vpa) = parse_semver(version);
    let (bma, bmi, bpa) = parse_semver(base);
    if bma == 0 {
        // 0.x.y: minor も一致が必要
        vma == 0 && vmi == bmi && version_ge(version, base)
    } else {
        // 1.x.y+: major が一致すれば OK
        vma == bma && version_ge(version, base)
    }
}

fn parse_semver(v: &str) -> (u32, u32, u32) {
    let parts: Vec<u32> = v.split('.').filter_map(|p| p.parse().ok()).collect();
    (parts.get(0).copied().unwrap_or(0),
     parts.get(1).copied().unwrap_or(0),
     parts.get(2).copied().unwrap_or(0))
}

fn version_ge(a: &str, b: &str) -> bool {
    let (ama, ami, apa) = parse_semver(a);
    let (bma, bmi, bpa) = parse_semver(b);
    (ama, ami, apa) >= (bma, bmi, bpa)
}

fn semver_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    let (ama, ami, apa) = parse_semver(a);
    let (bma, bmi, bpa) = parse_semver(b);
    (ama, ami, apa).cmp(&(bma, bmi, bpa))
}
```

### `list() -> Vec<PackageEntry>`

```rust
pub struct PackageEntry {
    pub name:     String,
    pub versions: Vec<String>,  // 降順ソート済み
}

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
```

### `search(query) -> Vec<PackageEntry>`

```rust
pub fn search(&self, query: &str) -> Vec<PackageEntry> {
    self.list()
        .into_iter()
        .filter(|e| e.name.contains(query))
        .collect()
}
```

### `info(name) -> Option<PackageMeta>`

```rust
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

pub fn info(&self, name: &str) -> Option<PackageMeta> {
    let versions = self.installed_versions(name);
    let latest = versions.first()?;
    let pkg_dir = self.root.join(name).join(latest);
    let meta_path = pkg_dir.join("fav.pkg.toml");
    let meta_str = std::fs::read_to_string(&meta_path).ok()?;
    let mut meta = parse_pkg_toml(&meta_str);
    meta.files = collect_rune_files(&pkg_dir.join("runes"));
    Some(meta)
}
```

### `publish(meta, rune_files) -> Result<(), String>`

```rust
pub fn publish(
    &self,
    meta: &PackageMeta,
    rune_files: &[(String, Vec<u8>)],  // (相対パス, 内容)
) -> Result<(), String> {
    let dest = self.root.join(&meta.name).join(&meta.version);
    std::fs::create_dir_all(&dest).map_err(|e| e.to_string())?;

    // fav.pkg.toml 書き出し
    let toml_content = format_pkg_toml(meta);
    std::fs::write(dest.join("fav.pkg.toml"), toml_content)
        .map_err(|e| e.to_string())?;

    // rune ファイル書き出し
    for (rel_path, content) in rune_files {
        let file_dest = dest.join(rel_path);
        if let Some(parent) = file_dest.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        std::fs::write(&file_dest, content).map_err(|e| e.to_string())?;
    }
    Ok(())
}
```

### `install(name, version, dest_runes_dir) -> Result<(), String>`

```rust
pub fn install(&self, name: &str, version: &str, dest: &Path) -> Result<(), String> {
    let src = self.root.join(name).join(version).join("runes").join(name);
    if !src.exists() {
        return Err(format!("{}@{} not found in registry", name, version));
    }
    let dest_pkg = dest.join(name);
    copy_dir_all(&src, &dest_pkg).map_err(|e| e.to_string())?;
    Ok(())
}
```

### `rune_path(name) -> Option<PathBuf>`

resolver.rs から呼ぶ：最新バージョンの rune ディレクトリを返す。

```rust
pub fn rune_path(&self, name: &str) -> Option<PathBuf> {
    let versions = self.installed_versions(name);
    let latest = versions.first()?;
    let path = self.root.join(name).join(latest).join("runes").join(name);
    if path.exists() { Some(path) } else { None }
}
```

---

## Phase 3: `fav/src/middle/resolver.rs` — レジストリフォールバック

`resolve_rune_path`（またはそれに相当する rune 探索箇所）に第 2 段フォールバックを追加：

```rust
// 既存: ローカル runes/ ディレクトリを探索
let local = project_root.join("runes").join(name);
if local.exists() {
    return Some(local);
}

// 新規: ローカルレジストリにフォールバック
let reg = crate::registry::Registry::new();
if let Some(path) = reg.rune_path(name) {
    return Some(path);
}
```

既存テストが壊れないよう、`project_root` が指定されていない場合は
`std::env::current_dir()` を使うことで後方互換を維持する。

---

## Phase 4: `fav/src/driver.rs` — `cmd_publish` / `cmd_install` / `cmd_registry`

### `cmd_publish(name_override, version_override, dry_run)`

```rust
pub fn cmd_publish(
    name_override:    Option<&str>,
    version_override: Option<&str>,
    dry_run:          bool,
    force:            bool,
) {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let root = FavToml::find_root(&cwd).unwrap_or_else(|| {
        eprintln!("error: no fav.toml found");
        process::exit(1);
    });
    let toml = FavToml::load(&root).unwrap_or_else(|| {
        eprintln!("error: could not read fav.toml");
        process::exit(1);
    });

    let pkg_name    = name_override.unwrap_or(&toml.name).to_string();
    let pkg_version = version_override.unwrap_or(&toml.version).to_string();

    // runes/ 以下の .fav ファイルを収集
    let rune_files = collect_fav_files(&root.join("runes"));

    println!("[publish] Package: {} v{}", pkg_name, pkg_version);
    println!("[publish] Files:");
    for (path, _) in &rune_files {
        println!("  {}", path);
    }

    let archive_path = format!("/tmp/{}-{}.fav.pkg", pkg_name, pkg_version);
    let reg_path = format!("~/.fav/registry/{}/{}/", pkg_name, pkg_version);

    println!("[publish] Archive: {}{}", archive_path, if dry_run { " (DRY RUN)" } else { "" });
    println!("[publish] Registry: {}{}", reg_path, if dry_run { " (DRY RUN)" } else { "" });

    if !dry_run {
        let meta = PackageMeta {
            name:        pkg_name.clone(),
            version:     pkg_version.clone(),
            description: toml.description.clone().unwrap_or_default(),
            author:      toml.authors.first().cloned().unwrap_or_default(),
            license:     toml.license.clone().unwrap_or_else(|| "MIT".to_string()),
            published:   chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            files:       vec![],
        };

        // レジストリに既存バージョンがある場合は確認（force なら上書き）
        let reg = Registry::new();
        if reg.installed_versions(&pkg_name).contains(&pkg_version) && !force {
            eprintln!("error: {}@{} already published. Use --force to overwrite.", pkg_name, pkg_version);
            process::exit(1);
        }

        reg.publish(&meta, &rune_files).unwrap_or_else(|e| {
            eprintln!("error: {}", e);
            process::exit(1);
        });
        println!("[publish] Done — {}@{} published to local registry", pkg_name, pkg_version);
    } else {
        println!("[publish] Done (dry run — no changes made)");
    }
}
```

### `cmd_install(pkg_name_arg, force)`

```rust
pub fn cmd_install(pkg_name_arg: Option<&str>, force: bool) {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let root = FavToml::find_root(&cwd).unwrap_or_else(|| {
        eprintln!("error: no fav.toml found");
        process::exit(1);
    });
    let toml = FavToml::load(&root).unwrap_or_else(|| {
        eprintln!("error: could not read fav.toml");
        process::exit(1);
    });

    let reg = Registry::new();
    let runes_dir = root.join("runes");

    let deps_to_install: Vec<(String, RuneDepSpec)> = if let Some(name) = pkg_name_arg {
        // 引数で指定された 1 件のみ
        let spec = toml.dependencies.get(name).cloned().unwrap_or(RuneDepSpec {
            version: "*".to_string(), registry: None,
        });
        vec![(name.to_string(), spec)]
    } else {
        toml.dependencies.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
    };

    if deps_to_install.is_empty() {
        println!("[install] No dependencies declared in fav.toml");
        return;
    }

    println!("[install] Reading fav.toml...");
    let mut installed = 0usize;

    for (name, spec) in deps_to_install {
        let resolved = reg.resolve_version(&name, &spec.version);
        match resolved {
            None => {
                eprintln!("[install] error: {}@{} not found in registry", name, spec.version);
                process::exit(1);
            }
            Some(ver) => {
                println!("[install] Resolving {}@{} → {}", name, spec.version, ver);
                let dest = runes_dir.clone();
                if dest.join(&name).exists() && !force {
                    println!("[install] {} already in ./runes/ (skipped, use --force to overwrite)", name);
                    continue;
                }
                reg.install(&name, &ver, &dest).unwrap_or_else(|e| {
                    eprintln!("[install] error: {}", e);
                    process::exit(1);
                });
                println!("[install] Copying {}@{} → ./runes/{}/", name, ver, name);
                installed += 1;
            }
        }
    }

    println!("[install] Done — {} package(s) installed", installed);
}
```

### `cmd_registry(subcommand, args)`

```rust
pub fn cmd_registry(subcommand: Option<&str>, args: &[String]) {
    let reg = Registry::new();
    match subcommand {
        Some("list") | None => {
            let entries = reg.list();
            if entries.is_empty() {
                println!("(no packages in local registry)");
            } else {
                for e in entries {
                    println!("{:<16} {}", e.name, e.versions.join(", "));
                }
            }
        }
        Some("search") => {
            let query = args.first().map(|s| s.as_str()).unwrap_or("");
            let results = reg.search(query);
            if results.is_empty() {
                println!("(no packages matching \"{}\")", query);
            } else {
                for e in results {
                    let desc = reg.info(&e.name)
                        .map(|m| m.description)
                        .unwrap_or_default();
                    let latest = e.versions.first().cloned().unwrap_or_default();
                    println!("{:<16} {:<8} {}", e.name, latest, desc);
                }
            }
        }
        Some("info") => {
            let name = args.first().map(|s| s.as_str()).unwrap_or_else(|| {
                eprintln!("error: usage: fav registry info <name>");
                process::exit(1);
            });
            match reg.info(name) {
                None => {
                    eprintln!("error: package '{}' not found in local registry", name);
                    process::exit(1);
                }
                Some(m) => {
                    println!("Name:        {}", m.name);
                    let versions = reg.installed_versions(&m.name);
                    println!("Versions:    {}", versions.join(", "));
                    println!("Description: {}", m.description);
                    println!("Author:      {}", m.author);
                    println!("License:     {}", m.license);
                    println!("Published:   {}", &m.published[..10]);
                    println!("Files:");
                    for f in &m.files {
                        println!("  {}", f);
                    }
                }
            }
        }
        Some(unknown) => {
            eprintln!("error: unknown registry subcommand '{}'", unknown);
            eprintln!("usage: fav registry [list|search <q>|info <name>]");
            process::exit(1);
        }
    }
}
```

---

## Phase 5: CLI 配線 (main.rs)

```rust
Some("publish") => {
    let mut name_override:    Option<String> = None;
    let mut version_override: Option<String> = None;
    let mut dry_run = false;
    let mut force   = false;
    let mut i = 2usize;
    while i < args.len() {
        match args[i].as_str() {
            "--name"    => { name_override    = args.get(i+1).cloned(); i += 2; }
            "--version" => { version_override = args.get(i+1).cloned(); i += 2; }
            "--dry-run" => { dry_run = true;  i += 1; }
            "--force"   => { force   = true;  i += 1; }
            other => { eprintln!("error: unexpected publish argument `{}`", other); process::exit(1); }
        }
    }
    cmd_publish(name_override.as_deref(), version_override.as_deref(), dry_run, force);
}

Some("install") => {
    let pkg_name = args.get(2).map(|s| s.as_str());
    let force    = args.iter().any(|a| a == "--force");
    cmd_install(pkg_name, force);
}

Some("registry") => {
    let subcommand = args.get(2).map(|s| s.as_str());
    let sub_args: Vec<String> = args.iter().skip(3).cloned().collect();
    cmd_registry(subcommand, &sub_args);
}
```

HELP テキスト追記：

```
    publish [--name <n>] [--version <v>] [--dry-run] [--force]
                  Publish runes to local registry (~/.fav/registry/).
    install [<name>] [--force]
                  Install dependencies from fav.toml (or a specific rune by name).
    registry [list|search <q>|info <name>]
                  Manage the local Rune registry.
```

---

## Phase 6: テスト

### `fav/src/registry/mod.rs` 内 `#[cfg(test)]`

tempdir ベースのユニットテスト（8 件）。`Registry::with_root(tempdir)` を使う。

```rust
fn make_dummy_pkg(reg: &Registry, name: &str, version: &str, files: &[(&str, &str)]) {
    let meta = PackageMeta {
        name: name.to_string(), version: version.to_string(),
        description: format!("{} rune", name),
        author: "test".to_string(), license: "MIT".to_string(),
        published: "2026-05-17T00:00:00Z".to_string(), files: vec![],
    };
    let rune_files: Vec<(String, Vec<u8>)> = files.iter()
        .map(|(p, c)| (p.to_string(), c.as_bytes().to_vec()))
        .collect();
    reg.publish(&meta, &rune_files).unwrap();
}
```

### `driver.rs` 統合テスト（7 件）

```rust
#[cfg(test)]
mod registry_tests {
    use super::*;

    #[test]
    fn publish_dry_run_prints_steps() {
        // fav.toml + runes/ を持つ tempdir を作成
        // cmd_publish(..., dry_run=true) の出力に "[publish]" が含まれることを確認
    }

    #[test]
    fn install_from_local_registry() {
        // Registry::with_root(tempdir) に dummy pkg を publish
        // cmd_install で ./runes/<name>/ にファイルが展開されることを確認
    }

    // ... etc.
}
```

---

## Cargo.toml 変更

新規クレートなし（`zip`, `chrono`, `serde_json` は既存）。
ただし `walkdir` が既存依存にない場合は追加が必要。

```toml
# walkdir が未追加なら追加
walkdir = "2"
```

> `zip = "0.6"` と `walkdir = "2"` は v4.11.0 で追加済みのため追加不要。

---

## 実装メモ

- **`FavToml` への `dependencies: HashMap::new()` 追加**: `checker.rs` ×2、`resolver.rs` ×2、`driver.rs` ×1 の FavToml リテラルすべて
- **`FavToml.authors`**: 既存フィールドが `Vec<String>` か確認。なければ `Option<Vec<String>>` として追加
- **`FavToml.description`**: `Option<String>` として追加（未定義プロジェクトでも動くよう）
- **`FavToml.license`**: `Option<String>` として追加
- **ホームディレクトリ**: `HOME`（Unix）/ `USERPROFILE`（Windows）環境変数から取得（`dirs` クレート追加しない）
- **テスト用 Registry**: `Registry::with_root(tempdir)` パターンで本物の `~/.fav/` を汚染しない
- **`copy_dir_all`**: `fn copy_dir_all(src, dst)` — `std::fs::copy` を再帰的に呼ぶヘルパー
- **`collect_fav_files`**: `walkdir::WalkDir` で `.fav` ファイルを収集 → `Vec<(String, Vec<u8>)>`
- **`format_pkg_toml` / `parse_pkg_toml`**: 既存 toml.rs の簡易パーサーを流用
- **resolver.rs への変更**: rune 探索が失敗した場合にのみ Registry を参照する（既存の探索順を変えない）
