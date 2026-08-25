# v83.9.0 タスクリスト

Status: COMPLETE

---

## T0: 事前確認

- [x] `cargo test` が 3,903 tests pass、0 failures であることを確認する（前提: v83.8.0 完了済み）

## T1: `driver.rs` に `v83900_tests` を追加

- [x] `v83800_tests` の直後に `#[cfg(test)] mod v83900_tests` を追加する
  - `observability_full_sprint_all_stable`:
    - StageMetrics → PipelineMetrics → AlertRule → evaluate_alert_rules（発火確認）→ SloStatus（未超過）→ compute_pipeline_health（Degraded）→ ObserveReport → format_observe_report（"=== Observe:" 含む）→ cmd_observe（E2E: format_observe_report と等価）→ PerfBaseline → detect_perf_regression（None）
    - ※ TraceContext / ExecutionCost 系は個別テスト（v83500_tests / v83400_tests）で網羅済みのため対象外
  - `health_dashboard_and_alerts_integrated`:
    - rows_failed=10 の PipelineMetrics → AlertRule（rows_failed > 5）→ evaluate_alert_rules（1件発火）→ SloStatus（breached=true）→ compute_pipeline_health（Critical）→ HealthDashboard → format_health_dashboard（"Critical" 含む）

## T2: `CHANGELOG.md` 更新

- [x] `CHANGELOG.md` の先頭に v83.9.0 エントリを追加する

## T3: テスト通過確認

- [x] `cargo test` が 3,905 tests pass（+2）、0 failures であることを確認する

## T4: 最終確認（CI チェック）

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する

## 付記: 実装中の修正

- v83.8.0 `compute_pipeline_health` の `debug_assert!` に `manual_range_contains` Clippy 警告 → `quality >= 0.0 && quality <= 100.0` を `(0.0..=100.0).contains(&quality)` に修正

## code-reviewer 対応

- [LOW] `ObserveReport` 構築時の不要な `.clone()` 3件（metrics / alerts / slo）: ムーブに変更
- [LOW] JSON フォーマットパスが統合テストで未確認: `format_observe_report(&report, &ObserveFormat::Json)` + `assert!(json.contains("sprint_test"))` を追加
