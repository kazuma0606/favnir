# v83.6.0 仕様書 — パフォーマンス回帰検知（`PerfBaseline` / `PerfRegression`）

## Background

v83.5.0 で分散トレーシング型が導入された。次のステップとして、
パイプライン実行のパフォーマンスをベースラインと比較し、回帰（性能劣化）を検知する仕組みを整備する。

本バージョンは Observability 2.0 スプリント（v83.1〜v84.0）の第 6 段階。

ロードマップ参照: `versions/roadmap/roadmap-v83.1-v84.0.md` — v83.6.0 セクション

## Goals

1. `PerfBaseline` 構造体を追加する（p50/p95/p99 ベースライン）
2. `PerfRegression` 構造体を追加する（回帰レポート）
3. `PerfBaseline::from_samples(pipeline_name: &str, samples_ms: &[u64]) -> PerfBaseline` を追加する
4. `detect_perf_regression(baseline: &PerfBaseline, current_ms: u64, threshold_pct: f64) -> Option<PerfRegression>` を追加する
5. `format_regression_report(regression: &PerfRegression) -> String` を追加する

## 型定義・API

```rust
/// パイプライン実行のパフォーマンスベースライン。
#[derive(Debug, Clone, PartialEq)]
pub struct PerfBaseline {
    pub pipeline_name: String,
    pub p50_ms: u64,
    pub p95_ms: u64,
    pub p99_ms: u64,
}

impl PerfBaseline {
    /// サンプル一覧からベースラインを算出する。
    ///
    /// サンプルをソートし、インデックスベースの百分位数で p50/p95/p99 を求める:
    /// - `p50_ms = sorted[n * 50 / 100]`
    /// - `p95_ms = sorted[n * 95 / 100]`
    /// - `p99_ms = sorted[n * 99 / 100]`
    ///
    /// `samples_ms` が空の場合は p50/p95/p99 すべて 0 を返す。
    pub fn from_samples(pipeline_name: &str, samples_ms: &[u64]) -> PerfBaseline
}

/// 回帰が検知された際の詳細。
#[derive(Debug, Clone, PartialEq)]
pub struct PerfRegression {
    pub pipeline_name: String,
    pub baseline: PerfBaseline,
    pub current_ms: u64,
    /// (current_ms - baseline.p95_ms) / baseline.p95_ms * 100.0
    pub regression_pct: f64,
}

/// `current_ms` を `baseline.p95_ms` と比較し、回帰率が `threshold_pct` を超えた場合に
/// `Some(PerfRegression)` を返す。それ以外は `None`。
///
/// `baseline.p95_ms == 0` の場合は回帰なし（`None`）とみなす（ゼロ除算ガード）。
pub fn detect_perf_regression(
    baseline: &PerfBaseline,
    current_ms: u64,
    threshold_pct: f64,
) -> Option<PerfRegression>

/// 回帰レポートのテキストを返す。
///
/// 例:
/// "PerfRegression: etl_main\nBaseline p95: 200ms\nCurrent: 260ms\nRegression: +30.00%"
pub fn format_regression_report(regression: &PerfRegression) -> String
```

## 百分位数の計算式

```
n = samples_ms.len()
sorted = samples_ms をソートしたもの

p50_ms = sorted[n * 50 / 100]
p95_ms = sorted[n * 95 / 100]
p99_ms = sorted[n * 99 / 100]
```

インデックス計算はすべて `usize` の整数除算（floor）。

## 回帰判定ロジック

```
regression_pct = (current_ms as f64 - baseline.p95_ms as f64) / baseline.p95_ms as f64 * 100.0

if regression_pct > threshold_pct:
    Some(PerfRegression { pipeline_name, baseline, current_ms, regression_pct })
else:
    None
```

`current_ms <= baseline.p95_ms` の場合 `regression_pct` は 0 以下となり `threshold_pct > 0` ならば必ず `None`。

## テスト（v83.6.0 で追加）

実際のテスト数ベース（※ drift 補正後）: **3897 + 2 = 3899**

（ロードマップ記載値 3885 + 2 = 3887 は旧バージョン到達時点のドリフト。
 実際の v83.5.0 完了テスト数は 3897。）

### `perf_regression_detected_above_threshold`

```rust
let samples = vec![100u64, 150, 180, 200, 210, 220, 230, 240, 250, 300];
let baseline = PerfBaseline::from_samples("etl_main", &samples);
// sorted: [100,150,180,200,210,220,230,240,250,300], n=10
// p95 = sorted[9] = 300
// current = 390 → regression = (390-300)/300*100 = 30%
// threshold = 20% → 30 > 20 → Some
let result = detect_perf_regression(&baseline, 390, 20.0);
assert!(result.is_some(), "regression should be detected above threshold");
let reg = result.unwrap();
assert_eq!(reg.current_ms, 390);
assert!((reg.regression_pct - 30.0).abs() < 0.1);
```

### `perf_no_regression_within_threshold`

```rust
let samples = vec![100u64, 150, 180, 200, 210, 220, 230, 240, 250, 300];
let baseline = PerfBaseline::from_samples("etl_main", &samples);
// p95 = 300, current = 330 → regression = 10%
// threshold = 20% → 10 < 20 → None
let result = detect_perf_regression(&baseline, 330, 20.0);
assert!(result.is_none(), "no regression should be detected within threshold");
```

## Success Criteria

- `cargo test` が 3899 tests pass（+2）、0 failures
- `PerfBaseline::from_samples` が正しく p50/p95/p99 を算出する（インデックスベース）
- `detect_perf_regression` が閾値超過で `Some`、閾値以内で `None` を返す
- `samples_ms` が空のとき `from_samples` が p50/p95/p99 = 0 の `PerfBaseline` を返す
- `baseline.p95_ms == 0` のとき `detect_perf_regression` が `None` を返す（ゼロ除算なし）

## Files to Modify

- `fav/src/test_framework.rs` — 型定義・impl・関数追加
- `fav/src/driver.rs` — `v83600_tests` モジュール追加
- `CHANGELOG.md` — v83.6.0 エントリ追加
