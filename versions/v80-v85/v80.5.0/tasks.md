# Tasks: v80.5.0 — ステージ単体テスト（`StageTestCase`）

> `lib.rs` への変更は不要（`pub mod test_framework;` は v80.1.0 で宣言済み）。
> MILESTONE.md / README.md / `site/content/docs/` の更新は v81.0.0 宣言バージョンで実施する。
> ロードマップのテスト数（3817）と実際のベース（3821）が 4 件ずれているが、
> v80.2.0（+1）・v80.3.0（+1）・v80.4.0（+2）の各 code-reviewer 対応で
> 合計 4 件追加されたことが原因。完了時の目標は **3823**。

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、3821 tests, 0 failures を確認する
- [x] `Cargo.toml` バージョンが `80.0.0` であることを確認する
- [x] `fav/src/driver.rs` に `mod v80400_tests` が存在することを確認する（v80.4.0 完了済みの証拠）
- [x] `fav/src/test_framework.rs` に v80.4.0 の `PropertyTestSuite` / `run_property_test_suite` が定義済みであることを確認する

## T1: `fav/src/test_framework.rs` に追記

- [x] `StageInput` 構造体（`#[derive(Debug, Clone)]`、`name: String`, `rows: Vec<Vec<String>>`）を追加する
- [x] `StageOutput` 構造体（`#[derive(Debug, Clone)]`、`name: String`, `rows: Vec<Vec<String>>`）を追加する
- [x] `StageTestCase` 構造体（`#[derive(Debug)]`、`stage_name: String`, `input: StageInput`, `expected: StageOutput`）を追加する
- [x] `run_stage_test(test: &StageTestCase, actual: &StageOutput) -> TestCase` を実装する
  - 行単位で `test.expected.rows` と `actual.rows` を比較する
  - 超過行も diff として扱う（`max_len = expected.len().max(actual.rows.len())`）
  - 最初の不一致行で `TestStatus::Fail` を返し `message` に差分情報を含める
  - 全行一致で `TestStatus::Pass` / `message: None` を返す
- [x] `format_stage_test_result(result: &TestCase) -> String` を実装する
  - `Pass` → `"PASS: <name>"`
  - `Fail` → `"FAIL: <name> — <message>"`
  - `Skip` → `"SKIP: <name>"`

## T2: `fav/src/driver.rs` に `mod v80500_tests` を追加

- [x] `mod v80400_tests { ... }` の直後に `#[cfg(test)] mod v80500_tests { ... }` を追加する
- [x] `use fav_core::test_framework::*;` でインポートする
- [x] `make_stage_test()` ヘルパー関数（alice/30 の 1 行フィクスチャ）を定義する
- [x] `stage_test_pass_when_output_matches` テストを実装する
  - 同一行の actual → `TestStatus::Pass`、`message.is_none()`
  - `format_stage_test_result` が `"PASS: transform"` を返すことを確認する
- [x] `stage_test_fail_when_output_differs` テストを実装する
  - 異なる行の actual → `TestStatus::Fail`、`message.is_some()`
  - message に `"row 0 differs"` / `"alice"` / `"bob"` が含まれることを確認する
  - `format_stage_test_result` が `"FAIL: transform"` で始まることを確認する

## T3: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | tail -5` を実行し、3823 tests, 0 failures であることを確認する

## T4: CHANGELOG 更新

- [x] `CHANGELOG.md` の先頭に v80.5.0 エントリを追加する

## T-last: CI 事前確認

`cargo test` 完了後（`target/debug/fav` バイナリが存在することを前提）に実行する。

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
