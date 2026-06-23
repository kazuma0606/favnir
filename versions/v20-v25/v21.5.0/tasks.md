# v21.5.0 — LSP コードアクション強化 タスク

## ステータス: COMPLETE（2026-06-20）

---

## タスク一覧

### T0: `fav/src/lsp/document_store.rs` — `program` フィールド追加

- [x] **事前確認**: `grep -n "struct CheckedDoc\|pub source\|pub errors" fav/src/lsp/document_store.rs | head -15` で既存フィールドを確認
- [x] `CheckedDoc` に `pub program: Option<Program>` フィールドを追加
- [x] `use crate::frontend::ast::Program;` を冒頭に追加（未追加なら）
- [x] `open_or_change` 内の `Parser::parse_str` 成功時に `program: Some(prog)` を格納
- [x] パース失敗時は `program: None` を設定
- [x] `CheckedDoc` のすべての構築箇所を修正（`cargo check` でエラー箇所を確認）
- [x] `cargo check` でコンパイルエラー 0

---

### T1: `fav/src/lsp/protocol.rs` — 型追加

- [x] **事前確認**: `grep -n "struct TextEdit\|struct WorkspaceEdit\|struct CodeAction" fav/src/lsp/protocol.rs` で既存定義を確認
- [x] `TextEdit { range: Range, new_text: String }` を追加（`#[serde(rename = "newText")]`）
- [x] `WorkspaceEdit { changes: HashMap<String, Vec<TextEdit>> }` を追加（`use std::collections::HashMap`）
- [x] `CodeAction { title: String, kind: Option<String>, edit: Option<WorkspaceEdit> }` を追加（`kind` / `edit` ともに `skip_serializing_if = "Option::is_none"`）
- [x] `cargo check` でコンパイルエラー 0

---

### T2: `fav/src/lsp/references.rs` — 新規作成

- [x] **事前確認**: `grep -n "pub uses" fav/src/frontend/ast.rs | head -5` で uses フィールドの型（`Vec<Vec<String>>`）を確認
- [x] **事前確認**: `grep -n "struct Span" fav/src/frontend/lexer.rs | head -5` で Span フィールド（start/end バイトオフセット）を確認
- [x] `word_at_offset(src: &str, offset: usize) -> Option<String>` を実装（前後をアルファベット / `_` でスキャン）
- [x] `span_to_range(span: &Span) -> Range` を実装（`src` 引数不要 — バイトオフセット使用）
  - `start.line = span.line - 1`、`start.character = span.col - 1`（0-based）
  - `end.character = start.character + (span.end - span.start)` — ASCII 識別子はバイト長 = 文字長
- [x] `collect_in_expr(expr: &Expr, name: &str, spans: &mut Vec<Span>)` を実装
  - `Expr::Ident(n, span)` → push
  - 再帰: Apply / Block / If / Match / Pipeline / Closure / BinOp 等
- [x] `collect_in_block(block: &Block, name: &str, spans: &mut Vec<Span>)` を実装
- [x] `collect_in_type_expr(ty: &TypeExpr, name: &str, spans: &mut Vec<Span>)` を実装
  - `TypeExpr::Named(n, type_args, span)` → push + 型引数を再帰
- [x] `collect_in_item(item: &Item, name: &str, spans: &mut Vec<Span>)` を実装
  - 各 Item の名前チェック + body 走査
- [x] `pub fn collect_symbol_occurrences(program: &Program, name: &str) -> Vec<Span>` を実装
- [x] `pub fn handle_references(store: &DocumentStore, uri: &str, pos: Position) -> Vec<Location>` を実装
- [x] `cargo check` でコンパイルエラー 0

---

### T3: `fav/src/lsp/rename.rs` — 新規作成

- [x] `FAVNIR_KEYWORDS: &[&str]` 定数を定義
- [x] `pub fn handle_rename(store: &DocumentStore, uri: &str, pos: Position, new_name: &str) -> Option<WorkspaceEdit>` を実装
  1. `position_to_char_offset` でオフセット取得（`hover.rs` から import）
  2. `word_at_offset`（`references.rs` から import）でシンボル名取得
  3. キーワードチェック → None
  4. `collect_symbol_occurrences` で全 Span 取得 → None（空なら）
  5. Span → `TextEdit { range, new_text: new_name.to_string() }` に変換
  6. `WorkspaceEdit { changes }` を返す
- [x] `cargo check` でコンパイルエラー 0

---

### T4: `fav/src/lsp/code_action.rs` — 新規作成

- [x] **事前確認**: `grep -n "pub uses" fav/src/frontend/ast.rs | head -5` で uses フィールドが `Vec<Vec<String>>` であることを確認（UseDecl 型は存在しない）
- [x] `fn check_add_missing_import(doc: &CheckedDoc, uri: &str, line_text: &str) -> Option<CodeAction>` を実装
  - `line_text` から英字 + `_` の連続 + `.` パターンで NS を抽出
  - `crate::lsp::completion::KNOWN_RUNES` に含まれるか確認（コピーせず再 export を使う）
  - `doc.program.as_ref()` の `uses: Vec<Vec<String>>` を `path.first() == Some(ns)` でチェック
  - WorkspaceEdit: `Range{start:{0,0},end:{0,0}}` に `"use <ns>\n"` を挿入
  - `kind: Some("quickfix".to_string())`
- [x] `fn check_convert_to_fstring(line_text: &str) -> Option<CodeAction>` を実装
  - `line_text.contains("String.concat(")` のみ
  - `kind: Some("refactor.rewrite".to_string())`、`edit: None`
- [x] `fn check_inline_binding(line_text: &str) -> Option<CodeAction>` を実装
  - `trim_start().starts_with("bind ")` のみ
  - bind 後の名前を抽出
  - `kind: Some("refactor.inline".to_string())`、`edit: None`
- [x] `pub fn handle_code_action(store: &DocumentStore, uri: &str, range: Range) -> Vec<CodeAction>` を実装
  - `range.start.line` 行のテキストを取得
  - 3 関数を呼び出して結果を集約
- [x] `cargo check` でコンパイルエラー 0

---

### T5: `fav/src/lsp/mod.rs` — 統合

- [x] `pub mod code_action;` / `pub mod references;` / `pub mod rename;` を `lsp/mod.rs` 冒頭に追加（T0〜T4 完了後）
- [x] use 宣言に `code_action::handle_code_action`, `references::handle_references`, `rename::handle_rename` を追加
- [x] `initialize` レスポンスの capabilities に追加:
  ```json
  "codeActionProvider": true,
  "renameProvider": true,
  "referencesProvider": true
  ```
- [x] `extract_code_action_params(params) -> Option<(String, Range)>` を実装
- [x] `extract_rename_params(params) -> Option<(String, Position, String)>` を実装
- [x] `"textDocument/codeAction"` ハンドラ追加
- [x] `"textDocument/rename"` ハンドラ追加
- [x] `"textDocument/references"` ハンドラ追加
- [x] `cargo check` でコンパイルエラー 0
- [x] `cargo test lsp` — 既存 LSP テストがリグレッションしていないことを確認

---

### T6: `fav/Cargo.toml` バージョン更新

- [x] `version = "21.4.0"` → `"21.5.0"` に変更
- [x] `v214000_tests::version_is_21_4_0` に `#[ignore]` を追加

---

### T7: `fav/src/driver.rs` — v215000_tests 追加

- [x] **事前確認**: `grep -n "mod v214000_tests" fav/src/driver.rs | head -3` で追加位置を確認
- [x] `v214000_tests` の後に `v215000_tests` モジュールを追加
- [x] ヘルパー関数:
  - `fn open_doc_server(src: &str, uri: &str) -> (LspServer<Vec<u8>>, Vec<u8>)` — didOpen まで実行
  - `fn send_request(server: &mut LspServer<...>, method, params) -> String` — レスポンス文字列を返す
- [x] 12 件のテストを実装（spec.md テスト表参照）
- [x] `cargo test v215000` — 12/12 PASS を確認
- [x] `cargo test` — リグレッションなし（exit 0）を確認

---

### T8: `CHANGELOG.md` + `site/content/docs/tools/lsp.mdx`

- [x] `CHANGELOG.md` の先頭に v21.5.0 エントリを追加
- [x] `site/content/docs/tools/lsp.mdx` を新規作成（既存なら更新）
  - [x] コードアクションセクション（CA-1〜CA-3 説明）
  - [x] リネームセクション（使い方・VS Code での操作）
  - [x] 参照検索セクション

---

## テスト一覧（v215000_tests、12件）

| テスト名 | 内容 |
|----------|------|
| `version_is_21_5_0` | Cargo.toml に `"21.5.0"` が含まれる |
| `lsp_capabilities_code_action` | initialize → `"codeActionProvider":true` |
| `lsp_capabilities_rename` | initialize → `"renameProvider":true` |
| `lsp_capabilities_references` | initialize → `"referencesProvider":true` |
| `lsp_code_action_returns_array` | codeAction リクエスト → JSON 配列（`[`） |
| `lsp_code_action_convert_to_fstring` | String.concat 行 → "Convert to f-string" |
| `lsp_code_action_add_missing_import` | http.get 行（use なし）→ "Add missing import" |
| `lsp_references_returns_call_sites` | `double` の参照 → Location 含む |
| `lsp_references_empty_for_unknown` | `__xyzunknown` → `[]` |
| `lsp_rename_fn_updates_occurrences` | `double` → `multiply` → WorkspaceEdit 含む |
| `lsp_rename_returns_null_for_keyword` | `fn` 位置でリネーム → null |
| `lsp_rename_response_contains_changes_key` | rename → `"changes"` キーあり |

---

## 完了条件チェックリスト

- [x] `cargo test v215000` — 12/12 PASS
- [x] `cargo test` — リグレッションなし（1817 tests pass、0 failures）
- [x] `CHANGELOG.md` に v21.5.0 エントリ
- [x] `fav/Cargo.toml` version が `21.5.0`
- [x] `site/content/docs/tools/lsp.mdx` に更新済み

---

## 優先度

```
T0（document_store.rs — program 追加）  ← 最初（T2/T3/T4 が依存）
T1（protocol.rs 型追加）                ← T0 と並列可
T2（references.rs）                     ← T0, T1 後（rename と word_at_offset を共有）
T3（rename.rs）                         ← T2 後（word_at_offset を import）
T4（code_action.rs）                    ← T0, T1 後（T2/T3 と並列可）
T5（mod.rs 統合）                       ← T2, T3, T4 完了後
T6（Cargo.toml）                        ← いつでも
T7（driver.rs テスト）                  ← T5 完了後
T8（CHANGELOG + MDX）                   ← 最後
```

---

## 実装リスクと対策

| リスク | 対策 |
|--------|------|
| ParsedDoc に program フィールドがない | document_store.rs を確認し、必要なら追加するか都度 parse する |
| UseDecl のフィールド名が不明 | ast.rs を grep して確認してから実装 |
| Span に end_col がない | `span.end - span.start` でバイト長を直接計算（plan.md 参照）。ASCII 識別子はバイト長 = 文字長のため近似不要 |
| word_at_offset がキーワードを返す | FAVNIR_KEYWORDS でフィルタリング |
| add_missing_import で NS 抽出が壊れる | 英数字 / `_` + `.` のシンプルな文字列スキャンで安全に実装 |
