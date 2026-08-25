# spec: v84.3.0 — 品質統合ショーケース（`fav quality` E2E）

## Background

> **テスト数注記**: ロードマップ計画値は 3,901/3,903 だったが、code-reviewer 対応の
> 累積により実際のベースは **3,913 tests**（v84.2.0 完了時点）。
> v84.3.0 完了目標は **3,915 tests**（+2）。

v84.2.0 でショーケースに Test-Driven Data 1.0（TestSuite / StageTestCase / GoldenDataset /
SchemaSnapshot）を統合した。v84.3.0 では Sprint 2「Data Quality 2.0」の機能
（QualityCheck / QualityGate / AnomalyDetector）を `pipeline.fav` に統合し、
ショーケースが品質チェックを示すことを確認する。

## Goals

1. `infra/e2e-demo/favnir4-showcase/pipeline.fav` に Quality 統合セクションを追加する
   - `QualityCheck` + `run_quality_check` による行単位品質チェック
   - `QualityGate` + `evaluate_quality_gate` によるパイプライン停止条件チェック
   - `AnomalyDetector` + `detect_anomaly` による外れ値検知
2. Rust テスト 2 件でショーケースの内容を検証する
   - `showcase_quality_gate_passes` — QualityGate / QualityCheck の存在確認
   - `showcase_anomaly_detector_integrated` — AnomalyDetector / detect_anomaly の存在確認

## Syntax / API Examples（実際の型定義に基づく）

### pipeline.fav への追加セクション

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

### v84300_tests（Rust テスト）

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

## 実際の型定義（参照）

以下は Rust 実装シグネチャ。Favnir 側では `Vec<T>` → `List<T>` に読み替える。

| 型 / 関数 | Rust シグネチャ | Favnir 側の表記 |
|---|---|---|
| `QualityCheck` | `rules: Vec<QualityRule>` | `rules: List<QualityRule>` |
| `QualityGate` | `min_overall_score: f64`, `required_dimensions: Vec<QualityDimension>`, `min_dimension_score: f64` | `Vec<T>` → `List<T>` |
| `evaluate_quality_gate` | `(gate: &QualityGate, score: &QualityScore) -> GateDecision` | 同名関数 |
| `AnomalyDetector` | `baseline_stats: DistributionStats`, `z_threshold: f64` | 同名フィールド |
| `detect_anomaly` | `(detector: &AnomalyDetector, value: f64) -> AnomalyResult` | 同名関数 |
| `DistributionStats` | `mean: f64`, `std_dev: f64`, `min: f64`, `max: f64`, `count: usize` | 同名フィールド |

> **注意**: ロードマップ記載の `QualityGate.strict()` は Rust 側では `QualityGate::strict()`
> として存在する（`impl QualityGate` に `min_overall_score: 0.9` の厳格ゲートを返す関数）。
> ただし Favnir 構文から Rust `impl` メソッドを直接呼び出す手段がないため、ショーケースでは
> struct 初期化構文 `QualityGate { min_overall_score: 0.9, ... }` で表現する。

## Success Criteria

- `infra/e2e-demo/favnir4-showcase/pipeline.fav` に `QualityGate`・`QualityCheck`・
  `AnomalyDetector`・`detect_anomaly` の各識別子が含まれること
- `cargo test` が 3,915 tests pass（+2）、0 failures であること

## Error Codes

なし（本バージョンはファイル更新のみ）

## Files to Modify / Create

### 更新
- `infra/e2e-demo/favnir4-showcase/pipeline.fav` — Quality 統合セクションを末尾に追加

### 追記
- `fav/src/driver.rs` — `v84300_tests` モジュール追加（2 テスト）
- `CHANGELOG.md` — v84.3.0 エントリ追加

### パス起点（v84.1.0 から踏襲）

`v84300_tests` は `include_str!("../../infra/...")` を使用する。
パス起点は `fav/src/driver.rs` の位置（`fav/src/`）。`driver.rs` を移動した場合はパスを更新すること。
