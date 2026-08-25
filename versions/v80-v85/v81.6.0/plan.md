# Plan: v81.6.0 — 品質ゲート（`QualityGate` / パイプライン停止条件）

## Step 1: 前提確認

- `cargo test` を実行し、3853 tests, 0 failures を確認する
- `fav/src/test_framework.rs` に v81.4.0 の `QualityScore` / `QualityDimension` が定義済みであることを確認する

## Step 2: `fav/src/test_framework.rs` に追記

`worst_quality_source` の定義の直後に以下を追加する。

```rust
// ── v81.6.0: QualityGate ──────────────────────────────────────────────────────

/// 品質ゲートの評価結果。
///
/// - `Pass`: すべての条件を満たした
/// - `Fail(String)`: 最初に失敗した条件の説明
/// - `Warn(String)`: 条件は満たしているが注意が必要（将来拡張用、現在は未使用）
#[derive(Debug, Clone, PartialEq)]
pub enum GateDecision {
    Pass,
    Fail(String),
    Warn(String),
}

/// パイプライン停止条件を定義する品質ゲート。
#[derive(Debug, Clone)]
pub struct QualityGate {
    pub min_overall_score: f64,
    pub required_dimensions: Vec<QualityDimension>,
    pub min_dimension_score: f64,
}

impl QualityGate {
    /// 全ディメンション 0.9 以上を要求する厳格ゲート。
    pub fn strict() -> QualityGate {
        QualityGate {
            min_overall_score: 0.9,
            required_dimensions: vec![
                QualityDimension::Completeness,
                QualityDimension::Consistency,
                QualityDimension::Timeliness,
                QualityDimension::Accuracy,
                QualityDimension::Validity,
            ],
            min_dimension_score: 0.9,
        }
    }

    /// overall 0.6 以上のみを要求する緩やかなゲート。
    pub fn permissive() -> QualityGate {
        QualityGate {
            min_overall_score: 0.6,
            required_dimensions: vec![],
            min_dimension_score: 0.6,
        }
    }
}

/// `gate` の条件に対して `score` を評価する。
///
/// 評価順序:
/// 1. overall が `min_overall_score` を下回ると `Fail`
/// 2. required_dimensions の各次元スコアが `min_dimension_score` を下回ると `Fail`
/// 3. すべて通過 → `Pass`
pub fn evaluate_quality_gate(gate: &QualityGate, score: &QualityScore) -> GateDecision {
    if score.overall < gate.min_overall_score {
        return GateDecision::Fail(format!(
            "overall score {:.3} below minimum {:.3}",
            score.overall, gate.min_overall_score,
        ));
    }
    for dim in &gate.required_dimensions {
        let dim_score = score.dimensions.iter()
            .find(|d| &d.dimension == dim)
            .map(|d| d.score);
        match dim_score {
            None => {
                return GateDecision::Fail(format!(
                    "dimension {:?} score not found (minimum {:.3})",
                    dim, gate.min_dimension_score,
                ));
            }
            Some(s) if s < gate.min_dimension_score => {
                return GateDecision::Fail(format!(
                    "dimension {:?} score {:.3} below minimum {:.3}",
                    dim, s, gate.min_dimension_score,
                ));
            }
            _ => {}
        }
    }
    GateDecision::Pass
}

/// `GateDecision` を人間向けの文字列に変換する。
pub fn format_gate_decision(decision: &GateDecision) -> String {
    match decision {
        GateDecision::Pass        => "PASS".to_string(),
        GateDecision::Fail(msg)   => format!("FAIL: {msg}"),
        GateDecision::Warn(msg)   => format!("WARN: {msg}"),
    }
}
```

## Step 3: `fav/src/driver.rs` に `mod v81600_tests` を追加

`mod v81500_tests { ... }` の直後に追加する。

```rust
#[cfg(test)]
mod v81600_tests {
    use fav_core::test_framework::*;

    #[test]
    fn quality_gate_fails_below_threshold() {
        let gate = QualityGate::permissive(); // min_overall_score = 0.6
        let score = QualityScore { dimensions: vec![], overall: 0.5 };
        let decision = evaluate_quality_gate(&gate, &score);
        assert!(matches!(decision, GateDecision::Fail(_)), "should fail: {decision:?}");
        let report = format_gate_decision(&decision);
        assert!(report.contains("FAIL"),  "report should contain FAIL: {report}");
        assert!(report.contains("0.500"), "report should mention actual score: {report}");
        assert!(report.contains("0.600"), "report should mention minimum score: {report}");
    }

    #[test]
    fn quality_gate_passes_above_threshold() {
        // permissive: overall 0.8 >= 0.6 → Pass
        let gate_perm = QualityGate::permissive();
        let score_high = QualityScore { dimensions: vec![], overall: 0.8 };
        let decision_perm = evaluate_quality_gate(&gate_perm, &score_high);
        assert_eq!(decision_perm, GateDecision::Pass, "permissive gate should pass: {decision_perm:?}");
        assert_eq!(format_gate_decision(&GateDecision::Pass), "PASS");

        // strict: overall 0.95, 全次元 0.95 → Pass
        let gate_strict = QualityGate::strict();
        let dims = vec![
            DimensionScore { dimension: QualityDimension::Completeness, score: 0.95, weight: 1.0 },
            DimensionScore { dimension: QualityDimension::Consistency,  score: 0.95, weight: 1.0 },
            DimensionScore { dimension: QualityDimension::Timeliness,   score: 0.95, weight: 1.0 },
            DimensionScore { dimension: QualityDimension::Accuracy,     score: 0.95, weight: 1.0 },
            DimensionScore { dimension: QualityDimension::Validity,     score: 0.95, weight: 1.0 },
        ];
        let score_strict = QualityScore { dimensions: dims, overall: 0.95 };
        let decision_strict = evaluate_quality_gate(&gate_strict, &score_strict);
        assert_eq!(decision_strict, GateDecision::Pass, "strict gate should pass: {decision_strict:?}");
    }
}
```

## Step 4: `cargo test` で全 pass 確認

以下は `fav/` ディレクトリで実行する。

```
cargo test 2>&1 | grep "test result"
# 期待: 3855 tests, 0 failures
```

## Step 5: CHANGELOG 更新

`CHANGELOG.md` の先頭に v81.6.0 エントリを追加する。

## Step 6: CI 事前確認

以下はすべて `fav/` ディレクトリで実行する。

```
cargo clippy --locked -- -D warnings
./target/debug/fav fmt --check self/compiler.fav
./target/debug/fav fmt --check self/checker.fav
```
