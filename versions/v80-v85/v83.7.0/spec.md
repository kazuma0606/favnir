# v83.7.0 仕様書 — `fav observe` コマンド（メトリクス・アラート統合）

## Background

v83.1〜v83.6 で PipelineMetrics / AlertRule / SloStatus / PerfBaseline など個別の Observability 型を整備した。
次のステップとして、これらを一括収集・評価するコマンド層 `fav observe` を追加する。

本バージョンは Observability 2.0 スプリント（v83.1〜v84.0）の第 7 段階。

ロードマップ参照: `versions/roadmap/roadmap-v83.1-v84.0.md` — v83.7.0 セクション

## Goals

1. `ObserveFormat` enum を追加する（`Text` / `Json`）
2. `ObserveOptions` 構造体を追加する
3. `ObserveReport` 構造体を追加する
4. `format_observe_report(report: &ObserveReport, format: &ObserveFormat) -> String` を追加する
5. `cmd_observe` 関数シグネチャ（構造体ベース）を追加する

## 型定義・API

```rust
/// `fav observe` コマンドの出力フォーマット。
#[derive(Debug, Clone, PartialEq)]
pub enum ObserveFormat {
    Text,
    Json,
}

/// `fav observe` コマンドのオプション。
#[derive(Debug, Clone, PartialEq)]
pub struct ObserveOptions {
    pub pipeline_name: String,
    pub format: ObserveFormat,
    pub show_alerts: bool,
    pub show_slo: bool,
}

/// `fav observe` コマンドの実行レポート。
#[derive(Debug, Clone, PartialEq)]
pub struct ObserveReport {
    pub metrics: PipelineMetrics,
    pub alerts: Vec<AlertFiring>,
    pub slo_statuses: Vec<SloStatus>,
}

/// `ObserveReport` をフォーマットして文字列として返す。
///
/// `ObserveFormat::Text` の場合:
/// - "=== Observe: {pipeline_name} ===" ヘッダ（`report.metrics.pipeline_name` を使用）
/// - メトリクスサマリー（`format_metrics_summary` 利用）
/// - alerts が空でない場合、各 AlertFiring の rule.name と fired_at を出力
/// - slo_statuses が空でない場合、各 SloStatus を `format_slo_status` で出力
/// - alerts が空のとき "Alert:" 行を含まない
/// - slo_statuses が空のとき "SLO:" 行を含まない
///
/// `ObserveFormat::Json` の場合:
/// - `{"pipeline":"<name>","alerts_count":<n>,"slo_count":<n>}` 形式の簡易 JSON
/// - `"pipeline"` キーには `report.metrics.pipeline_name` を使用する
/// - `show_alerts` / `show_slo` フラグは JSON 出力に影響しない（常に全カウントを出力）
pub fn format_observe_report(report: &ObserveReport, format: &ObserveFormat) -> String

/// `fav observe` コマンドハンドラ（CLIフラグ: --pipeline / --format / --alerts / --slo）。
/// テストフレームワーク層では構造体引数ベースで実装する。
///
/// `show_alerts` / `show_slo` フラグは CLI レイヤーのフィルタリング意図を示すものであり、
/// `format_observe_report` には影響しない。`cmd_observe` は `format_observe_report(report, &options.format)` を返す。
pub fn cmd_observe(options: &ObserveOptions, report: &ObserveReport) -> String
```

## Text フォーマット出力例

```
=== Observe: etl_main ===
Pipeline: etl_main
Total Duration: 450ms
Stages: 2
Alert: row_failure_spike fired at 2026-08-21T00:00:00Z
SLO: etl_slo
  Objective: 99.50%
  Current: 98.00%
  Error Budget Remaining: ...
  Status: BREACHED
```

## JSON フォーマット出力例

```json
{"pipeline":"etl_main","alerts_count":1,"slo_count":1}
```

## Success Criteria

- `cargo test` が 3901 tests pass（+2）、0 failures
- `ObserveReport` が `PipelineMetrics` / `Vec<AlertFiring>` / `Vec<SloStatus>` を保持する
- `format_observe_report` が `ObserveFormat::Text` で "=== Observe:" ヘッダを含む文字列を返す
- `format_observe_report` が `ObserveFormat::Json` で `{"pipeline":...}` 形式の文字列を返す
- alerts が空のとき Text 出力に "Alert:" 行が含まれない
- slo_statuses が空のとき Text 出力に "SLO:" 行が含まれない
- `cmd_observe(options, report)` が `format_observe_report(report, &options.format)` と等価な文字列を返す

## Files to Modify

- `fav/src/test_framework.rs` — 型定義・関数追加
- `fav/src/driver.rs` — `v83700_tests` モジュール追加
- `CHANGELOG.md` — v83.7.0 エントリ追加
