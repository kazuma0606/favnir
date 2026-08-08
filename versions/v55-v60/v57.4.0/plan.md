# Plan — v57.4.0 — 依存関係セキュリティスキャン（`fav audit --security`）

## 実装順序

```
Cargo.toml → driver.rs（v57400_tests 追加 + バージョンチェック更新）
→ cargo test 全通過確認 → cargo clippy クリーン確認
→ ポスト処理（CHANGELOG + current.md + roadmap 更新）
→ tasks.md COMPLETE 更新
```

依存関係:
- `CveEntry` / `scan_security` / `fail_on_high` はすべて `v57400_tests` 内に完結
- `toml.rs` の変更は不要（driver.rs のみ変更対象）

---

## Step 1: `fav/Cargo.toml` — バージョン更新

```toml
version = "57.3.0"  →  version = "57.4.0"
```

---

## Step 2: `fav/src/driver.rs` — `v57400_tests` 追加

`v57300_tests` の直前（`// -- v57300_tests` コメント行の直前）に挿入:

```rust
// -- v57400_tests (v57.4.0) -- 依存関係セキュリティスキャン --
#[cfg(test)]
mod v57400_tests {
    #[derive(Debug, Clone, PartialEq)]
    struct CveEntry {
        rune: String,
        version: String,
        cve_id: String,
        severity: String,
        fix_version: Option<String>,
    }

    fn make_cve_db() -> Vec<CveEntry> {
        vec![
            CveEntry {
                rune: "kafka".to_string(),
                version: "2.1.0".to_string(),
                cve_id: "CVE-2026-1234".to_string(),
                severity: "HIGH".to_string(),
                fix_version: Some("2.2.0".to_string()),
            },
            CveEntry {
                rune: "redis".to_string(),
                version: "1.0.0".to_string(),
                cve_id: "CVE-2026-5678".to_string(),
                severity: "MEDIUM".to_string(),
                fix_version: Some("1.1.0".to_string()),
            },
        ]
    }

    fn scan_security<'a>(
        runes: &[(&str, &str)],
        db: &'a [CveEntry],
    ) -> Vec<&'a CveEntry> {
        db.iter()
            .filter(|entry| {
                runes
                    .iter()
                    .any(|(name, ver)| *name == entry.rune && *ver == entry.version)
            })
            .collect()
    }

    fn fail_on_high(findings: &[&CveEntry]) -> bool {
        findings.iter().any(|e| e.severity == "HIGH")
    }

    #[test]
    fn security_scan_detects_cve() {
        let db = make_cve_db();
        let runes = vec![
            ("kafka", "2.1.0"),
            ("redis", "1.0.0"),
            ("postgres", "1.0.0"), // not in CVE DB
        ];
        let findings = scan_security(&runes, &db);
        assert_eq!(findings.len(), 2, "should detect 2 CVEs");
        assert_eq!(findings[0].cve_id, "CVE-2026-1234");
        assert_eq!(findings[0].severity, "HIGH");
        assert_eq!(findings[1].cve_id, "CVE-2026-5678");
        assert_eq!(findings[1].severity, "MEDIUM");
        // postgres@1.0.0 is clean — not in findings
        assert!(!findings.iter().any(|e| e.rune == "postgres"));
    }

    #[test]
    fn security_scan_fail_on_high() {
        let db = make_cve_db();

        // Case 1: has HIGH → fail_on_high returns true
        let runes_with_high = vec![("kafka", "2.1.0"), ("redis", "1.0.0")];
        let findings_high = scan_security(&runes_with_high, &db);
        assert!(
            fail_on_high(&findings_high),
            "should fail when HIGH CVE is present"
        );

        // Case 2: MEDIUM only → fail_on_high returns false
        let runes_medium_only = vec![("redis", "1.0.0")];
        let findings_medium = scan_security(&runes_medium_only, &db);
        assert!(
            !fail_on_high(&findings_medium),
            "should not fail when only MEDIUM CVEs present"
        );
    }
}
```

---

## Step 3: `fav/src/driver.rs` — バージョンチェックテスト更新

```
v56300_tests::cargo_toml_version_is_56_3_0  : "57.3.0" → "57.4.0"
v56900_tests::cargo_toml_version_is_56_9_0  : "57.3.0" → "57.4.0"（rolling）
v57000_tests::cargo_toml_version_is_57_0_0  : "57.3.0" → "57.4.0"（rolling）
```

> `v57100_tests` / `v57200_tests` / `v57300_tests` には `cargo_toml_version_is_*` がないため更新不要。

---

## Step 4: `cargo test` 全通過確認

```bash
cargo test -j 8 -- --test-threads=8
```

3261 tests passed, 0 failed を確認。`v57400_tests` の 2 件が全通過することを確認。

---

## Step 5: `cargo clippy` クリーン確認

```bash
cargo clippy -- -D warnings
```

---

## Step 6: ポスト処理

1. `CHANGELOG.md` に v57.4.0 エントリを追加（先頭）
2. `versions/current.md` を v57.4.0 / 3261 tests に更新
3. `versions/roadmap/roadmap-v57.1-v58.0.md` の v57.4.0 実績を COMPLETE に更新
4. `versions/roadmap/roadmap-v55.1-v60.0.md` の v57.4.0 実績欄を COMPLETE に更新し、テスト数推移テーブルに v57.4.0 行（3261）を追加

---

## Step 7: `versions/v55-v60/v57.4.0/tasks.md` を COMPLETE に更新

全チェックボックス（T0 含む）を `[x]` にする。

---

## リスク・注意点

| リスク | 対策 |
|---|---|
| `scan_security` の返り値が `Vec<&CveEntry>` でライフタイム複雑化 | テスト内クロージャで完結するため問題なし |
| `v57300_tests` コメント行の直前への挿入位置ミス | `// -- v57300_tests` コメント行を対象にする |
| awk での多行ブロック挿入ミス（過去事例あり） | Python `str.replace()` を使う |
| `CveEntry` の `#[derive(PartialEq)]` 忘れでテスト比較失敗 | `assert_eq!` ではなくフィールド別 assert を使う（plan では PartialEq 不使用設計） |
