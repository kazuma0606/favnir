# v78.5.0 仕様書 — `fav explain plan` 可視化

Date: 2026-08-16

---

## Background

v78.4.0 で `CostEstimate` と各戦略のコスト推定関数を導入した。
v78.5.0 ではパイプラインの実行計画をステージ単位で保持し、テキスト形式で可視化する型と関数を追加する。
`ExecutionPlan` は v78.6.0 以降の `!Parallel` 統合や実行計画キャッシュ（v78.8.0）でも参照される基盤型。

---

## Goals

1. `PlanStage` 構造体を追加する（name, operation, cost, strategy）
2. `ExecutionPlan` 構造体を追加する（pipeline, stages, total_cost）
3. `format_execution_plan(plan: &ExecutionPlan) -> String` を追加する
4. テスト 2 件を追加する（3770 → 3772）

---

## API 仕様

### `PlanStage`

```rust
// CostEstimate(f64 含む) を内包するため Eq / Hash は付与しない
#[derive(Debug, Clone, PartialEq)]
pub struct PlanStage {
    pub name:      String,
    pub operation: String,
    pub cost:      CostEstimate,
    pub strategy:  Option<ExecutionStrategy>,
}
```

### `ExecutionPlan`

```rust
// CostEstimate(f64 含む) を内包するため Eq / Hash は付与しない
#[derive(Debug, Clone, PartialEq)]
pub struct ExecutionPlan {
    pub pipeline:   String,
    pub stages:     Vec<PlanStage>,
    pub total_cost: CostEstimate,
}
```

### `format_execution_plan`

```rust
pub fn format_execution_plan(plan: &ExecutionPlan) -> String
```

出力形式:

```
Execution Plan: {pipeline}
  Stage 1: {name}  [{operation}]  cost={cpu_units:.1} units  → {strategy}
  Stage 2: {name}  [{operation}]  cost={cpu_units:.1} units
  ───────────────────────────────────────────────────
  Total: {total_cost.cpu_units:.1} units  |  Memory peak: {total_cost.memory_mb:.0}MB
```

- ステージ番号は 1 始まり
- `strategy` が `Some(s)` の場合のみ戦略部分を追記
  - フォーマット: `"  → {variant}"` （半角スペース 2 個 + `→` + スペース 1 個 + variant 名）
  - 例: `"  → BroadcastJoin"`（`Strategy:` プレフィックスなし、variant 名のみ）
- `strategy` が `None` の場合は戦略部分を省略（空文字列として結合）
- セパレーター行は固定文字列 `  ───────────────────────────────────────────────────`

---

## テストシナリオ

### `explain_plan_format_output`

3 ステージの `ExecutionPlan` を構築し、`format_execution_plan` の出力が:
- `"Execution Plan: OrderPipeline"` を含む
- `"Stage 1:"` を含む
- `"Total:"` を含む

ことを検証する。

### `explain_plan_total_cost_summed`

`total_cost.cpu_units` が各ステージの `cost.cpu_units` の合計と一致することを検証する
（合計は呼び出し元が設定するため、ここではその値が正しく `format_execution_plan` に使われていることを確認）。

---

## Success Criteria

- `PlanStage` / `ExecutionPlan` が `Debug / Clone / PartialEq` を持つ（Eq/Hash なし）
- `format_execution_plan` がヘッダー・ステージ行・セパレーター・トータル行を含む文字列を返す
- テスト 2 件が pass する
- `cargo test` 全体が 3772 tests pass する

---

## Notes

- `changelog_has_v78_5_0` テストは x.0.0 宣言バージョンのみに追加する慣例につき、本バージョンでは追加しない。
- ロードマップのサンプル出力にある `→ cache hit expected` は概念図。実装では `strategy: None` のステージに戦略表示は行わない（`ExecutionStrategy` enum に対応 variant なし）。

---

## Files to Modify

- `fav/src/driver.rs` — 型・関数・テストモジュール追加
- `CHANGELOG.md` — v78.5.0 エントリ追加
- `fav/Cargo.toml` — version を `78.4.0` → `78.5.0` に更新
- `versions/current.md` — 進行中バージョン更新

---

## Error Codes

新規エラーコードなし。
