# v63.8.0 タスクリスト

Status: COMPLETE
Version: 63.8.0
Base tests: 3423
Target tests: 3425

---

## T0: 事前確認

- [x] `cargo test -j 8 -- --test-threads=8` でベース 3423 tests passed, 0 failed を確認
- [x] `driver.rs` に `cmd_bench_aot_vm` が存在することを確認（`cmd_bench_suite` 内部で再利用）
- [x] `driver.rs` に `cmd_bench_suite` が存在しないことを確認（新規追加）
- [x] `driver.rs` に `v63700_tests` が存在することを確認（`v63800_tests` の挿入位置確認）
- [x] `driver.rs` で `cmd_bench_aot_vm` 直後の関数名を確認し、`cmd_bench_suite` の挿入点を特定する
- [x] spec 記載のベンチ用 Favnir ソース（`public stage LoadCsv: Int -> Int = |x| { x }` 等）が現行パーサーでパースエラーにならないことを `cmd_bench_aot_vm` 呼び出し結果で確認する（"parse error" を含まないこと）

---

## T1: `driver.rs` — `cmd_bench_suite` 追加

- [x] `cmd_bench_aot_vm` の直後に `cmd_bench_suite` を追加
- [x] `cargo build` でエラーなし

---

## T2: `driver.rs` — `v63800_tests` 追加

- [x] `v63700_tests` の直前に `v63800_tests` を挿入
  - `bench_suite_etl_standard`
  - `bench_regression_check`
- [x] `cargo build` でエラーなし

---

## T3: ビルド・テスト

- [x] `cargo build` でコンパイルエラー 0
- [x] `cargo test --bin fav v63800_tests` で 2 件 PASS
  - `bench_suite_etl_standard` PASS
  - `bench_regression_check` PASS
- [x] `cargo test -j 8 -- --test-threads=8` で 3425 tests passed, 0 failed を確認

---

## T4: ドキュメント更新

- [x] `CHANGELOG.md` 先頭に v63.8.0 エントリを追加
- [x] `versions/roadmap/roadmap-v63.1-v64.0.md` v63.8.0 セクションに実績追記（完了条件のテスト数が 3422 のままなら 3425 に修正する）
- [x] `versions/current.md` の「進行中」を v63.8.0（3425 tests）に更新
- [x] tasks.md を COMPLETE に更新（本ファイル）
