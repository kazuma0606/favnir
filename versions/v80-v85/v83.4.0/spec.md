# v83.4.0 仕様書 — コスト追跡（`ExecutionCost` / `CostBudget`）

## Background

v83.3.0 で SLO 型が導入された。次のステップとして、
パイプライン実行のコスト（CPU 秒・メモリ・IO・クラウド課金見込み）を型で追跡し、
予算（`CostBudget`）と照合して `BudgetStatus` を返す仕組みを整備する。

本バージョンは Observability 2.0 スプリント（v83.1〜v84.0）の第 4 段階。

ロードマップ参照: `versions/roadmap/roadmap-v83.1-v84.0.md` — v83.4.0 セクション

## Goals

1. `ResourceUsage` 構造体を追加する（CPU・メモリ・IO の使用量）
2. `ExecutionCost` 構造体を追加する（リソース使用量 + 課金見積 + パイプライン名）
3. `CostBudget` 構造体を追加する（実行ごとの上限）
4. `BudgetStatus` enum を追加する（`UnderBudget` / `NearLimit(f64)` / `OverBudget(f64)`）
5. `evaluate_cost_budget(cost: &ExecutionCost, budget: &CostBudget) -> BudgetStatus` を追加する
6. `format_cost_report(cost: &ExecutionCost, status: &BudgetStatus) -> String` を追加する

## 型定義・API

```rust
/// パイプライン実行のリソース使用量。
#[derive(Debug, Clone, PartialEq)]
pub struct ResourceUsage {
    pub cpu_seconds: f64,
    pub memory_mb: f64,
    pub io_mb: f64,
}

/// 実行コストの集計。
#[derive(Debug, Clone, PartialEq)]
pub struct ExecutionCost {
    pub resource: ResourceUsage,
    pub estimated_usd: f64,
    pub pipeline_name: String,
}

/// 実行コストの予算上限。
#[derive(Debug, Clone, PartialEq)]
pub struct CostBudget {
    pub max_usd_per_run: f64,
    pub max_cpu_seconds: f64,
}

/// 予算チェック結果。
///
/// - `UnderBudget` — USD・CPU ともに予算の 80% 未満
/// - `NearLimit(f64)` — いずれかが 80%〜100% の範囲（f64 は最大使用率 %）
/// - `OverBudget(f64)` — いずれかが 100% 超（f64 は最大超過率 % ）
///
/// **80% 閾値の根拠**: Prometheus / Grafana 等の一般的な監視ツールでの
/// "warn before breach" 慣習に準拠。残り 20% で早期警告を発し、
/// 次の実行までに予算調整できるようにする。
#[derive(Debug, Clone, PartialEq)]
pub enum BudgetStatus {
    UnderBudget,
    NearLimit(f64),
    OverBudget(f64),
}

/// 予算チェックのロジック:
/// 1. USD 使用率: `estimated_usd / max_usd_per_run * 100.0`
/// 2. CPU 使用率: `cpu_seconds / max_cpu_seconds * 100.0`
/// 3. 最大使用率 = max(USD 使用率, CPU 使用率)
/// 4. 最大使用率 > 100.0 → `OverBudget(最大使用率 - 100.0)` (超過分 %)
/// 5. 最大使用率 >= 80.0 → `NearLimit(最大使用率)`
/// 6. それ以外 → `UnderBudget`
///
/// `max_usd_per_run == 0.0` / `max_cpu_seconds == 0.0` はゼロ除算を避けるため
/// それぞれ使用率 0.0 として扱う（上限なしとみなす）。
pub fn evaluate_cost_budget(cost: &ExecutionCost, budget: &CostBudget) -> BudgetStatus

/// コストレポートのテキストを返す。
///
/// 例:
/// "Pipeline: etl_main\nCPU: 30.0s\nMemory: 512.0MB\nIO: 100.0MB\nCost: $1.50\nBudget: UnderBudget"
/// "Pipeline: etl_main\nCPU: 30.0s\nMemory: 512.0MB\nIO: 100.0MB\nCost: $1.50\nBudget: NearLimit (90.00%)"
/// "Pipeline: etl_main\nCPU: 30.0s\nMemory: 512.0MB\nIO: 100.0MB\nCost: $1.50\nBudget: OverBudget (+20.00%)"
pub fn format_cost_report(cost: &ExecutionCost, status: &BudgetStatus) -> String
```

## テスト（v83.4.0 で追加）

実際のテスト数ベース（※ drift 補正後）: **3893 + 2 = 3895**

（ロードマップ記載値 3881 + 2 = 3883 は旧バージョン到達時点のドリフト。
 実際の v83.3.0 完了テスト数は 3893。）

### `cost_budget_under_limit`

```rust
let cost = ExecutionCost {
    resource: ResourceUsage { cpu_seconds: 10.0, memory_mb: 256.0, io_mb: 50.0 },
    estimated_usd: 0.50,
    pipeline_name: "etl_main".into(),
};
let budget = CostBudget { max_usd_per_run: 2.00, max_cpu_seconds: 60.0 };
// USD: 25%, CPU: 16.7% → UnderBudget
let status = evaluate_cost_budget(&cost, &budget);
assert_eq!(status, BudgetStatus::UnderBudget, "should be under budget");
// format_cost_report のスモークテスト
let report = format_cost_report(&cost, &status);
assert!(report.contains("Pipeline:"), "report should contain 'Pipeline:'");
assert!(report.contains("Cost:"), "report should contain 'Cost:'");
assert!(report.contains("Budget:"), "report should contain 'Budget:'");
```

### `cost_budget_over_limit`

```rust
let cost = ExecutionCost {
    resource: ResourceUsage { cpu_seconds: 80.0, memory_mb: 1024.0, io_mb: 200.0 },
    estimated_usd: 2.40,
    pipeline_name: "etl_main".into(),
};
let budget = CostBudget { max_usd_per_run: 2.00, max_cpu_seconds: 60.0 };
// USD: 120%, CPU: 133.3% → OverBudget
let status = evaluate_cost_budget(&cost, &budget);
assert!(matches!(status, BudgetStatus::OverBudget(_)), "should be over budget");
```

## Success Criteria

- `cargo test` が 3895 tests pass（+2）、0 failures
- `UnderBudget` は `cost_budget_under_limit` で、`OverBudget` は `cost_budget_over_limit` で検証する
- `NearLimit` は `cost_budget_under_limit` 内でのスポットアサーション（境界付近の入力）により検証する
- `format_cost_report` が "Pipeline:"、"Cost:"、"Budget:" を含む文字列を返す

## Files to Modify

- `fav/src/test_framework.rs` — 型定義・関数追加
- `fav/src/driver.rs` — `v83400_tests` モジュール追加
