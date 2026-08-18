# Roadmap v81.1.0 〜 v82.0.0 — Data Quality 2.0

Date: 2026-08-16
Status: 未着手（v81.0.0 完了後に開始）

マスターロードマップ: [roadmap-v80.1-v85.0.md](roadmap-v80.1-v85.0.md)

---

## 前提

- 直前完了: v81.0.0「Test-Driven Data 1.0 宣言」（tests = 3,831）
- 本スプリントは Quality-First Era の第 2 スプリント
- 目標: v82.0.0「Data Quality 2.0 宣言」（tests = 3,853）
- **依存**: v80.7.0 で `SchemaSnapshot` / `SchemaSnapshotDiff` / `ColumnSnapshot` が導入済みであること（v81.3.0 の `SchemaDriftDetector` が参照する）

### スプリントの性格

v37.0「Data Quality First」（`DataQualityReport` / `validate_schema` の基盤）を土台に、
データ品質ルールを **型** として表現し直す。`QualityRule` / `QualityScore` を軸に、
`PipelineInvariant`（Favnir 3.0）・`ProvenanceTag`（Favnir 3.0）と統合して
「品質の証明がパイプラインの型システムの一部になる」状態を実現する。
A（新機能）60% + B（統合）40% の構成。

---

## バージョン一覧

| バージョン | 内容 | テスト数 | 状態 |
|---|---|---|---|
| v81.1.0 | `QualityRule` / `QualityCheck` 型基盤 | 3831 + 2 = 3833 | 未着手 |
| v81.2.0 | 統計的品質チェック（`StatisticalCheck` / 分布・外れ値） | 3833 + 2 = 3835 | 未着手 |
| v81.3.0 | スキーマドリフト検出（`SchemaDriftDetector`） | 3835 + 2 = 3837 | 未着手 |
| v81.4.0 | 品質スコアリング（`QualityScore` / `QualityDimension`） | 3837 + 2 = 3839 | 未着手 |
| v81.5.0 | 来歴付き品質レポート（Provenance + Quality 統合） | 3839 + 2 = 3841 | 未着手 |
| v81.6.0 | 品質ゲート（`QualityGate` / パイプライン停止条件） | 3841 + 2 = 3843 | 未着手 |
| v81.7.0 | `fav quality report` コマンド | 3843 + 2 = 3845 | 未着手 |
| v81.8.0 | 異常検知（`AnomalyDetector` / Z スコアベース） | 3845 + 2 = 3847 | 未着手 |
| v81.9.0 | 安定化・コードフリーズ | 3847 + 2 = 3849 | 未着手 |
| v82.0.0 | Data Quality 2.0 宣言 ★クリーンアップ | 3849 + 4 = 3853 | 未着手 |

---

## v81.1.0 — `QualityRule` / `QualityCheck` 型基盤

品質ルールをファーストクラスの型として表現する基盤を構築する。

**実装内容:**
- `QualityRuleKind` enum（`NotNull`, `Unique`, `Range { min: f64, max: f64 }`, `Regex(String)`, `Custom(String)`）
- `QualityRule` 構造体（`column: String`, `kind: QualityRuleKind`, `severity: RuleSeverity`）
- `RuleSeverity` enum（`Error` / `Warning`）
- `QualityCheck` 構造体（`rules: Vec<QualityRule>`）
- `QualityViolation` 構造体（`rule: QualityRule`, `row_index: usize`, `actual: String`）
- `run_quality_check(check: &QualityCheck, rows: &[Vec<String>]) -> Vec<QualityViolation>`

**完了条件**: Rust テスト 2 件（3831 + 2 = 3833）
- `quality_rule_not_null_catches_violation`
- `quality_check_returns_all_violations`

---

## v81.2.0 — 統計的品質チェック（`StatisticalCheck`）

数値カラムの分布・外れ値を統計的に検出する型を追加する。

**実装内容:**
- `DistributionStats` 構造体（`mean: f64`, `std_dev: f64`, `min: f64`, `max: f64`, `count: usize`）
- `compute_distribution_stats(values: &[f64]) -> DistributionStats`
- `StatisticalCheck` 構造体（`column: String`, `z_score_threshold: f64`）
- `detect_outliers(check: &StatisticalCheck, values: &[f64]) -> Vec<usize>`
- `format_distribution_report(stats: &DistributionStats) -> String`

**完了条件**: Rust テスト 2 件（3833 + 2 = 3835）
- `distribution_stats_computed_correctly`
- `outlier_detection_finds_extreme_values`

---

## v81.3.0 — スキーマドリフト検出（`SchemaDriftDetector`）

実行時のスキーマ変化を検出し、品質チェックを自動トリガーする仕組みを作る。

**実装内容:**
- `SchemaDriftDetector` 構造体（`baseline: SchemaSnapshot`, `tolerance: DriftTolerance`）
- `DriftTolerance` enum（`Strict`（追加も禁止）/ `Additive`（追加のみ許可）/ `Permissive`）
- `detect_schema_drift(detector: &SchemaDriftDetector, current: &SchemaSnapshot) -> DriftResult`
- `DriftResult` 構造体（`has_drift: bool`, `severity: RuleSeverity`, `diff: SchemaSnapshotDiff`）
- `format_drift_report(result: &DriftResult) -> String`

**完了条件**: Rust テスト 2 件（3835 + 2 = 3837）
- `drift_detector_strict_mode_catches_addition`
- `drift_detector_additive_mode_allows_new_column`

---

## v81.4.0 — 品質スコアリング（`QualityScore` / `QualityDimension`）

複数の品質次元をスコアリングして総合品質スコアを算出する。

**実装内容:**
- `QualityDimension` enum（`Completeness` / `Consistency` / `Timeliness` / `Accuracy` / `Validity`）
- `DimensionScore` 構造体（`dimension: QualityDimension`, `score: f64`, `weight: f64`）
- `QualityScore` 構造体（`dimensions: Vec<DimensionScore>`, `overall: f64`）
- `compute_quality_score(dimensions: &[DimensionScore]) -> QualityScore`
- `format_quality_score(score: &QualityScore) -> String`
- `quality_grade(score: &QualityScore) -> &'static str`（A/B/C/D/F）

**完了条件**: Rust テスト 2 件（3837 + 2 = 3839）
- `quality_score_weighted_average`
- `quality_grade_a_when_perfect`

---

## v81.5.0 — 来歴付き品質レポート（Provenance + Quality 統合）

Favnir 3.0 の `ProvenanceTag` と品質スコアを統合し、
「どのソースから来たデータがどの品質スコアを持つか」を追跡する。

**実装内容:**
- `ProvenanceQualityEntry` 構造体（`source_name: String`, `provenance_hash: String`, `quality_score: f64`）（`provenance_hash` は本バージョンでは `String` スタブとして扱う）
- `ProvenanceQualityReport` 構造体（`entries: Vec<ProvenanceQualityEntry>`, `pipeline_name: String`）
- `build_provenance_quality_report(entries: Vec<ProvenanceQualityEntry>, pipeline: &str) -> ProvenanceQualityReport`
- `format_provenance_quality_report(report: &ProvenanceQualityReport) -> String`
- `worst_quality_source(report: &ProvenanceQualityReport) -> Option<&ProvenanceQualityEntry>`

**完了条件**: Rust テスト 2 件（3839 + 2 = 3841）
- `provenance_quality_report_built`
- `worst_source_identified`

---

## v81.6.0 — 品質ゲート（`QualityGate` / パイプライン停止条件）

品質スコアが閾値を下回った場合にパイプラインを停止する仕組みを作る。

**実装内容:**
- `QualityGate` 構造体（`min_overall_score: f64`, `required_dimensions: Vec<QualityDimension>`, `min_dimension_score: f64`）
- `GateDecision` enum（`Pass` / `Fail(String)` / `Warn(String)`）
- `evaluate_quality_gate(gate: &QualityGate, score: &QualityScore) -> GateDecision`
- `format_gate_decision(decision: &GateDecision) -> String`
- `QualityGate::strict() -> QualityGate`（全ディメンション 0.9 以上）
- `QualityGate::permissive() -> QualityGate`（overall 0.6 以上）

**完了条件**: Rust テスト 2 件（3841 + 2 = 3843）
- `quality_gate_fails_below_threshold`
- `quality_gate_passes_above_threshold`

---

## v81.7.0 — `fav quality report` コマンド

品質チェック結果を人間が読める形式で出力するコマンドを追加する。

**実装内容:**
- `QualityReportOptions` 構造体（`format: ReportFormat`, `include_violations: bool`, `include_stats: bool`）
- `ReportFormat` enum（`Text` / `Json` / `Markdown`）
- `build_quality_report(check: &QualityCheck, rows: &[Vec<String>], opts: &QualityReportOptions) -> String`
- `cmd_quality_report` 関数（`fav quality report` コマンドハンドラ）

**完了条件**: Rust テスト 2 件（3843 + 2 = 3845）
- `quality_report_text_format`
- `quality_report_json_format`

---

## v81.8.0 — 異常検知（`AnomalyDetector` / Z スコアベース）

時系列・バッチ間の異常値を Z スコアで検出する型を追加する。

**実装内容:**
- `AnomalyDetector` 構造体（`baseline_stats: DistributionStats`, `z_threshold: f64`）（依存: v81.2.0 の `DistributionStats` / `compute_distribution_stats`）
- `AnomalyResult` 構造体（`is_anomaly: bool`, `z_score: f64`, `value: f64`）
- `AnomalyDetector::from_baseline(values: &[f64], z_threshold: f64) -> AnomalyDetector`
- `detect_anomaly(detector: &AnomalyDetector, value: f64) -> AnomalyResult`
- `scan_for_anomalies(detector: &AnomalyDetector, values: &[f64]) -> Vec<AnomalyResult>`
- `format_anomaly_report(results: &[AnomalyResult]) -> String`

**完了条件**: Rust テスト 2 件（3845 + 2 = 3847）
- `anomaly_detector_catches_outlier`
- `anomaly_scan_returns_all_results`

---

## v81.9.0 — 安定化・コードフリーズ

v81.1〜v81.8 の全スプリント統合確認。バグ修正のみ。

**実装内容:**
- v81.1〜v81.8 の全テスト通過確認（`cargo test` 全 pass）
- `fav quality report` コマンド E2E 動作確認（サンプル CSV 入力 → `QualityViolation` / `QualityScore` 出力テキスト検証）
- `QualityGate` + `SchemaDriftDetector` 連携確認
- バグ修正のみ受け入れ（新機能追加なし）

**完了条件**: Rust テスト 2 件（3847 + 2 = 3849）
- `data_quality_full_sprint_all_stable`
- `quality_gate_and_drift_detector_integrated`

---

## v82.0.0 — Data Quality 2.0 宣言 ★クリーンアップ

**宣言文**:
> 「品質が型になった。外れ値はコンパイル時に検出され、
>  スキーマドリフトはパイプライン起動前に止まる。
>  Favnir のデータは今、品質を型で保証する。」

**クリーンアップ作業:**
- `cargo clean` 実施
- `Cargo.toml` バージョンを `82.0.0` に更新
- `CHANGELOG.md` / `MILESTONE.md` / `README.md` 更新
- `versions/current.md` の現行マスターロードマップが `roadmap-v80.1-v85.0.md` を指していることを確認してから更新
- `roadmap-v80.1-v85.0.md` の Sprint 2 バージョン一覧テーブルを全行「完了」に更新

**完了条件**: `v82000_tests` 4 件（3849 + 4 = 3853）
- `cargo_toml_version_is_82_0_0`
- `changelog_has_v82_0_0`
- `milestone_has_data_quality_2`
- `readme_mentions_quality_gate`

---

## テスト数推移（本スプリント）

| バージョン | テスト数 | 増加 |
|---|---|---|
| v81.0.0（ベース） | 3,831 | — |
| v81.1.0 | 3,833 | +2 |
| v81.2.0 | 3,835 | +2 |
| v81.3.0 | 3,837 | +2 |
| v81.4.0 | 3,839 | +2 |
| v81.5.0 | 3,841 | +2 |
| v81.6.0 | 3,843 | +2 |
| v81.7.0 | 3,845 | +2 |
| v81.8.0 | 3,847 | +2 |
| v81.9.0 | 3,849 | +2 |
| v82.0.0（宣言） | 3,853 | +4 |

**本スプリント合計**: +22 tests（3,831 → 3,853）

---

## 参考リンク

- マスターロードマップ: [roadmap-v80.1-v85.0.md](roadmap-v80.1-v85.0.md)
- 前スプリント: [roadmap-v80.1-v81.0.md](roadmap-v80.1-v81.0.md)
- 次スプリント: [roadmap-v82.1-v83.0.md](roadmap-v82.1-v83.0.md)
- 達成宣言: `MILESTONE.md`
- 進行状況: `versions/current.md`
