# v83.2.0 仕様書 — `AlertRule` / `AlertThreshold`（アラート型）

## Background

v83.1.0 で `PipelineMetrics` が導入された。次のステップとして、
メトリクスの閾値超過をアラートとして型で定義し、`PipelineMetrics` を評価して
発火した `AlertFiring` を返す仕組みを整備する。

本バージョンは Observability 2.0 スプリント（v83.1〜v84.0）の第 2 段階。

ロードマップ参照: `versions/roadmap/roadmap-v83.1-v84.0.md` — v83.2.0 セクション

## Goals

1. `AlertSeverity` enum を追加する（`Critical` / `Warning` / `Info`）
2. `ThresholdOp` enum を追加する（`GreaterThan` / `LessThan` / `EqualTo`）
3. `AlertThreshold` 構造体を追加する
4. `AlertRule` 構造体を追加する
5. `AlertFiring` 構造体を追加する
6. `evaluate_alert_rules` 関数を追加する

## 型定義・API

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum AlertSeverity {
    Critical,
    Warning,
    Info,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ThresholdOp {
    GreaterThan,
    LessThan,
    EqualTo,
}

/// メトリクスの閾値定義。
/// `metric` は `PipelineMetrics` のフィールドに対応する文字列
/// （`"total_duration_ms"` / `"rows_failed"` / `"rows_processed"` 等）
#[derive(Debug, Clone, PartialEq)]
pub struct AlertThreshold {
    pub metric: String,
    pub operator: ThresholdOp,
    pub value: f64,
}

/// アラートルール定義。
#[derive(Debug, Clone, PartialEq)]
pub struct AlertRule {
    pub name: String,
    pub threshold: AlertThreshold,
    pub severity: AlertSeverity,
    pub message: String,
}

/// 発火したアラート。
#[derive(Debug, Clone, PartialEq)]
pub struct AlertFiring {
    pub rule: AlertRule,
    pub current_value: f64,
    pub fired_at: String,  // RFC 3339 / ISO 8601 形式
}

/// ルール一覧を `PipelineMetrics` に対して評価し、発火したアラートを返す。
///
/// `metric` の対応:
/// - `"total_duration_ms"` → `metrics.total_duration_ms as f64`
/// - `"rows_failed"` → 全ステージの `rows_failed` 合計
/// - `"rows_processed"` → 全ステージの `rows_processed` 合計
/// - 上記以外の `metric` 文字列は評価をスキップ（アラートを発火しない）
///
/// `fired_at` は呼び出し元が渡す文字列（テスト容易性のため）
pub fn evaluate_alert_rules(
    rules: &[AlertRule],
    metrics: &PipelineMetrics,
    fired_at: &str,
) -> Vec<AlertFiring>
```

## テスト（v83.2.0 で追加）

実際のテスト数ベース（※ drift 補正後）: **3889 + 2 = 3891**

（ロードマップ記載値 3877 + 2 = 3879 は旧バージョン到達時点のドリフト。
 実際の v83.1.0 完了テスト数は 3889。）

### `alert_fires_when_threshold_exceeded`

```rust
let rule = AlertRule {
    name: "slow_pipeline".into(),
    threshold: AlertThreshold {
        metric: "total_duration_ms".into(),
        operator: ThresholdOp::GreaterThan,
        value: 300.0,
    },
    severity: AlertSeverity::Warning,
    message: "Pipeline took too long".into(),
};
// total_duration_ms = 350 (> 300) → 発火
let stages = vec![
    StageMetrics { stage_name: "load".into(), duration_ms: 200, rows_processed: 1000, rows_failed: 0 },
    StageMetrics { stage_name: "transform".into(), duration_ms: 150, rows_processed: 1000, rows_failed: 0 },
];
let metrics = compute_pipeline_metrics("etl", stages, "2026-08-21T00:00:00Z");
let firings = evaluate_alert_rules(&[rule], &metrics, "2026-08-21T00:01:00Z");
assert_eq!(firings.len(), 1);
assert_eq!(firings[0].rule.name, "slow_pipeline");
assert_eq!(firings[0].current_value, 350.0);
```

### `alert_silent_when_within_threshold`

```rust
let rule = AlertRule {
    name: "slow_pipeline".into(),
    threshold: AlertThreshold {
        metric: "total_duration_ms".into(),
        operator: ThresholdOp::GreaterThan,
        value: 500.0,
    },
    severity: AlertSeverity::Critical,
    message: "Very slow".into(),
};
// total_duration_ms = 350 (≤ 500) → 発火しない
let stages = vec![
    StageMetrics { stage_name: "load".into(), duration_ms: 200, rows_processed: 1000, rows_failed: 0 },
    StageMetrics { stage_name: "transform".into(), duration_ms: 150, rows_processed: 1000, rows_failed: 0 },
];
let metrics = compute_pipeline_metrics("etl", stages, "2026-08-21T00:00:00Z");
let firings = evaluate_alert_rules(&[rule], &metrics, "2026-08-21T00:01:00Z");
assert!(firings.is_empty(), "threshold not exceeded, no alerts should fire");
```

## Success Criteria

- `cargo test` が 3891 tests pass（+2）、0 failures
- `GreaterThan` を使うアラートが閾値超過で発火し、閾値以内では発火しないことを 2 件のテストで確認
- 実装仕様（テストで担保しないが実装必須）:
  - `LessThan` / `EqualTo` も `ThresholdOp` として実装済みであること
  - 未知の `metric` 文字列はスキップされること（アラートを発火しない）

## Files to Modify

- `fav/src/test_framework.rs` — 型定義・関数追加
- `fav/src/driver.rs` — `v83200_tests` モジュール追加
