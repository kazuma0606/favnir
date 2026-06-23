# v22.6.0 実装計画 — SLA 宣言（タイムアウト・リトライ・サーキットブレーカー）

## 実装順序

```
T1（ast.rs）           ← 最初（T2/T3 の依存元）
T2（parser.rs）        ← T1 完了後
T3（checker.rs）       ← T2 完了後
T4（error_catalog.rs） ← T3 と並行可
T5（driver.rs）        ← T3 完了後
T6（main.rs）          ← T5 完了後
T7（Cargo + doc）      ← T6 完了後
```

---

## T1: `fav/src/ast.rs` — SLA struct 追加 + `TrfDef` フィールド追加

### 事前確認コマンド

```bash
grep -n "TriggerAnnotation\|pub struct TrfDef\|pub checkpoint\|pub span: Span" fav/src/ast.rs | head -15
```

### 1-1: SLA struct を追加

`PipelineDef` ブロックと `// ── FnDef` コメントの間（`TriggerAnnotation` ブロックの直後）に追加:

```rust
// ── SLA Annotations (v22.6.0) ────────────────────────────────────────────────

/// `#[timeout(seconds = 30)]` annotation on stage definitions.
#[derive(Debug, Clone)]
pub struct TimeoutAnnotation {
    pub seconds: f64,
    pub span: Span,
}

/// `#[retry(max = 3, backoff = "exponential")]` annotation on stage definitions.
/// `backoff` は `"exponential"` / `"linear"` / `"none"` のいずれか。
#[derive(Debug, Clone)]
pub struct RetryAnnotation {
    pub max: u32,
    pub backoff: String,
    pub span: Span,
}

/// `#[circuit_breaker(threshold = 0.5, window = 60)]` annotation on stage definitions.
/// `threshold` は 0.0 超〜1.0 以下、`window` は秒単位（正の整数）。
#[derive(Debug, Clone)]
pub struct CircuitBreakerAnnotation {
    pub threshold: f64,
    pub window: u64,
    pub span: Span,
}
```

### 1-2: `TrfDef` に 3 フィールドを追加

`pub checkpoint: bool,` の直後（`pub span: Span,` の前）に追加:

```rust
    /// v22.6.0: `#[timeout(seconds = N)]` annotation.
    pub timeout: Option<TimeoutAnnotation>,
    /// v22.6.0: `#[retry(max = N, backoff = "...")]` annotation.
    pub retry_ann: Option<RetryAnnotation>,
    /// v22.6.0: `#[circuit_breaker(threshold = F, window = N)]` annotation.
    pub circuit_breaker: Option<CircuitBreakerAnnotation>,
```

### 1-3: `TrfDef` struct literal を使っている箇所を修正

`cargo check --bin fav` でエラーが出た箇所（struct literal で `TrfDef { ... }` を作っている場所）に以下を追加:

```rust
timeout: None,
retry_ann: None,
circuit_breaker: None,
```

**典型的な箇所**: `parser.rs` の `parse_trf_def()` 内（後続の T2 で対応）。

### 確認

```bash
cargo check --bin fav
# T2 で parser.rs の TrfDef struct literal も修正するため、T2 完了後に再確認
```

---

## T2: `fav/src/frontend/parser.rs` — 3 アノテーションパーサー追加 + `parse_item` 統合

### 事前確認コマンド

```bash
grep -n "parse_trigger_annotation\|trigger_ann\|checkpoint_ann" fav/src/frontend/parser.rs | head -15
```

### 2-1: `parse_timeout_annotation` を追加

`parse_trigger_annotation` の直後に追加:

```rust
/// v22.6.0: parse optional `#[timeout(seconds = N)]` annotation on stage definitions.
fn parse_timeout_annotation(&mut self) -> Result<Option<crate::ast::TimeoutAnnotation>, ParseError> {
    // Lookahead: # [ timeout
    let is_timeout = self.peek() == &TokenKind::Hash
        && matches!(self.tokens.get(self.pos + 1), Some(t) if t.kind == TokenKind::LBracket)
        && matches!(self.tokens.get(self.pos + 2), Some(t) if matches!(&t.kind, TokenKind::Ident(n) if n == "timeout"));
    if !is_timeout {
        return Ok(None);
    }
    let start = self.peek_span().clone();
    self.advance();                            // #
    self.expect(&TokenKind::LBracket)?;        // [  ← advance() ではなく expect() で検証
    self.expect_ident_name("timeout")?;
    self.expect(&TokenKind::LParen)?;
    self.expect_ident_name("seconds")?;
    self.expect(&TokenKind::Eq)?;
    // seconds は整数または小数の両方に対応
    let seconds = match self.peek().clone() {
        TokenKind::Int(n) => { self.advance(); n as f64 }
        TokenKind::Float(f) => { self.advance(); f }
        other => return Err(ParseError::new(
            format!("expected number after `seconds =`, got {:?}", other),
            self.peek_span().clone(),
        )),
    };
    self.expect(&TokenKind::RParen)?;
    self.expect(&TokenKind::RBracket)?;
    Ok(Some(crate::ast::TimeoutAnnotation { seconds, span: self.span_from(&start) }))
}
```

### 2-2: `parse_retry_annotation` を追加

`parse_timeout_annotation` の直後に追加:

```rust
/// v22.6.0: parse optional `#[retry(max = N, backoff = "...")]` annotation.
fn parse_retry_annotation(&mut self) -> Result<Option<crate::ast::RetryAnnotation>, ParseError> {
    let is_retry = self.peek() == &TokenKind::Hash
        && matches!(self.tokens.get(self.pos + 1), Some(t) if t.kind == TokenKind::LBracket)
        && matches!(self.tokens.get(self.pos + 2), Some(t) if matches!(&t.kind, TokenKind::Ident(n) if n == "retry"));
    if !is_retry {
        return Ok(None);
    }
    let start = self.peek_span().clone();
    self.advance();                            // #
    self.expect(&TokenKind::LBracket)?;        // [
    self.expect_ident_name("retry")?;
    self.expect(&TokenKind::LParen)?;
    self.expect_ident_name("max")?;
    self.expect(&TokenKind::Eq)?;
    let max = match self.peek().clone() {
        TokenKind::Int(n) => { self.advance(); n as u32 }
        other => return Err(ParseError::new(
            format!("expected integer after `max =`, got {:?}", other),
            self.peek_span().clone(),
        )),
    };
    self.expect(&TokenKind::Comma)?;
    self.expect_ident_name("backoff")?;
    self.expect(&TokenKind::Eq)?;
    let backoff = self.expect_str()?;
    self.expect(&TokenKind::RParen)?;
    self.expect(&TokenKind::RBracket)?;
    Ok(Some(crate::ast::RetryAnnotation { max, backoff, span: self.span_from(&start) }))
}
```

### 2-3: `parse_circuit_breaker_annotation` を追加

`parse_retry_annotation` の直後に追加:

```rust
/// v22.6.0: parse optional `#[circuit_breaker(threshold = F, window = N)]` annotation.
fn parse_circuit_breaker_annotation(&mut self) -> Result<Option<crate::ast::CircuitBreakerAnnotation>, ParseError> {
    let is_cb = self.peek() == &TokenKind::Hash
        && matches!(self.tokens.get(self.pos + 1), Some(t) if t.kind == TokenKind::LBracket)
        && matches!(self.tokens.get(self.pos + 2), Some(t) if matches!(&t.kind, TokenKind::Ident(n) if n == "circuit_breaker"));
    if !is_cb {
        return Ok(None);
    }
    let start = self.peek_span().clone();
    self.advance();                            // #
    self.expect(&TokenKind::LBracket)?;        // [
    self.expect_ident_name("circuit_breaker")?;
    self.expect(&TokenKind::LParen)?;
    self.expect_ident_name("threshold")?;
    self.expect(&TokenKind::Eq)?;
    let threshold = match self.peek().clone() {
        TokenKind::Int(n) => { self.advance(); n as f64 }
        TokenKind::Float(f) => { self.advance(); f }
        other => return Err(ParseError::new(
            format!("expected number after `threshold =`, got {:?}", other),
            self.peek_span().clone(),
        )),
    };
    self.expect(&TokenKind::Comma)?;
    self.expect_ident_name("window")?;
    self.expect(&TokenKind::Eq)?;
    let window = match self.peek().clone() {
        TokenKind::Int(n) => { self.advance(); n as u64 }
        other => return Err(ParseError::new(
            format!("expected integer after `window =`, got {:?}", other),
            self.peek_span().clone(),
        )),
    };
    self.expect(&TokenKind::RParen)?;
    self.expect(&TokenKind::RBracket)?;
    Ok(Some(crate::ast::CircuitBreakerAnnotation { threshold, window, span: self.span_from(&start) }))
}
```

### 2-4: `parse_item()` に 3 アノテーション呼び出しを追加

`trigger_ann` 行の直後に追加（既存行を含む完全コンテキスト）:

```rust
let checkpoint_ann      = self.parse_checkpoint_annotation()?;
let trigger_ann         = self.parse_trigger_annotation()?;         // 既存（v22.4.0）
let timeout_ann         = self.parse_timeout_annotation()?;         // v22.6.0
let retry_ann           = self.parse_retry_annotation()?;           // v22.6.0
let circuit_breaker_ann = self.parse_circuit_breaker_annotation()?; // v22.6.0
```

### 2-5: Stage ブランチの TrfDef 初期化に 3 フィールドを追加

`td.trigger = trigger_ann;` の直後に追加（**2 箇所** — 通常 stage と async stage）:

```rust
td.checkpoint      = checkpoint_ann;
td.trigger         = trigger_ann;         // 既存
td.timeout         = timeout_ann;         // v22.6.0
td.retry_ann       = retry_ann;           // v22.6.0
td.circuit_breaker = circuit_breaker_ann; // v22.6.0
```

### 2-6: `TrfDef` struct literal のコンパイルエラーを修正

T1 で発生したコンパイルエラーを修正する。`TrfDef { ... }` の各インスタンスに以下を追加:

```rust
timeout: None,
retry_ann: None,
circuit_breaker: None,
```

### 確認

```bash
cargo check --bin fav
```

---

## T3: `fav/src/middle/checker.rs` — E0401 / E0402 / E0403 追加

### 事前確認コマンド

```bash
grep -n "fn check_trf_def\|self\.type_error\|E0314\|E0335" fav/src/middle/checker.rs | head -10
```

> **重要**: `check_item()` は `Item::TrfDef(td) => self.check_trf_def(td)` と委譲している。SLA バリデーションは **`check_trf_def()` 末尾**に追加する（`check_item` に直接書かない）。

### 3-1: `check_trf_def()` 末尾に SLA バリデーションを追加

```rust
// ── SLA バリデーション (v22.6.0) ──────────────────────────────────────────────
if let Some(t) = &td.timeout {
    if t.seconds <= 0.0 {
        self.type_error("E0401", format!("timeout seconds must be > 0, got {}", t.seconds), &t.span);
    }
}
if let Some(r) = &td.retry_ann {
    if r.max == 0 {
        self.type_error("E0402", "retry max must be >= 1".to_string(), &r.span);
    }
    if !matches!(r.backoff.as_str(), "exponential" | "linear" | "none") {
        self.type_error(
            "E0402",
            format!("unknown backoff strategy {:?}; expected \"exponential\", \"linear\", or \"none\"", r.backoff),
            &r.span,
        );
    }
}
if let Some(cb) = &td.circuit_breaker {
    if cb.threshold <= 0.0 || cb.threshold > 1.0 {
        self.type_error(
            "E0403",
            format!("circuit_breaker threshold must be in (0.0, 1.0], got {}", cb.threshold),
            &cb.span,
        );
    }
    if cb.window == 0 {
        self.type_error("E0403", "circuit_breaker window must be > 0".to_string(), &cb.span);
    }
}
```

> **注意**: `self.type_error(code, message, &span)` の引数順・型を既存呼び出しと照合すること。`type_error` の第 3 引数は `&Span` であることを確認してから使う。

### 確認

```bash
cargo check --bin fav
```

---

## T4: `fav/src/error_catalog.rs` — E0401〜E0403 エントリ追加

### 事前確認コマンド

```bash
grep -n "E03[0-9][0-9]\|ERROR_CATALOG\|pub const" fav/src/error_catalog.rs | tail -10
```

### 4-1: E0401〜E0403 エントリを追加

既存の最大エラーコード（E03xx）の直後に追加:

```rust
("E0401", "timeout seconds must be > 0"),
("E0402", "invalid retry annotation (max must be >= 1 / backoff must be exponential|linear|none)"),
("E0403", "invalid circuit_breaker annotation (threshold in (0.0,1.0] / window > 0)"),
```

### 確認

```bash
cargo check --bin fav
```

---

## T5: `fav/src/driver.rs` — `cmd_explain_sla` + `v226000_tests`

### 事前確認コマンド

```bash
grep -n "pub fn cmd_explain_lineage\|// ── v22.5.0\|v225000_tests" fav/src/driver.rs | head -10
```

### 5-1: `cmd_explain_sla` を追加

`// ── v22.5.0: Pipeline Orchestration` ブロックの直前（または `cmd_explain_lineage` の直後）に追加:

```rust
// ── v22.6.0: fav explain --sla ───────────────────────────────────────────────

/// `fav explain --sla [file]` — stage の SLA アノテーションと最悪実行時間を出力する。
/// アノテーションなし stage も行に含めるが worst_case を `—` とし合計には含めない。
pub fn cmd_explain_sla(file: Option<&str>) {
    use crate::ast::Item;
    let src = match file {
        Some(f) => load_file(f),
        None => {
            eprintln!("error: fav explain --sla requires a file argument");
            process::exit(1);
        }
    };
    let file_name = file.unwrap_or("<stdin>");
    let prog = Parser::parse_str(&src, file_name).unwrap_or_else(|e| {
        eprintln!("{}", e);
        process::exit(1);
    });

    let separator = "━".repeat(60);
    println!("SLA Summary — {}", file_name);
    println!("{}", separator);
    println!("{:<20} {:<10} {:<8} {:<20} {}", "stage", "timeout", "retry", "circuit_breaker", "worst_case");

    let mut total_worst_secs = 0.0f64;
    let mut found_any = false;

    for item in &prog.items {
        if let Item::TrfDef(td) = item {
            let timeout_str = td.timeout.as_ref()
                .map(|t| format!("{}s", t.seconds))
                .unwrap_or_else(|| "—".to_string());
            let retry_str = td.retry_ann.as_ref()
                .map(|r| format!("{}×", r.max))
                .unwrap_or_else(|| "—".to_string());
            let cb_str = td.circuit_breaker.as_ref()
                .map(|cb| format!("{}/{}", cb.threshold, cb.window))
                .unwrap_or_else(|| "—".to_string());

            // 最悪実行時間: timeout_secs * retry.max（アノテーションなしは加算しない）
            let worst: Option<f64> = match (&td.timeout, &td.retry_ann) {
                (Some(t), Some(r)) => Some(t.seconds * r.max as f64),
                (Some(t), None)    => Some(t.seconds),
                _                  => None,
            };
            let worst_str = worst.map(|s| format!("{:.0}s", s)).unwrap_or_else(|| "—".to_string());

            println!("{:<20} {:<10} {:<8} {:<20} {}", td.name, timeout_str, retry_str, cb_str, worst_str);
            if let Some(w) = worst {
                total_worst_secs += w;
            }
            found_any = true;
        }
    }

    println!("{}", separator);
    if !found_any {
        println!("(no stage definitions found)");
    } else {
        println!("Total worst-case (SLA-annotated stages only): {:.0}s", total_worst_secs);
    }
}
```

### 5-2: `v225000_tests::version_is_22_5_0` に `#[ignore]` を追加

### 5-3: `v226000_tests` モジュールを追加（8 テスト）

```rust
// ── v226000_tests (v22.6.0) — SLA Annotations ────────────────────────────────
#[cfg(test)]
mod v226000_tests {
    use super::*;

    #[test]
    fn version_is_22_6_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("version = \"22.6.0\""), "Cargo.toml should have version 22.6.0");
    }

    #[test]
    fn timeout_annotation_parsed() {
        let src = "#[timeout(seconds = 30)]\nstage Fetch: String -> String = |url| { url }";
        let tokens = crate::frontend::lexer::Lexer::new(src, "test.fav")
            .tokenize().expect("lex failed");
        let prog = crate::frontend::parser::Parser::new(tokens)
            .parse_program().expect("parse failed");
        assert_eq!(prog.items.len(), 1);
        if let crate::ast::Item::TrfDef(td) = &prog.items[0] {
            let t = td.timeout.as_ref().expect("timeout annotation should be present");
            assert!((t.seconds - 30.0).abs() < 1e-9, "seconds should be 30.0, got {}", t.seconds);
        } else {
            panic!("expected TrfDef");
        }
    }

    #[test]
    fn retry_annotation_parsed() {
        let src = "#[retry(max = 3, backoff = \"exponential\")]\nstage Call: String -> String = |s| { s }";
        let tokens = crate::frontend::lexer::Lexer::new(src, "test.fav")
            .tokenize().expect("lex failed");
        let prog = crate::frontend::parser::Parser::new(tokens)
            .parse_program().expect("parse failed");
        if let crate::ast::Item::TrfDef(td) = &prog.items[0] {
            let r = td.retry_ann.as_ref().expect("retry annotation should be present");
            assert_eq!(r.max, 3);
            assert_eq!(r.backoff, "exponential");
        } else {
            panic!("expected TrfDef");
        }
    }

    #[test]
    fn circuit_breaker_annotation_parsed() {
        let src = "#[circuit_breaker(threshold = 0.5, window = 60)]\nstage CB: String -> String = |s| { s }";
        let tokens = crate::frontend::lexer::Lexer::new(src, "test.fav")
            .tokenize().expect("lex failed");
        let prog = crate::frontend::parser::Parser::new(tokens)
            .parse_program().expect("parse failed");
        if let crate::ast::Item::TrfDef(td) = &prog.items[0] {
            let cb = td.circuit_breaker.as_ref().expect("circuit_breaker annotation should be present");
            assert!((cb.threshold - 0.5).abs() < 1e-9, "threshold should be 0.5");
            assert_eq!(cb.window, 60);
        } else {
            panic!("expected TrfDef");
        }
    }

    #[test]
    fn sla_invalid_timeout_checker_err() {
        let src = "#[timeout(seconds = 0)]\nstage Bad: String -> String = |s| { s }";
        let tokens = crate::frontend::lexer::Lexer::new(src, "test.fav")
            .tokenize().expect("lex failed");
        let prog = crate::frontend::parser::Parser::new(tokens)
            .parse_program().expect("parse failed");
        let (errs, _) = crate::middle::checker::Checker::check_program(&prog);
        assert!(errs.iter().any(|e| e.code == "E0401"), "expected E0401, got: {:?}", errs);
    }

    #[test]
    fn sla_invalid_retry_checker_err() {
        let src = "#[retry(max = 0, backoff = \"none\")]\nstage Bad: String -> String = |s| { s }";
        let tokens = crate::frontend::lexer::Lexer::new(src, "test.fav")
            .tokenize().expect("lex failed");
        let prog = crate::frontend::parser::Parser::new(tokens)
            .parse_program().expect("parse failed");
        let (errs, _) = crate::middle::checker::Checker::check_program(&prog);
        assert!(errs.iter().any(|e| e.code == "E0402"), "expected E0402, got: {:?}", errs);
    }

    #[test]
    fn sla_invalid_circuit_breaker_checker_err() {
        let src = "#[circuit_breaker(threshold = 0.0, window = 60)]\nstage Bad: String -> String = |s| { s }";
        let tokens = crate::frontend::lexer::Lexer::new(src, "test.fav")
            .tokenize().expect("lex failed");
        let prog = crate::frontend::parser::Parser::new(tokens)
            .parse_program().expect("parse failed");
        let (errs, _) = crate::middle::checker::Checker::check_program(&prog);
        assert!(errs.iter().any(|e| e.code == "E0403"), "expected E0403, got: {:?}", errs);
    }

    #[test]
    fn changelog_has_v22_6_0() {
        let cl = include_str!("../../CHANGELOG.md");
        assert!(cl.contains("[v22.6.0]"), "CHANGELOG should have v22.6.0 entry");
    }
}
```

> **注意**: テストで使用する checker API は `crate::middle::checker::Checker::check_program(&prog)` であり、戻り値は `(Vec<TypeError>, Vec<FavWarning>)` 形式。`TypeError.code` の型（`&'static str` か `String`）を事前に確認してから `e.code == "E0401"` を書く。

### 確認

```bash
cargo test v226000 --bin fav   # 8/8 PASS を確認
cargo test --bin fav           # リグレッションなし（1865 件以上）確認
```

---

## T6: `fav/src/main.rs` — `fav explain --sla` フラグ対応

### 事前確認コマンド

```bash
grep -n "\"--lineage\"\|cmd_explain_lineage\|Some(\"explain\")" fav/src/main.rs | head -10
```

### 6-1: `fav explain --sla` 処理を追加

`Some("explain")` ブランチ内の `--lineage` チェックの直後に追加:

```rust
if args.iter().any(|a| a == "--sla") {
    let file = args.iter().skip(2).find(|a| !a.starts_with('-')).map(|s| s.as_str());
    crate::driver::cmd_explain_sla(file);
    return;
}
```

### 6-2: ヘルプテキスト更新（任意）

`main.rs` のヘルプテキストに `explain --sla [file]` を追加する。

### 確認

```bash
cargo check --bin fav
```

---

## T7: `fav/Cargo.toml` + `CHANGELOG.md` + MDX + benchmarks

### 7-1: バージョン更新

```
version = "22.5.0" → "22.6.0"
```

### 7-2: CHANGELOG に v22.6.0 エントリを先頭に追加

```markdown
## [v22.6.0] — 2026-06-21 — SLA 宣言（タイムアウト・リトライ・サーキットブレーカー）

### 追加

- `#[timeout(seconds = N)]` アノテーション — stage の最大実行時間を宣言（`TimeoutAnnotation`）
- `#[retry(max = N, backoff = "...")]` アノテーション — リトライ戦略を宣言（`RetryAnnotation`、`backoff` は `exponential`/`linear`/`none`）
- `#[circuit_breaker(threshold = F, window = N)]` アノテーション — 障害率閾値を宣言（`CircuitBreakerAnnotation`）
- E0401: `#[timeout]` の `seconds` が 0 以下の場合にコンパイルエラー
- E0402: `#[retry]` の `max` が 0 以下、または `backoff` が不正な場合にコンパイルエラー
- E0403: `#[circuit_breaker]` の `threshold` が範囲外、または `window` が 0 以下の場合にコンパイルエラー
- `fav explain --sla [file]` — SLA アノテーション一覧と最悪実行時間を出力
- `error_catalog.rs` に E0401〜E0403 エントリ追加
```

### 7-3: `site/content/docs/cli/sla.mdx` を新規作成

内容:
- `#[timeout]` / `#[retry]` / `#[circuit_breaker]` の構文と各パラメータの説明
- E0401〜E0403 のトリガー条件と対処法
- `fav explain --sla` の出力例
- 実行時適用（v22.7+ 予定）への言及
- `fn` 定義への SLA アノテーションはサイレントに無視される旨の注意書き

### 7-4: `benchmarks/v22.6.0.json` を作成

既存の `benchmarks/v22.5.0.json` を参照してフォーマットを合わせ、v22.6.0 の内容で作成する。

---

## 主要な落とし穴・注意事項

1. **`check_trf_def` vs `check_item`**: `check_item` の `TrfDef` アームは `self.check_trf_def(td)` に委譲している。バリデーションは `check_trf_def()` 末尾に追加すること。`check_item` に直接書いてはならない。

2. **`self.type_error` の使用**: checker 内では `CheckError::new` や `self.errors.push` は使わない。`self.type_error(code, message, &span)` で追加する。引数の型を `grep -n "fn type_error\|self\.type_error" fav/src/middle/checker.rs | head -5` で確認してから使う。

3. **`Checker::check_program` API**: テストでの checker 呼び出しは `crate::middle::checker::Checker::check_program(&prog)` 形式。戻り値は `(Vec<TypeError>, Vec<FavWarning>)` でデストラクチャする。`TypeError.code` が `&'static str` / `String` のどちらかを確認すること。

4. **`#` + `[` の消費**: 既存の `parse_checkpoint_annotation` / `parse_trigger_annotation` に合わせて、`#` は `self.advance()`、`[` は `self.expect(&TokenKind::LBracket)?` で消費する（`advance()` ではエラー検出できない）。

5. **`TrfDef` struct literal の多重修正**: `parser.rs` 内で `TrfDef { ... }` を構築している箇所を `cargo check` でリストアップし、漏れなく修正する。

6. **アノテーション順序は不問**: ユーザーが `#[retry]` を `#[timeout]` の前に書いても正しく動作する（各パーサーが独立して lookahead チェックするため）。

7. **`fn` 定義への SLA アノテーション**: `parse_item` は `FnDef` が来ても `timeout_ann` を消費せずに進む。その結果アノテーションは読み飛ばされてサイレントに無視される。意図的な仕様（E0404 は v22.7+ 予定）。
