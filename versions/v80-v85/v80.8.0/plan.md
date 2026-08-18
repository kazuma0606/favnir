# Plan: v80.8.0 — CI 統合レポート（`TestReport` / JUnit XML）

実装依存順（既存モジュール追記 → テスト追加）

> `lib.rs` 変更不要。`driver.rs` はバイナリクレートのため `fav_core::test_framework::*` を使用。
> `#[cfg(test)] mod v80800_tests` パターン（v80.1.0〜v80.7.0 の慣例）。
> `cmd_test` への `--format junit` / `--format summary` オプション追加はスコープ外（v80.9.0 で検討）。

---

## Step 1: `fav/src/test_framework.rs` に型と実装を追加

`schema_diff_is_breaking` の後ろに以下を追記する。

```rust
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
/// - XML エスケープは本バージョンではスコープ外（テストデータに特殊文字を含まない前提）。
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
        report.suite.name, total, result.failed, result.skipped, duration_s,
        cases
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
```

---

## Step 2: `fav/src/driver.rs` に `mod v80800_tests` を追加

`mod v80700_tests { ... }` の直後に以下を追加する。

```rust
#[cfg(test)]
mod v80800_tests {
    use fav_core::test_framework::*;

    fn make_report() -> TestReport {
        TestReport {
            suite: TestSuite {
                name: "pipeline_tests".to_string(),
                cases: vec![
                    TestCase { name: "load".to_string(), status: TestStatus::Pass, message: None },
                    TestCase {
                        name: "fail_case".to_string(),
                        status: TestStatus::Fail,
                        message: Some("expected 1 got 2".to_string()),
                    },
                ],
            },
            duration_ms: 42,
            timestamp: "2026-08-19T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn junit_xml_output_has_testsuite_tag() {
        let report = make_report();
        let xml = format_junit_xml(&report);
        assert!(xml.contains("<testsuite"), "should contain <testsuite: {xml}");
        assert!(xml.contains("<testcase"),  "should contain <testcase: {xml}");
        assert!(xml.contains("<failure"),   "should contain <failure for fail case: {xml}");
        assert!(xml.contains("expected 1 got 2"), "failure message should appear: {xml}");
    }

    #[test]
    fn test_report_summary_shows_pass_count() {
        let report = make_report();
        let summary = format_test_summary(&report);
        assert_eq!(
            summary,
            "pipeline_tests: 1 passed, 1 failed, 0 skipped (42ms) [2026-08-19T00:00:00Z]"
        );
    }
}
```

---

## Step 3: `cargo test` で全 pass を確認

```bash
cargo test 2>&1 | tail -5
```

3834 tests, 0 failures であることを確認する。
（ロードマップ記載は 3825 だが、v80.2.0〜v80.7.0 の code-reviewer 対応で累積 +9 されているため実際の目標は 3834）
