# Plan: v45.3.0 — `return` compiler + VM

---

## Step 0 — 事前確認

```bash
cd /c/Users/yoshi/favnir/fav && cargo test -j 8 -- --test-threads=8 2>&1 | tail -5
```

期待: `test result: ok. 2972 passed; 0 failed`

---

## Step 1 — `middle/ir.rs`: `IRStmt::Return` 追加

`IRStmt` enum に `Return(IRExpr)` variant を追加する。

追加位置: `IRStmt::Yield(IRExpr)` の直後。

```rust
Yield(IRExpr),
/// `return expr` early exit — emits Opcode::Return immediately (v45.3.0)
Return(IRExpr),
```

`collect_stmt_deps` 関数（exhaustive match）にアームを追加:

```rust
// 既存パターン（1行にまとめられている）の末尾に Return を追加:
IRStmt::Bind(_, e) | IRStmt::LegacyBind(_, e) | IRStmt::Chain(_, e)
    | IRStmt::Yield(e) | IRStmt::Return(e) | IRStmt::Expr(e) => {
    collect_expr_deps(e, globals, deps);
}
```

---

## Step 2 — exhaustive match 対応（ビルド維持）

`IRStmt::Return` 追加により、`IRStmt` を `match` する全ファイルに arm 追加が必要。
`cargo build` で漏れを確認しながら対応する。

**既知の対象ファイル:**

| ファイル | 対応方針 |
|---|---|
| `fav/src/backend/wasm_dce.rs` | `IRStmt::Return(e)` を Bind/Yield 等と同パターンで追加 |
| `fav/src/backend/wasm_codegen.rs` | 5 箇所の match サイトに `IRStmt::Return(e)` arm を追加。`emit_stmt` は `Yield` と同様に `UnsupportedStmt` を返却、残り 4 箇所は `walk_closures_in_expr(e, ir, map)` の式 walk |
| `fav/src/backend/cranelift_aot.rs` | 変更不要（catch-all `_` アームで自動カバー済み）。コメント追記のみ |
| `fav/src/middle/ast_lower_checker.rs` | 変更不要（`IRStmt` を match していない。`ast::Stmt` を match している） |

---

## Step 3 — `middle/compiler.rs`: `Stmt::Return` 実装

`compile_stmt_into` 関数の `Stmt::Return` アームを stub から実装に差し替える。

```rust
// Before (stub from v45.1.0):
Stmt::Return(_ret) => {} // TODO v45.3: emit Return opcode

// After:
Stmt::Return(r) => out.push(IRStmt::Return(compile_expr(&r.expr, ctx))),
```

---

## Step 4 — `backend/codegen.rs`: `IRStmt::Return` emit

`IRStmt::Yield` の処理直後に追加:

```rust
IRStmt::Yield(expr) => {
    emit_expr(expr, cg);
    cg.emit_opcode(Opcode::YieldValue);
}
// ↓ v45.3.0 追加
IRStmt::Return(expr) => {
    emit_expr(expr, cg);
    cg.emit_opcode(Opcode::Return);
}
```

`Opcode::Return (0x16)` は vm.rs の実行ループで既にハンドル済み:
- スタックトップの値（戻り値）を pop
- コールフレームを pop
- フレーム `base` 位置までスタックを truncate
- 戻り値を push して呼び出し元に復帰

---

## Step 5 — `driver.rs`: テストモジュール追加 + バージョン更新

### 5a. Cargo.toml: バージョン更新

```toml
version = "45.3.0"
```

### 5b. `v453000_tests` モジュールを追加

`v452000_tests` モジュールの直後に追加。`run_inline` パターンを使用
（`sql_rune_tests` モジュールと同一パターン）。

```rust
// -- v453000_tests (v45.3.0) -- return compiler + VM --
#[cfg(test)]
mod v453000_tests {
    use crate::frontend::parser::Parser;
    use crate::middle::checker::Checker;
    use crate::middle::compiler::compile_program;
    use crate::backend::codegen::codegen_program;
    use crate::backend::vm::VM;
    use crate::value::Value;

    fn run_inline(src: &str) -> Value {
        let prog = Parser::parse_str(src, "test.fav").expect("parse failed");
        let (errors, _) = Checker::check_program(&prog);
        assert!(errors.is_empty(), "type errors: {:?}",
            errors.iter().map(|e| &e.message).collect::<Vec<_>>());
        let ir = compile_program(&prog);
        let artifact = codegen_program(&ir);
        let fn_idx = artifact.fn_idx_by_name("main").expect("main not found");
        VM::run(&artifact, fn_idx, vec![]).expect("run failed")
    }

    #[test]
    fn return_early_exit_executes() {
        // clamp using early return; when v < lo, return lo immediately
        let src = r#"
fn clamp(v: Int, lo: Int, hi: Int) -> Int {
  if v < lo { return lo };
  if v > hi { return hi };
  v
}
public fn main() -> Int { clamp(-5, 0, 100) }
"#;
        let result = run_inline(src);
        assert_eq!(result, Value::Int(0),
            "clamp(-5, 0, 100) should early-return 0, got {:?}", result);
    }

    #[test]
    fn return_in_stage_executes() {
        // stage using early return
        let src = r#"
stage AbsVal: Int -> Int = |x| {
  if x < 0 { return 0 - x };
  x
}
public fn main() -> Int { AbsVal(-42) }
"#;
        let result = run_inline(src);
        assert_eq!(result, Value::Int(42),
            "AbsVal(-42) should return 42, got {:?}", result);
    }
}
```

---

## Step 6 — ビルド＆テスト

```bash
cd /c/Users/yoshi/favnir/fav && cargo test -j 8 -- --test-threads=8 2>&1 | tail -20
```

期待: `test result: ok. 2974 passed; 0 failed`

```bash
cargo clippy --locked -D warnings 2>&1 | grep -E "^error" | head -20
```

CHANGELOG.md に v45.3.0 エントリを追加する（`return` compiler + VM 実装）。
