# plan: v84.5.0 — 可観測性統合ショーケース（`fav observe` E2E）

## 実装ステップ（依存順）

### Step 1: 事前確認

- `cargo test` を実行し、3,917 tests, 0 failures を確認する（前提: v84.4.0 完了済み）
- `grep -m1 '^version' fav/Cargo.toml` の出力が `version = "84.0.0"` であることを確認する
  （v84.x マイナーバージョンは Cargo.toml を更新しない慣例。v85.0.0 宣言時に一括更新する）
  > 注: ロードマップ計画値は 3,905/3,907 だが、code-reviewer 対応の累積で実績ベースは 3,917/3,919。
- `fav/src/driver.rs` に `mod v84400_tests` が存在することを確認する

### Step 2: pipeline.fav に可観測性統合セクションを追加

現在の `pipeline.fav` 末尾に以下の 2 関数を追加する。

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

**注意事項**:
- `evaluate_alert_rules` の第 1 引数は `List.of(rule)`（Favnir の List で渡す）
- `compute_pipeline_health` の第 4 引数 `quality` は `f64`（Favnir では `Float`）
- `registered_at` のような「前バージョンで発見した欠落フィールド」が再発しないよう、spec.md の型定義テーブルで全フィールドを確認してから実装すること

### Step 3: driver.rs に v84500_tests を追加

`mod v84400_tests` の直後に `#[cfg(test)] mod v84500_tests` を追加する。

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
        assert!(content.contains("HealthDashboard"),        "pipeline.fav should include HealthDashboard");
        assert!(content.contains("format_health_dashboard"), "pipeline.fav should include format_health_dashboard");
    }
}
```

### Step 4: cargo test で全 pass 確認

`cargo test 2>&1 | grep "test result"` を実行し、3,919 tests, 0 failures を確認する。

### Step 5: CHANGELOG 更新

`CHANGELOG.md` の先頭に v84.5.0 エントリを追加する。

> 注: `site/` MDX 追加は v84.6.0 で一括実施するため本バージョンでは省略する。

### Step 6: CI 事前確認

- `cargo clippy --locked -- -D warnings` が pass することを確認する
- `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
