# Favnir v1.0.0 タスク一覧 — 安定版

作成日: 2026-04-30（Codex レビュー反映）

> [ ] 未完了 / [x] 完了
>
> **ゴール**: LSP hover/diagnostics・WASM String/closure・rune install の揃った安定版
> **前提**: v0.9.0 完了（289 テスト通過）
>
> **スコープ管理が最優先。Done definition を超えない。**

---

## Phase 0: バージョン更新 + 仕様書骨格

### 0-1: バージョン更新

- [ ] `Cargo.toml` の `version` を `"1.0.0"` に更新
- [ ] `main.rs` の HELP テキストを `v1.0.0` に更新
- [ ] `cargo build` が通ること

### 0-2: langspec.md 骨格作成

- [ ] `versions/v1.0.0/langspec.md` を作成（章立てのみ）
  - [ ] 1. 基本型
  - [ ] 2. 関数・trf・flw
  - [ ] 3. effect system
  - [ ] 4. パターンマッチング
  - [ ] 5. モジュールシステム
  - [ ] 6. 標準ライブラリ
  - [ ] 7. CLI リファレンス
  - [ ] 8. エラーコード一覧
  - [ ] 9. 後方互換ポリシー
- [ ] 後方互換ポリシーの文章だけ書く（「v1.0.0 以降は破壊的変更なし」）

---

## Phase 1: LSP 最小実装

### 1-1: scaffold

- [ ] `src/lsp/` ディレクトリを作成
- [ ] `src/lsp/mod.rs` を作成（`pub fn run_lsp_server()` スタブ）
- [ ] `src/lsp/protocol.rs` を作成
  - [ ] `RpcRequest { id, method, params }` (serde Deserialize)
  - [ ] `RpcResponse { jsonrpc, id, result }` (serde Serialize)
  - [ ] `Position { line: u32, character: u32 }`
  - [ ] `Range { start, end }`
  - [ ] `Diagnostic { range, severity, code, message }`
  - [ ] `Hover { contents: MarkupContent }`
  - [ ] `MarkupContent { kind, value }`
- [ ] `src/lsp/document_store.rs` を作成（`DocumentStore` スタブ）
- [ ] `src/lsp/hover.rs` を作成（`None` を返すスタブ）
- [ ] `src/lsp/diagnostics.rs` を作成（`vec![]` を返すスタブ）
- [ ] `src/main.rs` に `mod lsp;` を追加
- [ ] `src/main.rs` に `"lsp"` コマンドを追加（`lsp::run_lsp_server()` 呼び出し）
- [ ] `Cargo.toml` に `serde = { version = "1", features = ["derive"] }` を追加
- [ ] `cargo build` が通ること

### 1-2: JSON-RPC メッセージ読み書き

- [ ] `read_message(reader: &mut impl BufRead) -> Option<RpcRequest>` を実装
  - [ ] `Content-Length: N\r\n\r\n` ヘッダを読む
  - [ ] N バイト読んで `serde_json::from_slice` でパース
  - [ ] パース失敗時は `None` を返す（クラッシュしない）
- [ ] `write_message(writer: &mut impl Write, msg: &serde_json::Value)` を実装
  - [ ] `Content-Length: N\r\n\r\n<JSON>` 形式で出力

### 1-3: `Checker` に `type_at` を追加

- [ ] `checker.rs` に `pub type_at: HashMap<Span, Type>` フィールドを追加
- [ ] `Checker::new()` / `Checker::new_with_resolver()` で `type_at: HashMap::new()` を初期化
- [ ] `check_expr` の `Expr::Ident` 分岐で `self.type_at.insert(span, ty.clone())` を呼ぶ
- [ ] `check_expr` の `Expr::Call` 分岐で `self.type_at.insert(span, return_ty.clone())` を呼ぶ
- [ ] 既存テストが全通過すること

### 1-4: `DocumentStore` の実装

- [ ] `CheckedDoc { source, errors, type_at }` 構造体を定義
- [ ] `DocumentStore::open_or_change(uri, source)` を実装
  - [ ] `Parser::parse_str` でパース（失敗したら空の doc を保存）
  - [ ] `Checker::new()` で型チェック
  - [ ] `checker.type_at` を `CheckedDoc` に保存
- [ ] `DocumentStore::get(uri) -> Option<&CheckedDoc>` を実装

### 1-5: `diagnostics.rs` の実装

- [ ] `errors_to_diagnostics(errors: &[TypeError]) -> Vec<Diagnostic>` を実装
  - [ ] `span.line` を 0-origin に変換（`saturating_sub(1)`）
  - [ ] `span.col` を 0-origin に変換
  - [ ] `severity: 1` (Error)
- [ ] テスト: 型エラーのある source を診断して `code = "E001"` 等が返ること

### 1-6: `hover.rs` の実装

- [ ] `handle_hover(store, uri, pos) -> Option<Hover>` を実装
  - [ ] `store.get(uri)` でドキュメントを取得
  - [ ] `type_at` から `pos` を含む最小 Span を探す
  - [ ] 型を Markdown コードブロックにして `Hover` を返す
- [ ] `span_contains(span, pos) -> bool` を実装（Span の 1-origin と Position の 0-origin の変換に注意）
- [ ] テスト: `x: Int` の位置で hover → `"Int"` が含まれること

### 1-7: LspServer ハンドラの実装（mod.rs）

- [ ] `struct LspServer { store: DocumentStore, writer: Stdout }` を定義
- [ ] `initialize` → capabilities JSON を返す
  ```json
  { "textDocumentSync": 1, "hoverProvider": true, "definitionProvider": false }
  ```
- [ ] `initialized` → 何もしない（notification）
- [ ] `textDocument/didOpen` → `store.open_or_change` → `publishDiagnostics`
- [ ] `textDocument/didChange` → `store.open_or_change` → `publishDiagnostics`
- [ ] `textDocument/hover` → `handle_hover` → レスポンス送信
- [ ] `textDocument/definition` → `null` を返す（スタブ）
- [ ] `shutdown` → `null` を返す
- [ ] `exit` → `process::exit(0)`
- [ ] 未知のメソッド → 無視（クラッシュしない）

### 1-8: `run_lsp_server` の完成

- [ ] stdin から `read_message` でループ
- [ ] `LspServer::handle` にディスパッチ
- [ ] stdout に `write_message` で送信

### 1-9: LSP テスト

- [ ] `document_store`: 正しい source → errors が空
- [ ] `document_store`: 型エラーのある source → errors に TypeError が含まれる
- [ ] `diagnostics`: `errors_to_diagnostics` が正しい Range を生成する
- [ ] `hover`: 識別子位置で型が返る
- [ ] `cargo test` が全通過すること

---

## Phase 2: WASM String 戻り値

### Done definition

`fn greet(name: String) -> String { name }` を含むプログラムが
`wasm_codegen_program` を通り、wasmtime で実行できること。

### 2-1: `WasmLocal` 型の追加

- [ ] `wasm_codegen.rs` に `enum WasmLocal { Single(u32), StringPtrLen(u32, u32) }` を追加
- [ ] `build_wasm_function` 内の `slot_map` の型を `HashMap<u16, WasmLocal>` に変更

### 2-2: `favnir_type_to_wasm_results` の変更

- [ ] `Type::String` → `Ok(vec![ValType::I32, ValType::I32])` （W001 解除）
- [ ] 旧 W001 テスト `wasm_string_return_is_w001` を削除または更新

### 2-3: パラメータの `slot_map` 構築変更

- [ ] `Type::String` パラメータ → `WasmLocal::StringPtrLen(next, next+1)`, `next += 2`
- [ ] その他 → `WasmLocal::Single(next)`, `next += 1`

### 2-4: ローカル変数宣言の変更

- [ ] `Type::String` ローカル → `(1, I32)` × 2 を `local_decls` に追加、`next += 2`
- [ ] `slot_map` に `WasmLocal::StringPtrLen(next, next+1)` を登録

### 2-5: `emit_expr` の `IRExpr::Local` 変更

- [ ] `WasmLocal::Single(idx)` → `LocalGet(idx)`
- [ ] `WasmLocal::StringPtrLen(ptr, len)` → `LocalGet(ptr)` + `LocalGet(len)`

### 2-6: `emit_stmt` の変更

- [ ] `IRStmt::Bind` で `WasmLocal::StringPtrLen(ptr, len)`:
  - [ ] `LocalSet(len)` → `LocalSet(ptr)` の順で pop
- [ ] `IRStmt::Expr` の Drop: `favnir_type_to_wasm_results` の返り値数分 Drop を emit

### 2-7: `block_type_for` の変更

- [ ] String result の `if/else` → `UnsupportedExpr("multi-value block ...")` を返す (W002)

### 2-8: テスト

- [ ] `wasm_string_return_greet` — `fn greet() -> String { "hi" }` が codegen を通る
- [ ] `wasm_string_identity` — `fn id(s: String) -> String { s }` が codegen を通る
- [ ] `wasm_string_bind_and_print` — String ローカルに bind して `IO.println` できる
- [ ] `wasm_hello_string_exec` — wasmtime で実際に実行して出力が正しい
- [ ] `cargo test` が全通過すること

---

## Phase 3: WASM クロージャ

### Done definition

```fav
public fn main() -> Unit !Io {
    let f <- |x| x + 1
    IO.println_int(f(5))
}
```

これが wasmtime で実行でき `6` が出力されること。

### 3-1: GlobalSection: heap_ptr の追加

- [ ] `wasm_codegen.rs` に GlobalSection を追加
  - [ ] `$heap_ptr: (mut i32) = 65536` (1ページ目終端)
- [ ] Module に GlobalSection を追加（TypeSection の前）

### 3-2: bump_alloc 関数の生成

- [ ] `$bump_alloc(size: i32) -> i32` の WASM 命令列を生成
  - [ ] `global.get $heap_ptr` (戻り値を先にスタックに)
  - [ ] `global.get $heap_ptr; local.get $size; i32.add; global.set $heap_ptr`
  - [ ] `end`
- [ ] FunctionSection + CodeSection に追加
- [ ] TypeSection に `(i32) -> i32` 型を登録
- [ ] `ctx` に `bump_alloc_fn_idx: u32` を追加

### 3-3: `WasmLocal` の拡張

- [ ] `WasmLocal::FnTableEnv(fn_idx_local: u32, env_ptr_local: u32)` を追加

### 3-4: 合成関数の生成

- [ ] `IRExpr::Closure(fn_global_idx, captures, _)` の事前スキャンで合成関数を生成
  - [ ] シグネチャ: `(env_ptr: i32, ...original_params) -> return_ty`
  - [ ] 本体: `env_ptr + offset` から各 capture を load して元の関数本体を emit
- [ ] 合成関数を FunctionSection + CodeSection に追加（import の後ろ）
- [ ] table_idx を確定して `ctx` に記録

### 3-5: TableSection + ElementSection の追加

- [ ] TableSection: `funcref`, count = クロージャ合成関数の数
- [ ] ElementSection: offset 0 から合成関数の WASM 関数インデックスを登録
- [ ] Module に TableSection + ElementSection を追加

### 3-6: `emit_expr`: `IRExpr::Closure` の実装

- [ ] env サイズを計算
- [ ] `I32Const(env_size); Call(bump_alloc_idx)` で env_ptr を取得
- [ ] env_ptr を tmp local に保存（`LocalTee`）
- [ ] captures を env_ptr + offset に store
- [ ] `I32Const(table_idx)` + `LocalGet(env_ptr_local)` をスタックに push

### 3-7: `emit_expr`: クロージャ呼び出しの実装

- [ ] `IRExpr::Call(callee, args)` で `callee` がクロージャ型 local の場合:
  - [ ] `slot_map[callee_slot]` が `WasmLocal::FnTableEnv(fn_idx_local, env_ptr_local)` なら
  - [ ] `LocalGet(env_ptr_local)` を emit
  - [ ] args を emit
  - [ ] `LocalGet(fn_idx_local)` を emit
  - [ ] `CallIndirect { type_index, table_index: 0 }` を emit

### 3-8: テスト

- [ ] `wasm_closure_no_capture` — `|x| x + 1` を作って `f(5)` = 6
- [ ] `wasm_closure_capture_int` — `let n <- 10; let f <- |x| x + n; f(5)` = 15
- [ ] `wasm_closure_exec` — wasmtime で実際に実行して出力が正しい
- [ ] `cargo test` が全通過すること

---

## Phase 4: rune 依存管理

### Done definition

`fav.toml` に `[dependencies] csv_helper = { path = "../csv_helper" }` があり、
`fav install` が成功して `fav.lock` が生成されること。

### 4-1: `DependencySpec` の追加（toml.rs）

- [ ] `pub enum DependencySpec { Path { path: String }, Local { version: String } }` を追加
- [ ] `FavToml` に `pub dependencies: HashMap<String, DependencySpec>` を追加
- [ ] `FavToml` の TOML ミニパーサーを拡張して `[dependencies]` セクションを読む
  - [ ] `name = { path = "..." }` → `Path`
  - [ ] `name = { version = "...", registry = "local" }` → `Local`
- [ ] 既存 toml テストが全通過すること

### 4-2: `lock.rs` の作成

- [ ] `src/lock.rs` を新規作成
  - [ ] `struct LockedPackage { name, version, source }` を定義
  - [ ] `struct LockFile { packages: Vec<LockedPackage> }` を定義
  - [ ] `LockFile::load(path: &Path) -> Option<Self>` を実装（TOML ミニパーサー）
  - [ ] `LockFile::save(path: &Path) -> Result<(), String>` を実装（TOML 書き出し）
- [ ] `src/main.rs` に `mod lock;` を追加

### 4-3: `cmd_install` の実装（driver.rs）

- [ ] `pub fn cmd_install(rune: Option<&str>)` を追加
  - [ ] `fav.toml` を読む（なければエラー）
  - [ ] `DependencySpec::Path { path }`: 相対パスを絶対パスに変換して存在確認
  - [ ] `DependencySpec::Local { version }`: `~/.fav/registry/<name>/<version>/` の存在確認
  - [ ] `LockFile` を生成して `fav.lock` に書き出す
  - [ ] `"installed N dependencies"` を出力

### 4-4: `cmd_publish` の実装（driver.rs）

- [ ] `pub fn cmd_publish()` を追加
  - [ ] `fav.toml` を読んで `name` と `version` を取得
  - [ ] `~/.fav/registry/<name>/<version>/` を作成
  - [ ] `src/` 以下を再帰コピー
  - [ ] `"published <name>@<version> to local registry"` を出力

### 4-5: `main.rs` にコマンド追加

- [ ] `"install"` → `cmd_install(args.get(2).map(|s| s.as_str()))`
- [ ] `"publish"` → `cmd_publish()`
- [ ] HELP テキストに追加:
  ```
      install [rune]  Resolve and install dependencies from fav.toml.
      publish         Publish current rune to local registry (~/.fav/registry/).
  ```

### 4-6: テスト

- [ ] `install_path_dep_resolves_and_writes_lock` — tempdir に rune を置いて install → fav.lock 生成
- [ ] `publish_creates_registry_dir` — publish → `~/.fav/registry/` にディレクトリが作られる
- [ ] `cargo test` が全通過すること

---

## Phase 5: ドキュメント整備

### 5-1: examples の追加

- [ ] `examples/string_wasm.fav` を作成
  ```fav
  public fn greet(name: String) -> String {
      name
  }
  public fn main() -> Unit !Io {
      IO.println(greet("Favnir"))
  }
  ```
- [ ] `examples/closures_wasm.fav` を作成
  ```fav
  public fn main() -> Unit !Io {
      let f <- |x| x + 1
      IO.println_int(f(5))
  }
  ```
- [ ] 上記 2 例が `fav build --target wasm` + `fav exec` で動くこと

### 5-2: langspec.md の polish

- [ ] Phase 0 の骨格に最低限の内容を追加
  - [ ] **1. 基本型**: 型一覧 + 1行説明 + コード例
  - [ ] **3. effect system**: 種類一覧 + 使い方例
  - [ ] **8. エラーコード一覧**: E001–E040, W001–W004 を表形式で記載
  - [ ] **9. 後方互換ポリシー**: v1.0.0 以降の保証を文章で記載
- [ ] 他の章は箇条書きで概要のみでよい

### 5-3: README.md の整備

- [ ] 以下の構成で整理する（各セクション 10 行以内）
  - [ ] 概要（3 行）
  - [ ] Install (`cargo install fav`)
  - [ ] Quick Start (hello.fav → run → build → exec)
  - [ ] CLI Reference (全コマンド一覧)
  - [ ] WASM (`fav build --target wasm` + `fav exec`)
  - [ ] LSP（VS Code 設定例）
  - [ ] rune dependencies (`fav install` / `fav publish`)

### 5-4: RELEASE_NOTES.md の作成

- [ ] `versions/v1.0.0/RELEASE_NOTES.md` を作成
  - [ ] v0.x からの新機能一覧
  - [ ] WASM の変更点（String 戻り値・クロージャ）
  - [ ] LSP の設定例
  - [ ] 既知の制限事項
  - [ ] v1.1.0 の予告（List/Map WASM、HTTP registry、LSP completion）

### 5-5: 全体確認

- [ ] `cargo build` で警告ゼロ
- [ ] `cargo test` 全通過（目標 330+）
- [ ] `fav lsp` が起動して `initialize` に応答する
- [ ] `examples/string_wasm.fav` が WASM でビルド・実行できる
- [ ] `examples/closures_wasm.fav` が WASM でビルド・実行できる
- [ ] `fav install` が path 依存を解決して `fav.lock` を生成する
- [ ] `fav publish` が `~/.fav/registry/` に登録する
- [ ] `Cargo.toml` バージョンが `"1.0.0"`

---

## 全体完了条件

- [ ] `cargo build` で警告ゼロ
- [ ] `cargo test` 全テスト通過
- [ ] `fav lsp` が hover と diagnostics を返す
- [ ] `fn greet(name: String) -> String { name }` が WASM でビルド・実行できる
- [ ] `let f <- |x| x + 1; f(5)` が WASM でビルド・実行できる
- [ ] `fav install` がローカルパス依存を解決して `fav.lock` を生成する
- [ ] `Cargo.toml` バージョンが `"1.0.0"`

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
