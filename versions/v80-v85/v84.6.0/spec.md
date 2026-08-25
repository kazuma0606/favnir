# spec: v84.6.0 — ドキュメント完全化（`site/content/docs/v4/`）

## Background

> **テスト数注記**: ロードマップ計画値は 3,907/3,909 だったが、code-reviewer 対応の
> 累積により実際のベースは **3,919 tests**（v84.5.0 完了時点）。
> v84.6.0 完了目標は **3,921 tests**（+2）。

v84.1.0〜v84.5.0 で Favnir 4.0 の 4 スプリント（Test-Driven Data / Data Quality 2.0 /
Pipeline Contracts 1.0 / Observability 2.0）をショーケース統合した。v84.6.0 では
これら全機能のリファレンスドキュメントを `site/content/docs/v4/` に追加し、
v3（v80.0）→ v4（v85.0）移行ガイドも含めてドキュメントを完全化する。

## Goals

1. `site/content/docs/v4/` に 5 件の MDX ドキュメントを新規作成する
   - `test-driven-data.mdx` — `fav test` / `TestSuite` / `GoldenDataset` / `SchemaSnapshot`
   - `data-quality.mdx` — `QualityRule` / `QualityGate` / `AnomalyDetector`
   - `pipeline-contracts.mdx` — `IoContract` / `SlaContract` / `ContractRegistry`
   - `observability.mdx` — `PipelineMetrics` / `AlertRule` / `SloStatus` / `HealthDashboard`
   - `migration-v3-v4.mdx` — v3（v80.0）→ v4（v85.0）移行ガイド
2. Rust テスト 2 件で主要ドキュメントの存在を検証する
   - `docs_v4_test_driven_data_exists` — `test-driven-data.mdx` の存在確認
   - `docs_v4_migration_guide_exists` — `migration-v3-v4.mdx` の存在確認

## MDX ファイル構成

### `site/content/docs/v4/test-driven-data.mdx`

```markdown
# Test-Driven Data（Favnir 4.0）

Sprint 1 で導入したテスト駆動データパイプラインの機能を解説します。

## TestSuite / StageTestCase

`TestSuite` はパイプラインステージのテストケースをまとめる型です。

## GoldenDataset

`GoldenDataset` は期待値データセットを管理する型です。`compare_golden` 関数で
実行結果と比較します。

## SchemaSnapshot

`SchemaSnapshot` はスキーマの期待定義を保持します。`compare_schema_snapshots`
で現在のスキーマと差異を検出します。

## 参考

- [Favnir 4.0 ショーケース](../../infra/e2e-demo/favnir4-showcase/)
```

### `site/content/docs/v4/data-quality.mdx`

```markdown
# Data Quality 2.0（Favnir 4.0）

Sprint 2 で導入したデータ品質チェック機能を解説します。

## QualityRule / QualityCheck

`QualityRule` は品質評価ルール。`run_quality_check` で行単位に評価します。

## QualityGate

`QualityGate` はパイプライン停止条件を定義します。`evaluate_quality_gate` で
品質スコアが閾値を下回った場合にパイプラインを停止します。

## AnomalyDetector

`AnomalyDetector` は Z スコアに基づく外れ値検知を行います。`detect_anomaly`
で単一値の外れ値判定を返します。

## 参考

- [Favnir 4.0 ショーケース](../../infra/e2e-demo/favnir4-showcase/)
```

### `site/content/docs/v4/pipeline-contracts.mdx`

```markdown
# Pipeline Contracts 1.0（Favnir 4.0）

Sprint 3 で導入したパイプライン契約機能を解説します。

## IoContract

`IoContract` は入出力フィールドとバージョンを型として宣言します。

## SlaContract

`SlaContract` は SLA 目標値を型として宣言します。`adaptive_strategy` と
`cache_ttl_secs` を任意で設定できます。

## ContractRegistry / ContractRegistryEntry

`ContractRegistry` はパイプライン間で共有する契約レジストリです。
`ContractRegistryEntry` にはバージョン・登録日時（`registered_at`）が含まれます。

## 参考

- [Favnir 4.0 ショーケース](../../infra/e2e-demo/favnir4-showcase/)
```

### `site/content/docs/v4/observability.mdx`

```markdown
# Observability 2.0（Favnir 4.0）

Sprint 4 で導入した可観測性機能を解説します。

## PipelineMetrics / StageMetrics

`StageMetrics` はステージ単位の実行統計（duration_ms / rows_processed /
rows_failed）を保持します。`PipelineMetrics` はステージリストと全体統計を集約します。

## AlertRule / evaluate_alert_rules

`AlertRule` は `AlertThreshold`（metric / operator / value）と severity を持ちます。
`evaluate_alert_rules` でメトリクスに対してルールを評価し `AlertFiring` リストを返します。

## SloTarget / SloMeasurement / compute_slo_status

`SloTarget`（name / objective_pct / window_hours）と `SloMeasurement`
（good_events / total_events / window_hours）から `compute_slo_status` で
`SloStatus` を取得します。

## HealthDashboard / format_health_dashboard

`compute_pipeline_health` でパイプラインの総合健全性（`PipelineHealth`）を算出し、
`HealthDashboard` にまとめて `format_health_dashboard` で出力します。

## 参考

- [Favnir 4.0 ショーケース](../../infra/e2e-demo/favnir4-showcase/)
```

### `site/content/docs/v4/migration-v3-v4.mdx`

```markdown
# v3 → v4 移行ガイド

Favnir v3（v80.0）から v4（v85.0）への移行手順を説明します。

## 主な変更点

| 機能 | v3 | v4 |
|---|---|---|
| テスト | なし | Test-Driven Data（TestSuite / GoldenDataset / SchemaSnapshot）|
| 品質 | 基本 QualityRule | Data Quality 2.0（QualityGate / AnomalyDetector）|
| 契約 | なし | Pipeline Contracts（IoContract / SlaContract / ContractRegistry）|
| 可観測性 | なし | Observability 2.0（PipelineMetrics / AlertRule / HealthDashboard）|

## 移行手順

1. `TestSuite` でパイプラインステージのテストを定義する
2. `QualityGate` でパイプライン停止条件を設定する
3. `IoContract` で入出力スキーマを型として宣言する
4. `PipelineMetrics` でステージ実行統計を収集する

## 参考

- [Test-Driven Data](test-driven-data.mdx)
- [Data Quality 2.0](data-quality.mdx)
- [Pipeline Contracts](pipeline-contracts.mdx)
- [Observability 2.0](observability.mdx)
```

## 実際の型定義（参照）

| 型 / 関数 | v84.x バージョン |
|---|---|
| `TestSuite` / `StageTestCase` / `GoldenDataset` / `SchemaSnapshot` | v84.2.0（Sprint 1）|
| `QualityCheck` / `QualityGate` / `AnomalyDetector` | v84.3.0（Sprint 2）|
| `IoContract` / `SlaContract` / `ContractRegistry` | v84.4.0（Sprint 3）|
| `PipelineMetrics` / `AlertRule` / `SloStatus` / `HealthDashboard` | v84.5.0（Sprint 4）|

## Rust テスト（v84600_tests）

```rust
#[cfg(test)]
mod v84600_tests {
    #[test]
    fn docs_v4_test_driven_data_exists() {
        assert!(
            std::path::Path::new("../site/content/docs/v4/test-driven-data.mdx").exists(),
            "site/content/docs/v4/test-driven-data.mdx should exist"
        );
    }

    #[test]
    fn docs_v4_migration_guide_exists() {
        assert!(
            std::path::Path::new("../site/content/docs/v4/migration-v3-v4.mdx").exists(),
            "site/content/docs/v4/migration-v3-v4.mdx should exist"
        );
    }
}
```

## Success Criteria

- `site/content/docs/v4/` に 5 件の MDX ファイルが存在すること
  - `test-driven-data.mdx`、`data-quality.mdx`、`pipeline-contracts.mdx`、
    `observability.mdx`、`migration-v3-v4.mdx`
- 各 MDX ファイルに対応するスプリント機能の型名・関数名が含まれること
- `cargo test` が 3,921 tests pass（+2）、0 failures であること

## Error Codes

なし（本バージョンはファイル追加のみ）

## Files to Modify / Create

### 新規作成
- `site/content/docs/v4/test-driven-data.mdx`
- `site/content/docs/v4/data-quality.mdx`
- `site/content/docs/v4/pipeline-contracts.mdx`
- `site/content/docs/v4/observability.mdx`
- `site/content/docs/v4/migration-v3-v4.mdx`

### 追記
- `fav/src/driver.rs` — `v84600_tests` モジュール追加（2 テスト）
- `CHANGELOG.md` — v84.6.0 エントリ追加

### パス起点

`v84600_tests` は `std::path::Path::new("../site/...")` を使用。
パス起点は `fav/`（`cargo test` 実行時の CWD）。

> 注: `site/` MDX の内容検証（`include_str!`）は Rust テスト 2 件の範囲外。
> 存在確認のみ実施する。
