# Plan: v81.7.0 — `fav quality report` コマンド

## Step 1: 前提確認

- `cargo test` を実行し、3855 tests, 0 failures を確認する
- `fav/src/test_framework.rs` に v81.1.0 の `QualityCheck` / `run_quality_check` / `QualityViolation` が定義済みであることを確認する
  - `QualityViolation` のフィールド: `rule: QualityRule`, `row_index: usize`, `actual: String`
  - `QualityRule` のフィールド: `column: String`, `kind: QualityRuleKind`, `severity: RuleSeverity`

## Step 2: `fav/src/test_framework.rs` に追記

`format_gate_decision` の定義の直後（v81.6.0 セクション末尾）に以下を追加する。

```rust
// ── v81.7.0: QualityReportOptions / build_quality_report ─────────────────────

/// レポート出力フォーマット。
#[derive(Debug, Clone, PartialEq)]
pub enum ReportFormat {
    Text,
    Json,
    Markdown,
}

/// `build_quality_report` のオプション。
#[derive(Debug, Clone)]
pub struct QualityReportOptions {
    pub format: ReportFormat,
    pub include_violations: bool,
    pub include_stats: bool,
}

/// `check` のルールを `rows` に適用し、`opts` の形式でレポートを返す。
///
/// `include_stats` は将来拡張用スタブ。現バージョンでは参照しない。
pub fn build_quality_report(
    check: &QualityCheck,
    rows: &[Vec<String>],
    opts: &QualityReportOptions,
) -> String {
    let violations = run_quality_check(check, rows);
    // TODO: opts.include_stats は将来 DistributionStats 要約追記に使用予定（現在は未使用）
    let n = violations.len();
    match opts.format {
        ReportFormat::Text => {
            let mut out = format!("quality_report format=text violations={n}");
            if opts.include_violations {
                for v in &violations {
                    out.push_str(&format!(
                        "\n- row={} col={} rule={:?}",
                        v.row_index, v.rule.column, v.rule.kind,
                    ));
                }
            }
            out
        }
        ReportFormat::Json => {
            if opts.include_violations && !violations.is_empty() {
                let items: Vec<String> = violations.iter().map(|v| {
                    format!(
                        "{{\"row\":{},\"col\":\"{}\",\"rule\":\"{:?}\"}}",
                        v.row_index, v.rule.column, v.rule.kind,
                    )
                }).collect();
                format!(
                    "{{\"format\":\"json\",\"violations\":{n},\"items\":[{}]}}",
                    items.join(",")
                )
            } else {
                format!("{{\"format\":\"json\",\"violations\":{n}}}")
            }
        }
        ReportFormat::Markdown => {
            let mut out = format!("## Quality Report\nformat: markdown\nviolations: {n}");
            if opts.include_violations {
                for v in &violations {
                    out.push_str(&format!(
                        "\n- row={} col={} rule={:?}",
                        v.row_index, v.rule.column, v.rule.kind,
                    ));
                }
            }
            out
        }
    }
}

/// `build_quality_report` の薄いラッパー（CLI コマンドハンドラ相当）。
pub fn cmd_quality_report(
    check: &QualityCheck,
    rows: &[Vec<String>],
    opts: &QualityReportOptions,
) -> String {
    build_quality_report(check, rows, opts)
}
```

## Step 3: `fav/src/driver.rs` に `mod v81700_tests` を追加

`mod v81600_tests { ... }` の直後に追加する。

```rust
#[cfg(test)]
mod v81700_tests {
    use fav_core::test_framework::*;

    #[test]
    fn quality_report_text_format() {
        let check = QualityCheck { rules: vec![] };
        let rows: Vec<Vec<String>> = vec![];
        let opts = QualityReportOptions {
            format: ReportFormat::Text,
            include_violations: false,
            include_stats: false,
        };
        let out = build_quality_report(&check, &rows, &opts);
        assert!(out.contains("text"),       "should contain format name: {out}");
        assert!(out.contains("violations"), "should contain violations key: {out}");
        assert!(out.contains('0'),          "should show 0 violations: {out}");

        // cmd_quality_report は build_quality_report と同一結果を返す
        let out2 = cmd_quality_report(&check, &rows, &opts);
        assert_eq!(out, out2, "cmd_quality_report should match build_quality_report");
    }

    #[test]
    fn quality_report_json_format() {
        let check = QualityCheck { rules: vec![] };
        let rows: Vec<Vec<String>> = vec![];
        let opts = QualityReportOptions {
            format: ReportFormat::Json,
            include_violations: false,
            include_stats: false,
        };
        let out = build_quality_report(&check, &rows, &opts);
        assert!(out.contains("json"),       "should contain format name: {out}");
        assert!(out.contains("violations"), "should contain violations key: {out}");

        // Markdown フォーマットも smoke test
        let opts_md = QualityReportOptions {
            format: ReportFormat::Markdown,
            include_violations: false,
            include_stats: false,
        };
        let out_md = build_quality_report(&check, &rows, &opts_md);
        assert!(out_md.contains("Quality Report"), "markdown should contain header: {out_md}");
        assert!(out_md.contains("violations"),     "markdown should contain violations: {out_md}");
    }
}
```

## Step 4: CHANGELOG 更新

`CHANGELOG.md` の先頭に v81.7.0 エントリを追加する。

## Step 5: `cargo test` で全 pass 確認

以下は `fav/` ディレクトリで実行する。

```
cargo test 2>&1 | grep "test result"
# 期待: 3857 tests, 0 failures
```

## Step 6: CI 事前確認

以下はすべて `fav/` ディレクトリで実行する。

```
cargo clippy --locked -- -D warnings
./target/debug/fav fmt --check self/compiler.fav
./target/debug/fav fmt --check self/checker.fav
```
