# v16.1.0 Plan — エラーメッセージ品質向上

Date: 2026-06-14
Branch: master

---

## Phase A — Cargo バージョン更新 + strsim 依存追加

### A-1: `fav/Cargo.toml` version 更新

```toml
version = "16.1.0"
```

### A-2: `fav/Cargo.toml` strsim 依存追加

`[dependencies]` セクション（WASM でも動作するため通常依存に追加）:

```toml
strsim = "0.11"
```

---

## Phase B — テスト追加（v161000_tests）

`fav/src/driver.rs` の末尾付近（v160000_tests の後）に追加:

```rust
// ── v161000_tests (v16.1.0) — エラーメッセージ品質向上 ──────────────────────
#[cfg(test)]
mod v161000_tests {
    use std::fs;

    #[test]
    fn version_is_16_1_0() {
        let cargo = fs::read_to_string("Cargo.toml").unwrap();
        assert!(
            cargo.contains("version = \"16.1.0\""),
            "Cargo.toml version should be 16.1.0"
        );
    }

    #[test]
    fn error_output_has_line_number() {
        // E0001 を引き起こすソースをコンパイルして出力を確認
        let src = "fn main() -> String { undefined_var }";
        let output = crate::driver::check_source_to_string(src, "test.fav");
        assert!(
            output.contains(" --> "),
            "error output should contain ' --> ' location: got:\n{output}"
        );
    }

    #[test]
    fn error_output_has_caret() {
        let src = "fn main() -> String { undefined_var }";
        let output = crate::driver::check_source_to_string(src, "test.fav");
        assert!(
            output.contains('^'),
            "error output should contain '^' underline: got:\n{output}"
        );
    }

    #[test]
    fn error_output_has_hint() {
        let src = "fn main() -> String { undefined_var }";
        let output = crate::driver::check_source_to_string(src, "test.fav");
        assert!(
            output.contains("ヒント") || output.contains("hint") || output.contains("help"),
            "error output should contain hint text: got:\n{output}"
        );
    }

    #[test]
    fn error_output_has_url() {
        let src = "fn main() -> String { undefined_var }";
        let output = crate::driver::check_source_to_string(src, "test.fav");
        assert!(
            output.contains("favnir.dev/errors/"),
            "error output should contain error URL: got:\n{output}"
        );
    }
}
```

**補助関数 `check_source_to_string` を `driver.rs` に追加:**

```rust
#[cfg(test)]
pub fn check_source_to_string(src: &str, filename: &str) -> String {
    // checker を呼び出し、エラーメッセージを String として返す（テスト専用）
    match crate::middle::checker::check_str(src, filename) {
        Ok(_)    => String::new(),
        Err(msg) => msg,
    }
}
```

---

## Phase C — `fav/src/span.rs` 新規作成

`fav/src/span.rs` を新規作成:

```rust
/// ソースコード上の位置情報
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Span {
    /// ファイル ID（ソーステーブルのインデックス。単一ファイルの場合は 0）
    pub file: u32,
    /// 1-based 行番号
    pub line: u32,
    /// 1-based 列番号（バイト位置）
    pub col:  u32,
    /// トークンの長さ（バイト数）
    pub len:  u32,
}

impl Span {
    pub const DUMMY: Span = Span { file: 0, line: 0, col: 0, len: 0 };

    pub fn is_dummy(&self) -> bool {
        self.line == 0
    }
}
```

`fav/src/lib.rs`（または `main.rs`）に `pub mod span;` を追加。

---

## Phase D — Lexer に Span 追加（lexer.rs）

### D-1: `Token` 構造体に `span: Span` フィールド追加

```rust
// 変更前
pub enum Token {
    Ident(String),
    Int(i64),
    // ...
}

// 変更後
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

pub enum TokenKind {
    Ident(String),
    Int(i64),
    // ...（既存の Token variant をそのまま TokenKind に移動）
}
```

**注意**: `Token` が `TokenKind` に変わるため、`parser.rs` 内の全パターンマッチを
`Token::Ident(s)` → `TokenKind::Ident(s)`（+ `.kind` へのアクセス）に変更する。
変更箇所が多い場合は `impl Token { fn kind(&self) -> &TokenKind { &self.kind } }` を追加して
既存コードへの影響を最小化する。

### D-2: Lexer 内で行・列を追跡

```rust
struct Lexer {
    source:  Vec<char>,
    pos:     usize,
    line:    u32,
    col:     u32,
    file_id: u32,
}

impl Lexer {
    fn current_span(&self, start_col: u32, len: u32) -> Span {
        Span { file: self.file_id, line: self.line, col: start_col, len }
    }

    fn advance(&mut self) {
        if self.pos < self.source.len() {
            if self.source[self.pos] == '\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
            self.pos += 1;
        }
    }
}
```

各トークン生成時に `self.current_span(start_col, len)` で Span を作成して付与する。

---

## Phase E — AST ノードに Span 追加（ast.rs）

優先 5 ノードに `span: Span` を追加する:

```rust
// Expr::Var
Var { name: String, span: Span },

// Expr::Call
Call { fn_name: String, args: Vec<Expr>, span: Span },

// Stmt::Bind
Bind { name: String, expr: Box<Expr>, span: Span },

// FnDef
pub struct FnDef {
    pub name:    String,
    pub span:    Span,
    pub params:  Vec<Param>,
    pub ret_ty:  Type,
    pub effects: Vec<Effect>,
    pub body:    Vec<Stmt>,
}

// TypeAnnotation（型不一致エラー用）
Annotation { ty: Type, span: Span },
```

**既存コードへの影響を最小化する方針:**

Span なしで構築している既存のコードは `Span::DUMMY` を使う。
```rust
Expr::Var { name: name.to_string(), span: Span::DUMMY }
```
Parser を修正して正しい Span を付与するのは Phase F で行う。

---

## Phase F — Parser に Span 伝播（parser.rs）

### F-1: `parse_ident` で Span を取得

```rust
fn parse_ident(&mut self) -> Result<(String, Span), ParseError> {
    match self.current_token() {
        Some(tok) if matches!(tok.kind, TokenKind::Ident(_)) => {
            let span = tok.span;
            let name = tok.kind.as_ident().unwrap().to_string();
            self.advance();
            Ok((name, span))
        }
        _ => Err(ParseError::expected("identifier")),
    }
}
```

### F-2: `parse_expr` の変数参照で Span を付与

```rust
// 変数参照
TokenKind::Ident(name) => {
    let span = tok.span;
    self.advance();
    Ok(Expr::Var { name: name.clone(), span })
}
```

### F-3: `parse_call` で Span を付与

```rust
// 関数呼び出し
let span = fn_name_token.span;
Ok(Expr::Call { fn_name, args, span })
```

### F-4: `parse_bind_stmt` で Span を付与

```rust
// bind x <- expr
let span = bind_token.span;  // `bind` キーワードの位置
Ok(Stmt::Bind { name, expr, span })
```

---

## Phase G — `fav/src/error.rs` 新規作成

```rust
use crate::span::Span;

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub code:  String,
    pub msg:   String,
    pub span:  Span,
    pub label: Option<String>,   // アンダーライン横のラベル
    pub hints: Vec<String>,
    pub notes: Vec<String>,
}

impl Diagnostic {
    pub fn new(code: impl Into<String>, msg: impl Into<String>, span: Span) -> Self {
        Self {
            code:  code.into(),
            msg:   msg.into(),
            span,
            label: None,
            hints: vec![],
            notes: vec![],
        }
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hints.push(hint.into());
        self
    }

    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(note.into());
        self
    }

    pub fn with_url(mut self) -> Self {
        self.notes.push(format!(
            "参照: https://favnir.dev/errors/{}",
            self.code
        ));
        self
    }
}

/// rustc スタイルの診断メッセージ文字列を生成する
pub fn format_diagnostic(
    diag:   &Diagnostic,
    source: &str,
    file:   &str,
    color:  bool,
) -> String {
    let mut out = String::new();

    // 1 行目: [Exxxx] message
    let header = format!("[{}] {}", diag.code, diag.msg);
    out.push_str(&if color { format!("\x1b[31m{header}\x1b[0m") } else { header });
    out.push('\n');

    // Span が DUMMY の場合はここで終了
    if diag.span.is_dummy() {
        for hint in &diag.hints { out.push_str(&format!("  = ヒント: {hint}\n")); }
        for note in &diag.notes { out.push_str(&format!("  = {note}\n")); }
        return out;
    }

    let line_num = diag.span.line as usize;
    let col      = diag.span.col  as usize;
    let len      = diag.span.len  as usize;

    // 2 行目: --> file:line:col
    out.push_str(&format!(" --> {file}:{line_num}:{col}\n"));

    // 行番号の桁数（余白揃え用）
    let pad = line_num.to_string().len();

    // 3 行目: 余白 |
    out.push_str(&format!("{:pad$} |\n", ""));

    // 4 行目: ソース行
    let source_line = source
        .lines()
        .nth(line_num.saturating_sub(1))
        .unwrap_or("");
    out.push_str(&format!("{line_num} | {source_line}\n"));

    // 5 行目: アンダーライン + ラベル
    let underline = "^".repeat(len.max(1));
    let label = diag.label.as_deref().unwrap_or("");
    let caret_line = format!("{:pad$} | {:col$}{} {}", "", "", underline, label, pad = pad, col = col.saturating_sub(1));
    out.push_str(&if color {
        format!("\x1b[31m{caret_line}\x1b[0m\n")
    } else {
        format!("{caret_line}\n")
    });

    // 6 行目: 余白 |
    out.push_str(&format!("{:pad$} |\n", ""));

    // hint / note 行
    for hint in &diag.hints {
        out.push_str(&format!("  = ヒント: {hint}\n"));
    }
    for note in &diag.notes {
        out.push_str(&format!("  = {note}\n"));
    }

    out
}
```

`fav/src/lib.rs`（または `main.rs`）に `pub mod error;` を追加。

---

## Phase H — `levenshtein_candidates` 実装（checker.rs）

```rust
// fav/src/middle/checker.rs に追加

/// name と Levenshtein 距離 ≤ threshold の候補を candidates から最大 max_results 件返す
fn levenshtein_candidates(
    name:        &str,
    candidates:  &[&str],
    threshold:   usize,
    max_results: usize,
) -> Vec<String> {
    let mut scored: Vec<(usize, &str)> = candidates
        .iter()
        .filter_map(|c| {
            let d = strsim::levenshtein(name, c);
            if d <= threshold && *c != name { Some((d, *c)) } else { None }
        })
        .collect();
    scored.sort_by_key(|(d, _)| *d);
    scored.into_iter().take(max_results).map(|(_, c)| c.to_string()).collect()
}
```

---

## Phase I — checker.rs エラー出力を Diagnostic に統一

### I-1: 最頻出 10 エラーを Diagnostic に変換

現状の `String` 返却を `Diagnostic` に変更していく。
影響範囲が大きいため、v16.1.0 では **E0001 / E0007 / E0008 / E0009 / E0018** の 5 エラーを優先する。

**E0001（未定義変数）の変換例:**

```rust
// 変更前
Err(format!("[E0001] undefined variable: {name}"))

// 変更後
let mut diag = Diagnostic::new("E0001", format!("undefined variable: {name}"), var_span)
    .with_label("この変数は未定義です");

// typo 候補を生成
let scope_vars: Vec<&str> = env.variables().map(|s| s.as_str()).collect();
let candidates = levenshtein_candidates(name, &scope_vars, 2, 3);
if !candidates.is_empty() {
    let hint = format!("`{}` の typo ではないですか？", candidates.join("` / `"));
    diag = diag.with_hint(hint);
}

diag = diag.with_url();
Err(format_diagnostic(&diag, source, filename, use_color))
```

**E0007（未定義関数）の変換例:**

```rust
let mut diag = Diagnostic::new("E0007", format!("undefined function: {fn_name}"), call_span)
    .with_label("この関数は定義されていません");

let fn_names: Vec<&str> = env.functions().map(|s| s.as_str()).collect();
let candidates = levenshtein_candidates(fn_name, &fn_names, 2, 3);
if !candidates.is_empty() {
    diag = diag.with_hint(format!("`{}` の typo ではないですか？", candidates.join("` / `")));
}
diag = diag.with_url();
Err(format_diagnostic(&diag, source, filename, use_color))
```

### I-2: 残りのエラーコードに hint / note を追加

Span が DUMMY のエラー（既存の Span なし）でも hint と URL は付与できる。

```rust
// E0018 再束縛禁止
let diag = Diagnostic::new("E0018", format!("rebind not allowed: {name}"), Span::DUMMY)
    .with_hint(format!("`{name}` は既に束縛済みです。別の変数名を使ってください"))
    .with_url();
Err(format_diagnostic(&diag, source, filename, use_color))

// E0314 missing !AWS effect
let diag = Diagnostic::new("E0314", "AWS.* call requires !AWS effect", Span::DUMMY)
    .with_hint("この関数の宣言に `!AWS` エフェクトを追加してください")
    .with_url();

// E0319 missing !Stream effect
let diag = Diagnostic::new("E0319", "Kafka.* call requires !Stream effect", Span::DUMMY)
    .with_hint("この関数の宣言に `!Stream` エフェクトを追加してください")
    .with_url();
```

---

## Phase J — `driver.rs` 更新

### J-1: `fav check` のエラー表示を `format_diagnostic` 経由に統一

```rust
// driver.rs の check コマンドハンドラ
let use_color = !args.contains(&"--no-color");

match checker::check_file(path, &fav_toml) {
    Ok(_)    => println!("check passed"),
    Err(msg) => {
        // msg は既に format_diagnostic で整形済み（Diagnostic 化したエラーはここで表示）
        eprintln!("{msg}");
        std::process::exit(1);
    }
}
```

### J-2: `--no-color` フラグのパース追加

```rust
// CLI 引数解析部分
let no_color = args.iter().any(|a| a == "--no-color");
// no_color を checker / format_diagnostic に渡す
```

---

## Phase K — サイトドキュメント作成

### K-1: `site/content/docs/errors/index.mdx` 新規作成

エラーコード一覧ページ（E0001〜E0320 のリンク集）。

### K-2: `site/content/docs/errors/E0001.mdx` 〜 `E0020.mdx` 新規作成

各ファイルの構成:

```mdx
---
title: E0001 — undefined variable
description: 未定義の変数を参照しています
---

# E0001 — undefined variable

## 原因

参照した変数名がスコープ内に存在しません。

## よくある間違い

```fav
fn greet() -> String {
  String.concat("Hello, ", user_name)  // user_name が未定義
}
```

## 修正方法

```fav
fn greet(user_name: String) -> String {
  String.concat("Hello, ", user_name)  // 引数として受け取る
}
```

## 関連エラー

- E0007: undefined function
```

---

## Phase L — テスト・コミット

### L-1: `cargo test v161000` → 5/5 パス確認

### L-2: `cargo test` → 全件パス（リグレッションなし）確認

### L-3: コミット

```
feat: v16.1.0 — エラーメッセージ品質向上（rustc スタイル + typo ヒント）
```

---

## 変更ファイル一覧

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `fav/Cargo.toml` | 更新 | version 16.1.0 + strsim 依存追加 |
| `fav/src/span.rs` | 新規 | Span 構造体 |
| `fav/src/error.rs` | 新規 | Diagnostic 型 + format_diagnostic |
| `fav/src/frontend/lexer.rs` | 更新 | Token に Span 付与・行・列追跡 |
| `fav/src/frontend/parser.rs` | 更新 | 優先 5 ノードに Span 伝播 |
| `fav/src/ast.rs` | 更新 | Var / Call / Bind / FnDef / Annotation に Span フィールド追加 |
| `fav/src/middle/checker.rs` | 更新 | levenshtein_candidates + E0001/E0007/E0008/E0009/E0018 を Diagnostic 化 + 全コードに hint/URL 追加 |
| `fav/src/driver.rs` | 更新 | format_diagnostic 経由の出力 + --no-color + v161000_tests |
| `site/content/docs/errors/index.mdx` | 新規 | エラーコード一覧 |
| `site/content/docs/errors/E0001.mdx` 〜 `E0020.mdx` | 新規 | 最頻出 20 エラーの詳細ページ |

---

## 実装上の注意点

1. **Token → TokenKind の移行**: `Token` を `{ kind: TokenKind, span: Span }` に変えると、`parser.rs` の全パターンマッチが壊れる。移行中は `impl Token { fn as_ident(&self) -> Option<&str> { ... } }` 等のアクセサを追加して段階的に移行するか、`TokenKind` を `Token` の alias として残す期間を設ける。

2. **Span::DUMMY の多用**: v16.1.0 では Span が付与されていないノードは `Span::DUMMY` を使う。`format_diagnostic` は `is_dummy()` で分岐し、DUMMY の場合は `-->` 行・アンダーラインをスキップする。テストが `^` の有無を検査するため、テスト用ソースは必ず Span が付与されているノード（Var / Call）でエラーを起こす。

3. **checker.rs の `source` / `filename` の伝播**: 現状の checker は `source` 文字列を持っていない場合がある。`check_str(src, filename)` のシグネチャ変更時に伝播先を確認する。

4. **strsim の Levenshtein**: `strsim::levenshtein(a, b)` は O(|a|×|b|) の実装。変数名は通常短い（< 30 文字）ため、候補数が 100 件程度なら無視できるコスト。

5. **カラー出力の環境変数**: `NO_COLOR` 環境変数（`https://no-color.org/`）も尊重する。`--no-color` フラグに加えて `std::env::var("NO_COLOR").is_ok()` でも無効化する。
