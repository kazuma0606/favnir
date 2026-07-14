# v41.7.0 実装計画 — W030 lint: 冗長 refinement ガード検出

## 前提（確認済み）

- `lint.rs` の最後の lint ルールは W025（`check_w025_schema_mismatch`）
- W026〜W029 は未割り当て（空き番号）
- `TypeDef.invariants: Vec<Expr>` — v41.1.0 で追加済み
- `Expr::Closure(Vec<String>, Box<Expr>, Span)` — クロージャ構文
- `Expr::BinOp(BinOp, Box<Expr>, Box<Expr>, Span)` — 二項演算
- `BinOp::GtEq, Gt, LtEq, Lt, Eq, NotEq` が利用可能
- `TypeBody::Alias(TypeExpr)` — refinement 対象の type alias
- `run_lint` 内で `check_w025_schema_mismatch` の後に追加する
- v41600_tests の `cargo_toml_version_is_41_6_0` を次バージョン bump 時にスタブ化する
- **[spec-reviewer 確認済み]** `Block` 構造体は `expr: Box<Expr>`（`return_expr: Option<...>` は存在しない）
- **[spec-reviewer 確認済み]** `Lit::Int(i64)` / `Lit::Float(f64)` — 単フィールドバリアント（`_` 不要）
- **[spec-reviewer 確認済み]** `TypeExpr::Named(String, Vec<TypeExpr>, Span)` — 3 フィールド
- **[spec-reviewer 確認済み]** `Stmt::Expr(Expr::If(cond, _, _, span))` パターンは正しい（`Stmt::If` は存在しない）

---

## 実装ステップ

### Step 1: lint.rs — `collect_refinement_aliases` 追加

**追加位置**: `check_w025_schema_mismatch` 関数の直前（ファイル末尾付近）

```rust
use std::collections::HashMap;

/// type alias refinement の情報: closure_param, op, lhs, rhs
type RefinementInfo = (String, BinOp, Box<Expr>, Box<Expr>);

fn collect_refinement_aliases(program: &Program) -> HashMap<String, RefinementInfo> {
    let mut map = HashMap::new();
    for item in &program.items {
        if let Item::TypeDef(td) = item {
            if !matches!(td.body, TypeBody::Alias(_)) {
                continue;
            }
            if td.invariants.is_empty() {
                continue;
            }
            if let Expr::Closure(params, body, _) = &td.invariants[0] {
                if params.len() != 1 {
                    continue;
                }
                if let Expr::BinOp(op, lhs, rhs, _) = body.as_ref() {
                    map.insert(
                        td.name.clone(),
                        (params[0].clone(), op.clone(), lhs.clone(), rhs.clone()),
                    );
                }
            }
        }
    }
    map
}
```

**注意**: `HashMap` は `use std::collections::HashMap;` が lint.rs 内の他関数（W020）で既に使用されているため重複確認すること。

---

### Step 2: lint.rs — `check_w030_fn` ヘルパー追加

```rust
fn check_w030_fn(
    fd: &FnDef,
    refinements: &HashMap<String, RefinementInfo>,
    errors: &mut Vec<LintError>,
) {
    // param_name → (closure_param, op, inv_lhs, inv_rhs) のマップを構築
    let mut param_refinements: HashMap<String, &RefinementInfo> = HashMap::new();
    for param in &fd.params {
        if let TypeExpr::Named(type_name, _, _) = &param.ty {
            if let Some(info) = refinements.get(type_name) {
                param_refinements.insert(param.name.clone(), info);
            }
        }
    }
    if param_refinements.is_empty() {
        return;
    }

    for stmt in &fd.body.stmts {
        if let Stmt::Expr(Expr::If(cond, _, _, span)) = stmt {
            check_w030_cond(cond, &param_refinements, span, errors);
        }
    }
    // 末尾式が if の場合も検出（Block.expr は Option ではなく常に存在）
    if let Expr::If(cond, _, _, span) = fd.body.expr.as_ref() {
        check_w030_cond(cond, &param_refinements, span, errors);
    }
}

fn check_w030_cond(
    cond: &Expr,
    param_refinements: &HashMap<String, &RefinementInfo>,
    span: &crate::frontend::lexer::Span,
    errors: &mut Vec<LintError>,
) {
    if let Expr::BinOp(if_op, lhs, rhs, _) = cond {
        // パターン: param op literal
        if let Expr::Ident(param_name, _) = lhs.as_ref() {
            if let Some((_, inv_op, inv_lhs, inv_rhs)) = param_refinements.get(param_name) {
                // inv_lhs が Ident(closure_param) かつ inv_rhs がリテラル
                if let (Expr::Ident(_, _), _) = (inv_lhs.as_ref(), inv_rhs.as_ref()) {
                    if if_op == inv_op && exprs_lit_eq(rhs, inv_rhs) {
                        errors.push(LintError::new(
                            "W030",
                            format!(
                                "redundant guard: `{}` already has refinement type constraint that guarantees this condition",
                                param_name
                            ),
                            span.clone(),
                        ));
                        return;
                    }
                }
            }
        }
        // パターン: literal op param（左右逆）
        if let Expr::Ident(param_name, _) = rhs.as_ref() {
            if let Some((_, inv_op, inv_lhs, inv_rhs)) = param_refinements.get(param_name) {
                if let (_, Expr::Ident(_, _)) = (inv_lhs.as_ref(), inv_rhs.as_ref()) {
                    if if_op == inv_op && exprs_lit_eq(lhs, inv_lhs) {
                        errors.push(LintError::new(
                            "W030",
                            format!(
                                "redundant guard: `{}` already has refinement type constraint that guarantees this condition",
                                param_name
                            ),
                            span.clone(),
                        ));
                    }
                }
            }
        }
    }
}

/// 2 つの式がリテラルとして等しいか（Int/Float/Bool のみ対象）
/// NOTE: ast.rs の Lit enum は Lit::Int(i64), Lit::Float(f64), Lit::Bool(bool) — 単フィールド
fn exprs_lit_eq(a: &Expr, b: &Expr) -> bool {
    match (a, b) {
        (Expr::Lit(Lit::Int(x), _), Expr::Lit(Lit::Int(y), _)) => x == y,
        (Expr::Lit(Lit::Float(x), _), Expr::Lit(Lit::Float(y), _)) => x == y,
        (Expr::Lit(Lit::Bool(x), _), Expr::Lit(Lit::Bool(y), _)) => x == y,
        _ => false,
    }
}
```

---

### Step 3: lint.rs — `check_w030_redundant_refinement_guard` + `run_lint` 組み込み

```rust
// ── W030: redundant_refinement_guard (v41.7.0) ───────────────────────────────
pub fn check_w030_redundant_refinement_guard(program: &Program, errors: &mut Vec<LintError>) {
    let refinements = collect_refinement_aliases(program);
    for item in &program.items {
        if let Item::FnDef(fd) = item {
            check_w030_fn(fd, &refinements, errors);
        }
    }
}
```

`run_lint` に追加（`check_w025_schema_mismatch` 呼び出しの直後）:

```rust
// v41.7.0: W030
check_w030_redundant_refinement_guard(program, &mut errors);
```

---

### Step 4: driver.rs テストモジュール更新

- `v41600_tests::cargo_toml_version_is_41_6_0` をスタブ化
- `v41700_tests` モジュール（2 テスト）を末尾に追加:
  - `cargo_toml_version_is_41_7_0`（NOTE コメント付き）
  - `lint_w030_redundant_guard_detected`

---

### Step 5: Cargo.toml バージョン bump

`version = "41.6.0"` → `"41.7.0"`

---

### Step 6: CHANGELOG.md 更新

```markdown
## [v41.7.0] — 2026-07-11

### Added
- W030 lint: refinement 条件の冗長ガード検出（`type PositiveInt = Int where |v| v >= 0` の変数に `if x >= 0` ガードを書くと W030）
- `lint.rs`: `check_w030_redundant_refinement_guard` / `collect_refinement_aliases` / `check_w030_fn` / `exprs_lit_eq`
```

---

## 実装順序

1. `lint.rs` Step 1〜3（lint 実装） → `cargo build` 確認
2. `Cargo.toml` Step 5 → バージョン bump
3. `CHANGELOG.md` Step 6
4. `driver.rs` Step 4（テスト追加）
5. `cargo test` 実行・確認

---

## リスク

| リスク | 影響 | 対策 |
|---|---|------|
| `use std::collections::HashMap` 重複 | コンパイルエラー | W020 実装部で既に import されているか確認し、なければ追加 |
| `Lit::Int` / `Lit::Float` の単フィールド | ✅ plan.md 修正済み — `Lit::Int(x)` 形式を使用 | — |
| `Block.expr` への参照 | ✅ plan.md 修正済み — `fd.body.expr.as_ref()` を使用 | — |
| `TypeExpr::Named` 3 フィールド | ✅ plan.md 修正済み — `Named(type_name, _, _)` | — |
