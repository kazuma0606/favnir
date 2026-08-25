# Tasks: v81.2.0 — 統計的品質チェック（`StatisticalCheck`）

> COMPLETE — 2026-08-19
> 3847 tests, 0 failures（+2 from 3845）

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、3845 tests, 0 failures を確認する
- [x] `Cargo.toml` バージョンが `81.0.0` であることを確認する
  （v81.x マイナーバージョンは Cargo.toml を更新しない慣例。v82.0.0 宣言時に一括更新する）
- [x] `fav/src/driver.rs` に `mod v81100_tests` が存在することを確認する（v81.1.0 完了済みの証拠）
- [x] `fav/src/test_framework.rs` に v81.1.0 の `run_quality_check` が定義済みであることを確認する

## T1: `fav/src/test_framework.rs` に追記

- [x] `DistributionStats` 構造体（`#[derive(Debug, Clone)]`）を追加する
  - フィールド: `mean: f64`, `std_dev: f64`, `min: f64`, `max: f64`, `count: usize`
- [x] `compute_distribution_stats(values: &[f64]) -> DistributionStats` を実装する
  - 空スライスの場合は全フィールド 0.0 / count=0 を返す
  - `mean`: 合計 / count
  - `std_dev`: 母標準偏差（`n` で割る）= `sqrt(Σ(x - mean)² / n)`
  - `min` / `max`: `f64::INFINITY` / `f64::NEG_INFINITY` からの fold で計算
- [x] `StatisticalCheck` 構造体（`#[derive(Debug, Clone)]`）を追加する
  - フィールド: `column: String`, `z_score_threshold: f64`
- [x] `detect_outliers(check: &StatisticalCheck, values: &[f64]) -> Vec<usize>` を実装する
  - `compute_distribution_stats` を呼んで統計量を取得する
  - `std_dev == 0.0` のとき空 Vec を返す
  - Z スコア = `|v - mean| / std_dev` が `z_score_threshold` を超えるインデックスを返す
- [x] `format_distribution_report(stats: &DistributionStats) -> String` を実装する
  - 形式: `"count={count} mean={mean:.3} std={std_dev:.3} min={min:.3} max={max:.3}"`

## T2: `fav/src/driver.rs` に `mod v81200_tests` を追加

- [x] `mod v81100_tests { ... }` の直後に `#[cfg(test)] mod v81200_tests { ... }` を追加する
- [x] `use fav_core::test_framework::*;` でインポートする
- [x] `distribution_stats_computed_correctly` テストを実装する
  - `[1.0, 2.0, 3.0]` で `count=3`, `mean=2.0`, `min=1.0`, `max=3.0` を確認する
  - 母標準偏差 `sqrt(2/3) ≈ 0.8165` と誤差 `1e-9` 以内で一致することを確認する
  - `format_distribution_report` の出力に `"count=3"` と `"mean=2.000"` が含まれることを確認する
- [x] `outlier_detection_finds_extreme_values` テストを実装する
  - `[1.0, 2.0, 3.0, 2.0, 100.0]` に threshold=1.9 で index 4 が外れ値として検出されることを確認する
    （注: n=5 のとき外れ値の Z スコア ≈ sqrt(n-1) = 1.9997... で threshold=2.0 strict > を超えないため 1.9 を使用）
  - 均一データ `[5.0, 5.0, 5.0]` で空 Vec が返ることを確認する

## T3: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、3847 tests, 0 failures であることを確認する

## T4: CHANGELOG 更新

- [x] `CHANGELOG.md` の先頭に v81.2.0 エントリを追加する

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
