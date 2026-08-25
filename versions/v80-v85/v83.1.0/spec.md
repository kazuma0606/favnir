# v83.1.0 仕様書 — `PipelineMetrics` 型（実行統計・レイテンシ）

## Background

v83.0.0「Pipeline Contracts 1.0」でパイプラインの契約を型として宣言できるようになった。
次のステップとして、パイプライン実行の統計情報（ステージ別レイテンシ・処理行数・失敗行数）を
構造化した型として収集・集計する基盤を整備する。

本バージョンは Observability 2.0 スプリント（v83.1〜v84.0）の第 1 段階。

ロードマップ参照: `versions/roadmap/roadmap-v83.1-v84.0.md` — v83.1.0 セクション

## Goals

1. `StageMetrics` 構造体を追加する（ステージ単位の実行統計）
2. `PipelineMetrics` 構造体を追加する（パイプライン全体の統計集約）
3. `compute_pipeline_metrics` 関数を追加する
4. `format_metrics_summary` 関数を追加する
5. `slowest_stage` 関数を追加する

## 型定義・API

```rust
// ステージ単位の実行統計
#[derive(Debug, Clone, PartialEq)]
pub struct StageMetrics {
    pub stage_name: String,
    pub duration_ms: u64,
    pub rows_processed: usize,
    pub rows_failed: usize,
}

// パイプライン全体の統計
#[derive(Debug, Clone, PartialEq)]
pub struct PipelineMetrics {
    pub pipeline_name: String,
    pub stages: Vec<StageMetrics>,
    pub total_duration_ms: u64,
    pub started_at: String,  // RFC 3339 / ISO 8601 形式
}

// stages をソートせず順序保持のまま total_duration_ms を合算して構築
pub fn compute_pipeline_metrics(
    pipeline_name: &str,
    stages: Vec<StageMetrics>,
    started_at: &str,
) -> PipelineMetrics

// サマリーテキストを返す
// 例: "Pipeline: etl_main\nStarted: 2026-08-21T00:00:00Z\nTotal: 350ms\nStages:\n  load: 200ms (1000 rows, 0 failed)\n  transform: 150ms (1000 rows, 2 failed)"
pub fn format_metrics_summary(metrics: &PipelineMetrics) -> String

// 最も duration_ms が大きい StageMetrics への参照を返す
// stages が空の場合は None
pub fn slowest_stage(metrics: &PipelineMetrics) -> Option<&StageMetrics>
```

## テスト（v83.1.0 で追加）

実際のテスト数ベース（※ drift 補正後）: **3887 + 2 = 3889**

（ロードマップ記載値 3875 + 2 = 3877 は旧バージョン到達時点のドリフト。
 実際の v83.0.0 完了テスト数は 3887。）

### `pipeline_metrics_computed`

```rust
let stages = vec![
    StageMetrics { stage_name: "load".into(), duration_ms: 200, rows_processed: 1000, rows_failed: 0 },
    StageMetrics { stage_name: "transform".into(), duration_ms: 150, rows_processed: 1000, rows_failed: 2 },
];
let metrics = compute_pipeline_metrics("etl_main", stages, "2026-08-21T00:00:00Z");
assert_eq!(metrics.pipeline_name, "etl_main");
assert_eq!(metrics.total_duration_ms, 350);
assert_eq!(metrics.stages.len(), 2);
// format_metrics_summary のスモークテスト（"Pipeline:" / "Total:" / "Stages:" を含む）
let summary = format_metrics_summary(&metrics);
assert!(summary.contains("Pipeline:"), "summary should contain 'Pipeline:'");
assert!(summary.contains("Total:"), "summary should contain 'Total:'");
assert!(summary.contains("Stages:"), "summary should contain 'Stages:'");
```

### `slowest_stage_identified`

```rust
let stages = vec![
    StageMetrics { stage_name: "load".into(), duration_ms: 200, rows_processed: 1000, rows_failed: 0 },
    StageMetrics { stage_name: "transform".into(), duration_ms: 150, rows_processed: 1000, rows_failed: 2 },
];
let metrics = compute_pipeline_metrics("etl_main", stages, "2026-08-21T00:00:00Z");
let slowest = slowest_stage(&metrics).expect("slowest stage should exist");
assert_eq!(slowest.stage_name, "load");
assert_eq!(slowest.duration_ms, 200);
// 空ステージの場合は None を返すことを確認
let empty_metrics = compute_pipeline_metrics("empty_pipe", vec![], "2026-08-21T00:00:00Z");
assert!(slowest_stage(&empty_metrics).is_none(), "slowest_stage should return None for empty stages");
```

## Success Criteria

- `cargo test` が 3889 tests pass（+2）、0 failures
- `pipeline_metrics_computed` テスト内で `format_metrics_summary` が "Pipeline:"、"Total:"、"Stages:" を含むことをアサート
- `slowest_stage` が stages 空のとき `None` を返す（`slowest_stage_identified` テスト内で空ステージ確認を含む）

## Files to Modify

- `fav/src/test_framework.rs` — 型定義・関数追加
- `fav/src/driver.rs` — `v83100_tests` モジュール追加
