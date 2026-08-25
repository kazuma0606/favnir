# v83.3.0 タスクリスト

Status: COMPLETE

---

## T0: 事前確認

- [x] `cargo test` が 3,891 tests pass、0 failures であることを確認する（前提: v83.2.0 完了済み）

## T1: `test_framework.rs` に構造体を追加

- [x] `SloTarget` 構造体を追加する（`#[derive(Debug, Clone, PartialEq)]`）
  - `name: String`, `objective_pct: f64`, `window_hours: u64`
- [x] `SloMeasurement` 構造体を追加する（`#[derive(Debug, Clone, PartialEq)]`）
  - `good_events: u64`, `total_events: u64`, `window_hours: u64`
- [x] `SloStatus` 構造体を追加する（`#[derive(Debug, Clone, PartialEq)]`）
  - `target: SloTarget`, `current_pct: f64`, `error_budget_remaining_pct: f64`, `breached: bool`

## T2: `compute_slo_status` 関数を追加

- [x] `compute_slo_status(target: &SloTarget, measurement: &SloMeasurement) -> SloStatus` を追加する
  - `total_events == 0` のとき `current_pct = 100.0`、`error_budget_remaining_pct = 100.0`、`breached = false`
  - `objective_pct == 100.0` のとき `error_budget_remaining_pct = 0.0`（ゼロ除算ガード）
  - `breached = current_pct < objective_pct`

## T3: `format_slo_status` 関数を追加

- [x] `format_slo_status(status: &SloStatus) -> String` を追加する
  - "SLO:"、"Objective:"、"Current:"、"Error Budget:"、"Status:" の各行を含む

## T4: `driver.rs` に `v83300_tests` を追加

- [x] `v83200_tests` の直後に `#[cfg(test)] mod v83300_tests` を追加する
  - `slo_status_within_budget`（`format_slo_status` スモークテスト含む）
  - `slo_status_breached`

## T5: テスト通過確認

- [x] `cargo test` が 3,893 tests pass（+2）、0 failures であることを確認する

## T6: 最終確認（CI チェック）

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する

## code-reviewer 対応

- [x] [MED] `objective_pct == 100.0` breach 時の `error_budget_remaining_pct = 0.0` 挙動を doc コメントに明記（`breached` フィールド参照を促す）
- [x] [MED] `good_events > total_events` に `debug_assert!` を追加（デバッグビルドで不正入力を検知）
- [x] [LOW] `total_events == 0` と `objective_pct == 100.0` のエッジケース確認を `slo_status_within_budget` 内に追加（テスト数 3893 維持）
- [x] [LOW] `error_budget_remaining_pct` に具体値 50.0 のアサーション追加
