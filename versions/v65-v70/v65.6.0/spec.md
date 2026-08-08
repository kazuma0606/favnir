# v65.6.0 Spec — Time Series Rune（`Rune.timeseries`）

Version: 65.6.0
Status: 未着手
Base tests: 3463
Target tests: 3465

---

## 概要

時系列データの解析・予測・異常検知を型安全に扱う Rune `Rune.timeseries` を実装する。
`TimeSeries<T>` 型で時刻インデックス付きデータを表現し、ARIMA / SARIMA / STL などを提供する。

```favnir
// 利用例（用途のイメージ）
// ※ ロードマップ例は擬似コード。実際の Favnir 構文は技術ノートを参照。
public stage ForecastDemand: TimeSeries<Float> -> Forecast = |sales| {
    Rune.timeseries.predict(
        Rune.timeseries.fit(Rune.timeseries.sarima(1, 1, 1, 1, 1, 1, 7), sales),
        30
    )
}
```

ロードマップ `roadmap-v65.1-v66.0.md` の v65.6.0 セクションに準拠。

---

## 前提確認（T0 で実施）

- `cargo test -j 8 -- --test-threads=8` でベース 3463 tests passed, 0 failed を確認
- `fav/Cargo.toml` の version が `"65.0.0"` であることを確認（sub-version では Cargo.toml は更新しない）
- `runes/timeseries/` ディレクトリが存在しないことを確認（新規作成対象）
- `driver.rs` に `v65500_tests` が存在することを確認（`v65600_tests` の挿入位置）
- `driver.rs` に `v65600_tests` が存在しないことを確認（新規追加）
- `cargo test --bin fav v65500_tests` で 2 件 PASS することを確認（前バージョンが正常）
- `versions/current.md` の「進行中バージョン」が `v65.5.0` であることを確認

---

## 実装スコープ

### 1. `runes/timeseries/rune.toml` — Rune メタデータ

```toml
[rune]
name        = "timeseries"
version     = "0.1.0"
description = "Time Series Rune for Favnir — ARIMA/SARIMA, STL decomposition, change point detection, forecasting"
entry       = "timeseries.fav"
effects     = []

[dependencies]
```

### 2. `runes/timeseries/timeseries.fav` — Rune 実装スタブ

以下の全関数定義を含むスタブファイルを作成する。
今バージョンでは**関数シグネチャの確立**が目的。実際の時系列計算は将来フェーズ。

```favnir
// Time Series Rune — Rune.timeseries
// ARIMA/SARIMA modeling, STL decomposition, change point detection, forecasting
//
// NOTE: TimeSeries<Float>, ModelSpec, FittedModel, Forecast, SeasonalComponents,
//       DecompMethod, CpdMethod, TestResult 等の型は将来フェーズで型システムに登録する。
//       今バージョンは include_str! テストのみ（型チェックエラーは無視する）。

// --- モデル仕様コンストラクタ ---

// ARIMA (AutoRegressive Integrated Moving Average) モデル仕様
public fn arima(p: Int, d: Int, q: Int) -> ModelSpec {
    ModelSpec { kind: "ARIMA", p: p, d: d, q: q }
}

// SARIMA (Seasonal ARIMA) モデル仕様
public fn sarima(p: Int, d: Int, q: Int, cap_p: Int, cap_d: Int, cap_q: Int, s: Int) -> ModelSpec {
    ModelSpec { kind: "SARIMA", p: p, d: d, q: q, cap_p: cap_p, cap_d: cap_d, cap_q: cap_q, s: s }
}

// Exponential Smoothing (Holt-Winters) モデル仕様
public fn exponential_smoothing(alpha: Float, beta: Float, gamma: Float) -> ModelSpec {
    ModelSpec { kind: "ExponentialSmoothing", alpha: alpha, beta: beta, gamma: gamma }
}

// --- 学習・予測 ---

public fn fit(model_spec: ModelSpec, data: TimeSeries<Float>) -> FittedModel {
    FittedModel { spec: model_spec, params: [] }
}

public fn predict(model: FittedModel, horizon: Int) -> Forecast {
    Forecast { values: [], lower: [], upper: [] }
}

// --- 季節分解 ---

// STL (Seasonal-Trend decomposition using LOESS) 等による季節分解
public fn decompose(ts: TimeSeries<Float>, method: DecompMethod, period: Int) -> SeasonalComponents {
    SeasonalComponents { trend: [], seasonal: [], residual: [] }
}

// --- 変化点検出 ---

// ChangePointDetection — PELT / BOCPD アルゴリズムによる変化点検出
public fn detect_changepoints(ts: TimeSeries<Float>, method: CpdMethod) -> List<Int> {
    []
}

// --- 特徴量 ---

public fn autocorrelation(ts: TimeSeries<Float>, lags: Int) -> List<Float> {
    []
}

public fn partial_autocorrelation(ts: TimeSeries<Float>, lags: Int) -> List<Float> {
    []
}

// ADF 検定（単位根検定 — Augmented Dickey-Fuller Test）
public fn adf_test(ts: TimeSeries<Float>) -> TestResult {
    TestResult { statistic: 0.0, p_value: 1.0, reject_null: false }
}

// --- 前処理 ---

public fn resample(ts: TimeSeries<Float>, freq: Int) -> TimeSeries<Float> {
    ts
}

public fn rolling_mean(ts: TimeSeries<Float>, window: Int) -> TimeSeries<Float> {
    ts
}

// EWM (Exponentially Weighted Moving average — 指数加重平均)
public fn ewm(ts: TimeSeries<Float>, alpha: Float) -> TimeSeries<Float> {
    ts
}

public fn lag_features(ts: TimeSeries<Float>, lags: List<Int>) -> List<TimeSeries<Float>> {
    []
}

// --- 評価指標 ---

public fn mae(actual: List<Float>, predicted: List<Float>) -> Float {
    0.0
}

public fn rmse(actual: List<Float>, predicted: List<Float>) -> Float {
    0.0
}

public fn mape(actual: List<Float>, predicted: List<Float>) -> Float {
    0.0
}

public fn smape(actual: List<Float>, predicted: List<Float>) -> Float {
    0.0
}
```

### 3. `driver.rs` — `v65600_tests` 追加

挿入位置: `// -- v65500_tests (v65.5.0)` コメントの直前

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

挿入後、`cargo build` でエラーなしを確認。

---

## 完了条件

- `runes/timeseries/timeseries.fav` が存在し空でない
- `runes/timeseries/rune.toml` が存在する
- `timeseries.fav` に全 18 関数が定義されている:
  - `arima`, `sarima`, `exponential_smoothing`（モデル仕様コンストラクタ）
  - `fit`, `predict`（学習・予測）
  - `decompose`（季節分解）
  - `detect_changepoints`（変化点検出）
  - `autocorrelation`, `partial_autocorrelation`, `adf_test`（特徴量）
  - `resample`, `rolling_mean`, `ewm`, `lag_features`（前処理）
  - `mae`, `rmse`, `mape`, `smape`（評価指標）
  - ※ テストで検証するのはうち 7 要素（`fit`/`predict`/`ARIMA`文字列/`SARIMA`文字列/`decompose`/`ChangePointDetection`文字列/`adf_test`）。残りは tasks.md T1 チェックボックスで確認する
- `cargo test --bin fav v65600_tests` で 2 件 PASS
  - `timeseries_rune_arima_fit` PASS
  - `timeseries_rune_stl_decompose` PASS
- `cargo test -j 8 -- --test-threads=8` で 3465 tests passed, 0 failed

---

## 非スコープ

- 実際の時系列計算実装 — 将来フェーズ
- `TimeSeries<Float>` / `ModelSpec` / `FittedModel` / `Forecast` / `SeasonalComponents` / `DecompMethod` / `CpdMethod` / `TestResult` の型システム登録 — 将来フェーズ
- `fav check` での型チェック通過 — 今バージョンは `include_str!` テストのみ（型未定義エラーは無視する）
- CHANGELOG.md 更新 — v66.0.0 宣言時に一括追記
- site/ MDX ドキュメント作成 — v65.9.0 安定化時に一括作成するため今バージョンは省略

---

## 技術ノート

### `include_str!` パス（`fav/src/driver.rs` 起点）

- `"../../runes/timeseries/timeseries.fav"` → `favnir/runes/timeseries/timeseries.fav`

### `contains` 判定の設計方針

- `contains("fn fit(")` — `public fn fit(` にマッチ
- `contains("fn predict(")` — `public fn predict(` にマッチ
- `contains("ARIMA")` — コメント `// ARIMA` + 文字列リテラル `"ARIMA"` でマッチ。なお `"SARIMA"` も `ARIMA` を部分文字列として含むが、arima 関数自体が存在するため偽陽性の問題は実害なし
- `contains("SARIMA")` — コメント `// SARIMA` + 文字列リテラル `"SARIMA"` でマッチ
- `contains("fn decompose(")` — `public fn decompose(` にマッチ
- `contains("ChangePointDetection")` — コメント `// ChangePointDetection — PELT / BOCPD` でマッチ
- `contains("fn adf_test(")` — `public fn adf_test(` にマッチ

### ARIMA / SARIMA 文字列と偽陰性について

`contains("ARIMA")` は `"SARIMA"` 内の部分文字列としてもマッチする。
そのため `arima` 関数を誤って削除した場合でも、`"SARIMA"` 文字列リテラルがあれば
`contains("ARIMA")` が PASS してしまう（偽陰性 — 削除を見落とす）。
これは include_str! テストの限界として許容する（`arima` 関数の実際の存在は tasks.md T1 の
`grep -c 'public fn '` カウント（18件）で補完する）。

### v65.1.0〜v65.5.0 レビューで確立した構文ルール

- `bind x <- expr` は Result/Option を返す式にのみ使用する（スタブでは不要）
- `let` は使わない
- `Math.sqrt` を使う（`Float.sqrt` は VM に存在しない）
- `Float.from_int` は VM に存在しない

### rune.toml フォーマット

- `entry = "timeseries.fav"`（`main` ではなく `entry`）
- `effects = []` を明示
- `[dependencies]` セクションを含める
