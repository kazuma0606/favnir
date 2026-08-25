# Spec: v81.8.0 — 異常検知（`AnomalyDetector` / Z スコアベース）

## Background

v81.2.0 で追加した `DistributionStats` / `compute_distribution_stats` を活用し、
時系列・バッチ間の異常値を Z スコアで検出する **異常検知層** を追加する。

## Goals

- `AnomalyDetector` 構造体（`baseline_stats: DistributionStats`, `z_threshold: f64`）を追加する
- `AnomalyResult` 構造体（`is_anomaly: bool`, `z_score: f64`, `value: f64`）を追加する
- `AnomalyDetector::from_baseline(values, z_threshold) -> AnomalyDetector` を追加する
- `detect_anomaly(detector, value) -> AnomalyResult` を追加する
- `scan_for_anomalies(detector, values) -> Vec<AnomalyResult>` を追加する
- `format_anomaly_report(results) -> String` を追加する

## API

```rust
/// ベースライン統計と閾値を保持する異常検知器。
#[derive(Debug, Clone)]
pub struct AnomalyDetector {
    pub baseline_stats: DistributionStats,
    pub z_threshold: f64,
}

/// 単一値の異常検知結果。
#[derive(Debug, Clone)]
pub struct AnomalyResult {
    pub is_anomaly: bool,
    pub z_score: f64,
    pub value: f64,
}

impl AnomalyDetector {
    /// `values` から `compute_distribution_stats` でベースラインを構築する。
    pub fn from_baseline(values: &[f64], z_threshold: f64) -> AnomalyDetector
}

/// `detector` のベースラインに対して `value` の Z スコアを計算し、閾値と比較する。
///
/// Z スコア: |value - mean| / std_dev
/// std_dev == 0 のとき z_score = 0.0, is_anomaly = false（ゼロ除算ガード）。
pub fn detect_anomaly(detector: &AnomalyDetector, value: f64) -> AnomalyResult

/// `values` の各要素に `detect_anomaly` を適用し、結果をすべて返す。
pub fn scan_for_anomalies(detector: &AnomalyDetector, values: &[f64]) -> Vec<AnomalyResult>

/// `results` の集計サマリーを文字列に変換する。
///
/// フォーマット: `"anomaly_report total={n} anomalies={k}"`
pub fn format_anomaly_report(results: &[AnomalyResult]) -> String
```

## 出力例

```text
// detect_anomaly: baseline=[1,2,3,2,2], z_threshold=2.0, value=10.0
// mean=2.0, std_dev≈0.632, z≈12.66 → is_anomaly=true

// format_anomaly_report([is_anomaly=false, is_anomaly=true])
// → "anomaly_report total=2 anomalies=1"
```

## 実装注意点

- `std_dev == 0.0` のとき `z_score = 0.0`, `is_anomaly = false`（除算ガード必須）
  - `compute_distribution_stats` は全同値スライス（例: `[5.0, 5.0, 5.0]`）に対して厳密ゼロ `0.0` を返す（浮動小数点誤差が生じない入力を前提とする）
- `from_baseline` は空スライスを受け取ってもパニックしない（`compute_distribution_stats` に委ねる）
- Z スコアの計算: `(value - mean).abs() / std_dev`
- `is_anomaly = z_score > z_threshold`（`>=` ではなく `>`）

## Success Criteria

- `cargo test` 3859 tests, 0 failures（3857 + 2）
- `anomaly_detector_catches_outlier`: ベースライン値から大きく外れた値が `is_anomaly=true`、近い値が `is_anomaly=false` になることを確認
- `anomaly_scan_returns_all_results`: `scan_for_anomalies` が全要素の結果を返し、外れ値のみ `is_anomaly=true` になることを確認

## Files to Modify

| ファイル | 変更内容 |
|---|---|
| `fav/src/test_framework.rs` | `AnomalyDetector` / `AnomalyResult` / `from_baseline` / `detect_anomaly` / `scan_for_anomalies` / `format_anomaly_report` 追加 |
| `fav/src/driver.rs` | `mod v81800_tests` 追加（2 テスト） |
| `CHANGELOG.md` | v81.8.0 エントリ追加 |
