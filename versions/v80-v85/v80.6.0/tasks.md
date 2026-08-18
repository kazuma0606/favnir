# Tasks: v80.6.0 — テストカバレッジレポート（`TestCoverageReport`）

> `lib.rs` への変更は不要（`pub mod test_framework;` は v80.1.0 で宣言済み）。
> MILESTONE.md / README.md / `site/content/docs/` の更新は v81.0.0 宣言バージョンで実施する。
> ロードマップのテスト数（3821）と実際のベース（3825）が 6 件ずれているが、
> v80.2.0〜v80.5.0 の各 code-reviewer 対応で累積 6 件追加されたことが原因。完了時の目標は **3827**。

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、3825 tests, 0 failures を確認する
- [x] `Cargo.toml` バージョンが `80.0.0` であることを確認する（本スプリントでは Cargo.toml は v81.0.0 クリーンアップ時に更新する慣例のため、v80.x.x 実装中は `80.0.0` のまま）
- [x] `fav/src/driver.rs` に `mod v80500_tests` が存在することを確認する（v80.5.0 完了済みの証拠）
- [x] `fav/src/test_framework.rs` に v80.5.0 の `run_stage_test` / `format_stage_test_result` が定義済みであることを確認する

## T1: `fav/src/test_framework.rs` に追記

- [x] `CoverageEntry` 構造体（`#[derive(Debug, Clone)]`、`name: String`, `tested: bool`）を追加する
- [x] `TestCoverageReport` 構造体（`#[derive(Debug)]`、`entries: Vec<CoverageEntry>`, `total: usize`, `covered: usize`）を追加する
- [x] `compute_test_coverage(suite: &TestSuite, known_stages: &[String]) -> TestCoverageReport` を実装する
  - `HashSet` でケース名を収集し、known_stages ごとに `tested` を判定する
  - `TestCase.status` は問わず名前の一致のみで判定する
  - `total = known_stages.len()`、`covered = tested が true のエントリ数`
- [x] `format_coverage_report(report: &TestCoverageReport) -> String` を実装する
  - `"coverage: X/Y (Z.Zpct)"` 形式（小数点以下 1 桁）
- [x] `coverage_pct(report: &TestCoverageReport) -> f64` を実装する
  - `total == 0` の場合は `0.0` を返す（ゼロ除算ガード）

## T2: `fav/src/driver.rs` に `mod v80600_tests` を追加

- [x] `mod v80500_tests { ... }` の直後に `#[cfg(test)] mod v80600_tests { ... }` を追加する
- [x] `use fav_core::test_framework::*;` でインポートする
- [x] `coverage_report_counts_correctly` テストを実装する
  - suite に 2 ケース（load / transform）、known_stages に 3 件（load / transform / export）
  - `report.total == 3`、`report.covered == 2`
  - `entries[2].tested == false`（export は未テスト）
  - `coverage_pct` が 66.0 超 67.0 未満であることを確認する
  - `format_coverage_report` が `"coverage: 2/3 (66.7pct)"` を返すことを確認する
- [x] `coverage_pct_is_zero_when_nothing_tested` テストを実装する
  - 空の suite、known_stages に 1 件
  - `report.covered == 0`、`coverage_pct == 0.0`
  - `format_coverage_report` が `"coverage: 0/1 (0.0pct)"` を返すことを確認する

## T3: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | tail -5` を実行し、3827 tests, 0 failures であることを確認する

## T4: CHANGELOG 更新

- [x] `CHANGELOG.md` の先頭に v80.6.0 エントリを追加する

## T-last: CI 事前確認

`cargo test` 完了後（`target/debug/fav` バイナリが存在することを前提）に実行する。

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
