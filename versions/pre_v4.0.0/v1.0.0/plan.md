# Favnir v1.0.0 実装計画

作成日: 2026-04-30（Codex レビュー反映）

> スコープを守ることが最優先。各フェーズの Done definition を超えない。

---

## 実装順序

```
Phase 0 (version + 骨格)
  → Phase 1 (LSP)
  → Phase 2 (WASM String)   ← Phase 3 の前提
  → Phase 3 (WASM closure)
  → Phase 4 (rune install)  ← 独立。Phase 1 と並行可
  → Phase 5 (docs)
```

---

## Phase 0: バージョン更新 + 仕様書骨格

### Cargo.toml

```toml
version = "1.0.0"
```

### main.rs

```rust
const HELP: &str = "fav - Favnir language toolchain v1.0.0\n...";
```

### langspec.md

章立てと後方互換ポリシーのみ。内容は Phase 5 で埋める。

---

## Phase 1: LSP 最小実装

### ディレクトリ

```
src/lsp/
  mod.rs
  protocol.rs
  document_store.rs
  hover.rs
  diagnostics.rs
```

`main.rs` に `mod lsp;` と `"lsp"` コマンドを追加。

### JSON-RPC ループ（mod.rs）

```rust
pub fn run_lsp_server() {
    let stdin = std::io::stdin().lock();
    let stdout = std::io::stdout();
    let mut reader = std::io::BufReader::new(stdin);
    let mut server = LspServer::new(stdout);
    loop {
        match read_message(&mut reader) {
            Some(req) => server.handle(req),
            None => break,
        }
    }
}
```

メッセージフォーマット:
```
Content-Length: <N>\r\n
\r\n
<N bytes of JSON>
```

`read_message`: ヘッダから N を読み、N バイト読んで `serde_json::from_slice`。
`write_message`: `Content-Length: N\r\n\r\n<JSON>` を stdout に書く。

### protocol.rs

最小限の型定義。serde_json の `Value` を多用して型爆発を防ぐ。

```rust
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Deserialize, Debug)]
pub struct RpcRequest {
    pub id: Option<Value>,
    pub method: String,
    pub params: Value,
}

#[derive(Serialize)]
pub struct RpcResponse {
    pub jsonrpc: &'static str,
    pub id: Value,
    pub result: Value,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct Position { pub line: u32, pub character: u32 }

#[derive(Serialize, Deserialize, Clone)]
pub struct Range { pub start: Position, pub end: Position }

#[derive(Serialize)]
pub struct Diagnostic {
    pub range: Range,
    pub severity: u32,
    pub code: String,
    pub message: String,
}

#[derive(Serialize)]
pub struct Hover {
    pub contents: MarkupContent,
}

#[derive(Serialize)]
pub struct MarkupContent {
    pub kind: String,
    pub value: String,
}
```

### document_store.rs

```rust
use std::collections::HashMap;
use crate::middle::checker::{Checker, Type, TypeError};
use crate::frontend::parser::Parser;
use crate::ast::Span;

pub struct CheckedDoc {
    pub source: String,
    pub errors: Vec<TypeError>,
    pub type_at: HashMap<Span, Type>,
}

pub struct DocumentStore {
    docs: HashMap<String, CheckedDoc>,
}

impl DocumentStore {
    pub fn new() -> Self { Self { docs: HashMap::new() } }

    pub fn open_or_change(&mut self, uri: &str, source: String) {
        let program = match Parser::parse_str(&source, uri) {
            Ok(p) => p,
            Err(_) => {
                self.docs.insert(uri.to_string(), CheckedDoc {
                    source,
                    errors: vec![],
                    type_at: HashMap::new(),
                });
                return;
            }
        };
        let mut checker = Checker::new();  // type_at を記録するバージョン
        let errors = checker.check_with_self(&program);
        self.docs.insert(uri.to_string(), CheckedDoc {
            source,
            errors,
            type_at: checker.type_at,
        });
    }

    pub fn get(&self, uri: &str) -> Option<&CheckedDoc> {
        self.docs.get(uri)
    }
}
```

### Checker の変更（最小）

```rust
// checker.rs に追加
pub type_at: HashMap<Span, Type>,
```

`check_expr` の `Expr::Ident` と `Expr::Call` で:
```rust
self.type_at.insert(expr.span.clone(), ty.clone());
```

`Checker::new()` と既存コンストラクタで `type_at: HashMap::new()` を初期化。
既存テストへの影響なし（フィールド追加のみ）。

### hover.rs

```rust
use crate::middle::checker::Type;
use super::document_store::DocumentStore;
use super::protocol::{Hover, MarkupContent, Position};
use crate::ast::Span;

pub fn handle_hover(store: &DocumentStore, uri: &str, pos: Position) -> Option<Hover> {
    let doc = store.get(uri)?;
    let ty = find_type_at(&doc.type_at, pos)?;
    Some(Hover {
        contents: MarkupContent {
            kind: "markdown".into(),
            value: format!("```\n{}\n```", format_type(ty)),
        },
    })
}

fn find_type_at(map: &std::collections::HashMap<Span, Type>, pos: Position) -> Option<&Type> {
    map.iter()
        .filter(|(span, _)| span_contains(span, pos))
        .min_by_key(|(span, _)| span.end.saturating_sub(span.start))
        .map(|(_, ty)| ty)
}

fn span_contains(span: &Span, pos: Position) -> bool {
    let line = pos.line + 1;  // LSP は 0-origin, Span は 1-origin
    let col  = pos.character + 1;
    span.line == line as usize && span.col as u32 <= col
        && col < span.col as u32 + (span.end - span.start) as u32
}

fn format_type(ty: &Type) -> String {
    format!("{:?}", ty)  // 最小実装。後で format_type を整える
}
```

### diagnostics.rs

```rust
use crate::middle::checker::TypeError;
use super::protocol::{Diagnostic, Position, Range};

pub fn errors_to_diagnostics(errors: &[TypeError]) -> Vec<Diagnostic> {
    errors.iter().map(|e| {
        let line = e.span.line.saturating_sub(1) as u32;  // 0-origin
        let col  = e.span.col.saturating_sub(1) as u32;
        let len  = (e.span.end.saturating_sub(e.span.start)).max(1) as u32;
        Diagnostic {
            range: Range {
                start: Position { line, character: col },
                end:   Position { line, character: col + len },
            },
            severity: 1,
            code: e.code.clone(),
            message: e.message.clone(),
        }
    }).collect()
}
```

### LspServer のハンドラ（mod.rs）

```rust
impl LspServer {
    fn handle(&mut self, req: RpcRequest) {
        match req.method.as_str() {
            "initialize" => self.on_initialize(req),
            "initialized" => {}  // notification, no response
            "textDocument/didOpen" => self.on_did_open(req),
            "textDocument/didChange" => self.on_did_change(req),
            "textDocument/hover" => self.on_hover(req),
            "textDocument/definition" => self.respond_null(req),
            "shutdown" => self.respond_null(req),
            "exit" => std::process::exit(0),
            _ => {}  // unknown notification/request → ignore
        }
    }

    fn on_initialize(&mut self, req: RpcRequest) {
        self.send_response(req.id.unwrap_or(Value::Null), serde_json::json!({
            "capabilities": {
                "textDocumentSync": 1,
                "hoverProvider": true,
                "definitionProvider": false
            }
        }));
    }

    fn on_did_open(&mut self, req: RpcRequest) {
        let uri    = req.params["textDocument"]["uri"].as_str().unwrap_or("").to_string();
        let source = req.params["textDocument"]["text"].as_str().unwrap_or("").to_string();
        self.store.open_or_change(&uri, source);
        self.publish_diagnostics(&uri);
    }

    fn publish_diagnostics(&mut self, uri: &str) {
        let diags = self.store.get(uri)
            .map(|doc| errors_to_diagnostics(&doc.errors))
            .unwrap_or_default();
        let notif = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "textDocument/publishDiagnostics",
            "params": { "uri": uri, "diagnostics": diags }
        });
        write_message(&mut self.writer, &notif);
    }
}
```

### Cargo.toml 追加

```toml
serde = { version = "1", features = ["derive"] }
```

(`serde_json` は v0.9.0 時点で追加済み)

---

## Phase 2: WASM String 戻り値

### `WasmLocal` 型の追加

```rust
#[derive(Debug, Clone)]
enum WasmLocal {
    Single(u32),
    StringPtrLen(u32, u32),  // (ptr_local_idx, len_local_idx)
}
```

### `build_wasm_function` の変更

**パラメータ割り当て:**
```rust
for (slot, ty) in fn_def.param_tys.iter().enumerate() {
    match ty {
        Type::String => {
            slot_map.insert(slot as u16, WasmLocal::StringPtrLen(next, next + 1));
            next += 2;
        }
        _ => {
            slot_map.insert(slot as u16, WasmLocal::Single(next));
            next += 1;
        }
    }
}
```

**ローカル変数宣言:**
```rust
Type::String => {
    local_decls.push((1, ValType::I32));  // ptr
    local_decls.push((1, ValType::I32));  // len
    slot_map.insert(slot, WasmLocal::StringPtrLen(next, next + 1));
    next += 2;
}
```

### `emit_expr` の変更

**`IRExpr::Local`:**
```rust
IRExpr::Local(slot, _) => match slot_map.get(slot) {
    Some(WasmLocal::Single(idx)) => {
        func.instruction(&Instruction::LocalGet(*idx));
    }
    Some(WasmLocal::StringPtrLen(ptr, len)) => {
        func.instruction(&Instruction::LocalGet(*ptr));
        func.instruction(&Instruction::LocalGet(*len));
    }
    None => return Err(WasmCodegenError::UnsupportedExpr(...)),
},
```

### `emit_stmt` の変更

**`IRStmt::Bind` で String:**
```rust
// スタック: [..., ptr_val, len_val]
Some(WasmLocal::StringPtrLen(ptr, len)) => {
    func.instruction(&Instruction::LocalSet(*len));  // len を先に pop
    func.instruction(&Instruction::LocalSet(*ptr));
}
```

**`IRStmt::Expr` の Drop:**
```rust
let n = favnir_type_to_wasm_results(&resolved_expr_type(expr, ctx))
    .map(|v| v.len()).unwrap_or(0);
for _ in 0..n {
    func.instruction(&Instruction::Drop);
}
```

### `favnir_type_to_wasm_results` の変更

```rust
Type::String => Ok(vec![ValType::I32, ValType::I32]),  // W001 解除
```

### `block_type_for` の変更

String result の `if/else` は W002:
```rust
fn block_type_for(ty: &Type) -> Result<BlockType, WasmCodegenError> {
    match favnir_type_to_wasm_results(ty)?.as_slice() {
        []      => Ok(BlockType::Empty),
        [only]  => Ok(BlockType::Result(*only)),
        _       => Err(WasmCodegenError::UnsupportedExpr(
            "multi-value block (e.g. String-returning if/else) not supported in wasm MVP".into()
        )),
    }
}
```

---

## Phase 3: WASM クロージャ

### 設計

```
IRExpr::Closure(fn_idx, captures, ty)
  → 合成関数 $closure_N を生成 (env_ptr: i32, ...params)
  → bump_alloc で captures を線形メモリに書き込む
  → スタックに (table_idx: i32, env_ptr: i32) を push
```

### GlobalSection: heap_ptr

```rust
let mut global_section = GlobalSection::new();
global_section.global(
    GlobalType { val_type: ValType::I32, mutable: true, shared: false },
    &ConstExpr::i32_const(65536),
);
```

### bump_alloc 関数

FunctionSection + CodeSection に追加する内部 WASM 関数:

```
(func $bump_alloc (param $size i32) (result i32)
  global.get $heap_ptr     ;; 現在のヒープ先頭
  global.get $heap_ptr
  local.get $size
  i32.add
  global.set $heap_ptr     ;; heap_ptr を size 分進める
  ;; スタックに元の heap_ptr が残る
  end)
```

### 合成関数の生成

`IRExpr::Closure(fn_global_idx, captures, _)` を見たとき:

1. `fn_global_idx` の元関数シグネチャを取得
2. `(env_ptr: i32, param0, param1, ...) -> return_ty` の合成関数を生成
3. 合成関数本体:
   - `env_ptr + offset` から capture を load
   - load した値をローカルに bind
   - 元の関数本体を inline emit (または元関数を call)
4. 合成関数を FunctionSection + CodeSection に追加
5. TableSection の Element に登録 → table_idx を確定

### emit_expr: Closure

```rust
IRExpr::Closure(fn_global_idx, captures, _) => {
    // 1. env サイズ = captures の合計バイト数
    let env_size = captures.iter().map(|c| wasm_size_of(c.ty())).sum::<u32>();

    // 2. bump_alloc(env_size) を call → env_ptr がスタックに
    func.instruction(&Instruction::I32Const(env_size as i32));
    func.instruction(&Instruction::Call(bump_alloc_idx));

    // 3. env_ptr を ローカルに一時保存
    func.instruction(&Instruction::LocalTee(tmp_env_local));

    // 4. captures を env に store
    let mut offset = 0u32;
    for capture in captures {
        func.instruction(&Instruction::LocalGet(tmp_env_local));
        emit_expr(capture, ctx, slot_map, func)?;
        emit_store_for_type(capture.ty(), offset, func)?;
        offset += wasm_size_of(capture.ty());
    }

    // 5. (table_idx, env_ptr) をスタックに
    func.instruction(&Instruction::I32Const(table_idx as i32));
    func.instruction(&Instruction::LocalGet(tmp_env_local));
    Ok(())
}
```

### emit_expr: Call でクロージャ呼び出し

スタック上に `(table_idx, env_ptr)` がある場合:

```rust
// env_ptr を先頭に、次に args
func.instruction(&Instruction::LocalGet(env_ptr_local));
for arg in args {
    emit_expr(arg, ctx, slot_map, func)?;
}
func.instruction(&Instruction::LocalGet(fn_idx_local));
func.instruction(&Instruction::CallIndirect {
    type_index: closure_type_idx,
    table_index: 0,
});
```

### WasmLocal の拡張

```rust
enum WasmLocal {
    Single(u32),
    StringPtrLen(u32, u32),
    FnTableEnv(u32, u32),   // (fn_idx_local, env_ptr_local)
}
```

### TableSection + ElementSection

```rust
let mut table_section = TableSection::new();
table_section.table(TableType {
    element_type: RefType::FUNCREF,
    minimum: closure_count as u64,
    maximum: Some(closure_count as u64),
    ..Default::default()
});

let mut elem_section = ElementSection::new();
elem_section.active(
    Some(0),
    &ConstExpr::i32_const(0),
    Elements::Functions(&closure_fn_indices),
);
```

---

## Phase 4: rune 依存管理

### toml.rs の拡張

`FavToml` に `dependencies: HashMap<String, DependencySpec>` を追加。

```rust
pub enum DependencySpec {
    Path { path: String },
    Local { version: String },
}
```

パーサー拡張:
- `[dependencies]` セクションを検出
- `name = { path = "..." }` → `DependencySpec::Path`
- `name = { version = "...", registry = "local" }` → `DependencySpec::Local`

### lock.rs

```rust
pub struct LockedPackage {
    pub name: String,
    pub version: String,
    pub source: String,   // "path:../foo" or "local:foo@1.0.0"
}

pub struct LockFile {
    pub packages: Vec<LockedPackage>,
}

impl LockFile {
    pub fn load(path: &Path) -> Option<Self> { ... }  // miniparser で TOML 読む
    pub fn save(&self, path: &Path) -> Result<(), String> { ... }  // TOML 書く
}
```

`hash` フィールドは v1.0.0 では省略（v1.1.0 で追加）。

### cmd_install（driver.rs）

```rust
pub fn cmd_install(_rune: Option<&str>) {
    let cwd = std::env::current_dir()...;
    let root = FavToml::find_root(&cwd)...;
    let toml = FavToml::load(&root)...;

    let mut lock = LockFile { packages: vec![] };
    for (name, spec) in &toml.dependencies {
        match spec {
            DependencySpec::Path { path } => {
                let abs = root.join(path);
                // abs が存在するか確認
                lock.packages.push(LockedPackage {
                    name: name.clone(),
                    version: "0.0.0".into(),  // fav.toml から読む
                    source: format!("path:{path}"),
                });
            }
            DependencySpec::Local { version } => {
                let registry_path = home_dir().join(".fav/registry").join(name).join(version);
                // registry_path が存在するか確認
                lock.packages.push(LockedPackage {
                    name: name.clone(),
                    version: version.clone(),
                    source: format!("local:{name}@{version}"),
                });
            }
        }
        println!("resolved: {name}");
    }
    lock.save(&root.join("fav.lock"))...;
    println!("installed {} dependencies", lock.packages.len());
}
```

### cmd_publish（driver.rs）

```rust
pub fn cmd_publish() {
    let cwd = ...;
    let root = FavToml::find_root(&cwd)...;
    let toml = FavToml::load(&root)...;
    let name = toml.name.as_deref().unwrap_or("unnamed");
    let version = toml.version.as_deref().unwrap_or("0.0.0");
    let dest = home_dir().join(".fav/registry").join(name).join(version);
    std::fs::create_dir_all(&dest)...;
    // src/ 以下を dest/ にコピー
    copy_dir(&root.join(toml.src_dir(&root)), &dest)...;
    println!("published {name}@{version} to local registry");
}
```

---

## Phase 5: ドキュメント整備

### examples の追加

`examples/string_wasm.fav`:
```fav
public fn greet(name: String) -> String {
    name
}

public fn main() -> Unit !Io {
    IO.println(greet("Favnir"))
}
```

`examples/closures_wasm.fav`:
```fav
public fn main() -> Unit !Io {
    let f <- |x| x + 1
    IO.println_int(f(5))
}
```

### langspec.md の polish

Phase 0 の骨格に最低限の内容を追加:
- 基本型（定義 + 1行説明 + 例）
- effect system（種類一覧 + 使い方例）
- エラーコード一覧（E001–E040, W001–W004）

### README.md

既存 README を以下の構成に整理:
```
# Favnir
概要（3行）

## Install
## Quick Start
## CLI Reference
## WASM
## LSP（VS Code 設定例）
## rune dependencies
```

---

## Cargo.toml 追加依存

```toml
serde = { version = "1", features = ["derive"] }
```

(sha2 は fav.lock hash を v1.0.0 で省略するため不要)
