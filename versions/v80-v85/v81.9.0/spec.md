# v81.9.0 — 安定化・コードフリーズ

Date: 2026-08-20
Status: 計画中

---

## Background

v81.1〜v81.8 で実装した Data Quality 2.0 スプリントの全機能を統合確認する。
新機能追加は行わず、バグ修正と統合テストのみを受け入れる。

実装済み機能の一覧:
- v81.1.0: `QualityRule` / `QualityCheck` / `run_quality_check`
- v81.2.0: `DistributionStats` / `StatisticalCheck` / `detect_outliers`
- v81.3.0: `SchemaDriftDetector` / `DriftTolerance` / `DriftResult` / `detect_schema_drift`
- v81.4.0: `QualityDimension` / `DimensionScore` / `QualityScore` / `compute_quality_score`
- v81.5.0: `ProvenanceQualityEntry` / `ProvenanceQualityReport` / `worst_quality_source`
- v81.6.0: `QualityGate` / `GateDecision` / `evaluate_quality_gate`
- v81.7.0: `ReportFormat` / `QualityReportOptions` / `build_quality_report` / `cmd_quality_report`
- v81.8.0: `AnomalyDetector` / `AnomalyResult` / `detect_anomaly` / `scan_for_anomalies`

---

## Goals

1. v81.1〜v81.8 の全テストが `cargo test` で pass することを確認する
2. `QualityGate` + `SchemaDriftDetector` 連携の統合テストを追加する
3. v81.1〜v81.8 の主要機能を一気通貫で呼び出す全体統合テストを追加する
4. バグ修正のみ受け入れ（新 API / 新 struct の追加なし）

---

## Test Scenarios

### 統合テスト 1: `data_quality_full_sprint_all_stable`

v81.1〜v81.8 の主要関数を連鎖して呼び出し、クラッシュなく動作することを確認する。

```rust
// QualityRule → run_quality_check → compute_quality_score → build_quality_report → evaluate_quality_gate
let rule = QualityRule { column: "0".into(), kind: QualityRuleKind::NotNull, severity: RuleSeverity::Error }; // column は列インデックス文字列
let check = QualityCheck { rules: vec![rule] };
let rows = vec![vec!["25".into()], vec!["".into()]];
let violations = run_quality_check(&check, &rows);
assert_eq!(violations.len(), 1);

let dims = vec![DimensionScore { dimension: QualityDimension::Completeness, score: 0.8, weight: 1.0 }];
let score = compute_quality_score(&dims);
let opts = QualityReportOptions { format: ReportFormat::Text, include_violations: true, include_stats: false };
let report = build_quality_report(&check, &rows, &opts);
assert!(report.contains("violations"));

let gate = QualityGate::permissive();
let decision = evaluate_quality_gate(&gate, &score);
assert!(matches!(decision, GateDecision::Pass));
```

### 統合テスト 2: `quality_gate_and_drift_detector_integrated`

`SchemaDriftDetector` でドリフトを検出し、低品質スコアと組み合わせてゲートが Fail を返すことを確認する。

```rust
let col_a = ColumnSnapshot { name: "id".into(), type_name: "Int".into(), nullable: false };
let col_b = ColumnSnapshot { name: "name".into(), type_name: "Str".into(), nullable: false };
let baseline = SchemaSnapshot { pipeline_name: "p".into(), columns: vec![col_a.clone(), col_b] };
let current  = SchemaSnapshot { pipeline_name: "p".into(), columns: vec![col_a] }; // col_b 削除

let detector = SchemaDriftDetector { baseline, tolerance: DriftTolerance::Strict };
let drift = detect_schema_drift(&detector, &current);
assert!(drift.has_drift);

// ドリフトがある → 品質スコアが低 → strict ゲートが Fail
let dims = vec![DimensionScore { dimension: QualityDimension::Completeness, score: 0.5, weight: 1.0 }];
let score = compute_quality_score(&dims);
let gate = QualityGate::strict();
let decision = evaluate_quality_gate(&gate, &score);
assert!(matches!(decision, GateDecision::Fail(_)));
```

---

## Success Criteria

- `cargo test` 全 pass（3,861 tests = 3,859 + 2）
- 新規テスト 2 件:
  - `data_quality_full_sprint_all_stable`
  - `quality_gate_and_drift_detector_integrated`
- バグ修正以外の変更なし

---

## Files to Modify

| ファイル | 変更内容 |
|---|---|
| `fav/src/driver.rs` | `#[cfg(test)] mod v81900_tests` を新規追加し、統合テスト 2 件を収める |

> **注意**: 他バージョン同様、テストは `driver.rs` に追加する（`test_framework.rs` ではない）。
