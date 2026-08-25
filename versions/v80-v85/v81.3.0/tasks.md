# Tasks: v81.3.0 — スキーマドリフト検出（`SchemaDriftDetector`）

> COMPLETE — 2026-08-19
> 3849 tests, 0 failures（+2 from 3847）

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、3847 tests, 0 failures を確認する
- [x] `Cargo.toml` バージョンが `81.0.0` であることを確認する
  （v81.x マイナーバージョンは Cargo.toml を更新しない慣例。v82.0.0 宣言時に一括更新する）
- [x] `fav/src/driver.rs` に `mod v81200_tests` が存在することを確認する（v81.2.0 完了済みの証拠）
- [x] `fav/src/test_framework.rs` に `SchemaSnapshot` / `SchemaSnapshotDiff` / `ColumnSnapshot` / `compare_schema_snapshots` が定義済みであることを確認する
- [x] `fav/src/test_framework.rs` に `RuleSeverity` が定義済みであることを確認する

## T1: `fav/src/test_framework.rs` に追記

- [x] `DriftTolerance` enum（`#[derive(Debug, Clone, PartialEq)]`）を追加する
  - バリアント: `Strict`, `Additive`, `Permissive`
- [x] `SchemaDriftDetector` 構造体（`#[derive(Debug, Clone)]`）を追加する
  - フィールド: `baseline: SchemaSnapshot`, `tolerance: DriftTolerance`
- [x] `DriftResult` 構造体（`#[derive(Debug)]`）を追加する
  - フィールド: `has_drift: bool`, `severity: RuleSeverity`, `diff: SchemaSnapshotDiff`
- [x] `detect_schema_drift(detector: &SchemaDriftDetector, current: &SchemaSnapshot) -> DriftResult` を実装する
  - 内部で `compare_schema_snapshots(current, &detector.baseline)` を呼ぶ
  - `Strict`: 追加・削除・変更のいずれかがあれば `has_drift = true`
  - `Additive` / `Permissive`: 削除・変更があれば `has_drift = true`（追加のみは `false`）
  - `has_drift = true` のとき `severity = RuleSeverity::Error`、そうでないとき `Warning`
- [x] `format_drift_report(result: &DriftResult) -> String` を実装する
  - `has_drift = false`: `"OK: no schema drift detected"`
  - `has_drift = true`: `"DRIFT: added={:?} removed={:?} changed={:?}"`

## T2: `fav/src/driver.rs` に `mod v81300_tests` を追加

- [x] `mod v81200_tests { ... }` の直後に `#[cfg(test)] mod v81300_tests { ... }` を追加する
- [x] ヘルパー `make_snapshot(cols: &[(&str, &str, bool)]) -> SchemaSnapshot` を定義する
- [x] `drift_detector_strict_mode_catches_addition` テストを実装する
  - baseline: `[("id", "Int", false)]`、current: `[("id", ...), ("name", "String", true)]`
  - `DriftTolerance::Strict` で `has_drift = true` を確認する
  - `diff.added` に `"name"` が含まれることを確認する
  - `format_drift_report` の出力に `"DRIFT"` / `"added="` / `"name"` が含まれることを確認する
- [x] `drift_detector_additive_mode_allows_new_column` テストを実装する
  - 同じデータで `DriftTolerance::Additive` → `has_drift = false` を確認する
  - `diff.added` に `"name"` が含まれることを確認する（diff は常に計算）
  - 削除ありデータ `current_missing = []` で `has_drift = true` を確認する

## T3: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、3849 tests, 0 failures であることを確認する

## T4: CHANGELOG 更新

- [x] `CHANGELOG.md` の先頭に v81.3.0 エントリを追加する

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
