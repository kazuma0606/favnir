# Spec: v80.6.0 — テストカバレッジレポート（`TestCoverageReport`）

## Background

v80.1.0〜v80.5.0 でテストフレームワーク基盤（TestSuite / GoldenDataset / DataFactory / PropertyTest / StageTestCase）を構築した。
本バージョンでは「どのステージがテストされているか」を集計する **カバレッジレポート型** を追加する。
`TestSuite` のケース名と既知ステージ一覧を突き合わせ、テスト済み / 未テストのエントリを生成する。

ロードマップ: `versions/roadmap/roadmap-v80.1-v81.0.md`（v80.6.0 セクション）

> **テスト数補足**: ロードマップは 3819 + 2 = 3821 と記載しているが、
> v80.2.0〜v80.5.0 の code-reviewer 対応で累積 6 件追加されたため実際のベースは **3825**。
> 本バージョンの完了条件は **3825 + 2 = 3827**。

## Goals

- `CoverageEntry` 構造体を `test_framework.rs` に追加する
- `TestCoverageReport` 構造体を追加する
- `compute_test_coverage(suite: &TestSuite, known_stages: &[String]) -> TestCoverageReport` を実装する
- `format_coverage_report(report: &TestCoverageReport) -> String` を実装する
- `coverage_pct(report: &TestCoverageReport) -> f64` を実装する
- テスト 2 件を追加して **3827 tests** を達成する

## API / Type Definitions

```rust
// fav/src/test_framework.rs（既存ファイルに追記）

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
/// - 各 `known_stage` について、`suite.cases` に同名の `TestCase` が 1 件以上あれば `tested: true`。
/// - `total = known_stages.len()`、`covered = tested が true のエントリ数`。
/// - `known_stages` が空の場合は `total = 0`、`covered = 0` の空レポートを返す。
pub fn compute_test_coverage(suite: &TestSuite, known_stages: &[String]) -> TestCoverageReport;

/// カバレッジレポートを "coverage: X/Y (Z.Zpct)" 形式の文字列に変換する。
/// - Z.Z は小数点以下 1 桁（例: "coverage: 2/3 (66.7pct)"）。
/// - total が 0 の場合は "coverage: 0/0 (0.0pct)" を返す。
pub fn format_coverage_report(report: &TestCoverageReport) -> String;

/// カバレッジ率を 0.0〜100.0 の f64 で返す。
/// - total が 0 の場合は 0.0 を返す（ゼロ除算ガード）。
pub fn coverage_pct(report: &TestCoverageReport) -> f64;
```

### `compute_test_coverage` の動作例

```rust
let suite = TestSuite {
    name: "pipeline".to_string(),
    cases: vec![
        TestCase { name: "load".to_string(), status: TestStatus::Pass, message: None },
        TestCase { name: "transform".to_string(), status: TestStatus::Pass, message: None },
    ],
};
let known = vec!["load".to_string(), "transform".to_string(), "export".to_string()];
let report = compute_test_coverage(&suite, &known);
// report.total   == 3
// report.covered == 2
// report.entries == [
//   CoverageEntry { name: "load",      tested: true },
//   CoverageEntry { name: "transform", tested: true },
//   CoverageEntry { name: "export",    tested: false },
// ]
// coverage_pct(&report) == 66.66...
// format_coverage_report(&report) == "coverage: 2/3 (66.7pct)"
```

### `coverage_pct` が 0 を返す例

```rust
let empty_suite = TestSuite { name: "empty".to_string(), cases: vec![] };
let known = vec!["stage_a".to_string()];
let report = compute_test_coverage(&empty_suite, &known);
// report.covered == 0, report.total == 1
// coverage_pct(&report) == 0.0
// format_coverage_report(&report) == "coverage: 0/1 (0.0pct)"

// known_stages が空の場合
let report2 = compute_test_coverage(&empty_suite, &[]);
// coverage_pct(&report2) == 0.0  ← ゼロ除算ガード
// format_coverage_report(&report2) == "coverage: 0/0 (0.0pct)"
```

## Success Criteria

- `cargo test` が **3827 tests**, 0 failures
- `coverage_report_counts_correctly`:
  - suite に 2 ケース（load / transform）、known_stages に 3 件（load / transform / export）
  - `report.total == 3`、`report.covered == 2`
  - `coverage_pct(&report)` が 66.0 以上 67.0 未満であること
  - `format_coverage_report(&report)` が `"coverage: 2/3 (66.7pct)"` であること
- `coverage_pct_is_zero_when_nothing_tested`:
  - suite に 0 ケース、known_stages に 1 件
  - `coverage_pct(&report) == 0.0`
  - `format_coverage_report(&report)` が `"coverage: 0/1 (0.0pct)"` であること

## Files to Modify

| ファイル | 操作 | 内容 |
|---|---|---|
| `fav/src/test_framework.rs` | 追記 | `CoverageEntry` / `TestCoverageReport` / `compute_test_coverage` / `format_coverage_report` / `coverage_pct` |
| `fav/src/driver.rs` | 追記 | `mod v80600_tests`（テスト 2 件） |

> `lib.rs` への変更は不要（`pub mod test_framework;` は v80.1.0 で宣言済み）。

## Error Codes

新規エラーコードなし。

## 注記

- `compute_test_coverage` はステージ名の一致を文字列比較（`==`）で行う。大文字小文字を区別する。
- `TestCase.status`（Pass/Fail/Skip）は問わず、**名前が一致すれば `tested: true`** とする。
- `format_coverage_report` の小数点以下 1 桁は `format!("{:.1}", ...)` で実現する。
- ロードマップタイトルは「`fav test --coverage`」だが、`cmd_test` への `--coverage` フラグ接続は本バージョンのスコープ外とする。型基盤（`TestCoverageReport` / `compute_test_coverage` 等）のみを追加し、CLI への統合は v80.9.0 安定化フェーズで実施する。
- MILESTONE.md / README.md / `site/content/docs/` の更新は v81.0.0 宣言バージョンでまとめて実施する。
