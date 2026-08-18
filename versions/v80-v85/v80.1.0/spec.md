# Spec: v80.1.0 — `TestCase` / `TestSuite` 型基盤

## Background

v80.0.0「Favnir 3.0 宣言」完了後、Quality-First Era の第 1 スプリント（Test-Driven Data 1.0）を開始する。
本バージョンでは、データパイプラインの正しさを型付きテストで証明するためのフレームワーク基盤を構築する。

マスターロードマップ: `versions/roadmap/roadmap-v80.1-v85.0.md`
詳細ロードマップ: `versions/roadmap/roadmap-v80.1-v81.0.md`

> **注**: ロードマップ（`roadmap-v80.1-v81.0.md` 57行目）は `cmd_test(args: &[String]) -> i32` スタブを本バージョンで追加すると記載しているが、`cmd_test` は `fav/src/main.rs`（Some("test") アーム, l.1444）および `fav/src/driver.rs`（l.5824）にすでに完全実装済みである。そのため本バージョンでは **`cmd_test` を変更せず**、`test_framework.rs` モジュールの**新規追加のみ**を行う。

## Goals

- `TestStatus` / `TestCase` / `TestSuite` / `TestSuiteResult` を Rust 型として定義する
- `run_test_suite` / `format_test_suite_result` 関数を実装する
- テスト 2 件を `driver.rs` に追加して 3809 + 2 = 3811 tests を達成する

## API / Type Definitions

```rust
// fav/src/test_framework.rs

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

/// cases を走査し、Pass/Fail/Skip をカウントして返す。
pub fn run_test_suite(suite: &TestSuite) -> TestSuiteResult;

/// "N passed, M failed, K skipped" 形式の文字列を返す。
/// ※ スイート名は TestSuiteResult に含まれないため出力に含めない。
pub fn format_test_suite_result(result: &TestSuiteResult) -> String;
```

### `format_test_suite_result` の出力例

```
3 passed, 1 failed, 0 skipped
```

## Success Criteria

- `cargo test` が **3811 tests**, 0 failures
- `test_suite_type_exists`: `TestSuite` / `TestCase` / `TestStatus` / `TestSuiteResult` を構築し、フィールド値が期待通りであることを確認
- `test_case_run_formats_result`: `run_test_suite` + `format_test_suite_result` の結果文字列が "passed" / "failed" / "skipped" を含むことを確認

## Files to Modify

| ファイル | 操作 | 内容 |
|---|---|---|
| `fav/src/test_framework.rs` | 新規作成 | 型定義・関数実装 |
| `fav/src/lib.rs` | 追記 | `pub mod test_framework;` |
| `fav/src/driver.rs` | 追記 | テスト 2 件（`mod v80100_tests`） |

## `TestStatus` Exhaustive Match について

`TestStatus` の exhaustive match 箇所は現時点で `test_framework.rs` 内の `run_test_suite` のみ。
新 variant を追加する場合は同関数の更新が必要になる。

## Error Codes

新規エラーコードなし。
