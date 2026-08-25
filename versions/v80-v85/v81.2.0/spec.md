# Spec: v81.2.0 — 統計的品質チェック（`StatisticalCheck`）

## Background

v81.1.0 で `QualityRule` / `QualityCheck` 型基盤を構築した。
本バージョンでは数値カラムの分布・外れ値を統計的に検出する型を追加する。

ロードマップ: `versions/roadmap/roadmap-v81.1-v82.0.md`（v81.2.0 セクション）

> **テスト数**: ロードマップ drift 補正済み（3843 + 2 = 3845）と実際のベース（3845）が一致。
> v81.1.0 code-reviewer 対応で +2 件追加されたため実際のベースは **3845**。
> 本バージョンの完了条件は **3845 + 2 = 3847**。

## Goals

- `DistributionStats` 構造体を `test_framework.rs` に追加する
- `compute_distribution_stats(values: &[f64]) -> DistributionStats` を実装する
- `StatisticalCheck` 構造体を追加する
- `detect_outliers(check: &StatisticalCheck, values: &[f64]) -> Vec<usize>` を実装する
- `format_distribution_report(stats: &DistributionStats) -> String` を実装する
- テスト 2 件を追加して **3847 tests** を達成する

## API / Type Definitions

```rust
// fav/src/test_framework.rs（既存ファイルに追記）

/// 数値列の統計的分布情報。
#[derive(Debug, Clone)]
pub struct DistributionStats {
    pub mean: f64,
    pub std_dev: f64,
    pub min: f64,
    pub max: f64,
    pub count: usize,
}

/// 数値列の統計量を計算する。
///
/// `values` が空の場合は `mean=0.0`, `std_dev=0.0`, `min=0.0`, `max=0.0`, `count=0` を返す。
/// `std_dev` は母標準偏差（n で割る）で計算する。
pub fn compute_distribution_stats(values: &[f64]) -> DistributionStats;

/// Z スコアベースの外れ値検出設定。
///
/// `column` はカラムのインデックス文字列（"0", "1", ...）。
/// `z_score_threshold` はこの値より大きな Z スコアを外れ値とみなす（例: 2.0）。
#[derive(Debug, Clone)]
pub struct StatisticalCheck {
    pub column: String,
    pub z_score_threshold: f64,
}

/// `StatisticalCheck` で指定された Z スコア閾値を超える値のインデックスを返す。
///
/// Z スコア = `|value - mean| / std_dev`。
/// `std_dev == 0.0` の場合（全値が同一）は外れ値なしとして空 Vec を返す。
/// 返り値は `values` のインデックス（0 始まり）。
pub fn detect_outliers(check: &StatisticalCheck, values: &[f64]) -> Vec<usize>;

/// 分布情報を人間向けの文字列に変換する。
///
/// 出力形式: `"count={count} mean={mean:.3} std={std_dev:.3} min={min:.3} max={max:.3}"`
pub fn format_distribution_report(stats: &DistributionStats) -> String;
```

### 出力例

```rust
let values = vec![1.0, 2.0, 3.0, 4.0, 100.0];
let stats = compute_distribution_stats(&values);
// stats.count == 5
// stats.mean == 22.0
// stats.min == 1.0
// stats.max == 100.0

// 注: n=5 のとき 100.0 の Z スコア = (100-22)/std_dev ≈ 1.9993 であり、
// threshold=2.0 の strict > 比較では外れ値として検出されない。
// 実際のテストでは threshold=1.9 を使用する（Z ≈ 1.9993 > 1.9）。
let check = StatisticalCheck { column: "0".to_string(), z_score_threshold: 1.9 };
let outliers = detect_outliers(&check, &values);
// outliers == vec![4]  (100.0 は Z スコア ≈ 1.9993 > 1.9)

let report = format_distribution_report(&stats);
// "count=5 mean=22.000 std=38.XXX min=1.000 max=100.000"
```

## Success Criteria

- `cargo test` が **3847 tests**, 0 failures
- `distribution_stats_computed_correctly`:
  - `[1.0, 2.0, 3.0]` の stats で `count=3`, `mean=2.0`, `min=1.0`, `max=3.0` を確認する
  - `std_dev` が母標準偏差 `sqrt(2.0/3.0) ≈ 0.8165` と誤差 `1e-9` 以内で一致することを確認する
  - `format_distribution_report` の出力に `"count=3"` と `"mean=2.000"` が含まれることを確認する
- `outlier_detection_finds_extreme_values`:
  - 明らかな外れ値を含むデータで外れ値インデックスが検出されることを確認する
  - 正常値のみのデータで空 Vec が返ることを確認する

## Files to Modify

| ファイル | 操作 | 内容 |
|---|---|---|
| `fav/src/test_framework.rs` | 追記 | `DistributionStats` / `compute_distribution_stats` / `StatisticalCheck` / `detect_outliers` / `format_distribution_report` |
| `fav/src/driver.rs` | 追記 | `mod v81200_tests`（テスト 2 件） |

## Error Codes

新規エラーコードなし。

## 注記

- `std_dev` は **母標準偏差**（`n` で割る）で計算する（標本標準偏差 `n-1` ではない）。
- `detect_outliers` の `column` フィールドは `StatisticalCheck` の意味的なメタデータとして保持するが、
  `detect_outliers` は `values` を直接受け取るため `column` を参照しない。
- `format_distribution_report` の小数点桁数は `.3`（小数点以下 3 桁）とする。
