# v83.4.0 タスクリスト

Status: COMPLETE

---

## T0: 事前確認

- [x] `cargo test` が 3,893 tests pass、0 failures であることを確認する（前提: v83.3.0 完了済み）

## T1: `test_framework.rs` に構造体・enum を追加

- [x] `ResourceUsage` 構造体を追加する（`#[derive(Debug, Clone, PartialEq)]`）
  - `cpu_seconds: f64`, `memory_mb: f64`, `io_mb: f64`
- [x] `ExecutionCost` 構造体を追加する（`#[derive(Debug, Clone, PartialEq)]`）
  - `resource: ResourceUsage`, `estimated_usd: f64`, `pipeline_name: String`
- [x] `CostBudget` 構造体を追加する（`#[derive(Debug, Clone, PartialEq)]`）
  - `max_usd_per_run: f64`, `max_cpu_seconds: f64`
- [x] `BudgetStatus` enum を追加する（`#[derive(Debug, Clone, PartialEq)]`）
  - `UnderBudget` / `NearLimit(f64)` / `OverBudget(f64)`

## T2: `evaluate_cost_budget` 関数を追加

- [x] `evaluate_cost_budget(cost: &ExecutionCost, budget: &CostBudget) -> BudgetStatus` を追加する
  - ゼロ除算ガード（`max_usd_per_run == 0.0` / `max_cpu_seconds == 0.0` → 使用率 0.0）
  - `max_pct > 100.0` → `OverBudget(max_pct - 100.0)`
  - `max_pct >= 80.0` → `NearLimit(max_pct)`
  - それ以外 → `UnderBudget`

## T3: `format_cost_report` 関数を追加

- [x] `format_cost_report(cost: &ExecutionCost, status: &BudgetStatus) -> String` を追加する
  - "Pipeline:"、"CPU:"、"Memory:"、"IO:"、"Cost:"、"Budget:" の各行を含む

## T4: `driver.rs` に `v83400_tests` を追加

- [x] `v83300_tests` の直後に `#[cfg(test)] mod v83400_tests` を追加する
  - `cost_budget_under_limit`（`format_cost_report` スモークテスト + `NearLimit` スポットアサーション含む）
  - `cost_budget_over_limit`（`OverBudget` 超過 % が正値であることを確認）

## T5: テスト通過確認

- [x] `cargo test` が 3,895 tests pass（+2）、0 failures であることを確認する

## T6: 最終確認（CI チェック）

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する

## code-reviewer 対応

- [x] [MED] `max_pct == 100.0`（予算ちょうど使い切り）は `NearLimit(100.0)` を返す旨を `BudgetStatus` の doc コメントに追記
- [x] [LOW] `cost_budget_over_limit` に `OverBudget(33.3%)` の具体値アサーション追加（`(over - 33.3).abs() < 0.5`）
