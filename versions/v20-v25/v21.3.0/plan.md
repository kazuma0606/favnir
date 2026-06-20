# v21.3.0 実装計画 — テストカバレッジ HTML / LCOV 出力

## 実装順序

```
T1（coverage/mod.rs — HTML / LCOV / summary ロジック）  ← 最初（T6 テストが依存）
T2（driver.rs — cmd_test 更新）                         ← T1 完了後
T3（main.rs — CLI 引数パース更新）                      ← T2 完了後
T4（Cargo.toml バージョン更新）                         ← T1 と並列可
T5（CHANGELOG + coverage.mdx）                          ← T3 完了後
T6（driver.rs — v213000_tests）                         ← T1 完了後
```

**Rust コードへの変更は T1・T2 と T6。**

---

## T1: `fav/src/coverage/mod.rs` — 新規モジュール作成

### 事前確認

```bash
# coverage モジュールが未存在なことを確認
ls fav/src/coverage/ 2>/dev/null || echo "not exists"

# lib.rs / main.rs への mod 追加箇所を確認
grep -n "mod incremental\|mod pushdown" fav/src/lib.rs | head -5
grep -n "mod incremental\|mod pushdown" fav/src/main.rs | head -5
```

### 1-1. `CoverageFileStat` と `CoverageSummary`

```rust
/// ファイル単位のカバレッジ統計。
#[derive(Debug, Clone)]
pub struct CoverageFileStat {
    pub path: String,
    pub covered: usize,
    pub total: usize,
    pub fn_covered: usize,
    pub fn_total: usize,
    /// 行番号 → カバー済み（true） / 未カバー（false）
    pub line_hits: Vec<(u32, bool)>,
    /// 関数名 → カバー済みか
    pub fn_hits: Vec<(String, bool)>,
}

impl CoverageFileStat {
    pub fn pct(&self) -> f64 {
        if self.total == 0 { 100.0 } else { self.covered as f64 / self.total as f64 * 100.0 }
    }
}

/// プロジェクト全体の集計。
#[derive(Debug, Default)]
pub struct CoverageSummary {
    pub files: Vec<CoverageFileStat>,
}

impl CoverageSummary {
    pub fn total_covered(&self) -> usize { self.files.iter().map(|f| f.covered).sum() }
    pub fn total_lines(&self) -> usize   { self.files.iter().map(|f| f.total).sum() }
    pub fn overall_pct(&self) -> f64 {
        let t = self.total_lines();
        if t == 0 { 100.0 } else { self.total_covered() as f64 / t as f64 * 100.0 }
    }
}
```

### 1-2. `format_coverage_summary_console`

```rust
/// コンソール出力: "Coverage: XX.X% (N/M lines)" + ファイル別 ✓/✗ 行。
/// threshold 未満のファイルを ✗ で表示する。
pub fn format_coverage_summary_console(summary: &CoverageSummary, threshold: f64) -> String {
    let mut out = String::new();
    let total_covered = summary.total_covered();
    let total_lines   = summary.total_lines();
    let overall_pct   = summary.overall_pct();
    out.push_str(&format!(
        "Coverage: {:.1}% ({}/{} lines)\n",
        overall_pct, total_covered, total_lines
    ));
    for f in &summary.files {
        let pct = f.pct();
        let mark = if pct >= threshold { "✓" } else { "✗" };
        let note = if pct < threshold { "  ← 要改善" } else { "" };
        out.push_str(&format!(
            "  {} {:<40} {:.1}%  ({}/{}){}\n",
            mark, f.path, pct, f.covered, f.total, note
        ));
    }
    out
}
```

### 1-3. `generate_coverage_html`

```rust
/// カバレッジ HTML レポート（index.html）を生成する。外部依存なし。
pub fn generate_coverage_html(summary: &CoverageSummary, source_map: &HashMap<String, String>) -> String {
    let mut html = String::new();
    html.push_str(HTML_HEADER);  // <html><head><style>...</style></head><body>

    // サマリーヘッダー
    html.push_str(&format!(
        "<h1>Favnir Coverage Report</h1>\
         <p class=\"summary\">Coverage: <strong>{:.1}%</strong> ({}/{} lines)</p>",
        summary.overall_pct(), summary.total_covered(), summary.total_lines()
    ));

    // ファイル一覧テーブル
    html.push_str("<table><tr><th>File</th><th>Line %</th><th>Lines</th><th>Fn %</th></tr>");
    for f in &summary.files {
        let cls = if f.pct() >= 80.0 { "ok" } else { "warn" };
        html.push_str(&format!(
            "<tr class=\"{}\"><td>{}</td><td>{:.1}%</td><td>{}/{}</td><td>{}/{}</td></tr>",
            cls, f.path, f.pct(), f.covered, f.total, f.fn_covered, f.fn_total
        ));
    }
    html.push_str("</table>");

    // ファイル詳細（行ハイライト）
    for f in &summary.files {
        if let Some(source) = source_map.get(&f.path) {
            html.push_str(&format!("<h2>{}</h2><pre class=\"source\">", f.path));
            let hit_map: HashMap<u32, bool> = f.line_hits.iter().copied().collect();
            for (idx, line) in source.lines().enumerate() {
                let lineno = (idx + 1) as u32;
                let cls = match hit_map.get(&lineno) {
                    Some(true)  => "covered",
                    Some(false) => "uncovered",
                    None        => "nonexec",
                };
                let escaped = line.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;");
                html.push_str(&format!(
                    "<span class=\"{}\" id=\"L{}\">{:>4} | {}</span>\n",
                    cls, lineno, lineno, escaped
                ));
            }
            html.push_str("</pre>");
        }
    }

    html.push_str(HTML_FOOTER);
    html
}

const HTML_HEADER: &str = r#"<!DOCTYPE html>
<html lang="ja"><head><meta charset="utf-8">
<title>Favnir Coverage</title>
<style>
body { font-family: monospace; margin: 2em; }
table { border-collapse: collapse; width: 100%; }
th, td { border: 1px solid #ccc; padding: 4px 8px; }
.ok { background: #e8f5e9; }
.warn { background: #fff3e0; }
.source { background: #f8f8f8; padding: 1em; overflow-x: auto; }
.covered   { display:block; background: #c8e6c9; }
.uncovered { display:block; background: #ffcdd2; }
.nonexec   { display:block; color: #999; }
.summary { font-size: 1.2em; }
</style></head><body>"#;

const HTML_FOOTER: &str = "</body></html>";
```

### 1-4. `generate_lcov`

```rust
/// LCOV 形式（lcov.info）を生成する。
pub fn generate_lcov(summary: &CoverageSummary) -> String {
    let mut out = String::new();
    for f in &summary.files {
        out.push_str("TN:\n");
        out.push_str(&format!("SF:{}\n", f.path));
        // 関数定義（行番号は 1-indexed の連番。実際の定義行は CoverageFileStat 拡張時に対応予定）
        for (i, (name, _)) in f.fn_hits.iter().enumerate() {
            out.push_str(&format!("FN:{},{}\n", i + 1, name));
        }
        // 関数実行回数
        for (name, hit) in &f.fn_hits {
            out.push_str(&format!("FNDA:{},{}\n", if *hit { 1 } else { 0 }, name));
        }
        out.push_str(&format!("FNF:{}\n", f.fn_total));
        out.push_str(&format!("FNH:{}\n", f.fn_covered));
        // 行実行回数
        for (lineno, hit) in &f.line_hits {
            out.push_str(&format!("DA:{},{}\n", lineno, if *hit { 1 } else { 0 }));
        }
        out.push_str(&format!("LF:{}\n", f.total));
        out.push_str(&format!("LH:{}\n", f.covered));
        out.push_str("end_of_record\n");
    }
    out
}
```

### 1-5. `lib.rs` / `main.rs` に `mod coverage;` 追加

```rust
// lib.rs / main.rs 両方に追加
#[cfg(not(target_arch = "wasm32"))]
pub mod coverage;
```

---

## T2: `fav/src/driver.rs` — `cmd_test` 更新

### 事前確認

```bash
grep -n "coverage_report_dir\|format_coverage_report\|full_report" fav/src/driver.rs | head -10
```

### 2-1. coverage 関数を `use` でインポート

`driver.rs` 内で直接 `use` する（`pub use` は不要、テストも同クレート内でアクセス可能）。

```rust
#[cfg(not(target_arch = "wasm32"))]
use crate::coverage::{
    CoverageFileStat, CoverageSummary,
    format_coverage_summary_console, generate_coverage_html, generate_lcov,
};
```

**注意**: `lib.rs` に `pub mod coverage;` を宣言すれば外部クレートからも `fav_core::coverage::*` でアクセスできるため、`driver.rs` での `pub use` は不要。

### 2-2. `cmd_test` シグネチャ更新

```rust
pub fn cmd_test(
    file: Option<&str>,
    filter: Option<&str>,
    fail_fast: bool,
    no_capture: bool,
    coverage: bool,
    coverage_report_dir: Option<&str>,
    update_snapshots: bool,
    coverage_html: bool,   // ← 追加
    coverage_lcov: bool,   // ← 追加
)
```

### 2-3. coverage 出力部分の更新

既存の `if coverage { ... }` ブロックを以下のように置き換える:

```rust
if coverage {
    let threshold = 80.0f64;
    let mut summary = CoverageSummary::default();

    let source_paths: Vec<String> = tests_to_run
        .iter()
        .map(|(path, _, _, _)| path.clone())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();

    let mut source_map: HashMap<String, String> = HashMap::new();

    for path in &source_paths {
        if let Ok(source) = std::fs::read_to_string(path) {
            // 既存のテキストレポートも生成（後方互換）
            let text_report = format_coverage_report(path, &source, &all_covered);
            println!("{}", text_report);

            // CoverageFileStat を構築
            // is_executable_line は pub(crate) に昇格済み（T1 で対応）
            let total_lines = source.lines().count() as u32;
            let executable: Vec<u32> = (1..=total_lines)
                .filter(|l| is_executable_line(&source, *l))
                .collect();
            let line_hits: Vec<(u32, bool)> = executable.iter()
                .map(|&l| (l, all_covered.contains(&l)))
                .collect();
            let covered_count = line_hits.iter().filter(|(_, hit)| *hit).count();
            let stat = CoverageFileStat {
                path: path.clone(),
                covered: covered_count,
                total: executable.len(),
                fn_covered: 0,
                fn_total: 0,
                line_hits,
                fn_hits: vec![],
            };
            summary.files.push(stat);
            source_map.insert(path.clone(), source);
        }
    }

    // コンソールサマリー（新形式）
    println!("{}", format_coverage_summary_console(&summary, threshold));

    // coverage-report ディレクトリへの書き込み
    if let Some(dir) = coverage_report_dir {
        std::fs::create_dir_all(dir).ok();

        // coverage.txt（後方互換）
        // ...

        // HTML
        if coverage_html {
            let html = generate_coverage_html(&summary, &source_map);
            let html_path = std::path::Path::new(dir).join("index.html");
            if let Err(e) = std::fs::write(&html_path, &html) {
                eprintln!("warning: could not write HTML report: {e}");
            } else {
                println!("HTML report: {}", html_path.display());
            }
        }

        // LCOV
        if coverage_lcov {
            let lcov = generate_lcov(&summary);
            let lcov_path = std::path::Path::new(dir).join("lcov.info");
            if let Err(e) = std::fs::write(&lcov_path, &lcov) {
                eprintln!("warning: could not write LCOV report: {e}");
            } else {
                println!("LCOV report: {}", lcov_path.display());
            }
        }
    }
}
```

---

## T3: `fav/src/main.rs` — CLI 引数パース更新

### 変更内容

`fav test` のヘルプ文言と引数パースに `--html` / `--lcov` を追加:

```
test [--filter <pattern>] [--fail-fast] [--no-capture] [--coverage] [--html] [--lcov]
     [--coverage-report <dir>] [file]
```

```rust
"--html" => {
    coverage_html = true;
    i += 1;
}
"--lcov" => {
    coverage_lcov = true;
    i += 1;
}
```

`cmd_test` 呼び出しに `coverage_html` / `coverage_lcov` を追加。

---

## T4: `fav/Cargo.toml` バージョン更新

`version = "21.2.0"` → `"21.3.0"`

`v212000_tests::version_is_21_2_0` に `#[ignore]` を追加:
```bash
grep -n "mod v212000_tests\|version_is_21_2_0" fav/src/driver.rs
```

---

## T5: `CHANGELOG.md` + `site/content/docs/tools/coverage.mdx`

### CHANGELOG エントリ

```markdown
## [v21.3.0] — 2026-06-20 — テストカバレッジ HTML / LCOV 出力

### Added
- `fav test --coverage --html` — HTML カバレッジレポート（coverage/index.html）生成
- `fav test --coverage --lcov` — LCOV 形式（coverage/lcov.info）出力
- コンソールサマリーをファイル別 ✓/✗ 形式に改善
- `CoverageFileStat` / `CoverageSummary` 型（coverage モジュール）
- `format_coverage_summary_console` / `generate_coverage_html` / `generate_lcov`
- `site/content/docs/tools/coverage.mdx`
```

---

## T6: `fav/src/driver.rs` — `v213000_tests` 追加

```rust
// ── v213000_tests (v21.3.0) — テストカバレッジ HTML / LCOV ──────────────────
#[cfg(test)]
#[cfg(not(target_arch = "wasm32"))]
mod v213000_tests {
    #[test]
    fn version_is_21_3_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("21.3.0"), "Cargo.toml should have version 21.3.0");
    }

    #[test]
    fn coverage_html_contains_structure() {
        use crate::coverage::{CoverageFileStat, CoverageSummary, generate_coverage_html};
        use std::collections::HashMap;
        let stat = CoverageFileStat {
            path: "src/pipeline.fav".to_string(),
            covered: 8, total: 10,
            fn_covered: 1, fn_total: 1,
            line_hits: vec![(1, true), (2, true), (3, false)],
            fn_hits: vec![("LoadCsv".to_string(), true)],
        };
        let summary = CoverageSummary { files: vec![stat] };
        let mut src_map = HashMap::new();
        src_map.insert("src/pipeline.fav".to_string(), "stage A = |x| { x }\n".to_string());
        let html = generate_coverage_html(&summary, &src_map);
        assert!(html.contains("<!DOCTYPE html>"), "should have DOCTYPE");
        assert!(html.contains("<table>"), "should have table");
        assert!(html.contains("80.0%"), "should show coverage pct");
        assert!(html.contains("8/10"), "should show covered/total lines");
    }

    #[test]
    fn coverage_html_highlights_covered_lines() {
        use crate::coverage::{CoverageFileStat, CoverageSummary, generate_coverage_html};
        use std::collections::HashMap;
        let stat = CoverageFileStat {
            path: "src/test.fav".to_string(),
            covered: 1, total: 2,
            fn_covered: 0, fn_total: 0,
            line_hits: vec![(1, true), (2, false)],
            fn_hits: vec![],
        };
        let summary = CoverageSummary { files: vec![stat] };
        let mut src_map = HashMap::new();
        src_map.insert("src/test.fav".to_string(), "line1\nline2\n".to_string());
        let html = generate_coverage_html(&summary, &src_map);
        assert!(html.contains("class=\"covered\""), "should have covered class");
        assert!(html.contains("class=\"uncovered\""), "should have uncovered class");
    }

    #[test]
    fn coverage_lcov_format() {
        use crate::coverage::{CoverageFileStat, CoverageSummary, generate_lcov};
        let stat = CoverageFileStat {
            path: "src/pipeline.fav".to_string(),
            covered: 2, total: 3,
            fn_covered: 1, fn_total: 1,
            line_hits: vec![(10, true), (11, true), (12, false)],
            fn_hits: vec![("LoadCsv".to_string(), true)],
        };
        let summary = CoverageSummary { files: vec![stat] };
        let lcov = generate_lcov(&summary);
        assert!(lcov.contains("SF:src/pipeline.fav"), "should have SF:");
        assert!(lcov.contains("FN:1,LoadCsv"), "should have FN: line");
        assert!(lcov.contains("DA:10,1"), "should have covered line");
        assert!(lcov.contains("DA:12,0"), "should have uncovered line");
        assert!(lcov.contains("end_of_record"), "should have end_of_record");
    }

    #[test]
    fn coverage_summary_line_format() {
        use crate::coverage::{CoverageFileStat, CoverageSummary, format_coverage_summary_console};
        let stat = CoverageFileStat {
            path: "src/pipeline.fav".to_string(),
            covered: 8, total: 10,
            fn_covered: 1, fn_total: 1,
            line_hits: vec![],
            fn_hits: vec![],
        };
        let summary = CoverageSummary { files: vec![stat] };
        let out = format_coverage_summary_console(&summary, 80.0);
        assert!(out.contains("Coverage:"), "should have Coverage: header");
        assert!(out.contains("✓"), "should have check mark for 80%+ file");
    }
}
```
