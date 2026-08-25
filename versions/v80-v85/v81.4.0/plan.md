# Plan: v81.4.0 — 品質スコアリング（`QualityScore` / `QualityDimension`）

## Step 1: 前提確認

- `cargo test` を実行し、3849 tests, 0 failures を確認する
- `fav/src/test_framework.rs` に v81.3.0 の `format_drift_report` が定義済みであることを確認する

## Step 2: `fav/src/test_framework.rs` に追記

`format_drift_report` の定義の直後に以下を追加する。

```rust
// ── v81.4.0: QualityScore / QualityDimension ──────────────────────────────────

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
/// 空スライスまたはすべての `weight = 0.0` のとき `overall = 0.0`。
pub fn compute_quality_score(dimensions: &[DimensionScore]) -> QualityScore {
    let total_weight: f64 = dimensions.iter().map(|d| d.weight).sum();
    let overall = if total_weight == 0.0 {
        0.0
    } else {
        dimensions.iter().map(|d| d.score * d.weight).sum::<f64>() / total_weight
    };
    QualityScore { dimensions: dimensions.to_vec(), overall }
}

/// スコアを人間向けの文字列に変換する。
///
/// 出力形式: `"overall={:.3} grade={grade} dimensions={count}"`
pub fn format_quality_score(score: &QualityScore) -> String {
    format!(
        "overall={:.3} grade={} dimensions={}",
        score.overall,
        quality_grade(score),
        score.dimensions.len(),
    )
}

/// `overall` から品質グレードを返す。
///
/// - A: overall >= 0.90
/// - B: overall >= 0.80
/// - C: overall >= 0.70
/// - D: overall >= 0.60
/// - F: overall < 0.60
pub fn quality_grade(score: &QualityScore) -> &'static str {
    match score.overall {
        x if x >= 0.90 => "A",
        x if x >= 0.80 => "B",
        x if x >= 0.70 => "C",
        x if x >= 0.60 => "D",
        _              => "F",
    }
}
```

## Step 3: `fav/src/driver.rs` に `mod v81400_tests` を追加

`mod v81300_tests { ... }` の直後に追加する。

```rust
#[cfg(test)]
mod v81400_tests {
    use fav_core::test_framework::*;

    #[test]
    fn quality_score_weighted_average() {
        let dims = vec![
            DimensionScore { dimension: QualityDimension::Completeness, score: 0.9, weight: 2.0 },
            DimensionScore { dimension: QualityDimension::Accuracy,     score: 0.8, weight: 1.0 },
        ];
        let qs = compute_quality_score(&dims);
        // (0.9*2.0 + 0.8*1.0) / (2.0+1.0) = 2.6/3.0 ≈ 0.86667
        let expected = (0.9 * 2.0 + 0.8 * 1.0) / 3.0;
        assert!((qs.overall - expected).abs() < 1e-9,
            "overall mismatch: {} vs {}", qs.overall, expected);
        assert_eq!(qs.dimensions.len(), 2);
        let report = format_quality_score(&qs);
        assert!(report.contains("overall=0.867"), "report should contain overall=0.867: {report}");
        assert!(report.contains("grade=B"),       "report should contain grade=B: {report}");
        assert!(report.contains("dimensions=2"),  "report should contain dimensions=2: {report}");
    }

    #[test]
    fn quality_grade_a_when_perfect() {
        let perfect = QualityScore { dimensions: vec![], overall: 1.0 };
        assert_eq!(quality_grade(&perfect), "A", "overall=1.0 should be grade A");

        let boundary_a = QualityScore { dimensions: vec![], overall: 0.9 };
        assert_eq!(quality_grade(&boundary_a), "A", "overall=0.90 should be grade A (inclusive)");

        let just_b = QualityScore { dimensions: vec![], overall: 0.89 };
        assert_eq!(quality_grade(&just_b), "B", "overall=0.89 should be grade B");

        let zero = QualityScore { dimensions: vec![], overall: 0.0 };
        assert_eq!(quality_grade(&zero), "F", "overall=0.0 should be grade F");
    }
}
```

## Step 4: `cargo test` で全 pass 確認

```
cargo test 2>&1 | grep "test result"
# 期待: 3851 tests, 0 failures
```

## Step 5: CHANGELOG 更新

`CHANGELOG.md` の先頭に v81.4.0 エントリを追加する。

## Step 6: CI 事前確認

以下はすべて `fav/` ディレクトリで実行する。

```
cargo clippy --locked -- -D warnings
./target/debug/fav fmt --check self/compiler.fav
./target/debug/fav fmt --check self/checker.fav
```
