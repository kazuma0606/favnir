# v70.4.0 Plan — 構造化エラー診断

Date: 2026-08-09
Status: 計画中

---

## 実装ステップ（依存順）

### Step 1: `ErrorReport` 構造体と `format_diagnostic` を driver.rs に追加

既存の v38.1.0 suggest.rs コードの直後のセクションとして、driver.rs の末尾付近（v703000_tests の直前）に追加する。

まず driver.rs の `use` セクション（ファイル先頭付近）に以下を追加する:
```rust
use strsim;
```

続けて、以下の構造体・関数群を追加する:

```rust
// ── v70.4.0: 構造化エラー診断 ────────────────────────────────────────────────

/// 構造化エラー診断レポート。
/// rustc スタイルのターミナル出力 / LSP JSON 出力の両方を生成できる。
pub struct ErrorReport {
    pub code: &'static str,
    pub file: String,
    pub line: usize,
    pub col: usize,
    pub source_line: String,
    pub span_len: usize,
    pub message: String,
    pub hint: Option<String>,
    pub suggestion: Option<String>,
    pub doc_url: Option<String>,
}

/// Levenshtein 距離 ≤ 3 の候補から最近傍を返す。
/// 同距離の場合は辞書順で最初を返す。
pub fn suggest_similar_name(name: &str, candidates: &[&str]) -> Option<String> {
    candidates
        .iter()
        .filter_map(|&c| {
            let dist = strsim::levenshtein(name, c);
            if dist <= 3 { Some((dist, c)) } else { None }
        })
        .min_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)))
        .map(|(_, c)| c.to_string())
}

/// rustc スタイルの診断テキストを生成する。
pub fn format_diagnostic(r: &ErrorReport) -> String {
    let col_spaces = " ".repeat(r.col.saturating_sub(1));
    let underline = format!("{}^{}", col_spaces, "^".repeat(r.span_len.saturating_sub(1)));
    let ln = r.line.to_string();
    let mut out = format!(
        "error[{}] {}:{}:{}\n  |\n{}| {}\n  | {}\n  | {}\n  |",
        r.code, r.file, r.line, r.col,
        ln, r.source_line,
        underline,
        r.message,
    );
    if let Some(h) = &r.hint {
        out.push_str(&format!("\n  = ヒント: {h}"));
    }
    if let Some(s) = &r.suggestion {
        out.push_str(&format!("\n  = 自動移行: {s}"));
    }
    if let Some(u) = &r.doc_url {
        out.push_str(&format!("\n  = 参照: {u}"));
    }
    out
}

/// E0374（`!Effect` 廃止）専用レポートビルダー。
pub fn build_e0374_report(
    file: &str, line: usize, col: usize,
    source_line: &str, effect_name: &str,
) -> ErrorReport {
    ErrorReport {
        code: "E0374",
        file: file.to_string(),
        line, col,
        source_line: source_line.to_string(),
        span_len: effect_name.len() + 1, // "!" + effect_name
        message: "`!Effect` アノテーション構文は v35.4.0 で廃止されました".to_string(),
        hint: Some(format!(
            "`ctx: AppCtx` を第1引数として追加し、`!{}` を削除してください", effect_name
        )),
        suggestion: Some(format!("fav migrate --from v35 --in-place {file}")),
        doc_url: Some("https://favnir.dev/docs/language/ctx-migration".to_string()),
    }
}

/// E0001（未定義変数）専用レポートビルダー。
pub fn build_e0001_report(
    file: &str, line: usize, col: usize,
    source_line: &str, var_name: &str,
    candidates: &[&str],
) -> ErrorReport {
    let hint = suggest_similar_name(var_name, candidates)
        .map(|s| format!("`{s}` のことですか？（3文字以内の編集距離）"));
    ErrorReport {
        code: "E0001",
        file: file.to_string(),
        line, col,
        source_line: source_line.to_string(),
        span_len: var_name.len(),
        message: format!("未定義変数 `{var_name}`"),
        hint,
        suggestion: None,
        doc_url: None,
    }
}
```

確認: `cargo test` で既存テスト（3565 件）が引き続き pass することを確認。

---

### Step 2: `v704000_tests` モジュールを driver.rs 末尾に追加

```rust
#[cfg(test)]
mod v704000_tests {
    #[test]
    fn diagnostic_e0374_shows_migration_hint() {
        let report = super::build_e0374_report(
            "benchmarks/compare.fav", 43, 62,
            "fn write_results_md(data: JsonValue) -> Result<Unit, String> !IO {",
            "IO",
        );
        let diag = super::format_diagnostic(&report);
        assert!(diag.contains("error[E0374]"), "should show E0374 code");
        assert!(diag.contains("benchmarks/compare.fav"), "should show file");
        assert!(diag.contains("v35.4.0"), "should mention deprecation version");
        assert!(diag.contains("ctx: AppCtx"), "should hint ctx migration");
        assert!(diag.contains("fav migrate --from v35"), "should suggest migrate command");
        assert!(diag.contains("favnir.dev/docs/language/ctx-migration"), "should include doc_url");
    }

    #[test]
    fn diagnostic_e0001_suggests_similar_name() {
        let report = super::build_e0001_report(
            "pipeline.fav", 12, 28,
            "    bind result <- process(ordr)",
            "ordr",
            &["order", "other", "data"],
        );
        let diag = super::format_diagnostic(&report);
        assert!(diag.contains("error[E0001]"), "should show E0001 code");
        assert!(diag.contains("未定義変数"), "should show undefined var message");
        assert!(diag.contains("order"), "should suggest 'order' as similar name");
        // suggest_similar_name 単体テスト
        assert_eq!(
            super::suggest_similar_name("ordr", &["order", "other", "data"]),
            Some("order".to_string()),
            "levenshtein dist(ordr, order)=1 should be the closest"
        );
    }
}
```

確認: `cargo test v704000` で 2 件 pass することを確認。

---

### Step 3: Cargo.toml バージョン更新

- `fav/Cargo.toml` の `version = "70.3.0"` → `"70.4.0"`
- driver.rs 内のバージョン文字列を更新:
  - 対象: `cargo_toml_version_is_70_3_0` テスト関数内の `"70.3.0"` 文字列
  - `replace_all: true` で `"70.3.0"` → `"70.4.0"` に一括置換（Cargo.toml と driver.rs 両方）

---

### Step 4: CHANGELOG.md 更新

```markdown
## [v70.4.0] — 2026-08-09 — 構造化エラー診断

### Added
- `ErrorReport` 構造体（code / file / line / col / source_line / span_len / message / hint / suggestion / doc_url）
- `suggest_similar_name(name, candidates)` — `strsim::levenshtein` を使って距離 ≤ 3 の候補を返す
- `format_diagnostic(report)` — rustc スタイルのエラー診断テキスト生成
- `build_e0374_report` — E0374 専用ビルダー（ctx 移行ヒント + fav migrate 案内）
- `build_e0001_report` — E0001 専用ビルダー（タイポ候補提示）
- `v704000_tests`: 2 件追加（3565 → 3567 tests）
  - `diagnostic_e0374_shows_migration_hint`
  - `diagnostic_e0001_suggests_similar_name`
```

---

### Step 5: 最終確認

- `cargo test v704000` で 2 件 pass
- `cargo test` 全体で 3567 tests pass（0 failures）
- `versions/current.md` を v70.4.0 進行中に更新
