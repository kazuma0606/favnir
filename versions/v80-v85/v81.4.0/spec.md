# Spec: v81.4.0 — 品質スコアリング（`QualityScore` / `QualityDimension`）

## Background

v81.1.0 で `QualityRule` / `RuleSeverity` 型基盤を構築した。
本バージョンでは複数の品質次元をスコアリングして総合品質スコアを算出する型を追加する。

ロードマップ: `versions/roadmap/roadmap-v81.1-v82.0.md`（v81.4.0 セクション）

> **テスト数**: 実際のベースは **3849**（v81.3.0 完了後）。
> 本バージョンの完了条件は **3849 + 2 = 3851**。

## Goals

- `QualityDimension` enum を `test_framework.rs` に追加する
- `DimensionScore` 構造体を追加する
- `QualityScore` 構造体を追加する
- `compute_quality_score(dimensions: &[DimensionScore]) -> QualityScore` を実装する
- `format_quality_score(score: &QualityScore) -> String` を実装する
- `quality_grade(score: &QualityScore) -> &'static str` を実装する（A/B/C/D/F）
- テスト 2 件を追加して **3851 tests** を達成する

## API / Type Definitions

```rust
// fav/src/test_framework.rs（既存ファイルに追記）

/// 品質の評価次元。
#[derive(Debug, Clone, PartialEq)]
pub enum QualityDimension {
    Completeness,
    Consistency,
    Timeliness,
    Accuracy,
    Validity,
}

/// 単一品質次元のスコア。
///
/// `score` は 0.0〜1.0 の範囲。`weight` は重みづけ係数（0.0 以上）。
#[derive(Debug, Clone)]
pub struct DimensionScore {
    pub dimension: QualityDimension,
    pub score: f64,
    pub weight: f64,
}

/// 複数次元の総合品質スコア。
///
/// `overall` は加重平均: `Σ(score * weight) / Σ(weight)`。
/// `dimensions` が空のとき `overall = 0.0`。
#[derive(Debug, Clone)]
pub struct QualityScore {
    pub dimensions: Vec<DimensionScore>,
    pub overall: f64,
}

/// `dimensions` の加重平均を計算して `QualityScore` を返す。
///
/// 空スライスのとき `overall = 0.0` / `dimensions = vec![]`。
/// すべての `weight = 0.0` のとき `overall = 0.0`（ゼロ除算回避）。
pub fn compute_quality_score(dimensions: &[DimensionScore]) -> QualityScore;

/// スコアを人間向けの文字列に変換する。
///
/// 出力形式: `"overall={:.3} grade={grade} dimensions={count}"`
pub fn format_quality_score(score: &QualityScore) -> String;

/// `overall` から品質グレードを返す。
///
/// - A: overall >= 0.90
/// - B: overall >= 0.80
/// - C: overall >= 0.70
/// - D: overall >= 0.60
/// - F: overall < 0.60
pub fn quality_grade(score: &QualityScore) -> &'static str;
```

### 出力例

```rust
// 概念説明（Favnir 風疑似コード）
bind dims <- vec![
    DimensionScore { dimension: QualityDimension::Completeness, score: 0.9, weight: 2.0 },
    DimensionScore { dimension: QualityDimension::Accuracy,     score: 0.8, weight: 1.0 },
];
bind qs <- compute_quality_score(&dims);
// qs.overall == (0.9 * 2.0 + 0.8 * 1.0) / (2.0 + 1.0) = 2.6 / 3.0 ≈ 0.8667

bind grade <- quality_grade(&qs);
// grade == "B"

bind report <- format_quality_score(&qs);
// "overall=0.867 grade=B dimensions=2"
```

## Success Criteria

- `cargo test` が **3851 tests**, 0 failures
- `quality_score_weighted_average`:
  - `Completeness(score=0.9, weight=2.0)` + `Accuracy(score=0.8, weight=1.0)` で `overall ≈ 0.8667`（誤差 1e-9 以内）
  - `format_quality_score` の出力に `"overall=0.867"` と `"grade=B"` と `"dimensions=2"` が含まれること
- `quality_grade_a_when_perfect`:
  - `overall = 1.0` で `quality_grade` が `"A"` を返すこと
  - `overall = 0.9` で `"A"` を返すこと（境界値）
  - `overall = 0.89` で `"B"` を返すこと
  - `overall = 0.0` で `"F"` を返すこと

## Files to Modify

| ファイル | 操作 | 内容 |
|---|---|---|
| `fav/src/test_framework.rs` | 追記 | `QualityDimension` / `DimensionScore` / `QualityScore` / `compute_quality_score` / `format_quality_score` / `quality_grade` |
| `fav/src/driver.rs` | 追記 | `mod v81400_tests`（テスト 2 件） |

## Error Codes

新規エラーコードなし。

## 注記

- `compute_quality_score` は `dimensions` を clone して `QualityScore.dimensions` に格納する。
- `weight` の総和が 0.0 のとき `overall = 0.0`（ゼロ除算回避）。
- `format_quality_score` は内部で `quality_grade` を呼んでグレードを埋め込む。
- グレード境界は inclusive（`>= 0.90` → A、`>= 0.80` → B、など）。
