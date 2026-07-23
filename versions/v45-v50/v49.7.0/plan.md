# Plan: v49.7.0 — セキュリティ審査 2.0

Date: 2026-07-18

---

## 実装方針

### Step 1: `driver.rs` にヘルパー関数追加

`validate_import_path` と `validate_rune_name` を `pub fn` として追加。
既存の `compute_file_fingerprint` / `file_needs_recheck` と同じ領域（ヘルパー関数群）に配置。

#### `validate_import_path`

```rust
pub fn validate_import_path(path: &str) -> Result<(), String> {
    if path.is_empty() {
        return Err("import path must not be empty".to_string());
    }
    if path.contains('\\') {
        return Err("import path must not contain backslashes".to_string());
    }
    let parts: Vec<&str> = path.split('/').collect();
    for part in &parts {
        if *part == ".." {
            return Err(format!("import path traversal rejected: '{}'", path));
        }
    }
    Ok(())
}
```

#### `validate_rune_name`

```rust
pub fn validate_rune_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("rune name must not be empty".to_string());
    }
    if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
        return Err(format!("rune name '{}' contains invalid characters (only a-z, A-Z, 0-9, - allowed)", name));
    }
    if name.starts_with('-') || name.ends_with('-') {
        return Err(format!("rune name '{}' must not start or end with '-'", name));
    }
    if name.contains("--") {
        return Err(format!("rune name '{}' must not contain consecutive hyphens '--'", name));
    }
    Ok(())
}
```

### Step 2: `v497000_tests` モジュール追加

`v496000_tests` の直前に挿入。

### Step 3: バージョン更新・完了

- `Cargo.toml` version → `"49.7.0"`
- `cargo test` 3083 passed 確認
- `cargo clippy` クリーン確認
- `CHANGELOG.md` 更新
- `versions/current.md` 更新
- `roadmap-v49.1-v50.0.md` 実績記入

---

## 注意事項

- `validate_import_path` は `..` コンポーネントを部分一致ではなく **パスコンポーネント単位** で検査
  (`path.contains("..")` ではなく `split('/').any(|p| p == "..")` を使う）
- `validate_rune_name` のスペース検査は `is_ascii_alphanumeric() || c == '-'` の `all` で網羅される
