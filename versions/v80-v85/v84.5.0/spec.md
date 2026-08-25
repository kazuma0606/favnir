# spec: v84.5.0 — 可観測性統合ショーケース（`fav observe` E2E）

## Background

> **テスト数注記**: ロードマップ計画値は 3,905/3,907 だったが、code-reviewer 対応の
> 累積により実際のベースは **3,917 tests**（v84.4.0 完了時点）。
> v84.5.0 完了目標は **3,919 tests**（+2）。

v84.4.0 でショーケースに Pipeline Contracts 1.0（SlaContract / ContractDependency /
ContractRegistry）を統合した。v84.5.0 では Sprint 4「Observability 2.0」の機能
（PipelineMetrics / AlertRule / SloTarget / HealthDashboard）を `pipeline.fav` に統合し、
ショーケースが `fav observe` 出力を示すことを確認する。

## Goals

1. `infra/e2e-demo/favnir4-showcase/pipeline.fav` に可観測性統合セクションを追加する
   - `PipelineMetrics` + `StageMetrics` によるステージ単位統計収集
   - `AlertRule` + `evaluate_alert_rules` によるアラート評価
   - `SloTarget` + `SloMeasurement` + `compute_slo_status` による SLO 判定
   - `HealthDashboard` + `format_health_dashboard` による統合出力
2. Rust テスト 2 件でショーケースの内容を検証する
   - `showcase_observe_metrics_collected` — PipelineMetrics / AlertRule の存在確認
   - `showcase_health_dashboard_generated` — HealthDashboard / format_health_dashboard の存在確認

## Syntax / API Examples（実際の型定義に基づく）

### pipeline.fav への追加セクション

```favnir
-- ── 可観測性統合セクション（Sprint 4: Observability 2.0）──────────────────

fn showcase_pipeline_metrics(ctx: AppCtx) -> PipelineMetrics {
    -- PipelineMetrics: ステージ単位の実行統計を収集（Sprint 4 v83.1.0）
    bind stage <- StageMetrics {
        stage_name: "load_stage",
        duration_ms: 42,
        rows_processed: 1000,
        rows_failed: 0,
    }
    PipelineMetrics {
        pipeline_name: "favnir4-showcase",
        stages: List.of(stage),
        total_duration_ms: 42,
        started_at: "2026-08-22T00:00:00Z",
    }
}

fn showcase_health_dashboard(ctx: AppCtx) -> Result<String, String> {
    -- HealthDashboard: AlertRule + SloTarget + format_health_dashboard で統合出力
    bind rule <- AlertRule {
        name: "latency_alert",
        threshold: AlertThreshold {
            metric: "total_duration_ms",
            operator: ThresholdOp.GreaterThan,
            value: 5000.0,
        },
        severity: AlertSeverity.Warning,
        message: "Pipeline latency exceeded threshold",
    }
    bind metrics  <- showcase_pipeline_metrics(ctx)
    bind alerts   <- evaluate_alert_rules(List.of(rule), metrics, "2026-08-22T00:00:00Z")
    bind slo_tgt  <- SloTarget { name: "showcase-slo", objective_pct: 99.0, window_hours: 24 }
    bind slo_meas <- SloMeasurement { good_events: 990, total_events: 1000, window_hours: 24 }
    bind slo      <- compute_slo_status(slo_tgt, slo_meas)
    bind health   <- compute_pipeline_health(metrics, alerts, slo, 0.95)
    bind dashboard <- HealthDashboard {
        pipelines: List.of(health),
        generated_at: "2026-08-22T00:00:00Z",
    }
    Result.ok(format_health_dashboard(dashboard))
}
```

### v84500_tests（Rust テスト）

```rust
#[cfg(test)]
mod v84500_tests {
    #[test]
    fn showcase_observe_metrics_collected() {
        let content = include_str!("../../infra/e2e-demo/favnir4-showcase/pipeline.fav");
        assert!(content.contains("PipelineMetrics"), "pipeline.fav should include PipelineMetrics");
        assert!(content.contains("AlertRule"),       "pipeline.fav should include AlertRule");
    }

    #[test]
    fn showcase_health_dashboard_generated() {
        let content = include_str!("../../infra/e2e-demo/favnir4-showcase/pipeline.fav");
        assert!(content.contains("HealthDashboard"),       "pipeline.fav should include HealthDashboard");
        assert!(content.contains("format_health_dashboard"), "pipeline.fav should include format_health_dashboard");
    }
}
```

## 実際の型定義（参照）

Rust シグネチャ。Favnir 側では `Vec<T>` → `List<T>`、`usize`/`u64` → `Int`、`f64` → `Float` に読み替える。

| 型 / 関数 | Rust シグネチャ |
|---|---|
| `StageMetrics` | `stage_name: String`, `duration_ms: u64`, `rows_processed: usize`, `rows_failed: usize` |
| `PipelineMetrics` | `pipeline_name: String`, `stages: Vec<StageMetrics>`, `total_duration_ms: u64`, `started_at: String` |
| `AlertThreshold` | `metric: String`, `operator: ThresholdOp`, `value: f64` |
| `AlertRule` | `name: String`, `threshold: AlertThreshold`, `severity: AlertSeverity`, `message: String` |
| `evaluate_alert_rules` | `(rules: &[AlertRule], metrics: &PipelineMetrics, fired_at: &str) -> Vec<AlertFiring>` |
| `SloTarget` | `name: String`, `objective_pct: f64`, `window_hours: u64` |
| `SloMeasurement` | `good_events: u64`, `total_events: u64`, `window_hours: u64` |
| `compute_slo_status` | `(target: &SloTarget, measurement: &SloMeasurement) -> SloStatus` |
| `compute_pipeline_health` | `(metrics: &PipelineMetrics, alerts: &[AlertFiring], slo: &SloStatus, quality: f64) -> PipelineHealth` |
| `HealthDashboard` | `pipelines: Vec<PipelineHealth>`, `generated_at: String` |
| `format_health_dashboard` | `(dashboard: &HealthDashboard) -> String` |
| `SloStatus` | `target: SloTarget`, `measurement: SloMeasurement`, `actual_pct: f64`, `is_met: bool` |
| `PipelineHealth` | `metrics: PipelineMetrics`, `alerts: Vec<AlertFiring>`, `slo: SloStatus`, `quality: f64`, `is_healthy: bool` |
| `ThresholdOp` | `GreaterThan` / `LessThan` / `EqualTo` |
| `AlertSeverity` | `Critical` / `Warning` / `Info` |

## Success Criteria

- `infra/e2e-demo/favnir4-showcase/pipeline.fav` に `PipelineMetrics`・`AlertRule`・
  `HealthDashboard`・`format_health_dashboard` の各識別子が含まれること
- spec.md の型定義テーブルに全フィールドが列挙されており、実装前に照合すること
  （`StageMetrics`・`PipelineMetrics`・`AlertThreshold`・`AlertRule`・`SloTarget`・
  `SloMeasurement`・`SloStatus`・`PipelineHealth`・`HealthDashboard` の全フィールド）
- `cargo test` が 3,919 tests pass（+2）、0 failures であること

## Error Codes

なし（本バージョンはファイル更新のみ）

## Files to Modify / Create

### 更新
- `infra/e2e-demo/favnir4-showcase/pipeline.fav` — 可観測性統合セクションを末尾に追加

### 追記
- `fav/src/driver.rs` — `v84500_tests` モジュール追加（2 テスト）
- `CHANGELOG.md` — v84.5.0 エントリ追加

### パス起点（v84.1.0 から踏襲）

`v84500_tests` は `include_str!("../../infra/...")` を使用。
パス起点は `fav/src/driver.rs`（`fav/src/`）。`driver.rs` 移動時はパスを更新すること。

> 注: `site/` MDX 追加は v84.6.0 で一括実施するため本バージョンでは省略する。
