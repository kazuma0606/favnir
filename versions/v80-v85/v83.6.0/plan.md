# v83.6.0 実装計画 — パフォーマンス回帰検知（`PerfBaseline` / `PerfRegression`）

## 依存関係

新規型・関数のみ。既存コードへの変更なし。

## 実装ステップ

### Step 1: `test_framework.rs` に構造体と impl を追加

v83.5.0 追加ブロック（`compute_span_duration` 末尾）の後に追加する。

1. `PerfBaseline` 構造体（`#[derive(Debug, Clone, PartialEq)]`）
   - `pipeline_name: String`, `p50_ms: u64`, `p95_ms: u64`, `p99_ms: u64`

2. `impl PerfBaseline` — `from_samples(pipeline_name: &str, samples_ms: &[u64]) -> PerfBaseline`
   ```rust
   if samples_ms.is_empty() {
       return PerfBaseline { pipeline_name: pipeline_name.to_string(), p50_ms: 0, p95_ms: 0, p99_ms: 0 };
   }
   let mut sorted = samples_ms.to_vec();
   sorted.sort_unstable();
   let n = sorted.len();
   let p50_ms = sorted[n * 50 / 100];
   let p95_ms = sorted[n * 95 / 100];
   let p99_ms = sorted[n * 99 / 100];
   PerfBaseline { pipeline_name: pipeline_name.to_string(), p50_ms, p95_ms, p99_ms }
   ```

3. `PerfRegression` 構造体（`#[derive(Debug, Clone, PartialEq)]`）
   - `pipeline_name: String`, `baseline: PerfBaseline`, `current_ms: u64`, `regression_pct: f64`

### Step 2: `detect_perf_regression` / `format_regression_report` 関数を追加

```
fn detect_perf_regression(baseline: &PerfBaseline, current_ms: u64, threshold_pct: f64) -> Option<PerfRegression>
```

実装方針:
- `baseline.p95_ms == 0` → `None`（ゼロ除算ガード）
- `regression_pct = (current_ms as f64 - baseline.p95_ms as f64) / baseline.p95_ms as f64 * 100.0`
- `regression_pct > threshold_pct` → `Some(PerfRegression { ... })`
- それ以外 → `None`

```
fn format_regression_report(regression: &PerfRegression) -> String
```

出力形式:
```
PerfRegression: {pipeline_name}
Baseline p95: {p95_ms}ms
Current: {current_ms}ms
Regression: +{regression_pct:.2}%
```

### Step 3: `driver.rs` に `v83600_tests` を追加

`v83500_tests` の直後に追加する。

```rust
#[cfg(test)]
mod v83600_tests {
    use fav_core::test_framework::*;

    #[test]
    fn perf_regression_detected_above_threshold() { ... }

    #[test]
    fn perf_no_regression_within_threshold() { ... }
}
```

### Step 4: `CHANGELOG.md` 更新

先頭に v83.6.0 エントリを追加する。

### Step 5: `cargo test` で全テスト通過を確認

期待: 3899 tests pass、0 failures

### Step 6: CI チェック

- `cargo clippy --locked -- -D warnings` が pass することを確認
- `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認
- `./target/debug/fav fmt --check self/checker.fav` が pass することを確認
