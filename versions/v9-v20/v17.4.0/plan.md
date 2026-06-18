# v17.4.0 — `let` バインディング 実装計画

## 方針

`let x = expr` を `Stmt::Let` として AST に追加し、
コンパイラは `StoreLocal` のみ（Result unwrap なし）、
チェッカーは型が `Result<_, _>` なら E0326 を発出する。

`bind` との違いは最小限：`bind` は Result を unwrap する特殊 opcode を持つが、
`let` は単純な式評価 → ローカル変数格納のみ。
パーサー的には `let name = expr` vs `bind name <- expr`（`=` と `<-` の違い）。

---

## 実装ステップ

### Step 1: Lexer（`fav/src/frontend/lexer.rs`）

`TokenKind::Let` を追加：

```rust
// TokenKind enum に追加
Let,  // "let"

// keyword match に追加（keyword_or_ident 関数）
"let" => TokenKind::Let,
```

`let` はすでに spec/plan などの Favnir コード例で使われているが、
現行レキサーでは `Ident("let")` として扱われている可能性がある。
キーワード化してもパーサーが `Ident` として扱っていた箇所に影響がないか確認。

### Step 2: AST 拡張（`fav/src/ast.rs`）

`Stmt` enum に `Let` variant を追加：

```rust
Let {
    name: String,
    expr: Box<Expr>,
    span: Span,
},
```

`Bind { name, expr, span }` と並べて配置する。
`Stmt::span()` メソッドに `Stmt::Let { span, .. } => span` を追加。

### Step 3: パーサー（`fav/src/frontend/parser.rs`）

`parse_stmt` に `TokenKind::Let` ブランチを追加：

```rust
TokenKind::Let => {
    let start = self.peek_span().clone();
    self.advance();                          // consume 'let'
    let (name, _) = self.expect_ident()?;   // variable name
    self.expect(&TokenKind::Eq)?;           // '='
    let expr = self.parse_expr()?;          // right-hand side
    Ok(Stmt::Let {
        name,
        expr: Box::new(expr),
        span: self.span_from(&start),
    })
}
```

`bind` パーサーとほぼ同じ構造。`<-`（LArrow）の代わりに `=`（Eq）を使う。

### Step 4: 型チェッカー（`fav/src/middle/checker.rs`）

`check_stmt` の `Stmt::Bind` ブランチの近くに `Stmt::Let` を追加：

```rust
Stmt::Let { name, expr, span } => {
    let ty = self.check_expr(expr);
    // Result<_, _> なら E0326
    if matches!(ty, Type::Result(_, _)) {
        self.error(span, "E0326", "let cannot be used with Result values — use `bind` instead");
    }
    self.env.define(name.clone(), ty);
}
```

`Stmt::Bind` との違い：`bind` は Result を unwrap して内側の型をスコープに追加するが、
`let` は型をそのままスコープに追加する（ただし Result は E0326）。

### Step 5: コンパイラ（`fav/src/middle/compiler.rs`）

`compile_stmt` の `Stmt::Bind` ブランチの近くに `Stmt::Let` を追加：

```rust
Stmt::Let { name, expr, .. } => {
    compile_expr(expr, ctx);
    let slot = ctx.next_slot;
    ctx.next_slot += 1;
    ctx.locals.insert(name.clone(), slot);
    ctx.emit(IRStmt::Bind(slot, IRExpr::Local(slot)));
    // 実際は: expr をコンパイルして StoreLocal slot
}
```

`Stmt::Bind` の `LegacyBindCheck` / `SeqStageCheck` opcode は不要。
IR レベルでは `IRStmt::Bind(slot, compiled_expr)` として格納する
（`compile_stmt` が IR を返す方式なら、`Stmt::Bind` の Result opcode を除いたパス）。

実装上は `compile_block` 内で `Stmt::Let` を `Stmt::Bind` 相当として処理し、
Result チェック opcode を発出しないだけで十分。

### Step 6: Exhaustive match 対応

以下のファイルの `Stmt` match に `Stmt::Let` を追加：

- `fav/src/fmt.rs` — `Stmt::Let { name, expr, .. } => format!("let {} = {}", name, fmt_expr(expr))`
- `fav/src/emit_python.rs` — `Stmt::Let { name, expr, .. } => format!("{} = {}", name, emit_expr(expr))`
- `fav/src/lineage.rs` — `Stmt::Let { expr, .. }` の子ノードを再帰的に処理
- `fav/src/lint.rs` — `Stmt::Let { name, expr, .. }` の lint チェック（W001 使用/未使用等）
- `fav/src/driver.rs` — `format_stmt_compact` 等の match に追加

`Stmt::Bind` の処理に倣って機械的に追加する。

### Step 7: self-hosted 対応（`self/compiler.fav` / `self/checker.fav`）

Favnir セルフホストの compiler.fav と checker.fav に `let` 文の処理を追加する。
`bind` 文のパース・コンパイル処理の近くに `let` の処理を追加。

**compiler.fav の `parse_stmt` 相当：**
```fav
// "let" トークンの場合
bind name <- parse_ident()
bind _ <- expect_token("=")
bind expr <- parse_expr()
Result.ok(Stmt.Let(name, expr))
```

**checker.fav の `check_stmt` 相当：**
```fav
Stmt.Let(name, expr) => {
  bind ty <- check_expr(ctx, expr)
  if is_result_type(ty) {
    Result.err("E0326: ...")
  } else {
    Result.ok(Ctx.define(ctx, name, ty))
  }
}
```

### Step 8: テスト（`fav/src/driver.rs`）

`v174000_tests` モジュールを追加（5件）。

### Step 9: バージョン更新

`fav/Cargo.toml` を `17.4.0` に更新。

---

## `let` と `bind` の併用パターン

```fav
fn process(rows: List<Row>) -> Result<List<Output>, String> {
  let filtered = [r | r <- rows, r.active]    // let: List<Row>
  bind results <- List.map_result(filtered, transform)  // bind: Result<List<Output>, String>
  Result.ok(results)
}
```

チェッカーは `let` の右辺の型を通常通り推論し、
`Result<_, _>` なら E0326、そうでなければ型をスコープに追加するだけ。

---

## Lexer の注意点

`let` をキーワード化すると、既存の Favnir ファイル内で
識別子名として `let` を使っているケースが破壊的変更になる。
ただし既存のコードでは `let` を識別子として使っているケースはないはず
（`bind` が主力のため）。

念のため `grep -r "let " fav/src/` で Favnir ファイル内の使用確認をする。

---

## 実装順序まとめ

1. `lexer.rs` — `TokenKind::Let` 追加
2. `ast.rs` — `Stmt::Let` 追加
3. `parser.rs` — `let name = expr` パース
4. `checker.rs` — E0326 チェック追加
5. `compiler.rs` — `Stmt::Let` の StoreLocal コンパイル
6. exhaustive match 追加（fmt / emit_python / lineage / lint / driver）
7. self-hosted 対応（compiler.fav / checker.fav）
8. テスト追加
9. バージョン更新

Step 6（exhaustive match）は Step 2 の直後に行うと clippy エラーを早期に解消できる。

---

## リスク・注意点

- **`let` のキーワード化**: 既存コードで `let` が Ident として使われていないか確認が必要。
  もし使われていた場合、その箇所を rename する。
- **`Stmt::Bind` との型チェックの差異**: `bind` は `Result<T, E>` の `T` をスコープに追加するが、
  `let` は型をそのまま追加する。混乱しないよう checker の実装を慎重に分ける。
- **self-hosted compiler.fav**: `parse_stmt` で `"let"` トークンの場合分けを追加。
  Favnir の `compiler.fav` が `bind` 文と `let` 文を区別できるよう対応必須。
- **E0326 は型チェックエラー**: コンパイラ（IR 生成）ではなくチェッカーが出すエラー。
  `fav run` では型チェックが Favnir 化済みのため、`checker.fav` への対応も必要。
