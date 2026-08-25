# v84.5.0 タスクリスト

Status: COMPLETE

---

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、3,917 tests, 0 failures を確認する（前提: v84.4.0 完了済み）
- [x] `grep -m1 '^version' fav/Cargo.toml` の出力が `version = "84.0.0"` であることを確認する
  （v84.x マイナーバージョンは Cargo.toml を更新しない慣例。v85.0.0 宣言時に一括更新する）
- [x] `fav/src/driver.rs` に `mod v84400_tests` が存在することを確認する（v84.4.0 完了済みの証拠）

## T1: `pipeline.fav` に可観測性統合セクションを追加

- [x] 現在の `pipeline.fav` 末尾に `-- ── 可観測性統合セクション（Sprint 4: Observability 2.0）──────────────────` コメントを先頭に追加する
- [x] `showcase_pipeline_metrics` 関数を追加する
  - `StageMetrics { stage_name, duration_ms, rows_processed, rows_failed }` を構築（全フィールド必須）
  - `PipelineMetrics { pipeline_name, stages: List.of(stage), total_duration_ms, started_at }` を構築
  - `bind` 構文を使用する
- [x] `showcase_health_dashboard` 関数を追加する
  - `AlertRule { name, threshold: AlertThreshold { metric, operator, value }, severity, message }` を構築
  - `evaluate_alert_rules(List.of(rule), metrics, fired_at)` を呼ぶ
  - `SloTarget { name, objective_pct, window_hours }` を構築
  - `SloMeasurement { good_events, total_events, window_hours }` を構築
  - `compute_slo_status(slo_tgt, slo_meas)` で SloStatus を取得
  - `compute_pipeline_health(metrics, alerts, slo, 0.95)` で PipelineHealth を取得
  - `HealthDashboard { pipelines: List.of(health), generated_at }` を構築
  - `format_health_dashboard(dashboard)` で String を返す
- [x] spec.md の型定義テーブルで全フィールドが揃っているか確認してから実装する（`registered_at` 漏れの再発防止）

## T2: `fav/src/driver.rs` に `v84500_tests` を追加

- [x] `mod v84400_tests { ... }` の直後に `#[cfg(test)] mod v84500_tests { ... }` を追加する
  - `include_str!` は `"../../infra/..."` 形式（パス起点: `fav/src/driver.rs`）
- [x] `showcase_observe_metrics_collected` テストを実装する
  - pipeline.fav に `"PipelineMetrics"` が含まれること（メッセージ付き）
  - pipeline.fav に `"AlertRule"` が含まれること（メッセージ付き）
- [x] `showcase_health_dashboard_generated` テストを実装する
  - pipeline.fav に `"HealthDashboard"` が含まれること（メッセージ付き）
  - pipeline.fav に `"format_health_dashboard"` が含まれること（メッセージ付き）

## T3: テスト通過確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、3,919 tests, 0 failures（+2）であることを確認する

## T4: CHANGELOG 更新

- [x] `CHANGELOG.md` の先頭に v84.5.0 エントリを追加する

> 注: 本バージョンは `pipeline.fav` 更新とテスト追加のみ。`site/` MDX 追加は v84.6.0 で実施する。

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
