# v16.2.0 Plan — 文字列補間（String Interpolation）

Date: 2026-06-14

---

## 前提確認

実装前に以下を把握しておく。

### 実装済みの資産

| ファイル | 実装済み内容 |
|---|---|
| `fav/src/frontend/lexer.rs:270` | `$"..."` → `FStringRaw(String)` トークン（`lex_fstring_raw`） |
| `fav/src/frontend/parser.rs:2263` | `parse_fstring_parts` — `{...}` を分解して `Expr::FString` 生成 |
| `fav/src/ast.rs:172` | `FStringPart { Lit(String), Expr(Box<Expr>) }` / `Expr::FString(Vec<FStringPart>, Span)` |
| `fav/src/middle/checker.rs:4586` | `check_fstring` — E0253（ネスト）/ E0254（Display 未実装）を報告 |
| `fav/src/middle/compiler.rs:1761` | `compile_fstring` — `String.concat` + `Debug.show` への展開 |

### 修正・追加が必要な箇所

| ファイル | 変更内容 |
|---|---|
| `fav/src/frontend/lexer.rs` | `f"..."` / `f"""..."""` トークン追加 |
| `fav/src/middle/ast_lower_checker.rs:354` | `Expr::FString` を `_unsupported_` ではなく concat チェーンに変換 |
| `fav/self/compiler.fav` | f-string レキシング・パース・コンパイル追加 |
| `fav/src/driver.rs` | `get_help_text` に E0322 追加、`v162000_tests` 追加 |
| `site/content/docs/language/string-interpolation.mdx` | 新規作成 |

---

## Phase A — Cargo バージョン更新

**変更ファイル**: `fav/Cargo.toml`

```toml
version = "16.2.0"
```

新規 crate 依存なし。

---

## Phase B — `f"..."` 構文（lexer.rs）

**変更ファイル**: `fav/src/frontend/lexer.rs`

現在の実装（line 270）:
```rust
'$' if self.peek2() == Some('"') => {
    self.advance();
    self.advance();
    TokenKind::FStringRaw(self.lex_fstring_raw(sp, sl, sc)?)
}
```

追加する実装:
```rust
'f' if self.peek2() == Some('"') && self.peek3() != Some('"') => {
    self.advance(); // 'f'
    self.advance(); // '"'
    TokenKind::FStringRaw(self.lex_fstring_raw(sp, sl, sc)?)
}
```

`peek3()` を判定に使うため、`Lexer` に `peek3()` メソッドを追加する（または `source[pos..]` を使って判定）。

**実装メモ**:
- `lex_fstring_raw` はすでに `{` / `}` のネストを正しく処理しており、再利用可能
- `f"..."` は `$"..."` と完全に同じ FStringRaw トークンを生成するだけ
- `f` トークン判定は既存の `Ident` 判定の前に配置する（`f` がidentとして誤認されないよう）

---

## Phase C — `f"""..."""` triple-quote（lexer.rs）

**変更ファイル**: `fav/src/frontend/lexer.rs`

```rust
'f' if self.peek2() == Some('"') && self.peek3() == Some('"') => {
    self.advance(); // 'f'
    self.advance(); // '"' (1st)
    self.advance(); // '"' (2nd)
    self.advance(); // '"' (3rd)
    TokenKind::FStringRaw(self.lex_fstring_triple(sp, sl, sc)?)
}
```

新規メソッド `lex_fstring_triple`:
```rust
fn lex_fstring_triple(
    &mut self,
    start_pos: usize,
    start_line: u32,
    start_col: u32,
) -> Result<String, LexError> {
    let mut out = String::new();
    let mut depth = 0usize;
    loop {
        match self.peek() {
            None => {
                return Err(LexError::new(
                    "unterminated triple-quote string interpolation",
                    self.span_from(start_pos, start_line, start_col),
                ));
            }
            Some('"') if depth == 0 && self.peek2() == Some('"') && self.peek3() == Some('"') => {
                self.advance(); // '"'
                self.advance(); // '"'
                self.advance(); // '"'
                return Ok(out);
            }
            Some('\\') => {
                out.push(self.advance());
                if let Some(_) = self.peek() {
                    out.push(self.advance());
                }
            }
            Some('{') => {
                depth += 1;
                out.push(self.advance());
            }
            Some('}') => {
                depth = depth.saturating_sub(1);
                out.push(self.advance());
            }
            Some(_) => {
                out.push(self.advance());
            }
        }
    }
}
```

**注意**: `peek3()` が必要。Lexer に以下を追加:
```rust
fn peek3(&self) -> Option<char> {
    self.source.get(self.pos + 2).copied()
}
```

---

## Phase D — checker.fav パス対応（ast_lower_checker.rs）

**変更ファイル**: `fav/src/middle/ast_lower_checker.rs`

現在 line 354:
```rust
// Fallbacks for TypeApply, FString, AssertMatches, EmitExpr
_ => v1("EVar", sv("_unsupported_")),
```

変更後:
```rust
ast::Expr::FString(parts, _) => lower_fstring(parts),
// Fallbacks for TypeApply, AssertMatches, EmitExpr
_ => v1("EVar", sv("_unsupported_")),
```

新規追加関数 `lower_fstring`:
```rust
/// Desugar f-string to String.concat chain for checker.fav consumption.
///
/// f"Hello, {name}! Age: {age}"
/// → ECall("String", "concat", ["Hello, ", ECall("String", "concat", [EVar("name"), ECall("String", "concat", ["! Age: ", EVar("age")])])])
///
/// Note: Type coercion (Int→String via Debug.show) is handled by Rust checker.rs / compiler.rs.
/// For the checker.fav path we only need syntactic lowering; type checking is done in Rust.
fn lower_fstring(parts: &[ast::FStringPart]) -> Value {
    if parts.is_empty() {
        return v1("ELit", v1("LStr", sv("")));
    }
    let lowered: Vec<Value> = parts.iter().map(lower_fstring_part).collect();
    lowered.into_iter().reduce(|acc, next| {
        // ECall { ns: "String", fn: "concat", args: EArgList(acc, EArgList(next, EArgNil)) }
        v3(
            "ECall",
            sv("String"),
            sv("concat"),
            v2("EArgList", acc, v2("EArgList", next, v0("EArgNil"))),
        )
    }).unwrap()
}

fn lower_fstring_part(part: &ast::FStringPart) -> Value {
    match part {
        ast::FStringPart::Lit(s) => v1("ELit", v1("LStr", sv(s))),
        ast::FStringPart::Expr(expr) => lower_expr(expr),
    }
}
```

**注意**: `checker.fav` は `ECall` 引数の順序として `EArgList(arg1, EArgList(arg2, EArgNil))` を使っている（`compile_args` が args を逆順でスタックに積む）。`String.concat(left, right)` は `left` が第一引数なので引数リストに渡す順序を確認すること。

実際の `ECall` の構造（compiler.fav line 3677〜）:
```fav
ECall(parts) =>
  bind ns   <- parts._0;
  bind fname <- parts._1;
  bind args  <- parts._2;
```

引数リストは `EArgList({ _0: head, _1: tail })` の形式で `tail` が残り。
`String.concat(a, b)` の場合: `EArgList(a, EArgList(b, EArgNil))` と思われるが、
`compile_args` の動作を確認して正しい順序を使う。

---

## Phase E — compiler.fav f-string 対応

compiler.fav は独自のレキサー・パーサーを持つ。f-string を追加するには 3 つのステップが必要。

### E-1: トークン追加

```fav
// self/compiler.fav のトークン型に追加
type Token =
  | ...
  | TkFStr(String)   // f"..." / $"..." の中身（生文字列）
```

lexer 関数に追加:
```fav
// 'f' の次が '"' → f-string として読む
// '$' の次が '"' → 既存 f-string（後方互換）
// 共通ヘルパー関数 lex_fstring_raw を実装
```

### E-2: AST ノード追加

```fav
type Expr =
  | ...
  | EFString(List<FStringPart>)

type FStringPart =
  | FStrLit(String)
  | FStrExpr(Expr)
```

### E-3: パース追加

```fav
// parse_atom または parse_primary 内に追加
TkFStr(raw) => parse_fstring_raw(raw, toks)
```

```fav
fn parse_fstring_raw(raw: String, toks: List<Token>) -> Result<ParseResult<Expr>, String> {
  // raw を走査して { と } の間を式として再パース
  // 結果を List<FStringPart> に変換
  // EFString(parts) を返す
}
```

### E-4: コンパイル追加

```fav
// compile_expr 内に追加
EFString(parts) => compile_fstring(parts, ctx, env)
```

```fav
fn compile_fstring(parts: List<FStringPart>, ctx: CodegenCtx, env: List<KVPair>) -> Result<CodegenCtx, String> {
  // FStrLit(s) → ELit(LStr(s)) としてコンパイル
  // FStrExpr(e) → e をコンパイルし Debug.show を通す
  // 全 part を String.concat でチェーン（左畳み込み）
  // 空の場合は ELit(LStr("")) を返す
}
```

**スコープ**: Phase E は複雑なため、v16.2.0 では基礎構造の追加を行い、完全動作は v16.3.0 に延期可能。`v162000_tests` は Rust パイプライン（`exec_artifact_main` 経由）で検証するため、Phase E がなくてもテストは PASS する。

---

## Phase F — get_help_text 更新（driver.rs）

**変更ファイル**: `fav/src/driver.rs`

```rust
// get_help_text に追加
"E0253" => &[
    "f-string 内で別の f-string を使うことはできません",
    "ネストが必要な場合は中間変数に束縛してください: `bind s <- f\"...\"`",
],
"E0254" | "E0322" => &[
    "f-string には String / Int / Float / Bool のみ使用可能です（Display 実装型）",
    "他の型は事前に文字列化してください: `bind s <- my_val.to_string()`",
],
```

---

## Phase G — v162000_tests 追加（driver.rs）

**変更ファイル**: `fav/src/driver.rs`

末尾に追加:

```rust
// ── v162000_tests (v16.2.0) — 文字列補間（f-string） ─────────────────────────
#[cfg(test)]
mod v162000_tests {
    use crate::frontend::parser::Parser;
    use crate::middle::compiler::compile_program;
    use crate::backend::codegen::codegen_program;
    use crate::backend::vm::VM;

    fn run_source(src: &str) -> String {
        let program = Parser::parse_str(src, "fstring_test.fav").expect("parse");
        let ir = compile_program(&program);
        let artifact = codegen_program(&ir);
        let main_idx = artifact.fn_idx_by_name("main").expect("main");
        match VM::run(&artifact, main_idx, vec![]) {
            Ok(crate::value::Value::Str(s)) => s,
            other => panic!("unexpected result: {:?}", other),
        }
    }

    #[test]
    fn version_is_16_2_0() {
        let cargo = std::fs::read_to_string("Cargo.toml").unwrap();
        assert!(
            cargo.contains("version = \"16.2.0\""),
            "Cargo.toml version should be 16.2.0"
        );
    }

    #[test]
    fn fstring_basic_interpolation() {
        let result = run_source(r#"
public fn main() -> String {
    bind name <- "Alice"
    f"Hello, {name}!"
}
"#);
        assert_eq!(result, "Hello, Alice!");
    }

    #[test]
    fn fstring_int_interpolation() {
        let result = run_source(r#"
public fn main() -> String {
    bind age <- 42
    f"Age: {age}"
}
"#);
        assert_eq!(result, "Age: 42");
    }

    #[test]
    fn fstring_expr_interpolation() {
        let result = run_source(r#"
public fn main() -> String {
    bind xs <- List.of(1, 2, 3)
    f"len={List.length(xs)}"
}
"#);
        assert_eq!(result, "len=3");
    }

    #[test]
    fn fstring_triple_quote() {
        let result = run_source(r#"
public fn main() -> String {
    bind name <- "Bob"
    bind score <- 99
    f"""Name: {name}, Score: {score}"""
}
"#);
        assert_eq!(result, "Name: Bob, Score: 99");
    }
}
```

---

## Phase H — サイトドキュメント

**新規作成**: `site/content/docs/language/string-interpolation.mdx`

内容:
1. 概要（f-string で String.concat を置き換え）
2. 基本構文（`f"..."` / triple-quote）
3. 埋め込める型（String / Int / Float / Bool）
4. Before/After 比較
5. よくある間違いと修正方法（Display 未実装 → E0322）

---

## Phase I — コミット

```
feat: v16.2.0 — 文字列補間（f"..." 構文 + triple-quote + self-hosted 対応）
```

---

## 実装順序

```
Phase A (Cargo) → Phase B (f"..." lexer) → Phase C (triple-quote lexer)
→ Phase D (ast_lower_checker) → cargo build 確認
→ Phase F (get_help_text) → Phase G (テスト追加)
→ cargo test v162000 → 5/5 PASS 確認
→ Phase E (compiler.fav) ← 最も複雑、最後に実施
→ cargo test 全件確認
→ Phase H (docs) → Phase I (commit)
```

Phase E（compiler.fav）は他の Phase に独立しているため、最後に実施するか v16.3.0 に延期可能。
`v162000_tests` は Rust パイプラインを使用するため、Phase E なしでも PASS する。

---

## リスク・注意点

| リスク | 対策 |
|---|---|
| `f` が識別子の先頭として誤認識される | レキサーで `f"` を先読みしてから Ident へフォールバック |
| `peek3()` が Lexer に存在しない | `self.source.get(self.pos + 2).copied()` で実装 |
| `ast_lower_checker.rs` の `ECall` 引数順序 | `compile_args` の実装を確認（EArgList は LIFO）|
| compiler.fav の f-string パース内での再帰 | 内部式のパース深さ制限を設ける（E0253 と同様の制約）|
| 既存の `$"..."` テストへの影響 | 変更後も `$"..."` は動作する（テスト再実行で確認）|
