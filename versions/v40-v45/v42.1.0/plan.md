# v42.1.0 実装プラン — CEP DSL 基盤

**フェーズ**: Real-Time Power（v42.x スプリント）
**目標テスト数**: 2877（+3）

---

## ステップ概要

1. `fav/src/ast.rs` — `CepClause` / `CepPatternDef` 構造体 + `Item::CepPatternDef` バリアント + `span()` arm 追加
2. `fav/src/frontend/parser.rs` — `parse_cep_pattern_def()` + `"cep"` dispatch 追加
3. `fav/src/middle/checker.rs` — 2 箇所のスタブ追加
4. `fav/src/fmt.rs` — exhaustive match arm 追加
5. `fav/src/driver.rs` — exhaustive match スタブ + `v42100_tests` 3 件追加
6. `fav/self/checker.fav` — CEP 設計コメント追加
7. `fav/Cargo.toml` — version bump（42.0.0 → 42.1.0）
8. `CHANGELOG.md` — `[v42.1.0]` エントリ追加
9. `cargo test` — 2877 passed / 0 failed を確認

---

## Step 1: ast.rs 更新

`SchemaDef` の直後（行 906 付近）に追加:

```rust
// ── CepPatternDef (v42.1.0) ──────────────────────────────────────────────────

/// 単一イベント節: `Login within 60`
#[derive(Debug, Clone)]
pub struct CepClause {
    pub event: String,            // イベント名 ("Login")
    pub within_secs: Option<i64>, // `within N` 秒 (Some(60) or None)
    pub span: Span,
}

/// `cep pattern Name { clause... }` — CEP パターン宣言 (v42.1.0)
#[derive(Debug, Clone)]
pub struct CepPatternDef {
    pub name: String,
    pub body: Vec<CepClause>,
    pub span: Span,
}
```

`Item` enum に追加（`SchemaDef` の直後）:
```rust
/// `cep pattern Name { ... }` — CEP パターン宣言 (v42.1.0)
CepPatternDef(CepPatternDef),
```

`Item::span()` の exhaustive match に追加（`Item::SchemaDef(s) => &s.span` の直後）:
```rust
Item::CepPatternDef(c) => &c.span,
```

---

## Step 2: parser.rs 更新

### 2a. `parse_item()` に `"cep"` ディスパッチ追加

`"schema"` の dispatch ブロックの直前（または直後）に挿入:

```rust
// `cep pattern Name { ... }` — CEP パターン宣言 (v42.1.0)
TokenKind::Ident(n) if n == "cep" => {
    Ok(Item::CepPatternDef(self.parse_cep_pattern_def()?))
}
```

エラーメッセージの末尾リスト（`"...schema"` の後）に `"/cep"` を追加:
```rust
"expected item (...schema/cep), got {:?}", other
```

### 2b. `parse_cep_pattern_def()` 関数追加

`parse_schema_def()` の直後に追加。

**パーサー API の実態（確認済み）:**
- `peek()` → `&TokenKind`（`Some` ラップなし）
- `advance()` → `&Token`（`Option` ではない）
- `span_from(&start)` で Span 生成（`Span::merge()` は存在しない）
- `peek_ident_text("word")` で "word" という識別子かチェック
- `expect_ident_name("word")` で "word" ident を consume（失敗時 ParseError）

```rust
/// Parse `cep pattern Name { Event within N }` (v42.1.0)
fn parse_cep_pattern_def(&mut self) -> Result<CepPatternDef, ParseError> {
    let start = self.peek_span().clone();
    self.advance(); // consume `cep`
    self.expect_ident_name("pattern")?; // expect `pattern` keyword
    let (name, _) = self.expect_ident()?;
    self.expect(&TokenKind::LBrace)?;
    let mut body = Vec::new();
    while self.peek() != &TokenKind::RBrace && self.peek() != &TokenKind::Eof {
        let clause_start = self.peek_span().clone();
        let (event, _) = self.expect_ident()?;
        let within_secs = if self.peek_ident_text("within") {
            self.advance(); // consume `within`
            match self.peek().clone() {
                TokenKind::Int(n) => {
                    self.advance();
                    Some(n)
                }
                _ => return Err(ParseError::new(
                    "expected integer after `within`",
                    self.peek_span().clone(),
                )),
            }
        } else {
            None
        };
        body.push(CepClause {
            event,
            within_secs,
            span: self.span_from(&clause_start),
        });
    }
    self.expect(&TokenKind::RBrace)?;
    Ok(CepPatternDef { name, body, span: self.span_from(&start) })
}
```

---

## Step 3: checker.rs スタブ

### 3a. Pass 1（グローバルシンボル収集）— line 2368 付近

```rust
| Item::SchemaDef(..)
| Item::CepPatternDef(..) => {} // v42.1.0: スタブ（型チェックは v42.3.0）
```

### 3b. Pass 2（型チェック）— line 2411 付近

```rust
Item::SchemaDef(_) => {} // v36.1.0: 型チェックは v36.2 以降
Item::CepPatternDef(_) => {} // v42.1.0: 型チェックは v42.3.0
```

---

## Step 4: fmt.rs スタブ

`Item::SchemaDef` arm の直後に追加:

```rust
Item::CepPatternDef(cd) => format!("cep pattern {} {{ ... }}", cd.name), // v42.1.0: fmt スタブ
```

---

## Step 5: driver.rs 更新

### 5a. exhaustive match スタブ（line 13898 付近）

`Item::SchemaDef(..) => {} // v36.1.0: スタブ` の直後に追加:

```rust
Item::CepPatternDef(..) => {} // v42.1.0: スタブ
```

### 5b. `v42000_tests` スタブ化 + `v42100_tests` 追加

`v42000_tests::cargo_toml_version_is_42_0_0` スタブ化:
```rust
fn cargo_toml_version_is_42_0_0() {
    // Stubbed: version bumped to 42.1.0 -- assertion intentionally removed
}
```

`v42100_tests` モジュール追加（`v42000_tests` の直前）:
```rust
// -- v42100_tests (v42.1.0) -- CEP DSL 基盤 --
#[cfg(test)]
mod v42100_tests {
    #[test]
    fn cargo_toml_version_is_42_1_0() {
        // NOTE: この assert は次バージョン bump 時にスタブ化すること
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("42.1.0"), "Cargo.toml must contain version 42.1.0");
    }

    #[test]
    fn cep_pattern_parseable() {
        use crate::frontend::parser::Parser;
        let src = r#"cep pattern LoginEvent { Login within 60 }"#;
        let result = Parser::parse_str(src, "test.fav");
        assert!(result.is_ok(), "cep pattern should parse without error: {:?}", result.err());
    }

    #[test]
    fn cep_pattern_fields_correct() {
        use crate::frontend::parser::Parser;
        use crate::ast::Item;
        let src = r#"cep pattern LoginEvent { Login within 60 }"#;
        let prog = Parser::parse_str(src, "test.fav").expect("parse ok");
        let Item::CepPatternDef(ref cd) = prog.items[0] else {
            panic!("expected CepPatternDef");
        };
        assert_eq!(cd.name, "LoginEvent");
        assert_eq!(cd.body.len(), 1);
        assert_eq!(cd.body[0].event, "Login");
        assert_eq!(cd.body[0].within_secs, Some(60));
    }
}
```

---

## Step 6: checker.fav 設計コメント

`fav/self/checker.fav` の末尾に追加:
```
// ── CEP パターン型チェック（v42.3.0 以降）─────────────────────────────────────
// v42.1.0 では CepPatternDef は AST ノードとしてパースのみ。
// v42.3.0 で以下を実装予定:
//   - pattern ブロック内のイベント名が型環境に存在するか検証
//   - within_secs が正の整数か検証
//   - E0420: CEP パターンの型不一致エラー
```

---

## Step 7: Cargo.toml バージョン bump

```toml
version = "42.1.0"
```

---

## Step 8: CHANGELOG.md 更新

`[v42.0.0]` の直前に追加:

```markdown
## [v42.1.0] — 2026-07-12

### Added
- CEP DSL 基盤: `CepPatternDef` / `CepClause` AST ノード追加
- parser: `cep pattern Name { Event within N }` 構文対応
- checker.rs / fmt.rs / driver.rs: `CepPatternDef` スタブ追加（型チェックは v42.3.0）
- checker.fav: CEP 型チェック設計コメント追加（v42.3.0 向け）
- driver.rs `v42100_tests` 3 件追加（`cargo_toml_version_is_42_1_0` / `cep_pattern_parseable` / `cep_pattern_fields_correct`）

### Changed
- `fav/Cargo.toml`: version `42.0.0` → `42.1.0`
```

---

## 注意事項

- `peek()` は `&TokenKind` を返す（`Some(...)` でラップされていない）
- `advance()` は `&Token` を返す（`Option` ではない）。整数値の取得は `peek().clone()` でパターンマッチしてから `advance()` する
- `Span::merge()` は存在しない。`self.span_from(&start)` を使う（`parse_schema_def` と同じパターン）
- `TokenKind::Int(i64)` が整数リテラルトークン（確認済み）
- exhaustive-match-checker の対象外ファイル（`lineage.rs`/`lint.rs`/`emit_python.rs`）は `if let` / `matches!` で `CepPatternDef` を参照しないため更新不要
