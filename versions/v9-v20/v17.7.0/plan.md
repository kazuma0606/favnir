# v17.7.0 — `forall` プロパティベーステスト 実装計画

## 方針

`forall x: Type { body }` 構文を **AST レベルで `Stmt::Forall` として保持**し、
コンパイル時に **`ForIn` ループへデシュガー**する。
新 VM opcode は不要。値生成は VM Primitive（`__forall_gen_*`）が担う。

変更は lexer / ast / parser / checker / compiler / vm / exhaustive match 箇所 / main.rs / driver.rs。

---

## 実装ステップ

### Step 1: Lexer — `TokenKind::Forall` 追加

`fav/src/frontend/lexer.rs`:

```rust
// TokenKind enum に追加
Forall,  // "forall" keyword (v17.7.0)

// キーワードマッチに追加
"forall" => TokenKind::Forall,
```

---

### Step 2: AST — `Stmt::Forall` / `ForallStmt` / `ForallVar` 追加

`fav/src/ast.rs`:

```rust
// Stmt enum に追加
pub enum Stmt {
    // ... 既存 ...
    Forall(ForallStmt),
}

// Stmt::span() に追加
Stmt::Forall(f) => &f.span,

// 新構造体
pub struct ForallStmt {
    pub vars: Vec<ForallVar>,
    pub guard: Option<Expr>,
    pub body: Block,
    pub span: Span,
}

pub struct ForallVar {
    pub name: String,
    pub ty: TypeExpr,
    pub span: Span,
}
```

---

### Step 3: Parser — `parse_forall_stmt` 追加

`fav/src/frontend/parser.rs`:

#### 構文

```
forall <var>: <Type> [, <var>: <Type>]* [where { <expr> }] { <body> }
```

#### 実装

```rust
fn parse_forall_stmt(&mut self) -> Result<ForallStmt, ParseError> {
    let start = self.peek_span().clone();
    self.expect(&TokenKind::Forall)?;

    // 変数リスト: "x: Int" [, "y: String"]
    let mut vars = Vec::new();
    loop {
        let vstart = self.peek_span().clone();
        let (name, _) = self.expect_ident()?;
        self.expect(&TokenKind::Colon)?;
        let ty = self.parse_type_expr()?;
        vars.push(ForallVar { name, ty, span: self.span_from(&vstart) });
        if self.peek() != &TokenKind::Comma { break; }
        self.advance(); // consume ','
    }

    // オプション: where { guard }
    let guard = if self.peek_ident_text("where") {
        self.advance();
        self.expect(&TokenKind::LBrace)?;
        let expr = self.parse_expr()?;
        self.expect(&TokenKind::RBrace)?;
        Some(expr)
    } else {
        None
    };

    // body: { stmts }
    let body = self.parse_block()?;
    Ok(ForallStmt { vars, guard, body, span: self.span_from(&start) })
}

// parse_block_stmts に追加（bind より前に検出）
if self.peek() == &TokenKind::Forall {
    let forall_stmt = self.parse_forall_stmt()?;
    stmts.push(Stmt::Forall(forall_stmt));
    continue;
}
```

---

### Step 4: Checker — `Stmt::Forall` 型検査

`fav/src/middle/checker.rs`:

```rust
Stmt::Forall(f) => {
    // 変数型が対応型かを確認（Int / Float / String / Bool）
    for var in &f.vars {
        let ty = self.resolve_type_expr(&var.ty);
        match &ty {
            Type::Int | Type::Float | Type::Str | Type::Bool => {}
            _ => self.type_error(
                "E0327",
                &format!("forall does not support type `{}` in v17.7.0; \
                          supported: Int, Float, String, Bool", self.type_to_string(&ty)),
                &var.span,
            ),
        }
        // 変数をスコープに追加してボディを型チェック
        self.env.define(var.name.clone(), ty);
    }
    if let Some(guard) = &f.guard {
        self.check_expr(guard); // Bool が期待されるが強制はしない
    }
    self.check_block(&f.body);
}
```

`collect_helpers_in_stmt` / `scan_expr_for_pipeline_calls` にも `Stmt::Forall` アームを追加（各フィールドを再帰的に処理）。

---

### Step 5: VM Primitive — `__forall_gen_*` 追加

VM に新プリミティブを 4 種追加する。
`compiler.rs` のグローバル builtin 名前テーブル（2 箇所）と `vm.rs` の `exec_builtin` に追加。

```rust
// compiler.rs の builtin リストに追加:
"__forall_gen_int",
"__forall_gen_str",
"__forall_gen_bool",
"__forall_gen_float",

// vm.rs の exec_builtin に追加:
BuiltinPrimitive::ForallGenInt => {
    let n = stack.pop().as_int().max(1) as usize;
    let mut vals = vec![
        Value::Int(0), Value::Int(1), Value::Int(-1),
        Value::Int(i64::MAX), Value::Int(i64::MIN),
    ];
    // 疑似乱数（xorshift64 で決定論的に生成）
    let mut seed: u64 = 0xdeadbeefcafe1234;
    while vals.len() < n {
        seed ^= seed << 13; seed ^= seed >> 7; seed ^= seed << 17;
        vals.push(Value::Int(seed as i64));
    }
    vals.truncate(n);
    stack.push(Value::List(std::rc::Rc::new(vals)));
}

BuiltinPrimitive::ForallGenStr => {
    let n = stack.pop().as_int().max(1) as usize;
    let edge = ["", " ", "a", "\n", "hello world"];
    let mut vals: Vec<Value> = edge.iter()
        .map(|s| Value::Str(s.to_string()))
        .collect();
    let mut seed: u64 = 0x1234567890abcdef;
    while vals.len() < n {
        seed ^= seed << 13; seed ^= seed >> 7; seed ^= seed << 17;
        let len = (seed >> 56) as usize % 20;
        let s: String = (0..len).map(|_| {
            seed ^= seed << 13; seed ^= seed >> 7; seed ^= seed << 17;
            (32 + (seed % 95) as u8) as char
        }).collect();
        vals.push(Value::Str(s));
    }
    vals.truncate(n);
    stack.push(Value::List(std::rc::Rc::new(vals)));
}

BuiltinPrimitive::ForallGenBool => {
    let n = stack.pop().as_int().max(1) as usize;
    let vals: Vec<Value> = (0..n).map(|i| Value::Bool(i % 2 == 0)).collect();
    stack.push(Value::List(std::rc::Rc::new(vals)));
}

BuiltinPrimitive::ForallGenFloat => {
    let n = stack.pop().as_int().max(1) as usize;
    let edge = [0.0f64, 1.0, -1.0, 0.5, -0.5];
    let mut vals: Vec<Value> = edge.iter().map(|&f| Value::Float(f)).collect();
    let mut seed: u64 = 0xfedcba9876543210;
    while vals.len() < n {
        seed ^= seed << 13; seed ^= seed >> 7; seed ^= seed << 17;
        // [−1e6, 1e6] の範囲に正規化
        let f = (seed as i64 as f64) / (i64::MAX as f64) * 1_000_000.0;
        vals.push(Value::Float(f));
    }
    vals.truncate(n);
    stack.push(Value::List(std::rc::Rc::new(vals)));
}
```

---

### Step 6: Compiler — `Stmt::Forall` デシュガー

`fav/src/middle/compiler.rs`:

`compile_stmt_into` に `Stmt::Forall` アームを追加。
`collect_free_vars_block` にも追加。

```rust
Stmt::Forall(f) => {
    // v17.7.0: 単一変数のみサポート
    let var = &f.vars[0];
    let cases = get_forall_cases(); // env var FORALL_CASES (default 100)

    // 型に対応するジェネレータ primitive 名を選択
    let gen_fn = match resolve_type_name(&var.ty) {
        "Int"    => "__forall_gen_int",
        "Float"  => "__forall_gen_float",
        "String" => "__forall_gen_str",
        "Bool"   => "__forall_gen_bool",
        other    => panic!("unsupported forall type: {other}"),
    };

    // guard あり: CASES * 10 件生成して内包でフィルタ、CASES 件に絞る
    // guard なし: CASES 件をそのまま使う
    let gen_count = if f.guard.is_some() { cases * 10 } else { cases };

    // デシュガー後のソース（Favnir AST を再構築して compile_expr で処理）:
    // bind __vals <- gen_fn(gen_count)
    // [for guard filter if guard present]
    // for x in __vals { body }

    // 実装: IRExpr を直接構築
    let gen_call = IRExpr::CallBuiltin(gen_fn, vec![IRExpr::Const(Value::Int(gen_count as i64))]);
    let vals_slot = ctx.define_local("__forall_vals");
    out.push(IRStmt::Bind(vals_slot, gen_call));

    // guard フィルタ（guard がある場合）
    let iter_slot = if let Some(guard) = &f.guard {
        // bind __filtered <- [v | v <- __vals, guard(v)]
        // bind __taken <- List.take(__filtered, cases)
        let iter_var_slot = ctx.define_local(&var.name); // 一時的に var のスロット確保
        let guard_ir = compile_expr(guard, ctx);
        let filter_ir = IRExpr::CallBuiltin(
            "List.filter",
            vec![
                IRExpr::Local(vals_slot),
                IRExpr::Closure(vec![iter_var_slot], vec![IRStmt::Expr(guard_ir)], Box::new(IRExpr::Local(iter_var_slot))),
            ],
        );
        let filtered_slot = ctx.define_local("__forall_filtered");
        out.push(IRStmt::Bind(filtered_slot, filter_ir));
        let take_ir = IRExpr::CallBuiltin(
            "List.take",
            vec![
                IRExpr::Local(filtered_slot),
                IRExpr::Const(Value::Int(cases as i64)),
            ],
        );
        let taken_slot = ctx.define_local("__forall_taken");
        out.push(IRStmt::Bind(taken_slot, take_ir));
        taken_slot
    } else {
        vals_slot
    };

    // for x in __iter { body }
    let body_ir = compile_block(&f.body, ctx);
    let var_slot = ctx.resolve_or_define_local(&var.name);
    out.push(IRStmt::ForIn {
        var: var_slot,
        iter: IRExpr::Local(iter_slot),
        body: body_ir,
    });
}
```

> 注: 上記は概念コード。実際の IRExpr/IRStmt の型は既存コードに合わせて調整する。
> 特に `IRExpr::Closure` が存在しない場合は、guard フィルタのみ別の方法で実装する。

**簡易実装（guard フィルタを Favnir AST 再構築で実現できない場合のフォールバック）**:

guard なし版のみ完全実装し、guard ありは「guard が偽のイテレーションをスキップ」する方式にする:

```rust
// for x in __vals { if not guard { continue-equivalent } body }
// → ForIn ループ内で if/else によるスキップ
```

---

### Step 7: Exhaustive match 追加

`Stmt::Forall` を追加したため、以下の全ファイルに `Stmt::Forall` アームを追加：

| ファイル | 関数 | 追加内容 |
|---|---|---|
| `fmt.rs` | `fmt_stmt` | `Stmt::Forall(f) => format!("forall ...")` |
| `emit_python.rs` | `emit_stmt` | `Stmt::Forall(f) => { /* for loop */ }` |
| `lineage.rs` | `collect_sql_literals_stmt` / `collect_azure_kinds_stmt` / `collect_azure_blob_kinds_stmt` / `collect_sf_kinds_stmt` | 各 guard/body を再帰処理 |
| `lint.rs` | `lint_block_l008` / `collect_block_calls` / `lint_stmt_sub_blocks` / `stmt_references` / `collect_ambient_in_block` / `collect_deprecated_in_block` / `collect_type_state_in_stmt` | 各 guard/body を再帰処理 |
| `checker.rs` | `collect_helpers_in_stmt` / `scan_expr_for_pipeline_calls` | guard/body を再帰 |
| `compiler.rs` | `collect_free_vars_block` | guard/body の自由変数収集 |

---

### Step 8: `main.rs` — `--cases N` オプション追加

```rust
// fav test --cases N
Some("test") => {
    // 既存の parse ループに追加:
    "--cases" => {
        let raw = args.get(i + 1)...;
        let n = raw.parse::<u64>()...;
        // SAFETY: set before any test threads
        unsafe { std::env::set_var("FORALL_CASES", n.to_string()) };
        i += 2;
    }
}
```

---

### Step 9: `driver.rs` — `v177000_tests` 追加

```rust
#[cfg(test)]
mod v177000_tests {
    use super::{build_artifact, exec_artifact_main};
    use crate::frontend::parser::Parser;
    use crate::value::Value;

    fn run_test(src: &str) -> Result<Value, String> {
        let program = Parser::parse_str(src, "v177000_test.fav").expect("parse");
        let artifact = build_artifact(&program);
        let fn_idx = artifact.fn_idx_by_name("$test:trim_idempotent")
            .or_else(|| artifact.fn_idx_by_name("$test:abs_nonneg"))
            .or_else(|| artifact.fn_idx_by_name("$test:main"))
            .expect("test function not found");
        crate::backend::vm::VM::run(&artifact, fn_idx, vec![])
            .map_err(|e| e.message.clone())
    }

    #[test]
    fn version_is_17_7_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("\"17.7.0\""), "Cargo.toml should have version 17.7.0");
    }

    #[test]
    fn forall_int_parses() {
        let src = r#"
test "abs is nonneg" {
  forall n: Int {
    assert_true(Math.abs(n) >= 0)
  }
}
"#;
        let prog = Parser::parse_str(src, "test.fav").expect("parse");
        let has_forall = prog.items.iter().any(|i| {
            if let crate::ast::Item::TestDef(td) = i {
                td.body.stmts.iter().any(|s| matches!(s, crate::ast::Stmt::Forall(_)))
            } else { false }
        });
        assert!(has_forall, "forall stmt should be in test body");
    }

    #[test]
    fn forall_string_idempotent() {
        // String.trim の冪等性: trim(s) == trim(trim(s))
        let src = r#"
test "trim_idempotent" {
  forall s: String {
    bind t1 <- String.trim(s)
    bind t2 <- String.trim(t1)
    assert_eq(t1, t2)
  }
}
"#;
        let prog = Parser::parse_str(src, "test.fav").expect("parse");
        let artifact = build_artifact(&prog);
        let fn_idx = artifact.fn_idx_by_name("$test:trim_idempotent").expect("fn");
        let result = crate::backend::vm::VM::run(&artifact, fn_idx, vec![]);
        assert!(result.is_ok(), "trim idempotency should pass: {:?}", result);
    }

    #[test]
    fn forall_finds_counterexample() {
        // n > 0 はすべての Int では成立しない → 失敗する
        let src = r#"
test "all_positive" {
  forall n: Int {
    assert_true(n > 0)
  }
}
"#;
        let prog = Parser::parse_str(src, "test.fav").expect("parse");
        let artifact = build_artifact(&prog);
        let fn_idx = artifact.fn_idx_by_name("$test:all_positive").expect("fn");
        let result = crate::backend::vm::VM::run(&artifact, fn_idx, vec![]);
        assert!(result.is_err(), "forall with false property should fail");
    }

    #[test]
    fn forall_with_guard() {
        // ゼロ以外の整数での性質確認
        let src = r#"
test "nonzero_sign" {
  forall n: Int where { n != 0 } {
    assert_true(n > 0 || n < 0)
  }
}
"#;
        let prog = Parser::parse_str(src, "test.fav").expect("parse");
        let artifact = build_artifact(&prog);
        let fn_idx = artifact.fn_idx_by_name("$test:nonzero_sign").expect("fn");
        let result = crate::backend::vm::VM::run(&artifact, fn_idx, vec![]);
        assert!(result.is_ok(), "nonzero int should be positive or negative: {:?}", result);
    }
}
```

---

### Step 10: バージョン更新

- `fav/Cargo.toml`: `17.6.0` → `17.7.0`
- `cargo build` で `Cargo.lock` 更新

---

### Step 11: ドキュメント

- `site/content/docs/language/property-testing.mdx` 新規作成

---

## 実装上の注意点

### `collect_free_vars_block` への追加

`Stmt::Forall` でも自由変数を正しく収集する必要がある。
`vars` の各変数名は bound（束縛済み）として扱う：

```rust
Stmt::Forall(f) => {
    if let Some(guard) = &f.guard {
        collect_free_vars_expr(guard, &mut local_bound, free);
    }
    // vars は body/guard 内で束縛される
    let mut forall_bound = local_bound.clone();
    for var in &f.vars {
        forall_bound.insert(var.name.clone());
    }
    collect_free_vars_block(&f.body, &forall_bound, free);
}
```

### guard フィルタのデシュガー

`collect_result` (v17.3.0 で追加) や `List.take` が存在するかを確認してから使う。
存在しない場合は `ForIn` ループ内で `if guard { body }` の形で実装する（最も安全）。

### `ForallVar.ty` の型名取得

`TypeExpr::Name(name)` の場合に "Int" / "String" 等を取得：

```rust
fn forall_gen_fn_for(ty: &TypeExpr) -> &'static str {
    match ty {
        TypeExpr::Name(n) if n == "Int"    => "__forall_gen_int",
        TypeExpr::Name(n) if n == "Float"  => "__forall_gen_float",
        TypeExpr::Name(n) if n == "String" => "__forall_gen_str",
        TypeExpr::Name(n) if n == "Bool"   => "__forall_gen_bool",
        _ => panic!("unsupported forall type"),
    }
}
```

### `get_forall_cases()` の実装

```rust
fn get_forall_cases() -> i64 {
    std::env::var("FORALL_CASES")
        .ok()
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(100)
}
```

---

## 実装順序

1. Lexer（`TokenKind::Forall`）
2. AST（`Stmt::Forall` / `ForallStmt` / `ForallVar`）
3. Parser（`parse_forall_stmt`）
4. Checker（`Stmt::Forall` チェック + E0327）
5. VM Primitives（`__forall_gen_*` 4種）
6. Compiler（`Stmt::Forall` デシュガー + `collect_free_vars_block`）
7. Exhaustive match 追加（fmt / emit_python / lineage×4 / lint×7 / checker×2）
8. `main.rs` — `--cases N` オプション
9. `driver.rs` — `v177000_tests` 追加
10. Cargo.toml バージョン更新（17.7.0）
11. ドキュメント

---

## リスク・注意点

- **`IRExpr::Closure` が存在するか**: compiler.rs の IR 定義を確認してから guard フィルタの実装を決定。存在しない場合は `ForIn` ループ内での `if guard { body }` 方式にフォールバック。
- **`List.take` の存在**: v16.4 で実装済みのはずだが、builtin テーブルにあるか確認。なければ `List.length` + `ForIn` のカウンター方式で代替。
- **`Math.abs` の存在**: テスト `forall_int_parses` で使用。builtin にあるか確認。
- **型チェッカーの `Stmt::Forall` での変数スコープ**: `check_block` 呼び出し前に `self.env.define(var.name, ty)` を行い、テスト後にスコープを正しく抜ける。`push_scope` / `pop_scope` パターンに従う。
- **`where` の競合**: `where` は parser.rs で `is_rune_use_pattern` 等で使われていないか確認。`peek_ident_text("where")` で検出する方針。
