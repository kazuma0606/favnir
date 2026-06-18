# v16.3.0 Tasks — レコード更新構文（Record Spread / Update）

Date: 2026-06-14
Branch: master

---

## Phase A — Cargo バージョン更新

- [x] A-1: `fav/Cargo.toml` の `version` を `"16.3.0"` に変更
- [x] A-2: `cargo build` → コンパイルエラーなし確認

---

## Phase B — `DotDotDot` トークン追加（lexer.rs）

- [x] B-1: `fav/src/frontend/lexer.rs` の `TokenKind` enum に `DotDotDot` 追加
- [x] B-2: `next_token` の `'.'` 処理に `...` 先読みケースを追加
- [x] B-3: `cargo build` → コンパイルエラーなし確認

---

## Phase C — AST 拡張（ast.rs）

- [x] C-1: `fav/src/ast.rs` の `Expr` enum に `RecordSpread(Box<Expr>, Vec<(String, Expr)>, Span)` 追加
- [x] C-2: `Expr::span()` の match に `Expr::RecordSpread(_, _, s) => s` 追加
- [x] C-3: `cargo build` → コンパイルエラーなし確認

---

## Phase D — Parser 拡張（parser.rs）

- [x] D-1: `{` 直後が `DotDotDot` の場合に `parse_record_spread` を呼ぶよう分岐追加
- [x] D-2: `parse_record_spread` メソッド追加
- [x] D-3: `cargo build` → コンパイルエラーなし確認

---

## Phase E — IR 拡張（ir.rs）

- [x] E-1: `IRExpr::RecordSpread(Box<IRExpr>, Vec<(String, IRExpr)>, Type)` 追加
- [x] E-2: `IRExpr::ty()` の match に追加
- [x] E-3: `cargo build` → コンパイルエラーなし確認

---

## Phase F — Compiler 拡張（compiler.rs）

- [x] F-1: `compile_expr` に `Expr::RecordSpread` ケース追加
- [x] F-2: `collect_free_vars_expr` に追加
- [x] F-3: `collect_helpers_in_expr` に追加
- [x] F-4: `cargo build` → コンパイルエラーなし確認

---

## Phase G — Codegen 拡張（codegen.rs）

- [x] G-1: `Opcode::MergeRecord = 0x5C` 追加
- [x] G-2: `emit_expr` に `IRExpr::RecordSpread` ケース追加
- [x] G-3: `remap_string_operands` に `MergeRecord` ケース追加（バグ修正）
- [x] G-4: `cargo build` → コンパイルエラーなし確認

---

## Phase H — VM 実装（vm.rs）

- [x] H-1: dispatch loop に `Opcode::MergeRecord` ケース追加
- [x] H-2: `cargo build` → コンパイルエラーなし確認

---

## Phase I — 型チェック拡張（checker.rs）

- [x] I-1: `check_expr` に `Expr::RecordSpread` ケース追加（base/updates を check_expr、Type::Unknown を返す）
- [x] I-2: `collect_helpers_in_expr` に追加（Phase F で実施）

---

## Phase J — ast_lower_checker.rs 拡張

- [x] J-1: `lower_expr` に `Expr::RecordSpread` フォールバック追加

---

## Phase K — lineage.rs 拡張

- [x] K-1: `collect_sql_literals_inner` に追加
- [x] K-2: `collect_azure_kinds_inner` に追加
- [x] K-3: `collect_azure_blob_kinds_inner` に追加
- [x] K-4: `collect_sf_kinds_inner` に追加

---

## Phase L — wasm_codegen.rs + emit_python.rs + driver.rs 拡張

- [x] L-1: `wasm_codegen.rs` の 5 関数に `IRExpr::RecordSpread` 追加
- [x] L-2: `emit_python.rs` に `Expr::RecordSpread` 追加
- [x] L-3: `driver.rs` の `remap_ir_expr` / `format_expr_compact` / `expr_to_sql` に追加
- [x] L-4: `lint.rs` の 5 関数に追加
- [x] L-5: `fmt.rs` に追加

---

## Phase M — get_help_text 更新（driver.rs）

- [x] M-1: `"E0323"` ヒント追加
- [x] M-2: `"E0327"` ヒント追加
- [x] M-3: `"E0328"` ヒント追加

---

## Phase N — v163000_tests 追加（driver.rs）

- [x] N-1: `v163000_tests` モジュール追加
- [x] N-2: 6 テスト実装（version + 5 record spread テスト）
- [x] N-3: `cargo test v163000` → 6/6 PASS 確認

---

## Phase O — サイトドキュメント

- [x] O-1: `site/content/docs/language/record-update.mdx` 新規作成

---

## Phase P — テスト確認とコミット

- [x] P-1: `cargo test v163000` → 6/6 PASS 最終確認
- [x] P-2: `cargo test` → 1589 PASS（v163000_tests 6/6 含む）
- [x] P-3: コミット

---

## 完了条件

| 確認項目 | 状態 |
|---|---|
| `Cargo.toml version == "16.3.0"` | [x] |
| `{ ...row, field: val }` がコンパイル・実行される | [x] |
| 複数フィールドのスプレッドが動作する | [x] |
| フィールドの上書きが正しく動作する | [x] |
| スプレッドで元フィールドが保持される | [x] |
| `cargo test v163000` 全テストパス（6/6） | [x] |
| `cargo test` 1589 パス（リグレッションなし） | [x] |
| E0323 / E0327 / E0328 の `get_help_text` が追加されている | [x] |
| `site/content/docs/language/record-update.mdx` が存在する | [x] |
| `remap_string_operands` に MergeRecord ケース追加（バグ修正） | [x] |
