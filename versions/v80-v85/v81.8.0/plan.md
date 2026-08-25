# Plan: v81.8.0 — 異常検知（`AnomalyDetector` / Z スコアベース）

## Step 1: 前提確認

- `cargo test` を実行し、3857 tests, 0 failures を確認する
- `fav/src/test_framework.rs` に v81.2.0 の `DistributionStats` / `compute_distribution_stats` が定義済みであることを確認する
  - `DistributionStats` フィールド: `mean: f64`, `std_dev: f64`, `min: f64`, `max: f64`, `count: usize`

## Step 2: CHANGELOG 更新

`CHANGELOG.md` の先頭に v81.8.0 エントリを追加する。

## Step 3: `fav/src/test_framework.rs` に追記

`cmd_quality_report` の定義の直後（v81.7.0 セクション末尾）に以下を追加する。

```rust
// ── v81.8.0: AnomalyDetector / detect_anomaly ────────────────────────────────

/// ベースライン統計と Z スコア閾値を保持する異常検知器。
///
/// 依存: v81.2.0 の `DistributionStats` / `compute_distribution_stats`
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
    pub fn from_baseline(values: &[f64], z_threshold: f64) -> AnomalyDetector {
        AnomalyDetector {
            baseline_stats: compute_distribution_stats(values),
            z_threshold,
        }
    }
}

/// `detector` のベースラインに対して `value` の Z スコアを計算し、閾値と比較する。
///
/// - Z スコア: `|value - mean| / std_dev`
/// - `std_dev == 0.0` のとき `z_score = 0.0`, `is_anomaly = false`（ゼロ除算ガード）
/// - `is_anomaly = z_score > z_threshold`
pub fn detect_anomaly(detector: &AnomalyDetector, value: f64) -> AnomalyResult {
    let mean = detector.baseline_stats.mean;
    let std_dev = detector.baseline_stats.std_dev;
    let z_score = if std_dev == 0.0 {
        0.0
    } else {
        (value - mean).abs() / std_dev
    };
    AnomalyResult {
        is_anomaly: z_score > detector.z_threshold,
        z_score,
        value,
    }
}

/// `values` の各要素に `detect_anomaly` を適用し、結果をすべて返す。
pub fn scan_for_anomalies(detector: &AnomalyDetector, values: &[f64]) -> Vec<AnomalyResult> {
    values.iter().map(|&v| detect_anomaly(detector, v)).collect()
}

/// `results` の集計サマリーを文字列に変換する。
///
/// フォーマット: `"anomaly_report total={n} anomalies={k}"`
pub fn format_anomaly_report(results: &[AnomalyResult]) -> String {
    let total = results.len();
    let anomalies = results.iter().filter(|r| r.is_anomaly).count();
    format!("anomaly_report total={total} anomalies={anomalies}")
}
```

## Step 4: `fav/src/driver.rs` に `mod v81800_tests` を追加

`mod v81700_tests { ... }` の直後に追加する。

```rust
#[cfg(test)]
mod v81800_tests {
    use fav_core::test_framework::*;

    #[test]
    fn anomaly_detector_catches_outlier() {
        // baseline: mean=2.0, std_dev=sqrt(0.4)≈0.632
        let baseline = vec![1.0, 2.0, 3.0, 2.0, 2.0];
        let detector = AnomalyDetector::from_baseline(&baseline, 2.0);

        // 外れ値: z = |10.0 - 2.0| / 0.632 ≈ 12.66 > 2.0 → is_anomaly=true
        let result_outlier = detect_anomaly(&detector, 10.0);
        assert!(result_outlier.is_anomaly, "10.0 should be anomaly: z={}", result_outlier.z_score);
        assert!(result_outlier.z_score > 2.0, "z_score should exceed threshold: {}", result_outlier.z_score);

        // 正常値: z = |2.0 - 2.0| / 0.632 = 0.0 → is_anomaly=false
        let result_normal = detect_anomaly(&detector, 2.0);
        assert!(!result_normal.is_anomaly, "2.0 should not be anomaly: z={}", result_normal.z_score);

        // std_dev=0 ガード: 全要素が同値のとき z=0, is_anomaly=false
        let flat = vec![5.0, 5.0, 5.0];
        let detector_flat = AnomalyDetector::from_baseline(&flat, 1.0);
        let result_flat = detect_anomaly(&detector_flat, 5.0);
        assert!(!result_flat.is_anomaly, "flat baseline should not produce anomaly");
        assert_eq!(result_flat.z_score, 0.0, "z_score should be 0 when std_dev=0");
    }

    #[test]
    fn anomaly_scan_returns_all_results() {
        let baseline = vec![1.0, 2.0, 3.0, 2.0, 2.0];
        let detector = AnomalyDetector::from_baseline(&baseline, 2.0);

        let values = vec![2.0, 2.5, 10.0];
        let results = scan_for_anomalies(&detector, &values);

        assert_eq!(results.len(), 3, "should return one result per value");
        assert!(!results[0].is_anomaly, "2.0 should not be anomaly");
        assert!(!results[1].is_anomaly, "2.5 should not be anomaly");
        assert!(results[2].is_anomaly,  "10.0 should be anomaly");
        assert_eq!(results[2].value, 10.0, "value should be preserved");

        // format_anomaly_report の出力確認
        let report = format_anomaly_report(&results);
        assert!(report.contains("total=3"),    "should show total: {report}");
        assert!(report.contains("anomalies=1"), "should show anomaly count: {report}");
    }
}
```

## Step 5: `cargo test` で全 pass 確認

以下は `fav/` ディレクトリで実行する。

```
cargo test 2>&1 | grep "test result"
# 期待: 3859 tests, 0 failures
```

## Step 6: CI 事前確認

以下はすべて `fav/` ディレクトリで実行する。

```
cargo clippy --locked -- -D warnings
./target/debug/fav fmt --check self/compiler.fav
./target/debug/fav fmt --check self/checker.fav
```
