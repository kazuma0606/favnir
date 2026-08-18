# Tasks: v80.1.0 — `TestCase` / `TestSuite` 型基盤

**Status: COMPLETE** (2026-08-19)

> **注**: `cmd_test` はすでに実装済みのため本バージョンでは変更しない。
> MILESTONE.md / README.md の更新は v81.0.0 宣言バージョンで実施する。

## T0: 着手前チェックリスト

- [x] `versions/current.md` の現行マスターロードマップが `roadmap-v80.1-v85.0.md` を指していることを確認する
- [x] `versions/v80-v85/` ディレクトリが存在することを確認する
- [x] `cargo test` を実行し、0 failures を確認する（ベース値は 3809 だが、後から参照する場合は実際のテスト数を正とする）
- [x] `Cargo.toml` バージョンが `80.0.0` であることを確認する

## T1: `fav/src/test_framework.rs` 新規作成

- [x] `TestStatus` enum（`Pass` / `Fail` / `Skip`）を定義する
- [x] `TestCase` 構造体（`name: String`, `status: TestStatus`, `message: Option<String>`）を定義する
- [x] `TestSuite` 構造体（`name: String`, `cases: Vec<TestCase>`）を定義する
- [x] `TestSuiteResult` 構造体（`passed: usize`, `failed: usize`, `skipped: usize`）を定義する
- [x] `run_test_suite(suite: &TestSuite) -> TestSuiteResult` を実装する（Pass/Fail/Skip をカウント）
- [x] `format_test_suite_result(result: &TestSuiteResult) -> String` を実装する（`"N passed, M failed, K skipped"` 形式）

## T2: `fav/src/lib.rs` に `pub mod test_framework;` を追加

- [x] 既存 `pub mod` 宣言群の末尾に `pub mod test_framework;` を追記する

## T3: `fav/src/driver.rs` に `mod v80100_tests` を追加

- [x] `mod v80000_tests { ... }` の直後に `mod v80100_tests { ... }` を追加する
- [x] `use fav_core::test_framework::*;` でインポートする（driver.rs は fav バイナリクレート、test_framework は fav_core ライブラリクレートに属するため）
- [x] `test_suite_type_exists` テストを実装する（`TestSuite` / `TestCase` / `TestStatus` の構築とフィールド検証）
- [x] `test_case_run_formats_result` テストを実装する（`run_test_suite` + `format_test_suite_result` の結果検証）

## T4: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | tail -5` を実行し、**3811 tests, 0 failures** であることを確認した

## T5: CHANGELOG 更新

- [x] `CHANGELOG.md` の先頭に v80.1.0 エントリを追加した

## T-last: CI 事前確認

`cargo test` 完了後（`target/debug/fav` バイナリが存在することを前提）に実行する。

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する

## 実装メモ

- `driver.rs` は `fav` バイナリクレートに属するため `crate::test_framework` は解決できない。`fav_core::test_framework::*` を使用する必要がある。
- spec-reviewer 2回のレビューを実施。最終的に 0 件指摘。
