# Spec: v45.3.0 — `return` compiler + VM

Date: 2026-07-15
Sprint: Language Refinement (v45.1〜v46.0)

---

## 概要

v45.2.0 で型チェックを実装した `return` 構文に、コンパイル・VM 実行を実装する。
`compiler.rs` で `Stmt::Return` → `IRStmt::Return` を emit し、
`codegen.rs` で `Opcode::Return` をバイトコードに出力する。
VM はすでに `Opcode::Return (0x16)` を処理できているため、新規 VM ロジックは不要。

## 動機

```favnir
fn clamp(v: Int, lo: Int, hi: Int) -> Int {
  if v < lo { return lo }
  if v > hi { return hi }
  v
}
```

v45.2.0 の時点では `Stmt::Return` はコンパイラで空実装（stub）であり、
実行時に `return` は無視されて処理が継続する。本バージョンで早期脱出を実際に動作させる。

## 適用スコープ

`return` は `fn` ボディ・`stage` ボディで動作する（v45.2.0 checker で保証済み）。

## 変更ファイル

`IRStmt` enum への `Return` variant 追加により、`IRStmt` を `match` する全ファイルに
`IRStmt::Return(e) => { ... }` アームの追加が必要（ビルド維持）。

| ファイル | 変更内容 |
|---|---|
| `fav/src/middle/ir.rs` | `IRStmt::Return(IRExpr)` variant 追加、`collect_stmt_deps` の 1 行パターンに `\| IRStmt::Return(e)` を追記 |
| `fav/src/middle/compiler.rs` | `Stmt::Return` stub → `out.push(IRStmt::Return(compile_expr(&r.expr, ctx)))` |
| `fav/src/backend/codegen.rs` | `IRStmt::Return(expr)` → `emit_expr(expr, cg); cg.emit_opcode(Opcode::Return)` |
| `fav/src/backend/wasm_codegen.rs` | `IRStmt::Return(e)` arm を 5 箇所の match サイトに追加（`emit_stmt` は Yield と同様に UnsupportedExpr 返却、他 4 箇所は式 walk） |
| `fav/src/backend/wasm_dce.rs` | `IRStmt::Return(e)` arm 追加（Bind/Yield 等と同パターン） |
| `fav/src/backend/cranelift_aot.rs` | 変更不要（catch-all `_` アームで自動カバー済み）。コメントに `Return` を追記するのみ |
| `fav/src/middle/ast_lower_checker.rs` | 変更不要（`IRStmt` を match していない。`ast::Stmt` を match している） |
| `fav/src/driver.rs` | `v453000_tests` テストモジュール追加（2件） |
| `fav/Cargo.toml` | version `45.2.0` → `45.3.0` |
| `CHANGELOG.md` | v45.3.0 エントリ追加 |

## 設計詳細

### ir.rs — `IRStmt::Return`

```rust
/// `return expr` early exit (v45.3.0)
Return(IRExpr),
```

`IRStmt::Yield(IRExpr)` の直後に追加する。

### compiler.rs — `Stmt::Return` 実装

```rust
// Before (stub from v45.1.0):
Stmt::Return(_ret) => {} // TODO v45.3: emit Return opcode

// After:
Stmt::Return(r) => out.push(IRStmt::Return(compile_expr(&r.expr, ctx))),
```

### codegen.rs — `IRStmt::Return` emit

```rust
IRStmt::Return(expr) => {
    emit_expr(expr, cg);
    cg.emit_opcode(Opcode::Return);
}
```

`IRStmt::Yield` の処理直後に追加。`Opcode::Return (0x16)` は vm.rs で既にハンドル済み：
- スタックトップの値を pop（戻り値）
- コールフレームを pop
- フレームの `base` 位置までスタックを truncate
- 戻り値を push して呼び出し元に制御を返す

### exhaustive match 対応ファイル

- `wasm_codegen.rs` / `wasm_dce.rs`: arm を追加してビルドを維持する（WASM パスは stub）
- `cranelift_aot.rs`: catch-all `_` アームが既に存在するため arm 追加不要。コメント追記のみ
- `ast_lower_checker.rs`: `IRStmt` を match していない（`ast::Stmt` を match している）ため変更不要

### 負の値について

テスト `return_in_stage_executes` では `0 - x` を使用する（Favnir は変数への単項マイナス `-x` を
構文レベルでサポートしていないため）。負のリテラル（`-5` 等）は整数リテラルとして直接書ける。

### site/ MDX について

v45.3.0 はコンパイラ内部変更のみ。site/ MDX の追加・変更は不要。

## 完了条件

- `cargo test` 全通過（**2974 tests** passed, 0 failed）
- `v453000_tests` の 2 件が pass:
  - `return_early_exit_executes`
  - `return_in_stage_executes`
- `cargo clippy --locked -D warnings` クリーン
- `CHANGELOG.md` に v45.3.0 エントリ追加
