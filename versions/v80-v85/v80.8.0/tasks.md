# Tasks: v80.8.0 — CI 統合レポート（`TestReport` / JUnit XML）

> `lib.rs` への変更は不要（`pub mod test_framework;` は v80.1.0 で宣言済み）。
> MILESTONE.md / README.md / `site/content/docs/` の更新は v81.0.0 宣言バージョンで実施する。
> ロードマップのテスト数（3825）と実際のベース（3832）が 9 件ずれているが、
> v80.2.0〜v80.7.0 の各 code-reviewer 対応で累積 9 件追加されたことが原因。完了時の目標は **3834**。
> `cmd_test` への `--format` オプション追加はスコープ外（v80.9.0 で検討）。

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、3832 tests, 0 failures を確認する
- [x] `Cargo.toml` バージョンが `80.0.0` であることを確認する（本スプリントでは v81.0.0 クリーンアップ時に更新する慣例）
- [x] `fav/src/driver.rs` に `mod v80700_tests` が存在することを確認する（v80.7.0 完了済みの証拠）
- [x] `fav/src/test_framework.rs` に v80.7.0 の `compare_schema_snapshots` / `schema_diff_is_breaking` が定義済みであることを確認する

## T1: `fav/src/test_framework.rs` に追記

- [x] `TestReport` 構造体（`#[derive(Debug)]`、`suite: TestSuite`, `duration_ms: u64`, `timestamp: String`）を追加する
- [x] `format_junit_xml(report: &TestReport) -> String` を実装する
  - `run_test_suite` を呼び出して passed / failed / skipped を集計する
  - `time` は `duration_ms as f64 / 1000.0`（小数点以下 3 桁）
  - `<?xml version="1.0" encoding="UTF-8"?>` ヘッダーを出力する
  - `<testsuite name=... tests=... failures=... skipped=... time=...>` タグを出力する
  - Pass / Skip ケース: `<testcase name=... classname=.../>`
  - Fail ケース: `<testcase ...><failure message=.../></testcase>`
  - XML エスケープは本バージョンのスコープ外
- [x] `format_test_summary(report: &TestReport) -> String` を実装する
  - 出力形式: `"{suite.name}: N passed, M failed, K skipped ({duration_ms}ms) [{timestamp}]"`

## T2: `fav/src/driver.rs` に `mod v80800_tests` を追加

- [x] `mod v80700_tests { ... }` の直後に `#[cfg(test)] mod v80800_tests { ... }` を追加する
- [x] `use fav_core::test_framework::*;` でインポートする
- [x] `make_report()` ヘルパー関数（Pass 1 件 + Fail 1 件、duration_ms=42）を定義する
- [x] `junit_xml_output_has_testsuite_tag` テストを実装する
  - 出力に `"<testsuite"` / `"<testcase"` / `"<failure"` が含まれることを確認する
  - failure message（`"expected 1 got 2"`）が出力に含まれることを確認する
- [x] `test_report_summary_shows_pass_count` テストを実装する
  - 出力が `"pipeline_tests: 1 passed, 1 failed, 0 skipped (42ms) [2026-08-19T00:00:00Z]"` と完全一致することを確認する

## T3: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | tail -5` を実行し、3834 tests, 0 failures であることを確認する

## T4: CHANGELOG 更新

> 注意: テストモジュールに `changelog_has_vXX` テストが含まれるバージョンでは、
> T3（cargo test）より **前** に CHANGELOG を更新すること。
> 本バージョンの `v80800_tests` には CHANGELOG チェックテストは含まれないため順序は問わない。

- [x] `CHANGELOG.md` の先頭に v80.8.0 エントリを追加する

## T-last: CI 事前確認

`cargo test` 完了後（`target/debug/fav` バイナリが存在することを前提）に実行する。

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
