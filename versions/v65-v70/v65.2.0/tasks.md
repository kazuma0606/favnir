# v65.2.0 タスクリスト

Status: COMPLETE
Version: 65.2.0
Base tests: 3455
Target tests: 3457

---

## T0: 事前確認

- [x] `cargo test -j 8 -- --test-threads=8` でベース 3455 tests passed, 0 failed を確認
- [x] `fav/Cargo.toml` の version が `"65.0.0"` であることを確認（sub-version では Cargo.toml は更新しない）
- [x] `runes/stats/` ディレクトリが存在しないことを確認（新規作成対象）
- [x] `runes/stat/` が存在することを確認（既存のまま保持・削除しない）
- [x] `driver.rs` に `v65100_tests` が存在することを確認（`v65200_tests` の挿入位置）
- [x] `driver.rs` に `v65200_tests` が存在しないことを確認（新規追加）

---

## T1: Rune ファイル作成

- [x] `runes/stats/` ディレクトリ作成（`runes/stat/` とは別）
- [x] `runes/stats/rune.toml` 作成（`entry` / `effects = []` / `[dependencies]` 形式）
- [x] `runes/stats/stats.fav` 作成（以下の関数をすべて定義）
  - **記述統計**
  - [x] `mean` — 平均
  - [x] `variance` — 分散
  - [x] `std` — 標準偏差
  - [x] `median` — 中央値
  - [x] `percentile` — パーセンタイル
  - [x] `skewness` — 歪度
  - [x] `kurtosis` — 尖度
  - [x] `describe` — 要約統計（mean/std/median/p25/p75/p95/skewness/kurtosis/count）
  - **確率分布**
  - [x] `fit` — 分布フィッティング
  - [x] `sample` — サンプリング
  - [x] `pdf` — 確率密度関数
  - [x] `cdf` — 累積分布関数
  - **仮説検定**
  - [x] `t_test` — t 検定
  - [x] `chi_square` — カイ二乗検定
  - [x] `ks_test` — KS 検定
  - [x] `mannwhitney` — Mann-Whitney U 検定
  - [x] `anova` — 一元配置分散分析
  - **回帰**
  - [x] `linear_regression` — 線形回帰
  - [x] `logistic_regression` — ロジスティック回帰
  - **異常検知**
  - [x] `zscore_filter` — Z スコアフィルタ
  - [x] `iqr_filter` — IQR フィルタ
  - [x] `isolation_forest` — Isolation Forest
- [x] `stats.fav` 内に `let ` が含まれないことを確認
- [x] `stats.fav` 内に `bind.*=`（`<-` でない bind）が含まれないことを確認

---

## T2: `driver.rs` — `v65200_tests` 追加

- [x] `// -- v65100_tests (v65.1.0)` コメントの直前に `v65200_tests` を挿入
  - [x] `stats_rune_describe` — `fn mean(` / `fn std(` / `fn median(` / `fn describe(` を含む
  - [x] `stats_rune_hypothesis_test` — `fn t_test(` / `fn chi_square(` / `fn ks_test(` / `fn linear_regression(` / `fn zscore_filter(` を含む
- [x] `use super::*` は不要（`include_str!` のみ使用）
- [x] `cargo build` でエラーなし

---

## T3: ビルド・テスト

- [x] `cargo test --bin fav v65200_tests` で 2 件 PASS
  - [x] `stats_rune_describe` PASS
  - [x] `stats_rune_hypothesis_test` PASS
- [x] `cargo test -j 8 -- --test-threads=8` で 3457 tests passed, 0 failed を確認

---

## T4: ドキュメント・ステータス更新

- [x] `versions/roadmap/roadmap-v65.1-v66.0.md` の v65.2.0 行を「完了」に更新
- [x] `versions/current.md` の「進行中バージョン」を v65.2.0 に更新
- [x] 本 `tasks.md` を COMPLETE に更新（全チェックボックスを `[x]` に）

> **CHANGELOG 方針**: v65.1〜v65.9 では CHANGELOG.md を更新しない。v66.0.0 宣言時に一括追記する。
> **MDX 方針**: `site/` の MDX ドキュメントは v65.9.0 安定化時に一括作成するため今バージョンは省略。

---

## コードレビュー対応

実装は spec.md の通り。注意点:
- `bind x <- expr` 構文を使用（`=` ではなく `<-`）
- `Math.sqrt` を使用（`Float.sqrt` は VM に存在しない）
- `List.zip_with` は stats.fav では不使用（スタブのため）
- `runes/stat/`（既存・単数形）と `runes/stats/`（新規・複数形）は別ディレクトリ
