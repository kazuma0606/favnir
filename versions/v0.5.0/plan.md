# Favnir v0.5.0 実装計画

更新日: 2026-04-29

---

## Phase 1: Lexer / AST の拡張

### 新キーワード

`lexer.rs` に 4 つのキーワードを追加する。

| トークン | キーワード |
|---|---|
| `TokenKind::Chain`   | `chain`   |
| `TokenKind::Yield`   | `yield`   |
| `TokenKind::Collect` | `collect` |
| `TokenKind::Where`   | `where`   |

既存の `keywords()` 関数に追加するだけ。

### `Effect::Trace`

`ast.rs` の `Effect` 列挙体に `Trace` バリアントを追加する。

```rust
pub enum Effect {
    // 既存 ...
    Trace,
}
```

`Effect::display()` で `"Trace"` を返すアームを追加する。

### `Stmt` の拡張

`ast.rs` の `Stmt` 列挙体に 2 バリアントを追加する:

```rust
pub enum Stmt {
    // 既存
    Bind  { name: String, expr: Expr, span: Span },
    Expr  { expr: Expr, span: Span },
    // 追加
    Chain { name: String, expr: Expr, span: Span },
    Yield { expr: Expr, span: Span },
}
```

`Stmt::span()` に対応するアームを追加する。

### `Expr::Collect`

`ast.rs` の `Expr` 列挙体に追加する:

```rust
pub enum Expr {
    // 既存 ...
    Collect(Box<Block>, Span),
}
```

`Expr::span()` に対応するアームを追加する。

### `MatchArm.guard`

`ast.rs` の `MatchArm` 構造体にフィールドを追加する:

```rust
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Box<Expr>>,   // 追加
    pub body: Expr,
    pub span: Span,
}
```

既存の `MatchArm` 構築箇所を全て `guard: None` に更新する。

---

## Phase 2: Parser の拡張

### `parse_stmt` の更新

ブロック内のステートメントをパースする関数で、先頭トークンが:

- `TokenKind::Chain` → `parse_chain_stmt()` を呼ぶ
- `TokenKind::Yield` → `parse_yield_stmt()` を呼ぶ

#### `parse_chain_stmt`

```rust
fn parse_chain_stmt(&mut self) -> Result<Stmt, ParseError> {
    let span = self.span();
    self.expect(&TokenKind::Chain)?;
    let name = self.expect_ident()?.0;
    self.expect(&TokenKind::LArrow)?;   // <-
    let expr = self.parse_expr()?;
    Ok(Stmt::Chain { name, expr, span })
}
```

セミコロンは消費しない（`bind` と同じ）。

#### `parse_yield_stmt`

```rust
fn parse_yield_stmt(&mut self) -> Result<Stmt, ParseError> {
    let span = self.span();
    self.expect(&TokenKind::Yield)?;
    let expr = self.parse_expr()?;
    self.expect(&TokenKind::Semicolon)?;   // yield は expression 文扱いでセミコロン必須
    Ok(Stmt::Yield { expr, span })
}
```

### `parse_primary` の更新

`TokenKind::Collect` → `parse_collect_expr()` を呼ぶ:

```rust
fn parse_collect_expr(&mut self) -> Result<Expr, ParseError> {
    let span = self.span();
    self.expect(&TokenKind::Collect)?;
    let block = self.parse_block()?;
    Ok(Expr::Collect(Box::new(block), span))
}
```

### `parse_pipe_expr` の更新

`|>` の後に `TokenKind::Match` が来た場合のデシュガー:

```rust
// |> の後
if self.peek() == &TokenKind::Match {
    self.advance();  // consume `match`
    let arms = self.parse_match_arms()?;
    lhs = Expr::Match(Box::new(lhs), arms, span);
} else {
    // 既存処理
}
```

### `parse_match_arm` の更新

パターン後、`=>` の前に `TokenKind::Where` があればガード式をパース:

```rust
fn parse_match_arm(&mut self) -> Result<MatchArm, ParseError> {
    let span = self.span();
    let pattern = self.parse_pattern()?;
    let guard = if self.peek() == &TokenKind::Where {
        self.advance();
        Some(Box::new(self.parse_expr()?))
    } else {
        None
    };
    self.expect(&TokenKind::FatArrow)?;  // =>
    let body = self.parse_expr()?;
    Ok(MatchArm { pattern, guard, body, span })
}
```

### `parse_effect` の更新

`"Trace"` → `Effect::Trace` を認識する。

---

## Phase 3: Checker の拡張

### 新フィールド

```rust
pub struct Checker {
    // 既存 ...
    chain_context: Option<Type>,   // chain コンテキスト型
    in_collect: bool,              // collect ブロック内フラグ
}
```

`new()` / `new_with_resolver()` で `chain_context: None, in_collect: false` を追加する。

### `check_fn_def` の更新

戻り型を解決後、`chain_context` を設定する:

```rust
let return_ty = self.resolve_type_expr(&fd.return_ty);
self.chain_context = match &return_ty {
    Type::Option(_) => Some(return_ty.clone()),
    Type::Named(n, _) if n == "Option" => Some(return_ty.clone()),
    Type::Result(_, _) => Some(return_ty.clone()),
    Type::Named(n, _) if n == "Result" => Some(return_ty.clone()),
    _ => None,
};
// ... check body ...
self.chain_context = None;  // リセット
```

### `check_stmt` の更新（または同等処理）

#### `Stmt::Chain`

```rust
Stmt::Chain { name, expr, span } => {
    let expr_ty = self.check_expr(expr);
    match &self.chain_context {
        None => {
            self.type_error("E024", "chain used outside Result/Option context", span);
            self.env.define(name.clone(), Type::Unknown);
        }
        Some(ctx) => {
            // Result<T, E> の場合: expr_ty が Result<T, E> か確認
            // Option<T>    の場合: expr_ty が Option<T>    か確認
            let inner_ty = self.check_chain_expr_type(&expr_ty, ctx, span);
            self.env.define(name.clone(), inner_ty);
        }
    }
}
```

#### `Stmt::Yield`

```rust
Stmt::Yield { expr, span } => {
    if !self.in_collect {
        self.type_error("E026", "yield used outside collect block", span);
    }
    self.check_expr(expr)  // 型は collect 側で集約
}
```

### `check_expr` の更新

#### `Expr::Collect`

```rust
Expr::Collect(block, span) => {
    let old_in_collect = self.in_collect;
    let old_in_closure = /* クロージャスコープフラグ (既存) */;
    self.in_collect = true;
    // ブロック内の yield 式の型を収集
    let elem_ty = self.check_collect_block(block);
    self.in_collect = old_in_collect;
    Type::List(Box::new(elem_ty))
}
```

`check_collect_block`: ブロックを check しながら `Stmt::Yield` の型を全て収集し、unify して要素型を返す。

クロージャ（`Expr::Closure`）に入るとき: `in_collect = false` にリセット（クロージャ内の yield は E026）。

### `check_match_arm` の更新

```rust
if let Some(guard) = &arm.guard {
    let guard_ty = self.check_expr(guard);
    if !guard_ty.is_compatible(&Type::Bool) {
        self.type_error("E027", "pattern guard must be Bool", &arm.span);
    }
}
```

### `!Trace` effect の検査

`resolve_field_access` または `check_apply` で `Trace.*` を呼び出す場合:

```rust
fn require_trace_effect(&mut self, span: &Span) {
    if !self.has_effect(|e| matches!(e, Effect::Trace)) {
        self.type_error("E010", "Trace.* requires !Trace effect", span);
    }
}
```

### `register_builtins` の更新

`Trace` 型名前空間を `Type::Named("Trace", [])` として env に追加する。
`Trace.print` / `Trace.log` を `Unknown` (汎用) 型として登録する。

---

## Phase 4: 評価器の拡張

### `EvalResult` の導入

```rust
enum EvalResult {
    Value(Value),
    Escape(Value),   // chain 早期リターン
}
```

`eval_block` の返り値を `EvalResult` に変更する。

既存コードは `EvalResult::Value(v)` でラップして対応する。

### `eval_block` の更新

```rust
fn eval_block(&mut self, block: &Block, env: &mut Env) -> EvalResult {
    for stmt in &block.stmts {
        match stmt {
            Stmt::Bind  { name, expr, .. } => { ... }
            Stmt::Expr  { expr, .. }       => { ... }
            Stmt::Chain { name, expr, span } => {
                let v = self.eval_expr(expr, env);
                match v {
                    Value::Tagged("ok", inner) | Value::Some(inner) => {
                        env.define(name.clone(), *inner);
                    }
                    Value::Tagged("err", _) | Value::None => {
                        return EvalResult::Escape(v);
                    }
                    _ => env.define(name.clone(), v),
                }
            }
            Stmt::Yield { expr, .. } => {
                let v = self.eval_expr(expr, env);
                collect_yield(v);
                // EvalResult::Value(Value::Unit) で継続
            }
        }
        // Escape が返ってきたら即座に上に伝播
    }
    // tail expr
}
```

### `eval_call` / `eval_closure` の更新

関数・クロージャ呼び出し内で `EvalResult::Escape` を受け取ったら、それをそのまま関数の返り値にする:

```rust
match self.eval_block(&body, &mut local_env) {
    EvalResult::Value(v)  => v,
    EvalResult::Escape(v) => v,   // 早期リターン値を関数の戻り値にする
}
```

### `Expr::Collect` の評価

```rust
Expr::Collect(block, _) => {
    collect_push_frame();
    self.eval_block(block, env);
    let items = collect_pop_frame();
    Value::List(items)
}
```

`EvalResult::Escape` が block から返ってきた場合はフレームをポップしてから伝播させる。

### `COLLECT_STACK` の追加

```rust
thread_local! {
    static COLLECT_STACK: RefCell<Vec<Vec<Value>>> = RefCell::new(Vec::new());
}

fn collect_push_frame()              { COLLECT_STACK.with(|s| s.borrow_mut().push(vec![])); }
fn collect_yield(v: Value)           { COLLECT_STACK.with(|s| s.borrow_mut().last_mut().unwrap().push(v)); }
fn collect_pop_frame() -> Vec<Value> { COLLECT_STACK.with(|s| s.borrow_mut().pop().unwrap_or_default()) }
```

### `Trace.print` / `Trace.log` の実装

`register_builtins` で `Trace` 名前空間を登録:

```rust
env.define("Trace", Value::Namespace("Trace"));
```

`eval_field_access("Trace", field)`:

```rust
"print" => Value::Builtin("trace_print", ""),
"log"   => Value::Builtin("trace_log", ""),
```

`eval_builtin` に追加:

```rust
("trace_print", _, args) => {
    let repr = args[0].repr();
    eprintln!("{}", repr);
    args[0].clone()
}
("trace_log", _, args) => {
    let label = args[0].as_string().unwrap_or_default();
    let repr  = args[1].repr();
    eprintln!("{}: {}", label, repr);
    args[1].clone()
}
```

### `match` 評価でのガード対応

```rust
for arm in arms {
    if let Some(bindings) = pattern_match(&arm.pattern, &value) {
        // ガードがあれば評価
        if let Some(guard) = &arm.guard {
            let mut arm_env = env.extend_with(&bindings);
            if self.eval_expr(guard, &mut arm_env) != Value::Bool(true) {
                continue;  // ガード false → 次のアーム
            }
        }
        // body を評価
        ...
    }
}
```

---

## Phase 5: テストとサンプル

### 単体テスト

- `test_parse_chain_stmt` — `chain x <- expr` のパース
- `test_parse_yield_stmt` — `yield expr` のパース
- `test_parse_collect_expr` — `collect { ... }` のパース
- `test_parse_match_guard` — `x where x > 0 => ...` のパース
- `test_parse_pipe_match` — `expr |> match { ... }` のパース
- `test_chain_result_ok` — checker: `chain` が Result コンテキストで通る
- `test_chain_outside_context` — checker: E024 が出る
- `test_chain_type_mismatch` — checker: E025 が出る
- `test_yield_outside_collect` — checker: E026 が出る
- `test_guard_non_bool` — checker: E027 が出る
- `test_eval_chain_ok` — 評価器: `chain` が `ok(v)` で継続する
- `test_eval_chain_escape` — 評価器: `chain` が `err(e)` で早期リターンする
- `test_eval_collect_yield` — 評価器: `collect { yield 1; yield 2; }` → `[1, 2]`
- `test_eval_match_guard` — 評価器: guard が false なら次のアームを試みる
- `test_eval_pipe_match` — 評価器: `n |> match { ... }` が動く

### サンプルファイル

- `examples/chain.fav` — chain + Result / Option の伝播
- `examples/collect.fav` — collect / yield によるリスト構築
- `examples/pipe_match.fav` — pipe match + pattern guard の組み合わせ
