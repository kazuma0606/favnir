# v63.9.0 タスクリスト

Status: COMPLETE
Version: 63.9.0
Base tests: 3425
Target tests: 3427

---

## T0: 事前確認

- [x] `cargo test -j 8 -- --test-threads=8` でベース 3425 tests passed, 0 failed を確認
- [x] `driver.rs` に `cmd_run_with_cache` が存在することを確認（v63.2.0 実装済み）
- [x] `driver.rs` に `cmd_parallel_stats` が存在することを確認（v63.4.0 実装済み）
- [x] `driver.rs` に `cmd_opt_stats` が存在することを確認（v63.7.0 実装済み）
- [x] `driver.rs` に `v63800_tests` が存在することを確認（`v63900_tests` の挿入位置確認）
- [x] `driver.rs` に `v63900_tests` が存在しないことを確認（新規追加）

---

## T1: `driver.rs` — `v63900_tests` 追加

- [x] `v63800_tests` の直前に `v63900_tests` を挿入
  - `scale_e2e_incremental_par`（tempfile::tempdir + cmd_run_with_cache ×2 + cmd_parallel_stats）
  - `scale_dag_opt_dead_and_fused`（cmd_opt_stats: eliminated + fused 確認）
- [x] `cargo build` でエラーなし

---

## T2: ビルド・テスト

- [x] `cargo build` でコンパイルエラー 0（最終確認）
- [x] `cargo test --bin fav v63900_tests` で 2 件 PASS
  - `scale_e2e_incremental_par` PASS
  - `scale_dag_opt_dead_and_fused` PASS
- [x] `cargo test -j 8 -- --test-threads=8` で 3427 tests passed, 0 failed を確認

---

## T3: ドキュメント更新

- [x] `CHANGELOG.md` 先頭に v63.9.0 エントリを追加
- [x] `versions/roadmap/roadmap-v63.1-v64.0.md` v63.9.0 セクションに実績追記（完了条件テスト数 3424 → 3427 に修正 + base=3422 → base=3425 に修正）
- [x] `versions/current.md` の「進行中」を v63.9.0（3427 tests）に更新
- [x] tasks.md を COMPLETE に更新（本ファイル）
