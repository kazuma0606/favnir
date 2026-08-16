# v74.1.0 実装計画 — Rune マーケットプレイス（バージョン管理・依存解決）

Date: 2026-08-13

---

## 実装ステップ

### Step 1: `RunePackage` 構造体 + 関数を `driver.rs` に追加

```rust
// --- v74.1.0: Rune マーケットプレイス（バージョン管理・依存解決） ---

#[derive(Debug, Clone)]
pub struct RunePackage {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
}

pub fn format_rune_publish_manifest(pkg: &RunePackage) -> String {
    format!(
        r#"{{"name":{},"version":{},"description":{},"author":{}}}"#,
        json_escape(&pkg.name),
        json_escape(&pkg.version),
        json_escape(&pkg.description),
        json_escape(&pkg.author),
    )
}

/// "name@version" 形式を (name, version) にパースする
/// 例: "mycompany/crm@^1.0" → Ok(("mycompany/crm", "^1.0"))
pub fn parse_rune_dep_entry(entry: &str) -> Result<(String, String), String> {
    match entry.rfind('@') {
        Some(idx) if idx > 0 && idx + 1 < entry.len() => {
            Ok((entry[..idx].to_string(), entry[idx + 1..].to_string()))
        }
        _ => Err(format!("invalid rune dep entry: '{}' (expected 'name@version')", entry)),
    }
}
```

### Step 2: `v741000_tests` モジュールを追加

```rust
#[cfg(test)]
mod v741000_tests {
    use super::{RunePackage, format_rune_publish_manifest, parse_rune_dep_entry};

    #[test]
    fn rune_marketplace_publish_format() {
        let pkg = RunePackage {
            name: "mycompany/crm".to_string(),
            version: "1.0.0".to_string(),
            description: "CRM integration rune".to_string(),
            author: "mycompany".to_string(),
        };
        let manifest = format_rune_publish_manifest(&pkg);
        assert!(manifest.contains("mycompany/crm"), "name missing");
        assert!(manifest.contains("1.0.0"), "version missing");
        assert!(manifest.contains("mycompany"), "author missing");
        assert!(manifest.starts_with('{') && manifest.ends_with('}'), "not a JSON object");
    }

    #[test]
    fn rune_marketplace_add_updates_toml() {
        // 正常ケース
        let (name, ver) = parse_rune_dep_entry("mycompany/crm@^1.0")
            .expect("valid entry should parse");
        assert_eq!(name, "mycompany/crm");
        assert_eq!(ver, "^1.0");

        // バージョン指定なし → Err
        assert!(parse_rune_dep_entry("mycompany/crm").is_err());

        // @ のみ → Err
        assert!(parse_rune_dep_entry("@^1.0").is_err());

        // バージョン部が空 → Err
        assert!(parse_rune_dep_entry("mycompany/crm@").is_err());
    }
}
```

### Step 3: バージョン更新

- `fav/Cargo.toml`: `version = "74.0.0"` → `version = "74.1.0"`
- `driver.rs` 内の `version = "74.0.0"` 参照を `version = "74.1.0"` に replace_all

### Step 4: テスト確認

- `cargo test v741000` で 2 件 pass を確認（Step 3 後に実施）
- `cargo test` 全体で 3671 tests pass を確認

### Step 5: `CHANGELOG.md` 更新

v74.1.0 エントリを先頭に追加。

### Step 6: `versions/current.md` 更新

- 最終更新: `2026-08-13 (v74.1.0)`
- 進行中: `v74.1.0`
- 次: `v74.2.0`
