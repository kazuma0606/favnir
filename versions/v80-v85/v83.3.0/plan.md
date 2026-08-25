# v83.3.0 実装計画 — `SloTarget` / `SloStatus`（SLO 型）

## 依存関係

新規型・関数のみ。既存コードへの変更なし。
`compute_slo_status` は `SloTarget` と `SloMeasurement` のみに依存する（`PipelineMetrics` 不要）。

## 実装ステップ

### Step 1: `test_framework.rs` に構造体を追加

v83.2.0 追加ブロック（`evaluate_alert_rules` 末尾）の後に追加する。

1. `SloTarget` 構造体（`#[derive(Debug, Clone, PartialEq)]`）
   - `name: String`, `objective_pct: f64`, `window_hours: u64`

2. `SloMeasurement` 構造体（`#[derive(Debug, Clone, PartialEq)]`）
   - `good_events: u64`, `total_events: u64`, `window_hours: u64`

3. `SloStatus` 構造体（`#[derive(Debug, Clone, PartialEq)]`）
   - `target: SloTarget`, `current_pct: f64`, `error_budget_remaining_pct: f64`, `breached: bool`

### Step 2: `compute_slo_status` 関数を追加

```
fn compute_slo_status(target: &SloTarget, measurement: &SloMeasurement) -> SloStatus
```

実装方針:
- `total_events == 0` → `current_pct = 100.0`、`error_budget_remaining_pct = 100.0`、`breached = false`
- それ以外: `current_pct = measurement.good_events as f64 / measurement.total_events as f64 * 100.0`
- `error_budget_remaining_pct = (current_pct - target.objective_pct) / (100.0 - target.objective_pct) * 100.0`
- `breached = current_pct < target.objective_pct`
- `objective_pct == 100.0` のとき分母がゼロになるため、`error_budget_remaining_pct` の分母チェック:
  `(100.0 - target.objective_pct).abs() < f64::EPSILON` のときは `0.0` を返す

### Step 3: `format_slo_status` 関数を追加

```
fn format_slo_status(status: &SloStatus) -> String
```

出力形式:
```
SLO: {name}
Objective: {objective_pct:.1}%
Current: {current_pct:.2}%
Error Budget: {error_budget_remaining_pct:.2}% remaining
Status: OK  （または BREACHED）
```

### Step 4: `driver.rs` に `v83300_tests` を追加

`v83200_tests` の直後に追加する。

```rust
#[cfg(test)]
mod v83300_tests {
    use fav_core::test_framework::*;

    #[test]
    fn slo_status_within_budget() { ... }

    #[test]
    fn slo_status_breached() { ... }
}
```

### Step 5: `cargo test` で全テスト通過を確認

期待: 3893 tests pass、0 failures

### Step 6: CI チェック

- `cargo clippy --locked -- -D warnings` が pass することを確認
- `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認
- `./target/debug/fav fmt --check self/checker.fav` が pass することを確認
