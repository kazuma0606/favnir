# v83.8.0 仕様書 — 健全性ダッシュボード（`HealthDashboard` / テキスト形式）

## Background

v83.1〜v83.7 で PipelineMetrics / AlertFiring / SloStatus / ObserveReport を整備した。
次のステップとして、パイプラインの総合健全性を一目で把握できるテキスト形式のダッシュボードを追加する。

本バージョンは Observability 2.0 スプリント（v83.1〜v84.0）の第 8 段階。

ロードマップ参照: `versions/roadmap/roadmap-v83.1-v84.0.md` — v83.8.0 セクション

## Goals

1. `HealthStatus` enum を追加する（`Healthy` / `Degraded(String)` / `Critical(String)`）
2. `PipelineHealth` 構造体を追加する
3. `HealthDashboard` 構造体を追加する
4. `compute_pipeline_health(metrics: &PipelineMetrics, alerts: &[AlertFiring], slo: &SloStatus, quality: f64) -> PipelineHealth` を追加する
5. `format_health_dashboard(dashboard: &HealthDashboard) -> String` を追加する

## 型定義・API

```rust
/// パイプラインの総合健全性ステータス。
#[derive(Debug, Clone, PartialEq)]
pub enum HealthStatus {
    Healthy,
    /// 劣化状態。String は理由（例: "1 alert(s) firing"）。
    Degraded(String),
    /// 重大障害。String は理由（例: "SLO breached: etl_slo"）。
    Critical(String),
}

/// 単一パイプラインの健全性レポート。
#[derive(Debug, Clone, PartialEq)]
pub struct PipelineHealth {
    pub pipeline_name: String,
    pub status: HealthStatus,
    pub alerts_firing: usize,
    pub slo_breached: bool,
    pub quality_score: f64,
}

/// 複数パイプラインの健全性ダッシュボード。
#[derive(Debug, Clone, PartialEq)]
pub struct HealthDashboard {
    pub pipelines: Vec<PipelineHealth>,
    pub generated_at: String,
}

/// パイプラインの総合健全性を算出する。
///
/// `pipeline_name` は `metrics.pipeline_name` から取得する。
///
/// ステータス判定ロジック:
/// - `slo.breached == true` → `Critical("SLO breached: {slo.target.name}")`
/// - `slo.breached == false && alerts.len() > 0` → `Degraded("{n} alert(s) firing")`
/// - `slo.breached == false && alerts.len() == 0` → `Healthy`
///
/// `alerts_firing = alerts.len()`、`slo_breached = slo.breached`、`quality_score = quality`。
pub fn compute_pipeline_health(
    metrics: &PipelineMetrics,
    alerts: &[AlertFiring],
    slo: &SloStatus,
    quality: f64,
) -> PipelineHealth

/// HealthDashboard をテキスト形式で返す。
///
/// 出力形式:
/// ```
/// === Health Dashboard ===
/// Generated: {generated_at}
/// Pipeline: {pipeline_name}  Status: {status}  Alerts: {alerts_firing}  SLO: {OK|BREACHED}  Quality: {quality_score:.2}
/// ...（pipelines が空のときは "No pipelines." を出力）
/// ```
pub fn format_health_dashboard(dashboard: &HealthDashboard) -> String
```

## HealthStatus 判定ロジック

```
if slo.breached:
    Critical("SLO breached: {slo.target.name}")
elif alerts.len() > 0:
    Degraded("{alerts.len()} alert(s) firing")
else:
    Healthy
```

## Success Criteria

- `cargo test` が 3903 tests pass（+2）、0 failures
- `compute_pipeline_health` が SLO 未超過・アラートなしのとき `HealthStatus::Healthy` を返す（`health_dashboard_healthy_pipeline` で検証）
- `compute_pipeline_health` が SLO 超過のとき `HealthStatus::Critical(...)` を返す（`health_dashboard_critical_pipeline` で検証）
- `compute_pipeline_health` が SLO 未超過・アラートありのとき `HealthStatus::Degraded(...)` を返す（`health_dashboard_critical_pipeline` 内の追加アサートで検証）
- `format_health_dashboard` が "=== Health Dashboard ===" ヘッダを含む文字列を返す（`health_dashboard_healthy_pipeline` で検証）
- `format_health_dashboard` が `pipelines` が空のとき "No pipelines." を出力する（`health_dashboard_critical_pipeline` 内の追加アサートで検証）

## Files to Modify

- `fav/src/test_framework.rs` — 型定義・関数追加
- `fav/src/driver.rs` — `v83800_tests` モジュール追加
- `CHANGELOG.md` — v83.8.0 エントリ追加
