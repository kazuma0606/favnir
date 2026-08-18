# Favnir ロードマップ v80.1〜v85.0 — Quality-First Era

Date: 2026-08-16
Status: 計画中（v80.0.0 完了時点）

---

## 背景と方針

v80.0.0「Favnir 3.0 宣言」をもって、Favnir は
「時間・来歴・正しさ・実行戦略をすべて型で語れる言語」になった。

しかし「型で正しさを語れる」ことと「正しさを自動的に検証できる」ことは別問題である。
データエンジニアが毎日直面する問いは：

- **このパイプラインは本当に正しいか？**（テスト不足）
- **データ品質が落ちたことをどう知るか？**（観測不足）
- **スキーマが変わったとき何が壊れるか？**（契約不足）

v80.1〜v85.0 では **"Quality-First"** をテーマに、
テスト・品質・契約・可観測性の 4 層を積み上げ、
v85.0「Favnir 4.0 宣言」で完成させる。

> **前フェーズとの関係**
> v37.0「Data Quality First」では品質ルールの基盤（`DataQualityReport` / `validate_schema`）を整備した。
> v82.0「Data Quality 2.0」はその延長として、品質を **型** として表現し（`QualityRule` / `QualityScore`）、
> `PipelineInvariant`（Favnir 3.0）・`ProvenanceTag`（Favnir 3.0）と統合することで
> 「品質の証明がパイプラインの型システムの一部になる」ことを目標とする。
> また `SlaContract`（v82.2.0）は Favnir 3.0 の `!Adaptive` / `!Cached` エフェクトと連携し、
> 「SLA を満たすための実行戦略を型で宣言する」形で実装する。

```
v81.0 — Test-Driven Data 1.0  : 「テストで証明できる」
v82.0 — Data Quality 2.0      : 「品質が型になる」
v83.0 — Pipeline Contracts 1.0: 「契約で繋がる」
v84.0 — Observability 2.0     : 「壊れる前に分かる」
v85.0 — Favnir 4.0 宣言       : 「Quality-First 言語の完成」
```

---

## テスト数推移（本スプリント全体）

| バージョン | テスト数 | 増加 |
|---|---|---|
| v80.0.0（ベース） | 3,809 | — |
| v80.1.0〜v80.9.0 | +2 × 9 = +18 | 3,827 |
| v81.0.0（宣言） | +4 | 3,831 |
| v81.1.0〜v81.9.0 | +2 × 9 = +18 | 3,849 |
| v82.0.0（宣言） | +4 | 3,853 |
| v82.1.0〜v82.9.0 | +2 × 9 = +18 | 3,871 |
| v83.0.0（宣言） | +4 | 3,875 |
| v83.1.0〜v83.9.0 | +2 × 9 = +18 | 3,893 |
| v84.0.0（宣言） | +4 | 3,897 |
| v84.1.0〜v84.9.0 | +2 × 9 = +18 | 3,915 |
| v85.0.0（宣言） | +4 | 3,919 |

---

## ━━━━━━━━━━━━━━━━━━━━━━━━━━
## Sprint 1: Test-Driven Data 1.0（v80.1〜v81.0）
## ━━━━━━━━━━━━━━━━━━━━━━━━━━

**テーマ**: `fav test` でパイプラインの正しさを型付きで証明する。

### バージョン一覧

| バージョン | 内容 | テスト数 | 状態 |
|---|---|---|---|
| v80.1.0 | `TestCase` / `TestSuite` 型基盤 + `fav test` スタブ | 3809 + 2 = 3811 | 未着手 |
| v80.2.0 | `GoldenDataset` / ゴールデンデータセット比較 | 3811 + 2 = 3813 | 未着手 |
| v80.3.0 | `TestFixture` / `DataFactory` モックデータ生成 | 3813 + 2 = 3815 | 未着手 |
| v80.4.0 | プロパティベーステスト（`PipelineInvariant` 連携） | 3815 + 2 = 3817 | 未着手 |
| v80.5.0 | ステージ単体テスト（`StageTestCase`） | 3817 + 2 = 3819 | 未着手 |
| v80.6.0 | テストカバレッジレポート（`fav test --coverage`） | 3819 + 2 = 3821 | 未着手 |
| v80.7.0 | スキーマスナップショットテスト（`SchemaSnapshot`） | 3821 + 2 = 3823 | 未着手 |
| v80.8.0 | CI 統合レポート（JUnit XML 出力 / `TestReport`） | 3823 + 2 = 3825 | 未着手 |
| v80.9.0 | 安定化・コードフリーズ | 3825 + 2 = 3827 | 未着手 |
| v81.0.0 | Test-Driven Data 1.0 宣言 ★クリーンアップ | 3827 + 4 = 3831 | 未着手 |

---

### v80.1.0 — `TestCase` / `TestSuite` 型基盤 + `fav test` スタブ

テストフレームワークの基盤型と `fav test` コマンドの骨格を作る。

**実装内容:**
- `TestStatus` enum（`Pass` / `Fail` / `Skip`）
- `TestCase` 構造体（`name: String`, `status: TestStatus`, `message: Option<String>`）
- `TestSuite` 構造体（`name: String`, `cases: Vec<TestCase>`）
- `run_test_suite(suite: &TestSuite) -> TestSuiteResult`
- `TestSuiteResult` 構造体（`passed: usize`, `failed: usize`, `skipped: usize`）
- `format_test_suite_result(result: &TestSuiteResult) -> String`
- `cmd_test(args: &[String]) -> i32` スタブ（`fav test` コマンド骨格、`main.rs` の match アームに追加）

**完了条件**: Rust テスト 2 件（3809 + 2 = 3811）
- `test_suite_type_exists`
- `test_case_run_formats_result`

---

### v80.2.0 — `GoldenDataset` / ゴールデンデータセット比較

期待値ファイルとパイプライン出力を比較する型。

**実装内容:**
- `GoldenDataset` 構造体（`name: String`, `rows: Vec<Vec<String>>`）
- `load_golden_dataset(path: &str) -> Result<GoldenDataset, String>`
- `compare_golden(actual: &GoldenDataset, expected: &GoldenDataset) -> GoldenCompareResult`
- `GoldenCompareResult` 構造体（`matches: bool`, `diff_rows: Vec<usize>`）
- `format_golden_diff(result: &GoldenCompareResult) -> String`

**完了条件**: Rust テスト 2 件（3811 + 2 = 3813）
- `golden_dataset_compare_pass`
- `golden_dataset_compare_fail_shows_diff`

---

### v80.3.0 — `TestFixture` / `DataFactory` モックデータ生成

テスト用モックデータを型安全に生成する。

**実装内容:**
- `FieldSpec` enum（`Str(String)`, `Int(i64)`, `Float(f64)`, `Bool(bool)`, `Null`）
- `RowSpec` 型（`Vec<(String, FieldSpec)>`）
- `TestFixture` 構造体（`name: String`, `schema: Vec<String>`, `rows: Vec<RowSpec>`）
- `DataFactory` 構造体（`seed: u64`）
- `DataFactory::generate_rows(spec: &TestFixture, count: usize) -> Vec<Vec<String>>`
- `DataFactory::from_seed(seed: u64) -> DataFactory`

**完了条件**: Rust テスト 2 件（3813 + 2 = 3815）
- `data_factory_generates_rows`
- `test_fixture_schema_matches_rows`

---

### v80.4.0 — プロパティベーステスト（`PipelineInvariant` 連携）

Favnir 3.0 の `PipelineInvariant` / `generate_counter_example_values` を
テストフレームワークに統合する。

**実装内容:**
- `PropertyTest` 構造体（`name: String`, `invariant: AggregateInvariant`, `samples: usize`）
- `run_property_test(test: &PropertyTest, data: &[f64]) -> PropertyTestResult`
- `PropertyTestResult` 構造体（`passed: bool`, `counter_example: Option<Vec<f64>>`）
- `format_property_test_result(result: &PropertyTestResult) -> String`
- `PropertyTestSuite` 構造体（`tests: Vec<PropertyTest>`）

**完了条件**: Rust テスト 2 件（3815 + 2 = 3817）
- `property_test_pass_when_invariant_holds`
- `property_test_fail_shows_counter_example`

---

### v80.5.0 — ステージ単体テスト（`StageTestCase`）

パイプライン全体ではなく、個別ステージを単体テストする型。

**実装内容:**
- `StageInput` 構造体（`name: String`, `rows: Vec<Vec<String>>`）
- `StageOutput` 構造体（`name: String`, `rows: Vec<Vec<String>>`）
- `StageTestCase` 構造体（`stage_name: String`, `input: StageInput`, `expected: StageOutput`）
- `run_stage_test(test: &StageTestCase, actual: &StageOutput) -> TestCase`
- `format_stage_test_result(result: &TestCase) -> String`

**完了条件**: Rust テスト 2 件（3817 + 2 = 3819）
- `stage_test_pass_when_output_matches`
- `stage_test_fail_when_output_differs`

---

### v80.6.0 — テストカバレッジレポート（`fav test --coverage`）

どのステージ・どの型がテストされているかをレポートする。

**実装内容:**
- `CoverageEntry` 構造体（`name: String`, `tested: bool`）
- `TestCoverageReport` 構造体（`entries: Vec<CoverageEntry>`, `total: usize`, `covered: usize`）
- `compute_test_coverage(suite: &TestSuite, known_stages: &[String]) -> TestCoverageReport`
- `format_coverage_report(report: &TestCoverageReport) -> String`
- `coverage_pct(report: &TestCoverageReport) -> f64`

**完了条件**: Rust テスト 2 件（3819 + 2 = 3821）
- `coverage_report_counts_correctly`
- `coverage_pct_is_zero_when_nothing_tested`

---

### v80.7.0 — スキーマスナップショットテスト（`SchemaSnapshot`）

スキーマの変更を検出する型。「前回と同じスキーマか」を型レベルで保証。

**実装内容:**
- `ColumnSnapshot` 構造体（`name: String`, `type_name: String`, `nullable: bool`）
- `SchemaSnapshot` 構造体（`pipeline_name: String`, `columns: Vec<ColumnSnapshot>`）
- `compare_schema_snapshots(current: &SchemaSnapshot, baseline: &SchemaSnapshot) -> SchemaSnapshotDiff`
- `SchemaSnapshotDiff` 構造体（`added: Vec<String>`, `removed: Vec<String>`, `changed: Vec<String>`）
- `format_schema_diff(diff: &SchemaSnapshotDiff) -> String`
- `schema_diff_is_breaking(diff: &SchemaSnapshotDiff) -> bool`

**完了条件**: Rust テスト 2 件（3821 + 2 = 3823）
- `schema_snapshot_no_diff_when_equal`
- `schema_snapshot_detects_removed_column`

---

### v80.8.0 — CI 統合レポート（JUnit XML / `TestReport`）

CI パイプラインに組み込める形式でテスト結果を出力する。

**実装内容:**
- `TestReport` 構造体（`suite: TestSuite`, `duration_ms: u64`, `timestamp: String`）
- `format_junit_xml(report: &TestReport) -> String`
- `format_test_summary(report: &TestReport) -> String`
- `cmd_test` に `--format junit` / `--format summary` オプション追加

**完了条件**: Rust テスト 2 件（3823 + 2 = 3825）
- `junit_xml_output_has_testsuite_tag`
- `test_report_summary_shows_pass_count`

---

### v80.9.0 — 安定化・コードフリーズ

v80.1〜v80.8 の全スプリント統合確認。バグ修正のみ。

**完了条件**: Rust テスト 2 件（3825 + 2 = 3827）
- `test_framework_full_sprint_all_stable`
- `test_framework_e2e_pipeline_tested`

---

### v81.0.0 — Test-Driven Data 1.0 宣言 ★クリーンアップ

**宣言文**:
> 「テストが型になり、カバレッジが数値になり、スキーマ変更が検出される。
>  Favnir のパイプラインは今、その正しさを `fav test` で証明できる。」

**クリーンアップ作業:**
- `cargo clean` 実施
- `Cargo.toml` バージョンを `81.0.0` に更新
- `CHANGELOG.md` / `MILESTONE.md` / `README.md` 更新
- `versions/current.md` 更新
- 本ファイル（`roadmap-v80.1-v85.0.md`）の Sprint 1 バージョン一覧テーブルを全行「完了」に更新

**完了条件**: `v81000_tests` 4 件（3827 + 4 = 3831）
- `cargo_toml_version_is_81_0_0`
- `changelog_has_v81_0_0`
- `milestone_has_test_driven_data`
- `readme_mentions_fav_test`

---

## ━━━━━━━━━━━━━━━━━━━━━━━━━━
## Sprint 2: Data Quality 2.0（v81.1〜v82.0）
## ━━━━━━━━━━━━━━━━━━━━━━━━━━

**テーマ**: データ品質ルールを型で表現し、`fav quality` で測定・レポートする。

### バージョン一覧

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

### v82.0.0 — Data Quality 2.0 宣言

**クリーンアップ作業:**
- `cargo clean` 実施
- `Cargo.toml` バージョンを `82.0.0` に更新
- `CHANGELOG.md` / `MILESTONE.md` / `README.md` 更新
- `versions/current.md` 更新
- 本ファイル（`roadmap-v80.1-v85.0.md`）の Sprint 2 バージョン一覧テーブルを全行「完了」に更新

**宣言文**:
> 「品質が型になった。外れ値はコンパイル時に検出され、
>  スキーマドリフトはパイプライン起動前に止まる。
>  Favnir のデータは今、品質を型で保証する。」

**完了条件**: `v82000_tests` 4 件（3849 + 4 = 3853）
- `cargo_toml_version_is_82_0_0`
- `changelog_has_v82_0_0`
- `milestone_has_data_quality_2`
- `readme_mentions_quality_gate`

---

## ━━━━━━━━━━━━━━━━━━━━━━━━━━
## Sprint 3: Pipeline Contracts 1.0（v82.1〜v83.0）
## ━━━━━━━━━━━━━━━━━━━━━━━━━━

**テーマ**: パイプライン間の入出力契約を型で定義し、`fav verify --contract` で検証する。

### バージョン一覧

| バージョン | 内容 | テスト数 | 状態 |
|---|---|---|---|
| v82.1.0 | `IoContract` / `ContractField` 型基盤 | 3853 + 2 = 3855 | 未着手 |
| v82.2.0 | `SlaContract`（SLA 遵守型） | 3855 + 2 = 3857 | 未着手 |
| v82.3.0 | パイプライン間契約依存（`ContractDependency`） | 3857 + 2 = 3859 | 未着手 |
| v82.4.0 | 契約違反の詳細レポート（`ContractViolation`） | 3859 + 2 = 3861 | 未着手 |
| v82.5.0 | スキーマから契約自動生成（`infer_contract`） | 3861 + 2 = 3863 | 未着手 |
| v82.6.0 | 契約バージョニング（`ContractVersion` / 後方互換チェック） | 3863 + 2 = 3865 | 未着手 |
| v82.7.0 | `fav verify --contract` コマンド強化 | 3865 + 2 = 3867 | 未着手 |
| v82.8.0 | 契約レジストリ（`ContractRegistry` / ローカルキャッシュ） | 3867 + 2 = 3869 | 未着手 |
| v82.9.0 | 安定化・コードフリーズ | 3869 + 2 = 3871 | 未着手 |
| v83.0.0 | Pipeline Contracts 1.0 宣言 ★クリーンアップ | 3871 + 4 = 3875 | 未着手 |

---

### v83.0.0 — Pipeline Contracts 1.0 宣言

**クリーンアップ作業:**
- `cargo clean` 実施
- `Cargo.toml` バージョンを `83.0.0` に更新
- `CHANGELOG.md` / `MILESTONE.md` / `README.md` 更新
- `versions/current.md` 更新
- 本ファイル（`roadmap-v80.1-v85.0.md`）の Sprint 3 バージョン一覧テーブルを全行「完了」に更新

**宣言文**:
> 「パイプライン間の約束が型になった。
>  `IoContract` がインターフェースを定義し、`SlaContract` が応答時間を保証し、
>  `ContractRegistry` がチームを繋ぐ。
>  Favnir のパイプラインは今、契約で安全に接続できる。」

**完了条件**: `v83000_tests` 4 件（3871 + 4 = 3875）
- `cargo_toml_version_is_83_0_0`
- `changelog_has_v83_0_0`
- `milestone_has_pipeline_contracts`
- `readme_mentions_contract_registry`

---

## ━━━━━━━━━━━━━━━━━━━━━━━━━━
## Sprint 4: Observability 2.0（v83.1〜v84.0）
## ━━━━━━━━━━━━━━━━━━━━━━━━━━

**テーマ**: パイプラインの健全性を型で表現し、壊れる前に検知する。

### バージョン一覧

| バージョン | 内容 | テスト数 | 状態 |
|---|---|---|---|
| v83.1.0 | `PipelineMetrics` 型（実行統計・レイテンシ） | 3875 + 2 = 3877 | 未着手 |
| v83.2.0 | `AlertRule` / `AlertThreshold`（アラート型） | 3877 + 2 = 3879 | 未着手 |
| v83.3.0 | `SloTarget` / `SloStatus`（SLO 型） | 3879 + 2 = 3881 | 未着手 |
| v83.4.0 | コスト追跡（`ExecutionCost` / `CostBudget`） | 3881 + 2 = 3883 | 未着手 |
| v83.5.0 | 分散トレーシング強化（OpenTelemetry `TraceContext`） | 3883 + 2 = 3885 | 未着手 |
| v83.6.0 | パフォーマンス回帰検知（`PerfBaseline` / `PerfRegression`） | 3885 + 2 = 3887 | 未着手 |
| v83.7.0 | `fav observe` コマンド（メトリクス・アラート統合） | 3887 + 2 = 3889 | 未着手 |
| v83.8.0 | 健全性ダッシュボード（`HealthDashboard` / テキスト形式） | 3889 + 2 = 3891 | 未着手 |
| v83.9.0 | 安定化・コードフリーズ | 3891 + 2 = 3893 | 未着手 |
| v84.0.0 | Observability 2.0 宣言 ★クリーンアップ | 3893 + 4 = 3897 | 未着手 |

---

### v84.0.0 — Observability 2.0 宣言

**クリーンアップ作業:**
- `cargo clean` 実施
- `Cargo.toml` バージョンを `84.0.0` に更新
- `CHANGELOG.md` / `MILESTONE.md` / `README.md` 更新
- `versions/current.md` 更新
- 本ファイル（`roadmap-v80.1-v85.0.md`）の Sprint 4 バージョン一覧テーブルを全行「完了」に更新

**宣言文**:
> 「メトリクスが型になり、アラートが型になり、SLO が型になった。
>  Favnir のパイプラインは壊れる前に教えてくれる。」

**完了条件**: `v84000_tests` 4 件（3893 + 4 = 3897）
- `cargo_toml_version_is_84_0_0`
- `changelog_has_v84_0_0`
- `milestone_has_observability_2`
- `readme_mentions_fav_observe`

---

## ━━━━━━━━━━━━━━━━━━━━━━━━━━
## Sprint 5: Favnir 4.0 宣言（v84.1〜v85.0）
## ━━━━━━━━━━━━━━━━━━━━━━━━━━

**テーマ**: Quality-First Era の集大成。v80.1〜v84.9 の全スプリントを統合確認して宣言。

### バージョン一覧

| バージョン | 内容 | テスト数 | 状態 |
|---|---|---|---|
| v84.1.0 | E2E ショーケース基盤（`infra/e2e-demo/favnir4-showcase/`） | 3897 + 2 = 3899 | 未着手 |
| v84.2.0 | テスト統合ショーケース（`fav test` E2E） | 3899 + 2 = 3901 | 未着手 |
| v84.3.0 | 品質統合ショーケース（`fav quality` E2E） | 3901 + 2 = 3903 | 未着手 |
| v84.4.0 | 契約統合ショーケース（`fav verify --contract` E2E） | 3903 + 2 = 3905 | 未着手 |
| v84.5.0 | 可観測性統合ショーケース（`fav observe` E2E） | 3905 + 2 = 3907 | 未着手 |
| v84.6.0 | ドキュメント完全化（`site/content/docs/v4/`） | 3907 + 2 = 3909 | 未着手 |
| v84.7.0 | OSS 公開強化・コミュニティ整備 v2 | 3909 + 2 = 3911 | 未着手 |
| v84.8.0 | パフォーマンス最終調整 | 3911 + 2 = 3913 | 未着手 |
| v84.9.0 | 安定化・コードフリーズ | 3913 + 2 = 3915 | 未着手 |
| v85.0.0 | Favnir 4.0 宣言 ★クリーンアップ | 3915 + 4 = 3919 | 未着手 |

---

### v85.0.0 — Favnir 4.0 宣言

**クリーンアップ作業:**
- `cargo clean` 実施
- `Cargo.toml` バージョンを `85.0.0` に更新
- `CHANGELOG.md` / `MILESTONE.md` / `README.md` 更新
- `versions/current.md` 更新
- 本ファイル（`roadmap-v80.1-v85.0.md`）の Sprint 5 バージョン一覧テーブルを全行「完了」に更新

**宣言文**:
> 「テストが型となり、品質が型となり、契約が型となり、観測が型となった。
>
>  `fav test` がパイプラインの正しさを証明し、
>  `QualityGate` が品質基準を守り、
>  `IoContract` がチームを安全に繋ぎ、
>  `AlertRule` が壊れる前に教えてくれる。
>
>  Favnir 4.0 は、データパイプラインの品質を
>  コードと同じ言語で語れる、唯一の言語である。」

**完了条件**: `v85000_tests` 4 件（3915 + 4 = 3919）
- `cargo_toml_version_is_85_0_0`
- `changelog_has_v85_0_0`
- `milestone_has_favnir_4`
- `readme_mentions_favnir_4`

---

## 全スプリント総括

| スプリント | マイルストーン | バージョン範囲 | テスト増加 |
|---|---|---|---|
| Sprint 1 | Test-Driven Data 1.0 | v80.1〜v81.0 | +22 |
| Sprint 2 | Data Quality 2.0 | v81.1〜v82.0 | +22 |
| Sprint 3 | Pipeline Contracts 1.0 | v82.1〜v83.0 | +22 |
| Sprint 4 | Observability 2.0 | v83.1〜v84.0 | +22 |
| Sprint 5 | Favnir 4.0 宣言 | v84.1〜v85.0 | +22 |
| **合計** | | v80.1〜v85.0 | **+110** |

**最終テスト数: 3,809 + 110 = 3,919**

---

## 各スプリント詳細ロードマップ（別ファイル）

Sprint 1（v80.1〜v81.0）の全バージョン詳述は本ファイルに含まれる。
Sprint 2〜4 の先頭バージョン詳述は、各スプリント開始時に以下のファイルとして作成する:

| スプリント | 詳細ファイル | 作成タイミング |
|---|---|---|
| Sprint 2: Data Quality 2.0 | `roadmap-v81.1-v82.0.md` | v81.0.0 宣言完了後 |
| Sprint 3: Pipeline Contracts 1.0 | `roadmap-v82.1-v83.0.md` | v82.0.0 宣言完了後 |
| Sprint 4: Observability 2.0 | `roadmap-v83.1-v84.0.md` | v83.0.0 宣言完了後 |
| Sprint 5: Favnir 4.0 宣言 | `roadmap-v84.1-v85.0.md` | v84.0.0 宣言完了後 |
