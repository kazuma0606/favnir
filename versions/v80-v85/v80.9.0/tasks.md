# Tasks: v80.9.0 — 安定化・コードフリーズ（Test-Driven Data 1.0 完成宣言）

> `test_framework.rs` / `lib.rs` への変更は不要（全型・関数は v80.1.0〜v80.8.0 で定義済み）。
> MILESTONE.md / README.md / `site/content/docs/` の更新は v81.0.0 宣言バージョンで実施する。
> ロードマップのテスト数（3827）と実際のベース（3835）が 8 件ずれているが、
> v80.2.0〜v80.8.0 の各 code-reviewer 対応で累積 9 件追加されたことが原因。完了時の目標は **3837**。

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、3835 tests, 0 failures を確認する
- [x] `Cargo.toml` バージョンが `80.0.0` であることを確認する（本スプリントでは v81.0.0 クリーンアップ時に更新する慣例）
- [x] `fav/src/driver.rs` に `mod v80800_tests` が存在することを確認する（v80.8.0 完了済みの証拠）
- [x] `fav/src/test_framework.rs` に v80.8.0 の `TestReport` / `format_junit_xml` / `format_test_summary` が定義済みであることを確認する

## T1: `fav/src/driver.rs` に `mod v80900_tests` を追加

- [x] `mod v80800_tests { ... }` の直後に `#[cfg(test)] mod v80900_tests { ... }` を追加する
- [x] `use fav_core::test_framework::*;` でインポートする
- [x] `test_framework_full_sprint_all_stable` テストを実装する
  - v80.1〜v80.8 の全型（TestSuite / DataFactory / PropertyTest / StageTestCase / TestCoverageReport / SchemaSnapshot / TestReport）を各 1 インスタンス生成する
  - 各関数呼び出しがパニックしないことを確認する（戻り値は `let _ =` で受け取る）
- [x] `test_framework_e2e_pipeline_tested` テストを実装する
  - `DataFactory::from_seed(1)` → `generate_rows` → `StageOutput` → `run_stage_test` → `TestSuite` → `TestReport` → `format_test_summary` のフローを実行する
  - `format_test_summary` の出力に `"pipeline_tests"` が含まれることを確認する

## T2: （本バージョンは安定化のみのため新規実装ステップなし — 欠番）

## T3: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | tail -5` を実行し、3837 tests, 0 failures であることを確認する

## T4: CHANGELOG 更新

> 注意: テストモジュールに `changelog_has_vXX` テストが含まれるバージョンでは、
> T3（cargo test）より **前** に CHANGELOG を更新すること。
> 本バージョンの `v80900_tests` には CHANGELOG チェックテストは含まれないため順序は問わない。

- [x] `CHANGELOG.md` の先頭に v80.9.0 エントリを追加する

## T-last: CI 事前確認

`cargo test` 完了後（`target/debug/fav` バイナリが存在することを前提）に実行する。

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
