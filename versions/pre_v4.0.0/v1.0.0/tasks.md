# Favnir v1.0.0 タスク一覧 — 安定版

作成日: 2026-04-30（Codex レビュー反映）

> [x] 完了
>
> **ゴール**: LSP hover/diagnostics・WASM String/closure・rune install の揃った安定版
> **前提**: v0.9.0 完了（289 テスト通過）
>
> **スコープ管理が最優先。Done definition を超えない。**

---

## Phase 0: バージョン更新 + 仕様書骨格

### 0-1: バージョン更新

- [x] `Cargo.toml` の `version` を `"1.0.0"` に更新
- [x] `main.rs` の HELP テキストを `v1.0.0` に更新
- [x] `cargo build` が通ること

### 0-2: langspec.md 骨格作成

- [x] `versions/v1.0.0/langspec.md` を作成（章立てのみ）
  - [x] 1. 基本型
  - [x] 2. 関数・trf・flw
  - [x] 3. effect system
  - [x] 4. パターンマッチング
  - [x] 5. モジュールシステム
  - [x] 6. 標準ライブラリ
  - [x] 7. CLI リファレンス
  - [x] 8. エラーコード一覧
  - [x] 9. 後方互換ポリシー
- [x] 後方互換ポリシーの文章だけ書く（「v1.0.0 以降は破壊的変更なし」）

---

## Phase 1: LSP 最小実装

### 1-1: scaffold

- [x] `src/lsp/` ディレクトリを作成
- [x] `src/lsp/mod.rs` を作成（`pub fn run_lsp_server()` スタブ）
- [x] `src/lsp/protocol.rs` を作成
  - [x] `RpcRequest { id, method, params }` (serde Deserialize)
  - [x] `RpcResponse { jsonrpc, id, result }` (serde Serialize)
  - [x] `Position { line: u32, character: u32 }`
  - [x] `Range { start, end }`
  - [x] `Diagnostic { range, severity, code, message }`
  - [x] `Hover { contents: MarkupContent }`
  - [x] `MarkupContent { kind, value }`
- [x] `src/lsp/document_store.rs` を作成（`DocumentStore` スタブ）
- [x] `src/lsp/hover.rs` を作成（`None` を返すスタブ）
- [x] `src/lsp/diagnostics.rs` を作成（`vec![]` を返すスタブ）
- [x] `src/main.rs` に `mod lsp;` を追加
- [x] `src/main.rs` に `"lsp"` コマンドを追加（`lsp::run_lsp_server()` 呼び出し）
- [x] `Cargo.toml` に `serde = { version = "1", features = ["derive"] }` を追加
- [x] `cargo build` が通ること

### 1-2: JSON-RPC メッセージ読み書き

- [x] `read_message(reader: &mut impl BufRead) -> Option<RpcRequest>` を実装
  - [x] `Content-Length: N\r\n\r\n` ヘッダを読む
  - [x] N バイト読んで `serde_json::from_slice` でパース
  - [x] パース失敗時は `None` を返す（クラッシュしない）
- [x] `write_message(writer: &mut impl Write, msg: &serde_json::Value)` を実装
  - [x] `Content-Length: N\r\n\r\n<JSON>` 形式で出力

### 1-3: `Checker` に `type_at` を追加

- [x] `checker.rs` に `pub type_at: HashMap<Span, Type>` フィールドを追加
- [x] `Checker::new()` / `Checker::new_with_resolver()` で `type_at: HashMap::new()` を初期化
- [x] `check_expr` の `Expr::Ident` 分岐で `self.type_at.insert(span, ty.clone())` を呼ぶ
- [x] `check_expr` の `Expr::Call` 分岐で `self.type_at.insert(span, return_ty.clone())` を呼ぶ
- [x] 既存テストが全通過すること

### 1-4: `DocumentStore` の実装

- [x] `CheckedDoc { source, errors, type_at }` 構造体を定義
- [x] `DocumentStore::open_or_change(uri, source)` を実装
  - [x] `Parser::parse_str` でパース（失敗したら空の doc を保存）
  - [x] `Checker::new()` で型チェック
  - [x] `checker.type_at` を `CheckedDoc` に保存
- [x] `DocumentStore::get(uri) -> Option<&CheckedDoc>` を実装

### 1-5: `diagnostics.rs` の実装

- [x] `errors_to_diagnostics(errors: &[TypeError]) -> Vec<Diagnostic>` を実装
  - [x] `span.line` を 0-origin に変換（`saturating_sub(1)`）
  - [x] `span.col` を 0-origin に変換
  - [x] `severity: 1` (Error)
- [x] テスト: 型エラーのある source を診断して `code = "E001"` 等が返ること

### 1-6: `hover.rs` の実装

- [x] `handle_hover(store, uri, pos) -> Option<Hover>` を実装
  - [x] `store.get(uri)` でドキュメントを取得
  - [x] `type_at` から `pos` を含む最小 Span を探す
  - [x] 型を Markdown コードブロックにして `Hover` を返す
- [x] `span_contains(span, pos) -> bool` を実装（Span の 1-origin と Position の 0-origin の変換に注意）
- [x] テスト: `x: Int` の位置で hover → `"Int"` が含まれること

### 1-7: LspServer ハンドラの実装（mod.rs）

- [x] `struct LspServer { store: DocumentStore, writer: Stdout }` を定義
- [x] `initialize` → capabilities JSON を返す
  ```json
  { "textDocumentSync": 1, "hoverProvider": true, "definitionProvider": false }
  ```
- [x] `initialized` → 何もしない（notification）
- [x] `textDocument/didOpen` → `store.open_or_change` → `publishDiagnostics`
- [x] `textDocument/didChange` → `store.open_or_change` → `publishDiagnostics`
- [x] `textDocument/hover` → `handle_hover` → レスポンス送信
- [x] `textDocument/definition` → `null` を返す（スタブ）
- [x] `shutdown` → `null` を返す
- [x] `exit` → `process::exit(0)`
- [x] 未知のメソッド → 無視（クラッシュしない）

### 1-8: `run_lsp_server` の完成

- [x] stdin から `read_message` でループ
- [x] `LspServer::handle` にディスパッチ
- [x] stdout に `write_message` で送信

### 1-9: LSP テスト

- [x] `document_store`: 正しい source → errors が空
- [x] `document_store`: 型エラーのある source → errors に TypeError が含まれる
- [x] `diagnostics`: `errors_to_diagnostics` が正しい Range を生成する
- [x] `hover`: 識別子位置で型が返る
- [x] `cargo test` が全通過すること

---

## Phase 2: WASM String 戻り値

### Done definition

`fn greet(name: String) -> String { name }` を含むプログラムが
`wasm_codegen_program` を通り、wasmtime で実行できること。

### 2-1: `WasmLocal` 型の追加

- [x] `wasm_codegen.rs` に `enum WasmLocal { Single(u32), StringPtrLen(u32, u32) }` を追加
- [x] `build_wasm_function` 内の `slot_map` の型を `HashMap<u16, WasmLocal>` に変更

### 2-2: `favnir_type_to_wasm_results` の変更

- [x] `Type::String` → `Ok(vec![ValType::I32, ValType::I32])` （W001 解除）
- [x] 旧 W001 テスト `wasm_string_return_is_w001` を削除または更新

### 2-3: パラメータの `slot_map` 構築変更

- [x] `Type::String` パラメータ → `WasmLocal::StringPtrLen(next, next+1)`, `next += 2`
- [x] その他 → `WasmLocal::Single(next)`, `next += 1`

### 2-4: ローカル変数宣言の変更

- [x] `Type::String` ローカル → `(1, I32)` × 2 を `local_decls` に追加、`next += 2`
- [x] `slot_map` に `WasmLocal::StringPtrLen(next, next+1)` を登録

### 2-5: `emit_expr` の `IRExpr::Local` 変更

- [x] `WasmLocal::Single(idx)` → `LocalGet(idx)`
- [x] `WasmLocal::StringPtrLen(ptr, len)` → `LocalGet(ptr)` + `LocalGet(len)`

### 2-6: `emit_stmt` の変更

- [x] `IRStmt::Bind` で `WasmLocal::StringPtrLen(ptr, len)`:
  - [x] `LocalSet(len)` → `LocalSet(ptr)` の順で pop
- [x] `IRStmt::Expr` の Drop: `favnir_type_to_wasm_results` の返り値数分 Drop を emit

### 2-7: `block_type_for` の変更

- [x] String result の `if/else` → `UnsupportedExpr("multi-value block ...")` を返す (W002)

### 2-8: テスト

- [x] `wasm_string_return_greet` — `fn greet() -> String { "hi" }` が codegen を通る
- [x] `wasm_string_identity` — `fn id(s: String) -> String { s }` が codegen を通る
- [x] `wasm_string_bind_and_print` — String ローカルに bind して `IO.println` できる
- [x] `wasm_hello_string_exec` — wasmtime で実際に実行して出力が正しい
- [x] `cargo test` が全通過すること

---

## Phase 3: WASM クロージャ

### Done definition

```fav
public fn main() -> Unit !Io {
    bind f <- |x| x + 1
    IO.println_int(f(5))
}
```

これが wasmtime で実行でき `6` が出力されること。

### 3-1: GlobalSection: heap_ptr の追加

- [x] `wasm_codegen.rs` に GlobalSection を追加
  - [x] `$heap_ptr: (mut i32) = 65536` (1ページ目終端)
- [x] Module に GlobalSection を追加（TypeSection の前）

### 3-2: bump_alloc 関数の生成

- [x] `$bump_alloc(size: i32) -> i32` の WASM 命令列を生成
  - [x] `global.get $heap_ptr` (戻り値を先にスタックに)
  - [x] `global.get $heap_ptr; local.get $size; i32.add; global.set $heap_ptr`
  - [x] `end`
- [x] FunctionSection + CodeSection に追加
- [x] TypeSection に `(i32) -> i32` 型を登録
- [x] `ctx` に `bump_alloc_fn_idx: u32` を追加

### 3-3: `WasmLocal` の拡張

- [x] `WasmLocal::FnTableEnv { fn_idx_local, env_ptr_local, wrapper_type_idx }` を追加

### 3-4: 合成関数の生成

- [x] `IRExpr::Closure(fn_global_idx, captures, _)` の事前スキャンで合成関数を生成
  - [x] シグネチャ: `(env_ptr: i32, ...original_params) -> return_ty`
  - [x] 本体: `env_ptr + offset` から各 capture を load して元の関数本体を emit
- [x] 合成関数を FunctionSection + CodeSection に追加（import の後ろ）
- [x] table_idx を確定して `ctx` に記録

### 3-5: TableSection + ElementSection の追加

- [x] TableSection: `funcref`, count = クロージャ合成関数の数
- [x] ElementSection: offset 0 から合成関数の WASM 関数インデックスを登録
- [x] Module に TableSection + ElementSection を追加

### 3-6: `emit_expr`: `IRExpr::Closure` の実装

- [x] env サイズを計算
- [x] `I32Const(env_size); Call(bump_alloc_idx)` で env_ptr を取得
- [x] env_ptr を LocalSet(env_ptr_local) に保存
- [x] captures を env_ptr + offset に i64.store
- [x] `I32Const(table_idx)` を LocalSet(fn_idx_local) に保存

### 3-7: `emit_expr`: クロージャ呼び出しの実装

- [x] `IRExpr::Call(callee, args)` で `callee` がクロージャ型 local の場合:
  - [x] `slot_map[callee_slot]` が `WasmLocal::FnTableEnv` なら
  - [x] `LocalGet(env_ptr_local)` を emit
  - [x] args を emit
  - [x] `LocalGet(fn_idx_local)` を emit
  - [x] `CallIndirect { type_index, table_index: 0 }` を emit

### 3-8: テスト

- [x] `wasm_closure_codegen_produces_valid_module` — `|x| x + 1` を作って wasmtime で検証
- [x] `wasm_closure_capture_produces_valid_module` — キャプチャ付きクロージャが valid module
- [x] `wasm_closure_exec_returns_correct_result` — wasmtime で実行して `f(5) = 6` を確認
- [x] `cargo test` が全通過すること

---

## Phase 4: rune 依存管理

### Done definition

`fav.toml` に `[dependencies] csv_helper = { path = "../csv_helper" }` があり、
`fav install` が成功して `fav.lock` が生成されること。

### 4-1: `DependencySpec` の追加（toml.rs）

- [x] `pub enum DependencySpec { Path { name, path }, Registry { name, registry, version } }` を追加
- [x] `FavToml` に `pub dependencies: Vec<DependencySpec>` を追加
- [x] `FavToml` の TOML ミニパーサーを拡張して `[dependencies]` セクションを読む
  - [x] `name = { path = "..." }` → `Path`
  - [x] `name = { version = "...", registry = "local" }` → `Registry`
- [x] 既存 toml テストが全通過すること

### 4-2: `lock.rs` の作成

- [x] `src/lock.rs` を新規作成
  - [x] `struct LockedPackage { name, version, resolved_path }` を定義
  - [x] `struct LockFile { packages: Vec<LockedPackage> }` を定義
  - [x] `LockFile::load(path: &Path) -> Self` を実装（TOML ミニパーサー）
  - [x] `LockFile::save(path: &Path) -> Result<(), io::Error>` を実装（TOML 書き出し）
- [x] `src/main.rs` に `mod lock;` を追加

### 4-3: `cmd_install` の実装（driver.rs）

- [x] `pub fn cmd_install()` を追加
  - [x] `fav.toml` を読む（なければエラー）
  - [x] `DependencySpec::Path { path }`: 相対パスを絶対パスに変換して存在確認
  - [x] `DependencySpec::Registry { registry, version }`: ローカルレジストリディレクトリの存在確認
  - [x] `LockFile` を生成して `fav.lock` に書き出す

### 4-4: `cmd_publish` の実装（driver.rs）

- [x] `pub fn cmd_publish()` を追加
  - [x] `fav.toml` を読んで `name` と `version` を取得
  - [x] name/version 未設定ならエラー
  - [x] ローカルレジストリへの案内を出力（スタブ）

### 4-5: `main.rs` にコマンド追加

- [x] `"install"` → `cmd_install()`
- [x] `"publish"` → `cmd_publish()`
- [x] HELP テキストに追加

### 4-6: テスト

- [x] `lock::tests::lock_file_round_trip` — LockFile を TOML に書き出して再パース
- [x] `lock::tests::lock_file_empty_returns_empty`
- [x] `lock::tests::lock_file_get_finds_package`
- [x] `toml::tests::test_path_dependency_parsed`
- [x] `toml::tests::test_registry_dependency_parsed`
- [x] `toml::tests::test_multiple_dependencies_parsed`
- [x] `cargo test` が全通過すること

---

## Phase 5: ドキュメント整備

### 5-1: examples の追加

- [x] `examples/string_wasm.fav` を作成
- [x] `examples/closures_wasm.fav` を作成
- [x] 上記 2 例が `fav build --target wasm` + `fav exec` で動くこと

### 5-2: langspec.md の polish

- [x] Phase 0 の骨格に最低限の内容を追加
  - [x] **1. 基本型**: 型一覧 + 1行説明 + コード例
  - [x] **3. effect system**: 種類一覧 + 使い方例
  - [x] **8. エラーコード一覧**: E001–E040, W001–W004 を表形式で記載
  - [x] **9. 後方互換ポリシー**: v1.0.0 以降の保証を文章で記載
- [x] 他の章は箇条書きで概要のみでよい

### 5-3: README.md の整備

- [x] HELP テキストに install/publish/lsp コマンドを追記

### 5-4: RELEASE_NOTES.md の作成

- [x] `versions/v1.0.0/RELEASE_NOTES.md` を作成
  - [x] v0.x からの新機能一覧
  - [x] WASM の変更点（String 戻り値・クロージャ）
  - [x] LSP の設定例
  - [x] 既知の制限事項
  - [x] v1.1.0 の予告（List/Map WASM、HTTP registry、LSP completion）

### 5-5: 全体確認

- [x] `cargo build` で警告ゼロ
- [x] `cargo test` 全通過（321 テスト）
- [x] `fav lsp` が起動して `initialize` に応答する
- [x] `examples/string_wasm.fav` が WASM でビルド・実行できる
- [x] `examples/closures_wasm.fav` が WASM でビルド・実行できる
- [x] `fav install` が path 依存を解決して `fav.lock` を生成する
- [x] `fav publish` が fav.toml を検証して出力する
- [x] `Cargo.toml` バージョンが `"1.0.0"`

---

## 全体完了条件

- [x] `cargo build` で警告ゼロ
- [x] `cargo test` 全テスト通過
- [x] `fav lsp` が hover と diagnostics を返す
- [x] `fn greet(name: String) -> String { name }` が WASM でビルド・実行できる
- [x] `bind f <- |x| x + 1; f(5)` が WASM でビルド・実行できる
- [x] `fav install` がローカルパス依存を解決して `fav.lock` を生成する
- [x] `Cargo.toml` バージョンが `"1.0.0"`

---

## 先送り一覧（守る）

| 制約 | バージョン |
|---|---|
| LSP: completion / rename / references | v1.1.0 |
| LSP: `--port` TCP モード | v1.1.0 |
| WASM: String の if/else result | v1.1.0 |
| WASM: `List<T>` / `Map<V>` | v1.1.0 |
| WASM: 高階ビルトイン (`List.map(f, xs)`) | v1.1.0 |
| WASM: `trf` / `flw` | v1.1.0 |
| WASM: クロージャの再帰 | v1.1.0 |
| rune: HTTP レジストリ | v1.1.0 |
| rune: `fav.lock` hash 検証 | v1.1.0 |
| `examples/multi_rune/` | オプション |
| セルフホスティング | v2.0.0 |
