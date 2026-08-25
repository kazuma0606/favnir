# plan: v84.6.0 — ドキュメント完全化（`site/content/docs/v4/`）

## 実装ステップ（依存順）

### Step 1: 事前確認

- `cargo test` を実行し、3,919 tests, 0 failures を確認する（前提: v84.5.0 完了済み）
- `grep -m1 '^version' fav/Cargo.toml` の出力が `version = "84.0.0"` であることを確認する
  （v84.x マイナーバージョンは Cargo.toml を更新しない慣例）
- `fav/src/driver.rs` に `mod v84500_tests` が存在することを確認する
- `site/content/docs/v4/` ディレクトリが存在しないことを確認する（新規作成）

> 注: ロードマップ計画値は 3,907/3,909 だが、code-reviewer 対応の累積で実績ベースは 3,919/3,921。

### Step 2: `site/content/docs/v4/` ディレクトリ作成と MDX ファイル追加

`site/content/docs/v4/` ディレクトリを作成し、以下の 5 ファイルを追加する。

#### 2-1: `test-driven-data.mdx`

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

#### 2-2: `data-quality.mdx`

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

#### 2-3: `pipeline-contracts.mdx`

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

#### 2-4: `observability.mdx`

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

#### 2-5: `migration-v3-v4.mdx`

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

### Step 3: driver.rs に v84600_tests を追加

`mod v84500_tests` の直後に `#[cfg(test)] mod v84600_tests` を追加する。

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

### Step 4: cargo test で全 pass 確認

`cargo test 2>&1 | grep "test result"` を実行し、3,921 tests, 0 failures を確認する。

### Step 5: CHANGELOG 更新

`CHANGELOG.md` の先頭に v84.6.0 エントリを追加する。

### Step 6: CI 事前確認

- `cargo clippy --locked -- -D warnings` が pass することを確認する
- `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
