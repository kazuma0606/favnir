# v76.6.0 仕様書 — Cross-pipeline provenance

Date: 2026-08-15
Status: 計画中

---

## Background

複数のパイプラインを跨いだ来歴の連鎖を表現する。上流パイプライン（etl-pipeline-a）の来歴を下流パイプライン（ml-pipeline-b）が引き継ぐ仕組みを提供する。`PipelineProvenanceChain` でパイプライン名リストと合成済み来歴タグを保持し、`chain_provenance` で来歴を連鎖させ、`format_chain_report` でレポート文字列を生成する。

---

## Goals

1. `PipelineProvenanceChain` 構造体（pipelines: Vec<String>, merged_tag: ProvenanceTag）を追加する
2. `chain_provenance(upstream: &ProvenanceTag, pipeline_name: &str) -> ProvenanceTag` を追加する
3. `format_chain_report(chain: &PipelineProvenanceChain) -> String` を追加する
4. Rust テスト 2 件を追加し 3726 tests に到達する

---

## 型・関数仕様

### `PipelineProvenanceChain` 構造体

```rust
#[derive(Debug, Clone)]
pub struct PipelineProvenanceChain {
    pub pipelines:  Vec<String>,
    pub merged_tag: ProvenanceTag,
}
```

> **注**: ロードマップのサンプルコードでは `inherited_tag` と記載されていたが、"マージ済み来歴タグ" の意味をより正確に表す `merged_tag` に変更した。



---

### `chain_provenance`

```rust
pub fn chain_provenance(upstream: &ProvenanceTag, pipeline_name: &str) -> ProvenanceTag
```

**動作:**
- `source`: `upstream.source.clone()`
- `transforms`: `upstream.transforms` の末尾に `pipeline_name.to_string()` を追加したコピー
- `pii`: `upstream.pii`（そのまま引き継ぐ）

---

### `format_chain_report`

```rust
pub fn format_chain_report(chain: &PipelineProvenanceChain) -> String
```

**出力フォーマット:**

```
pipelines=[<p1>,<p2>,...] source=<source.name> pii=<bool>
```

**例:**
```
pipelines=[etl-pipeline-a,ml-pipeline-b] source=crm pii=false
```

- `pipelines` の各要素をカンマ区切りで `[...]` に囲む
- `source` は `chain.merged_tag.source.name`
- `pii` は `chain.merged_tag.pii`

---

## テスト仕様

### `cross_pipeline_provenance_chained`

```rust
let src = DataSource {
    name:        "crm".to_string(),
    uri:         "snowflake://crm/users".to_string(),
    source_type: DataSourceType::Snowflake,
};
let upstream = ProvenanceTag {
    source:     src,
    transforms: vec!["mask_pii".to_string()],
    pii:        false,
};
let chained = chain_provenance(&upstream, "ml-pipeline-b");
assert_eq!(chained.source.name, "crm");
assert!(chained.transforms.contains(&"mask_pii".to_string()));
assert!(chained.transforms.contains(&"ml-pipeline-b".to_string()));
assert_eq!(chained.pii, false);

let chain = PipelineProvenanceChain {
    pipelines:  vec!["etl-pipeline-a".to_string(), "ml-pipeline-b".to_string()],
    merged_tag: chained,
};
assert_eq!(chain.pipelines.len(), 2);
```

### `cross_pipeline_pii_propagated`

```rust
let src = DataSource {
    name:        "crm".to_string(),
    uri:         "snowflake://crm/users".to_string(),
    source_type: DataSourceType::Snowflake,
};
let pii_upstream = ProvenanceTag {
    source:     src,
    transforms: vec![],
    pii:        true,
};
let chained = chain_provenance(&pii_upstream, "ml-pipeline-b");
assert_eq!(chained.pii, true, "pii is propagated from upstream");

let chain = PipelineProvenanceChain {
    pipelines:  vec!["etl-pipeline-a".to_string(), "ml-pipeline-b".to_string()],
    merged_tag: chained,
};
let report = format_chain_report(&chain);
assert!(report.contains("pipelines="));
assert!(report.contains("etl-pipeline-a"));
assert!(report.contains("source=crm"));
assert!(report.contains("pii=true"));
```

---

## Success Criteria

- `PipelineProvenanceChain` 構造体が定義されている
- `chain_provenance` が upstream の source・pii を引き継ぎ transforms に pipeline_name を追加する
- `format_chain_report` が `pipelines=[...] source=<name> pii=<bool>` 形式の文字列を返す
- `cross_pipeline_provenance_chained` が pass
- `cross_pipeline_pii_propagated` が pass
- `cargo test` が 3726 tests all pass
- `CHANGELOG.md` の先頭に v76.6.0 エントリが存在する

---

## 変更ファイル

- `fav/src/driver.rs` — `PipelineProvenanceChain`, `chain_provenance`, `format_chain_report`, `v766000_tests` を追加
- `CHANGELOG.md` — v76.6.0 エントリを追加
- `versions/current.md` — 進行中バージョンを更新
- `fav/Cargo.toml` — バージョンを `76.5.0` → `76.6.0` に更新

---

## 依存（既実装）

- `DataSource` 構造体（v76.1.0）
- `DataSourceType` enum（v76.1.0）
- `ProvenanceTag` 構造体（v76.1.0）

---

## 対象外

- 実際のパイプライン実行時の自動来歴連鎖（将来バージョン）
- `format_chain_report` の JSON 形式出力（将来バージョン）
- パイプライン間の DAG 構築（将来バージョン）
