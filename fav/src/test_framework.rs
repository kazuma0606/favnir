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
    /// NaN/Inf を含まないこと。
    NonNull,
}

/// プロパティベーステストの定義。
#[derive(Debug)]
pub struct PropertyTest {
    pub name: String,
    pub kind: InvariantKind,
    pub samples: usize,
}

/// プロパティベーステストの実行結果。
#[derive(Debug)]
pub struct PropertyTestResult {
    pub passed: bool,
    /// 不変条件に違反した最初の値（違反がなければ None）。
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
        InvariantKind::NonNull => data.iter().find(|&&v| v.is_nan() || v.is_infinite()).copied(),
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
