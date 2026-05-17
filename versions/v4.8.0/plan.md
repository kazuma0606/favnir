# Favnir v4.8.0 実装計画 — LSP

作成日: 2026-05-17

---

## 実装フェーズ概要

| Phase | 内容 | 主要ファイル |
|-------|------|-------------|
| 0 | バージョン更新 | Cargo.toml, main.rs |
| 1 | LSP モジュール骨格 | src/lsp/ (mod, transport, types) |
| 2 | JSON-RPC サーバーループ | src/lsp/mod.rs |
| 3 | 診断 (Diagnostics) | src/lsp/handlers.rs, state.rs |
| 4 | 補完 (Completion) | src/lsp/completion.rs |
| 5 | ホバー (Hover) | src/lsp/hover.rs |
| 6 | 定義ジャンプ (Definition) | src/lsp/handlers.rs |
| 7 | `fav lsp` CLI コマンド | src/driver.rs, src/main.rs |
| 8 | テスト | src/lsp/*.rs, src/driver.rs |

---

## Phase 0: バージョン更新

`fav/Cargo.toml` の version を `"4.8.0"` に更新。
`fav/src/main.rs` のヘルプ文字列を `v4.8.0` に更新。

---

## Phase 1: LSP モジュール骨格

### `fav/src/lsp/mod.rs`

```rust
pub mod transport;
pub mod types;
pub mod state;
pub mod handlers;
pub mod completion;
pub mod hover;

pub use mod_impl::cmd_lsp;
```

### `fav/src/lsp/transport.rs`

Content-Length フレーミングの読み書き：

```rust
/// stdin から次の JSON-RPC メッセージを読む。
/// フォーマット: "Content-Length: N\r\n\r\n<N bytes of JSON>"
pub fn read_message(reader: &mut impl BufRead) -> Option<String> {
    let mut content_length = 0usize;
    loop {
        let mut header = String::new();
        reader.read_line(&mut header).ok()?;
        let header = header.trim();
        if header.is_empty() { break; }
        if let Some(val) = header.strip_prefix("Content-Length: ") {
            content_length = val.parse().ok()?;
        }
    }
    if content_length == 0 { return None; }
    let mut buf = vec![0u8; content_length];
    reader.read_exact(&mut buf).ok()?;
    String::from_utf8(buf).ok()
}

/// stdout へ JSON-RPC メッセージを書く。
pub fn write_message(writer: &mut impl Write, json: &str) {
    let bytes = json.as_bytes();
    write!(writer, "Content-Length: {}\r\n\r\n", bytes.len()).unwrap();
    writer.write_all(bytes).unwrap();
    writer.flush().unwrap();
}
```

### `fav/src/lsp/types.rs`

最小限の LSP 型定義（serde_json ベース、lsp-types クレート不使用）：

```rust
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Deserialize)]
pub struct RpcMessage {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: Option<String>,
    pub params: Option<Value>,
    pub result: Option<Value>,
    pub error: Option<Value>,
}

#[derive(Serialize)]
pub struct RpcResponse {
    pub jsonrpc: &'static str,
    pub id: Value,
    pub result: Value,
}

#[derive(Serialize)]
pub struct RpcNotification {
    pub jsonrpc: &'static str,
    pub method: &'static str,
    pub params: Value,
}

#[derive(Serialize)]
pub struct RpcError {
    pub jsonrpc: &'static str,
    pub id: Value,
    pub error: RpcErrorBody,
}

#[derive(Serialize)]
pub struct RpcErrorBody {
    pub code: i32,
    pub message: String,
}

// LSP geometry types
#[derive(Serialize, Clone)]
pub struct Position { pub line: u32, pub character: u32 }

#[derive(Serialize, Clone)]
pub struct Range { pub start: Position, pub end: Position }

#[derive(Serialize)]
pub struct Diagnostic {
    pub range: Range,
    pub severity: u32,   // 1=Error, 2=Warning, 3=Info, 4=Hint
    pub code: String,
    pub source: String,
    pub message: String,
}

#[derive(Serialize)]
pub struct CompletionItem {
    pub label: String,
    pub kind: u32,         // 1=Text, 2=Method, 3=Function, 14=Keyword
    #[serde(rename = "insertText", skip_serializing_if = "Option::is_none")]
    pub insert_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub documentation: Option<String>,
}

#[derive(Serialize)]
pub struct Hover {
    pub contents: HoverContents,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range: Option<Range>,
}

#[derive(Serialize)]
pub struct HoverContents {
    pub kind: String,    // "markdown" | "plaintext"
    pub value: String,
}

#[derive(Serialize)]
pub struct Location {
    pub uri: String,
    pub range: Range,
}
```

---

## Phase 2: JSON-RPC サーバーループ

### `fav/src/lsp/mod.rs` — `cmd_lsp()`

```rust
pub fn cmd_lsp() {
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut reader = std::io::BufReader::new(stdin.lock());
    let mut writer = std::io::BufWriter::new(stdout.lock());
    let mut state = state::LspState::new();

    loop {
        let msg = match transport::read_message(&mut reader) {
            Some(m) => m,
            None => break,
        };
        let rpc: RpcMessage = match serde_json::from_str(&msg) {
            Ok(m) => m,
            Err(e) => {
                lsp_log(&format!("JSON-RPC decode error: {}", e));
                continue;
            }
        };
        let method = rpc.method.as_deref().unwrap_or("");
        match method {
            "initialize"              => handlers::on_initialize(&mut writer, &rpc),
            "initialized"             => {}  // no-op
            "shutdown"                => handlers::on_shutdown(&mut writer, &rpc),
            "exit"                    => std::process::exit(0),
            "$/cancelRequest"         => {}  // no-op
            "textDocument/didOpen"    => handlers::on_did_open(&mut writer, &rpc, &mut state),
            "textDocument/didChange"  => handlers::on_did_change(&mut writer, &rpc, &mut state),
            "textDocument/didClose"   => handlers::on_did_close(&mut writer, &rpc, &mut state),
            "textDocument/completion" => handlers::on_completion(&mut writer, &rpc, &state),
            "textDocument/hover"      => handlers::on_hover(&mut writer, &rpc, &state),
            "textDocument/definition" => handlers::on_definition(&mut writer, &rpc, &state),
            _ => {
                // 未知のリクエスト → MethodNotFound エラー（id がある場合のみ）
                if rpc.id.is_some() {
                    handlers::send_method_not_found(&mut writer, rpc.id.unwrap());
                }
            }
        }
    }
}
```

---

## Phase 3: 診断 (Diagnostics)

### `fav/src/lsp/state.rs`

```rust
use std::collections::HashMap;

pub struct DocumentState {
    pub uri: String,
    pub text: String,
    pub version: i64,
}

pub struct LspState {
    pub documents: HashMap<String, DocumentState>,
    /// runes_root: fav.toml の [runes] path または None
    pub runes_root: Option<std::path::PathBuf>,
}

impl LspState {
    pub fn new() -> Self {
        // fav.toml を現在のディレクトリから上方探索して runes_root を決定
        let runes_root = crate::toml::FavToml::find_root(&std::env::current_dir().unwrap_or_default())
            .and_then(|root| crate::toml::FavToml::load(&root))
            .map(|toml| ...);
        LspState { documents: HashMap::new(), runes_root }
    }
}
```

### `fav/src/lsp/handlers.rs` — `analyze_and_publish`

```rust
fn analyze_and_publish(
    writer: &mut impl Write,
    uri: &str,
    text: &str,
    state: &LspState,
) {
    let diagnostics = match crate::frontend::parser::Parser::parse_str(text, uri) {
        Err(e) => vec![parse_error_to_diagnostic(e)],
        Ok(prog) => {
            // rune import を解決（state.runes_root があれば）
            let merged = if let Some(root) = &state.runes_root {
                // load_all_items を使ってインポートを解決
                ...
            } else {
                prog
            };
            let mut checker = crate::middle::checker::Checker::new_with_project_root(root);
            let (errors, _) = checker.check_with_self(&merged);
            errors.iter().map(type_error_to_diagnostic).collect()
        }
    };
    let notification = RpcNotification {
        jsonrpc: "2.0",
        method: "textDocument/publishDiagnostics",
        params: serde_json::json!({
            "uri": uri,
            "diagnostics": diagnostics,
        }),
    };
    transport::write_message(writer, &serde_json::to_string(&notification).unwrap());
}
```

### Span → LSP Range 変換

```rust
fn span_to_range(span: &crate::ast::Span, text: &str) -> Range {
    let start_line = span.line.saturating_sub(1);   // 1-based → 0-based
    let start_char = span.col;                        // already 0-based
    // end: byte offset → line/char
    let end = byte_offset_to_position(text, span.end);
    Range {
        start: Position { line: start_line as u32, character: start_char as u32 },
        end,
    }
}

fn byte_offset_to_position(text: &str, offset: usize) -> Position {
    let clamped = offset.min(text.len());
    let prefix = &text[..clamped];
    let line = prefix.chars().filter(|&c| c == '\n').count() as u32;
    let last_newline = prefix.rfind('\n').map(|i| i + 1).unwrap_or(0);
    let character = prefix[last_newline..].chars().count() as u32;
    Position { line, character }
}
```

---

## Phase 4: 補完 (Completion)

### `fav/src/lsp/completion.rs`

```rust
/// カーソル前のテキストを解析して補完候補を返す。
pub fn get_completions(text: &str, line: u32, character: u32) -> Vec<CompletionItem> {
    let line_text = get_line(text, line);
    let prefix = &line_text[..character.min(line_text.len() as u32) as usize];

    // "Namespace." パターン → namespace メソッド補完
    if let Some(ns) = extract_namespace_prefix(prefix) {
        return namespace_completions(ns);
    }

    // "rune_alias." パターン → rune 関数補完
    // (現在のファイルの import rune "X" as Y を解析)
    // ...

    // それ以外 → キーワード + 組み込み namespace 名補完
    keyword_completions()
}

fn extract_namespace_prefix(line: &str) -> Option<&str> {
    // line が "Foo.bar" のパターンで終わる → "Foo" を返す
    let last_dot = line.rfind('.')?;
    let before_dot = &line[..last_dot];
    let start = before_dot.rfind(|c: char| !c.is_alphanumeric() && c != '_')
        .map(|i| i + 1)
        .unwrap_or(0);
    let ns = &before_dot[start..];
    if ns.is_empty() { None } else { Some(ns) }
}
```

### キーワード補完リスト

```rust
const KEYWORDS: &[(&str, &str)] = &[
    ("fn",       "function definition"),
    ("public",   "public visibility"),
    ("type",     "type alias or record"),
    ("import",   "module import"),
    ("match",    "pattern match expression"),
    ("true",     "boolean true"),
    ("false",    "boolean false"),
    ("stage",    "pipeline stage"),
    ("test",     "test block"),
    ("bind",     "bind Task value"),
    ("effect",   "effect declaration"),
    ("abstract", "abstract definition"),
    ("seq",      "sequential pipeline"),
];
```

### 組み込み namespace メソッドリスト

各 namespace のメソッドをハードコードで定義する。例：

```rust
fn namespace_completions(ns: &str) -> Vec<CompletionItem> {
    let methods: &[(&str, &str, &str)] = match ns {
        "String" => &[
            ("contains",     "fn contains(s: String, sub: String) -> Bool",       "Returns true if the string contains the substring"),
            ("split",        "fn split(s: String, sep: String) -> List<String>",  "Splits string by separator"),
            ("concat",       "fn concat(a: String, b: String) -> String",         "Concatenates two strings"),
            ("length",       "fn length(s: String) -> Int",                       "Returns string length"),
            ("starts_with",  "fn starts_with(s: String, prefix: String) -> Bool","Returns true if starts with prefix"),
            ("ends_with",    "fn ends_with(s: String, suffix: String) -> Bool",   "Returns true if ends with suffix"),
            ("trim",         "fn trim(s: String) -> String",                      "Trims whitespace"),
            ("to_uppercase", "fn to_uppercase(s: String) -> String",              "Converts to uppercase"),
            ("to_lowercase", "fn to_lowercase(s: String) -> String",              "Converts to lowercase"),
            ("replace",      "fn replace(s: String, from: String, to: String) -> String", "Replaces occurrences"),
            ("from_bool",    "fn from_bool(b: Bool) -> String",                   "Converts Bool to String"),
            ("from_int",     "fn from_int(n: Int) -> String",                     "Converts Int to String"),
        ],
        "List" => &[
            ("map",      "fn map(list: List<A>, f: A -> B) -> List<B>",         "Transforms each element"),
            ("filter",   "fn filter(list: List<A>, f: A -> Bool) -> List<A>",   "Filters elements"),
            ("fold",     "fn fold(list: List<A>, init: B, f: B -> A -> B) -> B","Folds list"),
            ("any",      "fn any(list: List<A>, f: A -> Bool) -> Bool",         "Returns true if any matches"),
            ("all",      "fn all(list: List<A>, f: A -> Bool) -> Bool",         "Returns true if all match"),
            ("length",   "fn length(list: List<A>) -> Int",                     "Returns list length"),
            ("append",   "fn append(list: List<A>, item: A) -> List<A>",        "Appends an element"),
            ("first",    "fn first(list: List<A>) -> Option<A>",                "Returns first element"),
            ("last",     "fn last(list: List<A>) -> Option<A>",                 "Returns last element"),
            ("reverse",  "fn reverse(list: List<A>) -> List<A>",                "Reverses the list"),
            ("sort_by",  "fn sort_by(list: List<A>, f: A -> A -> Int) -> List<A>","Sorts by comparator"),
        ],
        // ... Map, Option, Result, Int, Bool, DB, Http, Log, Env, etc.
        _ => return vec![],
    };
    methods.iter().map(|(label, detail, doc)| CompletionItem {
        label: label.to_string(),
        kind: 2, // Method
        insert_text: None,
        detail: Some(detail.to_string()),
        documentation: Some(doc.to_string()),
    }).collect()
}
```

---

## Phase 5: ホバー (Hover)

### `fav/src/lsp/hover.rs`

```rust
pub fn get_hover(
    text: &str,
    line: u32,
    character: u32,
    state: &LspState,
) -> Option<Hover> {
    let word = get_word_at(text, line, character)?;

    // 組み込み namespace の場合: 説明を返す
    if let Some(desc) = builtin_namespace_hover(&word) {
        return Some(Hover {
            contents: HoverContents { kind: "markdown".into(), value: desc },
            range: None,
        });
    }

    // ユーザー定義関数: checker のスコープから型を検索（将来実装）
    None
}

fn builtin_namespace_hover(word: &str) -> Option<String> {
    match word {
        "String" => Some("**String** — Built-in string type and utilities\n\n`contains`, `split`, `concat`, `length`, ...".into()),
        "List"   => Some("**List<A>** — Immutable list type\n\n`map`, `filter`, `fold`, `any`, `all`, ...".into()),
        "Map"    => Some("**Map<K, V>** — Immutable map type\n\n`get`, `set`, `delete`, `keys`, `values`, ...".into()),
        "DB"     => Some("**DB** — Database access (requires `!Db` effect)\n\n`connect`, `query`, `execute`, ...".into()),
        "Log"    => Some("**Log** — Structured logging (requires `!Io` effect)\n\n`emit_raw`, `metric_raw`, `map_to_json_raw`".into()),
        "Env"    => Some("**Env** — Environment variable access (requires `!Env` effect)\n\n`get_raw`, `require_raw`, `get_int_raw`, `get_bool_raw`, ...".into()),
        _ => None,
    }
}
```

---

## Phase 6: 定義ジャンプ (Definition)

### `handlers.rs` — `on_definition`

```rust
pub fn on_definition(writer: &mut impl Write, rpc: &RpcMessage, state: &LspState) {
    let id = rpc.id.clone().unwrap_or(Value::Null);
    let params = rpc.params.as_ref().unwrap();
    let uri = params["textDocument"]["uri"].as_str().unwrap_or("");
    let line = params["position"]["line"].as_u64().unwrap_or(0) as u32;
    let char = params["position"]["character"].as_u64().unwrap_or(0) as u32;

    let result = state.documents.get(uri)
        .and_then(|doc| find_definition(&doc.text, line, char, state, uri))
        .map(|loc| serde_json::to_value(loc).unwrap())
        .unwrap_or(Value::Null);

    send_response(writer, id, result);
}

fn find_definition(
    text: &str,
    line: u32,
    char: u32,
    state: &LspState,
    current_uri: &str,
) -> Option<Location> {
    // 1. カーソル位置の単語を特定
    // 2. 同ファイルのトップレベル fn 定義から検索
    // 3. 見つからなければ rune ファイルを探索
    // v4.8.0 では同ファイル内の定義のみ対応
    let word = get_word_at(text, line, char)?;
    find_fn_in_text(text, &word, current_uri)
}
```

---

## Phase 7: CLI コマンド追加

### `fav/src/main.rs`

```rust
["lsp"] => cmd_lsp(),
```

### `fav/src/driver.rs`

```rust
pub fn cmd_lsp() {
    crate::lsp::cmd_lsp();
}
```

### HELP テキスト追加

```
  lsp               Start the Favnir Language Server (LSP)
```

---

## Phase 8: テスト

### `src/lsp/transport.rs` の単体テスト

```rust
#[test]
fn test_read_write_roundtrip() {
    // write_message → read_message が同じ内容を返す
}

#[test]
fn test_content_length_framing() {
    // ヘッダーが正しくパースされる
}
```

### `src/lsp/handlers.rs` の単体テスト

```rust
#[test]
fn test_diagnostics_from_parse_error() {
    // 構文エラーを含むソースが診断を生成する
}

#[test]
fn test_diagnostics_from_type_error() {
    // 型エラーを含むソースが診断を生成する
}

#[test]
fn test_no_diagnostics_on_valid() {
    // 正しいソースで診断が空
}
```

### `src/lsp/completion.rs` の単体テスト

```rust
#[test]
fn test_keyword_completions_returned() {
    // 空行でキーワードが補完される
}

#[test]
fn test_string_namespace_completions() {
    // "String." で contains 等が補完される
}

#[test]
fn test_namespace_prefix_extraction() {
    // "String.con" → "String"
}
```

### `driver.rs` 統合テスト

```rust
#[test]
fn lsp_initialize_returns_capabilities() {
    // initialize リクエストに対して正しいレスポンスを返す
}

#[test]
fn lsp_diagnostics_on_undefined_variable() {
    // 未定義変数を含むソースで E0102 診断が生成される
}
```

---

## 注意点・落とし穴

1. **stdout フラッシュ**: `write_message` は必ず `flush()` を呼ぶ。バッファリングで LSP クライアントがハングする。
2. **Span.col の基数**: Favnir の `col` が 0-based か 1-based かを `Span::new` の実装で確認してから使用。
3. **URI フォーマット**: LSP は `file:///path/to/file.fav` 形式。Windows では `file:///C:/...` の形式に変換。
4. **`cmd_lsp` の登録**: `main.rs` に `lsp` コマンドを追加し、`driver.rs` に `pub fn cmd_lsp` を追加、`src/lsp/mod.rs` をモジュールとして登録（`lib.rs` または `main.rs` の `mod` 宣言）。
5. **チェッカーの再利用**: `Checker::new_with_project_root` が存在するか確認。なければ `Checker::new_with_resolver` を使う。
6. **rune import の解決**: `analyze_and_publish` で `load_all_items` を呼ぶには `FavToml` が必要。LSP 起動時に `fav.toml` を一度ロードして `LspState` に保持する。
7. **スレッド安全性**: v4.8.0 は単一スレッド（同期 IO）。`thread_local!` の LSP テスト中の副作用に注意。
8. **テスト中の LSP サーバーループ**: `cmd_lsp` は無限ループのため、テストではループを呼ばず個別ハンドラを単体テストする。
