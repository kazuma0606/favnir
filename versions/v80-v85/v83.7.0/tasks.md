# v83.7.0 タスクリスト

Status: COMPLETE

---

## T0: 事前確認

- [x] `cargo test` が 3,899 tests pass、0 failures であることを確認する（前提: v83.6.0 完了済み）

## T1: `test_framework.rs` に enum・構造体を追加

- [x] `ObserveFormat` enum を追加する（`#[derive(Debug, Clone, PartialEq)]`）
  - `Text`, `Json`
- [x] `ObserveOptions` 構造体を追加する（`#[derive(Debug, Clone, PartialEq)]`）
  - `pipeline_name: String`, `format: ObserveFormat`, `show_alerts: bool`, `show_slo: bool`
- [x] `ObserveReport` 構造体を追加する（`#[derive(Debug, Clone, PartialEq)]`）
  - `metrics: PipelineMetrics`, `alerts: Vec<AlertFiring>`, `slo_statuses: Vec<SloStatus>`

## T2: `format_observe_report` / `cmd_observe` 関数を追加

- [x] `format_observe_report(report: &ObserveReport, format: &ObserveFormat) -> String` を追加する
  - `ObserveFormat::Text`: "=== Observe: {pipeline_name} ===" ヘッダ + `format_metrics_summary` + alerts + slo_statuses
  - `ObserveFormat::Json`: `{"pipeline":"<name>","alerts_count":<n>,"slo_count":<n>}` 形式
  - alerts が空のとき "Alert:" 行を出力しない
  - slo_statuses が空のとき "SLO:" 行を出力しない
- [x] `cmd_observe(options: &ObserveOptions, report: &ObserveReport) -> String` を追加する
  - `format_observe_report(report, &options.format)` を呼び出す

## T3: `driver.rs` に `v83700_tests` を追加

- [x] `v83600_tests` の直後に `#[cfg(test)] mod v83700_tests` を追加する
  - `observe_report_built`: `ObserveReport` の構築と `alerts.len()`/`slo_statuses.len()` を確認、`cmd_observe` と `format_observe_report` の等価性を確認
  - `observe_report_text_format`: Text フォーマット（"=== Observe:" 含む）、alerts を空にした別 `ObserveReport` を構築して "Alert:" 行が含まれないことを確認、JSON フォーマット（`{"pipeline":` 含む）スモークテスト

## T4: `CHANGELOG.md` 更新

- [x] `CHANGELOG.md` の先頭に v83.7.0 エントリを追加する

## T5: テスト通過確認

- [x] `cargo test` が 3,901 tests pass（+2）、0 failures であることを確認する

## T6: 最終確認（CI チェック）

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する

## code-reviewer 対応

- [HIGH] JSON 出力の `pipeline_name` エスケープ欠如: `.replace('\\', "\\\\").replace('"', "\\\"")` で手動エスケープを追加
- [MED] `ObserveOptions.pipeline_name` 未使用: 将来の CLI フィルタリング（`--pipeline <name>`）用であることを doc コメントに明記
- [MED] SLO 非空時の Text 出力テスト欠落: `assert!(text.contains("SLO:"))` を `observe_report_text_format` に追加
- [LOW] `format_slo_status` の改行と join の噛み合い: 意図的な設計（join で自然な区切りになる）のため対応不要
- [LOW] `cmd_observe` 拡張性: 将来フィルタ追加時は `report` をクローンして加工するパターンを採用予定、現時点はコード変更不要
