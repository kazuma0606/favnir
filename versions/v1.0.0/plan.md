# Favnir v1.0.0 実装計画

作成日: 2026-04-30

---

## Phase 0: 言語仕様書

### 作業内容

`versions/v1.0.0/langspec.md` を作成する（コード変更なし）。

### 構成

```
langspec.md
  1. 基本型システム
  2. 関数・トランスフォーマー・フロー
  3. effect system
  4. パターンマッチング
  5. モジュールシステム
  6. 標準ライブラリ
  7. CLI リファレンス
  8. エラーコード一覧
  9. 後方互換ポリシー
```

---

## Phase 1: LSP 最小実装

### ディレクトリ構成

```
src/lsp/
  mod.rs            — run_lsp_server() エントリポイント
  protocol.rs       — JSON-RPC + LSP 型定義
  document_store.rs — URI → (source, AST, 型情報) キャッシュ
  hover.rs          — hover ハンドラ
  diagnostics.rs    — TypeError → LSP Diagnostic 変換
```

`main.rs` に `"lsp"` コマンドを追加:
```rust
Some("lsp") => {
    let port = // --port パース
    crate::lsp::run_lsp_server(port);
}
```

### JSON-RPC ループ (mod.rs)

```rust
pub fn run_lsp_server(port: Option<u16>) {
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut server = LspServer::new(stdout);
    loop {
        let msg = read_message(&stdin);  // Content-Length ヘッダ読み取り
        server.handle(msg);
    }
}
```

メッセージ形式:
```
Content-Length: <bytes>\r\n
\r\n
<JSON>
```

### protocol.rs の型定義

```rust
#[derive(Deserialize)]
struct RpcRequest {
    id: Option<serde_json::Value>,
    method: String,
    params: serde_json::Value,
}

#[derive(Serialize)]
struct RpcResponse {
    jsonrpc: &'static str,
    id: serde_json::Value,
    result: serde_json::Value,
}

struct Position { line: u32, character: u32 }
struct Range { start: Position, end: Position }
struct Location { uri: String, range: Range }
struct Diagnostic { range: Range, severity: u32, code: String, message: String }
struct Hover { contents: MarkupContent }
struct MarkupContent { kind: String, value: String }
```

### document_store.rs

```rust
pub struct CheckedDoc {
    pub source: String,
    pub program: ast::Program,
    pub errors: Vec<TypeError>,
    pub type_at: HashMap<Span, Type>,       // ← Checker から収集
    pub effect_at: HashMap<Span, Vec<Effect>>,
}

pub struct DocumentStore {
    docs: HashMap<String, CheckedDoc>,  // URI → doc
}

impl DocumentStore {
    pub fn open(&mut self, uri: &str, source: String) { ... }
    pub fn change(&mut self, uri: &str, source: String) { ... }
    pub fn get(&self, uri: &str) -> Option<&CheckedDoc> { ... }
}
```

### Checker の変更: type_at マップ

`Checker` に以下を追加:

```rust
pub type_at: HashMap<Span, Type>,
```

`check_expr` の各分岐で `self.type_at.insert(expr.span(), ty.clone())` を呼ぶ。
ただし **全 AST ノードに span を持たせる** のは大変なため、
まず `Expr::Ident` と `Expr::Call` の span のみ記録する最小実装とする。

### hover.rs

```rust
pub fn handle_hover(doc: &CheckedDoc, pos: Position) -> Option<Hover> {
    // 1. pos を (line, col) に変換
    // 2. type_at の全 Span をスキャンして pos を含むものを探す
    // 3. 見つかったら Type + Effect を Markdown 文字列にして返す
    let ty = find_type_at_pos(&doc.type_at, pos)?;
    let contents = format!("```\n{}\n```", format_type(&ty));
    Some(Hover { contents: MarkupContent { kind: "markdown".into(), value: contents } })
}

fn find_type_at_pos(map: &HashMap<Span, Type>, pos: Position) -> Option<&Type> {
    map.iter()
       .filter(|(span, _)| span_contains(span, pos))
       .min_by_key(|(span, _)| span.end - span.start)  // 最小スパンを選ぶ
       .map(|(_, ty)| ty)
}
```

### diagnostics.rs

```rust
pub fn errors_to_diagnostics(errors: &[TypeError]) -> Vec<Diagnostic> {
    errors.iter().map(|e| Diagnostic {
        range: span_to_range(&e.span),
        severity: 1,  // Error
        code: e.code.clone(),
        message: e.message.clone(),
    }).collect()
}

fn span_to_range(span: &Span) -> Range {
    Range {
        start: Position { line: span.line.saturating_sub(1), character: span.col.saturating_sub(1) as u32 },
        end:   Position { line: span.line.saturating_sub(1), character: (span.col + (span.end - span.start).saturating_sub(1)) as u32 },
    }
}
```

### initialize レスポンス

```json
{
  "capabilities": {
    "textDocumentSync": 1,
    "hoverProvider": true,
    "definitionProvider": false
  }
}
```

定義ジャンプは v1.0.0 ではスタブ (空レスポンス) とする。

### Cargo.toml への追加

```toml
serde = { version = "1", features = ["derive"] }
```

(`serde_json` はすでに追加済み)

---

## Phase 2: WASM String 戻り値

### 変更箇所: wasm_codegen.rs

#### favnir_type_to_wasm_results の変更

```rust
pub fn favnir_type_to_wasm_results(ty: &Type) -> Result<Vec<ValType>, WasmCodegenError> {
    match ty {
        Type::String => Ok(vec![ValType::I32, ValType::I32]),  // ← W001 解除
        // 他は変わらず
    }
}
```

#### slot_map の型変更

```rust
// 変更前
slot_map: HashMap<u16, u32>           // slot → wasm_local_idx

// 変更後
slot_map: HashMap<u16, WasmLocal>

enum WasmLocal {
    Single(u32),                       // scalar
    StringPtrLen(u32, u32),            // (ptr_local, len_local)
}
```

#### パラメータの slot_map 構築

```rust
for (slot_idx, ty) in fn_def.param_tys.iter().enumerate() {
    match ty {
        Type::String => {
            slot_map.insert(slot_idx as u16, WasmLocal::StringPtrLen(next_local_idx, next_local_idx + 1));
            next_local_idx += 2;
        }
        _ => {
            slot_map.insert(slot_idx as u16, WasmLocal::Single(next_local_idx));
            next_local_idx += 1;
        }
    }
}
```

#### ローカル変数宣言

String 型のローカルは `(i32, i32)` × 2 として宣言:
```rust
Type::String => {
    local_decls.push((1, ValType::I32));  // ptr
    local_decls.push((1, ValType::I32));  // len
    slot_map.insert(slot, WasmLocal::StringPtrLen(next_local_idx, next_local_idx + 1));
    next_local_idx += 2;
}
```

#### emit_expr: IRExpr::Local の変更

```rust
IRExpr::Local(slot, _) => {
    match slot_map.get(slot) {
        Some(WasmLocal::Single(idx)) => {
            func.instruction(&Instruction::LocalGet(*idx));
        }
        Some(WasmLocal::StringPtrLen(ptr, len)) => {
            func.instruction(&Instruction::LocalGet(*ptr));
            func.instruction(&Instruction::LocalGet(*len));
        }
        None => return Err(...),
    }
    Ok(())
}
```

#### emit_stmt: IRStmt::Bind の変更

```rust
IRStmt::Bind(slot, expr) => {
    emit_expr(expr, ctx, slot_map, func)?;
    match slot_map.get(slot) {
        Some(WasmLocal::Single(idx)) => {
            func.instruction(&Instruction::LocalSet(*idx));
        }
        Some(WasmLocal::StringPtrLen(ptr, len)) => {
            // スタック: [..., ptr_val, len_val]
            func.instruction(&Instruction::LocalSet(*len));  // len を先に pop
            func.instruction(&Instruction::LocalSet(*ptr));  // ptr を pop
        }
        None => return Err(...),
    }
}
```

#### emit_stmt: IRStmt::Expr の Drop 変更

```rust
IRStmt::Expr(expr) => {
    emit_expr(expr, ctx, slot_map, func)?;
    let n = favnir_type_to_wasm_results(&resolved_expr_type(expr, ctx))?.len();
    for _ in 0..n {
        func.instruction(&Instruction::Drop);
    }
}
```

#### 単値チェックの削除

`single_wasm_valtype` は String → W001 を返していたが、
Phase 2 完了後は `block_type_for` では使わず、
`if/else` ブロックの BlockType 決定には別途対応が必要:

```rust
fn block_type_for(ty: &Type) -> Result<BlockType, WasmCodegenError> {
    match favnir_type_to_wasm_results(ty)?.as_slice() {
        [] => Ok(BlockType::Empty),
        [only] => Ok(BlockType::Result(*only)),
        _ => Err(WasmCodegenError::UnsupportedExpr(
            "multi-value block type not yet supported in wasm MVP if/else".into()
        )),
    }
}
```

つまり `if/else` の result が String 型の場合は引き続き W002。
String を返す `if/else` 式は v1.1.0 に先送り。

### wasm_exec.rs: String 戻り値の読み取り

`wasm_exec_main` は `main` のみ実行するので変更不要。
String を返す non-main 関数のテストは `#[cfg(test)]` の中で手動構築する。

---

## Phase 3: WASM クロージャ

### 設計

クロージャ = `(fn_table_idx: i32, env_ptr: i32)` の 2 値ペア。

#### 合成関数の生成

`IRExpr::Closure(fn_global_idx, captures, ty)` を見つけたとき:
1. キャプチャを受け取る合成関数 `$closure_N` を生成
   - シグネチャ: `(env_ptr: i32, ...original_params) -> return_ty`
   - 環境から captures を読み込んで元のクロージャ本体を実行
2. 合成関数を function table に登録
3. emit では `(I32Const(table_idx), env_ptr)` を push

#### 環境の書き込み

```rust
fn emit_env_write(captures: &[IRExpr], ctx: &WasmCodegenCtx, func: &mut Function) -> Result<u32, WasmCodegenError> {
    // bump_alloc(size) を call して env_ptr を取得
    // 各 capture の値を env_ptr + offset に store
    // env_ptr を返す
}
```

#### bump allocator

WASM linear memory の page 1 以降 (offset 65536) から bump 割り当て:

```rust
// グローバル変数として heap_ptr を追加
module.section(&GlobalSection::new().global(
    GlobalType { val_type: ValType::I32, mutable: true, shared: false },
    &ConstExpr::i32_const(65536),
));

// bump_alloc helper function を WASM 内に生成
// (global.get $heap_ptr) → ret addr
// (global.get $heap_ptr) (local.get $size) i32.add (global.set $heap_ptr)
```

#### クロージャ呼び出し

直接呼び出しのみサポート:
```
let f <- |x| x + 1
f(5)
```

これは `IRExpr::Call(IRExpr::Local(f_slot, _), [5])` のように表現される。
`f_slot` の型が `(fn_idx, env_ptr)` だと判断したら `call_indirect` を emit:

```rust
// emit_expr: Call(callee, args) where callee は クロージャ型
func.instruction(&Instruction::LocalGet(env_ptr_local));
// args を emit
func.instruction(&Instruction::LocalGet(fn_idx_local));
func.instruction(&Instruction::CallIndirect { type_index, table_index: 0 });
```

#### IRFnDef の変更

クロージャ対応のために `IRFnDef` に `is_closure: bool` フラグを追加し、
クロージャ由来の合成関数を識別する。

#### TableSection の追加

```rust
let mut table_section = TableSection::new();
table_section.table(TableType { element_type: RefType::FUNCREF, minimum: N, maximum: Some(N) });

let mut elem_section = ElementSection::new();
elem_section.active(Some(0), &ConstExpr::i32_const(0), Elements::Functions(&fn_indices));
```

---

## Phase 4: rune 依存管理

### toml.rs の拡張

```rust
pub struct FavToml {
    pub name: Option<String>,
    pub version: Option<String>,
    pub src: Option<String>,
    pub dependencies: HashMap<String, DependencySpec>,  // ← 追加
}

pub enum DependencySpec {
    Path { path: String },
    Registry { version: String, registry: String },
}
```

パーサーを拡張して `[dependencies]` セクションを読む:
```toml
[dependencies]
csv_helper = { path = "../csv_helper" }
data_utils = { version = "0.2.0", registry = "local" }
```

### fav.lock の読み書き

```rust
pub struct LockFile {
    pub packages: Vec<LockedPackage>,
}

pub struct LockedPackage {
    pub name: String,
    pub version: String,
    pub source: String,        // "path:...", "local:name@version"
    pub hash: String,          // sha256 of src/ contents
}
```

`fav.lock` は TOML フォーマット。`toml.rs` のミニパーサーで読み書き。

### `fav install` の実装

`driver.rs` に `pub fn cmd_install(rune: Option<&str>)`:

```rust
pub fn cmd_install(rune: Option<&str>) {
    let cwd = std::env::current_dir()...;
    let toml = FavToml::load(...)...;

    for (name, spec) in &toml.dependencies {
        match spec {
            DependencySpec::Path { path } => {
                // 相対パスを絶対パスに変換
                // ~/.fav/cache/<name>/ にシンボリックリンクまたはコピー
            }
            DependencySpec::Registry { version, registry: "local" } => {
                // ~/.fav/registry/<name>/<version>/ から ~/.fav/cache/<name>@<version>/ にコピー
            }
            _ => { eprintln!("error: unsupported registry type"); ... }
        }
    }

    // fav.lock を生成
}
```

### `fav publish` の実装

`pub fn cmd_publish()`:
1. `fav.toml` を読む
2. `src/` 配下を再帰的にリスト
3. `~/.fav/registry/<name>/<version>/` に src/ をコピー

### Cargo.toml への追加

sha256 計算のため:
```toml
sha2 = "0.10"
```

### `fav check` / `fav run` の依存解決

`load_all_items` 内で、`fav.lock` が存在する場合は
ロックされた依存の `src/` を追加で走査する:

```rust
fn resolve_dependencies(toml: &FavToml, root: &Path) -> Vec<PathBuf> {
    // fav.lock から各パッケージのソースパスを取得
    // ~/.fav/cache/<name>@<version>/src/ を返す
}
```

---

## Phase 5: ドキュメント + リリース

### README.md 完全版

`versions/v1.0.0/README_draft.md` に草稿を作成してから
プロジェクトルートの `README.md` に反映。

### RELEASE_NOTES.md

`versions/v1.0.0/RELEASE_NOTES.md`:
- v0.x.x からの機能追加一覧
- WASM の変更点 (String 戻り値、クロージャ)
- LSP の設定例
- 既知の制限事項

### langspec.md

`versions/v1.0.0/langspec.md`: 仕様書本体。

---

## テスト戦略

### LSP テスト

```rust
// lsp/mod.rs の #[cfg(test)]
fn test_hover_on_typed_identifier() { ... }
fn test_diagnostics_for_type_error() { ... }
fn test_initialize_response() { ... }
```

LSP は stdin/stdout 依存なのでユニットテストは
`DocumentStore` と `handle_hover` を直接テストする。

### WASM String テスト

```rust
// wasm_codegen.rs の #[cfg(test)]
fn wasm_string_return_greet() { ... }    // fn greet() -> String が動く
fn wasm_string_bind_local() { ... }     // let s <- greet(); IO.println(s) が動く
fn wasm_string_if_else_string_arg() { ... } // String を引数に渡せる
```

### WASM クロージャテスト

```rust
fn wasm_closure_direct_call() { ... }   // |x| x + 1 を作って直接呼ぶ
fn wasm_closure_capture() { ... }       // 外部変数をキャプチャ
```

### rune install テスト

```rust
fn install_local_path_dep() { ... }   // tempdir に rune を置いて install
fn publish_and_install_local_registry() { ... }
```

---

## 依存関係 (Cargo.toml 追加)

```toml
serde = { version = "1", features = ["derive"] }
sha2 = "0.10"
```

(`serde_json` は v0.9.0 時点で追加済み)

---

## 実装順序の推奨

```
Phase 0 (spec doc) → Phase 1 (LSP) → Phase 2 (WASM String) → Phase 3 (WASM closure) → Phase 4 (rune install) → Phase 5 (docs)
```

Phase 2 と Phase 3 は独立しているが、Phase 3 は Phase 2 の `WasmLocal` 型変更に依存するため
Phase 2 の後に着手する。

Phase 4 は完全独立。Phase 1 と並行して進めることができる。
