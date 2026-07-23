# Tasks: v45.1.0 — `return` 構文 AST + parser

Status: COMPLETE
Date: 2026-07-15

---

## T0 — 事前確認

- [x] `cargo test` 2966 passed, 0 failed を確認

## T1 — lexer.rs: Return TokenKind 追加

- [x] `TokenKind` enum に `Return` variant 追加
- [x] `"return" => TokenKind::Return` キーワードマッピング追加
- [x] reserved words リストに `"return"` 追加（存在する場合）— 別リスト不在のため不要

## T2 — ast.rs: ReturnStmt + Stmt::Return 追加

- [x] `ReturnStmt` 構造体追加
- [x] `Stmt::Return(ReturnStmt)` variant 追加
- [x] `Stmt::span()` に `Return` アーム追加

## T3 — parser.rs: return 構文解析

- [x] `parse_return_stmt()` 関数追加（`parse_yield_stmt()` の span パターンに倣う）
- [x] `parse_block()` に `return` 分岐追加（`yield` 分岐の直後）

## T4 — exhaustive match 対応（ビルド維持）

- [x] `fav/src/fmt.rs` に `Stmt::Return` アーム追加
- [x] `fav/src/emit_python.rs` に `Stmt::Return` アーム追加
- [x] `fav/src/lineage.rs` に `Stmt::Return` アーム追加（全箇所: sql/pg/azure/azure_blob/sf の5関数）
- [x] `fav/src/lint.rs` に `Stmt::Return` アーム追加（全箇所: 18箇所すべて対応）
- [x] `fav/src/lsp/references.rs` に `Stmt::Return` アーム追加
- [x] `fav/src/middle/checker.rs` に `Stmt::Return` アーム追加（stub + collect helper 2箇所）
- [x] `fav/src/middle/compiler.rs` に `Stmt::Return` アーム追加（free_vars + compile_stmt の2箇所）

## T5 — driver.rs: テストモジュール + バージョン更新

- [x] `fav/Cargo.toml` version → `45.1.0`
- [x] `v45000_tests::cargo_toml_version_is_45_0_0` スタブ化
- [x] `v451000_tests` モジュール追加（2件）
  - [x] `return_stmt_parses` — `fn early(x: Int) -> Int { return 42; x }` でトップレベル Return を検証
  - [x] `single_expr_body_no_return_needed` — 単一式ボディは stmts 空を確認

## T6 — テスト＆完了

- [x] `cargo test` 2968 passed, 0 failed
- [x] `cargo clippy -- -D warnings` クリーン
- [x] `CHANGELOG.md` に v45.1.0 エントリ追加
- [x] tasks.md を COMPLETE に更新（T0〜T6 全チェック）

## コードレビュー指摘と対応

- [HIGH-1] exhaustive match ファイル列挙漏れ → spec/plan/tasks に全対象ファイルを追記し実装
- [HIGH-2] seq ボディでの return 禁止方針未明記 → spec に「checker.rs v45.2 で E0415 として検出」と明記
- [MED-1] tasks.md テスト名列挙漏れ → T5 にサブ項目で明記
- [MED-2] v45.9.0 テスト名不整合 → roadmap-v45.1-v46.0.md を修正（`cargo_toml_version_is_45_9_0` → `examples_structure_valid`）
- [LOW-1] span_from 確認 → parse_yield_stmt と同パターンで問題なし
- [LOW-2] CHANGELOG/tasks 完了 → 本タスクで対応
