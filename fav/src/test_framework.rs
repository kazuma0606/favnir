/// Test framework types for typed pipeline testing.
/// Introduced in v80.1.0 as part of the Test-Driven Data 1.0 sprint.

#[derive(Debug)]
pub enum TestStatus {
    Pass,
    Fail,
    Skip,
}

#[derive(Debug)]
pub struct TestCase {
    pub name: String,
    pub status: TestStatus,
    pub message: Option<String>,
}

#[derive(Debug)]
pub struct TestSuite {
    pub name: String,
    pub cases: Vec<TestCase>,
}

/// Summary counts from running a test suite.
/// Suite name is not included here by design — callers combine name and result
/// as needed (e.g. `format!("{}: {}", suite.name, format_test_suite_result(&r))`).
/// Failed case names are not yet collected; that is planned for a future version.
#[derive(Debug)]
pub struct TestSuiteResult {
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
}

/// Run all cases in the suite and return a summary count.
pub fn run_test_suite(suite: &TestSuite) -> TestSuiteResult {
    let mut passed = 0;
    let mut failed = 0;
    let mut skipped = 0;
    for case in &suite.cases {
        match case.status {
            TestStatus::Pass => passed += 1,
            TestStatus::Fail => failed += 1,
            TestStatus::Skip => skipped += 1,
        }
    }
    TestSuiteResult { passed, failed, skipped }
}

/// Format as "N passed, M failed, K skipped".
pub fn format_test_suite_result(result: &TestSuiteResult) -> String {
    format!("{} passed, {} failed, {} skipped",
        result.passed, result.failed, result.skipped)
}

// ─── GoldenDataset ───────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct GoldenDataset {
    pub name: String,
    pub rows: Vec<Vec<String>>,
}

#[derive(Debug)]
pub struct GoldenCompareResult {
    pub matches: bool,
    pub diff_rows: Vec<usize>,
}

/// Compare actual output against expected row by row.
/// Rows beyond the shorter dataset are also counted as diffs.
pub fn compare_golden(actual: &GoldenDataset, expected: &GoldenDataset) -> GoldenCompareResult {
    let mut diff_rows = Vec::new();
    let max_len = actual.rows.len().max(expected.rows.len());
    for i in 0..max_len {
        if actual.rows.get(i) != expected.rows.get(i) {
            diff_rows.push(i);
        }
    }
    let matches = diff_rows.is_empty();
    GoldenCompareResult { matches, diff_rows }
}

/// Format diff result as "OK: datasets match" or "DIFF: N row(s) differ: [0, 2, ...]".
pub fn format_golden_diff(result: &GoldenCompareResult) -> String {
    if result.matches {
        "OK: datasets match".to_string()
    } else {
        let indices: Vec<String> = result.diff_rows.iter().map(|i| i.to_string()).collect();
        format!("DIFF: {} row(s) differ: [{}]", result.diff_rows.len(), indices.join(", "))
    }
}

// ─── TestFixture / DataFactory ───────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum FieldSpec {
    Str(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Null,
}

/// 1 行分のフィールド仕様: (列名, FieldSpec) のペア列。
pub type RowSpec = Vec<(String, FieldSpec)>;

#[derive(Debug)]
pub struct TestFixture {
    pub name: String,
    pub schema: Vec<String>,
    pub rows: Vec<RowSpec>,
}

#[derive(Debug)]
pub struct DataFactory {
    pub seed: u64,
}

impl DataFactory {
    pub fn from_seed(seed: u64) -> DataFactory {
        DataFactory { seed }
    }

    pub fn generate_rows(&self, spec: &TestFixture, count: usize) -> Vec<Vec<String>> {
        if spec.rows.is_empty() {
            return Vec::new();
        }
        let n = spec.rows.len();
        // stride = seed.max(1) にすることで seed=0 を stride=1 に正規化する。
        // インデックス式: (i * stride + i) % n = i * (stride + 1) % n
        // seed=1/n=2 等 gcd(stride+1, n)=n となるケースでは全行が同一テンプレートになる。
        // これは仕様通りの循環パターンであり、シード値の多様性ではなくテンプレート循環を目的とする。
        let stride = self.seed.max(1) as usize;
        (0..count)
            .map(|i| {
                let template = &spec.rows[(i * stride + i) % n];
                let field_map: std::collections::HashMap<&str, &FieldSpec> = template
                    .iter()
                    .map(|(k, v)| (k.as_str(), v))
                    .collect();
                spec.schema
                    .iter()
                    .map(|col| match field_map.get(col.as_str()) {
                        Some(FieldSpec::Str(s))   => s.clone(),
                        Some(FieldSpec::Int(n))   => n.to_string(),
                        Some(FieldSpec::Float(f)) => f.to_string(),
                        Some(FieldSpec::Bool(b))  => b.to_string(),
                        // FieldSpec::Null またはスキーマ列名がテンプレートに存在しない場合は空文字列を返す（設計上の意図）。
                        Some(FieldSpec::Null) | None => String::new(),
                    })
                    .collect()
            })
            .collect()
    }
}

/// Load a CSV file as a GoldenDataset (one row per line, comma-separated).
/// Empty lines are skipped. Not available on WASM targets.
///
/// **Limitation**: fields containing commas or quotes are not supported.
/// Use only simple string/number columns without embedded commas.
#[cfg(not(target_arch = "wasm32"))]
pub fn load_golden_dataset(path: &str) -> Result<GoldenDataset, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("failed to load golden dataset '{}': {}", path, e))?;
    let rows: Vec<Vec<String>> = content
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.split(',').map(|s| s.to_string()).collect())
        .collect();
    let name = std::path::Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(path)
        .to_string();
    Ok(GoldenDataset { name, rows })
}

// ─── PropertyTest / PropertyTestResult / PropertyTestSuite ───────────────────

/// データ列に対して検証する不変条件の種類。
#[derive(Debug, Clone)]
pub enum InvariantKind {
    /// 全値 >= 0.0 であること。
    NonNegative,
    /// 全値 <= 0.0 であること。
    NonPositive,
    /// NaN / Inf を含まない有限値であること（f64::is_finite()）。
    Finite,
}

/// プロパティベーステストの定義。
#[derive(Debug)]
pub struct PropertyTest {
    pub name: String,
    pub kind: InvariantKind,
    /// 呼び出し元が管理するサンプル数のメタデータ。
    /// `run_property_test` は `data` を直接受け取るため、このフィールドは
    /// 実行ロジックには影響しない（外部ジェネレータへのヒントとして使用する）。
    pub samples: usize,
}

/// プロパティベーステストの実行結果。
#[derive(Debug)]
pub struct PropertyTestResult {
    pub passed: bool,
    /// 不変条件に違反した最初の値（1 要素のみ）。違反がなければ None。
    pub counter_example: Option<Vec<f64>>,
}

/// `PropertyTest` を `data` に対して実行する。
///
/// data 内の各値が `kind` の不変条件を満たすか検証する。
/// 違反する最初の値を `counter_example` に記録する。
pub fn run_property_test(test: &PropertyTest, data: &[f64]) -> PropertyTestResult {
    let violation = match test.kind {
        InvariantKind::NonNegative => data.iter().find(|&&v| v < 0.0).copied(),
        InvariantKind::NonPositive => data.iter().find(|&&v| v > 0.0).copied(),
        InvariantKind::Finite => data.iter().find(|&&v| !v.is_finite()).copied(),
    };
    match violation {
        None => PropertyTestResult { passed: true, counter_example: None },
        Some(v) => PropertyTestResult { passed: false, counter_example: Some(vec![v]) },
    }
}

/// プロパティテスト結果を人間が読める文字列に変換する。
pub fn format_property_test_result(result: &PropertyTestResult) -> String {
    if result.passed {
        "PASS: invariant holds".to_string()
    } else {
        let ce = result.counter_example.as_deref().unwrap_or(&[]);
        let vals: Vec<String> = ce.iter().map(|v| v.to_string()).collect();
        format!("FAIL: counter_example=[{}]", vals.join(", "))
    }
}

/// 複数の `PropertyTest` をまとめるスイート。
#[derive(Debug)]
pub struct PropertyTestSuite {
    pub tests: Vec<PropertyTest>,
}

/// スイート内の全テストを `data` に対して実行し、結果一覧を返す。
pub fn run_property_test_suite(suite: &PropertyTestSuite, data: &[f64]) -> Vec<PropertyTestResult> {
    suite.tests.iter().map(|t| run_property_test(t, data)).collect()
}

// ─── StageTestCase ────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct StageInput {
    pub name: String,
    pub rows: Vec<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct StageOutput {
    pub name: String,
    pub rows: Vec<Vec<String>>,
}

#[derive(Debug)]
pub struct StageTestCase {
    pub stage_name: String,
    /// 呼び出し元が実際の stage 関数を呼び出す際に渡すフィールド。
    /// `run_stage_test` 自身は `input` を参照しない（expected と actual の比較のみを行う）。
    pub input: StageInput,
    pub expected: StageOutput,
}

/// `test.expected` と `actual` の rows を行単位で比較する。
/// - 全行一致: `TestCase { status: Pass, message: None }`
/// - 不一致: `TestCase { status: Fail, message: Some("row N differs: ...") }`
/// - 行数不一致は超過行も "row N differs" として記録する。
pub fn run_stage_test(test: &StageTestCase, actual: &StageOutput) -> TestCase {
    let expected = &test.expected.rows;
    let actual_rows = &actual.rows;
    let max_len = expected.len().max(actual_rows.len());
    for i in 0..max_len {
        let exp_row = expected.get(i);
        let act_row = actual_rows.get(i);
        if exp_row != act_row {
            let msg = format!(
                "row {} differs: expected {:?}, got {:?}",
                i,
                exp_row.map(|r| r.as_slice()).unwrap_or(&[]),
                act_row.map(|r| r.as_slice()).unwrap_or(&[]),
            );
            return TestCase {
                name: test.stage_name.clone(),
                status: TestStatus::Fail,
                message: Some(msg),
            };
        }
    }
    TestCase {
        name: test.stage_name.clone(),
        status: TestStatus::Pass,
        message: None,
    }
}

/// `TestCase` を人間が読める文字列に変換する。
/// - Pass: "PASS: <name>"
/// - Fail: "FAIL: <name> — <message>"
/// - Skip: "SKIP: <name>"
pub fn format_stage_test_result(result: &TestCase) -> String {
    match result.status {
        TestStatus::Pass => format!("PASS: {}", result.name),
        TestStatus::Fail => {
            let msg = result.message.as_deref().unwrap_or("");
            format!("FAIL: {} \u{2014} {}", result.name, msg)
        }
        TestStatus::Skip => format!("SKIP: {}", result.name),
    }
}

// ─── TestCoverageReport ───────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct CoverageEntry {
    pub name: String,
    pub tested: bool,
}

#[derive(Debug)]
pub struct TestCoverageReport {
    pub entries: Vec<CoverageEntry>,
    pub total: usize,
    pub covered: usize,
}

/// `suite` のケース名と `known_stages` を突き合わせてカバレッジレポートを生成する。
///
/// - ステージ名が `suite.cases` に 1 件以上あれば `tested: true`（status は問わない）。
/// - `total = known_stages.len()`、`covered = tested が true のエントリ数`。
pub fn compute_test_coverage(suite: &TestSuite, known_stages: &[String]) -> TestCoverageReport {
    let tested_names: std::collections::HashSet<&str> =
        suite.cases.iter().map(|c| c.name.as_str()).collect();
    let entries: Vec<CoverageEntry> = known_stages
        .iter()
        .map(|s| CoverageEntry {
            name: s.clone(),
            tested: tested_names.contains(s.as_str()),
        })
        .collect();
    let covered = entries.iter().filter(|e| e.tested).count();
    let total = entries.len();
    TestCoverageReport { entries, total, covered }
}

/// カバレッジレポートを "coverage: X/Y (Z.Zpct)" 形式の文字列に変換する。
pub fn format_coverage_report(report: &TestCoverageReport) -> String {
    format!(
        "coverage: {}/{} ({:.1}pct)",
        report.covered,
        report.total,
        coverage_pct(report)
    )
}

/// カバレッジ率を 0.0〜100.0 の f64 で返す。total が 0 の場合は 0.0（ゼロ除算ガード）。
pub fn coverage_pct(report: &TestCoverageReport) -> f64 {
    if report.total == 0 {
        0.0
    } else {
        report.covered as f64 / report.total as f64 * 100.0
    }
}

// ─── SchemaSnapshot ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub struct ColumnSnapshot {
    pub name: String,
    pub type_name: String,
    pub nullable: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SchemaSnapshot {
    /// パイプラインの識別子。compare_schema_snapshots では比較対象外（メタデータ）。
    pub pipeline_name: String,
    pub columns: Vec<ColumnSnapshot>,
}

#[derive(Debug)]
pub struct SchemaSnapshotDiff {
    /// current にあって baseline にない列名。
    pub added: Vec<String>,
    /// baseline にあって current にない列名。
    pub removed: Vec<String>,
    /// 両方に存在するが type_name または nullable が異なる列名。
    pub changed: Vec<String>,
}

/// current と baseline を比較してスキーマ差分を返す。
/// 列の突き合わせは名前（`name` フィールド）で行い、列順は問わない。
pub fn compare_schema_snapshots(
    current: &SchemaSnapshot,
    baseline: &SchemaSnapshot,
) -> SchemaSnapshotDiff {
    use std::collections::HashMap;

    let current_map: HashMap<&str, &ColumnSnapshot> =
        current.columns.iter().map(|c| (c.name.as_str(), c)).collect();
    let baseline_map: HashMap<&str, &ColumnSnapshot> =
        baseline.columns.iter().map(|c| (c.name.as_str(), c)).collect();

    let mut added = Vec::new();
    let mut removed = Vec::new();
    let mut changed = Vec::new();

    for (name, base_col) in &baseline_map {
        match current_map.get(name) {
            None => removed.push((*name).to_string()),
            Some(cur_col) => {
                if cur_col.type_name != base_col.type_name || cur_col.nullable != base_col.nullable {
                    changed.push((*name).to_string());
                }
            }
        }
    }

    for name in current_map.keys() {
        if !baseline_map.contains_key(name) {
            added.push((*name).to_string());
        }
    }

    added.sort();
    removed.sort();
    changed.sort();

    SchemaSnapshotDiff { added, removed, changed }
}

/// diff を "OK: schema unchanged" または "added=[...], removed=[...], changed=[...]" に変換する。
pub fn format_schema_diff(diff: &SchemaSnapshotDiff) -> String {
    if diff.added.is_empty() && diff.removed.is_empty() && diff.changed.is_empty() {
        return "OK: schema unchanged".to_string();
    }
    format!(
        "added=[{}], removed=[{}], changed=[{}]",
        diff.added.join(", "),
        diff.removed.join(", "),
        diff.changed.join(", "),
    )
}

/// removed または changed が 1 件以上あれば破壊的変更（true）。added のみなら後方互換（false）。
pub fn schema_diff_is_breaking(diff: &SchemaSnapshotDiff) -> bool {
    !diff.removed.is_empty() || !diff.changed.is_empty()
}

// ─── TestReport ───────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct TestReport {
    pub suite: TestSuite,
    /// テスト実行にかかった時間（ミリ秒）。
    pub duration_ms: u64,
    /// ISO 8601 形式のタイムスタンプ文字列（例: "2026-08-19T00:00:00Z"）。
    pub timestamp: String,
}

/// JUnit XML 形式のテストレポートを生成する。
///
/// - `time` は `duration_ms / 1000.0`（小数点以下 3 桁）。
/// - Pass / Skip ケースは `<testcase ... />` のみ。Fail は `<failure>` 子要素を持つ。
/// - XML エスケープは本バージョンではスコープ外。
///   **注意**: `name` / `message` フィールドに `<`, `>`, `&`, `"` を含む場合、
///   不正な XML が生成される。これらの文字を含まないテストデータのみを渡すこと。
/// - 出力末尾に改行は付かない。
pub fn format_junit_xml(report: &TestReport) -> String {
    let result = run_test_suite(&report.suite);
    let total = result.passed + result.failed + result.skipped;
    let duration_s = report.duration_ms as f64 / 1000.0;

    let mut cases = String::new();
    for case in &report.suite.cases {
        match case.status {
            TestStatus::Fail => {
                let msg = case.message.as_deref().unwrap_or("");
                cases.push_str(&format!(
                    "  <testcase name=\"{}\" classname=\"{}\">\n    <failure message=\"{}\"/>\n  </testcase>\n",
                    case.name, report.suite.name, msg
                ));
            }
            _ => {
                cases.push_str(&format!(
                    "  <testcase name=\"{}\" classname=\"{}\"/>\n",
                    case.name, report.suite.name
                ));
            }
        }
    }

    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<testsuite name=\"{}\" tests=\"{}\" failures=\"{}\" skipped=\"{}\" time=\"{:.3}\">\n{}",
        report.suite.name, total, result.failed, result.skipped, duration_s, cases
    ) + "</testsuite>"
}

/// 人間向けサマリー形式のレポートを生成する。
///
/// 出力形式: "{suite.name}: N passed, M failed, K skipped ({duration_ms}ms) [{timestamp}]"
pub fn format_test_summary(report: &TestReport) -> String {
    let result = run_test_suite(&report.suite);
    format!(
        "{}: {} passed, {} failed, {} skipped ({}ms) [{}]",
        report.suite.name,
        result.passed,
        result.failed,
        result.skipped,
        report.duration_ms,
        report.timestamp,
    )
}

// ── v81.1.0: QualityRule / QualityCheck 型基盤 ──────────────────────────────

/// 品質ルールの種類。
#[derive(Debug, Clone, PartialEq)]
pub enum QualityRuleKind {
    /// 値が空文字列（または空白のみ）でないことを確認する。
    NotNull,
    /// カラム内の値が一意であることを確認する。
    /// **注意**: `run_quality_check` は行単位チェックのため、`Unique` を指定しても常に違反 0 件となる。
    Unique,
    /// 数値が `[min, max]` 範囲内であることを確認する（`f64` にパース可能な文字列が対象）。
    /// `f64` にパースできない値はスキップする（違反として報告しない）。
    Range { min: f64, max: f64 },
    /// 値に指定パターンが部分文字列として含まれることを確認する（`value.contains(pattern)` で実装）。
    /// **注意**: 現バージョンでは正規表現ではなく部分文字列マッチ。将来バージョンで正規表現エンジンに昇格予定。
    Regex(String),
    /// カスタムルール名（現バージョンでは常に違反なしとして扱う）。
    Custom(String),
}

/// ルール違反時の重大度。
#[derive(Debug, Clone, PartialEq)]
pub enum RuleSeverity {
    Error,
    Warning,
}

/// 単一カラムへの品質ルール定義。
#[derive(Debug, Clone, PartialEq)]
pub struct QualityRule {
    /// カラムのインデックス文字列（"0", "1", ...）。カラム名ではない。
    pub column: String,
    pub kind: QualityRuleKind,
    pub severity: RuleSeverity,
}

/// 複数の品質ルールをまとめるチェックセット。
#[derive(Debug, Clone)]
pub struct QualityCheck {
    pub rules: Vec<QualityRule>,
}

/// 品質ルール違反の詳細。
#[derive(Debug, Clone)]
pub struct QualityViolation {
    pub rule: QualityRule,
    /// 違反が検出された行のインデックス（0 始まり）。
    pub row_index: usize,
    /// 違反した実際の値（文字列表現）。
    pub actual: String,
}

/// `QualityCheck` のルールを全行に対して適用し、違反一覧を返す。
///
/// `column` フィールドはカラムのインデックス文字列（"0", "1", ...）として `parse::<usize>()` で解釈する。
/// インデックスが行の範囲外の場合はその行をスキップする（違反として報告しない）。
///
/// `QualityRuleKind` 適用ルール:
/// - `NotNull`: `value.trim().is_empty()` なら違反
/// - `Range { min, max }`: `value.parse::<f64>()` して `v < min || v > max` なら違反（パース失敗はスキップ）
/// - `Regex(pattern)`: `!value.contains(pattern.as_str())` なら違反（部分文字列マッチ）
/// - `Unique` / `Custom`: 行単位チェック非対応のためスキップ（常に違反 0 件）
pub fn run_quality_check(check: &QualityCheck, rows: &[Vec<String>]) -> Vec<QualityViolation> {
    let mut violations = Vec::new();
    for rule in &check.rules {
        let col_idx: usize = match rule.column.parse() {
            Ok(i) => i,
            Err(_) => continue,
        };
        for (row_idx, row) in rows.iter().enumerate() {
            let value = match row.get(col_idx) {
                Some(v) => v,
                None => continue,
            };
            let violated = match &rule.kind {
                QualityRuleKind::NotNull => value.trim().is_empty(),
                QualityRuleKind::Range { min, max } => match value.parse::<f64>() {
                    Ok(v) => v < *min || v > *max,
                    Err(_) => false,
                },
                QualityRuleKind::Regex(pattern) => !value.contains(pattern.as_str()),
                QualityRuleKind::Unique | QualityRuleKind::Custom(_) => false,
            };
            if violated {
                violations.push(QualityViolation {
                    rule: rule.clone(),
                    row_index: row_idx,
                    actual: value.clone(),
                });
            }
        }
    }
    violations
}

// ── v81.2.0: DistributionStats / StatisticalCheck ────────────────────────────

/// 数値列の統計的分布情報。
#[derive(Debug, Clone)]
pub struct DistributionStats {
    pub mean: f64,
    pub std_dev: f64,
    pub min: f64,
    pub max: f64,
    pub count: usize,
}

/// 数値列の統計量を計算する（母標準偏差 n 割り）。
///
/// `values` が空の場合は全フィールド 0.0 / count=0 を返す。
pub fn compute_distribution_stats(values: &[f64]) -> DistributionStats {
    if values.is_empty() {
        return DistributionStats { mean: 0.0, std_dev: 0.0, min: 0.0, max: 0.0, count: 0 };
    }
    let count = values.len();
    let mean = values.iter().sum::<f64>() / count as f64;
    let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / count as f64;
    let std_dev = variance.sqrt();
    let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    DistributionStats { mean, std_dev, min, max, count }
}

/// Z スコアベースの外れ値検出設定。
///
/// `column` はカラムのインデックス文字列（意味的メタデータ。`detect_outliers` では参照しない）。
/// `z_score_threshold` はこの値より大きな Z スコアを外れ値とみなす。
/// **`z_score_threshold` は正の値を指定すること**。負の値を指定すると全要素が外れ値として返される。
#[derive(Debug, Clone)]
pub struct StatisticalCheck {
    pub column: String,
    pub z_score_threshold: f64,
}

/// Z スコアが `check.z_score_threshold` を超える値のインデックスを返す。
///
/// `std_dev == 0.0` のとき（全値が同一）は外れ値なし（空 Vec）を返す。
/// **`values` に `f64::NAN` が含まれる場合の動作は未定義**（NaN は外れ値として検出されない）。
pub fn detect_outliers(check: &StatisticalCheck, values: &[f64]) -> Vec<usize> {
    let stats = compute_distribution_stats(values);
    if stats.std_dev == 0.0 {
        return Vec::new();
    }
    values
        .iter()
        .enumerate()
        .filter_map(|(i, &v)| {
            let z = (v - stats.mean).abs() / stats.std_dev;
            if z > check.z_score_threshold { Some(i) } else { None }
        })
        .collect()
}

/// 分布情報を人間向けの文字列に変換する。
///
/// 出力形式: `"count={count} mean={mean:.3} std={std_dev:.3} min={min:.3} max={max:.3}"`
pub fn format_distribution_report(stats: &DistributionStats) -> String {
    format!(
        "count={} mean={:.3} std={:.3} min={:.3} max={:.3}",
        stats.count, stats.mean, stats.std_dev, stats.min, stats.max
    )
}

// ── v81.3.0: SchemaDriftDetector ──────────────────────────────────────────────

/// スキーマ変更の許容レベル。
///
/// - `Strict`: 追加・削除・変更をすべてドリフトとみなす
/// - `Additive`: 削除・変更のみドリフト（追加のみは `has_drift = false`）
/// - `Permissive`: 現バージョンでは `Additive` と同一動作
#[derive(Debug, Clone, PartialEq)]
pub enum DriftTolerance {
    Strict,
    Additive,
    Permissive,
}

/// ベースラインスキーマからのドリフトを検出する設定。
#[derive(Debug, Clone)]
pub struct SchemaDriftDetector {
    pub baseline: SchemaSnapshot,
    pub tolerance: DriftTolerance,
}

/// ドリフト検出結果。
///
/// `severity` は `has_drift = true` のときのみ意味を持つ。
/// **`has_drift = false` のとき `severity` は `RuleSeverity::Warning` だが、
/// この値を「軽微な問題あり」と解釈してはならない。必ず `has_drift` を先に確認すること。**
#[derive(Debug)]
pub struct DriftResult {
    pub has_drift: bool,
    pub severity: RuleSeverity,
    pub diff: SchemaSnapshotDiff,
}

/// ベースラインと current を比較してドリフト結果を返す。
///
/// - `DriftTolerance::Strict`: diff に追加・削除・変更が 1 件でもあれば `has_drift = true`
/// - `DriftTolerance::Additive` / `Permissive`: 削除・変更がある場合のみ `has_drift = true`
///   （追加のみの場合は `has_drift = false`）
///
/// `has_drift = true` のとき `severity = RuleSeverity::Error`。
/// `has_drift = false` のとき `severity = RuleSeverity::Warning`。
pub fn detect_schema_drift(detector: &SchemaDriftDetector, current: &SchemaSnapshot) -> DriftResult {
    let diff = compare_schema_snapshots(current, &detector.baseline);
    let any_change = !diff.added.is_empty() || !diff.removed.is_empty() || !diff.changed.is_empty();
    let breaking = !diff.removed.is_empty() || !diff.changed.is_empty();
    let has_drift = match detector.tolerance {
        DriftTolerance::Strict => any_change,
        DriftTolerance::Additive | DriftTolerance::Permissive => breaking,
    };
    let severity = if has_drift { RuleSeverity::Error } else { RuleSeverity::Warning };
    DriftResult { has_drift, severity, diff }
}

/// ドリフト結果を人間向けの文字列に変換する。
///
/// ドリフトなし: `"OK: no schema drift detected"`
/// ドリフトあり: `"DRIFT: added=[name, age] removed=[...] changed=[...]"`
pub fn format_drift_report(result: &DriftResult) -> String {
    if !result.has_drift {
        return "OK: no schema drift detected".to_string();
    }
    format!(
        "DRIFT: added=[{}] removed=[{}] changed=[{}]",
        result.diff.added.join(", "),
        result.diff.removed.join(", "),
        result.diff.changed.join(", "),
    )
}

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
///
/// # Note
/// `weight` は 0.0 以上の値を指定すること。負の `weight` を渡した場合の動作は未定義。
pub fn compute_quality_score(dimensions: &[DimensionScore]) -> QualityScore {
    debug_assert!(dimensions.iter().all(|d| d.weight >= 0.0), "weight must be >= 0.0");
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
///
/// # Note
/// `score.overall` のみを参照する。`dimensions` フィールドは使用しない。
pub fn quality_grade(score: &QualityScore) -> &'static str {
    match score.overall {
        x if x >= 0.90 => "A",
        x if x >= 0.80 => "B",
        x if x >= 0.70 => "C",
        x if x >= 0.60 => "D",
        _              => "F",
    }
}

// ── v81.5.0: ProvenanceQualityReport ──────────────────────────────────────────

/// ソース単位の来歴付き品質エントリ。
///
/// `provenance_hash` は本バージョンでは `String` スタブ（Favnir 3.0 連携は将来対応）。
/// `quality_score` は 0.0〜1.0 の範囲。NaN を渡した場合の動作は未定義。
#[derive(Debug, Clone)]
pub struct ProvenanceQualityEntry {
    pub source_name: String,
    pub provenance_hash: String,
    pub quality_score: f64,
}

/// パイプライン単位の来歴付き品質レポート。
#[derive(Debug)]
pub struct ProvenanceQualityReport {
    pub entries: Vec<ProvenanceQualityEntry>,
    pub pipeline_name: String,
}

/// `entries` と `pipeline` からレポートを構築する。
pub fn build_provenance_quality_report(
    entries: Vec<ProvenanceQualityEntry>,
    pipeline: &str,
) -> ProvenanceQualityReport {
    ProvenanceQualityReport { entries, pipeline_name: pipeline.to_string() }
}

/// レポートを人間向けの文字列に変換する。
///
/// 出力形式:
/// `"pipeline={name} sources={count}\n- {source}: score={:.3} hash={hash}\n..."`
pub fn format_provenance_quality_report(report: &ProvenanceQualityReport) -> String {
    let mut out = format!(
        "pipeline={} sources={}",
        report.pipeline_name,
        report.entries.len(),
    );
    for e in &report.entries {
        out.push_str(&format!(
            "\n- {}: score={:.3} hash={}",
            e.source_name, e.quality_score, e.provenance_hash,
        ));
    }
    out
}

/// `quality_score` が最も低いエントリを返す。空のとき `None`。
///
/// スコアが同値の場合は先に現れたエントリを返す。
/// `quality_score` に NaN を渡した場合の動作は未定義。
pub fn worst_quality_source(report: &ProvenanceQualityReport) -> Option<&ProvenanceQualityEntry> {
    report.entries.iter().reduce(|worst, e| {
        if e.quality_score < worst.quality_score { e } else { worst }
    })
}

// ── v81.6.0: QualityGate ──────────────────────────────────────────────────────

/// 品質ゲートの評価結果。
///
/// - `Pass`: すべての条件を満たした
/// - `Fail(String)`: 最初に失敗した条件の説明
/// - `Warn(String)`: 条件は満たしているが注意が必要（将来拡張用、現バージョンでは未使用）
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
        GateDecision::Pass      => "PASS".to_string(),
        GateDecision::Fail(msg) => format!("FAIL: {msg}"),
        GateDecision::Warn(msg) => format!("WARN: {msg}"),
    }
}

// ── v81.7.0: QualityReportOptions / build_quality_report ─────────────────────

/// レポート出力フォーマット。
#[derive(Debug, Clone, PartialEq)]
pub enum ReportFormat {
    Text,
    Json,
    Markdown,
}

/// `build_quality_report` のオプション。
///
/// `include_stats` は将来拡張用スタブ。現バージョンでは参照しない。
#[derive(Debug, Clone)]
pub struct QualityReportOptions {
    pub format: ReportFormat,
    pub include_violations: bool,
    pub include_stats: bool,
}

/// `check` のルールを `rows` に適用し、`opts` の形式でレポートを返す。
///
/// `include_stats` は将来拡張用スタブ。現バージョンでは参照しない。
pub fn build_quality_report(
    check: &QualityCheck,
    rows: &[Vec<String>],
    opts: &QualityReportOptions,
) -> String {
    let violations = run_quality_check(check, rows);
    // TODO: opts.include_stats は将来 DistributionStats 要約追記に使用予定（現在は未使用）
    let n = violations.len();
    match opts.format {
        ReportFormat::Text => {
            let mut out = format!("quality_report format=text violations={n}");
            if opts.include_violations {
                for v in &violations {
                    out.push_str(&format!(
                        "\n- row={} col={} rule={:?}",
                        v.row_index, v.rule.column, v.rule.kind,
                    ));
                }
            }
            out
        }
        ReportFormat::Json => {
            if opts.include_violations {
                let items: Vec<String> = violations.iter().map(|v| {
                    format!(
                        "{{\"row\":{},\"col\":\"{}\",\"rule\":\"{:?}\"}}",
                        v.row_index, v.rule.column, v.rule.kind,
                    )
                }).collect();
                format!(
                    "{{\"format\":\"json\",\"violations\":{n},\"items\":[{}]}}",
                    items.join(",")
                )
            } else {
                format!("{{\"format\":\"json\",\"violations\":{n}}}")
            }
        }
        ReportFormat::Markdown => {
            let mut out = format!("## Quality Report\nformat: markdown\nviolations: {n}");
            if opts.include_violations {
                for v in &violations {
                    out.push_str(&format!(
                        "\n- row={} col={} rule={:?}",
                        v.row_index, v.rule.column, v.rule.kind,
                    ));
                }
            }
            out
        }
    }
}

/// `build_quality_report` の薄いラッパー（CLI コマンドハンドラ相当）。
pub fn cmd_quality_report(
    check: &QualityCheck,
    rows: &[Vec<String>],
    opts: &QualityReportOptions,
) -> String {
    build_quality_report(check, rows, opts)
}

// ── v81.8.0: AnomalyDetector / detect_anomaly ────────────────────────────────

/// ベースライン統計と Z スコア閾値を保持する異常検知器。
///
/// 依存: v81.2.0 の `DistributionStats` / `compute_distribution_stats`
#[derive(Debug, Clone)]
pub struct AnomalyDetector {
    pub baseline_stats: DistributionStats,
    pub z_threshold: f64,
}

/// 単一値の異常検知結果。
#[derive(Debug, Clone)]
pub struct AnomalyResult {
    pub is_anomaly: bool,
    pub z_score: f64,
    pub value: f64,
}

impl AnomalyDetector {
    /// `values` から `compute_distribution_stats` でベースラインを構築する。
    pub fn from_baseline(values: &[f64], z_threshold: f64) -> AnomalyDetector {
        AnomalyDetector {
            baseline_stats: compute_distribution_stats(values),
            z_threshold,
        }
    }
}

/// `detector` のベースラインに対して `value` の Z スコアを計算し、閾値と比較する。
///
/// - Z スコア: `|value - mean| / std_dev`
/// - `std_dev == 0.0` のとき `z_score = 0.0`, `is_anomaly = false`（ゼロ除算ガード）
/// - `is_anomaly = z_score > z_threshold`
/// - `value` が `f64::NAN` の場合、Z スコアも NaN となり `is_anomaly = false` になる（未定義動作）
pub fn detect_anomaly(detector: &AnomalyDetector, value: f64) -> AnomalyResult {
    let mean = detector.baseline_stats.mean;
    let std_dev = detector.baseline_stats.std_dev;
    let z_score = if std_dev == 0.0 {
        0.0
    } else {
        (value - mean).abs() / std_dev
    };
    AnomalyResult {
        is_anomaly: z_score > detector.z_threshold,
        z_score,
        value,
    }
}

/// `values` の各要素に `detect_anomaly` を適用し、結果をすべて返す。
pub fn scan_for_anomalies(detector: &AnomalyDetector, values: &[f64]) -> Vec<AnomalyResult> {
    values.iter().map(|&v| detect_anomaly(detector, v)).collect()
}

/// `results` の集計サマリーを文字列に変換する。
///
/// フォーマット: `"anomaly_report total={n} anomalies={k}"`
pub fn format_anomaly_report(results: &[AnomalyResult]) -> String {
    let total = results.len();
    let anomalies = results.iter().filter(|r| r.is_anomaly).count();
    format!("anomaly_report total={total} anomalies={anomalies}")
}

// ── v82.1.0: IoContract / ContractField 型基盤 ────────────────────────────

/// パイプライン契約フィールドの型。
#[derive(Debug, Clone, PartialEq)]
pub enum ContractFieldType {
    Str,
    Int,
    Float,
    Bool,
    Nullable(Box<ContractFieldType>),
    List(Box<ContractFieldType>),
}

/// IoContract の個々のフィールド定義。
#[derive(Debug, Clone, PartialEq)]
pub struct ContractField {
    pub name: String,
    pub field_type: ContractFieldType,
    pub required: bool,
}

/// パイプラインの入出力インターフェース契約。
#[derive(Debug, Clone, PartialEq)]
pub struct IoContract {
    pub name: String,
    pub version: String,
    pub input: Vec<ContractField>,
    pub output: Vec<ContractField>,
}

/// IoContract 検証結果。
#[derive(Debug, PartialEq)]
pub struct ContractValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
}

/// `actual_input` が `contract.input` の必須フィールドをすべて含むかを検証する。
///
/// - `required=true` のフィールドが `actual_input` に存在しない場合は `errors` に追加
/// - フィールド名の照合のみ（field_type の照合は本バージョンでは非対応）
pub fn validate_io_contract(
    contract: &IoContract,
    actual_input: &[ContractField],
) -> ContractValidationResult {
    let mut errors = Vec::new();
    for expected in &contract.input {
        if expected.required {
            let found = actual_input.iter().any(|f| f.name == expected.name);
            if !found {
                errors.push(format!("missing required field: {}", expected.name));
            }
        }
    }
    ContractValidationResult {
        valid: errors.is_empty(),
        errors,
    }
}

// ── v82.2.0: SlaContract / SlaTarget / SlaStatus ─────────────────────────

/// SLA 目標値。
#[derive(Debug, Clone, PartialEq)]
pub struct SlaTarget {
    pub max_latency_ms: u64,
    pub min_throughput_rps: f64,
    /// 将来拡張用（本バージョンの `evaluate_sla` では評価しない）
    pub min_availability_pct: f64,
}

/// SLA を型として宣言するパイプライン契約。
/// `adaptive_strategy` は Favnir 3.0 の `!Adaptive` エフェクトと連携する文字列スタブ。
/// `cache_ttl_secs` は `!Cached` エフェクトと連携するキャッシュ TTL スタブ。
#[derive(Debug, Clone, PartialEq)]
pub struct SlaContract {
    pub name: String,
    pub target: SlaTarget,
    pub adaptive_strategy: Option<String>,
    pub cache_ttl_secs: Option<u64>,
}

/// SLA 評価結果。
/// - `Met`: すべての目標を満たしている
/// - `AtRisk(String)`: 目標に近いが超過はしていない（将来拡張用、本バージョンでは未使用）
/// - `Breached(String)`: 目標を超過している（メッセージに詳細を含む）
#[derive(Debug, PartialEq)]
pub enum SlaStatus {
    Met,
    AtRisk(String),
    Breached(String),
}

/// 実測値と SLA 目標を比較し、`SlaStatus` を返す。
///
/// 判定順序:
/// 1. `actual_latency_ms > target.max_latency_ms` → `Breached`
/// 2. `actual_rps < target.min_throughput_rps` → `Breached`
/// 3. いずれも違反なし → `Met`
///
/// `min_availability_pct` の評価は将来拡張用のため本バージョンでは行わない。
pub fn evaluate_sla(
    contract: &SlaContract,
    actual_latency_ms: u64,
    actual_rps: f64,
) -> SlaStatus {
    if actual_latency_ms > contract.target.max_latency_ms {
        return SlaStatus::Breached(format!(
            "latency exceeded: {} ms > {} ms",
            actual_latency_ms, contract.target.max_latency_ms
        ));
    }
    if actual_rps < contract.target.min_throughput_rps {
        return SlaStatus::Breached(format!(
            "throughput below minimum: {} rps < {} rps",
            actual_rps, contract.target.min_throughput_rps
        ));
    }
    SlaStatus::Met
}

/// `SlaStatus` を人間が読める文字列に変換する。
pub fn format_sla_status(status: &SlaStatus) -> String {
    match status {
        SlaStatus::Met => "SLA: Met".into(),
        SlaStatus::AtRisk(msg) => format!("SLA: AtRisk — {msg}"),
        SlaStatus::Breached(msg) => format!("SLA: Breached — {msg}"),
    }
}

// ── v82.3.0: ContractDependency / DependencyGraph ────────────────────────────

/// パイプライン間の単一契約依存エッジ。
#[derive(Debug, Clone, PartialEq)]
pub struct ContractDependency {
    pub upstream: String,
    pub downstream: String,
    pub output_contract: String,
}

/// 契約間の有向依存グラフ。
#[derive(Debug, Clone, PartialEq)]
pub struct DependencyGraph {
    pub dependencies: Vec<ContractDependency>,
}

/// `IoContract` のリストから依存グラフを構築する。
///
/// contracts[i].output の field name 群と contracts[j].input の field name 群に
/// 共通要素があれば i → j のエッジを作成する。
///
/// **注意**: `"id"` や `"name"` のような汎用フィールド名を複数の契約が共有する場合、
/// 意図しないエッジが生成されることがある。フィールド名の一意性は契約設計者が保証すること。
pub fn build_dependency_graph(contracts: &[IoContract]) -> DependencyGraph {
    let mut deps = Vec::new();
    for (i, upstream) in contracts.iter().enumerate() {
        let out_names: std::collections::HashSet<&str> =
            upstream.output.iter().map(|f| f.name.as_str()).collect();
        for (j, downstream) in contracts.iter().enumerate() {
            if i == j {
                continue;
            }
            let has_match = downstream
                .input
                .iter()
                .any(|f| out_names.contains(f.name.as_str()));
            if has_match {
                deps.push(ContractDependency {
                    upstream: upstream.name.clone(),
                    downstream: downstream.name.clone(),
                    output_contract: upstream.name.clone(),
                });
            }
        }
    }
    DependencyGraph { dependencies: deps }
}

/// 依存グラフ内の 2 ノードサイクル（A→B かつ B→A）を検出する。
///
/// v82.3.0 スコープ: 2 ノードサイクルのみ対応。
/// 長いサイクル（A→B→C→A 等）は将来バージョンで対応予定。
pub fn detect_circular_dependencies(graph: &DependencyGraph) -> Vec<Vec<String>> {
    let mut cycles: Vec<Vec<String>> = Vec::new();
    for dep in &graph.dependencies {
        // dep: A → B が存在するとき、B → A も存在すればサイクル
        let reverse_exists = graph.dependencies.iter().any(|d| {
            d.upstream == dep.downstream && d.downstream == dep.upstream
        });
        if reverse_exists {
            let mut cycle = vec![dep.upstream.clone(), dep.downstream.clone()];
            cycle.sort(); // 重複除去のため正規化
            if !cycles.contains(&cycle) {
                cycles.push(cycle);
            }
        }
    }
    cycles
}

/// 依存グラフを人間が読める文字列に変換する。
///
/// 各エッジを `"{upstream} -> {downstream} ({output_contract})"` 形式で改行結合する。
/// エッジが空の場合は空文字列を返す。
pub fn format_dependency_graph(graph: &DependencyGraph) -> String {
    graph
        .dependencies
        .iter()
        .map(|d| format!("{} -> {} ({})", d.upstream, d.downstream, d.output_contract))
        .collect::<Vec<_>>()
        .join("\n")
}

// ── v82.4.0: ContractViolation / ContractViolationReport ─────────────────────

/// 契約違反の種別。
///
/// `MissingField(String)` / `ExtraField(String)` / `NullNotAllowed(String)` の inner `String` は
/// 違反対象のフィールド名またはコンテキスト情報を保持する将来拡張用スロット。
/// `format_violation_report` では現バージョンにおいて `ContractViolation.field` を使用するため
/// inner String の値は出力に反映されないが、呼び出し元がデバッグ情報を添付できるよう保持する。
#[derive(Debug, Clone, PartialEq)]
pub enum ViolationKind {
    /// フィールドの型が期待と異なる。
    TypeMismatch { expected: String, actual: String },
    /// 必須フィールドが存在しない（inner String: 欠損フィールド名のコンテキスト）。
    MissingField(String),
    /// 契約に定義されていない余分なフィールドがある（inner String: 余分フィールド名のコンテキスト）。
    ExtraField(String),
    /// null 禁止フィールドに null が入った（inner String: 対象フィールド名のコンテキスト）。
    NullNotAllowed(String),
}

/// 単一フィールドの契約違反。
#[derive(Debug, Clone, PartialEq)]
pub struct ContractViolation {
    pub field: String,
    pub kind: ViolationKind,
    pub row_index: Option<usize>,
}

/// 契約検証の違反レポート。
#[derive(Debug, Clone, PartialEq)]
pub struct ContractViolationReport {
    pub contract_name: String,
    pub violations: Vec<ContractViolation>,
}

/// 違反の重大度を返す。
///
/// - `TypeMismatch` / `MissingField` / `NullNotAllowed` → `RuleSeverity::Error`
/// - `ExtraField` → `RuleSeverity::Warning`
pub fn violation_severity(violation: &ContractViolation) -> RuleSeverity {
    match &violation.kind {
        ViolationKind::TypeMismatch { .. } => RuleSeverity::Error,
        ViolationKind::MissingField(_) => RuleSeverity::Error,
        ViolationKind::NullNotAllowed(_) => RuleSeverity::Error,
        ViolationKind::ExtraField(_) => RuleSeverity::Warning,
    }
}

/// 契約違反レポートを人間が読める文字列に変換する。
///
/// 違反なし: `"{contract_name}: no violations"`
/// 違反あり: ヘッダ + 各違反を `"[{severity}] field '{field}': {description}"` 形式で改行結合。
pub fn format_violation_report(report: &ContractViolationReport) -> String {
    if report.violations.is_empty() {
        return format!("{}: no violations", report.contract_name);
    }
    let mut lines = vec![format!(
        "{}: {} violation(s)",
        report.contract_name,
        report.violations.len()
    )];
    for v in &report.violations {
        let sev_str = match violation_severity(v) {
            RuleSeverity::Error => "Error",
            RuleSeverity::Warning => "Warning",
        };
        let kind_str = match &v.kind {
            ViolationKind::TypeMismatch { expected, actual } => {
                format!("type mismatch (expected={expected}, actual={actual})")
            }
            ViolationKind::MissingField(_) => "missing required field".into(),
            ViolationKind::ExtraField(_) => "extra field not in contract".into(),
            ViolationKind::NullNotAllowed(_) => "null not allowed".into(),
        };
        let row_str = v
            .row_index
            .map_or(String::new(), |r| format!(" at row {r}"));
        lines.push(format!("[{sev_str}] field '{}': {kind_str}{row_str}", v.field));
    }
    lines.join("\n")
}

// ── v82.5.0: infer_contract / merge_contracts / format_contract_as_toml ───────

/// 型名文字列から `ContractFieldType` を推論する。
///
/// - `"Int"` → `Int`、`"Float"` → `Float`、`"Bool"` → `Bool`
/// - それ以外（`"Str"` / 未知の型）→ `Str`（デフォルト）
pub fn infer_field_type_from_str(type_name: &str) -> ContractFieldType {
    match type_name {
        "Int" => ContractFieldType::Int,
        "Float" => ContractFieldType::Float,
        "Bool" => ContractFieldType::Bool,
        _ => ContractFieldType::Str,
    }
}

/// `SchemaSnapshot` から `IoContract` を自動生成する。
///
/// - `nullable: false` の列 → base_type / `required: true`
/// - `nullable: true` の列 → `Nullable(base_type)` / `required: false`
/// - すべての列を `input` に設定し、`output` は空とする
pub fn infer_contract_from_schema(
    schema: &SchemaSnapshot,
    name: &str,
    version: &str,
) -> IoContract {
    let input = schema
        .columns
        .iter()
        .map(|col| {
            let base_type = infer_field_type_from_str(&col.type_name);
            let field_type = if col.nullable {
                ContractFieldType::Nullable(Box::new(base_type))
            } else {
                base_type
            };
            ContractField {
                name: col.name.clone(),
                field_type,
                required: !col.nullable,
            }
        })
        .collect();
    IoContract {
        name: name.into(),
        version: version.into(),
        input,
        output: vec![],
    }
}

/// 2 つの `IoContract` をマージする。`override_` が `base` を上書きする。
///
/// - 同名フィールドは `override_` の値を優先（`input` / `output` 独立して処理）
/// - `base` にしかないフィールドはそのまま残す（base の順を維持）
/// - `override_` にしかないフィールドは末尾に追加
/// - `name` / `version` は `override_` の値を使用
pub fn merge_contracts(base: &IoContract, override_: &IoContract) -> IoContract {
    let merge_fields = |base_fields: &[ContractField], override_fields: &[ContractField]| {
        let mut result: Vec<ContractField> = base_fields
            .iter()
            .map(|bf| {
                override_fields
                    .iter()
                    .find(|of| of.name == bf.name)
                    .cloned()
                    .unwrap_or_else(|| bf.clone())
            })
            .collect();
        for of in override_fields {
            if !base_fields.iter().any(|bf| bf.name == of.name) {
                result.push(of.clone());
            }
        }
        result
    };
    IoContract {
        name: override_.name.clone(),
        version: override_.version.clone(),
        input: merge_fields(&base.input, &override_.input),
        output: merge_fields(&base.output, &override_.output),
    }
}

/// `IoContract` を TOML ライクな文字列に変換する（toml クレート不使用）。
pub fn format_contract_as_toml(contract: &IoContract) -> String {
    fn field_type_to_str(ft: &ContractFieldType) -> String {
        match ft {
            ContractFieldType::Str => "Str".into(),
            ContractFieldType::Int => "Int".into(),
            ContractFieldType::Float => "Float".into(),
            ContractFieldType::Bool => "Bool".into(),
            ContractFieldType::Nullable(inner) => {
                format!("Nullable({})", field_type_to_str(inner))
            }
            ContractFieldType::List(inner) => format!("List({})", field_type_to_str(inner)),
        }
    }
    let mut lines = vec![
        "[contract]".into(),
        format!("name = \"{}\"", contract.name),
        format!("version = \"{}\"", contract.version),
    ];
    for field in &contract.input {
        lines.push(String::new());
        lines.push("[[input]]".into());
        lines.push(format!("name = \"{}\"", field.name));
        lines.push(format!("type = \"{}\"", field_type_to_str(&field.field_type)));
        lines.push(format!("required = {}", field.required));
    }
    for field in &contract.output {
        lines.push(String::new());
        lines.push("[[output]]".into());
        lines.push(format!("name = \"{}\"", field.name));
        lines.push(format!("type = \"{}\"", field_type_to_str(&field.field_type)));
        lines.push(format!("required = {}", field.required));
    }
    lines.join("\n")
}

// ── v82.6.0: ContractVersion / CompatibilityResult ───────────────────────────

/// セマンティックバージョニング形式の契約バージョン。
#[derive(Debug, Clone, PartialEq)]
pub struct ContractVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl ContractVersion {
    /// `"major.minor.patch"` 形式の文字列をパースする。
    /// フォーマット不正な場合は `Err("invalid version: {s}")` を返す。
    pub fn parse(s: &str) -> Result<ContractVersion, String> {
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 3 {
            return Err(format!("invalid version: {s}"));
        }
        let parse_u32 = |p: &str| {
            p.parse::<u32>().map_err(|_| format!("invalid version: {s}"))
        };
        Ok(ContractVersion {
            major: parse_u32(parts[0])?,
            minor: parse_u32(parts[1])?,
            patch: parse_u32(parts[2])?,
        })
    }
}

/// 契約の後方互換性チェック結果。
#[derive(Debug, PartialEq)]
pub enum CompatibilityResult {
    /// 変更なし。
    Compatible,
    /// 既存消費者への影響なしの変更（フィールドの追加）。
    BackwardsCompatible(Vec<String>),
    /// 既存消費者を壊す変更（必須フィールドの削除・型変更）。
    Breaking(Vec<String>),
}

/// 2 つの `IoContract` の input フィールドを比較して後方互換性を判定する。
///
/// 判定順序（優先度降順）:
/// 1. Breaking: old の required フィールドが new_ に存在しない
/// 2. Breaking: 同名フィールドの型が変わった（required/optional 問わず）
/// 3. BackwardsCompatible: new_ に old にないフィールドが追加された
/// 4. Compatible: 変更なし
///
/// `Breaking` / `BackwardsCompatible` の内部 `Vec<String>` はアルファベット順にソートされる。
pub fn check_contract_compatibility(old: &IoContract, new_: &IoContract) -> CompatibilityResult {
    let mut breaking: Vec<String> = Vec::new();

    // 1. required フィールドが削除された
    for old_field in &old.input {
        if old_field.required && !new_.input.iter().any(|f| f.name == old_field.name) {
            breaking.push(old_field.name.clone());
        }
    }

    // 2. 同名フィールドの型が変わった（required/optional 問わず）
    for old_field in &old.input {
        if let Some(new_field) = new_.input.iter().find(|f| f.name == old_field.name) {
            if new_field.field_type != old_field.field_type {
                breaking.push(old_field.name.clone());
            }
        }
    }

    if !breaking.is_empty() {
        breaking.sort();
        breaking.dedup();
        return CompatibilityResult::Breaking(breaking);
    }

    // 3. フィールドが追加された（required/optional 問わず）
    let mut added: Vec<String> = new_
        .input
        .iter()
        .filter(|nf| !old.input.iter().any(|of| of.name == nf.name))
        .map(|nf| nf.name.clone())
        .collect();

    if !added.is_empty() {
        added.sort(); // Breaking と同様にアルファベット順で返す
        return CompatibilityResult::BackwardsCompatible(added);
    }

    CompatibilityResult::Compatible
}

/// `CompatibilityResult` を人間が読める文字列に変換する。
///
/// `Breaking(vec![])` / `BackwardsCompatible(vec![])` が渡された場合、
/// `check_contract_compatibility` 経由では発生しないが、出力は `"Breaking: []"` となる。
pub fn format_compatibility_result(result: &CompatibilityResult) -> String {
    match result {
        CompatibilityResult::Compatible => "Compatible".into(),
        CompatibilityResult::BackwardsCompatible(fields) => {
            format!("BackwardsCompatible: added [{}]", fields.join(", "))
        }
        CompatibilityResult::Breaking(fields) => {
            format!("Breaking: [{}]", fields.join(", "))
        }
    }
}

// ── v82.7.0: VerifyContractOptions / ContractVerifyResult / cmd_verify_contract ──

/// `fav verify --contract` コマンドのオプション。
/// `contract_path` / `strict` は将来拡張のための予約フィールドであり、
/// 本バージョンの `cmd_verify_contract` では参照しない。
#[derive(Debug, Clone)]
pub struct VerifyContractOptions {
    pub contract_path: String,
    pub input_schema: Option<String>,
    pub strict: bool,
}

/// `cmd_verify_contract` の戻り値。
/// IoContract 検証結果と SLA 評価結果を保持する。
#[derive(Debug, PartialEq)]
pub struct ContractVerifyResult {
    pub io_result: ContractValidationResult,
    pub sla_result: Option<SlaStatus>,
}

/// `fav verify --contract` のコアロジック。
///
/// - `validate_io_contract` で IoContract を検証して `io_result` を取得する
/// - `sla_check` が Some なら `evaluate_sla` を呼んで `sla_result` を設定する
/// - `_options` は将来拡張用（本バージョンでは未使用）
pub fn cmd_verify_contract(
    _options: &VerifyContractOptions,
    contract: &IoContract,
    actual_input: &[ContractField],
    sla_check: Option<(&SlaContract, u64, f64)>,
) -> ContractVerifyResult {
    let io_result = validate_io_contract(contract, actual_input);
    let sla_result = sla_check.map(|(sla, lat, rps)| evaluate_sla(sla, lat, rps));
    ContractVerifyResult { io_result, sla_result }
}

/// `ContractVerifyResult` を人間が読める文字列に変換する。
///
/// - `io_result.valid == true` → `"Contract: PASS"`
/// - `io_result.valid == false` → `"Contract: FAIL ({n} error(s))"`
/// - `sla_result` が Some なら次行に `format_sla_status` の結果を追加する
pub fn format_verify_result(result: &ContractVerifyResult) -> String {
    let io_line = if result.io_result.valid {
        "Contract: PASS".to_string()
    } else {
        format!("Contract: FAIL ({} error(s))", result.io_result.errors.len())
    };
    match &result.sla_result {
        Some(sla_status) => format!("{}\n{}", io_line, format_sla_status(sla_status)),
        None => io_line,
    }
}

// ─── v82.8.0: ContractRegistry ───────────────────────────────────────────────

/// 契約レジストリの 1 エントリ。
#[derive(Debug, Clone, PartialEq)]
pub struct ContractRegistryEntry {
    pub name: String,
    pub version: ContractVersion,
    pub contract: IoContract,
    /// 登録日時。RFC 3339 / ISO 8601 形式（例: `"2026-08-20T00:00:00Z"`）を期待する。
    pub registered_at: String,
}

/// チーム間で契約を共有・検索するローカルレジストリ。
#[derive(Debug, Clone, PartialEq)]
pub struct ContractRegistry {
    pub entries: Vec<ContractRegistryEntry>,
}

impl Default for ContractRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ContractRegistry {
    /// 空のレジストリを作成する。
    pub fn new() -> Self {
        ContractRegistry { entries: vec![] }
    }

    /// エントリを登録した新しい `ContractRegistry` を返す（不変更新スタイル）。
    pub fn register(&self, entry: ContractRegistryEntry) -> ContractRegistry {
        let mut entries = self.entries.clone();
        entries.push(entry);
        ContractRegistry { entries }
    }

    /// 名前（とオプションのバージョン文字列）でエントリを検索する。
    ///
    /// - `version` が Some のとき: `ContractVersion::parse` でパースし完全一致検索。
    ///   バージョン文字列が不正（例: `"1.0"` や `"invalid"`）な場合も `None` を返す。
    /// - `version` が None のとき: 同名エントリのうち最後に登録されたものを返す。
    pub fn lookup(&self, name: &str, version: Option<&str>) -> Option<&ContractRegistryEntry> {
        match version {
            Some(v) => {
                let target = ContractVersion::parse(v).ok()?;
                self.entries.iter().find(|e| e.name == name && e.version == target)
            }
            None => self.entries.iter().rev().find(|e| e.name == name),
        }
    }

    /// 全エントリへの参照を登録順に返す。
    pub fn list_all(&self) -> Vec<&ContractRegistryEntry> {
        self.entries.iter().collect()
    }
}

/// レジストリの全エントリを一覧表示する文字列を返す。
///
/// ```text
/// Registry (2 entries):
///   orders v1.0.0 — registered_at: 2026-08-20T00:00:00Z
///   payments v2.1.0 — registered_at: 2026-08-20T01:00:00Z
/// ```
pub fn format_registry_listing(registry: &ContractRegistry) -> String {
    let mut lines = vec![format!("Registry ({} entries):", registry.entries.len())];
    for e in &registry.entries {
        lines.push(format!(
            "  {} v{}.{}.{} — registered_at: {}",
            e.name, e.version.major, e.version.minor, e.version.patch, e.registered_at
        ));
    }
    lines.join("\n")
}

// ---------------------------------------------------------------------------
// v83.1.0 — PipelineMetrics 型（実行統計・レイテンシ）
// ---------------------------------------------------------------------------

/// ステージ単位の実行統計。
#[derive(Debug, Clone, PartialEq)]
pub struct StageMetrics {
    pub stage_name: String,
    pub duration_ms: u64,
    pub rows_processed: usize,
    pub rows_failed: usize,
}

/// パイプライン全体の統計集約。
#[derive(Debug, Clone, PartialEq)]
pub struct PipelineMetrics {
    pub pipeline_name: String,
    pub stages: Vec<StageMetrics>,
    pub total_duration_ms: u64,
    pub started_at: String,
}

/// ステージ一覧から `PipelineMetrics` を構築する。
///
/// `total_duration_ms` はステージの `duration_ms` の単純合計（直列パイプライン想定）。
/// 並列ステージ（`par`）が混在する場合は wall-clock total とは異なる値になる。
pub fn compute_pipeline_metrics(
    pipeline_name: &str,
    stages: Vec<StageMetrics>,
    started_at: &str,
) -> PipelineMetrics {
    let total_duration_ms = stages.iter().map(|s| s.duration_ms).sum();
    PipelineMetrics {
        pipeline_name: pipeline_name.to_string(),
        stages,
        total_duration_ms,
        started_at: started_at.to_string(),
    }
}

/// メトリクスのサマリーテキストを返す。
///
/// ```text
/// Pipeline: etl_main
/// Started: 2026-08-21T00:00:00Z
/// Total: 350ms
/// Stages:
///   load: 200ms (1000 rows, 0 failed)
///   transform: 150ms (1000 rows, 2 failed)
/// ```
pub fn format_metrics_summary(metrics: &PipelineMetrics) -> String {
    let header = format!(
        "Pipeline: {}\nStarted: {}\nTotal: {}ms\nStages:",
        metrics.pipeline_name, metrics.started_at, metrics.total_duration_ms
    );
    if metrics.stages.is_empty() {
        return header;
    }
    let stage_lines: Vec<String> = metrics
        .stages
        .iter()
        .map(|s| {
            format!(
                "  {}: {}ms ({} rows, {} failed)",
                s.stage_name, s.duration_ms, s.rows_processed, s.rows_failed
            )
        })
        .collect();
    format!("{}\n{}", header, stage_lines.join("\n"))
}

/// 最も `duration_ms` が大きいステージへの参照を返す。`stages` が空の場合は `None`。
pub fn slowest_stage(metrics: &PipelineMetrics) -> Option<&StageMetrics> {
    metrics.stages.iter().max_by_key(|s| s.duration_ms)
}

// ---------------------------------------------------------------------------
// v83.2.0 — AlertRule / AlertThreshold（アラート型）
// ---------------------------------------------------------------------------

/// アラートの重要度。
#[derive(Debug, Clone, PartialEq)]
pub enum AlertSeverity {
    Critical,
    Warning,
    Info,
}

/// 閾値の比較演算子。
#[derive(Debug, Clone, PartialEq)]
pub enum ThresholdOp {
    GreaterThan,
    LessThan,
    EqualTo,
}

/// メトリクスの閾値定義。
///
/// `metric` は `PipelineMetrics` のフィールドに対応する文字列:
/// - `"total_duration_ms"` — パイプライン全体の実行時間
/// - `"rows_failed"` — 全ステージの失敗行数合計
/// - `"rows_processed"` — 全ステージの処理行数合計
#[derive(Debug, Clone, PartialEq)]
pub struct AlertThreshold {
    pub metric: String,
    pub operator: ThresholdOp,
    pub value: f64,
}

/// アラートルール定義。
#[derive(Debug, Clone, PartialEq)]
pub struct AlertRule {
    pub name: String,
    pub threshold: AlertThreshold,
    pub severity: AlertSeverity,
    pub message: String,
}

/// 発火したアラート。
#[derive(Debug, Clone, PartialEq)]
pub struct AlertFiring {
    pub rule: AlertRule,
    pub current_value: f64,
    pub fired_at: String,
}

/// ルール一覧を `PipelineMetrics` に対して評価し、発火したアラートを返す。
///
/// 未知の `metric` 文字列を持つルールは評価をスキップする（アラートを発火しない）。
/// `fired_at` は呼び出し元が渡す文字列（テスト容易性のため）。
pub fn evaluate_alert_rules(
    rules: &[AlertRule],
    metrics: &PipelineMetrics,
    fired_at: &str,
) -> Vec<AlertFiring> {
    let rows_failed: f64 = metrics.stages.iter().map(|s| s.rows_failed).sum::<usize>() as f64;
    let rows_processed: f64 = metrics.stages.iter().map(|s| s.rows_processed).sum::<usize>() as f64;

    let mut firings = Vec::new();
    for rule in rules {
        let current_value = match rule.threshold.metric.as_str() {
            "total_duration_ms" => metrics.total_duration_ms as f64,
            "rows_failed" => rows_failed,
            "rows_processed" => rows_processed,
            _ => continue,
        };
        let fired = match rule.threshold.operator {
            ThresholdOp::GreaterThan => current_value > rule.threshold.value,
            ThresholdOp::LessThan => current_value < rule.threshold.value,
            // NOTE: f64::EPSILON は整数由来メトリクス（usize→f64 / u64→f64）専用。
            // 浮動小数点演算を経由したメトリクスでは相対誤差が EPSILON を超える可能性がある。
            ThresholdOp::EqualTo => (current_value - rule.threshold.value).abs() < f64::EPSILON,
        };
        if fired {
            firings.push(AlertFiring {
                rule: rule.clone(),
                current_value,
                fired_at: fired_at.to_string(),
            });
        }
    }
    firings
}

// ---------------------------------------------------------------------------
// v83.3.0 — SloTarget / SloStatus（SLO 型）
// ---------------------------------------------------------------------------

/// SLO の目標宣言。
#[derive(Debug, Clone, PartialEq)]
pub struct SloTarget {
    pub name: String,
    pub objective_pct: f64,
    pub window_hours: u64,
}

/// 観測期間内の good/total イベント数。
#[derive(Debug, Clone, PartialEq)]
pub struct SloMeasurement {
    pub good_events: u64,
    pub total_events: u64,
    pub window_hours: u64,
}

/// SLO の達成状況。
#[derive(Debug, Clone, PartialEq)]
pub struct SloStatus {
    pub target: SloTarget,
    /// good_events / total_events * 100.0
    pub current_pct: f64,
    /// (current_pct - objective_pct) / (100.0 - objective_pct) * 100.0
    pub error_budget_remaining_pct: f64,
    /// current_pct < objective_pct
    pub breached: bool,
}

/// `SloMeasurement` から `SloStatus` を計算する。
///
/// `total_events == 0` の場合はイベントなし = 全成功とみなし、
/// `current_pct = 100.0`、`error_budget_remaining_pct = 100.0`、`breached = false` を返す。
///
/// `objective_pct == 100.0` の場合はゼロ除算を避けるため `error_budget_remaining_pct = 0.0` を返す。
/// このとき `current_pct < 100.0` であれば `breached = true` となるが、
/// `error_budget_remaining_pct` は `0.0` のまま変化しない。
/// breach の判定には必ず `breached` フィールドを参照すること。
///
/// # Panics（debug build のみ）
/// `good_events > total_events` の場合、デバッグビルドでアサーション違反が発生する。
pub fn compute_slo_status(target: &SloTarget, measurement: &SloMeasurement) -> SloStatus {
    debug_assert!(
        measurement.good_events <= measurement.total_events,
        "good_events ({}) must not exceed total_events ({})",
        measurement.good_events,
        measurement.total_events
    );
    if measurement.total_events == 0 {
        return SloStatus {
            target: target.clone(),
            current_pct: 100.0,
            error_budget_remaining_pct: 100.0,
            breached: false,
        };
    }
    let current_pct =
        measurement.good_events as f64 / measurement.total_events as f64 * 100.0;
    let error_budget_remaining_pct =
        if (100.0 - target.objective_pct).abs() < f64::EPSILON {
            0.0
        } else {
            (current_pct - target.objective_pct) / (100.0 - target.objective_pct) * 100.0
        };
    let breached = current_pct < target.objective_pct;
    SloStatus {
        target: target.clone(),
        current_pct,
        error_budget_remaining_pct,
        breached,
    }
}

/// SLO 達成状況のテキストサマリーを返す。
///
/// ```text
/// SLO: api_availability
/// Objective: 99.9%
/// Current: 99.95%
/// Error Budget: 50.00% remaining
/// Status: OK
/// ```
pub fn format_slo_status(status: &SloStatus) -> String {
    let status_str = if status.breached { "BREACHED" } else { "OK" };
    format!(
        "SLO: {}\nObjective: {:.1}%\nCurrent: {:.2}%\nError Budget: {:.2}% remaining\nStatus: {}",
        status.target.name,
        status.target.objective_pct,
        status.current_pct,
        status.error_budget_remaining_pct,
        status_str,
    )
}

// ---------------------------------------------------------------------------
// v83.4.0 — ExecutionCost / CostBudget（コスト追跡）
// ---------------------------------------------------------------------------

/// パイプライン実行のリソース使用量。
#[derive(Debug, Clone, PartialEq)]
pub struct ResourceUsage {
    pub cpu_seconds: f64,
    pub memory_mb: f64,
    pub io_mb: f64,
}

/// 実行コストの集計。
#[derive(Debug, Clone, PartialEq)]
pub struct ExecutionCost {
    pub resource: ResourceUsage,
    pub estimated_usd: f64,
    pub pipeline_name: String,
}

/// 実行コストの予算上限。
#[derive(Debug, Clone, PartialEq)]
pub struct CostBudget {
    pub max_usd_per_run: f64,
    pub max_cpu_seconds: f64,
}

/// 予算チェック結果。
///
/// - `UnderBudget` — USD・CPU ともに予算の 80% 未満
/// - `NearLimit(f64)` — いずれかが 80%〜100% の範囲（f64 は最大使用率 %）
///   `max_pct == 100.0`（予算ちょうど使い切り）も `NearLimit(100.0)` を返す。
/// - `OverBudget(f64)` — いずれかが 100% 超（f64 は超過分 %、つまり最大使用率 - 100.0）
///
/// 80% 閾値は Prometheus / Grafana 等の一般的な監視ツールでの
/// "warn before breach" 慣習に準拠。残り 20% で早期警告を発し、
/// 次の実行までに予算調整できるようにする。
#[derive(Debug, Clone, PartialEq)]
pub enum BudgetStatus {
    UnderBudget,
    NearLimit(f64),
    OverBudget(f64),
}

/// コストと予算を照合して `BudgetStatus` を返す。
///
/// `max_usd_per_run == 0.0` / `max_cpu_seconds == 0.0` は上限なしとみなし、
/// それぞれの使用率を 0.0 として扱う（ゼロ除算ガード）。
pub fn evaluate_cost_budget(cost: &ExecutionCost, budget: &CostBudget) -> BudgetStatus {
    let usd_pct = if budget.max_usd_per_run == 0.0 {
        0.0
    } else {
        cost.estimated_usd / budget.max_usd_per_run * 100.0
    };
    let cpu_pct = if budget.max_cpu_seconds == 0.0 {
        0.0
    } else {
        cost.resource.cpu_seconds / budget.max_cpu_seconds * 100.0
    };
    let max_pct = usd_pct.max(cpu_pct);
    if max_pct > 100.0 {
        BudgetStatus::OverBudget(max_pct - 100.0)
    } else if max_pct >= 80.0 {
        BudgetStatus::NearLimit(max_pct)
    } else {
        BudgetStatus::UnderBudget
    }
}

/// コストレポートのテキストを返す。
///
/// ```text
/// Pipeline: etl_main
/// CPU: 30.0s
/// Memory: 512.0MB
/// IO: 100.0MB
/// Cost: $1.50
/// Budget: UnderBudget
/// ```
pub fn format_cost_report(cost: &ExecutionCost, status: &BudgetStatus) -> String {
    let budget_str = match status {
        BudgetStatus::UnderBudget => "UnderBudget".to_string(),
        BudgetStatus::NearLimit(pct) => format!("NearLimit ({:.2}%)", pct),
        BudgetStatus::OverBudget(over) => format!("OverBudget (+{:.2}%)", over),
    };
    format!(
        "Pipeline: {}\nCPU: {:.1}s\nMemory: {:.1}MB\nIO: {:.1}MB\nCost: ${:.2}\nBudget: {}",
        cost.pipeline_name,
        cost.resource.cpu_seconds,
        cost.resource.memory_mb,
        cost.resource.io_mb,
        cost.estimated_usd,
        budget_str,
    )
}

// ---------------------------------------------------------------------------
// v83.5.0 — 分散トレーシング強化（TraceContext / TraceSpan）
// ---------------------------------------------------------------------------

/// OTel W3C Trace Context 形式のコンテキスト。
#[derive(Debug, Clone, PartialEq)]
pub struct TraceContext {
    /// 32 桁 hex（W3C traceparent の trace-id）
    pub trace_id: String,
    /// 16 桁 hex（W3C traceparent の parent-id）
    pub span_id: String,
    /// ルートスパンは None
    pub parent_span_id: Option<String>,
}

impl TraceContext {
    /// 新しいルートコンテキストを作成する。
    /// `trace_id` と `span_id` は `uuid::Uuid::new_v4()` で生成し、
    /// `parent_span_id = None`。
    pub fn new_root() -> TraceContext {
        let trace_uuid = uuid::Uuid::new_v4();
        let span_uuid = uuid::Uuid::new_v4();
        let trace_id = trace_uuid.simple().to_string();
        let span_str = span_uuid.simple().to_string();
        // uuid::Uuid::simple().to_string() は常に 32 文字の ASCII hex 文字列を返す。
        // 後半 16 文字（インデックス 16..32）を span_id として使用する。
        let span_id = span_str[16..32].to_string();
        TraceContext { trace_id, span_id, parent_span_id: None }
    }

    /// 子スパンのコンテキストを作成する（`self` を取らない関連付け関数）。
    /// `TraceContext::child_span(&root)` の形で呼び出す。
    /// 親と同じ `trace_id` を引き継ぎ、新しい `span_id` を生成し、
    /// `parent_span_id = Some(parent.span_id.clone())`。
    pub fn child_span(parent: &TraceContext) -> TraceContext {
        let span_uuid = uuid::Uuid::new_v4();
        let span_str = span_uuid.simple().to_string();
        // uuid::Uuid::simple().to_string() は常に 32 文字の ASCII hex 文字列を返す。
        let span_id = span_str[16..32].to_string();
        TraceContext {
            trace_id: parent.trace_id.clone(),
            span_id,
            parent_span_id: Some(parent.span_id.clone()),
        }
    }
}

/// 単一スパンの実行記録。
#[derive(Debug, Clone, PartialEq)]
pub struct TraceSpan {
    pub context: TraceContext,
    pub name: String,
    pub start_ms: u64,
    pub end_ms: u64,
    pub attributes: Vec<(String, String)>,
}

/// スパン情報のテキストを返す。
///
/// ```text
/// Span: load_stage
/// Trace: <trace_id>
/// Span ID: <span_id>
/// Duration: 150ms
/// Attributes: env=prod, region=us-east-1
/// ```
/// `attributes` が空の場合は "Attributes:" 行を省略する。
pub fn format_trace_span(span: &TraceSpan) -> String {
    let duration = compute_span_duration(span);
    let mut lines = vec![
        format!("Span: {}", span.name),
        format!("Trace: {}", span.context.trace_id),
        format!("Span ID: {}", span.context.span_id),
        format!("Duration: {}ms", duration),
    ];
    if !span.attributes.is_empty() {
        let attrs = span
            .attributes
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join(", ");
        lines.push(format!("Attributes: {}", attrs));
    }
    lines.join("\n")
}

/// スパンの実行時間（ミリ秒）を返す。アンダーフローを防ぐため `saturating_sub` を使用。
pub fn compute_span_duration(span: &TraceSpan) -> u64 {
    span.end_ms.saturating_sub(span.start_ms)
}

// ── v83.6.0: パフォーマンス回帰検知 ──────────────────────────────────────────

/// パイプライン実行のパフォーマンスベースライン（p50/p95/p99）。
#[derive(Debug, Clone, PartialEq)]
pub struct PerfBaseline {
    pub pipeline_name: String,
    pub p50_ms: u64,
    pub p95_ms: u64,
    pub p99_ms: u64,
}

impl PerfBaseline {
    /// サンプル一覧からベースラインを算出する。
    ///
    /// サンプルをソートし、インデックスベースの百分位数で p50/p95/p99 を求める:
    /// - `p50_ms = sorted[n * 50 / 100]`
    /// - `p95_ms = sorted[n * 95 / 100]`
    /// - `p99_ms = sorted[n * 99 / 100]`
    ///
    /// `samples_ms` が空の場合は p50/p95/p99 すべて 0 を返す。
    pub fn from_samples(pipeline_name: &str, samples_ms: &[u64]) -> Self {
        if samples_ms.is_empty() {
            return PerfBaseline {
                pipeline_name: pipeline_name.to_string(),
                p50_ms: 0,
                p95_ms: 0,
                p99_ms: 0,
            };
        }
        let mut sorted = samples_ms.to_vec();
        sorted.sort_unstable();
        let n = sorted.len();
        let p50_ms = sorted[n * 50 / 100];
        let p95_ms = sorted[n * 95 / 100];
        let p99_ms = sorted[n * 99 / 100];
        PerfBaseline { pipeline_name: pipeline_name.to_string(), p50_ms, p95_ms, p99_ms }
    }
}

/// 回帰が検知された際の詳細。
#[derive(Debug, Clone, PartialEq)]
pub struct PerfRegression {
    pub pipeline_name: String,
    pub baseline: PerfBaseline,
    pub current_ms: u64,
    /// (current_ms - baseline.p95_ms) / baseline.p95_ms * 100.0
    pub regression_pct: f64,
}

/// `current_ms` を `baseline.p95_ms` と比較し、回帰率が `threshold_pct` を**厳密に超えた**場合に
/// `Some(PerfRegression)` を返す（`regression_pct > threshold_pct`）。等しい場合は `None`。
///
/// `baseline.p95_ms == 0` の場合は回帰なし（`None`）とみなす（ゼロ除算ガード）。
pub fn detect_perf_regression(
    baseline: &PerfBaseline,
    current_ms: u64,
    threshold_pct: f64,
) -> Option<PerfRegression> {
    if baseline.p95_ms == 0 {
        return None;
    }
    let regression_pct = (current_ms as f64 - baseline.p95_ms as f64) / baseline.p95_ms as f64 * 100.0;
    if regression_pct > threshold_pct {
        Some(PerfRegression {
            pipeline_name: baseline.pipeline_name.clone(),
            baseline: baseline.clone(),
            current_ms,
            regression_pct,
        })
    } else {
        None
    }
}

/// 回帰レポートのテキストを返す。
///
/// 例: "PerfRegression: etl_main\nBaseline p95: 200ms\nCurrent: 260ms\nRegression: +30.00%"
pub fn format_regression_report(regression: &PerfRegression) -> String {
    let sign = if regression.regression_pct >= 0.0 { "+" } else { "" };
    format!(
        "PerfRegression: {}\nBaseline p95: {}ms\nCurrent: {}ms\nRegression: {}{:.2}%",
        regression.pipeline_name,
        regression.baseline.p95_ms,
        regression.current_ms,
        sign,
        regression.regression_pct,
    )
}

// ── v83.7.0: fav observe コマンド（メトリクス・アラート統合） ────────────────

/// `fav observe` コマンドの出力フォーマット。
#[derive(Debug, Clone, PartialEq)]
pub enum ObserveFormat {
    Text,
    Json,
}

/// `fav observe` コマンドのオプション。
///
/// `show_alerts` / `show_slo` は CLI レイヤーのフィルタリング意図を示すものであり、
/// `format_observe_report` には影響しない。
///
/// `pipeline_name` は将来の CLI フィルタリング（`--pipeline <name>`）で使用予定。
/// 現在は `format_observe_report` 内で参照せず、`report.metrics.pipeline_name` を使用する。
#[derive(Debug, Clone, PartialEq)]
pub struct ObserveOptions {
    pub pipeline_name: String,
    pub format: ObserveFormat,
    pub show_alerts: bool,
    pub show_slo: bool,
}

/// `fav observe` コマンドの実行レポート。
#[derive(Debug, Clone, PartialEq)]
pub struct ObserveReport {
    pub metrics: PipelineMetrics,
    pub alerts: Vec<AlertFiring>,
    pub slo_statuses: Vec<SloStatus>,
}

/// `ObserveReport` をフォーマットして文字列として返す。
///
/// `ObserveFormat::Text` の場合:
/// - "=== Observe: {pipeline_name} ===" ヘッダ（`report.metrics.pipeline_name` を使用）
/// - メトリクスサマリー（`format_metrics_summary` 利用）
/// - alerts が空でない場合、各 AlertFiring の rule.name と fired_at を出力
/// - slo_statuses が空でない場合、各 SloStatus を `format_slo_status` で出力
/// - alerts が空のとき "Alert:" 行を含まない
/// - slo_statuses が空のとき "SLO:" 行を含まない
///
/// `ObserveFormat::Json` の場合:
/// - `{"pipeline":"<name>","alerts_count":<n>,"slo_count":<n>}` 形式の簡易 JSON
/// - `"pipeline"` キーには `report.metrics.pipeline_name` を使用する
/// - `show_alerts` / `show_slo` フラグは JSON 出力に影響しない（常に全カウントを出力）
pub fn format_observe_report(report: &ObserveReport, format: &ObserveFormat) -> String {
    match format {
        ObserveFormat::Text => {
            let mut lines = vec![
                format!("=== Observe: {} ===", report.metrics.pipeline_name),
                format_metrics_summary(&report.metrics),
            ];
            for alert in &report.alerts {
                lines.push(format!("Alert: {} fired at {}", alert.rule.name, alert.fired_at));
            }
            for slo in &report.slo_statuses {
                lines.push(format_slo_status(slo));
            }
            lines.join("\n")
        }
        ObserveFormat::Json => {
            // pipeline_name を手動エスケープして不正 JSON を防ぐ
            let escaped_name = report.metrics.pipeline_name
                .replace('\\', "\\\\")
                .replace('"', "\\\"");
            format!(
                r#"{{"pipeline":"{}","alerts_count":{},"slo_count":{}}}"#,
                escaped_name,
                report.alerts.len(),
                report.slo_statuses.len(),
            )
        }
    }
}

/// `fav observe` コマンドハンドラ。`format_observe_report(report, &options.format)` を返す。
pub fn cmd_observe(options: &ObserveOptions, report: &ObserveReport) -> String {
    format_observe_report(report, &options.format)
}

// ── v83.8.0: 健全性ダッシュボード ────────────────────────────────────────────

/// パイプラインの総合健全性ステータス。
#[derive(Debug, Clone, PartialEq)]
pub enum HealthStatus {
    Healthy,
    /// 劣化状態。String は理由（例: "1 alert(s) firing"）。
    Degraded(String),
    /// 重大障害。String は理由（例: "SLO breached: etl_slo"）。
    Critical(String),
}

/// 単一パイプラインの健全性レポート。
#[derive(Debug, Clone, PartialEq)]
pub struct PipelineHealth {
    pub pipeline_name: String,
    pub status: HealthStatus,
    pub alerts_firing: usize,
    pub slo_breached: bool,
    pub quality_score: f64,
}

/// 複数パイプラインの健全性ダッシュボード。
#[derive(Debug, Clone, PartialEq)]
pub struct HealthDashboard {
    pub pipelines: Vec<PipelineHealth>,
    pub generated_at: String,
}

/// パイプラインの総合健全性を算出する。
///
/// `pipeline_name` は `metrics.pipeline_name` から取得する。
///
/// ステータス判定ロジック（優先順位順）:
/// - `slo.breached == true` → `Critical("SLO breached: {slo.target.name}")`
/// - `slo.breached == false && alerts.len() > 0` → `Degraded("{n} alert(s) firing")`
/// - `slo.breached == false && alerts.len() == 0` → `Healthy`
///
/// `alerts_firing = alerts.len()`、`slo_breached = slo.breached`、`quality_score = quality`。
pub fn compute_pipeline_health(
    metrics: &PipelineMetrics,
    alerts: &[AlertFiring],
    slo: &SloStatus,
    quality: f64,
) -> PipelineHealth {
    debug_assert!(quality.is_finite() && (0.0..=100.0).contains(&quality), "quality_score must be finite and in 0.0..=100.0");
    let alerts_firing = alerts.len();
    let slo_breached = slo.breached;
    let status = if slo_breached {
        HealthStatus::Critical(format!("SLO breached: {}", slo.target.name))
    } else if alerts_firing > 0 {
        HealthStatus::Degraded(format!("{} alert(s) firing", alerts_firing))
    } else {
        HealthStatus::Healthy
    };
    PipelineHealth {
        pipeline_name: metrics.pipeline_name.clone(),
        status,
        alerts_firing,
        slo_breached,
        quality_score: quality,
    }
}

/// `HealthDashboard` をテキスト形式で返す。
///
/// 出力形式:
/// ```text
/// === Health Dashboard ===
/// Generated: {generated_at}
/// Pipeline: {name}  Status: {status}  Alerts: {n}  SLO: OK|BREACHED  Quality: {:.2}
/// ```
/// `pipelines` が空のとき "No pipelines." を出力する。
pub fn format_health_dashboard(dashboard: &HealthDashboard) -> String {
    let mut lines = vec![
        "=== Health Dashboard ===".to_string(),
        format!("Generated: {}", dashboard.generated_at),
    ];
    if dashboard.pipelines.is_empty() {
        lines.push("No pipelines.".to_string());
    } else {
        for p in &dashboard.pipelines {
            let status_str = match &p.status {
                HealthStatus::Healthy => "Healthy".to_string(),
                HealthStatus::Degraded(reason) => format!("Degraded({})", reason),
                HealthStatus::Critical(reason) => format!("Critical({})", reason),
            };
            let slo_str = if p.slo_breached { "BREACHED" } else { "OK" };
            lines.push(format!(
                "Pipeline: {}  Status: {}  Alerts: {}  SLO: {}  Quality: {:.2}",
                p.pipeline_name, status_str, p.alerts_firing, slo_str, p.quality_score,
            ));
        }
    }
    lines.join("\n")
}
