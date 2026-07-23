# Tasks: v45.3.0 — `return` compiler + VM

Status: COMPLETE
Date: 2026-07-15

---

## T0 — 事前確認

- [x] `cargo test` 2972 passed, 0 failed を確認

## T1 — `middle/ir.rs`: `IRStmt::Return` 追加

- [x] `IRStmt::Return(IRExpr)` variant を `Yield` の直後に追加
- [x] `collect_stmt_deps` の exhaustive match に `Return(e)` アームを追加

## T2 — exhaustive match 対応（ビルド維持）

- [x] `cargo build` で missing arm エラーを全件確認
- [x] `fav/src/backend/wasm_dce.rs` に `IRStmt::Return(e)` arm 追加
- [x] `fav/src/backend/wasm_codegen.rs` に `IRStmt::Return(e)` arm 追加（6箇所: `emit_stmt` は UnsupportedStmt、残り 5 箇所は式 walk）
- [x] `fav/src/backend/cranelift_aot.rs` — 変更不要（catch-all `_` で自動カバー）
- [x] `fav/src/middle/ast_lower_checker.rs` — 変更不要（`IRStmt` を match していない）
- [x] `fav/src/driver.rs` の `collect_tracklines_in_expr` / `remap_ir_stmt` にも arm 追加

## T3 — `middle/compiler.rs`: `Stmt::Return` 実装

- [x] `Stmt::Return(_ret) => {}` stub を `out.push(IRStmt::Return(compile_expr(&r.expr, ctx)))` に差し替え

## T4 — `backend/codegen.rs`: `IRStmt::Return` emit

- [x] `IRStmt::Return(expr)` アームを追加（`emit_expr + Opcode::Return`）

## T5 — `driver.rs`: テストモジュール + バージョン更新

- [x] `fav/Cargo.toml` version → `45.3.0`
- [x] `v453000_tests` モジュール追加（2件）
  - [x] `run_inline` ヘルパー定義
  - [x] `return_early_exit_executes`
  - [x] `return_in_stage_executes`

## T6 — テスト＆完了

- [x] `cargo test` 2974 passed, 0 failed
- [x] `cargo clippy -- -D warnings` クリーン
- [x] `CHANGELOG.md` に v45.3.0 エントリ追加
- [x] tasks.md を COMPLETE に更新（T0〜T6 全チェック）

## コードレビュー指摘と対応

- [HIGH] `driver.rs` に IRStmt match サイトが 2 箇所（`collect_tracklines_in_expr`, `remap_ir_stmt`）あり、spec 記載外だったが `cargo build` で検出・対応
- [HIGH] `wasm_codegen.rs` match サイトが spec 記載の「5箇所」ではなく実際は 6 箇所だった → 全箇所追加で対応
- [MED] `checker.rs` の `std::mem::replace(..., Some(...))` が clippy `mem_replace_option_with_some` に引っかかる → `Option::replace()` に変更
