# v83.1.0 実装計画 — `PipelineMetrics` 型

## 依存関係

新規型・関数のみ。既存コードへの変更なし。

## 実装ステップ

### Step 1: `test_framework.rs` に型と関数を追加

`fav/src/test_framework.rs` の末尾に以下を追加する。

1. `StageMetrics` 構造体（`#[derive(Debug, Clone, PartialEq)]`）
   - `stage_name: String`
   - `duration_ms: u64`
   - `rows_processed: usize`
   - `rows_failed: usize`

2. `PipelineMetrics` 構造体（`#[derive(Debug, Clone, PartialEq)]`）
   - `pipeline_name: String`
   - `stages: Vec<StageMetrics>`
   - `total_duration_ms: u64`
   - `started_at: String`

3. `compute_pipeline_metrics(pipeline_name: &str, stages: Vec<StageMetrics>, started_at: &str) -> PipelineMetrics`
   - `total_duration_ms = stages.iter().map(|s| s.duration_ms).sum()`
   - `PipelineMetrics { pipeline_name: pipeline_name.to_string(), stages, total_duration_ms, started_at: started_at.to_string() }`

4. `format_metrics_summary(metrics: &PipelineMetrics) -> String`
   - ヘッダー: `"Pipeline: {name}\nStarted: {started_at}\nTotal: {total}ms\nStages:"`
   - ステージ行: `"  {stage_name}: {duration_ms}ms ({rows_processed} rows, {rows_failed} failed)"`
   - ステージ行は `\n` で結合し末尾改行なし（`stages.iter().map(...).collect::<Vec<_>>().join("\n")`）
   - ステージが空の場合、"Stages:" の後は何も追加しない（`"Stages:"` のみ）

5. `slowest_stage(metrics: &PipelineMetrics) -> Option<&StageMetrics>`
   - `metrics.stages.iter().max_by_key(|s| s.duration_ms)`

### Step 2: `driver.rs` に `v83100_tests` を追加

`driver.rs` の末尾近く（`v83000_tests` の直後）に追加する。

```rust
#[cfg(test)]
mod v83100_tests {
    use super::*;

    #[test]
    fn pipeline_metrics_computed() { ... }

    #[test]
    fn slowest_stage_identified() { ... }
}
```

### Step 3: `cargo test` で全テスト通過を確認

期待: 3889 tests pass、0 failures
