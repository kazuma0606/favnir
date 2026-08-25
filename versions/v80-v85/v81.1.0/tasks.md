# Tasks: v81.1.0 — `QualityRule` / `QualityCheck` 型基盤

> `lib.rs` への変更は不要（`pub mod test_framework;` は v80.1.0 で宣言済み）。
> MILESTONE.md / README.md / `site/content/docs/` の更新は v82.0.0 宣言バージョンで実施する。
> ロードマップのテスト数（3833）と実際のベース（3841）が 8 件ずれているが、
> v80.x スプリントの code-reviewer 累積 drift が原因。完了時の目標は **3843**。
> ロードマップは drift 補正済み（3841 ベースに更新）。

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、3841 tests, 0 failures を確認する
- [x] `Cargo.toml` バージョンが `81.0.0` であることを確認する（本スプリントでは v82.0.0 クリーンアップ時に更新する慣例）
- [x] `fav/src/driver.rs` に `mod v81000_tests` が存在することを確認する（v81.0.0 完了済みの証拠）
- [x] `fav/src/test_framework.rs` に v80.8.0 の `TestReport` / `format_junit_xml` / `format_test_summary` が定義済みであることを確認する

## T1: `fav/src/test_framework.rs` に追記

- [x] `QualityRuleKind` enum（`#[derive(Debug, Clone)]`）を追加する
  - バリアント: `NotNull` / `Unique` / `Range { min: f64, max: f64 }` / `Regex(String)` / `Custom(String)`
- [x] `RuleSeverity` enum（`#[derive(Debug, Clone)]`）を追加する（`Error` / `Warning`）
- [x] `QualityRule` 構造体（`#[derive(Debug, Clone)]`、`column: String`, `kind: QualityRuleKind`, `severity: RuleSeverity`）を追加する
- [x] `QualityCheck` 構造体（`#[derive(Debug)]`、`rules: Vec<QualityRule>`）を追加する
- [x] `QualityViolation` 構造体（`#[derive(Debug)]`、`rule: QualityRule`, `row_index: usize`, `actual: String`）を追加する
- [x] `run_quality_check(check: &QualityCheck, rows: &[Vec<String>]) -> Vec<QualityViolation>` を実装する
  - `column` をカラムインデックス文字列として `parse::<usize>()` で解釈する
  - `NotNull`: `value.trim().is_empty()` なら違反
  - `Range { min, max }`: `value.parse::<f64>()` して `v < min || v > max` なら違反（パース失敗はスキップ）
  - `Regex(pattern)`: `!value.contains(pattern)` なら違反
  - `Unique` / `Custom`: スキップ（行単位チェック非対応）

## T2: `fav/src/driver.rs` に `mod v81100_tests` を追加

- [x] `mod v81000_tests { ... }` の直後に `#[cfg(test)] mod v81100_tests { ... }` を追加する
- [x] `use fav_core::test_framework::*;` でインポートする
- [x] `quality_rule_not_null_catches_violation` テストを実装する
  - 空値を含む行と非空値の行を用意し、NotNull ルールが 1 件の違反のみ返すことを確認する
  - `violation.row_index == 0` と `violation.actual == ""` を確認する
- [x] `quality_check_returns_all_violations` テストを実装する
  - NotNull + Range の 2 ルールを持つ `QualityCheck` を適用する
  - 2 件の違反が返ること、違反のない行は含まれないことを確認する

## T3: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、3843 tests, 0 failures であることを確認する

## T4: CHANGELOG 更新

- [x] `CHANGELOG.md` の先頭に v81.1.0 エントリを追加する

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
