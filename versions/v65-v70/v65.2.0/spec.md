# v65.2.0 Spec — Statistics Rune（`Rune.stats`）

Version: 65.2.0
Status: 未着手
Base tests: 3455
Target tests: 3457

---

## 概要

記述統計・確率分布・仮説検定・回帰分析・異常検知を型安全に扱う統計 Rune `Rune.stats` を実装する。

既存 `runes/stat/`（サンプリング特化）とは**別ディレクトリ** `runes/stats/` に新設する。
ユーザー向けは `Rune.stats`（複数形）、既存は `Rune.stat`（単数形）として共存する。

```favnir
public stage SummaryStats: List<Float> -> StatsReport = |data| {
    Rune.stats.describe(data)
    // → { mean: 4.2, std: 1.1, median: 4.0, p95: 6.3, skewness: 0.3 }
}

public stage AnomalyDetect: List<Float> -> List<Anomaly> = |data| {
    bind dist <- Rune.stats.fit(NormalDist, data)
    Rune.stats.zscore_filter(data, dist, threshold: 3.0)
}
```

ロードマップ `roadmap-v65.1-v66.0.md` の v65.2.0 セクションに準拠。

---

## 前提確認（T0 で実施）

- `cargo test -j 8 -- --test-threads=8` でベース 3455 tests passed, 0 failed を確認
- `fav/Cargo.toml` の version が `"65.1.0"` であることを確認（v65.1.0 完了後は 65.1.0 のまま）
- `runes/stats/` ディレクトリが存在しないことを確認（`runes/stat/` は既存のまま保持）
- `driver.rs` に `v65100_tests` が存在することを確認（`v65200_tests` の挿入位置）
- `driver.rs` に `v65200_tests` が存在しないことを確認（新規追加）

---

## 実装スコープ

### 1. `runes/stats/rune.toml` — Rune メタデータ

既存 `runes/stat/rune.toml` の形式（`entry` / `effects = []` / `[dependencies]`）に合わせる。

```toml
[rune]
name        = "stats"
version     = "0.1.0"
description = "Statistics Rune for Favnir — descriptive stats, hypothesis tests, regression, anomaly detection"
entry       = "stats.fav"
effects     = []

[dependencies]
```

### 2. `runes/stats/stats.fav` — Rune 実装スタブ

以下の全関数定義を含むスタブファイルを作成する。
今バージョンでは**関数シグネチャの確立**が目的。実際の数値計算は将来フェーズ。

```favnir
// Statistics Rune — Rune.stats
// Descriptive statistics, distributions, hypothesis tests, regression, anomaly detection
//
// NOTE: 既存の Rune.stat（runes/stat/）はサンプリング特化。
//       このファイルは記述統計・仮説検定・回帰分析の Rune.stats（runes/stats/）。

// --- 記述統計 ---

public fn mean(xs: List<Float>) -> Float {
    List.sum(xs) / Float.from_int(List.length(xs))
}

public fn variance(xs: List<Float>) -> Float {
    bind m <- mean(xs)
    bind diffs <- List.map(xs, |x| { (x - m) * (x - m) })
    mean(diffs)
}

public fn std(xs: List<Float>) -> Float {
    variance(xs) |> Math.sqrt
}

public fn median(xs: List<Float>) -> Float {
    0.0
}

public fn percentile(xs: List<Float>, p: Float) -> Float {
    0.0
}

public fn skewness(xs: List<Float>) -> Float {
    0.0
}

public fn kurtosis(xs: List<Float>) -> Float {
    0.0
}

public fn describe(xs: List<Float>) -> StatsReport {
    StatsReport {
        mean:     mean(xs),
        std:      std(xs),
        variance: variance(xs),
        median:   median(xs),
        p25:      percentile(xs, 25.0),
        p75:      percentile(xs, 75.0),
        p95:      percentile(xs, 95.0),
        skewness: skewness(xs),
        kurtosis: kurtosis(xs),
        count:    List.length(xs)
    }
}

// --- 確率分布 ---

public fn fit(dist_type: DistType, xs: List<Float>) -> Distribution {
    Distribution { dist_type: dist_type, params: [] }
}

public fn sample(dist: Distribution, n: Int) -> List<Float> {
    []
}

public fn pdf(dist: Distribution, x: Float) -> Float {
    0.0
}

public fn cdf(dist: Distribution, x: Float) -> Float {
    0.0
}

// --- 仮説検定 ---

public fn t_test(a: List<Float>, b: List<Float>) -> TestResult {
    TestResult { statistic: 0.0, p_value: 1.0, reject_null: false }
}

public fn chi_square(observed: List<Float>, expected: List<Float>) -> TestResult {
    TestResult { statistic: 0.0, p_value: 1.0, reject_null: false }
}

public fn ks_test(a: List<Float>, b: List<Float>) -> TestResult {
    TestResult { statistic: 0.0, p_value: 1.0, reject_null: false }
}

public fn mannwhitney(a: List<Float>, b: List<Float>) -> TestResult {
    TestResult { statistic: 0.0, p_value: 1.0, reject_null: false }
}

public fn anova(groups: List<List<Float>>) -> TestResult {
    TestResult { statistic: 0.0, p_value: 1.0, reject_null: false }
}

// --- 回帰 ---

public fn linear_regression(xs: List<Float>, ys: List<Float>) -> RegressionResult {
    RegressionResult { coefficients: [], intercept: 0.0, r_squared: 0.0, p_values: [], residuals: [] }
}

public fn logistic_regression(xs: List<Float>, ys: List<Float>) -> RegressionResult {
    RegressionResult { coefficients: [], intercept: 0.0, r_squared: 0.0, p_values: [], residuals: [] }
}

// --- 異常検知 ---

public fn zscore_filter(xs: List<Float>, dist: Distribution, threshold: Float) -> List<Anomaly> {
    []
}

public fn iqr_filter(xs: List<Float>, multiplier: Float) -> List<Anomaly> {
    []
}

public fn isolation_forest(xs: List<Float>, n_trees: Int) -> List<Anomaly> {
    []
}
```

### 3. `driver.rs` — `v65200_tests` 追加

`v65100_tests` の直前（`// -- v65100_tests (v65.1.0)` コメントの直前）に挿入:

```rust
// -- v65200_tests (v65.2.0) -- Statistics Rune --
#[cfg(test)]
mod v65200_tests {
    #[test]
    fn stats_rune_describe() {
        let content = include_str!("../../runes/stats/stats.fav");
        assert!(!content.is_empty(), "stats.fav should not be empty");
        assert!(content.contains("fn mean("), "stats.fav should define mean");
        assert!(content.contains("fn std("), "stats.fav should define std");
        assert!(content.contains("fn median("), "stats.fav should define median");
        assert!(content.contains("fn describe("), "stats.fav should define describe");
    }

    #[test]
    fn stats_rune_hypothesis_test() {
        let content = include_str!("../../runes/stats/stats.fav");
        assert!(content.contains("fn t_test("), "stats.fav should define t_test");
        assert!(content.contains("fn chi_square("), "stats.fav should define chi_square");
        assert!(content.contains("fn ks_test("), "stats.fav should define ks_test");
        assert!(
            content.contains("fn linear_regression("),
            "stats.fav should define linear_regression"
        );
        assert!(
            content.contains("fn zscore_filter("),
            "stats.fav should define zscore_filter"
        );
    }
}
```

---

## 完了条件

- `runes/stats/stats.fav` が存在し空でない
- `runes/stats/rune.toml` が存在する
- `stats.fav` に `mean`, `std`, `median`, `describe`, `t_test`, `chi_square`, `ks_test`, `linear_regression`, `zscore_filter` が定義されている
- `cargo test --bin fav v65200_tests` で 2 件 PASS
  - `stats_rune_describe` PASS
  - `stats_rune_hypothesis_test` PASS
- `cargo test -j 8 -- --test-threads=8` で 3457 tests passed, 0 failed

---

## 非スコープ

- 実際の統計計算実装（数値アルゴリズム）— 将来フェーズ
- `StatsReport` / `Distribution` / `DistType` / `TestResult` / `RegressionResult` / `Anomaly` の型システム登録 — 将来フェーズ
- `NormalDist` / `PoissonDist` / `BinomialDist` / `ExponentialDist` の列挙型定義 — 将来フェーズ
- `fav check` での型チェック通過 — 今バージョンは `include_str!` テストのみ（型未定義エラーは無視する）
- `site/` MDX ドキュメント作成 — v65.9.0 安定化時に一括作成するため今バージョンは省略
- CHANGELOG.md 更新 — v66.0.0 宣言時に一括追記

---

## 技術ノート

### `include_str!` パス（`fav/src/driver.rs` 起点）

- `"../../runes/stats/stats.fav"` → `favnir/runes/stats/stats.fav`

### `StatsReport` / `DistType` 等の型未定義エラーについて

`stats.fav` を `fav check` した場合、`StatsReport` / `DistType` / `Distribution` / `TestResult` 等が
型システムに未登録のためエラーになる。これは想定内で今バージョンのスコープ外。
`driver.rs` のテストは `include_str!` で文字列として読み込むだけなので型チェックなしで動作する。

### v65.1.0 レビューで判明した正しい構文（必ず守ること）

- `bind x <- expr`（`=` ではなく `<-`）
- `Math.sqrt`（`Float.sqrt` は VM に存在しない）
- `List.zip_with(f, xs, ys)` — クロージャが第1引数

### rune.toml フォーマット

- `entry = "stats.fav"`（`main` ではなく `entry`）
- `effects = []` を明示
- `[dependencies]` セクションを含める
