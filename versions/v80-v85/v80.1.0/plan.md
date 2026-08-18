# Plan: v80.1.0 — `TestCase` / `TestSuite` 型基盤

実装依存順（新規モジュール → lib 公開 → テスト追加）

> **前提**: `cmd_test` は `main.rs`・`driver.rs` にすでに実装済みのため変更不要。

---

## Step 1: `fav/src/test_framework.rs` を新規作成

型定義と関数を実装する。

```rust
pub enum TestStatus {
    Pass,
    Fail,
    Skip,
}

pub struct TestCase {
    pub name: String,
    pub status: TestStatus,
    pub message: Option<String>,
}

pub struct TestSuite {
    pub name: String,
    pub cases: Vec<TestCase>,
}

pub struct TestSuiteResult {
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
}

// cases を走査して Pass/Fail/Skip をカウントする
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

// "N passed, M failed, K skipped" 形式の文字列
pub fn format_test_suite_result(result: &TestSuiteResult) -> String {
    format!("{} passed, {} failed, {} skipped",
        result.passed, result.failed, result.skipped)
}
```

---

## Step 2: `fav/src/lib.rs` に `pub mod test_framework;` を追加

既存の `pub mod` 宣言群の末尾（または適切な位置）に追記する。

---

## Step 3: `fav/src/driver.rs` に `mod v80100_tests` を追加

`mod v80000_tests { ... }` の直後に以下を追加する。
既存テストのパターン（`use crate::test_framework::*;`）を使用する。

```rust
mod v80100_tests {
    use fav_core::test_framework::*;

    #[test]
    fn test_suite_type_exists() {
        let suite = TestSuite {
            name: "my_suite".to_string(),
            cases: vec![
                TestCase { name: "t1".to_string(), status: TestStatus::Pass, message: None },
                TestCase { name: "t2".to_string(), status: TestStatus::Fail, message: Some("err".to_string()) },
                TestCase { name: "t3".to_string(), status: TestStatus::Skip, message: None },
            ],
        };
        assert_eq!(suite.name, "my_suite");
        assert_eq!(suite.cases.len(), 3);
    }

    #[test]
    fn test_case_run_formats_result() {
        let suite = TestSuite {
            name: "suite".to_string(),
            cases: vec![
                TestCase { name: "a".to_string(), status: TestStatus::Pass, message: None },
                TestCase { name: "b".to_string(), status: TestStatus::Fail, message: None },
            ],
        };
        let result = run_test_suite(&suite);
        assert_eq!(result.passed, 1);
        assert_eq!(result.failed, 1);
        assert_eq!(result.skipped, 0);
        let formatted = format_test_suite_result(&result);
        assert!(formatted.contains("passed"));
        assert!(formatted.contains("failed"));
        assert!(formatted.contains("skipped"));
    }
}
```

---

## Step 4: `cargo test` で全 pass を確認

```bash
cd fav && cargo test 2>&1 | tail -5
```

3811 tests, 0 failures になっていることを確認する。
