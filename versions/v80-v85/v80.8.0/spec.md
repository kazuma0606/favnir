# Spec: v80.8.0 — CI 統合レポート（`TestReport` / JUnit XML）

## Background

v80.1.0〜v80.7.0 でテストフレームワーク基盤を構築した。
本バージョンでは CI パイプラインに組み込める形式でテスト結果を出力する **`TestReport` 型** と
2 つのフォーマット関数を追加する。
`format_junit_xml` は JUnit XML 形式、`format_test_summary` は人間向けサマリー形式を出力する。

ロードマップ: `versions/roadmap/roadmap-v80.1-v81.0.md`（v80.8.0 セクション）

> **テスト数補足**: ロードマップは 3823 + 2 = 3825 と記載しているが、
> v80.2.0〜v80.7.0 の code-reviewer 対応で累積 9 件追加されたため実際のベースは **3832**。
> （計算式: 3823〔ロードマップ想定ベース〕 + 9〔code-reviewer 累積〕 = 3832）
> 本バージョンの完了条件は **3832 + 2 = 3834**。

> **スコープ補足**: ロードマップは `cmd_test` への `--format` オプション追加も記載しているが、
> CLI 統合は本バージョンのスコープ外とする。型・フォーマット関数のみを追加し、
> `cmd_test` への統合は v80.9.0 安定化フェーズで検討する。

## Goals

- `TestReport` 構造体を `test_framework.rs` に追加する
- `format_junit_xml(report: &TestReport) -> String` を実装する
- `format_test_summary(report: &TestReport) -> String` を実装する
- テスト 2 件を追加して **3834 tests** を達成する

## API / Type Definitions

```rust
// fav/src/test_framework.rs（既存ファイルに追記）

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
/// 出力形式:
/// ```xml
/// <?xml version="1.0" encoding="UTF-8"?>
/// <testsuite name="{suite.name}" tests="{total}" failures="{failed}" skipped="{skipped}" time="{duration_s:.3}">
///   <testcase name="{case.name}" classname="{suite.name}"/>
///   <testcase name="{case.name}" classname="{suite.name}">
///     <failure message="{message}"/>
///   </testcase>
///   ...
/// </testsuite>
/// ```
/// - `time` は `duration_ms / 1000.0` を小数点以下 3 桁で出力する。
/// - Pass / Skip ケースは `<testcase ... />` のみ。Fail ケースは `<failure>` 子要素を持つ。
/// - message が None の Fail ケースは `<failure message=""/>` とする。
pub fn format_junit_xml(report: &TestReport) -> String;

/// 人間向けサマリー形式のレポートを生成する。
///
/// 出力形式: "{suite.name}: N passed, M failed, K skipped ({duration_ms}ms) [{timestamp}]"
pub fn format_test_summary(report: &TestReport) -> String;
```

### 出力例

```rust
let report = TestReport {
    suite: TestSuite {
        name: "pipeline_tests".to_string(),
        cases: vec![
            TestCase { name: "load".to_string(), status: TestStatus::Pass, message: None },
            TestCase { name: "fail_case".to_string(), status: TestStatus::Fail,
                       message: Some("expected 1 got 2".to_string()) },
        ],
    },
    duration_ms: 42,
    timestamp: "2026-08-19T00:00:00Z".to_string(),
};

// format_test_summary(&report):
// "pipeline_tests: 1 passed, 1 failed, 0 skipped (42ms) [2026-08-19T00:00:00Z]"

// format_junit_xml(&report) の出力（末尾改行なし）:
// <?xml version="1.0" encoding="UTF-8"?>
// <testsuite name="pipeline_tests" tests="2" failures="1" skipped="0" time="0.042">
//   <testcase name="load" classname="pipeline_tests"/>
//   <testcase name="fail_case" classname="pipeline_tests">
//     <failure message="expected 1 got 2"/>
//   </testcase>
// </testsuite>
// ↑ 最後の </testsuite> の後に改行は付かない（テストは contains チェックのため完全一致不要）
```

## Success Criteria

- `cargo test` が **3834 tests**, 0 failures
- `junit_xml_output_has_testsuite_tag`:
  - TestReport を生成し `format_junit_xml` を呼び出す
  - 出力に `"<testsuite"` が含まれることを確認する
  - 出力に `"<testcase"` が含まれることを確認する
  - Fail ケースの出力に `"<failure"` が含まれることを確認する
- `test_report_summary_shows_pass_count`:
  - TestReport を生成し `format_test_summary` を呼び出す
  - 出力が `"pipeline_tests: 1 passed, 1 failed, 0 skipped (42ms) [2026-08-19T00:00:00Z]"` と一致する

## Files to Modify

| ファイル | 操作 | 内容 |
|---|---|---|
| `fav/src/test_framework.rs` | 追記 | `TestReport` / `format_junit_xml` / `format_test_summary` |
| `fav/src/driver.rs` | 追記 | `mod v80800_tests`（テスト 2 件） |

> `lib.rs` への変更は不要（`pub mod test_framework;` は v80.1.0 で宣言済み）。

## Error Codes

新規エラーコードなし。

## 注記

- `format_junit_xml` の XML エスケープ（`&` → `&amp;` 等）は本バージョンではスコープ外とする。テストデータにエスケープ必要文字を含まない前提で実装する。
- `cmd_test` への `--format junit` / `--format summary` オプション追加は本バージョンのスコープ外とする（v80.9.0 安定化フェーズで検討）。
- MILESTONE.md / README.md / `site/content/docs/` の更新は v81.0.0 宣言バージョンでまとめて実施する。
