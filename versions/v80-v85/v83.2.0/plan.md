# v83.2.0 実装計画 — `AlertRule` / `AlertThreshold`（アラート型）

## 依存関係

`evaluate_alert_rules` が `PipelineMetrics` / `StageMetrics` を参照するため、
v83.1.0 の型定義が `test_framework.rs` に存在することが前提。

## 実装ステップ

### Step 1: `test_framework.rs` に enum と構造体を追加

`format_registry_listing` の直後（v83.1.0 追加ブロックの後）に追加する。

1. `AlertSeverity` enum（`#[derive(Debug, Clone, PartialEq)]`）
   - `Critical` / `Warning` / `Info`

2. `ThresholdOp` enum（`#[derive(Debug, Clone, PartialEq)]`）
   - `GreaterThan` / `LessThan` / `EqualTo`

3. `AlertThreshold` 構造体（`#[derive(Debug, Clone, PartialEq)]`）
   - `metric: String`, `operator: ThresholdOp`, `value: f64`

4. `AlertRule` 構造体（`#[derive(Debug, Clone, PartialEq)]`）
   - `name: String`, `threshold: AlertThreshold`, `severity: AlertSeverity`, `message: String`

5. `AlertFiring` 構造体（`#[derive(Debug, Clone, PartialEq)]`）
   - `rule: AlertRule`, `current_value: f64`, `fired_at: String`

### Step 2: `evaluate_alert_rules` 関数を追加

```
fn evaluate_alert_rules(rules: &[AlertRule], metrics: &PipelineMetrics, fired_at: &str) -> Vec<AlertFiring>
```

実装方針:
- `metric` ごとに `current_value` を算出:
  - `"total_duration_ms"` → `metrics.total_duration_ms as f64`
  - `"rows_failed"` → `metrics.stages.iter().map(|s| s.rows_failed).sum::<usize>() as f64`
  - `"rows_processed"` → `metrics.stages.iter().map(|s| s.rows_processed).sum::<usize>() as f64`
  - その他 → `continue`（スキップ）
- `ThresholdOp` で評価:
  - `GreaterThan`: `current_value > rule.threshold.value`
  - `LessThan`: `current_value < rule.threshold.value`
  - `EqualTo`: `(current_value - rule.threshold.value).abs() < f64::EPSILON`
- 条件を満たした場合 `AlertFiring { rule: rule.clone(), current_value, fired_at: fired_at.to_string() }` を push

### Step 3: `driver.rs` に `v83200_tests` を追加

`v83100_tests` の直後に追加する。

```rust
#[cfg(test)]
mod v83200_tests {
    use fav_core::test_framework::*;

    #[test]
    fn alert_fires_when_threshold_exceeded() { ... }

    #[test]
    fn alert_silent_when_within_threshold() { ... }
}
```

### Step 4: `cargo test` で全テスト通過を確認

期待: 3891 tests pass、0 failures

### Step 5: CI チェック

- `cargo clippy --locked -- -D warnings` が pass することを確認
- `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認
- `./target/debug/fav fmt --check self/checker.fav` が pass することを確認
