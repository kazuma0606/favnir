# Roadmap v83.1.0 〜 v84.0.0 — Observability 2.0

Date: 2026-08-16
Status: 未着手（v83.0.0 完了後に開始）

マスターロードマップ: [roadmap-v80.1-v85.0.md](roadmap-v80.1-v85.0.md)

---

## 前提

- 直前完了: v83.0.0「Pipeline Contracts 1.0 宣言」（tests = 3,875）
- 本スプリントは Quality-First Era の第 4 スプリント
- 目標: v84.0.0「Observability 2.0 宣言」（tests = 3,909 ※ drift 補正後）
- **着手前確認**: `versions/current.md` の現行マスターロードマップが `roadmap-v80.1-v85.0.md` を指していること、最新安定版が v83.0.0 になっていることを確認してから開始する

### スプリントの性格

v29.0「Observability First」（OTel / Prometheus / Datadog 統合）を土台に、
パイプラインの健全性を **型** として表現し直す。
メトリクス・アラート・SLO・コスト追跡の 4 層を積み上げ、
「壊れる前に型が教えてくれる」観測基盤を完成させる。
A（新機能）50% + B（統合）50% の構成。

---

## バージョン一覧

| バージョン | 内容 | テスト数 | 状態 |
|---|---|---|---|
| v83.1.0 | `PipelineMetrics` 型（実行統計・レイテンシ） | 3887 + 2 = 3889 ※ drift 補正後 | 未着手 |
| v83.2.0 | `AlertRule` / `AlertThreshold`（アラート型） | 3889 + 2 = 3891 ※ drift 補正後 | 未着手 |
| v83.3.0 | `SloTarget` / `SloStatus`（SLO 型） | 3891 + 2 = 3893 ※ drift 補正後 | 未着手 |
| v83.4.0 | コスト追跡（`ExecutionCost` / `CostBudget`） | 3893 + 2 = 3895 ※ drift 補正後 | 未着手 |
| v83.5.0 | 分散トレーシング強化（OpenTelemetry `TraceContext`） | 3895 + 2 = 3897 ※ drift 補正後 | 未着手 |
| v83.6.0 | パフォーマンス回帰検知（`PerfBaseline` / `PerfRegression`） | 3897 + 2 = 3899 ※ drift 補正後 | 未着手 |
| v83.7.0 | `fav observe` コマンド（メトリクス・アラート統合） | 3899 + 2 = 3901 ※ drift 補正後 | 未着手 |
| v83.8.0 | 健全性ダッシュボード（`HealthDashboard` / テキスト形式） | 3901 + 2 = 3903 ※ drift 補正後 | 未着手 |
| v83.9.0 | 安定化・コードフリーズ | 3903 + 2 = 3905 ※ drift 補正後 | 未着手 |
| v84.0.0 | Observability 2.0 宣言 ★クリーンアップ | 3905 + 4 = 3909 ※ drift 補正後 | 未着手 |

---

## v83.1.0 — `PipelineMetrics` 型（実行統計・レイテンシ）

パイプライン実行の統計情報を構造化した型として収集・集計する。

**実装内容:**
- `StageMetrics` 構造体（`stage_name: String`, `duration_ms: u64`, `rows_processed: usize`, `rows_failed: usize`）
- `PipelineMetrics` 構造体（`pipeline_name: String`, `stages: Vec<StageMetrics>`, `total_duration_ms: u64`, `started_at: String`）
- `compute_pipeline_metrics(pipeline_name: &str, stages: Vec<StageMetrics>, started_at: &str) -> PipelineMetrics`
- `format_metrics_summary(metrics: &PipelineMetrics) -> String`
- `slowest_stage(metrics: &PipelineMetrics) -> Option<&StageMetrics>`

**完了条件**: Rust テスト 2 件（3887 + 2 = 3889 ※ drift 補正後）
- `pipeline_metrics_computed`
- `slowest_stage_identified`

---

## v83.2.0 — `AlertRule` / `AlertThreshold`（アラート型）

メトリクスの閾値超過をアラートとして型で定義し、評価する。

**実装内容:**
- `AlertSeverity` enum（`Critical` / `Warning` / `Info`）
- `AlertThreshold` 構造体（`metric: String`, `operator: ThresholdOp`, `value: f64`）（`metric` は `"total_duration_ms"` / `"rows_failed"` / `"rows_processed"` 等、`PipelineMetrics` のフィールド名に対応する文字列）
- `ThresholdOp` enum（`GreaterThan` / `LessThan` / `EqualTo`）
- `AlertRule` 構造体（`name: String`, `threshold: AlertThreshold`, `severity: AlertSeverity`, `message: String`）
- `AlertFiring` 構造体（`rule: AlertRule`, `current_value: f64`, `fired_at: String`）
- `evaluate_alert_rules(rules: &[AlertRule], metrics: &PipelineMetrics, fired_at: &str) -> Vec<AlertFiring>`

**完了条件**: Rust テスト 2 件（3889 + 2 = 3891 ※ drift 補正後）
- `alert_fires_when_threshold_exceeded`
- `alert_silent_when_within_threshold`

---

## v83.3.0 — `SloTarget` / `SloStatus`（SLO 型）

サービスレベル目標（SLO）を型で宣言し、エラーバジェット消費を追跡する。

**実装内容:**
- `SloTarget` 構造体（`name: String`, `objective_pct: f64`, `window_hours: u64`）
- `SloMeasurement` 構造体（`good_events: u64`, `total_events: u64`, `window_hours: u64`）
- `SloStatus` 構造体（`target: SloTarget`, `current_pct: f64`, `error_budget_remaining_pct: f64`, `breached: bool`）
- `compute_slo_status(target: &SloTarget, measurement: &SloMeasurement) -> SloStatus`
- `format_slo_status(status: &SloStatus) -> String`

**完了条件**: Rust テスト 2 件（3891 + 2 = 3893 ※ drift 補正後）
- `slo_status_within_budget`
- `slo_status_breached`

---

## v83.4.0 — コスト追跡（`ExecutionCost` / `CostBudget`）

パイプライン実行のコスト（CPU 秒・メモリ・クラウド課金見込み）を型で追跡する。

**実装内容:**
- `ResourceUsage` 構造体（`cpu_seconds: f64`, `memory_mb: f64`, `io_mb: f64`）
- `ExecutionCost` 構造体（`resource: ResourceUsage`, `estimated_usd: f64`, `pipeline_name: String`）
- `CostBudget` 構造体（`max_usd_per_run: f64`, `max_cpu_seconds: f64`）
- `BudgetStatus` enum（`UnderBudget` / `NearLimit(f64)` / `OverBudget(f64)`）
- `evaluate_cost_budget(cost: &ExecutionCost, budget: &CostBudget) -> BudgetStatus`
- `format_cost_report(cost: &ExecutionCost, status: &BudgetStatus) -> String`

**完了条件**: Rust テスト 2 件（3893 + 2 = 3895 ※ drift 補正後）
- `cost_budget_under_limit`
- `cost_budget_over_limit`

---

## v83.5.0 — 分散トレーシング強化（OpenTelemetry `TraceContext`）

v29.0 の OTel 統合を強化し、スパン伝播を型で扱う。

**実装内容:**
- `TraceContext` 構造体（`trace_id: String`, `span_id: String`, `parent_span_id: Option<String>`）
- `TraceSpan` 構造体（`context: TraceContext`, `name: String`, `start_ms: u64`, `end_ms: u64`, `attributes: Vec<(String, String)>`）
- `TraceContext::new_root() -> TraceContext`
- `TraceContext::child_span(parent: &TraceContext) -> TraceContext`
- `format_trace_span(span: &TraceSpan) -> String`
- `compute_span_duration(span: &TraceSpan) -> u64`

**完了条件**: Rust テスト 2 件（3895 + 2 = 3897 ※ drift 補正後）
- `trace_context_child_span_created`
- `span_duration_computed`

---

## v83.6.0 — パフォーマンス回帰検知（`PerfBaseline` / `PerfRegression`）

パイプライン実行のパフォーマンスをベースラインと比較し、回帰を検知する。

**実装内容:**
- `PerfBaseline` 構造体（`pipeline_name: String`, `p50_ms: u64`, `p95_ms: u64`, `p99_ms: u64`）
- `PerfRegression` 構造体（`pipeline_name: String`, `baseline: PerfBaseline`, `current_ms: u64`, `regression_pct: f64`）
- `detect_perf_regression(baseline: &PerfBaseline, current_ms: u64, threshold_pct: f64) -> Option<PerfRegression>`
- `format_regression_report(regression: &PerfRegression) -> String`
- `PerfBaseline::from_samples(pipeline_name: &str, samples_ms: &[u64]) -> PerfBaseline`（サンプルをソートして百分位数インデックス `samples[n * pct / 100]` で p50/p95/p99 を算出）

**完了条件**: Rust テスト 2 件（3897 + 2 = 3899 ※ drift 補正後）
- `perf_regression_detected_above_threshold`
- `perf_no_regression_within_threshold`

---

## v83.7.0 — `fav observe` コマンド（メトリクス・アラート統合）

`fav observe` コマンドでメトリクス収集・アラート評価・SLO 確認を一括実行する。

**実装内容:**
- `ObserveOptions` 構造体（`pipeline_name: String`, `format: ObserveFormat`, `show_alerts: bool`, `show_slo: bool`）
- `ObserveFormat` enum（`Text` / `Json`）
- `ObserveReport` 構造体（`metrics: PipelineMetrics`, `alerts: Vec<AlertFiring>`, `slo_statuses: Vec<SloStatus>`）
- `cmd_observe` 関数（`fav observe` コマンドハンドラ、CLI フラグ: `--pipeline <name>` / `--format text|json` / `--alerts` / `--slo`）
- `format_observe_report(report: &ObserveReport, format: &ObserveFormat) -> String`

**完了条件**: Rust テスト 2 件（3899 + 2 = 3901 ※ drift 補正後）
- `observe_report_built`
- `observe_report_text_format`

---

## v83.8.0 — 健全性ダッシュボード（`HealthDashboard` / テキスト形式）

パイプラインの総合健全性をテキスト形式のダッシュボードとして出力する。

**実装内容:**
- `HealthStatus` enum（`Healthy` / `Degraded(String)` / `Critical(String)`）
- `PipelineHealth` 構造体（`pipeline_name: String`, `status: HealthStatus`, `alerts_firing: usize`, `slo_breached: bool`, `quality_score: f64`）
- `HealthDashboard` 構造体（`pipelines: Vec<PipelineHealth>`, `generated_at: String`）
- `compute_pipeline_health(metrics: &PipelineMetrics, alerts: &[AlertFiring], slo: &SloStatus, quality: f64) -> PipelineHealth`
- `format_health_dashboard(dashboard: &HealthDashboard) -> String`

**完了条件**: Rust テスト 2 件（3901 + 2 = 3903 ※ drift 補正後）
- `health_dashboard_healthy_pipeline`
- `health_dashboard_critical_pipeline`

---

## v83.9.0 — 安定化・コードフリーズ

v83.1〜v83.8 の全スプリント統合確認。バグ修正のみ。

**実装内容:**
- v83.1〜v83.8 の全テスト通過確認（`cargo test` 全 pass）
- `fav observe` コマンド E2E 動作確認
- `HealthDashboard` + `AlertRule` + `SloStatus` 連携確認
- バグ修正のみ受け入れ（新機能追加なし）

**完了条件**: Rust テスト 2 件（3903 + 2 = 3905 ※ drift 補正後）（統合確認テスト — 新規実装なし）
- `observability_full_sprint_all_stable`（v83.1〜v83.8 の全テストが pass することを確認）
- `health_dashboard_and_alerts_integrated`（`HealthDashboard` + `AlertRule` + `SloStatus` の連携シナリオが通ることを確認）

---

## v84.0.0 — Observability 2.0 宣言 ★クリーンアップ

**宣言文**:
> 「メトリクスが型になり、アラートが型になり、SLO が型になった。
>  Favnir のパイプラインは壊れる前に教えてくれる。」

**クリーンアップ作業:**
- `cargo clean` 実施
- `Cargo.toml` バージョンを `84.0.0` に更新
- `CHANGELOG.md` / `MILESTONE.md` / `README.md` 更新
- `versions/current.md` の現行マスターロードマップが `roadmap-v80.1-v85.0.md` を指していることを確認してから更新
- `roadmap-v80.1-v85.0.md` の Sprint 4 バージョン一覧テーブルを全行「完了」に更新

**完了条件**: `v84000_tests` 4 件（3905 + 4 = 3909 ※ drift 補正後）
- `cargo_toml_version_is_84_0_0`
- `changelog_has_v84_0_0`
- `milestone_has_observability_2`
- `readme_mentions_fav_observe`（`README.md` に `"fav observe"` という文字列が含まれていることを確認）

---

## テスト数推移（本スプリント）

| バージョン | テスト数 | 増加 |
|---|---|---|
| v83.0.0（ベース） | 3,887 ※ drift 補正後 | — |
| v83.1.0 | 3,889 | +2 |
| v83.2.0 | 3,891 | +2 |
| v83.3.0 | 3,893 | +2 |
| v83.4.0 | 3,895 | +2 |
| v83.5.0 | 3,897 | +2 |
| v83.6.0 | 3,899 | +2 |
| v83.7.0 | 3,901 | +2 |
| v83.8.0 | 3,903 | +2 |
| v83.9.0 | 3,905 | +2 |
| v84.0.0（宣言） | 3,909 | +4 |

**本スプリント合計**: +22 tests（3,887 → 3,909）

---

## 参考リンク

- マスターロードマップ: [roadmap-v80.1-v85.0.md](roadmap-v80.1-v85.0.md)
- 前スプリント: [roadmap-v82.1-v83.0.md](roadmap-v82.1-v83.0.md)
- 次スプリント: [roadmap-v84.1-v85.0.md](roadmap-v84.1-v85.0.md)
- 達成宣言: `MILESTONE.md`
- 進行状況: `versions/current.md`
