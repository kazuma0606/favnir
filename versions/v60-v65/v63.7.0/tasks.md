# v63.7.0 タスクリスト

Status: COMPLETE
Version: 63.7.0
Base tests: 3421
Target tests: 3423

---

## T0: 事前確認

- [x] `cargo test -j 8 -- --test-threads=8` でベース 3421 tests passed, 0 failed を確認
- [x] `driver.rs` に `v63600_tests` が存在することを確認（`v63700_tests` の挿入位置確認）
- [x] `driver.rs` に `cmd_opt_stats` が存在しないことを確認（新規追加）
- [x] `driver.rs` に `cmd_parallel_stats` が存在することを確認（`cmd_opt_stats` の挿入位置参照）
- [x] `ast.rs` の `PipelineDef.steps[*]` が `seq_name: String` フィールドを持つことを確認
- [x] `ast.rs` の `TrfDef` に `effects` フィールドが存在しないことを確認（v35.6.0 で削除済み。pure 判定は body AST 解析で行う）
- [x] `parser.rs` の `parse_trf_def` が `TrfDef.body` を `parse_block()` で直接構築することを確認（`Lambda` ラップなし — `opt_block_has_effect_call` が正しく動作する前提）

**非スコープ注意**: `compiler.rs` への DAG パス統合・`petgraph` 活用・`fav run --opt-stats` CLI フラグは
本バージョンの非スコープ（後送り）。実装しないこと。spec.md §非スコープ を参照。

---

## T1: `driver.rs` — ヘルパー関数 + `cmd_opt_stats` 追加

- [x] `cmd_parallel_stats` の直後に以下を追加:
  - `fn opt_is_pure_stage(td: &TrfDef) -> bool`
  - `fn opt_block_has_effect_call(block: &Block) -> bool`
  - `fn opt_expr_has_effect_call(expr: &Expr) -> bool`（`Io` / `Http` / `Db` / `Kafka` / `S3` / `Sqs` / `Slack` / `Email` / `Llm` / `Snowflake` / `Postgres` の FieldAccess を検出）
  - `pub fn cmd_opt_stats(src: &str) -> String`
- [x] `cargo build` でエラーなし

---

## T2: `driver.rs` — `v63700_tests` 追加

- [x] `v63600_tests` の直前（ファイル先頭方向）に以下を挿入:
  ```rust
  // -- v63700_tests (v63.7.0) -- パイプライン DAG 最適化 --
  #[cfg(test)]
  mod v63700_tests {
      fn optimizer_dead_stage_eliminated() { ... }  // Unused が eliminated
      fn optimizer_pure_stages_fused() { ... }       // Normalize -> Trim が fused
  }
  ```
- [x] `cargo build` でエラーなし

---

## T3: ビルド・テスト

- [x] `cargo build` でコンパイルエラー 0（全ステップ完了後の最終確認）
- [x] `cargo test --bin fav v63700_tests` で 2 件 PASS
  - `optimizer_dead_stage_eliminated` PASS
  - `optimizer_pure_stages_fused` PASS
- [x] `cargo test -j 8 -- --test-threads=8` で 3423 tests passed, 0 failed を確認

---

## T4: ドキュメント更新

- [x] `CHANGELOG.md` 先頭に v63.7.0 エントリを追加
- [x] `versions/roadmap/roadmap-v63.1-v64.0.md` v63.7.0 セクションに実績追記（テスト数推移テーブルの v63.6.0 行・v63.7.0 行も実績値 3421/3423 に更新）
- [x] `versions/current.md` の「進行中」を v63.7.0（3423 tests）に更新
- [x] tasks.md を COMPLETE に更新（本ファイル）
- ~~`site/` MDX 追加~~ — 非スコープ
- ~~`compiler.rs` DAG パス統合~~ — 非スコープ

---

## 完了サマリー

- Status: COMPLETE
- Tests: 3423 passed, 0 failed
- 主要実装: `driver.rs`（`opt_is_pure_stage` / `opt_block_has_effect_call` / `opt_expr_has_effect_call` + `cmd_opt_stats` + `v63700_tests`）
- 完了日: 2026-08-02
