# Favnir v4.9.0 仕様書 — MCP (Model Context Protocol)

作成日: 2026-05-17

---

## 概要

Favnir を AI アシスタント（Claude・Cursor・Copilot 等）と連携させるため、Model Context Protocol (MCP) サーバーを実装する。MCP 対応クライアントから Favnir コードの実行・型チェック・テスト・Rune ドキュメント参照などを AI 経由で呼び出せるようにする。

**主な追加機能:**
- `fav mcp` コマンド（MCP サーバーとして起動）
- `tools/call` — Favnir コードの実行・型チェック・テスト実行・Rune 一覧・Rune ドキュメント
- `resources/read` — `favnir://` スキームで Rune ソース・stdlib docs・プロジェクトファイルを参照
- `prompts/get` — パイプライン雛形・型エラー修正の prompt テンプレート

---

## 設計方針

### トランスポート

LSP と同様に JSON-RPC 2.0 over stdin/stdout with `Content-Length` フレーミングを使用。外部クレート不使用、`serde_json`（既存依存）のみ。

```
Content-Length: 123\r\n
\r\n
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{...}}
```

### MCP プロトコルバージョン

`"2024-11-05"`（MCP 仕様 stable 版）

### モジュール構成

```
fav/src/mcp/
  mod.rs        — cmd_mcp エントリポイント; サーバーメインループ
  protocol.rs   — 最小限の MCP 型定義（serde_json ベース）
  tools.rs      — ツールハンドラ群
  resources.rs  — リソースハンドラ群
  prompts.rs    — プロンプトハンドラ群
```

---

## MCP 初期化フロー

```
Client → initialize    → Server: capabilities (tools, resources, prompts)
Client → initialized   → Server: (無視)
Client → tools/list    → Server: ツール一覧
Client → tools/call    → Server: ツール実行結果
Client → resources/list → Server: リソース一覧
Client → resources/read → Server: リソース内容
Client → prompts/list  → Server: プロンプト一覧
Client → prompts/get   → Server: プロンプト内容
Client → shutdown      → Server: null
Client → exit          → Server: process::exit(0)
```

---

## `initialize` レスポンス

```json
{
  "protocolVersion": "2024-11-05",
  "capabilities": {
    "tools": {},
    "resources": {},
    "prompts": {}
  },
  "serverInfo": {
    "name": "favnir-mcp",
    "version": "4.9.0"
  }
}
```

---

## Tools（ツール）

### `favnir_run`

Favnir コードスニペットを実行する。

**Input Schema:**
```json
{
  "type": "object",
  "properties": {
    "source": {
      "type": "string",
      "description": "Favnir source code to execute (must include a main() function)"
    }
  },
  "required": ["source"]
}
```

**処理:**
1. 一時ファイルに source を書き込む
2. Parser → Checker → Compiler → VM で実行
3. stdout キャプチャ → テキスト結果を返す

**Output:**
```json
{
  "content": [{ "type": "text", "text": "<実行出力>" }]
}
```

---

### `favnir_check`

Favnir コードを型チェックのみ実行する（VM は走らせない）。

**Input Schema:**
```json
{
  "type": "object",
  "properties": {
    "source": {
      "type": "string",
      "description": "Favnir source code to type-check"
    }
  },
  "required": ["source"]
}
```

**処理:**
1. Parser::parse_str → エラーがあれば返す
2. Checker::check_program → TypeError 一覧を返す

**Output (success):**
```json
{
  "content": [{ "type": "text", "text": "OK: no type errors" }]
}
```

**Output (error):**
```json
{
  "content": [{ "type": "text", "text": "E0102 at line 3: undefined: `foo`\nE0201 at line 5: type mismatch: ..." }]
}
```

---

### `favnir_test`

Favnir テストファイルを実行する。

**Input Schema:**
```json
{
  "type": "object",
  "properties": {
    "source": {
      "type": "string",
      "description": "Favnir source with test blocks (test \"name\" { ... })"
    }
  },
  "required": ["source"]
}
```

**処理:**
1. `test` ブロックを含むソースをパース・型チェック・コンパイル
2. 各 test ブロックを実行（driver の `exec_tests` 相当）
3. pass / fail サマリを返す

**Output:**
```json
{
  "content": [{ "type": "text", "text": "3 passed, 1 failed\n  FAIL: test \"add returns 5\": assertion failed" }]
}
```

---

### `favnir_list_runes`

利用可能な Rune の一覧を返す。

**Input Schema:**
```json
{
  "type": "object",
  "properties": {}
}
```

**処理:**
- `runes/` ディレクトリを走査し、`.fav` ファイルから public fn 一覧を抽出

**Output:**
```json
{
  "content": [{ "type": "text", "text": "db: connect, query, execute, ...\nhttp: get, post, put, ...\nlog: info, warn, error, ..." }]
}
```

---

### `favnir_rune_docs`

特定 Rune の関数ドキュメント（シグネチャ + doc コメント）を返す。

**Input Schema:**
```json
{
  "type": "object",
  "properties": {
    "rune": {
      "type": "string",
      "description": "Rune name (e.g., \"db\", \"http\", \"log\", \"gen\", \"auth\", \"env\")"
    }
  },
  "required": ["rune"]
}
```

**処理:**
1. `runes/<rune>/` ディレクトリの `.fav` ファイルを読む
2. `## fn name(args) -> ReturnType` 形式のドキュメントを生成
3. `///` doc コメントがあれば付加

**Output:**
```json
{
  "content": [{ "type": "text", "text": "## db\n\n### connect(url: String) -> Result<DbHandle, String> !Io\nEstablishes a database connection.\n\n### query(conn: DbHandle, sql: String, params: List<String>) -> Result<List<Map<String,String>>, String> !Io\n..." }]
}
```

---

## Resources（リソース）

### `favnir://runes/{name}`

指定 Rune のソースコード全文を返す。

例: `favnir://runes/db` → `runes/db/db.fav` の内容

### `favnir://docs/stdlib`

組み込み標準ライブラリ（String, List, Map, Option, Result, Int, Bool, Tuple）の型シグネチャ一覧を Markdown で返す。

### `favnir://project/files`

カレントプロジェクト（fav.toml のある場所）の `.fav` ファイル一覧を返す。

---

## Prompts（プロンプト）

### `write_pipeline`

データパイプライン雛形を生成するプロンプト。

**Arguments:**
- `source_type`: `"csv"` | `"parquet"` | `"db"` | `"http"` — 入力データソース
- `output_type`: `"csv"` | `"parquet"` | `"db"` | `"stdout"` — 出力先

**返す messages:**
```json
[
  {
    "role": "user",
    "content": {
      "type": "text",
      "text": "Write a Favnir pipeline that reads from {source_type} and writes to {output_type}. Use appropriate Runes and effects."
    }
  }
]
```

### `fix_type_error`

型エラーを修正するプロンプト。

**Arguments:**
- `source`: エラーのある Favnir コード
- `error`: エラーメッセージ

**返す messages:**
```json
[
  {
    "role": "user",
    "content": {
      "type": "text",
      "text": "Fix the following Favnir type error:\n\nCode:\n```\n{source}\n```\n\nError:\n{error}\n\nExplain the fix and provide the corrected code."
    }
  }
]
```

---

## CLI コマンド

```
fav mcp
```

MCP サーバーとして起動する。stdin/stdout で JSON-RPC を処理する。

`main.rs` のルーティング:
```rust
["mcp"] | ["mcp", ..] => mcp::run_mcp_server(),
```

ヘルプテキスト:
```
  mcp                    Start MCP server (JSON-RPC over stdin/stdout)
```

---

## エラーハンドリング

- JSON-RPC デコードエラー: stderr にログ、次のメッセージへ
- `tools/call` でのソースパースエラー: エラーテキストを content に含めて返す（isError: true）
- 不明なメソッド: JSON-RPC error code -32601 "Method not found"
- 不明なツール: `isError: true` + エラーメッセージ

---

## テスト方針

### ユニットテスト（`fav/src/mcp/` 内）

| テスト | 内容 |
|--------|------|
| `initialize_returns_capabilities` | `initialize` に正しいケーパビリティを返す |
| `tools_list_returns_five_tools` | 5 ツールが列挙される |
| `tools_call_favnir_check_ok` | 正しいコードで "OK: no type errors" |
| `tools_call_favnir_check_error` | 型エラーコードでエラーメッセージ |
| `tools_call_favnir_run_ok` | `main()` が実行されて stdout を返す |
| `tools_call_favnir_test_ok` | test ブロックが実行されてサマリを返す |
| `tools_call_favnir_list_runes` | Rune 名一覧が返る |
| `tools_call_favnir_rune_docs` | `db` Rune のドキュメントが返る |
| `tools_call_unknown_tool` | isError: true |
| `resources_list_returns_uris` | 3 リソース URI が返る |
| `resources_read_stdlib` | stdlib docs が返る |
| `prompts_list_returns_two` | 2 プロンプトが返る |
| `prompts_get_write_pipeline` | messages が返る |
| `prompts_get_fix_type_error` | messages が返る |
| `shutdown_returns_null` | null レスポンス |
| `exit_terminates_loop` | ループが終了する |

---

## 既知の制約

- v4.9.0 は stdin/stdout のみ（HTTP/SSE トランスポートは将来）
- `favnir_run` は sandboxing なし（ローカル実行）
- `favnir://runes/{name}` は `fav` バイナリと同じディレクトリ基準でパスを探す
- リソースの動的更新通知（`notifications/resources/updated`）は未対応
- サンプリング（`sampling/createMessage`）は未対応
