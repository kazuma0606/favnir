# v16.2.0 Tasks — 文字列補間（String Interpolation）

Date: 2026-06-14
Branch: master

---

## Phase A — Cargo バージョン更新

- [x] A-1: `fav/Cargo.toml` の `version` を `"16.2.0"` に変更
- [x] A-2: `cargo build` → コンパイルエラーなし確認

---

## Phase B — `f"..."` 構文追加（lexer.rs）

- [x] B-1: `fav/src/frontend/lexer.rs` に `peek3()` メソッド追加
- [x] B-2: レキサーの文字処理に `f"..."` ケースを追加（`$"..."` の前後に配置）
  - `'f'` かつ `peek2() == '"'` かつ `peek3() != '"'` → `FStringRaw` トークン生成
- [x] B-3: `cargo build` → コンパイルエラーなし確認
- [x] B-4: `f"Hello {name}"` を手動でレキサーを通して `FStringRaw("Hello {name}")` が返ることを確認

---

## Phase C — `f"""..."""` triple-quote 追加（lexer.rs）

- [x] C-1: `fav/src/frontend/lexer.rs` に `lex_fstring_triple` メソッド追加
  - `"""` が出現するまでスキャン（改行を含む）
  - 内部の `{...}` のネスト深さを正しく管理
- [x] C-2: レキサーの文字処理に `f"""..."""` ケースを追加（`f"..."` の前に配置）
  - `'f'` かつ `peek2() == '"'` かつ `peek3() == '"'` → triple-quote として処理
- [x] C-3: `cargo build` → コンパイルエラーなし確認
- [x] C-4: `f"""Name: {name}"""` を手動でレキサーを通して正しいトークンが返ることを確認

---

## Phase D — checker.fav パス対応（ast_lower_checker.rs）

- [x] D-1: `fav/src/middle/ast_lower_checker.rs` に `lower_fstring` 関数追加
  - `Vec<FStringPart>` を `String.concat` の ECall チェーンに変換
  - `FStringPart::Lit(s)` → `v1("ELit", v1("LStr", sv(s)))`
  - `FStringPart::Expr(e)` → `lower_expr(e)`
- [x] D-2: `lower_fstring_part` ヘルパー追加
- [x] D-3: `lower_expr` の `Expr::FString` ケースを `lower_fstring(parts)` に変更
  - `ast::Expr::FString(parts, _) => lower_fstring(parts)` を `_ => v1("EVar", sv("_unsupported_"))` の前に追加
- [x] D-4: `cargo build` → コンパイルエラーなし確認
- [x] D-5: checker.fav パスで f-string コードの型チェックが通ることを確認

---

## Phase E — compiler.fav f-string 対応（self/compiler.fav）

> **注意**: Phase E は複雑であり、v162000_tests は Rust パイプラインを使うため Phase E なしでも PASS する。
> 時間的制約がある場合は v16.3.0 に延期可能。

- [ ] E-1: `fav/self/compiler.fav` のトークン型に `TkFStr(String)` 追加
- [ ] E-2: `fav/self/compiler.fav` のレキサー関数に `f"..."` / `$"..."` 認識を追加
  - `lex_fstring_raw` ヘルパー関数（`{` / `}` の深さ管理）
- [ ] E-3: `fav/self/compiler.fav` の AST 型に `EFString(List<FStringPart>)` 追加
  - `FStringPart` 型（`FStrLit(String)` / `FStrExpr(Expr)`）追加
- [ ] E-4: `fav/self/compiler.fav` の `parse_atom` / `parse_primary` に `TkFStr` ケース追加
  - `parse_fstring_raw(raw)` → `EFString(parts)` への変換
- [ ] E-5: `fav/self/compiler.fav` の `compile_expr` に `EFString` ケース追加
  - `FStrLit(s)` → `ELit(LStr(s))` としてコンパイル
  - `FStrExpr(e)` → `e` をコンパイル後 `Debug.show` を通す
  - 全 part を `String.concat` でチェーン（左畳み込み）
- [ ] E-6: `cargo build` → コンパイルエラーなし確認
- [ ] E-7: `fav run` パス（compiler.fav 経由）で f-string が動作することを確認

---

## Phase F — get_help_text 更新（driver.rs）

- [x] F-1: `fav/src/driver.rs` の `get_help_text` に `"E0253"` ヒント追加
  - 「f-string 内で別の f-string を使うことはできません」
  - 「中間変数に束縛してください: `bind s <- f"..."`」
- [x] F-2: `fav/src/driver.rs` の `get_help_text` に `"E0254" | "E0322"` ヒント追加
  - 「f-string には String / Int / Float / Bool のみ使用可能です」
  - 「他の型は事前に文字列化してください」

---

## Phase G — v162000_tests 追加（driver.rs）

- [x] G-1: `fav/src/driver.rs` に `run_source` テスト補助関数を含む `v162000_tests` モジュール追加
- [x] G-2: 以下の 5 テストを実装:
  - `version_is_16_2_0`: Cargo.toml version == "16.2.0"
  - `fstring_basic_interpolation`: `f"Hello, {name}!"` → `"Hello, Alice!"`
  - `fstring_int_interpolation`: `f"Age: {age}"` (Int) → `"Age: 42"`
  - `fstring_expr_interpolation`: `f"len={List.length(xs)}"` → `"len=3"`
  - `fstring_triple_quote`: `f"""Name: {name}, Score: {score}"""` → 正しい文字列
- [x] G-3: `cargo test v162000` → 5/5 PASS 確認

---

## Phase H — サイトドキュメント

- [x] H-1: `site/content/docs/language/string-interpolation.mdx` 新規作成
  - 概要・基本構文（`f"..."` / triple-quote）
  - 埋め込める型（String / Int / Float / Bool）
  - Before/After 比較（`String.concat` 連鎖 vs `f"..."`）
  - よくある間違い（Display 未実装 → E0322）

---

## Phase I — テスト確認とコミット

- [x] I-1: `cargo test v162000` → 5/5 PASS 最終確認
- [x] I-2: `cargo test` → 全件 PASS 確認（リグレッションなし）
- [x] I-3: コミット `feat: v16.2.0 — 文字列補間（f"..." 構文 + triple-quote）`

---

## 完了条件

| 確認項目 | 状態 |
|---|---|
| `Cargo.toml version == "16.2.0"` | [ ] |
| `f"Hello, {name}!"` が正しく動作する | [ ] |
| `f"Age: {age}"` (Int) が自動変換される | [ ] |
| `f"len={List.length(xs)}"` (式) が動作する | [ ] |
| `f"""..."""` triple-quote が動作する | [ ] |
| `cargo test v162000` 全テストパス（5/5） | [ ] |
| `cargo test` 全件パス（リグレッションなし） | [ ] |
| `$"..."` 旧構文が引き続き動作する | [ ] |
| E0253 / E0322 の get_help_text が追加されている | [ ] |
| `site/content/docs/language/string-interpolation.mdx` が存在する | [ ] |

---

## 実装メモ

### lexer.rs の peek3()

```rust
fn peek3(&self) -> Option<char> {
    self.source.get(self.pos + 2).copied()
}
```

`Lexer.source` が `Vec<char>` であることを確認（line 1 付近のフィールド定義を参照）。

### ast_lower_checker.rs の ECall 引数順序

compiler.fav の `ECall` は `(ns, fname, args)` の 3-tuple。
引数リストは `EArgList({ _0: first_arg, _1: EArgList({ _0: second_arg, _1: EArgNil }) })` の再帰構造。

`String.concat(a, b)` の場合:
```
ECall(
  "String",
  "concat",
  EArgList(a, EArgList(b, EArgNil))
)
```

`lower_fstring` では左から右へ `String.concat` をチェーンする:
```
f"A{x}B{y}C"
→ concat(concat(concat("A", x), "B"), concat(y, "C"))
```

よりシンプルには各 part を順番に連結:
```
String.concat("A", String.concat(x, String.concat("B", String.concat(y, "C"))))
```

### Lexer フィールド確認

`Lexer` の `source` フィールドが `Vec<char>` か `&str` か確認してから `peek3()` を実装する。
`lexer.rs` の `struct Lexer` 定義（line 200 前後）を参照。
