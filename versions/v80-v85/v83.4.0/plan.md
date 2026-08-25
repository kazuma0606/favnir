# v83.4.0 実装計画 — コスト追跡（`ExecutionCost` / `CostBudget`）

## 依存関係

新規型・関数のみ。既存コードへの変更なし。

## 実装ステップ

### Step 1: `test_framework.rs` に構造体・enum を追加

v83.3.0 追加ブロック（`format_slo_status` 末尾）の後に追加する。

1. `ResourceUsage` 構造体（`#[derive(Debug, Clone, PartialEq)]`）
   - `cpu_seconds: f64`, `memory_mb: f64`, `io_mb: f64`

2. `ExecutionCost` 構造体（`#[derive(Debug, Clone, PartialEq)]`）
   - `resource: ResourceUsage`, `estimated_usd: f64`, `pipeline_name: String`

3. `CostBudget` 構造体（`#[derive(Debug, Clone, PartialEq)]`）
   - `max_usd_per_run: f64`, `max_cpu_seconds: f64`

4. `BudgetStatus` enum（`#[derive(Debug, Clone, PartialEq)]`）
   - `UnderBudget`
   - `NearLimit(f64)` — 最大使用率 %（80〜100 の範囲）
   - `OverBudget(f64)` — 超過分 %（使用率 - 100.0）

### Step 2: `evaluate_cost_budget` 関数を追加

```
fn evaluate_cost_budget(cost: &ExecutionCost, budget: &CostBudget) -> BudgetStatus
```

実装方針:
- ゼロ除算ガード: `max_usd_per_run == 0.0` → `usd_pct = 0.0`、`max_cpu_seconds == 0.0` → `cpu_pct = 0.0`
- `usd_pct = cost.estimated_usd / budget.max_usd_per_run * 100.0`
- `cpu_pct = cost.resource.cpu_seconds / budget.max_cpu_seconds * 100.0`
- `max_pct = usd_pct.max(cpu_pct)`
- `max_pct > 100.0` → `OverBudget(max_pct - 100.0)`
- `max_pct >= 80.0` → `NearLimit(max_pct)`
- それ以外 → `UnderBudget`

### Step 3: `format_cost_report` 関数を追加

```
fn format_cost_report(cost: &ExecutionCost, status: &BudgetStatus) -> String
```

出力形式:
```
Pipeline: {pipeline_name}
CPU: {cpu_seconds:.1}s
Memory: {memory_mb:.1}MB
IO: {io_mb:.1}MB
Cost: ${estimated_usd:.2}
Budget: UnderBudget  |  NearLimit (90.00%)  |  OverBudget (+20.00%)
```

### Step 4: `driver.rs` に `v83400_tests` を追加

`v83300_tests` の直後に追加する。

```rust
#[cfg(test)]
mod v83400_tests {
    use fav_core::test_framework::*;

    #[test]
    fn cost_budget_under_limit() { ... }

    #[test]
    fn cost_budget_over_limit() { ... }
}
```

### Step 5: `cargo test` で全テスト通過を確認

期待: 3895 tests pass、0 failures

### Step 6: CI チェック

- `cargo clippy --locked -- -D warnings` が pass することを確認
- `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認
- `./target/debug/fav fmt --check self/checker.fav` が pass することを確認
