# v74.6.0 実装計画 — `fav audit` 拡張（依存関係セキュリティ機能追加）

Date: 2026-08-14

---

## 実装ステップ

### Step 1: 構造体 + 関数を `driver.rs` に追加

```rust
// --- v74.6.0: fav audit 拡張（依存関係セキュリティ機能追加） ---

#[derive(Debug, Clone, PartialEq)]
pub struct DepVulnerability {
    pub name: String,
    pub version: String,
    pub cve: String,
    /// 有効値: "CRITICAL" | "HIGH" | "MEDIUM" | "LOW"
    pub severity: String,
    pub fix_version: String,
}

/// 脆弱性一覧をテキスト形式でフォーマットする
/// 空スライスは "OK  0 vulnerabilities found" を返す
pub fn format_audit_deps_report(vulns: &[DepVulnerability]) -> String {
    if vulns.is_empty() {
        return "OK  0 vulnerabilities found".to_string();
    }
    vulns
        .iter()
        .map(|v| format!("{}  {} {}  {}  Update to {}", v.severity, v.name, v.version, v.cve, v.fix_version))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Cargo.toml 文字列中の `name = "old_version"` を `name = "fix_version"` に置換する
/// マッチしない場合は元の文字列をそのまま返す
pub fn apply_audit_fix(cargo_toml: &str, name: &str, fix_version: &str) -> String {
    // "name = \"x.y.z\"" パターンを探して fix_version に置換
    let pattern = format!("{} = \"", name);
    if let Some(start) = cargo_toml.find(&pattern) {
        let after_quote = start + pattern.len();
        if let Some(end_quote) = cargo_toml[after_quote..].find('"') {
            let old_ver_end = after_quote + end_quote;
            let mut result = cargo_toml.to_string();
            result.replace_range(after_quote..old_ver_end, fix_version);
            return result;
        }
    }
    cargo_toml.to_string()
}
```

### Step 2: `v746000_tests` モジュールを追加

`v745000_tests` の直後に追加する。

```rust
#[cfg(test)]
mod v746000_tests {
    use super::{DepVulnerability, format_audit_deps_report, apply_audit_fix};

    #[test]
    fn audit_detects_vulnerable_dep() {
        let vuln = DepVulnerability {
            name: "tokio".to_string(),
            version: "1.38.0".to_string(),
            cve: "CVE-2026-1234".to_string(),
            severity: "HIGH".to_string(),
            fix_version: "1.38.1".to_string(),
        };
        assert_eq!(vuln.name, "tokio");
        assert_eq!(vuln.cve, "CVE-2026-1234");
        assert_eq!(vuln.severity, "HIGH");

        // レポートフォーマット
        let report = format_audit_deps_report(&[vuln]);
        assert!(report.contains("HIGH"), "severity missing");
        assert!(report.contains("tokio"), "crate name missing");
        assert!(report.contains("CVE-2026-1234"), "CVE missing");
        assert!(report.contains("1.38.1"), "fix version missing");

        // 空スライス → "OK" を含む
        let empty = format_audit_deps_report(&[]);
        assert!(empty.contains("OK"), "empty should return OK");
    }

    #[test]
    fn audit_fix_updates_cargo_toml() {
        let cargo_toml = r#"[dependencies]
tokio = "1.38.0"
serde = "1.0.210"
"#;
        // tokio のバージョンが置換される
        let fixed = apply_audit_fix(cargo_toml, "tokio", "1.38.1");
        assert!(fixed.contains("tokio = \"1.38.1\""), "tokio version not updated");
        assert!(fixed.contains("serde = \"1.0.210\""), "serde should be unchanged");

        // 存在しないクレート → 元の文字列が返る
        let unchanged = apply_audit_fix(cargo_toml, "nonexistent", "9.9.9");
        assert_eq!(unchanged, cargo_toml, "non-existent crate should not change toml");
    }
}
```

### Step 3: バージョン更新

- `fav/Cargo.toml`: `version = "74.5.0"` → `version = "74.6.0"`
- `driver.rs` 内の `version = \"74.5.0\"` を `version = \"74.6.0\"` に replace_all
- `version should be 74.5.0` を `version should be 74.6.0` に replace_all
- `cargo build` で `Cargo.lock` が自動更新される

### Step 4: テスト確認

- `cargo test v746000` で 2 件 pass を確認
- `cargo test` 全体で 3682 tests pass を確認

### Step 5: `CHANGELOG.md` 更新

v74.6.0 エントリを先頭に追加。

### Step 6: `versions/current.md` 更新

- 最終更新: `2026-08-14 (v74.6.0)`
- 進行中: `v74.6.0`
- 次: `v74.7.0`
