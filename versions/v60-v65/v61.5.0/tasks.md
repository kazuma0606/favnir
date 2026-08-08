# v61.5.0 Tasks — 文字列補間強化（ネスト呼び出し・マルチライン `f"""..."""`）

Date: 2026-08-01
Status: COMPLETE

---

## T0: 事前確認

- [x] `cargo test` でベースラインが 3365 tests passed, 0 failed であることを確認
- [x] `fav/Cargo.toml` のバージョンが `"61.0.0"` であることを確認
  - `grep '^version' fav/Cargo.toml` → `version = "61.0.0"`
- [x] `v61500_tests` がまだ存在しないことを確認
  - `grep -c 'v61500_tests' fav/src/driver.rs` = 0 件
- [x] `v61400_tests` が存在すること（挿入先が実在すること）を確認
  - `grep -c 'v61400_tests' fav/src/driver.rs` ≥ 1 件
- [x] `FStringTripleRaw` がまだ存在しないことを確認
  - `grep -c 'FStringTripleRaw' fav/src/frontend/lexer.rs` = 0 件
- [x] `Expr::FString` の現在の定義を確認（2 引数か 3 引数か）
  - `grep -n 'FString(' fav/src/ast.rs`
- [x] `Expr::FString` の出現箇所数を確認
  - `grep -c 'Expr::FString' fav/src/lint.rs` = 9 件
  - `grep -c 'Expr::FString\|ast::Expr::FString' fav/src/lineage.rs` = 5 件
  - `grep -c 'Expr::FString\|ast::Expr::FString' fav/src/middle/checker.rs` = **4** 件（L5189 の `matches!` マクロ内を含む）
  - `grep -c 'Expr::FString' fav/src/middle/compiler.rs` = 2 件
  - `grep -c 'Expr::FString' fav/src/driver.rs` = **3** 件（L15697 の OR パターン内を含む）
- [x] `fmt.rs` の現在の FString 出力形式を確認
  - `grep -n 'FString\|\$\"' fav/src/fmt.rs`
- [x] `lsp/inlay_hints.rs` に `Expr::FString` の直接 match がないことを確認
  - `grep -n 'Expr::FString\|Expr::' fav/src/lsp/inlay_hints.rs | head -20`
- [x] `$"` 出力を assert している既存テストを確認
  - `grep -rn '\\$\\"' fav/src/` — 該当箇所は T5 で `f"` に更新する

---

## T1: `ast.rs` — `Expr::FString` に `multiline: bool` フラグ追加

`FString(Vec<FStringPart>, Span)` を以下に変更:

```rust
/// String interpolation (f"..." or f"""...""").
/// multiline = true if the source used triple-quote syntax. (v61.5.0)
FString(Vec<FStringPart>, bool /* multiline */, Span),
```

`span()` メソッドの更新（`Expr::FString(_, s) => s` を変更）:
```rust
Expr::FString(_, _, s) => s,
```

- [x] `Expr::FString` バリアントに `bool` フラグを追加した
- [x] `span()` メソッドを `Expr::FString(_, _, s) => s` に更新した
- [x] `cargo build 2>&1 | grep "^error" | head -30` でエラー一覧を確認した

---

## T2: コンパイルエラー修正（各 `Expr::FString` match 箇所）

`cargo build` のエラーを基に、各ファイルを修正する。
基本方針: `multiline` フラグを使わない箇所は `_` で受け流す。

### T2-1: `lint.rs` — 9 箇所を更新

`grep -n 'Expr::FString' fav/src/lint.rs` で行番号を確認し、全 9 箇所を更新:
- `Expr::FString(parts, _)` → `Expr::FString(parts, _, _)`

- [x] lint.rs の全 `Expr::FString` パターンを更新した（計 9 箇所）
- [x] `grep -c 'Expr::FString' fav/src/lint.rs` が 9 件のままであることを確認した

### T2-2: `lineage.rs` — 5 箇所を更新

各行の `ast::Expr::FString(_, _)` を `ast::Expr::FString(_, _, _)` に変更:
```rust
// 更新前
ast::Expr::Lit(_, _) | ast::Expr::Ident(_, _) | ast::Expr::FString(_, _) => {}
// 更新後
ast::Expr::Lit(_, _) | ast::Expr::Ident(_, _) | ast::Expr::FString(_, _, _) => {}
```

- [x] lineage.rs の全 `FString` パターンを更新した（計 5 箇所）

### T2-3: `checker.rs` — **4 箇所**を更新

`grep -n 'Expr::FString\|matches.*FString' fav/src/middle/checker.rs` で確認し、4 箇所を更新:
- `Expr::FString(parts, _)` → `Expr::FString(parts, _, _)`
- `Expr::FString(parts, span)` → `Expr::FString(parts, _, span)`（span を使う箇所）
- **L5189 付近の `matches!(inner.as_ref(), Expr::FString(_, _))`** → `Expr::FString(_, _, _)`
  （`matches!` マクロ内パターンは `cargo build` のエラーとして検出されない可能性があるため手動確認必須）

- [x] checker.rs の全 `Expr::FString` パターンを更新した（計 **4** 箇所）
- [x] L5189 付近の `matches!` マクロ内パターンを手動確認・更新した

### T2-4: `compiler.rs` — 2 箇所を更新

```rust
Expr::FString(parts, _) => compile_fstring(parts, ctx),
// →
Expr::FString(parts, _, _) => compile_fstring(parts, ctx),
```

```rust
Expr::FString(parts, _) => { ... }
// →
Expr::FString(parts, _, _) => { ... }
```

- [x] compiler.rs の全 `Expr::FString` パターンを更新した（計 2 箇所）

### T2-5: `ast_lower_checker.rs` — 1 箇所を更新

```rust
ast::Expr::FString(parts, _) => lower_fstring(parts),
// →
ast::Expr::FString(parts, _, _) => lower_fstring(parts),
```

- [x] ast_lower_checker.rs の `Expr::FString` パターンを更新した

### T2-6: `emit_python.rs` — 1 箇所を更新

```rust
Expr::FString(parts, _) => { ... }
// →
Expr::FString(parts, _, _) => { ... }
```

- [x] emit_python.rs の `Expr::FString` パターンを更新した

### T2-7: `driver.rs` — **3 箇所**を更新

`grep -n 'Expr::FString' fav/src/driver.rs` で行番号を確認し、3 箇所を更新:
- L15560 付近の通常 match アーム: `Expr::FString(parts, _)` → `Expr::FString(parts, _, _)`
- **L15697 付近の OR パターン内**: `| Expr::FString(_, _)` → `| Expr::FString(_, _, _)`
  （OR パターンの中間に位置するため見落とし注意）

- [x] driver.rs の全 `Expr::FString` パターンを更新した（計 **3** 箇所）
- [x] L15697 付近の OR パターン内パターンを更新した

### T2-8: `lsp/references.rs` — 1 箇所を更新

```rust
Expr::FString(parts, _) => { ... }
// →
Expr::FString(parts, _, _) => { ... }
```

- [x] lsp/references.rs の `Expr::FString` パターンを更新した

### T2-9: `parser.rs` — 既存 `Expr::FString` 生成をスタブ更新

`parse_fstring_parts` の戻り値（L3556 付近）:
```rust
// 暫定: multiline は P4 で正しく設定
Ok(Expr::FString(parts, false, base_span))
```

parser.rs 内のテストコード（`Expr::FString(parts, _)` → `Expr::FString(parts, _, _)`）:
- L3961, L3974, L3981 付近の 3 箇所

- [x] `parse_fstring_parts` の戻り値を `Expr::FString(parts, false, base_span)` に暫定更新した
- [x] `grep -n 'Expr::FString' fav/src/frontend/parser.rs` でテストコード内パターン箇所を確認し更新した

### T2-10: `ast.rs` — テストコード内パターン

`grep -n 'FString' fav/src/ast.rs` でテスト内の出現を確認し更新:
- [x] ast.rs 内のテストコードを必要に応じて更新した

### T2-11: コンパイル確認

- [x] `cargo build` でコンパイルエラーがないことを確認した

---

## T3: `lexer.rs` — `FStringTripleRaw` トークン追加

### T3-1: `TokenKind` に `FStringTripleRaw(String)` 追加

`FStringRaw(String)` の定義の直後に追加:
```rust
/// Triple-quote f-string raw content: f"""...""" (v61.5.0)
FStringTripleRaw(String),
```

- [x] `FStringTripleRaw(String)` を `TokenKind` に追加した

### T3-2: triple-quote 分岐を `FStringTripleRaw` に変更

L285-291 付近の triple-quote アーム:
```rust
'f' if self.peek2() == Some('"') && self.peek3() == Some('"') => {
    self.advance(); // 'f'
    self.advance(); // '"' (1st)
    self.advance(); // '"' (2nd)
    self.advance(); // '"' (3rd)
    TokenKind::FStringTripleRaw(self.lex_fstring_triple(sp, sl, sc)?)
}
```

- [x] triple-quote 分岐を `FStringTripleRaw` を生成するよう変更した
- [x] `cargo build` でコンパイルエラーがないことを確認した
  - `TokenKind` を網羅 match している箇所でエラーが出た場合は `_ =>` アームで対応

---

## T4: `parser.rs` — FStringTripleRaw アーム追加 + multiline フラグ設定

### T4-1: `parse_fstring_parts` シグネチャ変更

`multiline: bool` 引数を追加:
```rust
fn parse_fstring_parts(
    &mut self,
    raw: &str,
    base_span: Span,
    multiline: bool,   // v61.5.0
) -> Result<Expr, ParseError> {
    // ...（既存ロジックはそのまま）
    Ok(Expr::FString(parts, multiline, base_span))
}
```

- [x] `parse_fstring_parts` に `multiline: bool` 引数を追加した
- [x] 戻り値を `Expr::FString(parts, multiline, base_span)` に更新した

### T4-2: `FStringRaw` アームを `multiline=false` に更新

```rust
TokenKind::FStringRaw(raw) => {
    self.advance();
    self.parse_fstring_parts(&raw, start, false)
}
```

- [x] `FStringRaw` アームを `multiline=false` で呼び出すよう更新した

### T4-3: `FStringTripleRaw` アームを新規追加

`FStringRaw` アームの直後に追加:
```rust
TokenKind::FStringTripleRaw(raw) => {
    self.advance();
    self.parse_fstring_parts(&raw, start, true)
}
```

- [x] `FStringTripleRaw` アームを追加した
- [x] `cargo build` でコンパイルエラーがないことを確認した

---

## T5: `fmt.rs` — multiline 分岐追加

既存の `Expr::FString(parts, _)` アームを `multiline` 分岐付きに置き換え:

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

- [x] `Expr::FString` の multiline 分岐を追加した
- [x] 非 multiline の出力が `f"..."` になることを確認した（旧 `$"..."` から変更）
- [x] `grep -rn '\$\\"' fav/src/` で `$"` 出力を assert している既存テストを確認し、`f"` に更新した
- [x] `cargo build` でコンパイルエラーがないことを確認した

---

## T6: `driver.rs` — `v61500_tests` モジュール追加

`v61400_tests` の直前（上側）に挿入する:

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

- [x] `v61500_tests` モジュールを `v61400_tests` の直前（上側）に追加した
- [x] `use super::*;` が含まれている
- [x] `fstring_nested_call` テストが含まれている
- [x] `fstring_multiline` テストが含まれている

---

## T7: テスト実行・確認

- [x] `cargo test -j 8 -- --test-threads=8` を実行
- [x] `v61500_tests::fstring_nested_call` pass
- [x] `v61500_tests::fstring_multiline` pass
- [x] 既存 fstring テスト（`test_parse_fstring_simple` / `test_parse_fstring_literal_only` / `test_parse_fstring_escape_brace`）が引き続き pass
- [x] `test_fstring_raw_token` が引き続き pass
- [x] 総テスト数 **3369** tests passed, 0 failed を確認（code-reviewer 対応で +2）

---

## T8: 事後処理

- [x] `versions/current.md` を v61.5.0 / 3367 tests に更新
- [x] `versions/roadmap/roadmap-v61.1-v62.0.md` の v61.5.0 実績欄を更新
- [x] CHANGELOG.md: サブバージョンのため個別エントリは不要（v62.0 でまとめて記載）
- [x] このファイル（tasks.md）を COMPLETE ステータスに更新

---

## コードレビュー指摘と対応

code-reviewer 指摘（実装後）:
- [HIGH] lexer.rs の triple-quote guard が `f""x` も誤分類 → pos+3 チェック追加 + single-quote アームを `NOT(triple-quote条件)` に変更
- [HIGH] fmt.rs multiline Lit に `"""` が含まれるとラウンドトリップ破壊 → `fmt_fstring_triple_lit` 関数追加してエスケープ
- [MED] FStringTripleRaw のレクサーテスト欠如 → `test_fstring_triple_raw_token` / `test_fstring_double_quote_is_not_triple` 追加（合計テスト +2、3369）
- [MED] driver.rs `format_expr_compact` が `$"` のまま → `f"` に統一
- [LOW] `fav new` テンプレートに `$"` 残存 → `f"` に更新

spec-reviewer 指摘（実装前）:
- [HIGH-1] ロードマップのテスト数 3363+2=3365 → 3365+2=3367 に修正 → roadmap 修正済み
- [HIGH-2] checker.rs は 4 箇所（matches! マクロ内 L5189 含む） → 4 箇所全更新済み
- [HIGH-3] driver.rs は 2 箇所（OR パターン L15697 含む） → 2 箇所全更新済み（実際は 2 箇所で正しかった）
- [HIGH-4] $" → f" 変更の既存テスト影響 → grep 確認チェック追加・既存テスト pass 確認済み
- [MED-1] inlay_hints.rs 確認 → FString match なしを確認
- [MED-2] parser.rs 行番号固定 → grep 動的確認に変更
- [MED-3] multiline 終端検出確認 → fstring_multiline テストで確認
- [MED-4] T0 バージョン文言 → 運用上問題なし（v61.0.0 で正しい）

---

Status: COMPLETE
