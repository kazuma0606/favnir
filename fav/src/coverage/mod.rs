/// テストカバレッジ HTML / LCOV 出力モジュール（v21.3.0）。
///
/// `fav test --coverage --html` / `--lcov` の出力形式を担う。
/// 外部 CSS/JS 依存なし。
use std::collections::HashMap;

// ── 型定義 ───────────────────────────────────────────────────────────────────

/// ファイル単位のカバレッジ統計。
#[derive(Debug, Clone)]
pub struct CoverageFileStat {
    pub path: String,
    pub covered: usize,
    pub total: usize,
    pub fn_covered: usize,
    pub fn_total: usize,
    /// 実行可能行番号 → カバー済み（true）/ 未カバー（false）
    pub line_hits: Vec<(u32, bool)>,
    /// 関数名 → カバー済みか
    pub fn_hits: Vec<(String, bool)>,
}

impl CoverageFileStat {
    pub fn pct(&self) -> f64 {
        if self.total == 0 {
            100.0
        } else {
            self.covered as f64 / self.total as f64 * 100.0
        }
    }
}

/// プロジェクト全体の集計。
#[derive(Debug, Default)]
pub struct CoverageSummary {
    pub files: Vec<CoverageFileStat>,
}

impl CoverageSummary {
    pub fn total_covered(&self) -> usize {
        self.files.iter().map(|f| f.covered).sum()
    }
    pub fn total_lines(&self) -> usize {
        self.files.iter().map(|f| f.total).sum()
    }
    pub fn overall_pct(&self) -> f64 {
        let t = self.total_lines();
        if t == 0 {
            100.0
        } else {
            self.total_covered() as f64 / t as f64 * 100.0
        }
    }
}

// ── コンソールサマリー ────────────────────────────────────────────────────────

/// コンソール出力: "Coverage: XX.X% (N/M lines)" + ファイル別 ✓/✗ 行。
/// `threshold` 未満のファイルを `✗` で表示する。
pub fn format_coverage_summary_console(summary: &CoverageSummary, threshold: f64) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "Coverage: {:.1}% ({}/{} lines)\n",
        summary.overall_pct(),
        summary.total_covered(),
        summary.total_lines()
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

// ── HTML レポート ─────────────────────────────────────────────────────────────

/// カバレッジ HTML レポート（index.html）を生成する。外部依存なし。
pub fn generate_coverage_html(
    summary: &CoverageSummary,
    source_map: &HashMap<String, String>,
) -> String {
    let mut html = String::new();
    html.push_str(HTML_HEADER);

    // サマリーヘッダー
    html.push_str(&format!(
        "<h1>Favnir Coverage Report</h1>\
         <p class=\"summary\">Coverage: <strong>{:.1}%</strong> ({}/{} lines)</p>",
        summary.overall_pct(),
        summary.total_covered(),
        summary.total_lines()
    ));

    // ファイル一覧テーブル
    html.push_str("<table><tr><th>File</th><th>Line %</th><th>Lines</th><th>Fn %</th></tr>");
    for f in &summary.files {
        let cls = if f.pct() >= 80.0 { "ok" } else { "warn" };
        let fn_pct = if f.fn_total == 0 {
            100.0f64
        } else {
            f.fn_covered as f64 / f.fn_total as f64 * 100.0
        };
        html.push_str(&format!(
            "<tr class=\"{}\"><td>{}</td><td>{:.1}%</td><td>{}/{}</td><td>{:.0}% ({}/{})</td></tr>",
            cls, f.path, f.pct(), f.covered, f.total, fn_pct, f.fn_covered, f.fn_total
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
                    Some(true) => "covered",
                    Some(false) => "uncovered",
                    None => "nonexec",
                };
                let escaped = line
                    .replace('&', "&amp;")
                    .replace('<', "&lt;")
                    .replace('>', "&gt;");
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
.covered   { display: block; background: #c8e6c9; }
.uncovered { display: block; background: #ffcdd2; }
.nonexec   { display: block; color: #999; }
.summary { font-size: 1.2em; }
</style></head><body>"#;

const HTML_FOOTER: &str = "</body></html>";

// ── LCOV ─────────────────────────────────────────────────────────────────────

/// LCOV 形式（lcov.info）を生成する。
pub fn generate_lcov(summary: &CoverageSummary) -> String {
    let mut out = String::new();
    for f in &summary.files {
        out.push_str("TN:\n");
        out.push_str(&format!("SF:{}\n", f.path));
        // 関数定義（行番号は 1-indexed の連番）
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
