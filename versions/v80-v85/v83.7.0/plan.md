# v83.7.0 実装計画 — `fav observe` コマンド（メトリクス・アラート統合）

## 依存関係

既存型（`PipelineMetrics`, `AlertFiring`, `SloStatus`）を使用。新規型・関数のみ。既存コードへの変更なし。

## 実装ステップ

### Step 1: `test_framework.rs` に enum・構造体・関数を追加

v83.6.0 追加ブロック（`format_regression_report` 末尾）の後に追加する。

1. `ObserveFormat` enum（`#[derive(Debug, Clone, PartialEq)]`）
   - `Text`
   - `Json`

2. `ObserveOptions` 構造体（`#[derive(Debug, Clone, PartialEq)]`）
   - `pipeline_name: String`
   - `format: ObserveFormat`
   - `show_alerts: bool`
   - `show_slo: bool`

3. `ObserveReport` 構造体（`#[derive(Debug, Clone, PartialEq)]`）
   - `metrics: PipelineMetrics`
   - `alerts: Vec<AlertFiring>`
   - `slo_statuses: Vec<SloStatus>`

### Step 2: `format_observe_report` / `cmd_observe` 関数を追加

```rust
pub fn format_observe_report(report: &ObserveReport, format: &ObserveFormat) -> String {
    match format {
        ObserveFormat::Text => {
            let mut lines = vec![
                format!("=== Observe: {} ===", report.metrics.pipeline_name),
            ];
            lines.push(format_metrics_summary(&report.metrics));
            for alert in &report.alerts {
                lines.push(format!("Alert: {} fired at {}", alert.rule.name, alert.fired_at));
            }
            for slo in &report.slo_statuses {
                lines.push(format_slo_status(slo));
            }
            lines.join("\n")
        }
        ObserveFormat::Json => {
            format!(
                r#"{{"pipeline":"{}","alerts_count":{},"slo_count":{}}}"#,
                report.metrics.pipeline_name,
                report.alerts.len(),
                report.slo_statuses.len(),
            )
        }
    }
}
```

```rust
pub fn cmd_observe(options: &ObserveOptions, report: &ObserveReport) -> String {
    format_observe_report(report, &options.format)
}
```

### Step 3: `driver.rs` に `v83700_tests` を追加

`v83600_tests` の直後に追加する。

```rust
#[cfg(test)]
mod v83700_tests {
    use fav_core::test_framework::*;

    #[test]
    fn observe_report_built() { ... }

    #[test]
    fn observe_report_text_format() { ... }
}
```

`observe_report_built`:
- `StageMetrics` 2件 → `compute_pipeline_metrics` で `PipelineMetrics` 作成
- `AlertFiring` 1件（AlertRule + current_value + fired_at）
- `SloTarget` + `SloMeasurement` → `compute_slo_status` で `SloStatus` 1件
- `ObserveReport { metrics, alerts, slo_statuses }` 構築
- `report.alerts.len() == 1` / `report.slo_statuses.len() == 1` を assert

`observe_report_text_format`:
- 上記と同じ report を `format_observe_report(&report, &ObserveFormat::Text)` で変換
- "=== Observe:" が含まれることを assert
- alerts が空のとき "Alert:" 行が含まれないことを assert（alerts を空にした別 report で確認）
- JSON フォーマットのスモークテスト（`{"pipeline":` が含まれる）

### Step 4: `CHANGELOG.md` 更新

先頭に v83.7.0 エントリを追加する。

### Step 5: `cargo test` で全テスト通過を確認

期待: 3901 tests pass（+2）、0 failures

### Step 6: CI チェック

- `cargo clippy --locked -- -D warnings` が pass することを確認
- `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認
- `./target/debug/fav fmt --check self/checker.fav` が pass することを確認
