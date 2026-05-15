# Favnir v0.9.0 タスク一覧 — WASM Backend

更新日: 2026-04-30（Codex レビュー反映・実装突合完了）

> [x] 完了
>
> **ゴール**: typed IR → WebAssembly binary の生成と wasmtime による実行
> **前提**: v0.8.0 完了（251 テスト通過）
> **結果**: v0.9.0 完了（289 テスト通過）

---

## Phase 0: 依存追加 + scaffold

### 0-1: Cargo.toml に依存を追加

- [x] `wasm-encoder` を追加（bytecodealliance/wasm-tools）
- [x] `wasmtime` を追加（`default-features = false, features = ["cranelift", "runtime"]`）
- [x] `cargo build` が通ること（依存のダウンロード・コンパイル確認）
- [x] `Cargo.toml` バージョンを `"0.9.0"` に更新

### 0-2: 新規ファイルの scaffold

- [x] `src/backend/wasm_codegen.rs` を作成
  - [x] `pub enum WasmCodegenError` を定義
    - [x] `UnsupportedType(String)` — W001
    - [x] `UnsupportedExpr(String)` — W002
    - [x] `UnsupportedMainSignature` — W003
  - [x] `impl WasmCodegenError { fn code(&self) -> &str }` — "W001"/"W002"/"W003" を返す
  - [x] `impl std::fmt::Display for WasmCodegenError`
  - [x] `pub fn wasm_codegen_program(ir: &IRProgram) -> Result<Vec<u8>, WasmCodegenError>`
- [x] `src/backend/wasm_exec.rs` を作成
  - [x] `pub fn wasm_exec_main(bytes: &[u8]) -> Result<(), String>` — 実装済み
  - [x] `pub fn wasm_exec_info(bytes: &[u8]) -> String` — 実装済み
- [x] `src/backend/mod.rs` に `pub mod wasm_codegen; pub mod wasm_exec;` を追加
- [x] `cargo build` が通ること

### 0-3: `compiler.rs` に WASM 専用ビルトイン登録

- [x] `IO.println_int` / `IO.println_float` / `IO.println_bool` / `IO.print` をビルトインとして登録
- [x] `vm.rs` に対応するハンドラを追加（`.fvc` でも動くように）
  - [x] `"IO.println_int"` → `println!("{}", n)` → `VMValue::Unit`
  - [x] `"IO.println_float"` → `println!("{}", f)` → `VMValue::Unit`
  - [x] `"IO.println_bool"` → `println!("{}", if b {"true"} else {"false"})` → `VMValue::Unit`
- [x] `cargo test` が全通過すること（既存 251 テスト）

---

## Phase 1: 純粋算術 WASM codegen

### 1-1: 型マッピング関数の実装

- [x] `fn favnir_type_to_wasm_results(ty: &Type) -> Result<Vec<ValType>, WasmCodegenError>` を実装
  - [x] `Type::Unit` → `Ok(vec![])`
  - [x] `Type::Int` → `Ok(vec![ValType::I64])`
  - [x] `Type::Float` → `Ok(vec![ValType::F64])`
  - [x] `Type::Bool` → `Ok(vec![ValType::I32])`
  - [x] `Type::Str` → `Err(WasmCodegenError::UnsupportedType("String as return type".into()))` — W001
  - [x] その他 → `Err(W001)`
- [x] `fn favnir_type_to_wasm_params(ty: &Type) -> Result<Vec<ValType>, WasmCodegenError>` を実装
  - [x] `Type::Str` → `Ok(vec![ValType::I32, ValType::I32])` （パラメータは (ptr, len) で OK）
  - [x] 他は `favnir_type_to_wasm_results` に委譲

### 1-2: ローカル変数の型収集

- [x] `fn collect_local_types(expr: &IRExpr, map: &mut HashMap<u16, Type>)` を実装（再帰）
- [x] `fn collect_local_types_stmt(stmt: &IRStmt, map: &mut HashMap<u16, Type>)` を実装
- [x] `IRExpr::Local(idx, ty)` を収集して HashMap に格納

### 1-3: WasmCodegenCtx 構造体

- [x] `struct WasmCodegenCtx` を定義
  - [x] `fn_to_wasm_idx: HashMap<usize, u32>` — IR fn_idx → WASM function idx（import offset 込み）
  - [x] `builtin_to_wasm_idx: HashMap<String, u32>` — "IO.println" 等 → import fn idx
  - [x] `str_to_offset: HashMap<String, u32>` — 文字列 → data section offset
  - [x] `globals: &[IRGlobal]`

### 1-4: 関数シグネチャの生成（TypeSection）

- [x] `fn build_type_section(ir: &IRProgram, imports: &[HostImport]) -> Result<(TypeSection, HashMap<usize, u32>), WasmCodegenError>` を実装
  - [x] 各 import のシグネチャを先に追加
  - [x] 各 `IRFnDef` のシグネチャを追加（重複は一つにまとめても可）
  - [x] `main` が `() -> Unit` 以外のシグネチャを持つ場合は `Err(W003)`
- [x] FunctionSection に type index を登録

### 1-5: IRExpr の emit 実装

- [x] `fn emit_expr(expr: &IRExpr, ctx: &WasmCodegenCtx, func: &mut Function) -> Result<(), WasmCodegenError>` を実装
  - [x] `Lit(Int(n))` → `Instruction::I64Const(*n)`
  - [x] `Lit(Float(f))` → `Instruction::F64Const(*f)`
  - [x] `Lit(Bool(true))` → `Instruction::I32Const(1)`
  - [x] `Lit(Bool(false))` → `Instruction::I32Const(0)`
  - [x] `Lit(Unit)` → なし
  - [x] `Local(idx, _)` → `Instruction::LocalGet(*idx as u32)`
  - [x] `BinOp` on Int: Add/Sub/Mul/Div/Eq/Ne/Lt/Gt/LtEq/GtEq → I64 命令
  - [x] `BinOp` on Float: Add/Sub/Mul/Div/Eq/Ne/Lt/Gt/Le/Ge → F64 命令
  - [x] `If(cond, then, else_, ty)`: block_type 決定 → emit cond → If → emit then → Else → emit else_ → End
  - [x] `Block(stmts, final_expr, _)`: 各 stmt を emit → emit final_expr
  - [x] `Call(Global(fn_idx), args, _)`: args emit → Call(wasm_idx)
  - [x] 対応外 (`Closure`, `Collect`, `Emit`, `RecordConstruct`, `Match`) → `Err(W002)`

### 1-6: IRStmt の emit 実装

- [x] `fn emit_stmt(stmt: &IRStmt, ctx: &WasmCodegenCtx, func: &mut Function) -> Result<(), WasmCodegenError>` を実装
  - [x] `Bind(local_idx, expr)` → emit(expr) → `Instruction::LocalSet(*local_idx as u32)`
  - [x] `Expr(expr)`: emit(expr) → 型が Unit でなければ Drop
  - [x] `Chain(...)` / `Yield(...)` → `Err(W002)`

### 1-7: 関数コード生成

- [x] `fn build_wasm_function(fn_def: &IRFnDef, ctx: &WasmCodegenCtx) -> Result<Function, WasmCodegenError>` を実装
  - [x] param_count 個のパラメータを除いたローカル変数を `Function::new(locals)` で宣言
  - [x] `emit_expr(&fn_def.body, ctx, &mut func)`
  - [x] `Instruction::End`

### 1-8: ExportSection の生成

- [x] `main` 関数を `export "main"` として追加
- [x] ExportSection を Module に追加

### 1-9: W003 チェック（main シグネチャ検証）

- [x] `main` 関数が見つかったとき、パラメータなし・戻り値 Unit・effects = [Io] であることを確認
- [x] 違う場合は `Err(WasmCodegenError::UnsupportedMainSignature)` を返す

### 1-10: 最小動作確認テスト

- [x] `wasm_codegen_program` が `\0asm\x01\0\0\0` で始まるバイト列を返すこと
- [x] 純粋 Int 演算のみの `main` でバリデーションが通ること（wasmtime でロード確認）

---

## Phase 2: 文字列 + IO ホスト import

### 2-1: 文字列リテラルの収集

- [x] `fn collect_string_literals(ir: &IRProgram) -> (Vec<u8>, HashMap<String, u32>)` を実装
  - [x] 全 `IRExpr::Lit(Lit::Str(s), _)` を収集（再帰）
  - [x] 同じ文字列は intern（1度だけ data section に追加）
  - [x] `(data_bytes, str_to_offset_map)` を返す

### 2-2: MemorySection + DataSection の生成

- [x] MemorySection を Module に追加（1 ページ = 64KB）
- [x] `memory` を export
- [x] DataSection に文字列バイト列を配置（offset 0 から順に）
- [x] Module に DataSection を追加

### 2-3: 文字列リテラルの emit

- [x] `Lit(Str(s))` の emit を実装
  - [x] `ctx.str_to_offset[s]` からオフセットを取得
  - [x] `Instruction::I32Const(offset as i32)` → ptr
  - [x] `Instruction::I32Const(s.len() as i32)` → len

### 2-4: HostImport の定義とマッピング

- [x] `enum HostImport` を定義
  - [x] `IoPrintln`, `IoPrint`, `IoPrintlnInt`, `IoPrintlnFloat`, `IoPrintlnBool`
- [x] `fn host_imports()` で builtin name → HostImport マッピングを実装

### 2-5: 使用ビルトインの収集

- [x] `fn collect_used_builtins(ir: &IRProgram) -> HashSet<String>` を実装
- [x] 使用されるホスト関数のみ ImportSection に追加

### 2-6: ImportSection の生成

- [x] 使用される HostImport を ImportSection に追加（"fav_host" モジュール）
- [x] `ctx.builtin_to_wasm_idx` に import function idx を記録
- [x] `ctx.fn_to_wasm_idx` の offset を `import_count` 分ずらす

### 2-7: ビルトイン call の emit

- [x] `Call(FieldAccess(Global("IO"), "println"), args)` パターンを検出して `Instruction::Call(import_idx)` を emit
- [x] `Debug.show(...)` 等の未対応ビルトイン呼び出し → `Err(W002)`

### 2-8: wasmtime ホスト関数の登録（wasm_exec.rs 実装）

- [x] `fn register_host_functions(linker: &mut Linker<()>) -> Result<(), String>` を実装
  - [x] `io_println(ptr: i32, len: i32)` — memory から文字列を読んで println!
  - [x] `io_print(ptr: i32, len: i32)` — 改行なし print!
  - [x] `io_println_int(n: i64)` — println!("{}", n)
  - [x] `io_println_float(f: f64)` — println!("{}", f)
  - [x] `io_println_bool(b: i32)` — println!("{}", if b != 0 {"true"} else {"false"})

---

## Phase 3: `fav build --target wasm` CLI

### 3-1: `src/main.rs` の更新

- [x] `"build"` コマンドのパースに `"--target"` フラグを追加
- [x] HELP テキスト更新（`--target <fvc|wasm>` 記載）
- [x] ERROR CODES セクションに W001-W004 追加

### 3-2: `src/driver.rs` の更新

- [x] `pub fn cmd_build(file, out, target)` のシグネチャを変更（target: Option<&str> 追加）
- [x] `"fvc"` / `"wasm"` に分岐
- [x] WASM パス: load_and_check_program → compile_program → wasm_codegen_program → ファイル書き出し
- [x] エラー時は `error[Wxxx]:` メッセージを出力して exit(1)

---

## Phase 4: `fav exec <file.wasm>` via wasmtime

### 4-1: `wasm_exec_main` の実装（wasm_exec.rs）

- [x] `pub fn wasm_exec_main(bytes: &[u8]) -> Result<(), String>` を実装
  - [x] Engine/Module/Store/Linker 生成
  - [x] `register_host_functions` で ホスト関数登録
  - [x] linker.instantiate → get_typed_func::<(), ()>("main") → call

### 4-2: `wasm_exec_info` の実装

- [x] `pub fn wasm_exec_info(bytes: &[u8]) -> String` を実装
  - [x] artifact, format, size, status, imports, exports, memory 情報を表示

### 4-3: `src/driver.rs` の `cmd_exec` 更新

- [x] `.wasm` 拡張子を検出して分岐
- [x] **W004 チェック**: `db_path.is_some()` かつ `.wasm` → `error[W004]:` を出力して exit(1)
- [x] `.wasm` 用: `wasm_exec_main(bytes)` / `wasm_exec_info(bytes)` を呼ぶ
- [x] `.fvc` 用: 既存パスを維持

### 4-4: `.wasm` ファイルの読み込みヘルパー

- [x] `fn read_wasm_from_path(path: &Path) -> Result<Vec<u8>, String>` を実装

---

## Phase 5: テスト + examples

### 5-1: `examples/math_wasm.fav` の作成

- [x] `fn add(a: Int, b: Int) -> Int` / `fn factorial(n: Int) -> Int` / `fn abs(n: Int) -> Int` を含むファイルを作成
- [x] `IO.println_int` を使って結果を出力
- [x] driver テスト `example_math_wasm_build_and_exec` で確認

### 5-2: `examples/hello.fav` の WASM ビルド確認

- [x] `IO.println("Hello, Favnir!")` の文字列 WASM ビルドが通ること
- [x] driver テスト `example_hello_wasm_build_and_exec` で確認

### 5-3: Rust テスト（wasm_codegen.rs 内）

- [x] `wasm_hello_world` — `IO.println("Hello")` を含む main が動く
- [x] `wasm_int_arithmetic` — `21 + 21` → `IO.println_int(42)`
- [x] `wasm_if_else` — `fn abs(n: Int) -> Int { if n < 0 { 0 - n } else { n } }`
- [x] `wasm_recursive_factorial` — `factorial(5)` → 120
- [x] `wasm_bool_ops` — 比較演算が正しく動く
- [x] `wasm_w001_string_return` — `fn greet() -> String { "hi" }` → `Err(W001)`
- [x] `wasm_w002_debug_show` — `Debug.show(42)` → `Err(W002)`
- [x] `wasm_w003_main_returns_int` — `main() -> Int` → `Err(W003)`

### 5-4: W004 テスト

- [x] `cmd_exec` に `.wasm` + `--db` を渡したとき exit(1) + `error[W004]` が出ること
- [x] driver テスト `wasm_exec_bytes_rejects_db_path_with_w004` で確認

### 5-5: 全体確認

- [x] `cargo build` で警告ゼロ
- [x] `cargo test` 全通過（289 テスト）
- [x] `examples/math_wasm.fav` 存在確認済み
- [x] `examples/hello.fav` WASM ビルド確認済み（driver テスト）

---

## 全体完了条件

- [x] `cargo build` で警告ゼロ
- [x] `cargo test` 全テスト通過（289）
- [x] `fav build --target wasm examples/hello.fav` が動く（driver テスト確認）
- [x] `fav exec hello.wasm` で `Hello, Favnir!` が出力される（driver テスト確認）
- [x] `fav exec --db x.db hello.wasm` が W004 エラーを出す
- [x] String 戻り値の fn が W001 エラーを出す
- [x] `Debug.show` 呼び出しが W002 エラーを出す
- [x] `Cargo.toml` バージョンが `"0.9.0"`

---

## 既知の制約・先送り事項

| 制約 | 理由 | 対応バージョン |
|---|---|---|
| String を戻り値にできない | multi-value return の型管理が複雑 | v1.0.0 |
| `Debug.show` 非対応 | String を返すため | v1.0.0 |
| `List<T>` / `Map<V>` 非対応 | ヒープ管理が必要 | v1.0.0（WasmGC 検討） |
| `type` (record/sum) 非対応 | タグ付きポインタ表現が必要 | v1.0.0 |
| クロージャ非対応 | 関数テーブルが必要 | v1.0.0 |
| `trf` / `flw` 非対応 | 高階関数 + クロージャに依存 | v1.0.0 |
| `Db`/`Network`/`File` effect 非対応 | ホスト import 設計が必要 | v1.0.0 |
| `chain` / `collect` 非対応 | コレクション型に依存 | v1.0.0 |
| WASI 非対応 | 優先度低 | 検討中 |
| main に `!Pure` 効果不可 | ensure_supported_main_signature が !Io のみ許可 | 仕様通り |
