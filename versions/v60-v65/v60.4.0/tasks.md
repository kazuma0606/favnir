# v60.4.0 Tasks — LSP Diagnostic 完全統合（全エラーコードの位置情報付与）

Date: 2026-07-30
Status: COMPLETE

---

## T0: 事前確認

- [x] `cargo test` でベースラインが 3336 tests passed, 0 failed であることを確認
- [x] `fav/Cargo.toml` のバージョンが `"60.0.0"` であることを確認
- [x] `v60400_tests` がまだ存在しないことを確認
  - `grep -c 'v60400_tests' fav/src/driver.rs` = 0 件
- [x] `SpanOutput` がまだ存在しないことを確認
  - `grep -c 'SpanOutput' fav/src/driver.rs` = 0 件
- [x] `CheckDiagnostic` に `span` フィールドがまだないことを確認
  - `grep -c 'span:.*SpanOutput' fav/src/driver.rs` = 0 件
- [x] `Span::new` のシグネチャが `(file, start, end, line, col)` であることを `fav/src/frontend/lexer.rs` で確認

---

## T1: `SpanOutput` 構造体追加（`driver.rs`）

`CheckDiagnostic` の直前（`// ── fav check --json structs` ブロック内）に追加する。

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

- [x] `SpanOutput` 構造体を `CheckDiagnostic` の直前に追加した
- [x] `#[derive(serde::Serialize)]` が付いている

---

## T2: `CheckDiagnostic` に `span` フィールド追加（`driver.rs`）

既存の `col: u32,` の直後に挿入する。

```rust
    col:        u32,
    span:       SpanOutput,   // v60.4.0: 追加
    suggestion: String,
```

- [x] `span: SpanOutput` フィールドを `col` の直後に追加した
- [x] 既存の `file` / `line` / `col` フラットフィールドは残してある（後方互換）

---

## T3: `type_error_to_diag` に `span` フィールド追加（`driver.rs`）

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

- [x] `type_error_to_diag` に `span: SpanOutput { ... }` フィールドを追加した
- [x] `len` は `e.span.end.saturating_sub(e.span.start)` を使用している
- [x] `type_warning_to_diag` も `span` を追加した（struct の必須フィールドのためコンパイル必須；W コード拡張はスコープ外）

---

## T4: `v60400_tests` モジュール追加（`driver.rs`）

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

- [x] `v60400_tests` モジュールを `v60300_tests` の直前（上側）に追加した
- [x] `use super::*;` が含まれている（`type_error_to_diag` / `SpanOutput` / `CheckDiagnostic` アクセス用）
- [x] `check_json_includes_span` テストが含まれている
  - `serde_json::Value` にパースして `v["span"]["line"].is_number()` を検証している
- [x] `lsp_diagnostic_has_span` テストが含まれている
  - `src` の 3 行目が 10 文字以上（`"       foo"` など）でクランプが発生しない
  - `errors_to_diagnostics` は `use crate::lsp::diagnostics::errors_to_diagnostics` で明示 import

---

## T5: テスト実行・確認

- [x] `cargo test -j 8 -- --test-threads=8` を実行
- [x] `v60400_tests::check_json_includes_span` pass
- [x] `v60400_tests::lsp_diagnostic_has_span` pass
- [x] 総テスト数 **3338** tests passed, 0 failed を確認

---

## T6: 事後処理

- [x] `versions/current.md` を v60.4.0 / 3338 tests に更新
- [x] `versions/roadmap/roadmap-v60.1-v61.0.md` の v60.4.0 実績欄を更新
- [x] CHANGELOG.md: サブバージョンのため個別エントリは不要
- [x] このファイル（tasks.md）を COMPLETE ステータスに更新

---

## コードレビュー指摘と対応

spec-reviewer 指摘（実装前）:
- [HIGH] lsp_diagnostic_has_span クランプ FAIL → src 3 行目を "       foo"（10文字）に変更
- [HIGH] check_json_includes_span 偽陽性 → serde_json::Value パース + v["span"]["line"].is_number() 検証
- [HIGH] spec に既存 LSP テスト参照なし → 既存実装テーブルにテスト名追記
- [MED] use super::* 説明不足 → tasks.md T4 + plan.md 注意事項に追記
- [MED] len バイト長注記なし → spec.md 2 箇所に注記追加
- [LOW] Span::new シグネチャ根拠欠如 → tasks.md T0 に確認チェック追加

実装上の注意:
- `type_warning_to_diag` も `span` フィールドを追加（CheckDiagnostic の必須フィールドのためコンパイル必須）
- `e.span.end.saturating_sub(e.span.start) as u32`（Span.start/end は usize → u32 キャスト必要）

テスト 2/2 一発 pass。

---

Status: COMPLETE
