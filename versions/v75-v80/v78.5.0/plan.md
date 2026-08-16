# v78.5.0 実装計画 — `fav explain plan` 可視化

---

## Step 1: 事前確認

- `fav/Cargo.toml` のバージョンが `78.4.0` であることを確認
- `cargo test` が全 pass（3770 tests）であることを確認
- `fav/tmp/hello.fav` が存在することを確認

---

## Step 2: driver.rs — 型・関数追加

`fav/src/driver.rs` の末尾（v78.4.0 テストモジュールの直後）に追加する。

```rust
// --- v78.5.0: fav explain plan 可視化 ---

// CostEstimate(f64 含む) を内包するため Eq / Hash は付与しない
#[derive(Debug, Clone, PartialEq)]
pub struct PlanStage {
    pub name:      String,
    pub operation: String,
    pub cost:      CostEstimate,
    pub strategy:  Option<ExecutionStrategy>,
}

// CostEstimate(f64 含む) を内包するため Eq / Hash は付与しない
#[derive(Debug, Clone, PartialEq)]
pub struct ExecutionPlan {
    pub pipeline:   String,
    pub stages:     Vec<PlanStage>,
    pub total_cost: CostEstimate,
}

pub fn format_execution_plan(plan: &ExecutionPlan) -> String {
    let mut lines = Vec::new();
    lines.push(format!("Execution Plan: {}", plan.pipeline));
    for (i, stage) in plan.stages.iter().enumerate() {
        let strategy_part = match &stage.strategy {
            Some(s) => format!("  → {}", strategy_label(s)),
            None    => String::new(),
        };
        lines.push(format!(
            "  Stage {}: {}  [{}]  cost={:.1} units{}",
            i + 1, stage.name, stage.operation, stage.cost.cpu_units, strategy_part
        ));
    }
    lines.push("  ───────────────────────────────────────────────────".to_string());
    lines.push(format!(
        "  Total: {:.1} units  |  Memory peak: {:.0}MB",
        plan.total_cost.cpu_units, plan.total_cost.memory_mb
    ));
    lines.join("\n")
}

/// `format_strategy_selected` からラベル（variant 名部分）を抽出する。
/// 例: "Strategy: BroadcastJoin (small table detected)" → "BroadcastJoin"
fn strategy_label(strategy: &ExecutionStrategy) -> &'static str {
    match strategy {
        ExecutionStrategy::BroadcastJoin => "BroadcastJoin",
        ExecutionStrategy::HashJoin      => "HashJoin",
        ExecutionStrategy::SortMergeJoin => "SortMergeJoin",
        ExecutionStrategy::Auto          => "Auto",
    }
}
```

`cargo build` でコンパイルエラーがないことを確認する。

---

## Step 3: CHANGELOG.md 更新（テスト追加より先）

`CHANGELOG.md` の先頭（`[v78.4.0]` エントリの前）に v78.5.0 エントリを追加する。

---

## Step 4: driver.rs — テストモジュール追加

```rust
#[cfg(test)]
mod v785000_tests {
    use super::*;

    fn make_test_plan() -> ExecutionPlan {
        ExecutionPlan {
            pipeline: "OrderPipeline".to_string(),
            stages: vec![
                PlanStage {
                    name:      "LoadOrders".to_string(),
                    operation: "IO".to_string(),
                    cost:      CostEstimate { cpu_units: 1.2, memory_mb: 50.0, io_ops: 1000 },
                    strategy:  None,
                },
                PlanStage {
                    name:      "JoinCustomers".to_string(),
                    operation: "Adaptive".to_string(),
                    cost:      CostEstimate { cpu_units: 2.1, memory_mb: 128.0, io_ops: 45000 },
                    strategy:  Some(ExecutionStrategy::BroadcastJoin),
                },
                PlanStage {
                    name:      "AggregateRegion".to_string(),
                    operation: "Cached".to_string(),
                    cost:      CostEstimate { cpu_units: 0.3, memory_mb: 10.0, io_ops: 100 },
                    strategy:  None,
                },
            ],
            total_cost: CostEstimate { cpu_units: 3.6, memory_mb: 128.0, io_ops: 46100 },
        }
    }

    #[test]
    fn explain_plan_format_output() {
        let plan = make_test_plan();
        let output = format_execution_plan(&plan);
        assert!(output.contains("Execution Plan: OrderPipeline"));
        assert!(output.contains("Stage 1:"));
        assert!(output.contains("Stage 2:"));
        assert!(output.contains("Stage 3:"));
        assert!(output.contains("BroadcastJoin"));
        assert!(output.contains("Total:"));
    }

    #[test]
    fn explain_plan_total_cost_summed() {
        let plan = make_test_plan();
        // total_cost.cpu_units は各ステージの合計（1.2 + 2.1 + 0.3 = 3.6）
        let stage_sum: f64 = plan.stages.iter().map(|s| s.cost.cpu_units).sum();
        assert!((stage_sum - plan.total_cost.cpu_units).abs() < 0.01,
            "total {} != stage sum {}", plan.total_cost.cpu_units, stage_sum);
        // format_execution_plan が total_cost を正しく使用していることを確認
        let output = format_execution_plan(&plan);
        assert!(output.contains("3.6"));
        assert!(output.contains("128MB"));
    }
}
```

`cargo test v785000` で 2 件 pass を確認する。

---

## Step 5: Cargo.toml バージョン更新

- `version` を `"78.4.0"` → `"78.5.0"` に変更
- driver.rs 内のバージョン文字列アサーションを `78.4.0` → `78.5.0` に一括更新（`replace_all: true`）
- **replace_all 後に** `grep -c "78.4.0" fav/src/driver.rs` を実行し **出力が 1** であることを確認する
  - 残るのは `// --- v78.4.0: コスト推定モデル ---` セクションコメントの 1 件のみ

---

## Step 6: versions/current.md 更新

- `## 進行中バージョン` 欄を `**v78.5.0**（fav explain plan 可視化）` に更新
- `## 次に切る版` 欄を `**v78.6.0**（!Parallel エフェクト統合）` に更新

---

## Step 7: 最終確認

- `cargo test` が全 pass（3772 tests）であることを確認
- `cargo test v785000` で 2 件 pass を確認
- `fav/Cargo.toml` のバージョンが `78.5.0` であることを確認
- `CHANGELOG.md` の先頭が `[v78.5.0]` であることを確認
