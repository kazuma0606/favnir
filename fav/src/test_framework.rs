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
