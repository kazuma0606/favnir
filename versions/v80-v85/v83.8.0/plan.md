# v83.8.0 実装計画 — 健全性ダッシュボード（`HealthDashboard` / テキスト形式）

## 依存関係

既存型（`PipelineMetrics`, `AlertFiring`, `SloStatus`）を使用。新規型・関数のみ。既存コードへの変更なし。

## 実装ステップ

### Step 1: `test_framework.rs` に enum・構造体を追加

v83.7.0 追加ブロック（`cmd_observe` 末尾）の後に追加する。

1. `HealthStatus` enum（`#[derive(Debug, Clone, PartialEq)]`）
   - `Healthy`
   - `Degraded(String)` — 理由文字列
   - `Critical(String)` — 理由文字列

2. `PipelineHealth` 構造体（`#[derive(Debug, Clone, PartialEq)]`）
   - `pipeline_name: String`
   - `status: HealthStatus`
   - `alerts_firing: usize`
   - `slo_breached: bool`
   - `quality_score: f64`

3. `HealthDashboard` 構造体（`#[derive(Debug, Clone, PartialEq)]`）
   - `pipelines: Vec<PipelineHealth>`
   - `generated_at: String`

### Step 2: `compute_pipeline_health` / `format_health_dashboard` 関数を追加

```rust
pub fn compute_pipeline_health(
    metrics: &PipelineMetrics,
    alerts: &[AlertFiring],
    slo: &SloStatus,
    quality: f64,
) -> PipelineHealth {
    let alerts_firing = alerts.len();
    let slo_breached = slo.breached;
    let status = if slo_breached {
        HealthStatus::Critical(format!("SLO breached: {}", slo.target.name))
    } else if alerts_firing > 0 {
        HealthStatus::Degraded(format!("{} alert(s) firing", alerts_firing))
    } else {
        HealthStatus::Healthy
    };
    PipelineHealth {
        pipeline_name: metrics.pipeline_name.clone(),
        status,
        alerts_firing,
        slo_breached,
        quality_score: quality,
    }
}
```

```rust
pub fn format_health_dashboard(dashboard: &HealthDashboard) -> String {
    let mut lines = vec![
        "=== Health Dashboard ===".to_string(),
        format!("Generated: {}", dashboard.generated_at),
    ];
    if dashboard.pipelines.is_empty() {
        lines.push("No pipelines.".to_string());
    } else {
        for p in &dashboard.pipelines {
            let status_str = match &p.status {
                HealthStatus::Healthy => "Healthy".to_string(),
                HealthStatus::Degraded(reason) => format!("Degraded({})", reason),
                HealthStatus::Critical(reason) => format!("Critical({})", reason),
            };
            let slo_str = if p.slo_breached { "BREACHED" } else { "OK" };
            lines.push(format!(
                "Pipeline: {}  Status: {}  Alerts: {}  SLO: {}  Quality: {:.2}",
                p.pipeline_name, status_str, p.alerts_firing, slo_str, p.quality_score,
            ));
        }
    }
    lines.join("\n")
}
```

### Step 3: `driver.rs` に `v83800_tests` を追加

`v83700_tests` の直後に追加する。

```rust
#[cfg(test)]
mod v83800_tests {
    use fav_core::test_framework::*;

    #[test]
    fn health_dashboard_healthy_pipeline() { ... }

    #[test]
    fn health_dashboard_critical_pipeline() { ... }
}
```

`health_dashboard_healthy_pipeline`:
- `StageMetrics` 2件 → `compute_pipeline_metrics` で `PipelineMetrics`
- `SloTarget` + `SloMeasurement`（good=990, total=1000、objective=99.5% → 99% で未超過）→ `compute_slo_status`
- `compute_pipeline_health(&metrics, &[], &slo_status, 95.0)` → `HealthStatus::Healthy` を assert
- `HealthDashboard { pipelines: vec![health], generated_at: "..." }` 構築
- `format_health_dashboard(&dashboard)` が "=== Health Dashboard ===" を含むことを assert
- `format_health_dashboard(&dashboard)` が "Healthy" を含むことを assert

`health_dashboard_critical_pipeline`:
- `SloMeasurement`（good=970, total=1000、objective=99.5% → 97% で超過）→ `compute_slo_status`
- `AlertFiring` 1件
- `compute_pipeline_health(&metrics, &[alert], &slo_status, 60.0)` → `HealthStatus::Critical(...)` を assert
- `format_health_dashboard` が "Critical" を含むことを assert
- 空 pipelines の `HealthDashboard` で "No pipelines." を含むことを assert
- Degraded ケース追加アサート: SLO 未超過（good=995, total=1000）+ アラート 1件 →
  `compute_pipeline_health` が `HealthStatus::Degraded(...)` を返すことを assert

### Step 4: `CHANGELOG.md` 更新

先頭に v83.8.0 エントリを追加する。

### Step 5: `cargo test` で全テスト通過を確認

期待: 3903 tests pass（+2）、0 failures

### Step 6: CI チェック

- `cargo clippy --locked -- -D warnings` が pass することを確認
- `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認
- `./target/debug/fav fmt --check self/checker.fav` が pass することを確認
