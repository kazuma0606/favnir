# v61.5.0 Plan — 文字列補間強化（ネスト呼び出し・マルチライン `f"""..."""`）

Date: 2026-08-01
Status: REVIEWED

---

## 実装順序

AST 追加 → Lexer 追加 → コンパイルエラー駆動で全 Expr::FString match 箇所を修正 → パーサー更新 → fmt.rs 更新 → テスト

---

## Phase 1: AST 変更（`ast.rs`）

### P1: `Expr::FString` に `multiline: bool` フラグ追加

`FString(Vec<FStringPart>, Span)` を以下に変更:
```rust
/// String interpolation (f"..." or f"""...""").
/// multiline = true if the source used triple-quote syntax. (v61.5.0)
FString(Vec<FStringPart>, bool /* multiline */, Span),
```

`span()` メソッドの match を更新:
```rust
Expr::FString(_, _, s) => s,
```

この変更でコンパイルエラーが多数発生 → Phase 2 で解消。

---

## Phase 2: コンパイルエラー修正（各 `Expr::FString` match 箇所）

`cargo build` でエラー一覧を確認後、exhaustive match が壊れた箇所を順次修正。
基本方針: `multiline` フラグを使わない箇所はすべて `_` で受け流す。

### P2-1: `lint.rs` — 9 箇所を更新

`grep -n 'Expr::FString' fav/src/lint.rs` で行番号を確認し、全 9 箇所を更新:
- `Expr::FString(parts, _)` → `Expr::FString(parts, _, _)`

### P2-2: `lineage.rs` — 5 箇所を更新

```rust
// 更新前
ast::Expr::Lit(_, _) | ast::Expr::Ident(_, _) | ast::Expr::FString(_, _) => {}
// 更新後
ast::Expr::Lit(_, _) | ast::Expr::Ident(_, _) | ast::Expr::FString(_, _, _) => {}
```

5 箇所すべて同様に更新。

### P2-3: `checker.rs` — 3 箇所を更新

```rust
// 更新前例
Expr::FString(parts, _) => { ... }
// 更新後例
Expr::FString(parts, _, _) => { ... }
```

3 箇所すべて同様に更新。span を使う箇所は `span` を第 3 要素で受ける。

### P2-4: `compiler.rs` — 2 箇所を更新

```rust
Expr::FString(parts, _) => compile_fstring(parts, ctx),
// →
Expr::FString(parts, _, _) => compile_fstring(parts, ctx),
```

### P2-5: `ast_lower_checker.rs` — 1 箇所を更新

```rust
ast::Expr::FString(parts, _) => lower_fstring(parts),
// →
ast::Expr::FString(parts, _, _) => lower_fstring(parts),
```

### P2-6: `emit_python.rs` — 1 箇所を更新

```rust
Expr::FString(parts, _) => { ... }
// →
Expr::FString(parts, _, _) => { ... }
```

### P2-7: `driver.rs` — 2 箇所を更新

```rust
// remap_ir_expr または walk 関数内
Expr::FString(parts, _) => { ... }
// →
Expr::FString(parts, _, _) => { ... }
```

### P2-8: `lsp/references.rs` — 1 箇所を更新

```rust
Expr::FString(parts, _) => { ... }
// →
Expr::FString(parts, _, _) => { ... }
```

### P2-9: `parser.rs` — 既存 `Expr::FString(parts, base_span)` 生成箇所（スタブ）

`parse_fstring_parts` の戻り値を仮の `false` で通過させる:
```rust
Ok(Expr::FString(parts, false, base_span))  // multiline は P3 で正しく設定
```

既存テストコード内の `Expr::FString(parts, _)` も `Expr::FString(parts, _, _)` に更新。

### P2-10: コンパイル確認

- `cargo build` でコンパイルエラーがないことを確認

---

## Phase 3: Lexer 追加（`lexer.rs`）

### P3-1: `FStringTripleRaw(String)` トークン追加

`TokenKind` 定義の `FStringRaw(String)` の直後に追加:
```rust
/// Triple-quote f-string raw content: f"""...""" (v61.5.0)
FStringTripleRaw(String),
```

**注**: `TokenKind` を使う全ての `match` を確認。`FStringTripleRaw` はパーサーと一部の fmt テスト以外では `_ =>` アームに落ちるため、基本的に追加変更不要。ビルドエラーで確認。

### P3-2: triple-quote 分岐を `FStringTripleRaw` に変更

```rust
'f' if self.peek2() == Some('"') && self.peek3() == Some('"') => {
    self.advance(); // 'f'
    self.advance(); // '"' (1st)
    self.advance(); // '"' (2nd)
    self.advance(); // '"' (3rd)
    TokenKind::FStringTripleRaw(self.lex_fstring_triple(sp, sl, sc)?)
}
```

- `cargo build` でコンパイルエラーがないことを確認

---

## Phase 4: パーサー更新（`parser.rs`）

### P4-1: `parse_fstring_parts` シグネチャ変更

`multiline: bool` 引数を追加し、戻り値の `Expr::FString` 生成を更新:
```rust
fn parse_fstring_parts(
    &mut self,
    raw: &str,
    base_span: Span,
    multiline: bool,
) -> Result<Expr, ParseError> {
    // ...（既存の解析ロジックはそのまま）
    Ok(Expr::FString(parts, multiline, base_span))
}
```

### P4-2: `FStringRaw` アームを `multiline=false` に更新

```rust
TokenKind::FStringRaw(raw) => {
    self.advance();
    self.parse_fstring_parts(&raw, start, false)
}
```

### P4-3: `FStringTripleRaw` アームを新規追加

`FStringRaw` アームの直後に追加:
```rust
TokenKind::FStringTripleRaw(raw) => {
    self.advance();
    self.parse_fstring_parts(&raw, start, true)
}
```

- `cargo build` でコンパイルエラーがないことを確認

---

## Phase 5: `fmt.rs` 更新

### P5: `Expr::FString` の multiline 分岐

既存の `Expr::FString(parts, _)` アームを以下に置き換え:

```rust
Expr::FString(parts, multiline, _) => {
    if *multiline {
        let mut out = String::from("f\"\"\"");
        for part in parts {
            match part {
                FStringPart::Lit(s) => out.push_str(s),
                FStringPart::Expr(expr) => {
                    out.push('{');
                    out.push_str(&self.expr(expr));
                    out.push('}');
                }
            }
        }
        out.push_str("\"\"\"");
        out
    } else {
        let mut out = String::from("f\"");
        for part in parts {
            match part {
                FStringPart::Lit(s) => out.push_str(&fmt_fstring_lit(s)),
                FStringPart::Expr(expr) => {
                    out.push('{');
                    out.push_str(&self.expr(expr));
                    out.push('}');
                }
            }
        }
        out.push('"');
        out
    }
}
```

**注**: 非 multiline の出力形式を `$"..."` から `f"..."` に変更する。
lexer は `$"..."` も `f"..."` も同じ `FStringRaw` として受け入れるため、
`fav fmt` 後は `f"..."` に正規化される（破壊的変更ではない）。

- `cargo build` でコンパイルエラーがないことを確認

---

## Phase 6: テスト追加（`driver.rs`）

### P6: `v61500_tests` モジュール追加

`v61400_tests` の直前（上側）に挿入:

```rust
// -- v61500_tests (v61.5.0) -- f-string 強化 --
#[cfg(test)]
mod v61500_tests {
    use super::*;

    /// f-string 内のネストした関数呼び出し・フィールドアクセスが正しくパースされることを確認
    #[test]
    fn fstring_nested_call() {
        let src = concat!(
            "type User = { name: String score: Int }\n",
            "fn greet(user: User) -> String {\n",
            "  f\"hello {user.name} score={Int.to_string(user.score)}\"\n",
            "}\n",
        );
        let prog = Parser::parse_str(src, "test.fav").expect("parse failed");
        let (errors, _) = crate::middle::checker::Checker::check_program(&prog);
        assert!(
            errors.is_empty(),
            "fstring with nested field access and function call should type-check; errors: {:?}",
            errors
        );
    }

    /// f"""...""" マルチライン文字列補間が parse + type-check を通過することを確認
    #[test]
    fn fstring_multiline() {
        let src = concat!(
            "type Report = { name: String total: Int }\n",
            "fn summarize(r: Report) -> String {\n",
            "  f\"\"\"\n",
            "  Summary for {r.name}:\n",
            "  - Total: {Int.to_string(r.total)}\n",
            "  \"\"\"\n",
            "}\n",
        );
        let prog = Parser::parse_str(src, "test.fav").expect("parse failed");
        let (errors, _) = crate::middle::checker::Checker::check_program(&prog);
        assert!(
            errors.is_empty(),
            "multiline fstring should type-check without errors; errors: {:?}",
            errors
        );
    }
}
```

---

## Phase 7: テスト実行・確認

- `cargo test -j 8 -- --test-threads=8`
- `v61500_tests::fstring_nested_call` pass
- `v61500_tests::fstring_multiline` pass
- 既存 fstring テスト（`test_parse_fstring_simple` 等）が引き続き pass
- 総テスト数 **3367** tests passed, 0 failed

---

## Phase 8: 事後処理

- `versions/current.md` を v61.5.0 / 3367 tests に更新
- `versions/roadmap/roadmap-v61.1-v62.0.md` の v61.5.0 実績欄を更新
- このファイル（plan.md）と tasks.md を COMPLETE に更新

---

## リスク・注意事項

1. **`TokenKind` の網羅的 match**: `FStringTripleRaw` を追加すると、`TokenKind` をパターンマッチするすべての箇所でビルドエラーが発生する可能性がある。ほとんどは `_ =>` アームがあるため問題ないが、`fmt_token` や `Display` 等の完全 match 箇所は要確認。

2. **`fmt.rs` の `$"..."` → `f"..."` 変更**: 非 multiline の出力形式が変わる。既存テストで `$"..."` の出力を期待しているものがあれば更新が必要。`grep -n 'fmt.*fstring\|fstring.*fmt\|\$\"' fav/src/` で確認。

3. **`parse_fstring_parts` のネスト式**: `{user.name}` のフィールドアクセスや `{Int.to_string(x)}` の関数呼び出しは `parse_str_expr` → `parse_expr()` で既に処理されるため動作するはず。ただし `{f"inner"}` のようなネスト f-string は今回のスコープ外。

4. **lexer の `peek2` / `peek3`**: `peek3` は既存実装に存在する（`lex_fstring_triple` の条件で使用中）。確認不要。

5. **テストの `f\"\"\"` エスケープ**: Rust の raw string は `concat!` 内では使いにくいため、`\"\"\"` で `"""` をエスケープする。

6. **lineage.rs の複数パターン OR**: `Lit(_, _) | Ident(_, _) | FString(_, _)` の形式で書かれているため、`FString(_, _, _)` への変更はすべての行で機械的に対応。
