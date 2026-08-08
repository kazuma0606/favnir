# v64.2.0 タスクリスト

Status: COMPLETE
Version: 64.2.0
Base tests: 3433
Target tests: 3435

---

## T0: 事前確認

- [x] `cargo test -j 8 -- --test-threads=8` でベース 3433 tests passed, 0 failed を確認
- [x] `driver.rs` に `cmd_bench_suite` が存在することを確認（`cmd_bench_compare` の挿入位置参照）
- [x] `driver.rs` に `cmd_bench_compare` が存在しないことを確認（新規追加）
- [x] `driver.rs` に `v64100_tests` が存在することを確認（`v64200_tests` の挿入位置）
- [x] `driver.rs` に `v64200_tests` が存在しないことを確認（新規追加）
- [x] `toml.rs` に `BenchTomlConfig` が存在しないことを確認（新規追加）
- [x] `FavToml` に `bench` フィールドが存在しないことを確認（新規追加）
- [x] `toml.rs` に `parse_fav_toml_pub` が存在することを確認（テストから呼ぶため）

---

## T1: `toml.rs` — `BenchTomlConfig` + `FavToml` 更新

- [x] `BackpressureConfig` 定義の直後に `BenchTomlConfig { regression_threshold_pct: Option<u32> }` を追加
- [x] `FavToml` の `backpressure` フィールドの直後に `pub bench: Option<BenchTomlConfig>` を追加
- [x] `parse_fav_toml` に `let mut bench_cfg: Option<BenchTomlConfig> = None;` を追加
- [x] `[backpressure]` セクション検出の直後に `[bench]` セクション検出を追加
- [x] `"backpressure"` アームの直後に `"bench"` アーム（`regression_threshold_pct` パース）を追加
- [x] `FavToml` 構造体リテラルに `bench: bench_cfg` を追加
- [x] `cargo build` でエラーなし

---

## T2: `driver.rs` — `cmd_bench_compare` 追加

- [x] `cmd_bench_suite` の直後に `fn parse_bench_mean_ms(json: &str, mode: &str) -> Option<f64>` を追加
- [x] `parse_bench_mean_ms` の直後に `pub fn cmd_bench_compare(ref_a: &str, ref_b: &str) -> String` を追加
  - base_ms / curr_ms が parse 失敗時は `"bench_compare: could not parse ..."` を返す
  - `pct > 10.0` 時: `"Regression detected: AOT +X.X% slower ..."` を返す
  - それ以外: `"No regression detected. AOT X.X% ..."` を返す
- [x] `cargo build` でエラーなし

---

## T3: `driver.rs` — `v64200_tests` 追加

- [x] `v64100_tests` の直前に `v64200_tests` を挿入
  - `bench_compare_detects_regression`
  - `bench_toml_threshold`
- [x] `cargo build` でエラーなし

---

## T4: ビルド・テスト

- [x] `cargo test --bin fav v64200_tests` で 2 件 PASS
  - `bench_compare_detects_regression` PASS（リグレッション検出・No regression・改善時の各ケース確認）
  - `bench_toml_threshold` PASS（TOML パース確認のみ、driver との結合は非スコープ）
- [x] `cargo test -j 8 -- --test-threads=8` で 3435 tests passed, 0 failed を確認

---

## T5: ドキュメント更新

- [x] `CHANGELOG.md` 先頭に v64.2.0 エントリを追加
- [x] `versions/roadmap/roadmap-v64.1-v65.0.md` v64.2.0 セクションに実績追記
- [x] `versions/current.md` の「進行中」を v64.2.0（3435 tests）に更新
- [x] `MILESTONE.md` は v65.0 で更新（本バージョンでは不要）
- [x] tasks.md を COMPLETE に更新（本ファイル）
