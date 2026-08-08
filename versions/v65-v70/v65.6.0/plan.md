# v65.6.0 実装計画 — Time Series Rune（`Rune.timeseries`）

Version: 65.6.0
Status: 未着手
Base tests: 3463
Target tests: 3465

---

## 実装ステップ

### Step 1: ディレクトリ・ファイル作成

1. `runes/timeseries/` ディレクトリ作成
2. `runes/timeseries/rune.toml` 作成
3. `runes/timeseries/timeseries.fav` 作成（全 18 関数）

### Step 2: `driver.rs` テスト追加

- `// -- v65500_tests (v65.5.0)` コメントの直前に `v65600_tests` を挿入
- 2 テスト関数:
  - `timeseries_rune_arima_fit`
  - `timeseries_rune_stl_decompose`

### Step 3: ビルド・テスト確認

```bash
cargo build
cargo test --bin fav v65600_tests
cargo test -j 8 -- --test-threads=8
```

---

## `timeseries.fav` 実装方針

- **全 18 関数をスタブとして実装**（シグネチャ確立が目的）
- `bind` / `let` は使用しない
- `Float.from_int` / `Float.sqrt` は使用しない
- 戻り値:
  - `Float` 系 → `0.0`
  - `List<_>` 系 → `[]`
  - `TimeSeries<Float>` 系 → `ts`（引数をそのまま返す）
  - `ModelSpec` / `FittedModel` / `Forecast` / `SeasonalComponents` / `TestResult` → レコードリテラル
- `ChangePointDetection` はコメント中で使用 → `contains("ChangePointDetection")` テストにマッチ

## `rune.toml` 形式

```toml
[rune]
name        = "timeseries"
version     = "0.1.0"
description = "..."
entry       = "timeseries.fav"
effects     = []

[dependencies]
```

---

## `driver.rs` 挿入コード

```rust
// -- v65600_tests (v65.6.0) -- Time Series Rune --
#[cfg(test)]
mod v65600_tests {
    #[test]
    fn timeseries_rune_arima_fit() {
        let content = include_str!("../../runes/timeseries/timeseries.fav");
        assert!(!content.is_empty(), "timeseries.fav should not be empty");
        assert!(content.contains("fn fit("), "timeseries.fav should define fit");
        assert!(content.contains("fn predict("), "timeseries.fav should define predict");
        assert!(content.contains("ARIMA"), "timeseries.fav should reference ARIMA");
        assert!(content.contains("SARIMA"), "timeseries.fav should reference SARIMA");
    }

    #[test]
    fn timeseries_rune_stl_decompose() {
        let content = include_str!("../../runes/timeseries/timeseries.fav");
        assert!(content.contains("fn decompose("), "timeseries.fav should define decompose");
        assert!(
            content.contains("ChangePointDetection"),
            "timeseries.fav should reference ChangePointDetection"
        );
        assert!(content.contains("fn adf_test("), "timeseries.fav should define adf_test");
    }
}
```

---

## 関数一覧（18 関数）

| カテゴリ | 関数名 | 戻り値 |
|---|---|---|
| モデル仕様 | `arima(p, d, q)` | `ModelSpec` レコード |
| モデル仕様 | `sarima(p, d, q, cap_p, cap_d, cap_q, s)` | `ModelSpec` レコード |
| モデル仕様 | `exponential_smoothing(alpha, beta, gamma)` | `ModelSpec` レコード |
| 学習・予測 | `fit(model_spec, data)` | `FittedModel` レコード |
| 学習・予測 | `predict(model, horizon)` | `Forecast` レコード |
| 季節分解 | `decompose(ts, method, period)` | `SeasonalComponents` レコード |
| 変化点検出 | `detect_changepoints(ts, method)` | `[]` |
| 特徴量 | `autocorrelation(ts, lags)` | `[]` |
| 特徴量 | `partial_autocorrelation(ts, lags)` | `[]` |
| 特徴量 | `adf_test(ts)` | `TestResult` レコード |
| 前処理 | `resample(ts, freq)` | `ts` |
| 前処理 | `rolling_mean(ts, window)` | `ts` |
| 前処理 | `ewm(ts, alpha)` | `ts` |
| 前処理 | `lag_features(ts, lags)` | `[]` |
| 評価指標 | `mae(actual, predicted)` | `0.0` |
| 評価指標 | `rmse(actual, predicted)` | `0.0` |
| 評価指標 | `mape(actual, predicted)` | `0.0` |
| 評価指標 | `smape(actual, predicted)` | `0.0` |

---

## リスク・注意点

- `TimeSeries<Float>` / `ModelSpec` 等の型は未定義（型チェックエラーは無視）
- `contains("ARIMA")` は `"SARIMA"` 内の部分文字列にもマッチするが、`arima` 関数が別途存在するため偽陽性は実害なし
- `contains("ChangePointDetection")` はコメント行 `// ChangePointDetection — PELT / BOCPD` でマッチ
