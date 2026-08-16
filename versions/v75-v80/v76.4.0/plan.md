# v76.4.0 実装計画 — OpenLineage 統合強化

Date: 2026-08-15

---

## Step 1: driver.rs — OpenLineageFacet 構造体追加

`fav/src/driver.rs` の末尾に `// --- v76.4.0: OpenLineage 統合強化 ---` コメントと `OpenLineageFacet` 構造体を追加する。

```rust
#[derive(Debug, Clone)]
pub struct OpenLineageFacet {
    pub producer:        String,
    pub data_source_uri: String,
    pub transforms:      Vec<String>,
}
```

---

## Step 2: driver.rs — provenance_to_openlineage 追加

```rust
pub fn provenance_to_openlineage(tag: &ProvenanceTag) -> OpenLineageFacet {
    OpenLineageFacet {
        producer:        "favnir/v76".to_string(),
        data_source_uri: tag.source.uri.clone(),
        transforms:      tag.transforms.clone(),
    }
}
```

---

## Step 3: driver.rs — format_openlineage_json 追加

手書き JSON フォーマット（外部ライブラリ不使用）。

```rust
pub fn format_openlineage_json(facet: &OpenLineageFacet) -> String {
    let transforms_json = if facet.transforms.is_empty() {
        "[]".to_string()
    } else {
        let items: Vec<String> = facet.transforms.iter()
            .map(|t| format!("\"{}\"", t))
            .collect();
        format!("[{}]", items.join(","))
    };
    format!(
        "{{\"_producer\":\"{}\",\"dataSource\":{{\"uri\":\"{}\"}},\"transforms\":{}}}",
        facet.producer, facet.data_source_uri, transforms_json
    )
}
```

---

## Step 4: cargo test（既存テスト通過確認）

`cargo test` を実行し、既存の 3720 テストが引き続き pass することを確認する（新規テストモジュールはまだ追加しない）。

---

## Step 5: CHANGELOG.md 更新

`CHANGELOG.md` の先頭に v76.4.0 エントリを追加する（テストモジュール追加より先）。

---

## Step 6: driver.rs — v764000_tests モジュール追加

```rust
#[cfg(test)]
mod v764000_tests {
    use super::*;

    #[test]
    fn openlineage_facet_from_provenance() { ... }

    #[test]
    fn openlineage_json_format() { ... }
}
```

---

## Step 7: Cargo.toml バージョン更新

`76.3.0` → `76.4.0`

---

## Step 8: versions/current.md 更新

進行中バージョンを v76.4.0 に、次に切る版を v76.5.0 に更新する。

---

## Step 9: 最終確認

`cargo test` が 3722 tests all pass であることを確認する。
