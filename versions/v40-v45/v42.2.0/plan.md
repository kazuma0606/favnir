# v42.2.0 実装計画 — CEP パターン: `seq` / `any` / `not`

## T0 — 事前確認

1. `cargo test` → 2877 passed, 0 failed を確認
2. `fav/Cargo.toml` version が `"42.1.0"` であることを確認
3. `v42100_tests::cargo_toml_version_is_42_1_0` が NOTE コメント付きライブアサーションであることを確認し行番号を記録
4. `v42100_tests` の閉じ `}` の行番号を確認し記録（`v42200_tests` 挿入位置のため）
5. `ast.rs` に `CepExpr` が存在しないことを確認
6. `CepClause` が `event: String` フィールドを持つことを確認
7. `parser.rs` に `parse_cep_expr` が存在しないことを確認

---

## T1 — `ast.rs` 更新

**`CepExpr` enum を `CepClause` の直前に追加**:

```rust
/// CEP パターン式 (v42.2.0)
#[derive(Debug, Clone)]
pub enum CepExpr {
    /// 単純イベント名: `Login`
    Event(String),
    /// 順序結合: `seq(Login, Purchase)`
    Seq(Vec<CepExpr>),
    /// 選択: `any(DiskFull, OOM, NetworkDown)`
    Any(Vec<CepExpr>),
    /// 否定: `not(Login)`
    Not(Box<CepExpr>),
}
```

**`CepClause` の `event: String` を `expr: CepExpr` に変更**:

```rust
pub struct CepClause {
    pub expr: CepExpr,          // v42.2.0: event: String → expr: CepExpr
    pub within_secs: Option<i64>,
    pub span: Span,
}
```

---

## T2 — `parser.rs` 更新

### 2-A: `parse_cep_expr()` を `parse_cep_pattern_def()` の直前に追加

```rust
/// Parse a CEP expression: Event name, seq(...), any(...), or not(...) (v42.2.0)
fn parse_cep_expr(&mut self) -> Result<CepExpr, ParseError> {
    if self.peek_ident_text("seq") {
        self.advance(); // consume `seq`
        self.expect(&TokenKind::LParen)?;
        let mut args = Vec::new();
        while self.peek() != &TokenKind::RParen && self.peek() != &TokenKind::Eof {
            args.push(self.parse_cep_expr()?);
            if self.peek() == &TokenKind::Comma {
                self.advance();
            }
        }
        self.expect(&TokenKind::RParen)?;
        return Ok(CepExpr::Seq(args));
    }
    if self.peek_ident_text("any") {
        self.advance(); // consume `any`
        self.expect(&TokenKind::LParen)?;
        let mut args = Vec::new();
        while self.peek() != &TokenKind::RParen && self.peek() != &TokenKind::Eof {
            args.push(self.parse_cep_expr()?);
            if self.peek() == &TokenKind::Comma {
                self.advance();
            }
        }
        self.expect(&TokenKind::RParen)?;
        return Ok(CepExpr::Any(args));
    }
    if self.peek_ident_text("not") {
        self.advance(); // consume `not`
        self.expect(&TokenKind::LParen)?;
        let inner = self.parse_cep_expr()?;
        self.expect(&TokenKind::RParen)?;
        return Ok(CepExpr::Not(Box::new(inner)));
    }
    // Simple event name
    let (name, _) = self.expect_ident()?;
    Ok(CepExpr::Event(name))
}
```

### 2-B: `parse_cep_pattern_def()` の節ループを修正

変更前:
```rust
let (event, _) = self.expect_ident()?;
// ... within_secs ...
body.push(CepClause {
    event,
    within_secs,
    span: self.span_from(&clause_start),
});
```

変更後:
```rust
// clause_start は既存コードの定義位置（ループ先頭の `let clause_start = self.peek_span().clone();`）をそのまま使用
let expr = self.parse_cep_expr()?;
// ... within_secs (変更なし) ...
body.push(CepClause {
    expr,
    within_secs,
    span: self.span_from(&clause_start),
});
```

---

## T3 — `driver.rs` 更新

### 3-A: `v42100_tests::cargo_toml_version_is_42_1_0` をスタブ化

（T0 で確認した行番号を使用）

### 3-B: `v42100_tests::cep_pattern_fields_correct` を AST 変更に合わせて更新

変更前（コンパイルエラーになる部分）:
```rust
assert_eq!(cd.body[0].event, "Login");
```

変更後:
```rust
let crate::ast::CepExpr::Event(ref ev) = cd.body[0].expr else {
    panic!("expected CepExpr::Event");
};
assert_eq!(ev, "Login");
```

### 3-C: `v42200_tests` モジュールを `v42100_tests` の直前に追加

```rust
// -- v42200_tests (v42.2.0) -- CEP seq/any/not コンビネータ --
#[cfg(test)]
mod v42200_tests {
    #[test]
    fn cargo_toml_version_is_42_2_0() {
        // NOTE: この assert は次バージョン bump 時にスタブ化すること
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("42.2.0"), "Cargo.toml must contain version 42.2.0");
    }

    #[test]
    fn cep_seq_parseable() {
        use crate::frontend::parser::Parser;
        use crate::ast::{Item, CepExpr};
        let src = r#"cep pattern LoginThenPurchase { seq(Login, Purchase) within 300 }"#;
        let prog = Parser::parse_str(src, "test.fav").expect("parse ok");
        let Item::CepPatternDef(ref cd) = prog.items[0] else {
            panic!("expected CepPatternDef");
        };
        assert_eq!(cd.body.len(), 1);
        let CepExpr::Seq(ref args) = cd.body[0].expr else {
            panic!("expected CepExpr::Seq");
        };
        assert_eq!(args.len(), 2);
        assert_eq!(cd.body[0].within_secs, Some(300));
    }

    #[test]
    fn cep_any_parseable() {
        use crate::frontend::parser::Parser;
        use crate::ast::{Item, CepExpr};
        let src = r#"cep pattern AnyAlert { any(DiskFull, OOM, NetworkDown) }"#;
        let prog = Parser::parse_str(src, "test.fav").expect("parse ok");
        let Item::CepPatternDef(ref cd) = prog.items[0] else {
            panic!("expected CepPatternDef");
        };
        assert_eq!(cd.body.len(), 1);
        let CepExpr::Any(ref args) = cd.body[0].expr else {
            panic!("expected CepExpr::Any");
        };
        assert_eq!(args.len(), 3);
    }
}
```

---

## T4 — Cargo.toml バージョン bump

`version = "42.1.0"` → `"42.2.0"`

---

## T5 — CHANGELOG.md 更新

`[v42.2.0]` エントリを `[v42.1.0]` の直前に追加。

---

## T6 — `cargo test` 実行・確認

- 2880 passed, 0 failed を確認
- `v42200_tests` 3 件 pass を確認
- `v42100_tests::cep_pattern_fields_correct` が引き続き pass することを確認

---

## T7 — バージョン管理ドキュメント更新

- `versions/current.md` を v42.2.0（最新安定版）・v42.3.0（次に切る版）に更新
- `versions/roadmap/roadmap-v42.1-v43.0.md` の v42.2.0 を `✅ COMPLETE（2026-07-12）` にマーク
- `versions/v40-v45/v42.2.0/tasks.md` を COMPLETE ステータスに更新（全チェックボックス `[x]`）

---

## 注意事項

- `CepClause.event` フィールドの削除は **破壊的変更**。`driver.rs::cep_pattern_fields_correct` テストが即コンパイルエラーになる → T3-B で同時修正すること
- `parse_cep_expr()` の `seq` / `any` ループ: `while peek() != &RParen && peek() != &Eof` で無限ループを防ぐ
- `not` のテストは本バージョンでは追加しない（スコープ外）
