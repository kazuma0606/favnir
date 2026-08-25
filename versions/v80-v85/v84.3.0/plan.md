# plan: v84.3.0 — 品質統合ショーケース（`fav quality` E2E）

## 実装ステップ（依存順）

### Step 1: 事前確認

- `cargo test` を実行し、3,913 tests, 0 failures を確認する（前提: v84.2.0 完了済み）
- `Cargo.toml` バージョンが `84.0.0` であることを確認する
  （v84.x マイナーバージョンは Cargo.toml を更新しない慣例。v85.0.0 宣言時に一括更新する。
   この慣例は v84.0.0 宣言時から適用。v84.3.0 で独自に変更してはならない）
- `fav/src/driver.rs` に `mod v84200_tests` が存在することを確認する

### Step 2: pipeline.fav に Quality 統合セクションを追加

現在の `pipeline.fav`（4 ステージ骨格 + テスト統合セクション）の末尾に、
QualityCheck / QualityGate / AnomalyDetector を使ったデモ関数 3 本を追加する。

追加する関数（実際の型定義に基づく）:

```favnir
-- ── 品質統合セクション（Sprint 2: Data Quality 2.0）────────────────────

fn showcase_quality_check(rows: List<Row>) -> Result<List<QualityViolation>, String> {
    -- QualityCheck: run_quality_check で行単位に品質ルールを適用
    bind check <- QualityCheck {
        rules: List.of(QualityRule {
            column: "0",
            kind: QualityRuleKind.NotNull,
            severity: RuleSeverity.Error,
        }),
    }
    Result.ok(run_quality_check(check, rows))
}

fn showcase_quality_gate(score: QualityScore) -> Result<GateDecision, String> {
    -- QualityGate: evaluate_quality_gate でパイプライン停止条件を判定
    bind gate <- QualityGate {
        min_overall_score: 0.9,
        required_dimensions: List.empty(),
        min_dimension_score: 0.8,
    }
    Result.ok(evaluate_quality_gate(gate, score))
}

fn showcase_anomaly_detection(value: Float) -> Result<AnomalyResult, String> {
    -- AnomalyDetector: detect_anomaly で Z スコア外れ値検知（Sprint 2 v81.2.0）
    bind stats    <- DistributionStats { mean: 50.0, std_dev: 10.0, min: 0.0, max: 100.0, count: 100 }
    bind detector <- AnomalyDetector { baseline_stats: stats, z_threshold: 2.0 }
    Result.ok(detect_anomaly(detector, value))
}
```

### Step 3: driver.rs に v84300_tests を追加

`mod v84200_tests` の直後に `#[cfg(test)] mod v84300_tests` を追加する。

```rust
#[cfg(test)]
mod v84300_tests {
    #[test]
    fn showcase_quality_gate_passes() {
        let content = include_str!("../../infra/e2e-demo/favnir4-showcase/pipeline.fav");
        assert!(content.contains("QualityGate"), "pipeline.fav should include QualityGate");
        assert!(content.contains("QualityCheck"), "pipeline.fav should include QualityCheck");
    }

    #[test]
    fn showcase_anomaly_detector_integrated() {
        let content = include_str!("../../infra/e2e-demo/favnir4-showcase/pipeline.fav");
        assert!(content.contains("AnomalyDetector"), "pipeline.fav should include AnomalyDetector");
        assert!(content.contains("detect_anomaly"), "pipeline.fav should include detect_anomaly");
    }
}
```

### Step 4: cargo test で全 pass 確認

`cargo test 2>&1 | grep "test result"` を実行し、3,915 tests, 0 failures を確認する。

### Step 5: CHANGELOG 更新

`CHANGELOG.md` の先頭に v84.3.0 エントリを追加する。

> 注: `site/` MDX 追加は v84.6.0 で一括実施するため本バージョンでは省略する。

### Step 6: CI 事前確認

- `cargo clippy --locked -- -D warnings` が pass することを確認する
- `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
