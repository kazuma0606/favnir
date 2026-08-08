# v61.5.0 Spec — 文字列補間強化（ネスト呼び出し・マルチライン `f"""..."""`）

Date: 2026-08-01
Status: REVIEWED

---

## 概要

既存の `FString` 実装を拡張して以下を実現する：

1. **ネストした式の明示的動作確認** — `{user.name}` / `{Float.format(score)}` のような関数呼び出し・フィールドアクセスが `{...}` 内で正しくパースされることをテストで保証する
2. **マルチライン f-string の AST 区別** — `f"""..."""` と `f"..."` を明示的に区別できるよう `Expr::FString` に `multiline: bool` フラグを追加する
3. **`fmt.rs` のマルチライン整形** — multiline フラグが真の場合は `f"""..."""` 形式で出力し、インデントを保持する整形ルールを追加する

```favnir
// ネストした式（既に動作するが E2E テストで確認）
bind msg <- f"user={user.name} score={Float.format(score)}"

// マルチライン f-string（新規対応）
bind report <- f"""
  Summary for {user.name}:
  - Total: {total}
  - Avg:   {avg}
"""
```

---

## 現状分析

### lexer.rs（現状）

- `lex_fstring_raw`: `f"..."` / `$"..."` → `TokenKind::FStringRaw(String)` を生成
- `lex_fstring_triple`: `f"""..."""` → **同じく** `TokenKind::FStringRaw(String)` を生成（区別なし）
- **問題**: 両方が同一トークン `FStringRaw` → パーサー・フォーマッタが区別不可能

### parser.rs（現状）

- `FStringRaw(raw)` → `parse_fstring_parts(&raw, start)` → `Expr::FString(parts, span)`
- `parse_fstring_parts` 内で `Parser::parse_str_expr(&expr_src, file)` を呼ぶ
- `parse_str_expr` は完全な `parse_expr()` を使用するため、フィールドアクセス・メソッドチェーン等のネスト式は**既に動作する**

### ast.rs（現状）

```rust
FString(Vec<FStringPart>, Span),  // multiline フラグなし
```

### fmt.rs（現状）

- 常に `$"..."` 形式で出力（`f"..."` / `f"""..."""` の区別なし）
- マルチライン内容があっても `$"..."` で出力されてしまう

---

## 実装スコープ（12 ファイル）

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `fav/src/frontend/lexer.rs` | 更新 | `FStringTripleRaw(String)` トークン追加、triple-quote をこのトークンに変更 |
| `fav/src/ast.rs` | 更新 | `Expr::FString(Vec<FStringPart>, bool, Span)` に `multiline` フラグ追加 |
| `fav/src/frontend/parser.rs` | 更新 | `FStringTripleRaw` アーム追加、`parse_fstring_parts` に `multiline` 引数追加 |
| `fav/src/fmt.rs` | 更新 | multiline フラグで `f"""..."""` 出力、インデント保持ロジック追加 |
| `fav/src/lint.rs` | 更新 | `Expr::FString(parts, _, _)` パターン更新（9 箇所） |
| `fav/src/lineage.rs` | 更新 | `Expr::FString(_, _)` → `Expr::FString(_, _, _)` パターン更新（5 箇所） |
| `fav/src/middle/checker.rs` | 更新 | `Expr::FString` パターン更新（4 箇所、`matches!` マクロ内 L5189 を含む） |
| `fav/src/middle/compiler.rs` | 更新 | `Expr::FString` パターン更新（2 箇所） |
| `fav/src/middle/ast_lower_checker.rs` | 更新 | `Expr::FString` パターン更新（1 箇所） |
| `fav/src/emit_python.rs` | 更新 | `Expr::FString` パターン更新（1 箇所） |
| `fav/src/driver.rs` | 更新 | `Expr::FString` パターン更新（3 箇所、OR パターン内 L15697 を含む）+ `v61500_tests` モジュール追加 |
| `fav/src/lsp/references.rs` | 更新 | `Expr::FString` パターン更新（1 箇所） |

新規ファイルなし。`vm.rs` / `codegen.rs` / `ir.rs` は変更不要（FString は compiler.rs でコンパイル済み）。

---

## Lexer 変更（`lexer.rs`）

### `TokenKind` に `FStringTripleRaw` 追加

`FStringRaw(String)` の直後に追加:
```rust
/// Triple-quote f-string raw content: f"""...""" (v61.5.0)
FStringTripleRaw(String),
```

### triple-quote 分岐を `FStringTripleRaw` に変更

```rust
'f' if self.peek2() == Some('"') && self.peek3() == Some('"') => {
    self.advance(); // 'f'
    self.advance(); // '"' (1st)
    self.advance(); // '"' (2nd)
    self.advance(); // '"' (3rd)
    TokenKind::FStringTripleRaw(self.lex_fstring_triple(sp, sl, sc)?)
}
```

これにより `FStringRaw` は `f"..."` / `$"..."` のみを表すようになる。

---

## AST 変更（`ast.rs`）

### `Expr::FString` に `multiline: bool` フラグ追加

```rust
/// String interpolation (f"..." or f"""...""").
/// multiline = true if the source used triple-quote syntax. (v61.5.0)
FString(Vec<FStringPart>, bool /* multiline */, Span),
```

`span()` メソッドの更新:
```rust
Expr::FString(_, _, s) => s,
```

---

## パーサー変更（`parser.rs`）

### `FStringTripleRaw` アーム追加

`FStringRaw` アームの直後に追加:
```rust
TokenKind::FStringTripleRaw(raw) => {
    self.advance();
    self.parse_fstring_parts(&raw, start, true)
}
```

既存の `FStringRaw` アームを `multiline=false` に更新:
```rust
TokenKind::FStringRaw(raw) => {
    self.advance();
    self.parse_fstring_parts(&raw, start, false)
}
```

### `parse_fstring_parts` シグネチャ変更

```rust
fn parse_fstring_parts(
    &mut self,
    raw: &str,
    base_span: Span,
    multiline: bool,   // v61.5.0
) -> Result<Expr, ParseError> {
    // ...（既存の解析ロジックはそのまま）
    Ok(Expr::FString(parts, multiline, base_span))
}
```

---

## フォーマッタ変更（`fmt.rs`）

### `Expr::FString(parts, multiline, _)` の分岐

```rust
Expr::FString(parts, multiline, _) => {
    if *multiline {
        // f"""...""" 形式（インデント保持）
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
        // f"..." 形式（従来 $"..." だったが f"..." に統一）
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

**注**: `fmt_fstring_lit` は既存関数（L988）で `{` / `}` をエスケープする。
multiline の lit 部分はエスケープ不要なためそのまま push_str する。

---

## その他ファイルの機械的更新

### パターンマッチ更新方針

以下のファイルでは `Expr::FString(parts, _)` が `Expr::FString(parts, _, _)` に、
`Expr::FString(_, _)` が `Expr::FString(_, _, _)` に変わるだけの機械的更新。
`multiline` フラグを使う必要はなく、`_` で受け流せばよい。

| ファイル | 変更箇所数 | 対応 |
|---|---|---|
| `lint.rs` | 9 箇所 | `(parts, _)` → `(parts, _, _)` |
| `lineage.rs` | 5 箇所 | `(_, _)` → `(_, _, _)` |
| `checker.rs` | **4 箇所** | パターン更新（L5189 の `matches!` マクロ内パターンを含む — ビルドエラーに出ない可能性があるため要注意） |
| `compiler.rs` | 2 箇所 | パターン更新 |
| `ast_lower_checker.rs` | 1 箇所 | `(parts, _)` → `(parts, _, _)` |
| `emit_python.rs` | 1 箇所 | `(parts, _)` → `(parts, _, _)` |
| `driver.rs` | **3 箇所** | パターン更新（L15697 の OR パターン内を含む） |
| `lsp/references.rs` | 1 箇所 | パターン更新 |

**合計 26 箇所**（ビルドエラー駆動で全箇所を修正。checker.rs L5189 の `matches!` マクロ内はビルドエラーに出ないため手動確認必須）

---

## テスト仕様（`v61500_tests` 2 件）

テストモジュールの先頭:
```rust
#[cfg(test)]
mod v61500_tests {
    use super::*;
```

### `fstring_nested_call`

```rust
/// f-string 内のネストした関数呼び出し・フィールドアクセスが正しくパースされることを確認
#[test]
fn fstring_nested_call() {
    // user.name はフィールドアクセス、Int.to_string はモジュール関数呼び出し
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
```

### `fstring_multiline`

```rust
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
```

---

## 完了条件

- `fstring_nested_call` pass
- `fstring_multiline` pass
- 総テスト数: **3367** tests passed, 0 failed（ベース 3365 + 2）
- 既存の f-string テスト（`test_parse_fstring_*`）が引き続き pass
- `cargo test -j 8 -- --test-threads=8` 0 failures
- `fav fmt` で `f"..."` と `f"""..."""` が正しく整形される
- CHANGELOG は v62.0 でまとめて記載のため本バージョンでの個別更新不要

---

## ベーステスト数の注意点

ロードマップ記載「ベース 3365 + 2 = 3367」は v61.4.0 code-reviewer 対応後の実績値を使用。
実際の v61.4.0 テスト数: **3365**（E0396/E0397 ネガティブテスト追加で +2）
実際のテスト数目標: **3365 + 2 = 3367** tests passed, 0 failed

---

## テスト数推移（参照用）

| バージョン | テスト数 | 備考 |
|---|---|---|
| v61.3.0 | 3361 | OR パターンガード拡張（code-reviewer +1 で +3） |
| v61.4.0 | 3365 | record update 式（code-reviewer +2 で +4） |
| v61.5.0 | **3367** | f-string 強化 |
