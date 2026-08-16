# v78.5.0 タスクリスト — `fav explain plan` 可視化

Date: 2026-08-16
Status: COMPLETE

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `78.4.0` であることを確認
- [x] `cargo test` が全 pass（3770 tests）であることを確認
- [x] `fav/tmp/hello.fav` が存在することを確認

---

## T1: driver.rs — 型・関数追加

- [x] `fav/src/driver.rs` の末尾に `// --- v78.5.0: fav explain plan 可視化 ---` コメントを追加する
- [x] `PlanStage` 構造体を追加する（`#[derive(Debug, Clone, PartialEq)]`、Eq/Hash なし）
  - `name: String`, `operation: String`, `cost: CostEstimate`, `strategy: Option<ExecutionStrategy>`
- [x] `ExecutionPlan` 構造体を追加する（`#[derive(Debug, Clone, PartialEq)]`、Eq/Hash なし）
  - `pipeline: String`, `stages: Vec<PlanStage>`, `total_cost: CostEstimate`
- [x] `format_execution_plan(plan: &ExecutionPlan) -> String` を追加する
  - ヘッダー行 `Execution Plan: {pipeline}`
  - 各ステージ行: `Stage N: {name}  [{operation}]  cost={cpu:.1} units  → {strategy variant}`（strategy は Some の場合のみ）
  - セパレーター行 `  ───────────────────────────────────────────────────`
  - トータル行 `Total: {cpu:.1} units  |  Memory peak: {mem:.0}MB`
- [x] `strategy_label(strategy: &ExecutionStrategy) -> &'static str` ヘルパーを追加する（非 pub）
- [x] `cargo build` でコンパイルエラーがないことを確認する
- [x] `cargo test` で既存 3770 tests が pass することを確認する

---

## T2: CHANGELOG.md 更新（テスト追加より先）

- [x] `CHANGELOG.md` の先頭に v78.5.0 エントリを追加する（形式: `## [v78.5.0] — 2026-08-16 — fav explain plan 可視化`）
- [x] Added セクション（構造体 2 件・関数 1 件）と Tests セクション（2 件）を含める

---

## T3: driver.rs — テストモジュール追加

- [x] `fav/src/driver.rs` に `v785000_tests` モジュールを追加する（`use super::*` 必須）
- [x] `make_test_plan()` ヘルパー関数を実装する（3 ステージの OrderPipeline、total_cost.cpu_units = 3.6）
- [x] `explain_plan_format_output` テストを実装する
  - `format_execution_plan` の出力が以下を含むことを assert:
    - `"Execution Plan: OrderPipeline"`
    - `"Stage 1:"` / `"Stage 2:"` / `"Stage 3:"`
    - `"BroadcastJoin"`（strategy=Some の Stage 2 の戦略表示）
    - `"Total:"`
- [x] `explain_plan_total_cost_summed` テストを実装する
  - `stage_sum = 1.2 + 2.1 + 0.3 = 3.6` と `plan.total_cost.cpu_units = 3.6` が一致することを assert
  - `format_execution_plan` の出力が `"3.6"` と `"128MB"` を含むことを assert
- [x] `cargo test v785000` で 2 件が pass することを確認する

---

## T4: Cargo.toml バージョン更新

- [x] `fav/Cargo.toml` の `version` を `"78.4.0"` → `"78.5.0"` に変更する
- [x] driver.rs 内の `78.4.0` バージョン文字列アサーションを `78.5.0` に一括更新（`replace_all: true`）
- [x] **replace_all 後に** `grep -c "78.4.0" fav/src/driver.rs` を実行し **出力が 1** であることを確認する
  - 残るのは `// --- v78.4.0: コスト推定モデル ---` の 1 件のみ

---

## T5: versions/current.md 更新

- [x] `## 進行中バージョン` 欄を `**v78.5.0**（fav explain plan 可視化）` に更新する
- [x] `## 次に切る版` 欄を `**v78.6.0**（!Parallel エフェクト統合）` に更新する

---

## T6: 最終確認

- [x] `cargo test` が全 pass であることを確認する（3772 tests）
- [x] `cargo test v785000` で 2 件が pass することを確認する
- [x] `fav/Cargo.toml` のバージョンが `78.5.0` であることを確認する
- [x] `CHANGELOG.md` の先頭が `[v78.5.0]` であることを確認する
- [x] `versions/current.md` の「進行中バージョン」が v78.5.0 であることを確認する

---

## 完了チェックリスト

- [x] 全タスク（T0〜T6）が完了している
- [x] `explain_plan_format_output` が pass
- [x] `explain_plan_total_cost_summed` が pass
- [x] テスト総数: 3772（+2）
- [x] site/ MDX 追加: 本バージョンでは対象外（型基盤のみ）
- [x] `changelog_has_v78_5_0` テストの追加: 対象外（x.0.0 宣言バージョンのみに追加する慣例）
