# v83.8.0 タスクリスト

Status: COMPLETE

---

## T0: 事前確認

- [x] `cargo test` が 3,901 tests pass、0 failures であることを確認する（前提: v83.7.0 完了済み）

## T1: `test_framework.rs` に enum・構造体を追加

- [x] `HealthStatus` enum を追加する（`#[derive(Debug, Clone, PartialEq)]`）
  - `Healthy`, `Degraded(String)`, `Critical(String)`
- [x] `PipelineHealth` 構造体を追加する（`#[derive(Debug, Clone, PartialEq)]`）
  - `pipeline_name: String`, `status: HealthStatus`, `alerts_firing: usize`, `slo_breached: bool`, `quality_score: f64`
- [x] `HealthDashboard` 構造体を追加する（`#[derive(Debug, Clone, PartialEq)]`）
  - `pipelines: Vec<PipelineHealth>`, `generated_at: String`

## T2: `compute_pipeline_health` / `format_health_dashboard` 関数を追加

- [x] `compute_pipeline_health(metrics: &PipelineMetrics, alerts: &[AlertFiring], slo: &SloStatus, quality: f64) -> PipelineHealth` を追加する
  - `slo.breached == true` → `Critical("SLO breached: {slo.target.name}")`
  - `slo.breached == false && alerts.len() > 0` → `Degraded("{n} alert(s) firing")`
  - `slo.breached == false && alerts.len() == 0` → `Healthy`
  - `alerts_firing = alerts.len()`, `slo_breached = slo.breached`, `quality_score = quality`
- [x] `format_health_dashboard(dashboard: &HealthDashboard) -> String` を追加する
  - "=== Health Dashboard ===" ヘッダ + "Generated: {generated_at}"
  - 各パイプライン: "Pipeline: {name}  Status: {status}  Alerts: {n}  SLO: OK|BREACHED  Quality: {:.2}"
  - `pipelines` が空のとき "No pipelines." を出力

## T3: `driver.rs` に `v83800_tests` を追加

- [x] `v83700_tests` の直後に `#[cfg(test)] mod v83800_tests` を追加する
  - `health_dashboard_healthy_pipeline`: アラートなし・SLO 未超過 → `Healthy`、`format_health_dashboard` に "Healthy" / "=== Health Dashboard ===" が含まれることを確認
  - `health_dashboard_critical_pipeline`: SLO 超過 → `Critical(...)`、アラート 1件 → `alerts_firing=1`、`format_health_dashboard` に "Critical" が含まれることを確認、空 pipelines で "No pipelines." を確認、SLO 未超過＋アラートあり → `Degraded(...)` を確認（同テスト内の追加アサート）

## T4: `CHANGELOG.md` 更新

- [x] `CHANGELOG.md` の先頭に v83.8.0 エントリを追加する

## T5: テスト通過確認

- [x] `cargo test` が 3,903 tests pass（+2）、0 failures であることを確認する

## T6: 最終確認（CI チェック）

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する

## code-reviewer 対応

- [MED] `quality_score` 非有限値・範囲外値: `debug_assert!(quality.is_finite() && quality >= 0.0 && quality <= 100.0)` を `compute_pipeline_health` 先頭に追加
- [LOW] Critical「アラートゼロ」ケース未テスト: `health_dashboard_critical_pipeline` 末尾に `compute_pipeline_health(&metrics, &[], &slo_status, 50.0)` → `Critical` のアサートを追加
- [LOW] テスト関数の責務過多: ロードマップが2件指定のため構造変更は不要
