# v17.4.0 — `let` バインディング除去（誤実装の修正） タスク

## ステータス: 完了

---

## 背景

v17.4.0 では当初「`let x = expr` を非 Result 値のバインディングとして追加する」計画だったが、
これは設計上の誤りだった。`bind x <- expr` はもともと **任意の値型**（Result 以外も含む）に使えるため、
`let` キーワードは不要であり、かえって言語の一貫性を損なう。

v17.4.0 は「`let` を追加する」リリースではなく、**「誤って追加した `let` を除去する」修正リリース**とする。

---

## タスク一覧

### T1: Lexer — `TokenKind::Let` 削除

- [x] `fav/src/frontend/lexer.rs` の `TokenKind` enum から `Let` variant を削除
- [x] キーワードマッチ関数の `"let" => TokenKind::Let` を削除
- [x] `let` を識別子として使えるようになることを確認（既存 ID と衝突なし）

### T2: AST — `Stmt::Let` + `LetStmt` 削除

- [x] `fav/src/ast.rs` の `Stmt::Let { name, expr, span }` を削除
- [x] `Stmt::span()` の `Stmt::Let` マッチアームを削除
- [x] `LetStmt` 構造体を削除

### T3: パーサー — `parse_let_stmt` 削除

- [x] `fav/src/frontend/parser.rs` の `TokenKind::Let` ブランチを削除
- [x] `parse_let_stmt` 関数を削除

### T4: 型チェッカー — E0326 削除

- [x] `fav/src/middle/checker.rs` の `check_stmt` の `Stmt::Let` マッチアームを削除（E0326 含む）
- [x] `collect_helpers_in_stmt` の `Stmt::Let` を削除
- [x] `scan_expr_for_pipeline_calls` の `Stmt::Let` を削除

### T5: コンパイラ — `Stmt::Let` 削除

- [x] `fav/src/middle/compiler.rs` の `compile_stmt_into` から `Stmt::Let` を削除
- [x] `collect_free_vars_block` から `Stmt::Let` を削除

### T6: Exhaustive match 削除

- [x] `fav/src/fmt.rs` — `Stmt::Let` ブランチを削除
- [x] `fav/src/emit_python.rs` — `Stmt::Let` ブランチを削除
- [x] `fav/src/lineage.rs` — 4 箇所の `Stmt::Let` を削除
- [x] `fav/src/lint.rs` — 7 箇所の `Stmt::Let` を削除

### T7: テスト更新（`fav/src/driver.rs`）

- [x] `v174000_tests` モジュールから `let` を使うテストを削除（`let_binding_basic` / `let_binding_string` / `let_with_bind_mix` / `let_with_list_comp`）
- [x] `bind x <- non_result_expr` を使うテストに置き換え（`bind_non_result_basic` / `bind_non_result_string` / `bind_mix_result_and_non_result` / `bind_with_list_comp`）
- [x] `let_keyword_not_recognized` テスト追加（`let` がパースエラーになることを確認）
- [x] `cargo test v174000` — 5/5 PASS
- [x] `cargo test` — 1637 tests、リグレッションなし

備考: `version_is_17_4_0` テストは削除（バージョンは v17.5.0 のまま維持）。
`bind_mix_result_and_non_result` は `safe_add`（Result 返し）を使わない形に修正（`bind` は unwrap しないため）。
`bind_with_list_comp` は `Result.ok(...)` で包まず直接 List をバインドする形に修正。

### T8: ドキュメント削除

- [x] `site/content/docs/language/let-binding.mdx` を削除

### T9: バージョン更新

- [x] バージョンは `17.5.0` を維持（v17.5.0 が既に完了済みのため、ダウングレードしない）

---

## 完了条件チェックリスト

- [x] `TokenKind::Let` が lexer から削除されている
- [x] `Stmt::Let` が ast.rs から削除されている
- [x] `parse_let_stmt` がパーサーから削除されている
- [x] E0326 が checker.rs から削除されている
- [x] `compiler.rs` の `Stmt::Let` マッチアームが削除されている
- [x] `fmt.rs` / `emit_python.rs` / `lineage.rs` / `lint.rs` の exhaustive match が更新されている
- [x] `bind x <- non_result_expr` が正しく動作する（既存動作の確認）
- [x] `cargo test v174000` — 5/5 PASS
- [x] `cargo test` — リグレッションなし
