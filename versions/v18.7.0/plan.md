# v18.7.0 実装計画 — 型レベル定数（Const Generics）

Date: 2026-06-16

## 実装順序

```
T1: ast.rs — GenericParam に const フィールド追加・TypeExpr::ConstInt 追加
    ↓
T2: 波及ファイル exhaustive match 修正（cargo build が通るまで）
    ↓
T3: parser.rs — const N: Int パース・整数リテラル型引数パース
    ↓
T4: checker.rs — E0335 const 制約違反チェック
    ↓
T5: v187000_tests 追加（5件）
    ↓
T6: Cargo.toml バージョン更新（18.6.0 → 18.7.0）
    ↓
T7: site/content/docs/language/const-generics.mdx 作成
```

---

## T1: `fav/src/ast.rs` — 型追加

### 1-A: `GenericParam` にフィールド追加

既存の struct に3フィールドを末尾に追加:

```rust
pub struct GenericParam {
    pub name: String,
    pub bounds: Vec<TypeConstraint>,
    pub variance: Variance,
    pub is_const: bool,                      // v18.7.0
    pub const_ty: Option<TypeExpr>,          // v18.7.0: `Int` etc.
    pub const_constraint: Option<Box<Expr>>, // v18.7.0: `where { N > 0 }`
}
```

`unbounded()` コンストラクタ更新:

```rust
pub fn unbounded(name: impl Into<String>) -> Self {
    Self {
        name: name.into(),
        bounds: vec![],
        variance: Variance::Invariant,
        is_const: false,
        const_ty: None,
        const_constraint: None,
    }
}
```

### 1-B: `TypeExpr::ConstInt` 追加

```rust
pub enum TypeExpr {
    // ...既存バリアント...
    /// Integer constant in type argument position: `f::<100>(...)` (v18.7.0)
    ConstInt(i64, Span),
}
```

`TypeExpr::span()` に追加:

```rust
TypeExpr::ConstInt(_, s) => s,
```

---

## T2: 波及ファイル修正

### 2-A: `GenericParam { ... }` struct リテラル修正

`is_const: false, const_ty: None, const_constraint: None` を全箇所に追加。

`parse_type_params` / `parse_variance_type_params` の `GenericParam { name, bounds, variance }` → 拡張版に更新。
`GenericParam::unbounded()` 経由の箇所は自動的に正しいので修正不要。

**影響ファイル（予想）:**
- `fav/src/frontend/parser.rs` — `parse_type_params` / `parse_variance_type_params` 内の構造体リテラル（4〜6箇所）
- `fav/src/driver.rs` — テスト内の `GenericParam { ... }` リテラル（あれば）

### 2-B: `TypeExpr::ConstInt` exhaustive match 修正

`TypeExpr` を exhaustive match している全箇所に `ConstInt(_, _) => ...` を追加。

**影響ファイル（予想）:**
- `fav/src/fmt.rs` — `type_expr()` / `fmt_type_expr_simple()`
- `fav/src/emit_python.rs` — `type_expr_to_python_str()`
- `fav/src/middle/ast_lower_checker.rs` — `lower_te()` / `te_to_string()`
- `fav/src/middle/compiler.rs` — `lower_type_expr_with_subst()` / `substitute_self_in_type_expr()` / 3番目の `lower_type_expr()`
- `fav/src/middle/checker.rs` — `resolve_type_expr_with_subst()` / `resolve_type_expr_with_self()` / `validate_type_expr_arity()` / `type_expr_contains()`
- `fav/src/driver.rs` — `format_type_expr()` / `favnir_type_display()` / `graphql_type_from_type_expr_nonnull()` / `proto_type_from_type_expr_nonwrapper()` / `favnir_type_to_sql_from_expr()`

**各バリアントの処理方針:**
- `fmt.rs`: `ConstInt(n, _) => format!("{}", n)`
- `emit_python.rs`: `ConstInt(_, _) => "int".to_string()`
- `ast_lower_checker.rs lower_te`: `ConstInt(n, _) => v1("TeConst", IRValue::Int(*n))`（または `v1("TeInt", ...)` 等）
- `compiler.rs lower_type_expr_*`: `ConstInt(_, _) => Type::Int`（定数は型として Int 扱い）
- `checker.rs resolve_type_expr_*`: `ConstInt(_, _) => Type::Int`
- `checker.rs validate_type_expr_arity`: `ConstInt(_, _) => {}`（検証不要）
- `checker.rs type_expr_contains`: `ConstInt(_, _) => false`
- `driver.rs format_type_expr`: `ConstInt(n, _) => format!("{}", n)`
- `driver.rs favnir_type_display` 等: `ConstInt(n, _) => format!("{}", n)`

> **ビルドが通ることを確認してから T3 に進む。**

---

## T3: `fav/src/frontend/parser.rs` — パース実装

### 3-A: `parse_type_params` への `const N: Int` 対応

`parse_type_params` のループ内で、ident の前に `"const"` ソフトキーワードを検出:

```rust
// ループの先頭
if self.peek_ident_value() == Some("const") {
    self.advance(); // consume `const`
    let (name, _) = self.expect_ident()?;
    self.expect(&TokenKind::Colon)?;
    // 型（Int のみサポート）
    let const_ty = self.parse_base_type()?;
    // オプショナルな where 制約
    let const_constraint = if self.peek_ident_value() == Some("where") {
        self.advance(); // consume `where`
        self.expect(&TokenKind::LBrace)?;
        let expr = self.parse_expr()?;
        self.expect(&TokenKind::RBrace)?;
        Some(Box::new(expr))
    } else {
        None
    };
    params.push(GenericParam {
        name,
        bounds: vec![],
        variance: Variance::Invariant,
        is_const: true,
        const_ty: Some(const_ty),
        const_constraint,
    });
} else {
    // 既存の variance + bounds パース
    ...
}
```

`peek_ident_value()` はすでに存在するか確認してから使用（なければ `peek_ident_text()` 等の既存メソッドを使う）。

### 3-B: `parse_type_arg_list` への整数リテラル対応

```rust
fn parse_type_arg_list(&mut self) -> Result<Vec<TypeExpr>, ParseError> {
    self.expect(&TokenKind::LAngle)?;
    let mut args = vec![self.parse_type_arg()?];  // parse_type_arg に委ねる
    while self.peek() == &TokenKind::Comma {
        self.advance();
        args.push(self.parse_type_arg()?);
    }
    self.expect(&TokenKind::RAngle)?;
    Ok(args)
}

/// Parse a single type argument: either an integer literal or a type expression.
fn parse_type_arg(&mut self) -> Result<TypeExpr, ParseError> {
    if let TokenKind::Int(n) = self.peek().clone() {
        let sp = self.peek_span().clone();
        self.advance();
        Ok(TypeExpr::ConstInt(n, sp))
    } else {
        self.parse_type_expr()
    }
}
```

---

## T4: `fav/src/middle/checker.rs` — E0335 実装

### 4-A: `fn_bounds_registry` への const param 登録

`register_item_signatures` の `FnDef` 処理はすでに `fn_bounds_registry` に登録済み。
const params は `GenericParam.is_const == true` で識別できるので、追加処理不要（既存登録に含まれる）。

### 4-B: `check_type_apply` / `Expr::TypeApply` での E0335 チェック

`check_expr` の `Expr::TypeApply(func, type_args, span)` ハンドラを探し、const 制約チェックを追加:

```rust
// 対象関数名を取得
if let Expr::Ident(fn_name, _) = func.as_ref() {
    if let Some(bparams) = self.fn_bounds_registry.get(fn_name).cloned() {
        for (param, arg) in bparams.iter().zip(type_args.iter()) {
            if param.is_const {
                if let TypeExpr::ConstInt(n, _) = arg {
                    if let Some(constraint) = &param.const_constraint {
                        if let Some(false) = eval_const_expr(constraint, &param.name, *n) {
                            let constraint_str = /* format constraint */ "N > 0".to_string();
                            self.type_error_h(
                                "E0335",
                                format!(
                                    "const constraint violation: `{}` is not satisfied ({}={})",
                                    constraint_str, param.name, n
                                ),
                                span,
                                vec![format!(
                                    "`{}` requires `{}`; provide a value that satisfies the constraint",
                                    fn_name, constraint_str
                                )],
                            );
                        }
                    }
                }
            }
        }
    }
}
```

### 4-C: `eval_const_expr` フリー関数

checker.rs の末尾（`type_expr_contains` 等と同じ位置）に追加:

```rust
/// Evaluate a const constraint expression with a single variable substitution.
/// Returns Some(true/false) for evaluable expressions, None if can't evaluate statically.
fn eval_const_expr(expr: &Expr, var_name: &str, var_val: i64) -> Option<bool> {
    match expr {
        Expr::BinOp { op, lhs, rhs, .. } => {
            let l = eval_const_int(lhs, var_name, var_val)?;
            let r = eval_const_int(rhs, var_name, var_val)?;
            Some(match op.as_str() {
                ">" => l > r,
                ">=" => l >= r,
                "<" => l < r,
                "<=" => l <= r,
                "==" => l == r,
                "!=" => l != r,
                _ => return None,
            })
        }
        Expr::BinOp { op, .. } if op == "&&" => {
            // handled separately
            None
        }
        _ => None,
    }
}

fn eval_const_int(expr: &Expr, var_name: &str, var_val: i64) -> Option<i64> {
    match expr {
        Expr::Lit(Lit::Int(n), _) => Some(*n),
        Expr::Ident(name, _) if name == var_name => Some(var_val),
        _ => None,
    }
}
```

**実際の AST 構造を確認してから実装する**（`Expr::BinOp` の実際のフィールド名・`op` の型等）。

---

## T5: `v187000_tests` 追加（5件）

```rust
mod v187000_tests {
    fn version_is_18_7_0()
    fn const_generic_parses()         // GenericParam { is_const: true }
    fn const_generic_constraint_parses()  // const_constraint: Some(...)
    fn const_generic_violation()      // f::<0>() → E0335
    fn const_generic_valid()          // f::<100>() → no error
}
```

---

## T6: バージョン更新

- `fav/Cargo.toml`: `18.6.0` → `18.7.0`
- `driver.rs`: `version_is_18_6_0` に `#[ignore]`

---

## T7: ドキュメント作成

`site/content/docs/language/const-generics.mdx`:
- `const N: Int` 構文
- `where { N > 0 }` 制約
- `f::<100>(...)` 呼び出し構文
- E0335 エラー説明
- データパイプラインでのバッチサイズ制御の例

---

## 注意事項

### `Expr::BinOp` の実際の構造を確認すること

`eval_const_expr` を書く前に、`fav/src/ast.rs` で `BinOp` の実際のフィールド名と演算子の表現方法を確認する。

### `const` はソフトキーワード

`const` を `TokenKind::Const` として追加する必要はない。`peek_ident_text("const")` で判定できる。ただし既存の `peek_ident_text` の実装を確認すること。

### `TypeExpr::ConstInt` の `lower_type_expr`

コンパイラで const 引数を式の中の `N` に代入する際、`CompileCtx` に `const_subst: HashMap<String, i64>` を持たせて `Expr::Ident(n)` のコンパイル時に定数に置き換える方法が最もシンプル。ただし v18.7.0 では const param を使った関数本体のコンパイルまではスコープ外（パーサー + チェッカーレベルで完結）。

### `TypeExpr::ConstInt` の波及が最大の作業

`TypeExpr` に新バリアントを追加すると 7〜10 ファイルで exhaustive match エラーが発生する。T2 で全ファイルを一括修正することが重要。
