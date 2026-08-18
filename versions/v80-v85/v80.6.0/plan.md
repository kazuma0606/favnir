# Plan: v80.6.0 — テストカバレッジレポート（`TestCoverageReport`）

実装依存順（既存モジュール追記 → テスト追加）

> `lib.rs` 変更不要。`driver.rs` はバイナリクレートのため `fav_core::test_framework::*` を使用。
> `#[cfg(test)] mod v80600_tests` パターン（v80.1.0〜v80.5.0 の慣例）。

---

## Step 1: `fav/src/test_framework.rs` に型と実装を追加

`format_stage_test_result` の後ろに以下を追記する。

```rust
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

/// カバレッジ率を 0.0〜100.0 の f64 で返す。total が 0 の場合は 0.0。
pub fn coverage_pct(report: &TestCoverageReport) -> f64 {
    if report.total == 0 {
        0.0
    } else {
        report.covered as f64 / report.total as f64 * 100.0
    }
}
```

---

## Step 2: `fav/src/driver.rs` に `mod v80600_tests` を追加

`mod v80500_tests { ... }` の直後に以下を追加する。

```rust
#[cfg(test)]
mod v80600_tests {
    use fav_core::test_framework::*;

    #[test]
    fn coverage_report_counts_correctly() {
        let suite = TestSuite {
            name: "pipeline".to_string(),
            cases: vec![
                TestCase { name: "load".to_string(),      status: TestStatus::Pass, message: None },
                TestCase { name: "transform".to_string(), status: TestStatus::Pass, message: None },
            ],
        };
        let known = vec![
            "load".to_string(),
            "transform".to_string(),
            "export".to_string(),
        ];
        let report = compute_test_coverage(&suite, &known);
        assert_eq!(report.total,   3);
        assert_eq!(report.covered, 2);
        assert_eq!(report.entries[0].tested, true);
        assert_eq!(report.entries[1].tested, true);
        assert_eq!(report.entries[2].tested, false);
        let pct = coverage_pct(&report);
        assert!(pct > 66.0 && pct < 67.0, "expected ~66.7, got {pct}");
        assert_eq!(format_coverage_report(&report), "coverage: 2/3 (66.7pct)");
    }

    #[test]
    fn coverage_pct_is_zero_when_nothing_tested() {
        let suite = TestSuite { name: "empty".to_string(), cases: vec![] };
        let known = vec!["stage_a".to_string()];
        let report = compute_test_coverage(&suite, &known);
        assert_eq!(report.total,   1);
        assert_eq!(report.covered, 0);
        assert_eq!(coverage_pct(&report), 0.0);
        assert_eq!(format_coverage_report(&report), "coverage: 0/1 (0.0pct)");
    }
}
```

---

## Step 3: `cargo test` で全 pass を確認

```bash
cargo test 2>&1 | tail -5
```

3827 tests, 0 failures であることを確認する。
（ロードマップ記載は 3821 だが、v80.2.0〜v80.5.0 の code-reviewer 対応で累積 +6 されているため実際の目標は 3827）
