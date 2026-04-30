# Favnir v1.0.0 タスク一覧 — 安定版

作成日: 2026-04-30

> [ ] 未完了 / [x] 完了
>
> **ゴール**: LSP・WASM String/クロージャ・rune 依存管理・ドキュメントを揃えた安定版
> **前提**: v0.9.0 完了（289 テスト通過）

---

## Phase 0: 言語仕様書

### 0-1: langspec.md の作成

- [ ] `versions/v1.0.0/langspec.md` を作成
  - [ ] 基本型一覧 (Bool, Int, Float, String, Unit, List<T>, Map<K,V>)
  - [ ] 複合型 (Option<T>, Result<T,E>, T?, T!)
  - [ ] 関数構文 (`fn`, パラメータ, 戻り値型, effects)
  - [ ] `trf` / `flw` 構文と意味論
  - [ ] `bind` / `chain` / `yield` / `collect` の意味論
  - [ ] effect system: Pure, Io, Db, Network, File, Trace, Emit<T>
  - [ ] パターンマッチング (リテラル, 変数, variant, record, ガード)
  - [ ] モジュールシステム (namespace, use, pub)
  - [ ] エラーコード一覧 E001–E040, W001–W004
  - [ ] 後方互換ポリシー (v1.0.0 以降は破壊的変更なし)

### 0-2: Cargo.toml バージョン更新

- [ ] `version = "1.0.0"` に更新
- [ ] `main.rs` の HELP テキストを `v1.0.0` に更新

---

## Phase 1: LSP 最小実装

### 1-1: ディレクトリと scaffold

- [ ] `src/lsp/` ディレクトリを作成
- [ ] `src/lsp/mod.rs` を作成
  - [ ] `pub fn run_lsp_server(port: Option<u16>)` — メインループのスタブ
- [ ] `src/lsp/protocol.rs` を作成
  - [ ] `struct RpcRequest { id, method, params }` (serde Deserialize)
  - [ ] `struct RpcResponse { jsonrpc, id, result }` (serde Serialize)
  - [ ] `struct Position { line: u32, character: u32 }`
  - [ ] `struct Range { start: Position, end: Position }`
  - [ ] `struct Diagnostic { range, severity, code, message }`
  - [ ] `struct Hover { contents: MarkupContent }`
  - [ ] `struct MarkupContent { kind: String, value: String }`
- [ ] `src/lsp/document_store.rs` を作成
  - [ ] `struct CheckedDoc { source, program, errors, type_at, effect_at }`
  - [ ] `struct DocumentStore { docs: HashMap<String, CheckedDoc> }`
  - [ ] `fn open(&mut self, uri, source)`
  - [ ] `fn change(&mut self, uri, source)` — open と同じ処理
  - [ ] `fn get(&self, uri) -> Option<&CheckedDoc>`
- [ ] `src/lsp/hover.rs` を作成 — スタブ (`None` を返す)
- [ ] `src/lsp/diagnostics.rs` を作成 — スタブ (`vec![]` を返す)
- [ ] `src/main.rs` に `mod lsp;` を追加
- [ ] `src/main.rs` に `"lsp"` コマンドを追加 (`cmd_lsp()` のスタブ呼び出し)
- [ ] `Cargo.toml` に `serde = { version = "1", features = ["derive"] }` を追加
- [ ] `cargo build` が通ること

### 1-2: JSON-RPC メッセージ読み書き

- [ ] `fn read_message(reader: &mut impl BufRead) -> Option<RpcRequest>` を実装
  - [ ] `Content-Length: N\r\n\r\n` ヘッダを読む
  - [ ] N バイト読んで `serde_json::from_slice` でパース
- [ ] `fn write_message(writer: &mut impl Write, response: &RpcResponse)` を実装
  - [ ] `Content-Length: N\r\n\r\n<JSON>` 形式で出力
- [ ] `run_lsp_server` のループを実装
  - [ ] stdin から読んで `handle_message` でディスパッチ
  - [ ] stdout に書き出し

### 1-3: `initialize` ハンドラ

- [ ] `initialize` メソッドのレスポンスを実装
  ```json
  {
    "capabilities": {
      "textDocumentSync": 1,
      "hoverProvider": true,
      "definitionProvider": false
    }
  }
  ```
- [ ] `initialized` notification を受け取ったとき何もしない (OK)
- [ ] `cargo test` が通ること

### 1-4: Checker に `type_at` マップを追加

- [ ] `Checker` に `pub type_at: HashMap<Span, Type>` フィールドを追加
- [ ] `check_expr` の `Expr::Ident` 分岐で `self.type_at.insert(span, ty.clone())` を呼ぶ
- [ ] `check_expr` の `Expr::Call` 分岐で `self.type_at.insert(span, return_ty.clone())` を呼ぶ
- [ ] `check_expr` の `Expr::BinOp` 分岐で `self.type_at.insert(span, ty.clone())` を呼ぶ
- [ ] `check_program` / `check_with_self` の戻り値に `type_at` を含める、
      または `Checker` インスタンスを返せるようにする
- [ ] 既存テストが全通過すること

### 1-5: `textDocument/didOpen` + diagnostics

- [ ] `document_store.rs` の `open` を実装
  - [ ] `Parser::parse_str` でパース
  - [ ] `Checker::check_program` で型チェック
  - [ ] `Checker.type_at` をキャプチャして `CheckedDoc` に保存
  - [ ] `errors` に TypeError 一覧を保存
- [ ] `diagnostics.rs` の `errors_to_diagnostics` を実装
  - [ ] `TypeError.span` → LSP `Range` 変換 (line は 0-origin)
  - [ ] `severity: 1` (Error)
- [ ] `textDocument/didOpen` ハンドラを実装
  - [ ] `DocumentStore::open` を呼ぶ
  - [ ] `textDocument/publishDiagnostics` notification を送信
- [ ] `textDocument/didChange` ハンドラを実装 (`didOpen` と同じ処理)
- [ ] テスト: 型エラーのある source を open して diagnostics が返ること

### 1-6: `textDocument/hover` ハンドラ

- [ ] `hover.rs` の `handle_hover` を実装
  - [ ] `params` から `textDocument.uri` と `position` を取得
  - [ ] `DocumentStore::get` でドキュメントを取得
  - [ ] `type_at` マップから `position` を含む最小 Span を探す
  - [ ] `Type` を `format_type(&ty)` で文字列化して Hover レスポンスを生成
- [ ] `textDocument/hover` ハンドラを実装
- [ ] テスト: 識別子の上で hover すると型が返ること

### 1-7: `textDocument/definition` スタブ

- [ ] `textDocument/definition` リクエストに対して `null` を返す (未実装)

### 1-8: `shutdown` + `exit`

- [ ] `shutdown` リクエストに対して `null` を返す
- [ ] `exit` notification でプロセス終了

### 1-9: LSP テスト

- [ ] `lsp::document_store` のユニットテスト
  - [ ] 正しい source を open → errors が空
  - [ ] 型エラーのある source を open → errors に TypeError が含まれる
- [ ] `lsp::diagnostics` のユニットテスト
  - [ ] `errors_to_diagnostics` が正しい Range を生成する
- [ ] `lsp::hover` のユニットテスト
  - [ ] `handle_hover` が正しい型文字列を返す
- [ ] `cargo test` が全通過すること

---

## Phase 2: WASM String 戻り値

### 2-1: `WasmLocal` 型の定義

- [ ] `wasm_codegen.rs` に `enum WasmLocal` を追加
  ```rust
  enum WasmLocal {
      Single(u32),
      StringPtrLen(u32, u32),
  }
  ```
- [ ] `slot_map` の型を `HashMap<u16, WasmLocal>` に変更
- [ ] `build_wasm_function` の内部を `WasmLocal` で書き直す

### 2-2: `favnir_type_to_wasm_results` の変更

- [ ] `Type::String` → `Ok(vec![ValType::I32, ValType::I32])` (W001 解除)
- [ ] `single_wasm_valtype` は内部ヘルパーとして維持するが、
      String の場合は `Err(W002("multi-value not supported in this context"))` を返す

### 2-3: パラメータの `slot_map` 構築変更

- [ ] `build_wasm_function` で `Type::String` パラメータは
      `WasmLocal::StringPtrLen(next, next+1)` として 2 ローカルを割り当てる

### 2-4: ローカル変数宣言の変更

- [ ] String 型ローカルは `(1, I32)` × 2 として `local_decls` に追加する

### 2-5: `emit_expr` の変更

- [ ] `IRExpr::Local` で `WasmLocal::StringPtrLen(ptr, len)` の場合は
      `LocalGet(ptr)` + `LocalGet(len)` を emit する

### 2-6: `emit_stmt` の変更

- [ ] `IRStmt::Bind` で `WasmLocal::StringPtrLen(ptr, len)` の場合は
      `LocalSet(len)` → `LocalSet(ptr)` の順で pop する
- [ ] `IRStmt::Expr` の Drop を結果型の wasm 値数分繰り返す

### 2-7: `block_type_for` の変更

- [ ] String 型の `if/else` block は W002 を返す（multi-value block は未対応）

### 2-8: テスト

- [ ] `wasm_string_return_greet` — `fn greet() -> String { "hello" }` が codegen を通る
- [ ] `wasm_string_param_and_return` — `fn identity(s: String) -> String { s }` が codegen を通る
- [ ] `wasm_string_bind_local` — String ローカルに bind してから IO.println できる
- [ ] `wasm_w001_lifted` — 旧 W001 テストを更新 (String return は通るように)
- [ ] `cargo test` が全通過すること

---

## Phase 3: WASM クロージャ

### 3-1: bump allocator の追加

- [ ] `wasm_codegen.rs` に bump allocator WASM 関数を生成する実装を追加
  - [ ] GlobalSection に `$heap_ptr: (mut i32) = 65536` を追加
  - [ ] `$bump_alloc(size: i32) -> i32` 関数を生成
    - [ ] `global.get $heap_ptr` → 戻り値 (ret addr)
    - [ ] `global.get $heap_ptr; local.get $size; i32.add; global.set $heap_ptr`
    - [ ] `end`
- [ ] Module に GlobalSection を追加
- [ ] `$bump_alloc` を FunctionSection + CodeSection に追加

### 3-2: TableSection + ElementSection の追加

- [ ] クロージャ由来の合成関数のインデックスを収集する
- [ ] `TableSection` を追加 (`funcref`, count = クロージャ数)
- [ ] `ElementSection` を追加 (offset 0 から合成関数を登録)
- [ ] Module に TableSection + ElementSection を追加

### 3-3: クロージャ合成関数の生成

- [ ] `IRExpr::Closure(fn_idx, captures, _)` を処理する実装
  - [ ] キャプチャ数分のパラメータ (`env_ptr: i32`) + 元のパラメータを持つ合成関数を生成
  - [ ] 環境から captures を load して元の関数本体を実行
- [ ] 合成関数を `ir.fns` に追加 (`is_closure: bool` フラグで識別)
- [ ] table インデックスを `ctx` に記録

### 3-4: `emit_expr` でのクロージャ emit

- [ ] `IRExpr::Closure(fn_idx, captures, _)` の emit を実装
  - [ ] `$bump_alloc(size)` を call して env_ptr を取得
  - [ ] captures を env_ptr + offset に store (i64.store / f64.store / i32.store)
  - [ ] `I32Const(table_idx)` → スタックに fn_table_idx
  - [ ] `env_ptr` → スタックに env_ptr
- [ ] クロージャ型ローカルを `WasmLocal::FnTableEnv(idx_local, env_local)` として表現

### 3-5: クロージャ呼び出しの emit

- [ ] `IRExpr::Call` でカリー先がクロージャ型ローカルの場合
  - [ ] `LocalGet(env_local)` を emit (env_ptr を引数として渡す)
  - [ ] 通常の args を emit
  - [ ] `LocalGet(fn_idx_local)` を emit
  - [ ] `CallIndirect { type_index, table_index: 0 }` を emit

### 3-6: テスト

- [ ] `wasm_closure_direct_call` — `|x| x + 1` を作って `f(5)` を呼ぶ
- [ ] `wasm_closure_capture_int` — `let n <- 10; let f <- |x| x + n; f(5)`
- [ ] `wasm_closure_int_result` — 実際に wasmtime で実行して 15 を返す
- [ ] `cargo test` が全通過すること

---

## Phase 4: rune 依存管理

### 4-1: `toml.rs` の拡張

- [ ] `DependencySpec` enum を追加
  ```rust
  pub enum DependencySpec {
      Path { path: String },
      Registry { version: String, registry: String },
  }
  ```
- [ ] `FavToml` に `pub dependencies: HashMap<String, DependencySpec>` を追加
- [ ] `FavToml` の TOML パーサーを拡張して `[dependencies]` セクションを読む
  - [ ] `csv_helper = { path = "../csv_helper" }` → `Path` 変換
  - [ ] `data_utils = { version = "0.2.0", registry = "local" }` → `Registry` 変換
- [ ] 既存テストが全通過すること

### 4-2: `fav.lock` の読み書き

- [ ] `src/lock.rs` を新規作成
  - [ ] `struct LockedPackage { name, version, source, hash }`
  - [ ] `struct LockFile { packages: Vec<LockedPackage> }`
  - [ ] `LockFile::load(path: &Path) -> Option<LockFile>`
  - [ ] `LockFile::save(path: &Path) -> Result<(), String>`
  - [ ] TOML フォーマットで読み書き (miniparser 使用)
- [ ] `src/main.rs` に `mod lock;` を追加

### 4-3: `Cargo.toml` に sha2 を追加

- [ ] `sha2 = "0.10"` を追加
- [ ] `cargo build` が通ること

### 4-4: `cmd_install` の実装

- [ ] `src/driver.rs` に `pub fn cmd_install(rune: Option<&str>)` を追加
  - [ ] `fav.toml` を読む
  - [ ] `DependencySpec::Path { path }`:
    - [ ] 相対パスを絶対パスに変換
    - [ ] 対象ディレクトリが存在することを確認
    - [ ] `~/.fav/cache/<name>/` に src/ をコピー
  - [ ] `DependencySpec::Registry { version, registry: "local" }`:
    - [ ] `~/.fav/registry/<name>/<version>/` が存在することを確認
    - [ ] `~/.fav/cache/<name>@<version>/` にコピー
  - [ ] `LockFile` を生成して `fav.lock` に書き出す
  - [ ] 成功したら `"installed N dependencies"` を表示

### 4-5: `cmd_publish` の実装

- [ ] `src/driver.rs` に `pub fn cmd_publish()` を追加
  - [ ] `fav.toml` を読む
  - [ ] `name` と `version` を取得
  - [ ] `src/` 配下をすべてコピーして `~/.fav/registry/<name>/<version>/` に配置
  - [ ] `"published <name>@<version> to local registry"` を表示

### 4-6: `main.rs` にコマンド追加

- [ ] `"install"` コマンド → `cmd_install(file.as_deref())`
- [ ] `"publish"` コマンド → `cmd_publish()`
- [ ] HELP テキストに `install` / `publish` を追加
  ```
  install [rune]    Resolve and install dependencies from fav.toml.
  publish           Publish current rune to local registry.
  ```

### 4-7: 依存解決の `fav run` / `fav check` への組み込み

- [ ] `load_and_check_program` 内で `fav.lock` が存在する場合、
      ロックされた依存の `src/` を `load_all_items` に追加する
- [ ] 依存 rune の `src/` 配下の `.fav` ファイルを再帰的にロードする

### 4-8: テスト

- [ ] `install_path_dep_copies_source` — tempdir に rune を置いて install
- [ ] `publish_creates_registry_entry` — publish して `~/.fav/registry/` に登録される
- [ ] `install_after_publish_resolves` — publish → install の往復テスト
- [ ] `cargo test` が全通過すること

---

## Phase 5: ドキュメント + リリース

### 5-1: langspec.md の完成

- [ ] Phase 0 で作成した langspec.md を完成させる
  - [ ] 全文を見直し、v0.x の変更を反映する
  - [ ] コード例を最低 20 個追加する

### 5-2: examples/ の整備

- [ ] `examples/string_wasm.fav` を作成 (WASM String 戻り値のデモ)
  ```fav
  public fn greet(name: String) -> String {
      name
  }
  public fn main() -> Unit !Io {
      IO.println(greet("Favnir"))
  }
  ```
- [ ] `examples/closures_wasm.fav` を作成 (WASM クロージャのデモ)
  ```fav
  public fn apply(f: Int -> Int, x: Int) -> Int {
      f(x)
  }
  public fn main() -> Unit !Io {
      IO.println_int(apply(|x| x * 2, 21))
  }
  ```
- [ ] `examples/multi_rune/` を作成 (依存管理のデモ)
  - [ ] `fav.toml` に `[dependencies]` を含む例
  - [ ] 依存先の `src/helper.fav` を使うメインファイル

### 5-3: README.md の完全版

- [ ] インストール方法
- [ ] クイックスタート (hello.fav → run → build → exec)
- [ ] 言語概要 (型、effect、trf、chain)
- [ ] CLI リファレンス (全コマンド)
- [ ] WASM backend の使い方
- [ ] LSP 設定例 (VS Code, Neovim)
- [ ] rune 依存管理の使い方

### 5-4: docs/lsp.md の作成

- [ ] LSP の対応機能一覧
- [ ] VS Code: `.vscode/settings.json` に `fav lsp` を設定する方法
- [ ] Neovim: `nvim-lspconfig` 設定例

### 5-5: RELEASE_NOTES.md の作成

- [ ] `versions/v1.0.0/RELEASE_NOTES.md` を作成
  - [ ] v0.x.x からの変更点一覧
  - [ ] WASM の変更点 (String 戻り値、クロージャ)
  - [ ] LSP の設定例
  - [ ] 既知の制限事項
  - [ ] v1.1.0 予告 (List/Map WASM、HTTP registry)

### 5-6: 全体確認

- [ ] `cargo build` で警告ゼロ
- [ ] `cargo test` 全通過 (目標: 330+)
- [ ] `fav lsp` が起動して `initialize` に応答する
- [ ] `fav build --target wasm examples/string_wasm.fav` が通る
- [ ] `fav build --target wasm examples/closures_wasm.fav` が通る
- [ ] `fav install` が `fav.toml` の依存を解決する
- [ ] `fav publish` が `~/.fav/registry/` に登録する
- [ ] `langspec.md` が完成している
- [ ] `README.md` が完全版になっている

---

## 全体完了条件

- [ ] `cargo build` で警告ゼロ
- [ ] `cargo test` 全テスト通過
- [ ] `fav lsp` が VS Code または Neovim で動作する
- [ ] `fn greet() -> String { "hello" }` が WASM でビルド・実行できる
- [ ] クロージャを含む関数が WASM でビルド・実行できる
- [ ] `fav install` がローカルパス依存を解決できる
- [ ] `fav publish` がローカルレジストリに登録できる
- [ ] `langspec.md` が完成している
- [ ] `README.md` が完全版になっている
- [ ] `Cargo.toml` バージョンが `"1.0.0"`
- [ ] roadmap.md の v0.9.0 を完了マーク、v1.0.0 を進行中マーク

---

## 既知の制約・先送り事項 (v1.1.0 以降)

| 制約 | 対応バージョン |
|---|---|
| `List<T>` / `Map<V>` の WASM 対応 | v1.1.0 (WasmGC) |
| `trf` / `flw` の WASM 対応 | v1.1.0 |
| `chain` / `collect` の WASM 対応 | v1.1.0 |
| `Db`/`Network`/`File` の WASM 対応 | v1.1.0 |
| HTTP レジストリ | v1.1.0 |
| LSP: 補完 (completion) | v1.1.0 |
| LSP: リネーム (rename) | v1.1.0 |
| String を返す `if/else` の WASM 対応 | v1.1.0 |
| セルフホスティング | v2.0.0 |
