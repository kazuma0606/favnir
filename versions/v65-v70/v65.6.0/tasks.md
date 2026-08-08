# v65.6.0 タスクリスト

Status: COMPLETE
Version: 65.6.0
Base tests: 3463
Target tests: 3465

---

## T0: 事前確認

- [x] `cargo test -j 8 -- --test-threads=8` でベース 3463 tests passed, 0 failed を確認
- [x] `fav/Cargo.toml` の version が `"65.0.0"` であることを確認（sub-version では変更しない）
- [x] `runes/timeseries/` ディレクトリが存在しないことを確認（新規作成対象）
- [x] `driver.rs` に `v65500_tests` が存在することを確認（`v65600_tests` の挿入位置）
- [x] `driver.rs` に `v65600_tests` が存在しないことを確認（新規追加）
- [x] `cargo test --bin fav v65500_tests` で 2 件 PASS することを確認（前バージョンが正常）
- [x] `versions/current.md` の「進行中バージョン」が `v65.5.0` であることを確認

---

## T1: Rune ファイル作成

- [x] `runes/timeseries/` ディレクトリ作成
- [x] `runes/timeseries/rune.toml` 作成（`entry` / `effects = []` / `[dependencies]` 形式）
- [x] `runes/timeseries/timeseries.fav` 作成（以下の全 18 関数を定義）
  - **モデル仕様コンストラクタ**
  - [x] `arima` — ARIMA モデル仕様
  - [x] `sarima` — Seasonal ARIMA モデル仕様
  - [x] `exponential_smoothing` — Holt-Winters モデル仕様
  - **学習・予測**
  - [x] `fit` — モデル学習
  - [x] `predict` — 予測
  - **季節分解**
  - [x] `decompose` — STL 等による季節分解
  - **変化点検出**
  - [x] `detect_changepoints` — PELT / BOCPD 変化点検出
  - **特徴量**
  - [x] `autocorrelation` — 自己相関
  - [x] `partial_autocorrelation` — 偏自己相関
  - [x] `adf_test` — ADF 検定（単位根検定）
  - **前処理**
  - [x] `resample` — リサンプリング
  - [x] `rolling_mean` — ローリング平均
  - [x] `ewm` — 指数加重移動平均
  - [x] `lag_features` — ラグ特徴量
  - **評価指標**
  - [x] `mae` — 平均絶対誤差
  - [x] `rmse` — 二乗平均平方根誤差
  - [x] `mape` — 平均絶対パーセント誤差
  - [x] `smape` — 対称平均絶対パーセント誤差
- [x] `timeseries.fav` 内に `let ` が含まれないことを確認
- [x] `timeseries.fav` 内に `bind.*=`（`<-` でない bind）が含まれないことを確認
- [x] `timeseries.fav` 内に `Float.from_int` / `Float.sqrt` が含まれないことを確認
- [x] `grep -c 'public fn ' timeseries.fav` で 18 が出ることを確認

---

## T2: `driver.rs` — `v65600_tests` 追加

- [x] `// -- v65500_tests (v65.5.0)` コメントの直前に `v65600_tests` を挿入
  - [x] `timeseries_rune_arima_fit` — `fn fit(` / `fn predict(` / `ARIMA` / `SARIMA` を含む
  - [x] `timeseries_rune_stl_decompose` — `fn decompose(` / `ChangePointDetection`（コメント行 `// ChangePointDetection — PELT / BOCPD` でマッチ）/ `fn adf_test(` を含む
- [x] `use super::*` は不要（`include_str!` のみ使用）
- [x] `cargo build` でエラーなし

---

## T3: ビルド・テスト

- [x] `cargo test --bin fav v65600_tests` で 2 件 PASS
  - [x] `timeseries_rune_arima_fit` PASS
  - [x] `timeseries_rune_stl_decompose` PASS
- [x] `cargo test --bin fav` で 3465 tests passed, 0 failed を確認

---

## T4: ドキュメント・ステータス更新

- [x] `versions/roadmap/roadmap-v65.1-v66.0.md` の v65.6.0 行を「完了」に更新
- [x] `versions/current.md` の「進行中バージョン」を v65.6.0 に更新
- [x] 本 `tasks.md` を COMPLETE に更新（全チェックボックスを `[x]` に）

> **CHANGELOG 方針**: v65.1〜v65.9 では CHANGELOG.md を更新しない。v66.0.0 宣言時に一括追記する。
> **MDX 方針**: `site/` の MDX ドキュメントは v65.9.0 安定化時に一括作成するため今バージョンは省略。
