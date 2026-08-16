# v76.2.0 実装計画 — `TracedData` 型

Date: 2026-08-15

---

## Step 1: driver.rs — 型・関数追加

`fav/src/driver.rs` の末尾に以下を追加する：

```rust
// --- v76.2.0: TracedData 型 ---

#[derive(Debug, Clone)]
pub struct TracedData {
    pub data:       String,
    pub provenance: ProvenanceTag,
}

pub fn map_traced(t: TracedData, transform_label: &str) -> TracedData {
    let mut new_transforms = t.provenance.transforms.clone();
    new_transforms.push(transform_label.to_string());
    TracedData {
        data: t.data,
        provenance: ProvenanceTag {
            source:     t.provenance.source,
            transforms: new_transforms,
            pii:        t.provenance.pii,
        },
    }
}

pub fn merge_provenance(a: &ProvenanceTag, b: &ProvenanceTag) -> ProvenanceTag {
    let mut transforms = a.transforms.clone();
    transforms.extend(b.transforms.iter().cloned());
    ProvenanceTag {
        source:     a.source.clone(),
        transforms,
        pii:        a.pii || b.pii,
    }
}
```

---

## Step 2: cargo check

`cargo check` でコンパイルエラーがないことを確認する。

---

## Step 3: CHANGELOG.md 更新（テスト追加より先）

`CHANGELOG.md` の先頭に v76.2.0 エントリを追加する。

---

## Step 4: テストモジュール v762000_tests 追加

```rust
#[cfg(test)]
mod v762000_tests {
    use super::*;

    #[test]
    fn traced_map_appends_transform() {
        let source = DataSource {
            name:        "crm".to_string(),
            uri:         "snowflake://warehouse/crm".to_string(),
            source_type: DataSourceType::Snowflake,
        };
        let tag = ProvenanceTag { source, transforms: vec![], pii: false };
        let t = TracedData { data: "rows".to_string(), provenance: tag };

        // 1 回変換
        let t2 = map_traced(t, "mask_pii");
        assert!(t2.provenance.transforms.contains(&"mask_pii".to_string()));
        assert_eq!(t2.provenance.transforms.len(), 1);

        // 2 回変換
        let t3 = map_traced(t2, "normalize_email");
        assert_eq!(t3.provenance.transforms.len(), 2);
        assert!(t3.provenance.transforms.contains(&"normalize_email".to_string()));

        // data は変化しない
        assert_eq!(t3.data, "rows");
    }

    #[test]
    fn traced_merge_propagates_pii() {
        let src_a = DataSource {
            name: "a".to_string(), uri: "s3://a".to_string(), source_type: DataSourceType::S3,
        };
        let src_b = DataSource {
            name: "b".to_string(), uri: "s3://b".to_string(), source_type: DataSourceType::Manual,
        };

        // pii=false + pii=true → merged pii=true
        let tag_a = ProvenanceTag {
            source: src_a.clone(), transforms: vec!["t1".to_string()], pii: false,
        };
        let tag_b = ProvenanceTag {
            source: src_b.clone(), transforms: vec!["t2".to_string()], pii: true,
        };
        let merged = merge_provenance(&tag_a, &tag_b);
        assert!(merged.pii, "pii propagates via OR");
        assert_eq!(merged.transforms.len(), 2); // t1 + t2
        assert_eq!(merged.source.name, "a");    // 左辺ソースが優先

        // pii=false + pii=false → merged pii=false
        let tag_c = ProvenanceTag {
            source: src_a, transforms: vec![], pii: false,
        };
        let tag_d = ProvenanceTag {
            source: src_b, transforms: vec![], pii: false,
        };
        let merged2 = merge_provenance(&tag_c, &tag_d);
        assert!(!merged2.pii, "both false → false");
    }
}
```

- `cargo test v762000` で 2 件が pass することを確認する

---

## Step 5: Cargo.toml バージョン更新

`fav/Cargo.toml`: `76.1.0` → `76.2.0`
`driver.rs` 内の `76.1.0` バージョン文字列アサーションを `76.2.0` に一括更新（replace_all）。

---

## Step 6: versions/current.md 更新

- 進行中バージョン: v76.2.0
- 次に切る版: v76.3.0

---

## Step 7: 最終確認

`cargo test` が 3718 tests all pass であることを確認。
