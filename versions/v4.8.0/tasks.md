# Favnir v4.8.0 タスクリスト — LSP (Language Server Protocol)

作成日: 2026-05-17
完了日: 2026-05-17

> **注**: LSP 実装はすでに存在していた。バージョン番号更新のみ実施。

---

## Phase 0: バージョン更新 ✅

- [x] `fav/Cargo.toml` の version を `"4.8.0"` に変更
- [x] `fav/src/main.rs` のヘルプ文字列・バージョン表示を `4.8.0` に更新

---

## Phase 1: LSP モジュール骨格 ✅（既存実装）

- [x] `fav/src/lsp/mod.rs` — `LspServer<W>` + `run_lsp_server` + `read_message` + `write_message`
- [x] `fav/src/lsp/protocol.rs` — `RpcRequest`, `RpcResponse`, `Position`, `Range`, `Diagnostic`, `Hover`, `CompletionItem`, `Location`
- [x] `fav/src/lsp/document_store.rs` — `DocumentStore` + `CheckedDoc`
- [x] `fav/src/lsp/diagnostics.rs` — `errors_to_diagnostics`, `span_to_range`
- [x] `fav/src/lsp/completion.rs` — `handle_completion` (keywords + global symbols + snippets + field completions)
- [x] `fav/src/lsp/hover.rs` — `handle_hover` (type-at-cursor + doc comments)
- [x] `fav/src/lsp/definition.rs` — `handle_definition` (def_at map lookup)
- [x] `fav/src/lsp/doc_comment.rs` — `extract_doc_comments`

---

## Phase 2: JSON-RPC サーバーループ ✅（既存実装）

- [x] Content-Length フレーミング（`read_message` / `write_message`）
- [x] `LspServer::handle` — initialize / didOpen / didChange / hover / completion / definition / shutdown / exit
- [x] `run_lsp_server(port: Option<u16>)` — stdin/stdout ループ

---

## Phase 3: 診断 ✅（既存実装）

- [x] `DocumentStore::open_or_change` — Parser + Checker → `CheckedDoc`
- [x] `errors_to_diagnostics` — TypeError → LSP Diagnostic (span.line - 1 で 0-based 変換)
- [x] `publish_diagnostics` — `textDocument/publishDiagnostics` 通知送信

---

## Phase 4: 補完 ✅（既存実装）

- [x] `.` トリガーでレコードフィールド補完（`checker.record_fields` 参照）
- [x] グローバルシンボル補完（`checker.symbol_index` 参照）
- [x] キーワード補完（fn, type, stage, match, bind, test 等）
- [x] スニペット補完（insertTextFormat: 2）

---

## Phase 5: ホバー ✅（既存実装）

- [x] カーソル位置の最小スパンの型を表示（`checker.type_at` 参照）
- [x] ドキュメントコメントをホバーに含める（`extract_doc_comments` + `checker.def_at`）
- [x] `position_to_char_offset` — LSP Position → byte offset
- [x] `span_contains` — スパン内判定

---

## Phase 6: 定義ジャンプ ✅（既存実装）

- [x] `handle_definition` — `checker.def_at` でカーソル位置の定義スパンを返す
- [x] `byte_offset_to_position` — byte offset → LSP Position

---

## Phase 7: CLI コマンド ✅（既存実装）

- [x] `main.rs` の `["lsp"] | ["lsp", ..]` → `lsp::run_lsp_server(port)`
- [x] HELP テキストに `lsp [--port <n>]` 記載

---

## Phase 8: テスト ✅（既存実装、24 件）

| テスト | ファイル |
|--------|---------|
| `read_message_parses_content_length_frame` | mod.rs |
| `write_message_emits_content_length_frame` | mod.rs |
| `initialize_returns_capabilities` | mod.rs |
| `did_open_publishes_diagnostics` | mod.rs |
| `run_lsp_loop_processes_initialize_and_exit` | mod.rs |
| `run_lsp_loop_processes_did_change_and_hover` | mod.rs |
| `run_lsp_loop_recovers_from_parse_error_and_then_hovers` | mod.rs |
| `completion_request_returns_items` | mod.rs |
| `definition_request_returns_location` | mod.rs |
| `completion_returns_field_items_on_dot_trigger` | completion.rs |
| `completion_returns_global_fn_name` | completion.rs |
| `completion_includes_keywords` | completion.rs |
| `completion_includes_snippets` | completion.rs |
| `hover_returns_smallest_matching_type` | hover.rs |
| `hover_returns_none_when_position_is_outside_any_span` | hover.rs |
| `hover_includes_doc_comment_for_symbol_use` | hover.rs |
| `definition_returns_location_for_global_fn` | definition.rs |
| `definition_returns_none_for_unknown_position` | definition.rs |
| `converts_checker_error_to_zero_origin_diagnostic` | diagnostics.rs |
| `parse_error_e000_gets_single_char_range` | diagnostics.rs |
| `open_or_change_collects_checker_types` | document_store.rs |
| `open_or_change_reports_parse_error_as_e000` | document_store.rs |
| `extract_doc_comment_before_fn` | doc_comment.rs |
| `extract_doc_comment_multiline` | doc_comment.rs |

---

## 完了条件 ✅

- [x] `cargo build` が通る
- [x] 既存 874 件が全て pass（バージョン bump 後も変わらず）
- [x] LSP 24 件のテストが pass
- [x] `fav lsp` コマンドが機能する（stdin JSON-RPC ループ）
- [x] `initialize` に capabilities を返す
- [x] `didOpen`/`didChange` で型エラーを診断として送信
- [x] `.` トリガーでフィールド補完
- [x] ホバーで型情報・doc コメントを返す
- [x] 定義ジャンプが同ファイル内で動作

---

## checker.rs の LSP 拡張（既存実装）

- [x] `Checker::type_at: HashMap<Span, Type>` — 式の型を記録
- [x] `Checker::def_at: HashMap<Span, Span>` — 使用箇所 → 定義箇所マップ
- [x] `Checker::symbol_index: Vec<LspSymbol>` — グローバルシンボルリスト
- [x] `Checker::record_fields: HashMap<String, Vec<(String, Type)>>` — レコード型フィールド情報

---

## 実装メモ

- **stdout フラッシュ**: `write_json_message` で `writer.flush()` 済み
- **Span.col**: 1-based → `span.col.saturating_sub(1)` で 0-based に変換
- **Full sync**: `textDocumentSync: 1` — `didChange` の `contentChanges[0].text` を全文として扱う
- **TCP モード**: `--port` オプションは未実装（未対応のメッセージを stderr に出力するのみ）
- **rune import 解決なし**: `DocumentStore` は単一ファイルの checker のみ（rune 関数は補完・型チェック対象外）
