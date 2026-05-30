# Favnir v4.9.0 タスクリスト — MCP (Model Context Protocol)

作成日: 2026-05-17
完了日: 2026-05-17

---

## Phase 0: バージョン更新 ✅

- [x] `fav/Cargo.toml` の version を `"4.9.0"` に変更
- [x] `fav/src/main.rs` のヘルプ文字列・バージョン表示を `4.9.0` に更新

---

## Phase 1: MCP モジュール骨格 ✅

- [x] `fav/src/mcp/mod.rs` — `McpServer<W>` 構造体 + `run_mcp_server` + `read_message` + `write_json_message`（単一ファイル実装）
- [x] `McpRequest` 型定義（serde Deserialize）
- [x] `fav/src/main.rs` に `mod mcp;` 追加

---

## Phase 2: JSON-RPC サーバーループ ✅

- [x] Content-Length フレーミング実装（`read_message` / `write_json_message`）
- [x] `McpServer::handle` — initialize / initialized / tools/* / resources/* / prompts/* / shutdown / exit
- [x] 不明メソッド → JSON-RPC error -32601
- [x] `run_mcp_server()` — stdin/stdout ループ

---

## Phase 3: `initialize` ハンドラ ✅

- [x] `protocolVersion: "2024-11-05"` を返す
- [x] capabilities: `tools: {}`, `resources: {}`, `prompts: {}`
- [x] serverInfo: `name: "favnir-mcp"`, `version: "4.9.0"`

---

## Phase 4: Tools ✅

- [x] `tools/list` — 5 ツールの定義（name / description / inputSchema）を返す
- [x] `tools/call` ディスパッチ — ツール名で分岐
- [x] `favnir_check` — Parser → Checker → エラー一覧 or "OK: no type errors"
- [x] `favnir_run` — parse + check + compile + VM::run（stdout 非キャプチャ）
- [x] `favnir_test` — parse + check + compile + テストブロック実行、pass/fail サマリ
- [x] `favnir_list_runes` — Rune 名と公開関数一覧を返す（ハードコード）
- [x] `favnir_rune_docs` — FAV_RUNES_PATH または ./runes/ から読む、フォールバックでハードコード docs
- [x] 不明ツール名 → `isError: true`
- [x] `tool_text` / `tool_error` ヘルパー関数

---

## Phase 5: Resources ✅

- [x] `resources/list` — 8 リソース URI を返す（stdlib + 6 Rune + project/files）
- [x] `resources/read` — URI でディスパッチ
- [x] `favnir://docs/stdlib` — 組み込み型・関数シグネチャ Markdown（ハードコード）
- [x] `favnir://runes/{name}` — `tool_favnir_rune_docs` 経由
- [x] `favnir://project/files` — カレントディレクトリ配下 `.fav` ファイル再帰列挙
- [x] 不明 URI → error フィールドを含む JSON

---

## Phase 6: Prompts ✅

- [x] `prompts/list` — `write_pipeline` / `fix_type_error` の 2 プロンプト定義
- [x] `prompts/get` — プロンプト名 + 引数で messages を返す
- [x] `write_pipeline` — source_type / output_type 引数を展開した user メッセージ
- [x] `fix_type_error` — source / error 引数を展開した user メッセージ
- [x] 不明プロンプト名 → error フィールドを含む JSON

---

## Phase 7: CLI 配線 ✅

- [x] `fav/src/main.rs` に `Some("mcp")` アーム追加 → `mcp::run_mcp_server()`
- [x] HELP テキストに `mcp` コマンド記載

---

## Phase 8: テスト ✅（19 件）

| テスト | ファイル |
|--------|---------|
| `initialize_returns_capabilities` | mod.rs |
| `tools_list_returns_five_tools` | mod.rs |
| `tools_call_favnir_check_ok` | mod.rs |
| `tools_call_favnir_check_error` | mod.rs |
| `tools_call_favnir_run_ok` | mod.rs |
| `tools_call_favnir_run_type_error` | mod.rs |
| `tools_call_favnir_test_ok` | mod.rs |
| `tools_call_favnir_test_fail` | mod.rs |
| `tools_call_favnir_list_runes` | mod.rs |
| `tools_call_favnir_rune_docs_db` | mod.rs |
| `tools_call_favnir_rune_docs_unknown` | mod.rs |
| `tools_call_unknown_tool_returns_error` | mod.rs |
| `resources_list_returns_uris` | mod.rs |
| `resources_read_stdlib` | mod.rs |
| `resources_read_rune_db` | mod.rs |
| `prompts_list_returns_two` | mod.rs |
| `prompts_get_write_pipeline` | mod.rs |
| `prompts_get_fix_type_error` | mod.rs |
| `shutdown_returns_null` | mod.rs |

---

## 完了条件 ✅

- [x] `cargo build` が通る
- [x] 既存テスト（874 件）が全て pass（計 893 件で pass）
- [x] MCP 19 件のテストが pass
- [x] `fav mcp` コマンドが機能する（stdin JSON-RPC ループ）
- [x] `initialize` に正しいケーパビリティを返す
- [x] `tools/call favnir_check` が型エラーを検出する
- [x] `tools/call favnir_list_runes` が Rune 一覧を返す
- [x] `resources/read favnir://docs/stdlib` が stdlib docs を返す
- [x] `prompts/get write_pipeline` が messages を返す
