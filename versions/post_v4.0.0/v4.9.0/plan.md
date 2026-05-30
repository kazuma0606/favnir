# Favnir v4.9.0 実装計画 — MCP (Model Context Protocol)

作成日: 2026-05-17

---

## Phase 0: バージョン更新

- `fav/Cargo.toml` の version を `"4.9.0"` に変更
- `fav/src/main.rs` のヘルプ文字列・バージョン表示を `4.9.0` に更新

---

## Phase 1: MCP モジュール骨格

### `fav/src/mcp/mod.rs`

```rust
pub mod protocol;
pub mod tools;
pub mod resources;
pub mod prompts;

pub fn run_mcp_server() {
    // stdin/stdout JSON-RPC ループ
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut server = McpServer::new(stdout.lock());
    loop {
        match read_message(&mut stdin.lock()) {
            Ok(msg) => server.handle(msg),
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => break,
            Err(e) => eprintln!("[favnir-mcp] read error: {e}"),
        }
    }
}
```

LSP の `read_message` / `write_message` と同じ Content-Length フレーミングを再利用（または `mcp/mod.rs` に独立実装）。

---

## Phase 2: protocol.rs — MCP 型定義

最小限の型を `serde_json::Value` ベースで定義:

```rust
pub struct McpRequest {
    pub id: Option<Value>,
    pub method: String,
    pub params: Option<Value>,
}

pub struct McpResponse {
    pub id: Option<Value>,
    pub result: Option<Value>,
    pub error: Option<McpError>,
}

pub struct McpError {
    pub code: i64,
    pub message: String,
}
```

定数:
```rust
pub const MCP_PROTOCOL_VERSION: &str = "2024-11-05";
pub const ERROR_METHOD_NOT_FOUND: i64 = -32601;
pub const ERROR_INVALID_PARAMS: i64 = -32602;
```

---

## Phase 3: サーバーループ (`McpServer::handle`)

```rust
impl<W: Write> McpServer<W> {
    pub fn handle(&mut self, req: McpRequest) {
        match req.method.as_str() {
            "initialize"      => self.handle_initialize(&req),
            "initialized"     => { /* ignore */ }
            "tools/list"      => self.handle_tools_list(&req),
            "tools/call"      => self.handle_tools_call(&req),
            "resources/list"  => self.handle_resources_list(&req),
            "resources/read"  => self.handle_resources_read(&req),
            "prompts/list"    => self.handle_prompts_list(&req),
            "prompts/get"     => self.handle_prompts_get(&req),
            "shutdown"        => self.send_result(&req.id, Value::Null),
            "exit"            => std::process::exit(0),
            _                 => self.send_error(&req.id, ERROR_METHOD_NOT_FOUND, "Method not found"),
        }
    }
}
```

---

## Phase 4: tools.rs — ツールハンドラ

### ツール定義リスト

```rust
pub fn tool_definitions() -> Value {
    json!([
        {
            "name": "favnir_run",
            "description": "Execute a Favnir code snippet",
            "inputSchema": { "type": "object", "properties": { "source": { "type": "string" } }, "required": ["source"] }
        },
        {
            "name": "favnir_check",
            "description": "Type-check a Favnir code snippet without executing",
            "inputSchema": { "type": "object", "properties": { "source": { "type": "string" } }, "required": ["source"] }
        },
        {
            "name": "favnir_test",
            "description": "Run test blocks in a Favnir source file",
            "inputSchema": { "type": "object", "properties": { "source": { "type": "string" } }, "required": ["source"] }
        },
        {
            "name": "favnir_list_runes",
            "description": "List all available Rune modules and their exported functions",
            "inputSchema": { "type": "object", "properties": {} }
        },
        {
            "name": "favnir_rune_docs",
            "description": "Get documentation for a specific Rune",
            "inputSchema": { "type": "object", "properties": { "rune": { "type": "string" } }, "required": ["rune"] }
        }
    ])
}
```

### `favnir_check` 実装

```rust
fn tool_favnir_check(source: &str) -> Value {
    match Parser::parse_str(source) {
        Err(e) => tool_error(format!("Parse error: {}", e.message)),
        Ok(program) => {
            let mut checker = Checker::new();
            let errs = checker.check_program(&program);
            if errs.is_empty() {
                tool_text("OK: no type errors")
            } else {
                let msg = errs.iter().map(|e| format!("{} at line {}: {}", e.code, e.span.line, e.message)).collect::<Vec<_>>().join("\n");
                tool_error_text(msg)
            }
        }
    }
}
```

### `favnir_run` 実装

```rust
fn tool_favnir_run(source: &str) -> Value {
    // 一時ファイル書き込み → exec_project_main_source 相当
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("__mcp__.fav");
    std::fs::write(&path, source).unwrap();
    // stdout キャプチャは難しいため、
    // VM の print 出力を文字列バッファに集める専用パスを用意するか、
    // 一時ファイルにリダイレクトする
    // v4.9.0 では: parse + check + compile + exec し、
    // VM が panic しなければ "Executed successfully" を返す
    // （stdout キャプチャは v4.10.0 以降）
    ...
}
```

> **注**: stdout キャプチャの完全実装は複雑なため、v4.9.0 では型チェック + コンパイル成功を確認して "Executed successfully (output not captured)" を返す。実際の実行出力は将来バージョンで対応。

### `favnir_test` 実装

```rust
fn tool_favnir_test(source: &str) -> Value {
    // parse → check → compile → exec each test block
    // pass/fail カウントを返す
    ...
}
```

### `favnir_list_runes` 実装

```rust
fn tool_favnir_list_runes() -> Value {
    // 既知 Rune のハードコードリスト（v4.9.0 はシンプルに）
    let runes = [
        ("db", vec!["connect", "query", "execute", "query_one", "paginate", "batch_insert", ...]),
        ("http", vec!["get", "post", "put", "delete", "patch", ...]),
        ...
    ];
    let text = runes.iter().map(|(name, fns)| format!("{}: {}", name, fns.join(", "))).collect::<Vec<_>>().join("\n");
    tool_text(text)
}
```

### ヘルパー関数

```rust
fn tool_text(text: impl Into<String>) -> Value {
    json!({ "content": [{ "type": "text", "text": text.into() }] })
}

fn tool_error(text: impl Into<String>) -> Value {
    json!({ "content": [{ "type": "text", "text": text.into() }], "isError": true })
}
```

---

## Phase 5: resources.rs — リソースハンドラ

### リソース一覧

```rust
pub fn resource_list() -> Value {
    json!([
        { "uri": "favnir://docs/stdlib", "name": "Favnir stdlib docs", "mimeType": "text/markdown" },
        { "uri": "favnir://runes/db",    "name": "db Rune source",      "mimeType": "text/plain" },
        { "uri": "favnir://runes/http",  "name": "http Rune source",    "mimeType": "text/plain" },
        { "uri": "favnir://runes/log",   "name": "log Rune source",     "mimeType": "text/plain" },
        { "uri": "favnir://runes/gen",   "name": "gen Rune source",     "mimeType": "text/plain" },
        { "uri": "favnir://runes/auth",  "name": "auth Rune source",    "mimeType": "text/plain" },
        { "uri": "favnir://runes/env",   "name": "env Rune source",     "mimeType": "text/plain" },
        { "uri": "favnir://project/files", "name": "Project .fav files", "mimeType": "text/plain" }
    ])
}
```

### `resources/read` 実装

- `favnir://docs/stdlib` → ハードコードされた stdlib Markdown を返す
- `favnir://runes/{name}` → `runes/<name>/<name>.fav` を読んで返す（パスは実行ファイル基準 or 環境変数 `FAV_RUNES_PATH`）
- `favnir://project/files` → `std::env::current_dir()` 配下の `.fav` ファイル一覧

---

## Phase 6: prompts.rs — プロンプトハンドラ

### プロンプト一覧

```rust
pub fn prompt_list() -> Value {
    json!([
        {
            "name": "write_pipeline",
            "description": "Generate a Favnir data pipeline template",
            "arguments": [
                { "name": "source_type", "description": "Input data source type (csv/parquet/db/http)", "required": true },
                { "name": "output_type", "description": "Output destination type (csv/parquet/db/stdout)", "required": true }
            ]
        },
        {
            "name": "fix_type_error",
            "description": "Fix a Favnir type error",
            "arguments": [
                { "name": "source", "description": "Favnir source code with the error", "required": true },
                { "name": "error",  "description": "The error message", "required": true }
            ]
        }
    ])
}
```

---

## Phase 7: CLI 配線

### `fav/src/main.rs`

```rust
["mcp"] | ["mcp", ..] => {
    mcp::run_mcp_server();
}
```

ヘルプテキストに追加:
```
  mcp                    Start MCP server (JSON-RPC over stdin/stdout)
```

### `fav/src/lib.rs` または `mod` 宣言

```rust
pub mod mcp;
```

---

## Phase 8: テスト

`fav/src/mcp/mod.rs` 内に `#[cfg(test)]` モジュール。

テスト手法: LSP と同様に `Vec<u8>` バッファに書き込む `McpServer<Vec<u8>>` を作り、メッセージを直接 `handle()` してレスポンスを検証。

```rust
fn make_request(id: u64, method: &str, params: Value) -> Vec<u8> {
    let body = serde_json::to_string(&json!({
        "jsonrpc": "2.0", "id": id, "method": method, "params": params
    })).unwrap();
    format!("Content-Length: {}\r\n\r\n{}", body.len(), body).into_bytes()
}

fn parse_last_response(buf: &[u8]) -> Value {
    // Content-Length フレームを逆パース
    ...
}
```

---

## 実装メモ

- **Content-Length フレーミング**: LSP の `read_message` / `write_message` をそのままコピーして `mcp/mod.rs` に置く（依存関係をシンプルに保つ）
- **stdout フラッシュ**: `write_message` 後に `writer.flush()` 必須
- **`favnir_run` の stdout キャプチャ**: v4.9.0 では行わない。型チェック + コンパイル成功を返す
- **Rune パス解決**: `std::env::var("FAV_RUNES_PATH")` → なければ `./runes/` → なければ空リスト
- **`initialized` 通知**: 無視（レスポンス不要）
- **`$/cancelRequest`**: 無視
