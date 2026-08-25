# v83.9.0 仕様書 — 安定化・コードフリーズ

## Background

v83.1〜v83.8 で Observability 2.0 スプリントの全型・関数を整備した。
v83.9.0 は安定化フェーズ。新機能追加なしで、スプリント全体の統合確認テストを追加する。

本バージョンは Observability 2.0 スプリント（v83.1〜v84.0）の第 9 段階（コードフリーズ）。

ロードマップ参照: `versions/roadmap/roadmap-v83.1-v84.0.md` — v83.9.0 セクション

## Goals

1. v83.1〜v83.8 の全テスト通過を確認する（`cargo test` 全 pass）
2. `observability_full_sprint_all_stable` 統合テストを追加する
   - StageMetrics → PipelineMetrics → AlertRule → AlertFiring → SloStatus → HealthDashboard → ObserveReport の主要型が連携して動作することを確認
   - `cmd_observe` が `ObserveOptions` + `ObserveReport` から正しく出力を返すことを E2E 確認
   - ※ TraceContext（v83.5.0）/ ExecutionCost・CostBudget（v83.4.0）/ PerfBaseline（v83.6.0）は個別テストで網羅済みのため、このテストでは一部のみ確認
3. `health_dashboard_and_alerts_integrated` 統合テストを追加する
   - `HealthDashboard` + `AlertRule`（`evaluate_alert_rules`）+ `SloStatus`（`compute_slo_status`）の連携シナリオが通ることを確認

**新規型・関数の追加なし。バグ修正のみ受け入れ。**

## テスト内容

### `observability_full_sprint_all_stable`

v83.1〜v83.8 で追加した型・関数を一通り呼び出し、スプリント全体が壊れていないことを確認する統合テスト。

シナリオ:
1. `StageMetrics` 2件を `compute_pipeline_metrics` でまとめて `PipelineMetrics` を作成
2. `AlertRule` を定義し `evaluate_alert_rules` で `AlertFiring` を評価（duration_ms > 500 → 発火）
3. `SloTarget` + `SloMeasurement` → `compute_slo_status`（SLO 未超過）
4. `compute_pipeline_health` で `HealthStatus::Degraded`（アラートあり・SLO OK）を確認
5. `ObserveReport` を構築し `format_observe_report(&report, &ObserveFormat::Text)` が "=== Observe:" を含むことを確認
6. `cmd_observe` を `ObserveOptions`（`format: ObserveFormat::Text`）と共に呼び出し、`format_observe_report` と等価な文字列を返すことを確認（`fav observe` E2E）
7. `PerfBaseline::from_samples` → `detect_perf_regression`（閾値以内 → None）を確認

全アサートが pass → スプリント全体が安定していることを宣言。

### `health_dashboard_and_alerts_integrated`

`HealthDashboard` + `AlertRule`（`evaluate_alert_rules`）+ `SloStatus`（`compute_slo_status`）を組み合わせた連携シナリオ。

シナリオ:
1. `PipelineMetrics` を作成（rows_failed=10 のステージを含む）
2. `AlertRule`（rows_failed > 5 → Warning）を `evaluate_alert_rules` で評価 → `AlertFiring` 1件が発火
3. `SloTarget`（99.5%）+ `SloMeasurement`（good=970/1000）→ `compute_slo_status`（breached=true）
4. `compute_pipeline_health` → `HealthStatus::Critical(...)` を確認
5. `HealthDashboard` を構築し `format_health_dashboard` が "Critical" を含むことを確認

## Success Criteria

- `cargo test` が 3905 tests pass（+2）、0 failures
- `observability_full_sprint_all_stable` が pass する（v83.1〜v83.8 全型の連携確認）
- `health_dashboard_and_alerts_integrated` が pass する（HealthDashboard + AlertRule + SloStatus 連携）

## Files to Modify

- `fav/src/driver.rs` — `v83900_tests` モジュール追加
- `CHANGELOG.md` — v83.9.0 エントリ追加
