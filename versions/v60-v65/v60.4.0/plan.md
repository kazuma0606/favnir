# v60.4.0 Plan — LSP Diagnostic 完全統合（全エラーコードの位置情報付与）

Date: 2026-07-30

---

## 実装方針

3 箇所の変更（`SpanOutput` 追加・`CheckDiagnostic` 更新・`type_error_to_diag` 更新）と
`v60400_tests` モジュール追加を行う。

LSP 側（`lsp/diagnostics.rs` / `lsp/mod.rs`）は変更不要。

---

## ステップ詳細

### Step 1: `SpanOutput` 構造体を `CheckDiagnostic` の直前に追加

対象位置: `driver.rs` の `// ── fav check --json structs (v12.5.0)` ブロック（L3927 付近）、
`CheckDiagnostic` の直前。

```rust
/// v60.4.0: `fav check --json` の span サブオブジェクト
#[derive(serde::Serialize)]
struct SpanOutput {
    file: String,
    line: u32,
    col:  u32,
    len:  u32,
}
```

### Step 2: `CheckDiagnostic` に `span` フィールドを追加

既存の `col: u32,` の直後に `span: SpanOutput,` を挿入する。

```rust
#[derive(serde::Serialize)]
struct CheckDiagnostic {
    code:       String,
    message:    String,
    file:       String,
    line:       u32,
    col:        u32,
    span:       SpanOutput,   // v60.4.0: 追加
    suggestion: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    hints:      Vec<String>,
}
```

### Step 3: `type_error_to_diag` に `span` フィールドを追加

```rust
fn type_error_to_diag(e: &crate::middle::checker::TypeError, suggestion: &str) -> CheckDiagnostic {
    CheckDiagnostic {
        code:       e.code.to_string(),
        message:    e.message.clone(),
        file:       e.span.file.clone(),
        line:       e.span.line,
        col:        e.span.col,
        span:       SpanOutput {
            file: e.span.file.clone(),
            line: e.span.line,
            col:  e.span.col,
            len:  e.span.end.saturating_sub(e.span.start),
        },
        suggestion: suggestion.to_string(),
        hints:      e.hints.clone(),
    }
}
```

### Step 4: `v60400_tests` モジュール追加（`driver.rs`）

`v60300_tests` の直前（上側）に挿入する。

```rust
// -- v60400_tests (v60.4.0) -- LSP Diagnostic 完全統合 --
#[cfg(test)]
mod v60400_tests {
    use super::*;

    #[test]
    fn check_json_includes_span() {
        // type_error_to_diag が span サブオブジェクトを JSON に含むことを確認
        // json.contains("\"line\"") では既存フラットフィールドでも通過するため、
        // JSON パースして span サブオブジェクト内の値を検証する
        use crate::frontend::lexer::Span;
        use crate::middle::checker::TypeError;
        let e = TypeError::new(
            "E0102",
            "undefined: `foo`",
            Span::new("test.fav", 0, 7, 1, 4),
        );
        let diag = type_error_to_diag(&e, "");
        let json = serde_json::to_string(&diag).expect("serialize CheckDiagnostic");
        let v: serde_json::Value = serde_json::from_str(&json).expect("parse JSON");
        assert!(v.get("span").is_some(), "check --json should include span field, got: {json}");
        assert!(v["span"]["line"].is_number(), "span.line should be a number, got: {}", v["span"]);
        assert!(v["span"]["col"].is_number(), "span.col should be a number, got: {}", v["span"]);
    }

    #[test]
    fn lsp_diagnostic_has_span() {
        // errors_to_diagnostics が TypeError.span を LSP range に正しく変換することを確認
        // span_to_range は start_char = col.saturating_sub(1).min(line_len) でクランプするため
        // 3行目 "       foo"（10文字）で col=7 → min(6, 10) = 6 となりクランプが発生しない
        use crate::frontend::lexer::Span;
        use crate::lsp::diagnostics::errors_to_diagnostics;
        use crate::middle::checker::TypeError;
        // Span: line=3, col=7（1-indexed）→ LSP range.start: line=2, character=6（0-indexed）
        let errors = vec![TypeError::new(
            "E0102",
            "undefined: `foo`",
            Span::new("test.fav", 0, 3, 3, 7),
        )];
        // 3行目 "       foo" = 10文字 → col=7 のクランプなし
        let src = "fn f() -> Int {\n  bind a <- 1\n       foo\n}";
        let diags = errors_to_diagnostics(&errors, src);
        assert!(!diags.is_empty(), "expected at least one diagnostic");
        assert_eq!(diags[0].range.start.line, 2, "line should be 0-indexed (3-1=2)");
        assert_eq!(diags[0].range.start.character, 6, "col should be 0-indexed (7-1=6)");
    }
}
```

---

## 注意事項

- **後方互換**: `CheckDiagnostic` の `file`/`line`/`col` フラットフィールドはそのまま残す
  （既存の `fav check --json` パーサーへの影響を避けるため）
- **`type_warning_to_diag`**: W コードはスコープ外（ロードマップは E コードのみ言及）
- **`use super::*` の意味**: `type_error_to_diag` / `SpanOutput` / `CheckDiagnostic` は非 pub だが、
  同一ファイル内テストモジュールの `use super::*` でアクセス可能（Rust の慣例）。
  `errors_to_diagnostics` は `use crate::lsp::diagnostics::errors_to_diagnostics` で明示 import する。
- `Cargo.toml` version は `"60.0.0"` のまま変更しない
- rolling check の更新は不要
- `v60400_tests` は `v60300_tests` の直前（上側）に追加
- `use super::*;` を使用（`type_error_to_diag` / `SpanOutput` / `CheckDiagnostic` にアクセスするため）
- テスト実行: `cargo test -j 8 -- --test-threads=8`

---

## テスト数推移

| バージョン | テスト数 | 増加 |
|---|---|---|
| v60.3.0（ベース） | 3336 | — |
| v60.4.0 | 3338 | +2 |
