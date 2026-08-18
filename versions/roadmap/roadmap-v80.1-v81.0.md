# Roadmap v80.1.0 〜 v81.0.0 — Test-Driven Data 1.0

Date: 2026-08-16
Status: 未着手（v80.0.0 完了後に開始）

マスターロードマップ: [roadmap-v80.1-v85.0.md](roadmap-v80.1-v85.0.md)

---

## 前提

- 直前完了: v80.0.0「Favnir 3.0 宣言 ★クリーンアップ」（tests = 3,809）
- 本スプリントは Quality-First Era の第 1 スプリント
- 目標: v81.0.0「Test-Driven Data 1.0 宣言」（tests = 3,831）

### 着手前チェックリスト

- `versions/current.md` の現行マスターロードマップが `roadmap-v80.1-v85.0.md` を指していることを確認する
- `versions/v80-v85/v80.1.0/` ディレクトリを作成し、spec.md / plan.md / tasks.md を準備すること

### スプリントの性格

`fav test` コマンドとテストフレームワーク型基盤を新規構築する。
データパイプラインの正しさを「型付きテスト」で証明できる仕組みを作る。
A（新機能）70% + B（統合）30% の構成。

---

## バージョン一覧

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

## v80.1.0 — `TestCase` / `TestSuite` 型基盤 + `fav test` スタブ

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

## v80.2.0 — `GoldenDataset` / ゴールデンデータセット比較

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

## v80.3.0 — `TestFixture` / `DataFactory` モックデータ生成

テスト用モックデータを型安全に生成する。

**実装内容:**
- `FieldSpec` enum（`Str(String)`, `Int(i64)`, `Float(f64)`, `Bool(bool)`, `Null`）
- `RowSpec` 型（`Vec<(String, FieldSpec)>`）
- `TestFixture` 構造体（`name: String`, `schema: Vec<String>`, `rows: Vec<RowSpec>`）
- `DataFactory` 構造体（`seed: u64`）
- `DataFactory::generate_rows(&self, spec: &TestFixture, count: usize) -> Vec<Vec<String>>`
- `DataFactory::from_seed(seed: u64) -> DataFactory`

**完了条件**: Rust テスト 2 件（3814 + 2 = 3816）
- `data_factory_generates_rows`
- `test_fixture_schema_matches_rows`

---

## v80.4.0 — プロパティベーステスト（`PipelineInvariant` 連携）

Favnir 3.0 の `AggregateInvariant` / `generate_counter_example_values` を
テストフレームワークに統合する（概念名: `PipelineInvariant`）。

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

## v80.5.0 — ステージ単体テスト（`StageTestCase`）

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

## v80.6.0 — テストカバレッジレポート（`fav test --coverage`）

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

## v80.7.0 — スキーマスナップショットテスト（`SchemaSnapshot`）

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

## v80.8.0 — CI 統合レポート（JUnit XML / `TestReport`）

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

## v80.9.0 — 安定化・コードフリーズ

v80.1〜v80.8 の全スプリント統合確認。バグ修正のみ。

**実装内容:**
- v80.1〜v80.8 の全テスト通過確認（`cargo test` 全 pass）
- `fav test` コマンド E2E 動作確認
- バグ修正のみ受け入れ（新機能追加なし）

**完了条件**: Rust テスト 2 件（3825 + 2 = 3827）（統合確認テスト — 新規実装なし）
- `test_framework_full_sprint_all_stable`
- `test_framework_e2e_pipeline_tested`（既存 E2E パイプラインが `fav test` で通ることを確認）

---

## v81.0.0 — Test-Driven Data 1.0 宣言 ★クリーンアップ

**宣言文**:
> 「テストが型になり、カバレッジが数値になり、スキーマ変更が検出される。
>  Favnir のパイプラインは今、その正しさを `fav test` で証明できる。」

**クリーンアップ作業:**
- `cargo clean` 実施
- `Cargo.toml` バージョンを `81.0.0` に更新
- `CHANGELOG.md` / `MILESTONE.md` / `README.md` 更新
- `versions/current.md` 更新
- `roadmap-v80.1-v85.0.md` の Sprint 1 バージョン一覧テーブルを全行「完了」に更新

**完了条件**: `v81000_tests` 4 件（3827 + 4 = 3831）
- `cargo_toml_version_is_81_0_0`
- `changelog_has_v81_0_0`
- `milestone_has_test_driven_data`
- `readme_mentions_fav_test`

---

## テスト数推移（本スプリント）

| バージョン | テスト数 | 増加 |
|---|---|---|
| v80.0.0（ベース） | 3,809 | — |
| v80.1.0 | 3,811 | +2 |
| v80.2.0 | 3,813 | +2 |
| v80.3.0 | 3,815 | +2 |
| v80.4.0 | 3,817 | +2 |
| v80.5.0 | 3,819 | +2 |
| v80.6.0 | 3,821 | +2 |
| v80.7.0 | 3,823 | +2 |
| v80.8.0 | 3,825 | +2 |
| v80.9.0 | 3,827 | +2 |
| v81.0.0（宣言） | 3,831 | +4 |

**本スプリント合計**: +22 tests（3,809 → 3,831）

---

## 参考リンク

- マスターロードマップ: [roadmap-v80.1-v85.0.md](roadmap-v80.1-v85.0.md)
- 前スプリント: [roadmap-v79.1-v80.0.md](roadmap-v79.1-v80.0.md)
- 次スプリント: [roadmap-v81.1-v82.0.md](roadmap-v81.1-v82.0.md)
- 達成宣言: `MILESTONE.md`
- 進行状況: `versions/current.md`
