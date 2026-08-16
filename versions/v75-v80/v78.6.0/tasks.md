# v78.6.0 タスクリスト — `!Parallel` エフェクト統合

Date: 2026-08-16
Status: COMPLETE

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `78.5.0` であることを確認
- [x] `cargo test` が全 pass（3772 tests）であることを確認
- [x] `fav/tmp/hello.fav` が存在することを確認

---

## T1: driver.rs — 型・関数追加

- [x] `fav/src/driver.rs` の末尾に `// --- v78.6.0: !Parallel エフェクト統合 ---` コメントを追加する
- [x] `ParallelConfig` 構造体を追加する（`#[derive(Debug, Clone, PartialEq, Eq)]`、Hash なし）
  - `threads: usize`, `partition_count: usize`, `partition_key: String`
- [x] `PartitionPlan` 構造体を追加する（`#[derive(Debug, Clone, PartialEq, Eq)]`）
  - `partition_id: usize`, `rows_estimate: u64`, `thread_id: usize`
- [x] `plan_parallel_execution(total_rows: u64, config: &ParallelConfig) -> Vec<PartitionPlan>` を追加する
  - `partition_count == 0` → 空 Vec を返す
  - 基本行数 = `total_rows / partition_count`（整数除算）
  - 端数 = `total_rows % partition_count`（最後のパーティションに加算）
  - `thread_id = partition_id % config.threads`（threads == 0 は 0）
- [x] `format_parallel_plan(plans: &[PartitionPlan]) -> String` を追加する
  - 空スライス → `"No partitions."` を返す
  - ヘッダー行: `Parallel Plan: {n} partitions / {threads} threads`
  - 各パーティション行: `  Partition {id}: ~{rows} rows  thread={thread_id}`
  - フッター行: `  Total rows: {total}`
- [x] `cargo build` でコンパイルエラーがないことを確認する
- [x] `cargo test` で既存 3772 tests が pass することを確認する

---

## T2: CHANGELOG.md 更新（テスト追加より先）

- [x] `CHANGELOG.md` の先頭に v78.6.0 エントリを追加する（形式: `## [v78.6.0] — 2026-08-16 — !Parallel エフェクト統合`）
- [x] Added セクション（構造体 2 件・関数 2 件）と Tests セクション（2 件）を含める

---

## T3: driver.rs — テストモジュール追加

- [x] `fav/src/driver.rs` に `v786000_tests` モジュールを追加する（`use super::*` 必須）
- [x] `make_config(threads, partitions)` ヘルパー関数を実装する
- [x] `parallel_plan_creates_correct_partitions` テストを実装する
  - `plan_parallel_execution(1000, config(4, 8))` → len==8 を assert
  - `plans[0].thread_id==0`, `plans[4].thread_id==0`, `plans[3].thread_id==3` を assert
  - `format_parallel_plan` の出力が `"Parallel Plan:"` / `"Partition 0:"` / `"Total rows:"` を含むことを assert
- [x] `parallel_plan_distributes_evenly` テストを実装する
  - `plan_parallel_execution(100, config(2, 4))` → len==4 を assert
  - `plans.iter().map(rows_estimate).sum() == 100` を assert
  - `plans[0..3]` 各 rows_estimate == 25 を assert
  - 端数確認: `plan_parallel_execution(101, config(2, 4))` → sum==101、`plans[3].rows_estimate==26` を assert
- [x] `cargo test v786000` で 2 件が pass することを確認する

---

## T4: Cargo.toml バージョン更新

- [x] `fav/Cargo.toml` の `version` を `"78.5.0"` → `"78.6.0"` に変更する
- [x] driver.rs 内の `78.5.0` バージョン文字列アサーションを `78.6.0` に一括更新（`replace_all: true`）
- [x] **replace_all 後に** `grep -c "78.5.0" fav/src/driver.rs` を実行し **出力が 1** であることを確認する
  - 残るのは `// --- v78.5.0: fav explain plan 可視化 ---` の 1 件のみ

---

## T5: versions/current.md 更新

- [x] `## 進行中バージョン` 欄を `**v78.6.0**（!Parallel エフェクト統合）` に更新する
- [x] `## 次に切る版` 欄を `**v78.7.0**（Stream / Batch 統合実行モード）` に更新する

---

## T6: 最終確認

- [x] `cargo test` が全 pass であることを確認する（3774 tests）
- [x] `cargo test v786000` で 2 件が pass することを確認する
- [x] `fav/Cargo.toml` のバージョンが `78.6.0` であることを確認する
- [x] `CHANGELOG.md` の先頭が `[v78.6.0]` であることを確認する
- [x] `versions/current.md` の「進行中バージョン」が v78.6.0 であることを確認する

---

## 完了チェックリスト

- [x] 全タスク（T0〜T6）が完了している
- [x] `parallel_plan_creates_correct_partitions` が pass
- [x] `parallel_plan_distributes_evenly` が pass
- [x] テスト総数: 3775（+3、code-reviewer 対応で境界値テスト parallel_plan_boundary_cases +1）
- [x] site/ MDX 追加: 本バージョンでは対象外（型基盤のみ）
- [x] `changelog_has_v78_6_0` テストの追加: 対象外（x.0.0 宣言バージョンのみに追加する慣例）
