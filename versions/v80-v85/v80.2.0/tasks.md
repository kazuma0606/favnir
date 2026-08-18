# Tasks: v80.2.0 — `GoldenDataset` / ゴールデンデータセット比較

**Status: COMPLETE** (2026-08-19)

> `lib.rs` への変更は不要（`pub mod test_framework;` は v80.1.0 で宣言済み）。
> MILESTONE.md / README.md および `site/content/docs/` への MDX 追加は v81.0.0 宣言バージョンでまとめて実施する。

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、3811 tests, 0 failures を確認する
- [x] `Cargo.toml` バージョンが `80.1.0` であることを確認する（v80.1.0 完了後に着手するため）
- [x] `fav/src/test_framework.rs` が存在し、v80.1.0 の型が定義済みであることを確認する

## T1: `fav/src/test_framework.rs` に追記

- [x] `GoldenDataset` 構造体（`#[derive(Debug)]`、`name: String`, `rows: Vec<Vec<String>>`）を追加する
- [x] `GoldenCompareResult` 構造体（`#[derive(Debug)]`、`matches: bool`, `diff_rows: Vec<usize>`）を追加する
- [x] `compare_golden(actual: &GoldenDataset, expected: &GoldenDataset) -> GoldenCompareResult` を実装する
  - 行単位で比較し、異なる行インデックスを `diff_rows` に収集する
  - 行数が異なる場合は超過分も diff として記録する
- [x] `format_golden_diff(result: &GoldenCompareResult) -> String` を実装する
  - 一致: `"OK: datasets match"`
  - 不一致: `"DIFF: N row(s) differ: [0, 2, ...]"`
- [x] `load_golden_dataset(path: &str) -> Result<GoldenDataset, String>` を実装する
  - `#[cfg(not(target_arch = "wasm32"))]` を付与する（`std::fs` を使用するため）
  - CSV ライン → カンマ分割 → `Vec<String>` の行として収集する
  - 空行はスキップする

## T2: `fav/src/driver.rs` に `mod v80200_tests` を追加

- [x] `mod v80100_tests { ... }` の直後に `#[cfg(test)] mod v80200_tests { ... }` を追加する
- [x] `use fav_core::test_framework::*;` でインポートする
- [x] `golden_dataset_compare_pass` テストを実装する（同一内容 → `matches = true`、空 `diff_rows`、`"OK: datasets match"`）
- [x] `golden_dataset_compare_fail_shows_diff` テストを実装する（行 1 が異なる → `matches = false`、`diff_rows = [1]`、`"DIFF: 1 row(s) differ: [1]"`）

## T3: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | tail -5` を実行し、**3813 tests, 0 failures** であることを確認した

## T4: CHANGELOG 更新

- [x] `CHANGELOG.md` の先頭に v80.2.0 エントリを追加した

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認した
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認した
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認した
