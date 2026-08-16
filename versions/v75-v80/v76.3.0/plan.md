# v76.3.0 実装計画 — PII 来歴追跡・GDPR 消去計画

Date: 2026-08-15

---

## Step 1: driver.rs — 型・関数追加

`fav/src/driver.rs` の末尾に以下を追加する：

```rust
// --- v76.3.0: PII 来歴追跡・GDPR 消去計画 ---

#[derive(Debug, Clone)]
pub struct PiiProvenanceReport {
    pub fields:     Vec<String>,
    pub source_uri: String,
    pub masked:     bool,
}

pub fn detect_pii_in_tag(tag: &ProvenanceTag) -> Vec<String> {
    if tag.pii {
        vec!["pii_detected".to_string()]
    } else {
        vec![]
    }
}

#[derive(Debug, Clone)]
pub struct ErasurePlan {
    pub target_uri: String,
    pub fields:     Vec<String>,
    pub reason:     String,
}

pub fn generate_erasure_plan(tag: &ProvenanceTag) -> Option<ErasurePlan> {
    if tag.pii {
        Some(ErasurePlan {
            target_uri: tag.source.uri.clone(),
            fields:     detect_pii_in_tag(tag),
            reason:     "GDPR erasure request".to_string(),
        })
    } else {
        None
    }
}
```

---

## Step 2: cargo check

`cargo check` でコンパイルエラーがないことを確認する。

---

## Step 3: CHANGELOG.md 更新（テスト追加より先）

`CHANGELOG.md` の先頭に v76.3.0 エントリを追加する。

---

## Step 4: テストモジュール v763000_tests 追加

```rust
#[cfg(test)]
mod v763000_tests {
    use super::*;

    #[test]
    fn pii_detected_in_provenance() {
        let src = DataSource {
            name:        "crm".to_string(),
            uri:         "snowflake://crm/users".to_string(),
            source_type: DataSourceType::Snowflake,
        };
        let pii_tag   = ProvenanceTag { source: src.clone(), transforms: vec![], pii: true };
        let clean_tag = ProvenanceTag { source: src,         transforms: vec![], pii: false };

        let detected = detect_pii_in_tag(&pii_tag);
        assert!(!detected.is_empty(), "pii=true → non-empty");

        let clean = detect_pii_in_tag(&clean_tag);
        assert!(clean.is_empty(), "pii=false → empty");
    }

    #[test]
    fn gdpr_erasure_plan_generated() {
        let src = DataSource {
            name:        "crm".to_string(),
            uri:         "snowflake://crm/users".to_string(),
            source_type: DataSourceType::Snowflake,
        };
        let pii_tag   = ProvenanceTag { source: src.clone(), transforms: vec![], pii: true };
        let clean_tag = ProvenanceTag { source: src,         transforms: vec![], pii: false };

        let plan = generate_erasure_plan(&pii_tag);
        assert!(plan.is_some(), "pii=true → Some");
        let plan = plan.unwrap();
        assert!(plan.target_uri.contains("snowflake://crm/users"), "target_uri");
        assert!(!plan.fields.is_empty(), "fields non-empty");
        assert!(plan.reason.contains("GDPR"), "reason contains GDPR");

        assert!(generate_erasure_plan(&clean_tag).is_none(), "pii=false → None");
    }
}
```

- `cargo test v763000` で 2 件が pass することを確認する
- **注:** この時点で `cargo test`（全体）はまだ実行しない。Step 5 でバージョン文字列を更新するまで `cargo_toml_version_is_X` 系テストが FAIL する。

---

## Step 5: Cargo.toml バージョン更新

`fav/Cargo.toml`: `76.2.0` → `76.3.0`
`driver.rs` 内の `76.2.0` バージョン文字列アサーションを `76.3.0` に一括更新（replace_all）。

---

## Step 6: versions/current.md 更新

- 進行中バージョン: v76.3.0
- 次に切る版: v76.4.0

---

## Step 7: 最終確認

`cargo test` が 3720 tests all pass であることを確認。
