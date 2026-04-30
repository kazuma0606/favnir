# Favnir v0.9.0 タスク一覧 — WASM Backend

更新日: 2026-04-30（Codex レビュー反映）

> [ ] 未完了 / [x] 完了
>
> **ゴール**: typed IR → WebAssembly binary の生成と wasmtime による実行
> **前提**: v0.8.0 完了（251 テスト通過）

---

## Phase 0: 依存追加 + scaffold

### 0-1: Cargo.toml に依存を追加

- [ ] `wasm-encoder` を追加（bytecodealliance/wasm-tools）
- [ ] `wasmtime` を追加（`default-features = false, features = ["cranelift", "runtime"]`）
- [ ] `cargo build` が通ること（依存のダウンロード・コンパイル確認）
- [ ] `Cargo.toml` バージョンを `"0.9.0"` に更新

### 0-2: 新規ファイルの scaffold

- [ ] `src/backend/wasm_codegen.rs` を作成
  - [ ] `pub enum WasmCodegenError` を定義
    - [ ] `UnsupportedType(String)` — W001
    - [ ] `UnsupportedExpr(String)` — W002
    - [ ] `UnsupportedMainSignature` — W003
  - [ ] `impl WasmCodegenError { fn code(&self) -> &str }` — "W001"/"W002"/"W003" を返す
  - [ ] `impl std::fmt::Display for WasmCodegenError`
  - [ ] `pub fn wasm_codegen_program(ir: &IRProgram) -> Result<Vec<u8>, WasmCodegenError>` — スタブ（`\0asm\x01\0\0\0` を返す）
- [ ] `src/backend/wasm_exec.rs` を作成
  - [ ] `pub fn wasm_exec_main(bytes: &[u8]) -> Result<(), String>` — スタブ
  - [ ] `pub fn wasm_exec_info(bytes: &[u8]) -> String` — スタブ
- [ ] `src/backend/mod.rs` または適切な場所に `pub mod wasm_codegen; pub mod wasm_exec;` を追加
- [ ] `cargo build` が通ること

### 0-3: `compiler.rs` に WASM 専用ビルトイン登録

- [ ] `IO.println_int` / `IO.println_float` / `IO.println_bool` / `IO.print` をビルトインとして登録
- [ ] `vm.rs` に対応するハンドラを追加（`.fvc` でも動くように）
  - [ ] `"IO.println_int"` → `println!("{}", n)` → `VMValue::Unit`
  - [ ] `"IO.println_float"` → `println!("{}", f)` → `VMValue::Unit`
  - [ ] `"IO.println_bool"` → `println!("{}", if b {"true"} else {"false"})` → `VMValue::Unit`
- [ ] `cargo test` が全通過すること（既存 251 テスト）

---

## Phase 1: 純粋算術 WASM codegen

### 1-1: 型マッピング関数の実装

- [ ] `fn favnir_type_to_wasm_results(ty: &Type) -> Result<Vec<ValType>, WasmCodegenError>` を実装
  - [ ] `Type::Unit` → `Ok(vec![])`
  - [ ] `Type::Int` → `Ok(vec![ValType::I64])`
  - [ ] `Type::Float` → `Ok(vec![ValType::F64])`
  - [ ] `Type::Bool` → `Ok(vec![ValType::I32])`
  - [ ] `Type::Str` → `Err(WasmCodegenError::UnsupportedType("String as return type".into()))` — W001
  - [ ] その他 → `Err(W001)`
- [ ] `fn favnir_type_to_wasm_params(ty: &Type) -> Result<Vec<ValType>, WasmCodegenError>` を実装
  - [ ] `Type::Str` → `Ok(vec![ValType::I32, ValType::I32])` （パラメータは (ptr, len) で OK）
  - [ ] 他は `favnir_type_to_wasm_results` に委譲

### 1-2: ローカル変数の型収集

- [ ] `fn collect_local_types(expr: &IRExpr, map: &mut HashMap<u16, Type>)` を実装（再帰）
- [ ] `fn collect_local_types_stmt(stmt: &IRStmt, map: &mut HashMap<u16, Type>)` を実装
- [ ] `IRExpr::Local(idx, ty)` を収集して HashMap に格納

### 1-3: WasmCodegenCtx 構造体

- [ ] `struct WasmCodegenCtx` を定義
  - [ ] `fn_to_wasm_idx: HashMap<usize, u32>` — IR fn_idx → WASM function idx（import offset 込み）
  - [ ] `builtin_to_wasm_idx: HashMap<String, u32>` — "IO.println" 等 → import fn idx
  - [ ] `str_to_offset: HashMap<String, u32>` — 文字列 → data section offset
  - [ ] `globals: &[IRGlobal]`

### 1-4: 関数シグネチャの生成（TypeSection）

- [ ] `fn build_type_section(ir: &IRProgram, imports: &[HostImport]) -> Result<(TypeSection, HashMap<usize, u32>), WasmCodegenError>` を実装
  - [ ] 各 import のシグネチャを先に追加
  - [ ] 各 `IRFnDef` のシグネチャを追加（重複は一つにまとめても可）
  - [ ] `main` が `() -> Unit` 以外のシグネチャを持つ場合は `Err(W003)`
- [ ] FunctionSection に type index を登録

### 1-5: IRExpr の emit 実装

- [ ] `fn emit_expr(expr: &IRExpr, ctx: &WasmCodegenCtx, func: &mut Function) -> Result<(), WasmCodegenError>` を実装
  - [ ] `Lit(Int(n))` → `Instruction::I64Const(*n)`
  - [ ] `Lit(Float(f))` → `Instruction::F64Const(*f)`
  - [ ] `Lit(Bool(true))` → `Instruction::I32Const(1)`
  - [ ] `Lit(Bool(false))` → `Instruction::I32Const(0)`
  - [ ] `Lit(Unit)` → なし
  - [ ] `Local(idx, _)` → `Instruction::LocalGet(*idx as u32)`
  - [ ] `BinOp` on Int:
    - [ ] emit lhs, emit rhs
    - [ ] `Add` → `Instruction::I64Add`
    - [ ] `Sub` → `Instruction::I64Sub`
    - [ ] `Mul` → `Instruction::I64Mul`
    - [ ] `Div` → `Instruction::I64DivS`
    - [ ] `Eq` → `Instruction::I64Eq`
    - [ ] `NotEq` → `Instruction::I64Ne`
    - [ ] `Lt` → `Instruction::I64LtS`
    - [ ] `Gt` → `Instruction::I64GtS`
    - [ ] `LtEq` → `Instruction::I64LeS`
    - [ ] `GtEq` → `Instruction::I64GeS`
  - [ ] `BinOp` on Float:
    - [ ] `Add/Sub/Mul/Div` → `F64Add` 等
    - [ ] `Eq/Ne/Lt/Gt/Le/Ge` → `F64Eq` 等
  - [ ] `If(cond, then, else_, ty)`:
    - [ ] `favnir_type_to_wasm_results(ty)` で `BlockType` を決定
    - [ ] emit cond
    - [ ] `Instruction::If(block_type)`
    - [ ] emit then
    - [ ] `Instruction::Else`
    - [ ] emit else_
    - [ ] `Instruction::End`
  - [ ] `Block(stmts, final_expr, _)`:
    - [ ] 各 stmt を emit
    - [ ] emit final_expr
  - [ ] `Call(Global(fn_idx), args, _)`:
    - [ ] 各 arg を emit
    - [ ] `ctx.fn_to_wasm_idx` から WASM 関数 index を取得
    - [ ] `Instruction::Call(wasm_idx)`
  - [ ] 対応外 (`Closure`, `Collect`, `Emit`, `RecordConstruct`, `Match`) → `Err(W002)`

### 1-6: IRStmt の emit 実装

- [ ] `fn emit_stmt(stmt: &IRStmt, ctx: &WasmCodegenCtx, func: &mut Function) -> Result<(), WasmCodegenError>` を実装
  - [ ] `Bind(local_idx, expr)` → emit(expr) → `Instruction::LocalSet(*local_idx as u32)`
  - [ ] `Expr(expr)`:
    - [ ] emit(expr)
    - [ ] 式の型が Unit でなければ `Instruction::Drop`
  - [ ] `Chain(...)` / `Yield(...)` → `Err(W002)`

### 1-7: 関数コード生成

- [ ] `fn build_wasm_function(fn_def: &IRFnDef, ctx: &WasmCodegenCtx) -> Result<Function, WasmCodegenError>` を実装
  - [ ] param_count 個のパラメータを除いたローカル変数を `Function::new(locals)` で宣言
  - [ ] `emit_expr(&fn_def.body, ctx, &mut func)`
  - [ ] `Instruction::End`

### 1-8: ExportSection の生成

- [ ] `main` 関数を `export "main"` として追加
- [ ] ExportSection を Module に追加

### 1-9: W003 チェック（main シグネチャ検証）

- [ ] `main` 関数が見つかったとき、パラメータなし・戻り値 Unit であることを確認
- [ ] 違う場合は `Err(WasmCodegenError::UnsupportedMainSignature)` を返す

### 1-10: 最小動作確認テスト

- [ ] `wasm_codegen_program` が `\0asm\x01\0\0\0` で始まるバイト列を返すこと
- [ ] 純粋 Int 演算のみの `main` でバリデーションが通ること（`wasmparser` で検証、または wasmtime でロードのみ）

---

## Phase 2: 文字列 + IO ホスト import

### 2-1: 文字列リテラルの収集

- [ ] `fn collect_string_literals(ir: &IRProgram) -> (Vec<u8>, HashMap<String, u32>)` を実装
  - [ ] 全 `IRExpr::Lit(Lit::Str(s), _)` を収集（再帰）
  - [ ] 同じ文字列は intern（1度だけ data section に追加）
  - [ ] `(data_bytes, str_to_offset_map)` を返す

### 2-2: MemorySection + DataSection の生成

- [ ] MemorySection を Module に追加（1 ページ = 64KB）
- [ ] `(memory (export "memory") 1)` として memory を export
- [ ] DataSection に文字列バイト列を配置（offset 0 から順に）
- [ ] Module に DataSection を追加

### 2-3: 文字列リテラルの emit

- [ ] `Lit(Str(s))` の emit を実装
  - [ ] `ctx.str_to_offset[s]` からオフセットを取得
  - [ ] `Instruction::I32Const(offset as i32)` → ptr
  - [ ] `Instruction::I32Const(s.len() as i32)` → len

### 2-4: HostImport の定義とマッピング

- [ ] `enum HostImport` を定義
  - [ ] `IoPrintln`, `IoPrint`, `IoPrintlnInt`, `IoPrintlnFloat`, `IoPrintlnBool`
- [ ] `fn builtin_name_to_host_import(name: &str) -> Option<HostImport>` を実装
  - [ ] `"IO.println"` → `Some(IoPrintln)` 等

### 2-5: 使用ビルトインの収集

- [ ] `fn collect_used_builtins(ir: &IRProgram) -> HashSet<String>` を実装
  - [ ] `IRExpr::FieldAccess(Global("IO"), field)` 等を収集
- [ ] 使用されるホスト関数のみ ImportSection に追加

### 2-6: ImportSection の生成

- [ ] 使用される HostImport を ImportSection に追加
  - [ ] `import("fav_host", "io_println", EntityType::Function(type_idx))`
  - [ ] `import("fav_host", "io_println_int", EntityType::Function(type_idx))`
  - [ ] 他も同様
- [ ] `ctx.builtin_to_wasm_idx` に import function idx を記録
- [ ] `ctx.fn_to_wasm_idx` の offset を `import_count` 分ずらす

### 2-7: ビルトイン call の emit

- [ ] `Call(FieldAccess(Global("IO"), "println"), args)` のパターンを検出
- [ ] `ctx.builtin_to_wasm_idx["IO.println"]` を使って `Instruction::Call(import_idx)` を emit
- [ ] `Debug.show(...)` の呼び出しを検出したら `Err(W002)` を返す

### 2-8: wasmtime ホスト関数の登録（wasm_exec.rs 実装）

- [ ] `fn register_host_functions(linker: &mut Linker<()>) -> Result<(), String>` を実装
  - [ ] `io_println(ptr: i32, len: i32)` — memory から文字列を読んで println!
  - [ ] `io_print(ptr: i32, len: i32)` — 改行なし print!
  - [ ] `io_println_int(n: i64)` — println!("{}", n)
  - [ ] `io_println_float(f: f64)` — println!("{}", f)
  - [ ] `io_println_bool(b: i32)` — println!("{}", if b != 0 {"true"} else {"false"})

---

## Phase 3: `fav build --target wasm` CLI

### 3-1: `src/main.rs` の更新

- [ ] `"build"` コマンドのパースに `"--target"` フラグを追加
  - [ ] `--target <fvc|wasm>` を解析して `cmd_build` に渡す
- [ ] HELP テキスト更新:
  ```
      build [-o <file>] [--target <fvc|wasm>] [file]
                    --target fvc   build .fvc (default)
                    --target wasm  build WebAssembly binary (.wasm)
  ```
- [ ] ERROR CODES セクションに追加:
  ```
      W001  WASM codegen: unsupported type (String as return type, etc.)
      W002  WASM codegen: unsupported expression (Debug.show, List, closure, etc.)
      W003  WASM codegen: main signature not supported (must be () -> Unit !Io)
      W004  --db cannot be used with .wasm artifacts
  ```

### 3-2: `src/driver.rs` の更新

- [ ] `pub fn cmd_build(file, out, target)` のシグネチャを変更（target: Option<&str> 追加）
- [ ] `cmd_build_fvc` と `cmd_build_wasm` に分岐
- [ ] `cmd_build_wasm` を実装:
  - [ ] load_and_check_program → compile_program → wasm_codegen_program
  - [ ] エラー時は `error[Wxxx]:` + hint を出力して exit(1)
  - [ ] `.wasm` ファイルに書き出し
  - [ ] `"built <path> (wasm)"` を表示

---

## Phase 4: `fav exec <file.wasm>` via wasmtime

### 4-1: `wasm_exec_main` の実装（wasm_exec.rs）

- [ ] `pub fn wasm_exec_main(bytes: &[u8]) -> Result<(), String>` を実装
  - [ ] `Engine::default()` でエンジン生成
  - [ ] `Module::new(&engine, bytes)` でモジュールロード
  - [ ] `Store::new(&engine, ())` でストア生成
  - [ ] `Linker::new(&engine)` でリンカ生成
  - [ ] `register_host_functions(&mut linker)?`
  - [ ] `linker.instantiate(&mut store, &module)` でインスタンス化
  - [ ] `instance.get_typed_func::<(), ()>(&mut store, "main")` で main 取得
  - [ ] `main.call(&mut store, ())` で実行

### 4-2: `wasm_exec_info` の実装

- [ ] `pub fn wasm_exec_info(bytes: &[u8]) -> String` を実装
  - [ ] `artifact: .wasm` / `format: WebAssembly binary`
  - [ ] `exports:` — export 一覧
  - [ ] `imports:` — import 一覧
  - [ ] `memory:` — ページ数

### 4-3: `src/driver.rs` の `cmd_exec` 更新

- [ ] `.wasm` 拡張子を検出して分岐
- [ ] **W004 チェック**: `db_path.is_some()` かつ `.wasm` → `error[W004]:` を出力して exit(1)
- [ ] `.wasm` 用: `wasm_exec_main(bytes)` / `wasm_exec_info(bytes)` を呼ぶ
- [ ] `.fvc` 用: 既存パスを維持

### 4-4: `.wasm` ファイルの読み込みヘルパー

- [ ] `fn read_wasm_from_path(path: &str) -> Result<Vec<u8>, String>` を実装

---

## Phase 5: テスト + examples

### 5-1: `examples/math_wasm.fav` の作成

- [ ] `fn add(a: Int, b: Int) -> Int` / `fn factorial(n: Int) -> Int` を含むファイルを作成
- [ ] `IO.println_int` を使って結果を出力
- [ ] `fav build --target wasm examples/math_wasm.fav` が通ること
- [ ] `fav exec math_wasm.wasm` で正しい出力が得られること

### 5-2: `examples/hello.fav` の WASM ビルド確認

- [ ] `fav build --target wasm examples/hello.fav` → `hello.wasm` 生成
- [ ] `fav exec hello.wasm` → `Hello, Favnir!` 出力

### 5-3: Rust テスト（wasm_codegen.rs 内）

- [ ] `wasm_hello_world` — `IO.println("Hello")` を含む main が動く
- [ ] `wasm_int_arithmetic` — `21 + 21` → `IO.println_int(42)`
- [ ] `wasm_if_else` — `fn abs(n: Int) -> Int { if n < 0 { 0 - n } else { n } }`
- [ ] `wasm_recursive_factorial` — `factorial(5)` → 120
- [ ] `wasm_bool_ops` — 比較演算が正しく動く
- [ ] `wasm_w001_string_return` — `fn greet() -> String { "hi" }` → `Err(W001)`
- [ ] `wasm_w002_debug_show` — `Debug.show(42)` → `Err(W002)`
- [ ] `wasm_w003_main_returns_int` — `main() -> Int` → `Err(W003)`

### 5-4: W004 テスト

- [ ] `cmd_exec` に `.wasm` + `--db` を渡したとき exit(1) + `error[W004]` が出ること

### 5-5: 全体確認

- [ ] `cargo build` で警告ゼロ
- [ ] `cargo test` 全通過（テスト数 > 251）
- [ ] `fav build --target wasm examples/hello.fav` → `hello.wasm` 生成
- [ ] `fav exec hello.wasm` → `Hello, Favnir!`
- [ ] `fav exec --info hello.wasm` → メタデータ表示
- [ ] `fav exec --db x.db hello.wasm` → `error[W004]`

---

## 全体完了条件

- [ ] `cargo build` で警告ゼロ
- [ ] `cargo test` 全テスト通過
- [ ] `fav build --target wasm examples/hello.fav` が動く
- [ ] `fav exec hello.wasm` で `Hello, Favnir!` が出力される
- [ ] `fav exec --db x.db hello.wasm` が W004 エラーを出す
- [ ] String 戻り値の fn が W001 エラーを出す
- [ ] `Debug.show` 呼び出しが W002 エラーを出す
- [ ] `Cargo.toml` バージョンが `"0.9.0"`
- [ ] roadmap.md の v0.8.0 を完了マーク、v0.9.0 を進行中マーク

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
