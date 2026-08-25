# v83.9.0 実装計画 — 安定化・コードフリーズ

## 依存関係

新規型・関数の追加なし。`driver.rs` にテストモジュールを追加するのみ。

## 実装ステップ

### Step 1: `driver.rs` に `v83900_tests` を追加

`v83800_tests` の直後に追加する。

```rust
#[cfg(test)]
mod v83900_tests {
    use fav_core::test_framework::*;

    #[test]
    fn observability_full_sprint_all_stable() {
        // 1. StageMetrics → PipelineMetrics
        let stages = vec![
            StageMetrics { stage_name: "load".into(), duration_ms: 300, rows_processed: 1000, rows_failed: 0 },
            StageMetrics { stage_name: "transform".into(), duration_ms: 600, rows_processed: 1000, rows_failed: 0 },
        ];
        let metrics = compute_pipeline_metrics("sprint_test", stages, "2026-08-21T00:00:00Z");

        // 2. AlertRule → evaluate_alert_rules（transform duration 600ms > 500ms → 発火）
        let rules = vec![AlertRule {
            name: "slow_stage".into(),
            threshold: AlertThreshold { metric: "total_duration_ms".into(), operator: ThresholdOp::GreaterThan, value: 500.0 },
            severity: AlertSeverity::Warning,
            message: "Pipeline too slow".into(),
        }];
        let alerts = evaluate_alert_rules(&rules, &metrics, "2026-08-21T00:00:00Z");
        assert!(!alerts.is_empty(), "alert should fire for slow pipeline");

        // 3. SloTarget + SloMeasurement → SloStatus（未超過）
        let slo = compute_slo_status(
            &SloTarget { name: "sprint_slo".into(), objective_pct: 99.0, window_hours: 24 },
            &SloMeasurement { good_events: 995, total_events: 1000, window_hours: 24 },
        );
        assert!(!slo.breached, "slo should not be breached");

        // 4. compute_pipeline_health → Degraded（アラートあり・SLO OK）
        let health = compute_pipeline_health(&metrics, &alerts, &slo, 80.0);
        match &health.status {
            HealthStatus::Degraded(_) => {}
            other => panic!("expected Degraded, got {:?}", other),
        }

        // 5. ObserveReport → format_observe_report
        let report = ObserveReport {
            metrics: metrics.clone(),
            alerts: alerts.clone(),
            slo_statuses: vec![slo.clone()],
        };
        let text = format_observe_report(&report, &ObserveFormat::Text);
        assert!(text.contains("=== Observe:"), "observe report should have header");

        // 6. cmd_observe E2E: ObserveOptions + ObserveReport → format_observe_report と等価
        let options = ObserveOptions {
            pipeline_name: "sprint_test".into(),
            format: ObserveFormat::Text,
            show_alerts: true,
            show_slo: true,
        };
        let cmd_text = cmd_observe(&options, &report);
        assert_eq!(cmd_text, text, "cmd_observe should equal format_observe_report");

        // 7. PerfBaseline → detect_perf_regression（閾値以内 → None）
        let samples = vec![100u64, 150, 200, 250, 300];
        let baseline = PerfBaseline::from_samples("sprint_test", &samples);
        let reg = detect_perf_regression(&baseline, 310, 20.0);
        // p95 = sorted[4] = 300, regression = (310-300)/300*100 = 3.3% < 20% → None
        assert!(reg.is_none(), "small regression should not trigger alert");
    }

    #[test]
    fn health_dashboard_and_alerts_integrated() {
        // 1. PipelineMetrics（rows_failed=10 のステージを含む）
        let stages = vec![
            StageMetrics { stage_name: "load".into(), duration_ms: 200, rows_processed: 1000, rows_failed: 10 },
            StageMetrics { stage_name: "transform".into(), duration_ms: 250, rows_processed: 990, rows_failed: 0 },
        ];
        let metrics = compute_pipeline_metrics("int_test", stages, "2026-08-21T00:00:00Z");

        // 2. AlertRule（rows_failed > 5）→ evaluate_alert_rules → AlertFiring 1件
        let rules = vec![AlertRule {
            name: "row_failure_alert".into(),
            threshold: AlertThreshold { metric: "rows_failed".into(), operator: ThresholdOp::GreaterThan, value: 5.0 },
            severity: AlertSeverity::Critical,
            message: "Too many row failures".into(),
        }];
        let alerts = evaluate_alert_rules(&rules, &metrics, "2026-08-21T00:00:00Z");
        assert_eq!(alerts.len(), 1, "should fire 1 alert for rows_failed=10");

        // 3. SloStatus（breached=true）
        let slo = compute_slo_status(
            &SloTarget { name: "int_slo".into(), objective_pct: 99.5, window_hours: 24 },
            &SloMeasurement { good_events: 970, total_events: 1000, window_hours: 24 },
        );
        assert!(slo.breached, "slo should be breached");

        // 4. compute_pipeline_health → Critical
        let health = compute_pipeline_health(&metrics, &alerts, &slo, 60.0);
        match &health.status {
            HealthStatus::Critical(_) => {}
            other => panic!("expected Critical, got {:?}", other),
        }

        // 5. HealthDashboard → format_health_dashboard
        let dashboard = HealthDashboard {
            pipelines: vec![health],
            generated_at: "2026-08-21T00:00:00Z".into(),
        };
        let text = format_health_dashboard(&dashboard);
        assert!(text.contains("=== Health Dashboard ==="), "dashboard should have header");
        assert!(text.contains("Critical"), "dashboard should show Critical status");
    }
}
```

### Step 2: `CHANGELOG.md` 更新

先頭に v83.9.0 エントリを追加する。

### Step 3: `cargo test` で全テスト通過を確認

期待: 3905 tests pass（+2）、0 failures

### Step 4: CI チェック

- `cargo clippy --locked -- -D warnings` が pass することを確認
- `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認
- `./target/debug/fav fmt --check self/checker.fav` が pass することを確認
