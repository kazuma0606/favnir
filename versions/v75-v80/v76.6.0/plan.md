# v76.6.0 実装計画 — Cross-pipeline provenance

Date: 2026-08-15

---

## Step 1: driver.rs — PipelineProvenanceChain 構造体追加

`fav/src/driver.rs` の末尾に `// --- v76.6.0: Cross-pipeline provenance ---` コメントと構造体を追加する。

```rust
#[derive(Debug, Clone)]
pub struct PipelineProvenanceChain {
    pub pipelines:  Vec<String>,
    pub merged_tag: ProvenanceTag,
}
```

---

## Step 2: driver.rs — chain_provenance 追加

```rust
pub fn chain_provenance(upstream: &ProvenanceTag, pipeline_name: &str) -> ProvenanceTag {
    let mut transforms = upstream.transforms.clone();
    transforms.push(pipeline_name.to_string());
    ProvenanceTag {
        source:     upstream.source.clone(),
        transforms,
        pii:        upstream.pii,
    }
}
```

---

## Step 3: driver.rs — format_chain_report 追加

```rust
pub fn format_chain_report(chain: &PipelineProvenanceChain) -> String {
    let pipelines = chain.pipelines.join(",");
    format!(
        "pipelines=[{}] source={} pii={}",
        pipelines,
        chain.merged_tag.source.name,
        chain.merged_tag.pii,
    )
}
```

---

## Step 4: cargo test（既存テスト通過確認）

`cargo test` を実行し、既存の 3724 テストが引き続き pass することを確認する（新規テストモジュールはまだ追加しない）。

---

## Step 5: CHANGELOG.md 更新

`CHANGELOG.md` の先頭に v76.6.0 エントリを追加する（テストモジュール追加より先）。

---

## Step 6: driver.rs — v766000_tests モジュール追加

```rust
#[cfg(test)]
mod v766000_tests {
    use super::*;  // PipelineProvenanceChain, chain_provenance, format_chain_report, DataSource, DataSourceType, ProvenanceTag を参照するため必須

    #[test]
    fn cross_pipeline_provenance_chained() { ... }

    #[test]
    fn cross_pipeline_pii_propagated() { ... }
}
```

---

## Step 7: Cargo.toml バージョン更新

`76.5.0` → `76.6.0`

---

## Step 8: versions/current.md 更新

進行中バージョンを v76.6.0 に、次に切る版を v76.7.0 に更新する。

---

## Step 9: 最終確認

`cargo test` が 3726 tests all pass であることを確認する。
