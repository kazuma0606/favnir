# Tasks: v52.1.0 — `assert_schema` Phase 1（型チェック）

Status: COMPLETE
Date: 2026-07-20

---

## T0 — 事前確認

- [x] `cargo test` 3135 passed, 0 failed を確認（ベース確認）
- [x] `cargo clippy -- -D warnings` クリーンであることを確認
- [x] `ast.rs` に `AssertSchema` が**存在しない**ことを確認（新規追加対象）
- [x] `middle/ir.rs` に `AssertSchema` が**存在しない**ことを確認（新規追加対象）
- [x] `backend/vm.rs` に `AssertSchema` が**存在しない**ことを確認（新規追加対象）
- [x] `error_catalog.rs` の E0419 が「予約」コメントのみであることを確認（実装対象）
- [x] `v52000_tests` に `cargo_toml_version_is_52_0_0` が存在することを確認（削除対象）
- [x] `include_str!` パス確認（`fav/src/driver.rs` 起点）:
  - [x] `include_str!("ast.rs")` → `fav/src/ast.rs` ✓
  - [x] `include_str!("backend/vm.rs")` → `fav/src/backend/vm.rs` ✓
  - [x] `include_str!("error_catalog.rs")` → `fav/src/error_catalog.rs` ✓

## T1 — `ast.rs` に `Expr::AssertSchema` 追加

- [x] `Expr` enum に `AssertSchema { ty_name: String, arg: Box<Expr>, span: Span }` を追加
- [x] `cargo build` でコンパイルエラーを確認し、exhaustive match 対象ファイルを全対応:
  - [x] `fmt.rs` に `AssertSchema` アームを追加
  - [x] `lint.rs` に `AssertSchema` アームを追加（8 箇所）
  - [x] `emit_python.rs` に `AssertSchema` アームを追加
  - [x] `middle/checker.rs`（`collect_helpers_in_expr` / `check_expr` / `collect_calls_from_expr`）に `AssertSchema` アームを追加
  - [x] `middle/compiler.rs`（`collect_free_vars_expr`）に `AssertSchema` アームを追加
  - [x] `lineage.rs`（4 関数）に `AssertSchema` アームを追加
  - [x] `lsp/references.rs`（`collect_in_expr`）に `AssertSchema` アームを追加
  - [x] `driver.rs`（`format_expr_compact` / exhaustive match）に `AssertSchema` アームを追加

## T2 — `middle/ir.rs` に `IRExpr::AssertSchema` 追加

- [x] `IRExpr` enum に `AssertSchema { ty_name: String, arg: Box<IRExpr>, ty: Type }` を追加
- [x] `ty()` メソッドの OR パターンに `IRExpr::AssertSchema { ty, .. }` を追加
- [x] `collect_expr_deps` に `AssertSchema` アームを追加

## T3 — `middle/compiler.rs` に変換ロジック追加

- [x] `compile_expr` の `Expr::AssertSchema` アームを追加し `IRExpr::AssertSchema` に変換

## T4 — `backend/vm.rs` に実行時評価ハンドラ追加

- [x] `Opcode::AssertSchema = 0x64` を `backend/codegen.rs` に追加
- [x] `IRExpr::AssertSchema` arm を `emit_expr`（codegen.rs）に追加
- [x] `Opcode::AssertSchema as u8` ハンドラを VM dispatch ループに追加
  - [x] `arg` を評価して `VMValue::Record` を取得
  - [x] スキーマ照合ロジックを実装（FieldMeta.ty vs vmvalue_type_name）
  - [x] 不一致時は `err_vm(VMValue::Str("E0419: ..."))` を返す
  - [x] 一致時は `ok_vm(val)` を返す

## T5 — `backend/wasm_dce.rs` の exhaustive match 対応

- [x] `collect_expr_fns` に `IRExpr::AssertSchema { arg, .. }` アームを追加

## T6 — `backend/wasm_codegen.rs` の exhaustive match 対応

- [x] `walk_closures_in_expr` に `IRExpr::AssertSchema` アームを追加
- [x] `collect_local_types` に `IRExpr::AssertSchema` アームを追加
- [x] `collect_expr_string_literals` に `IRExpr::AssertSchema` アームを追加（ty_name もインターン）
- [x] `walk_expr` に `IRExpr::AssertSchema` アームを追加
- [x] `emit_wasm_expr` に `IRExpr::AssertSchema` → `UnsupportedExpr` アームを追加

## T7 — `error_catalog.rs` に E0419 定義

- [x] 既存の「予約」コメントを `ErrorEntry { code: "E0419", title: "assert_schema type mismatch", ... }` に置き換える

## T8 — `driver.rs` にテスト追加 + バージョン更新

- [x] `v52100_tests` モジュールを `v52000_tests` の直前に追加（2 件）:
  - [x] `assert_schema_type_ok`
  - [x] `assert_schema_type_fail`
- [x] `v52000_tests` から `cargo_toml_version_is_52_0_0` を削除
- [x] `fav/Cargo.toml` version → `"52.1.0"`
- [x] `cargo test` 実行 → 3136 passed, 0 failed を確認
- [x] `cargo clippy -- -D warnings` クリーンを確認

## T9 — 後処理

- [x] `CHANGELOG.md` に v52.1.0 エントリ追加
- [x] `versions/current.md` を v52.1.0（3136 tests）に更新
- [x] `roadmap-v52.1-v53.0.md` の v52.1.0 実績欄を更新
- [x] tasks.md を COMPLETE に更新（T0〜T9 全 `[x]`）
