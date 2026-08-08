# Plan — v57.6.0 — コンプライアンスレポート（GDPR / SOC2 対応）

## 実装順序

```
Cargo.toml → driver.rs（v57600_tests 追加 + バージョンチェック更新）
→ cargo test 全通過確認 → cargo clippy クリーン確認
→ ポスト処理（CHANGELOG + current.md + roadmap 更新）
→ tasks.md COMPLETE 更新
```

依存関係:
- `ComplianceFramework` / `ComplianceReport` / `generate_report` はすべて `v57600_tests` 内に完結
- `toml.rs` への変更は不要（driver.rs のみ）

---

## Step 1: `fav/Cargo.toml` — バージョン更新

```toml
version = "57.5.0"  →  version = "57.6.0"
```

---

## Step 2: `fav/src/driver.rs` — `v57600_tests` 追加

`v57500_tests` の直前（`// -- v57500_tests` コメント行の直前）に挿入:

```rust
// -- v57600_tests (v57.6.0) -- コンプライアンスレポート --
#[cfg(test)]
mod v57600_tests {
    #[derive(Debug, PartialEq)]
    enum ComplianceFramework {
        Gdpr,
        Soc2,
    }

    #[derive(Debug)]
    struct ComplianceReport {
        framework: ComplianceFramework,
        entry_count: usize,
        sections: Vec<String>,
    }

    fn generate_report(framework: ComplianceFramework, entries: &[&str]) -> ComplianceReport {
        let sections = match framework {
            ComplianceFramework::Gdpr => vec![
                "## Data Access Log".to_string(),
                "## Deletion Records".to_string(),
            ],
            ComplianceFramework::Soc2 => vec![
                "## Access Control".to_string(),
                "## Audit Trail".to_string(),
            ],
        };
        ComplianceReport {
            framework,
            entry_count: entries.len(),
            sections,
        }
    }

    #[test]
    fn compliance_report_gdpr_generates() {
        let entries = vec![
            r#"{"event":"data.access","user":"alice","resource":"user_profile"}"#,
            r#"{"event":"data.delete","user":"bob","resource":"order_history"}"#,
        ];
        let report = generate_report(ComplianceFramework::Gdpr, &entries);

        assert_eq!(report.framework, ComplianceFramework::Gdpr);
        assert_eq!(report.entry_count, 2, "should count all audit entries");
        assert_eq!(report.sections.len(), 2, "GDPR report should have 2 sections");
        assert_eq!(report.sections[0], "## Data Access Log");
        assert_eq!(report.sections[1], "## Deletion Records");
        // Confirm SOC2 sections are not present (both SOC2 sections)
        assert!(!report.sections.iter().any(|s| s.contains("Access Control")));
        assert!(!report.sections.iter().any(|s| s.contains("Audit Trail")));
    }

    #[test]
    fn compliance_report_soc2_generates() {
        let entries = vec![
            r#"{"event":"auth.login","user":"carol","role":"admin"}"#,
            r#"{"event":"auth.logout","user":"carol","role":"admin"}"#,
            r#"{"event":"resource.access","user":"dave","resource":"pipeline_config"}"#,
        ];
        let report = generate_report(ComplianceFramework::Soc2, &entries);

        assert_eq!(report.framework, ComplianceFramework::Soc2);
        assert_eq!(report.entry_count, 3, "should count all audit entries");
        assert_eq!(report.sections.len(), 2, "SOC2 report should have 2 sections");
        assert_eq!(report.sections[0], "## Access Control");
        assert_eq!(report.sections[1], "## Audit Trail");
        // Confirm GDPR sections are not present (both GDPR sections)
        assert!(!report.sections.iter().any(|s| s.contains("Data Access Log")));
        assert!(!report.sections.iter().any(|s| s.contains("Deletion Records")));
    }
}
```

---

## Step 3: `fav/src/driver.rs` — バージョンチェックテスト更新

```
v56300_tests::cargo_toml_version_is_56_3_0  : "57.5.0" → "57.6.0"（failure メッセージも更新）
v56900_tests::cargo_toml_version_is_56_9_0  : "57.5.0" → "57.6.0"（rolling）
v57000_tests::cargo_toml_version_is_57_0_0  : "57.5.0" → "57.6.0"（rolling）
```

> `v57100_tests` 〜 `v57500_tests` には `cargo_toml_version_is_*` がないため更新不要。

---

## Step 4: `cargo test` 全通過確認

```bash
cargo test -j 8 -- --test-threads=8
```

3265 tests passed, 0 failed を確認。`v57600_tests` の 2 件が全通過することを確認。

---

## Step 5: `cargo clippy` クリーン確認

```bash
cargo clippy -- -D warnings
```

---

## Step 6: ポスト処理

1. `CHANGELOG.md` に v57.6.0 エントリを追加（先頭）
2. `versions/current.md` を v57.6.0 / 3265 tests に更新
3. `versions/roadmap/roadmap-v57.1-v58.0.md` の v57.6.0 実績を COMPLETE に更新
4. `versions/roadmap/roadmap-v55.1-v60.0.md` の v57.6.0 実績欄を COMPLETE に更新し、テスト数推移テーブルに v57.6.0 行（3265）を追加

---

## Step 7: `versions/v55-v60/v57.6.0/tasks.md` を COMPLETE に更新

全チェックボックス（T0 含む）を `[x]` にする。

---

## リスク・注意点

| リスク | 対策 |
|---|---|
| `ComplianceFramework` の `PartialEq` 忘れで `assert_eq!` がコンパイルエラー | `#[derive(Debug, PartialEq)]` を必ず付ける |
| `v57500_tests` コメント行の直前への挿入位置ミス | Python `str.replace()` を使う（awk 多行挿入は過去に失敗実績あり） |
| sections の交差汚染（GDPR テストに SOC2 セクションが混入） | 両テストで「相手方のセクションが含まれないこと」を `assert!(!...)` で検証する |
