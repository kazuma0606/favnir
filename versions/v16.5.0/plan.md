# v16.5.0 Plan — 型エイリアス（Type Alias）

Date: 2026-06-14

---

## 実装フェーズ一覧

| Phase | 内容 | 主要ファイル |
|---|---|---|
| A | Cargo バージョン更新 | `Cargo.toml` |
| B | Lexer: `alias` キーワード追加 | `lexer.rs` |
| C | AST: `TopLevel::AliasDecl` 追加 | `ast.rs` |
| D | Parser: `alias` 構文パース | `parser.rs` |
| E | Checker: エイリアス収集・展開 | `checker.rs` |
| F | Compiler: `AliasDecl` をスキップ | `compiler.rs` |
| G | exhaustive match 対応（lineage / lint / wasm / driver 等） | 各ファイル |
| H | テスト追加（v165000_tests — 5 件） | `driver.rs` |
| I | サイトドキュメント | `site/content/docs/language/type-alias.mdx` |
| J | テスト確認とコミット | — |

---

## Phase A — Cargo バージョン更新

```toml
[package]
version = "16.5.0"
```

**確認:** `cargo build` でコンパイルエラーなし。

---

## Phase B — Lexer: `alias` キーワード追加（lexer.rs）

`TokenKind` に `Alias` を追加:

```rust
Alias,
```

`next_token` の識別子認識部分に追加:

```rust
"alias" => TokenKind::Alias,
```

**確認:** `cargo build` でコンパイルエラーなし。

---

## Phase C — AST: `TopLevel::AliasDecl` 追加（ast.rs）

`TopLevel` enum に追加:

```rust
AliasDecl {
    name: String,
    params: Vec<String>,
    ty: TypeExpr,
    span: Span,
},
```

**確認:** `cargo build` → exhaustive match エラーを確認（次フェーズで修正）。

---

## Phase D — Parser: `alias` 構文パース（parser.rs）

`parse_top_level` に分岐追加:

```rust
TokenKind::Alias => self.parse_alias_decl(),
```

`parse_alias_decl` の実装:

```rust
fn parse_alias_decl(&mut self) -> Result<TopLevel, ParseError> {
    let span = self.current_span();
    self.expect(TokenKind::Alias)?;
    let name = self.expect_ident()?;
    // ジェネリックパラメータ: <T, U>
    let params = if self.peek_is(TokenKind::LAngle) {
        self.parse_type_param_names()?
    } else {
        vec![]
    };
    self.expect(TokenKind::Eq)?;
    let ty = self.parse_type_expr()?;
    Ok(TopLevel::AliasDecl { name, params, ty, span })
}
```

**確認:** `cargo build` でコンパイルエラーなし（exhaust エラーは Phase G で対処）。

---

## Phase E — Checker: エイリアス収集・展開（checker.rs）

### 5-1: `alias_env` フィールド追加

```rust
pub alias_env: HashMap<String, (Vec<String>, Type)>,
// key: alias name, value: (param_names, expanded_type)
```

### 5-2: 収集フェーズ

`check_program` または `init_env` の先頭で全 `AliasDecl` を走査:

```rust
for tl in &program.top_levels {
    if let TopLevel::AliasDecl { name, params, ty, .. } = tl {
        let expanded = lower_type_expr(ty);  // TypeExpr → Type
        self.alias_env.insert(name.clone(), (params.clone(), expanded));
    }
}
```

### 5-3: `resolve_alias` 実装

型チェック時に `Named(name, args)` を展開:

```rust
fn resolve_alias(&self, ty: Type) -> Type {
    match &ty {
        Type::Named(name, args) => {
            if let Some((params, base)) = self.alias_env.get(name) {
                if params.is_empty() {
                    // 非ジェネリック: そのまま展開
                    return self.resolve_alias(base.clone());
                } else {
                    // ジェネリック: 型引数を代入して展開
                    let subst: HashMap<String, Type> = params.iter()
                        .zip(args.iter())
                        .map(|(p, a)| (p.clone(), a.clone()))
                        .collect();
                    let substituted = subst_type(base.clone(), &subst);
                    return self.resolve_alias(substituted);
                }
            }
            ty
        }
        // List / Result / Option 等は再帰展開
        Type::List(inner) => Type::List(Box::new(self.resolve_alias(*inner.clone()))),
        Type::Result(ok, err) => Type::Result(
            Box::new(self.resolve_alias(*ok.clone())),
            Box::new(self.resolve_alias(*err.clone())),
        ),
        Type::Option(inner) => Type::Option(Box::new(self.resolve_alias(*inner.clone()))),
        _ => ty,
    }
}
```

### 5-4: 型チェック時に `resolve_alias` を適用

`lower_type_expr` の結果を `check_fn_def` で使う箇所と `check_expr` の型比較箇所に `resolve_alias` を挿入。具体的には:
- `check_fn_def`: パラメータ型・戻り型の `lower_type_expr` 後に `resolve_alias` 適用
- `unify` / `check_type_compat`: 比較前に両辺を `resolve_alias`

**確認:** `cargo build` でコンパイルエラーなし。

---

## Phase F — Compiler: `AliasDecl` をスキップ（compiler.rs）

`compile_top_level` に分岐追加:

```rust
TopLevel::AliasDecl { .. } => {
    // 型情報のみ — IR 生成なし
}
```

または既存の `_ => {}` フォールスルーに該当する場合はそのまま。

**確認:** `cargo build` でコンパイルエラーなし。

---

## Phase G — exhaustive match 対応

`TopLevel::AliasDecl` を追加したことで exhaustive match エラーが発生するファイルを修正:

- `driver.rs` の `TopLevel` match
- `lineage.rs` の `TopLevel` match（あれば）
- `lint.rs` の `TopLevel` match（あれば）
- `emit_python.rs` の `TopLevel` match（あれば）
- `fmt.rs` の `TopLevel` match（あれば）

各箇所に:
```rust
TopLevel::AliasDecl { .. } => { /* 型情報のみ、スキップ */ }
```

**確認:** `cargo build` でコンパイルエラーなし。

---

## Phase H — テスト追加（driver.rs）

`fav/src/driver.rs` に `v165000_tests` モジュールを追加:

```rust
#[cfg(test)]
mod v165000_tests {
    // version_is_16_5_0
    // alias_basic            -- alias Email = String を使った関数が動作
    // alias_interchangeable  -- alias UserId = Int、Int 引数として渡せる
    // alias_generic          -- alias Result2<T> = Result<T, String> が動作
    // alias_in_signature     -- エイリアスを引数型・戻り型に使った fn が動作
}
```

**テスト実行:** `cargo test v165000` → 5/5 PASS 確認。

---

## Phase I — サイトドキュメント

`site/content/docs/language/type-alias.mdx` を新規作成:
- `alias` 構文の説明
- `type Name(Inner)` との違い
- 使用例（Email / Timestamp / ジェネリック）
- 制約（循環エイリアス禁止、`where` 不可）

---

## 実装の注意点

1. **`lower_type_expr` と `resolve_alias` の関係**: `lower_type_expr` は `TypeExpr`（構文木）を `Type`（意味型）に変換する。`resolve_alias` はさらに名前解決を行う。どちらも必要。

2. **ジェネリック展開の実装方針**: `subst_type(base, subst)` は既存の型変数置換関数（`Type::Var` を使った HM 推論で実装済みの可能性あり）を流用できる。なければ新規実装。

3. **`AliasDecl` の収集タイミング**: `check_program` の最初（関数定義チェック前）に全 `alias` を収集する。前方参照（alias より後に定義された alias を参照）も動作させるため 2 パスが望ましい。

4. **エラーメッセージ**: 型不一致時のメッセージはエイリアス名で表示（`"Expected Email, got Int"` ではなく `"Expected String, got Int"`）— エイリアスは透明。ただし将来的に改善可能。

5. **`ast_lower_checker.rs`**: checker.fav 経由のパスにも `AliasDecl` の対応が必要（フォールバックでスキップ可）。
