# Plan: v80.5.0 — ステージ単体テスト（`StageTestCase`）

実装依存順（既存モジュール追記 → テスト追加）

> `lib.rs` 変更不要。`driver.rs` はバイナリクレートのため `fav_core::test_framework::*` を使用。
> `#[cfg(test)] mod v80500_tests` パターン（v80.1.0〜v80.4.0 の慣例）。

---

## Step 1: `fav/src/test_framework.rs` に型と実装を追加

`run_property_test_suite` の後ろに以下を追記する。

```rust
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
    pub input: StageInput,
    pub expected: StageOutput,
}

/// `test.expected` と `actual` の rows を行単位で比較する。
/// - 全行一致: `TestCase { status: Pass, message: None }`
/// - 不一致: `TestCase { status: Fail, message: Some("row N differs: ...") }`
/// - 行数不一致は行数が短い方を基準に、超過行も "row N differs" として記録する。
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
            format!("FAIL: {} — {}", result.name, msg)
        }
        TestStatus::Skip => format!("SKIP: {}", result.name),
    }
}
```

---

## Step 2: `fav/src/driver.rs` に `mod v80500_tests` を追加

`mod v80400_tests { ... }` の直後に以下を追加する。

```rust
#[cfg(test)]
mod v80500_tests {
    use fav_core::test_framework::*;

    fn make_stage_test() -> StageTestCase {
        let input = StageInput {
            name: "load".to_string(),
            rows: vec![vec!["alice".to_string(), "30".to_string()]],
        };
        let expected = StageOutput {
            name: "transform".to_string(),
            rows: vec![vec!["alice".to_string(), "30".to_string()]],
        };
        StageTestCase {
            stage_name: "transform".to_string(),
            input,
            expected,
        }
    }

    #[test]
    fn stage_test_pass_when_output_matches() {
        let test = make_stage_test();
        let actual = StageOutput {
            name: "transform".to_string(),
            rows: vec![vec!["alice".to_string(), "30".to_string()]],
        };
        let result = run_stage_test(&test, &actual);
        assert!(matches!(result.status, TestStatus::Pass));
        assert!(result.message.is_none());
        assert_eq!(format_stage_test_result(&result), "PASS: transform");
    }

    #[test]
    fn stage_test_fail_when_output_differs() {
        let test = make_stage_test();
        let actual = StageOutput {
            name: "transform".to_string(),
            rows: vec![vec!["bob".to_string(), "25".to_string()]],
        };
        let result = run_stage_test(&test, &actual);
        assert!(matches!(result.status, TestStatus::Fail));
        assert!(result.message.is_some());
        let msg = result.message.as_deref().unwrap();
        assert!(msg.contains("row 0 differs"), "message should mention row 0: {msg}");
        assert!(msg.contains("alice"), "message should mention expected value: {msg}");
        assert!(msg.contains("bob"), "message should mention actual value: {msg}");
        let formatted = format_stage_test_result(&result);
        assert!(formatted.starts_with("FAIL: transform"));
    }
}
```

---

## Step 3: `cargo test` で全 pass を確認

```bash
cargo test 2>&1 | tail -5
```

3823 tests, 0 failures であることを確認する。
