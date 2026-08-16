# v76.9.0 仕様書 — 安定化・コードフリーズ（Data Provenance 前最終調整）

Date: 2026-08-15
Status: 計画中

---

## Background

v76.1.0〜v76.8.0 で実装した Data Provenance 1.0 スプリントの全型・関数を統合的に検証し、v77.0.0 宣言前の最終安定化を行う。新型の追加は行わず、スプリント全体の E2E 統合テスト 2 件を追加してコードフリーズする。

---

## Goals

1. `provenance_full_sprint_all_stable` テストを追加する — スプリント全体の主要型が協調動作することを確認
2. `provenance_e2e_pipeline_valid` テストを追加する — 来歴付きパイプラインの E2E シナリオを検証
3. 3730 + 2 = 3732 tests に到達する

---

## テスト仕様

### `provenance_full_sprint_all_stable`

スプリント全体（v76.1.0〜v76.8.0）の主要型を一連のフローで組み合わせ、パニックや実行時エラーが発生しないことを確認する。

```rust
// DataSource → ProvenanceTag → TracedData → chain_provenance → OpenLineageFacet
let src = DataSource {
    name: "crm".to_string(),
    uri:  "snowflake://crm/users".to_string(),
    source_type: DataSourceType::Snowflake,
};
let tag = ProvenanceTag { source: src.clone(), transforms: vec![], pii: false };

// TracedData マッピング
let traced = TracedData { data: "row1".to_string(), provenance: tag.clone() };
let traced2 = map_traced(traced, "normalize");
assert!(traced2.provenance.transforms.contains(&"normalize".to_string()));

// chain_provenance
let chained = chain_provenance(&traced2.provenance, "ml-pipeline");
assert!(chained.transforms.contains(&"ml-pipeline".to_string()));

// OpenLineage 変換
let facet = provenance_to_openlineage(&chained);
assert_eq!(facet.producer, "favnir/v76");

// LineageGraph 構築
let graph = LineageGraph {
    nodes: vec![
        LineageNode { id: "n1".to_string(), node_type: LineageNodeType::Source, label: src.uri.clone() },
    ],
    edges: vec![],
};
let dot = format_lineage_dot(&graph);
assert!(dot.starts_with("digraph lineage {"));
```

### `provenance_e2e_pipeline_valid`

Snowflake から S3 へのデータパイプラインシナリオ。PII なし・ソース宣言済み・コントラクト準拠を一気通貫で検証する。

```rust
// ソース定義
let src = DataSource {
    name: "snowflake-crm".to_string(),
    uri:  "snowflake://crm/users".to_string(),
    source_type: DataSourceType::Snowflake,
};
// 来歴タグ（マスク済み・PII なし）
let tag = ProvenanceTag {
    source:     src,
    transforms: vec!["mask_pii".to_string(), "normalize_email".to_string()],
    pii:        false,
};

// ErasurePlan → None（pii=false）
assert!(generate_erasure_plan(&tag).is_none());

// DataProduct 検証 → Ok
let product = DataProduct {
    name:  "customer-360".to_string(),
    owner: "data-platform-team".to_string(),
    sla:   DataProductSla { freshness_minutes: 60 },
    provenance_policy: ProvenancePolicy {
        require_source_declared: true,
        pii_must_be_masked:      true,
    },
};
assert!(validate_data_product(&product, &tag).is_ok());

// ProvenanceContract 検証 → Ok
let contract = ProvenanceContract {
    allowed_sources: vec![DataSourceType::Snowflake, DataSourceType::S3],
    pii_policy:      PiiPolicy::MustBeMasked,
};
assert!(validate_provenance_contract(&contract, &tag).is_ok());

// OpenLineage JSON 出力
let facet = provenance_to_openlineage(&tag);
let json = format_openlineage_json(&facet);
assert!(json.contains("\"_producer\""));
assert!(json.contains("\"mask_pii\""));
```

---

## Success Criteria

- `provenance_full_sprint_all_stable` が pass — スプリント全型の協調動作確認
- `provenance_e2e_pipeline_valid` が pass — E2E パイプラインシナリオ全検証通過
- `cargo test` が 3732 tests all pass
- `driver.rs` 内の `76.8.0` バージョン文字列アサーションがすべて `76.9.0` に更新されている
- `CHANGELOG.md` の先頭に v76.9.0 エントリが存在する

---

## 変更ファイル

- `fav/src/driver.rs` — `v769000_tests` を追加（新型・新関数の追加はなし）。`76.8.0` バージョン文字列アサーション（`cargo_toml_version_is_76_0_0` の `version = "76.8.0"` と `changelog_has_v76_0_0` の `[v76.8.0]` を含む）を `76.9.0` へ一括置換する
- `CHANGELOG.md` — v76.9.0 エントリを追加
- `versions/current.md` — 進行中バージョンを更新
- `fav/Cargo.toml` — バージョンを `76.8.0` → `76.9.0` に更新

---

## 対象外

- 新しい型・関数の追加（コードフリーズ版）
- site/ MDX 追加（v77.0 宣言バージョンで実施）
- API や CLI の変更
- `changelog_has_v76_9_0` テストの追加（x.0.0 宣言バージョンのみに追加する慣例）
