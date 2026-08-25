# v83.1.0 タスクリスト

Status: COMPLETE

---

## T0: 事前確認

- [x] `cargo test` が 3,887 tests pass、0 failures であることを確認する（前提: v83.0.0 完了済み）

## T1: `test_framework.rs` に型定義を追加

- [x] `StageMetrics` 構造体を追加する（`#[derive(Debug, Clone, PartialEq)]`）
  - `stage_name: String`, `duration_ms: u64`, `rows_processed: usize`, `rows_failed: usize`
- [x] `PipelineMetrics` 構造体を追加する（`#[derive(Debug, Clone, PartialEq)]`）
  - `pipeline_name: String`, `stages: Vec<StageMetrics>`, `total_duration_ms: u64`, `started_at: String`

## T2: `test_framework.rs` に関数を追加

- [x] `compute_pipeline_metrics(pipeline_name: &str, stages: Vec<StageMetrics>, started_at: &str) -> PipelineMetrics` を追加する
- [x] `format_metrics_summary(metrics: &PipelineMetrics) -> String` を追加する
- [x] `slowest_stage(metrics: &PipelineMetrics) -> Option<&StageMetrics>` を追加する

## T3: `driver.rs` に `v83100_tests` を追加

- [x] `v83000_tests` の直後に `#[cfg(test)] mod v83100_tests` を追加する
  - `pipeline_metrics_computed`（`format_metrics_summary` アサーション含む）
  - `slowest_stage_identified`（空ステージ `None` 確認含む）

## T4: テスト通過確認

- [x] `cargo test` が 3,889 tests pass（+2）、0 failures であることを確認する

## T5: 最終確認（CI チェック）

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する

## code-reviewer 対応

- [x] [LOW] `compute_pipeline_metrics` のドキュメントコメントに並列ステージ時の注意（wall-clock total との差異）を明記

## 実装メモ

- `v83100_tests` は `use fav_core::test_framework::*;` を使用（`use super::*;` ではない — 他の v8x000_tests と同様）
- spec-reviewer 指摘対応: ロードマップの `compute_pipeline_metrics` シグネチャ修正 + テスト数ドリフト補正（3875→3887 基準）
- `pipeline_metrics_computed` 内に `format_metrics_summary` スモークテストを追加
- `slowest_stage_identified` 内に空ステージ `None` 確認を追加
