# Tasks: v81.7.0 — `fav quality report` コマンド

> COMPLETE — 2026-08-19
> 3857 tests, 0 failures（+2 from 3855）

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、3855 tests, 0 failures を確認する
- [x] `grep -m1 '^version' fav/Cargo.toml` の出力が `version = "81.0.0"` であることを確認する
  （v81.x マイナーバージョンは Cargo.toml を更新しない慣例。このバージョン完了後も `81.0.0` のまま変更しない）
- [x] `fav/src/driver.rs` に `mod v81600_tests` が存在することを確認する（v81.6.0 完了済みの証拠）
- [x] `fav/src/test_framework.rs` に `QualityCheck` / `run_quality_check` / `QualityViolation` が定義済みであることを確認する

## T1: `fav/src/test_framework.rs` に追記

- [x] `ReportFormat` enum（`#[derive(Debug, Clone, PartialEq)]`）を追加する
  - バリアント: `Text`, `Json`, `Markdown`
- [x] `QualityReportOptions` 構造体（`#[derive(Debug, Clone)]`）を追加する
  - フィールド: `format: ReportFormat`, `include_violations: bool`, `include_stats: bool`
- [x] `build_quality_report(check: &QualityCheck, rows: &[Vec<String>], opts: &QualityReportOptions) -> String` を実装する
  - `run_quality_check` を内部で呼び出す
  - `Text`: `"quality_report format=text violations={n}"` + include_violations 時に `"\n- row={} col={} rule={:?}"` を追記
  - `Json`: `{"format":"json","violations":{n}}` + include_violations 時に `"items":[...]` を追記
  - `Markdown`: `"## Quality Report\nformat: markdown\nviolations: {n}"` + include_violations 時にリスト追記
- [x] `cmd_quality_report(check, rows, opts) -> String` を実装する（`build_quality_report` に委譲）

## T2: `fav/src/driver.rs` に `mod v81700_tests` を追加

- [x] `mod v81600_tests { ... }` の直後に `#[cfg(test)] mod v81700_tests { ... }` を追加する
- [x] `use fav_core::test_framework::*;` でインポートする
- [x] `quality_report_text_format` テストを実装する
  - `ReportFormat::Text` の出力に `"text"` / `"violations"` / `'0'` が含まれることを確認する
  - `cmd_quality_report` が `build_quality_report` と同一結果を返すことを確認する
- [x] `quality_report_json_format` テストを実装する
  - `ReportFormat::Json` の出力に `"json"` / `"violations"` が含まれることを確認する
  - `ReportFormat::Markdown` の出力に `"Quality Report"` / `"violations"` が含まれることを確認する（smoke test）

## T3: CHANGELOG 更新

- [x] `CHANGELOG.md` の先頭に v81.7.0 エントリを追加する

## T4: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、3857 tests, 0 failures であることを確認する

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
