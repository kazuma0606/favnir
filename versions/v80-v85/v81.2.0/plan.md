# Plan: v81.2.0 — 統計的品質チェック（`StatisticalCheck`）

## Step 1: 前提確認

- `cargo test` を実行し、3845 tests, 0 failures を確認する
- `fav/src/test_framework.rs` に v81.1.0 の `run_quality_check` が定義済みであることを確認する

## Step 2: `fav/src/test_framework.rs` に追記

`run_quality_check` の定義の直後に以下を追加する。

```rust
// ── v81.2.0: DistributionStats / StatisticalCheck ────────────────────────────

/// 数値列の統計的分布情報。
#[derive(Debug, Clone)]
pub struct DistributionStats {
    pub mean: f64,
    pub std_dev: f64,
    pub min: f64,
    pub max: f64,
    pub count: usize,
}

/// 数値列の統計量を計算する（母標準偏差 n 割り）。
///
/// `values` が空の場合は全フィールド 0.0 / count=0 を返す。
pub fn compute_distribution_stats(values: &[f64]) -> DistributionStats {
    if values.is_empty() {
        return DistributionStats { mean: 0.0, std_dev: 0.0, min: 0.0, max: 0.0, count: 0 };
    }
    let count = values.len();
    let mean = values.iter().sum::<f64>() / count as f64;
    let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / count as f64;
    let std_dev = variance.sqrt();
    let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    DistributionStats { mean, std_dev, min, max, count }
}

/// Z スコアベースの外れ値検出設定。
///
/// `column` はカラムのインデックス文字列（意味的メタデータ。`detect_outliers` では参照しない）。
/// `z_score_threshold` はこの値より大きな Z スコアを外れ値とみなす。
#[derive(Debug, Clone)]
pub struct StatisticalCheck {
    pub column: String,
    pub z_score_threshold: f64,
}

/// Z スコアが `check.z_score_threshold` を超える値のインデックスを返す。
///
/// `std_dev == 0.0` のとき（全値が同一）は外れ値なし（空 Vec）を返す。
pub fn detect_outliers(check: &StatisticalCheck, values: &[f64]) -> Vec<usize> {
    let stats = compute_distribution_stats(values);
    if stats.std_dev == 0.0 {
        return Vec::new();
    }
    values
        .iter()
        .enumerate()
        .filter_map(|(i, &v)| {
            let z = (v - stats.mean).abs() / stats.std_dev;
            if z > check.z_score_threshold { Some(i) } else { None }
        })
        .collect()
}

/// 分布情報を人間向けの文字列に変換する。
///
/// 出力形式: `"count={count} mean={mean:.3} std={std_dev:.3} min={min:.3} max={max:.3}"`
pub fn format_distribution_report(stats: &DistributionStats) -> String {
    format!(
        "count={} mean={:.3} std={:.3} min={:.3} max={:.3}",
        stats.count, stats.mean, stats.std_dev, stats.min, stats.max
    )
}
```

## Step 3: `fav/src/driver.rs` に `mod v81200_tests` を追加

`mod v81100_tests { ... }` の直後に追加する。

```rust
#[cfg(test)]
mod v81200_tests {
    use fav_core::test_framework::*;

    #[test]
    fn distribution_stats_computed_correctly() {
        let values = vec![1.0, 2.0, 3.0];
        let stats = compute_distribution_stats(&values);
        assert_eq!(stats.count, 3);
        assert!((stats.mean - 2.0).abs() < 1e-9, "mean should be 2.0: {}", stats.mean);
        assert_eq!(stats.min, 1.0);
        assert_eq!(stats.max, 3.0);
        // 母標準偏差: sqrt(((1-2)^2 + (2-2)^2 + (3-2)^2) / 3) = sqrt(2/3) ≈ 0.8165
        assert!((stats.std_dev - (2.0_f64 / 3.0).sqrt()).abs() < 1e-9,
            "std_dev mismatch: {}", stats.std_dev);
        // format_distribution_report の出力形式確認
        let report = format_distribution_report(&stats);
        assert!(report.contains("count=3"), "report should contain count=3: {report}");
        assert!(report.contains("mean=2.000"), "report should contain mean=2.000: {report}");
    }

    #[test]
    fn outlier_detection_finds_extreme_values() {
        let check = StatisticalCheck {
            column: "0".to_string(),
            z_score_threshold: 2.0,
        };
        // 100.0 が明らかな外れ値
        let values = vec![1.0, 2.0, 3.0, 2.0, 100.0];
        let outliers = detect_outliers(&check, &values);
        assert!(!outliers.is_empty(), "should detect at least one outlier");
        assert!(outliers.contains(&4), "index 4 (100.0) should be an outlier: {:?}", outliers);

        // 均一データ: 外れ値なし
        let uniform = vec![5.0, 5.0, 5.0];
        let no_outliers = detect_outliers(&check, &uniform);
        assert_eq!(no_outliers.len(), 0, "uniform data should have no outliers");
    }
}
```

## Step 4: `cargo test` で全 pass 確認

```
cargo test 2>&1 | grep "test result"
# 期待: 3847 tests, 0 failures
```

## Step 5: CHANGELOG 更新

`CHANGELOG.md` の先頭に v81.2.0 エントリを追加する。

## Step 6: CI 事前確認

以下はすべて `fav/` ディレクトリで実行する。

```
cargo clippy --locked -- -D warnings
./target/debug/fav fmt --check self/compiler.fav
./target/debug/fav fmt --check self/checker.fav
```
