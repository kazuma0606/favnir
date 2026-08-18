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

/// Load a CSV file as a GoldenDataset (one row per line, comma-separated).
/// Empty lines are skipped. Not available on WASM targets.
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
