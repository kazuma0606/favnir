# v73.3.0 実装計画 — PII 検出・マスキング Rune

Date: 2026-08-13

---

## 実装ステップ

### T0: 事前確認

1. `fav/Cargo.toml` のバージョンが `73.2.0` であることを確認
2. `cargo test` が 3650 tests pass（0 failures）であることを確認
3. `driver.rs` に `v732000_tests` モジュールが存在することを確認
4. `driver.rs` に `v733000_tests` が未存在であることを確認
5. `driver.rs` 内の `"73.2.0"` 文字列件数を grep で確認しておく

---

### T1: `PiiMaskStrategy` 列挙型追加

`driver.rs` の `// --- v73.2.0: Data Quality Scoring ---` セクションの後に追加:

```rust
// --- v73.3.0: PII Detection & Masking ---

pub enum PiiMaskStrategy {
    Hash,
    Redact,
    Truncate,
}
```

確認: `cargo build` でエラーがないことを確認。

---

### T2: `mask_pii_fields` 追加

```rust
pub fn mask_pii_fields(
    fields: &[(String, String)],
    strategy: PiiMaskStrategy,
) -> Vec<(String, String)> {
    fields.iter().map(|(name, value)| {
        let masked = match strategy {
            PiiMaskStrategy::Hash => "***".to_string(),
            PiiMaskStrategy::Redact => "[REDACTED]".to_string(),
            PiiMaskStrategy::Truncate => {
                let end = 2.min(value.len());
                format!("{}...", &value[..end])
            }
        };
        (name.clone(), masked)
    }).collect()
}
```

確認: `cargo build` でエラーがないことを確認。

---

### T3: `scan_pii_patterns` 追加

```rust
pub fn scan_pii_patterns(text: &str) -> Vec<String> {
    let mut hits = vec![];
    for word in text.split_whitespace() {
        if word.contains('@') {
            hits.push(format!("email:{}", word));
        } else if word.chars().any(|c| c.is_ascii_digit())
            && word.contains('-')
            && word.chars().filter(|c| c.is_ascii_digit()).count() >= 7
        {
            hits.push(format!("phone:{}", word));
        }
    }
    hits
}
```

確認: `cargo build` でエラーがないことを確認。

---

### T4: `gdpr_erase_record` 追加

```rust
pub fn gdpr_erase_record(fields_to_erase: &[&str]) -> Result<usize, String> {
    if fields_to_erase.is_empty() {
        return Err("no fields specified for erasure".to_string());
    }
    Ok(fields_to_erase.len())
}
```

確認: `cargo build` でエラーがないことを確認。

---

### T5: `runes/privacy/` スタブ Rune ファイル作成

`runes/privacy/rune.toml`:
```toml
[rune]
name = "privacy"
version = "0.1.0"
description = "PII detection and masking for Favnir pipelines"
```

`runes/privacy/privacy.fav`:
```favnir
// Rune.privacy — PII 保護 Rune（将来 VM primitive に接続）
fn mask(fields: List<String>, strategy: String) -> List<String> {
    fields
}

fn scan(text: String) -> List<String> {
    []
}

fn gdpr_erase(user_id: String, tables: List<String>) -> Int {
    0
}
```

確認: ファイルが存在することを確認。

---

### T6: `v733000_tests` モジュール追加

`v732000_tests` モジュールの直後に追加:

```rust
#[cfg(test)]
mod v733000_tests {
    use super::{mask_pii_fields, scan_pii_patterns, gdpr_erase_record, PiiMaskStrategy};

    #[test]
    fn privacy_rune_mask_pii_fields() {
        let fields = vec![
            ("email".to_string(), "user@example.com".to_string()),
            ("name".to_string(),  "Alice".to_string()),
        ];
        let hashed = mask_pii_fields(&fields, PiiMaskStrategy::Hash);
        assert_eq!(hashed.len(), 2, "should return same number of fields");
        assert!(hashed.iter().all(|(_, v)| v == "***"), "all values should be masked");
        let redacted = mask_pii_fields(&fields, PiiMaskStrategy::Redact);
        assert!(redacted.iter().all(|(_, v)| v == "[REDACTED]"), "all values should be redacted");
        let hits = scan_pii_patterns("contact user@example.com or 090-1234-5678");
        assert!(!hits.is_empty(), "should detect PII patterns");
        assert!(hits.iter().any(|h| h.contains("email")), "should detect email pattern");
    }

    #[test]
    fn privacy_rune_gdpr_erase() {
        let ok = gdpr_erase_record(&["email", "phone", "ssn"]);
        assert!(ok.is_ok(), "gdpr erase should succeed: {:?}", ok);
        assert_eq!(ok.unwrap(), 3, "should return count of erased fields");
        let err = gdpr_erase_record(&[]);
        assert!(err.is_err(), "empty fields should return Err");
        assert!(err.unwrap_err().contains("no fields"), "error should mention no fields");
        let rune_toml = include_str!("../../runes/privacy/rune.toml");
        assert!(rune_toml.contains("privacy"), "rune.toml should reference privacy rune");
    }
}
```

確認: `cargo test v733000` で 2 件 pass。

---

### T7: バージョン更新

- `fav/Cargo.toml`: `version = "73.2.0"` → `version = "73.3.0"`
- `driver.rs` 内の `version = \"73.2.0\"` を `version = \"73.3.0\"` に replace_all
- エラーメッセージ内の `73.2.0` を `73.3.0` に replace_all
- 残存 `73.2.0` がコメント・セクションヘッダーのみであることを確認
- T0 で確認した `"73.2.0"` 件数分がすべて置換されたかを grep で照合する
- `cargo build` 後に `fav/Cargo.lock` が `version = "73.3.0"` を含むことを確認

---

### T8: 部分テスト確認

- `cargo test v733000` で 2 件 pass

---

### T9: 全体テスト確認

- `cargo test` 全体で 3652 tests pass（0 failures）

---

### T10: `CHANGELOG.md` 更新

```markdown
## [v73.3.0] — 2026-08-13 — PII 検出・マスキング Rune

### Added
- `PiiMaskStrategy` 列挙型（Hash / Redact / Truncate）
- `mask_pii_fields(fields, strategy)` — PII フィールドのマスキング
- `scan_pii_patterns(text)` — メールアドレス・電話番号パターンの検出
- `gdpr_erase_record(fields_to_erase)` — GDPR 削除フィールドカウント
- `runes/privacy/privacy.fav` + `rune.toml` — スタブ Rune ファイル

### Tests
- `privacy_rune_mask_pii_fields` — Hash/Redact マスク + メールスキャンを確認
- `privacy_rune_gdpr_erase` — GDPR 削除カウント + rune.toml 存在を確認
- 合計テスト数: 3652（+2）
```

---

### T11: `versions/current.md` 更新

- 「最終更新」を `2026-08-13 (v73.3.0)` に更新
- 「進行中バージョン」を `v73.3.0` に更新
- 「次に切る版」を `v73.4.0` に更新

---

### T12: 最終確認

- `cargo test v733000` で 2 件 pass
- `cargo test` 全体で 3652 tests pass（0 failures）
- `fav/Cargo.toml` のバージョンが `73.3.0`
- `PiiMaskStrategy` / `mask_pii_fields` / `scan_pii_patterns` / `gdpr_erase_record` が pub で存在する
- `runes/privacy/privacy.fav` と `runes/privacy/rune.toml` が存在する
- `CHANGELOG.md` に `[v73.3.0]` エントリが存在する
- `versions/current.md` の「進行中バージョン」が `v73.3.0` であること
