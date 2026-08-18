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
