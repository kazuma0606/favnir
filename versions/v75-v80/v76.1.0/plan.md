# v76.1.0 実装計画 — `DataSource` / `ProvenanceTag` 型基盤

Date: 2026-08-15

---

## Step 1: driver.rs — 型・関数追加

`fav/src/driver.rs` の末尾に以下を追加する：

```rust
// --- v76.1.0: DataSource / ProvenanceTag 型基盤 ---

#[derive(Debug, Clone, PartialEq)]
pub enum DataSourceType {
    Snowflake,
    S3,
    Api,
    Manual,
    Pipeline,
}

#[derive(Debug, Clone)]
pub struct DataSource {
    pub name:        String,
    pub uri:         String,
    pub source_type: DataSourceType,
}

#[derive(Debug, Clone)]
pub struct ProvenanceTag {
    pub source:     DataSource,
    pub transforms: Vec<String>,
    pub pii:        bool,
}

pub fn format_provenance_tag(tag: &ProvenanceTag) -> String {
    let transforms = tag.transforms.join(",");
    format!(
        "source={} type={:?} transforms=[{}] pii={}",
        tag.source.name,
        tag.source.source_type,
        transforms,
        tag.pii,
    )
}
```

---

## Step 2: cargo check

`cargo check` でコンパイルエラーがないことを確認する。

---

## Step 3: CHANGELOG.md 更新（テスト追加より先）

`CHANGELOG.md` の先頭に v76.1.0 エントリを追加する。

---

## Step 4: テストモジュール v761000_tests 追加

```rust
#[cfg(test)]
mod v761000_tests {
    use super::*;

    #[test]
    fn provenance_tag_created() {
        let source = DataSource {
            name:        "crm".to_string(),
            uri:         "snowflake://warehouse/crm/users".to_string(),
            source_type: DataSourceType::Snowflake,
        };
        let tag = ProvenanceTag {
            source,
            transforms: vec!["mask_pii".to_string(), "normalize_email".to_string()],
            pii: false,
        };
        let s = format_provenance_tag(&tag);
        assert!(s.contains("crm"),        "source name");
        assert!(s.contains("Snowflake"),  "source type");
        assert!(s.contains("mask_pii"),   "transform present");
        assert!(s.contains("pii=false"),  "pii flag");
    }

    #[test]
    fn provenance_pii_flagged() {
        // PII あり・変換なし
        let source_pii = DataSource {
            name:        "upload".to_string(),
            uri:         "s3://bucket/upload".to_string(),
            source_type: DataSourceType::S3,
        };
        let tag_pii = ProvenanceTag { source: source_pii, transforms: vec![], pii: true };
        let s_pii = format_provenance_tag(&tag_pii);
        assert!(s_pii.contains("pii=true"),       "pii flag true");
        assert!(s_pii.contains("S3"),              "source type S3");
        assert!(s_pii.contains("transforms=[]"),   "empty transforms");

        // PII なし・Api ソース
        let source_clean = DataSource {
            name:        "api".to_string(),
            uri:         "https://api.example.com".to_string(),
            source_type: DataSourceType::Api,
        };
        let tag_clean = ProvenanceTag { source: source_clean, transforms: vec![], pii: false };
        assert!(format_provenance_tag(&tag_clean).contains("pii=false"));
    }
}
```

---

## Step 5: Cargo.toml バージョン更新

`fav/Cargo.toml`: `76.0.0` → `76.1.0`
`driver.rs` 内に `76.0.0` をアサートしているテスト（`cargo_toml_version_is_76_0_0` 等）が存在する場合は replace_all で `76.1.0` に更新する。存在しない場合はスキップ。

---

## Step 6: versions/current.md 更新

- 進行中バージョン: v76.1.0
- 次に切る版: v76.2.0

---

## Step 7: 最終確認

`cargo test` が 3716 tests all pass であることを確認。
