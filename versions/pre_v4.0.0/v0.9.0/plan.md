# Favnir v0.9.0 実装計画 — WASM Backend

更新日: 2026-04-30（Codex レビュー反映）

---

## フェーズ構成と依存関係

```
Phase 0: 依存追加 + scaffold
    │
    ├── Phase 1: 純粋算術 WASM codegen
    │       (Int/Bool/Float + if/else + bind + 直接 fn call)
    │       │
    │       ├── Phase 2: 文字列 + IO ホスト import
    │       │       (data section, io_println, io_println_int/float/bool)
    │       │       │
    │       │       ├── Phase 3: fav build --target wasm CLI
    │       │       └── Phase 4: fav exec <file.wasm> via wasmtime
    │       │               │
    │       │               └── Phase 5: tests + examples
    │       │
    │       └── （Phase 6 以降: List/variant/closure — v1.0.0 先送り）
```

---

## Phase 0: 依存追加 + scaffold

### Cargo.toml 追加

```toml
[dependencies]
wasm-encoder = "0.x"   # bytecodealliance/wasm-tools
wasmtime = { version = "x", default-features = false, features = ["cranelift", "runtime"] }
```

> バージョンは実装開始時点の最新 stable に合わせる。
> `wasmtime` は full build が重いので feature を絞る。
> テスト環境でビルドが遅い場合は `features = ["winch"]` (軽量 JIT) も検討。

### 新規ファイル

```
src/backend/wasm_codegen.rs   — IR → WASM binary 生成
src/backend/wasm_exec.rs      — wasmtime による .wasm 実行
```

### mod 登録

`src/backend/mod.rs`（または backend/mod.rs が存在しない場合は `src/backend.rs`）に:

```rust
pub mod wasm_codegen;
pub mod wasm_exec;
```

---

## Phase 1: 純粋算術 WASM codegen

### 設計方針: 型変換を Vec<ValType> で持つ

WASM の型変換関数は**複数の ValType を返せる**設計にする。
String = `(i32, i32)` の 2 値表現に備えるためだが、
**v0.9.0 では String を戻り値にしないため**、実質 0〜1 値になる。

```rust
/// Favnir 型を WASM の ValType リストに変換する。
/// Unit → [] (空)
/// Int  → [I64]
/// Bool → [I32]
/// Float→ [F64]
/// String（戻り値）→ Err(W001) ← v0.9.0 では非対応
/// その他 → Err(W001)
fn favnir_type_to_wasm_results(ty: &Type) -> Result<Vec<ValType>, WasmCodegenError> {
    match ty {
        Type::Unit    => Ok(vec![]),
        Type::Int     => Ok(vec![ValType::I64]),
        Type::Float   => Ok(vec![ValType::F64]),
        Type::Bool    => Ok(vec![ValType::I32]),
        Type::Str     => Err(WasmCodegenError::UnsupportedType(
            "String as return type".into())),
        other         => Err(WasmCodegenError::UnsupportedType(
            format!("{:?}", other))),
    }
}

/// パラメータ用（String は(i32,i32)ペアで渡せる）
fn favnir_type_to_wasm_params(ty: &Type) -> Result<Vec<ValType>, WasmCodegenError> {
    match ty {
        Type::Str  => Ok(vec![ValType::I32, ValType::I32]),  // (ptr, len)
        other      => favnir_type_to_wasm_results(other),
    }
}
```

### wasm-encoder の使い方

```rust
use wasm_encoder::{
    Module, TypeSection, FunctionSection, CodeSection, ExportSection,
    ImportSection, MemorySection, DataSection, EntityType,
    Function, Instruction, ValType, MemoryType, ExportKind,
    BlockType,
};
```

### IRExpr → WASM instruction 対応

| IRExpr | WASM instructions |
|---|---|
| `Lit(Int(n))` | `i64.const n` |
| `Lit(Float(f))` | `f64.const f` |
| `Lit(Bool(true))` | `i32.const 1` |
| `Lit(Bool(false))` | `i32.const 0` |
| `Lit(Unit)` | なし（スタックに積まない） |
| `Lit(Str(s))` | `i32.const <offset>`, `i32.const <len>` |
| `Local(idx)` | `local.get idx` |
| `BinOp(Add, l, r)` on Int | emit(l), emit(r), `i64.add` |
| `BinOp(Sub, l, r)` on Int | emit(l), emit(r), `i64.sub` |
| `BinOp(Mul, l, r)` on Int | emit(l), emit(r), `i64.mul` |
| `BinOp(Div, l, r)` on Int | emit(l), emit(r), `i64.div_s` |
| `BinOp(Add, l, r)` on Float | emit(l), emit(r), `f64.add` |
| `BinOp(Eq/Ne/Lt/Gt/Le/Ge)` on Int | emit(l), emit(r), 対応 `i64.*` 命令 |
| `BinOp(Eq/Ne/Lt/Gt/Le/Ge)` on Float | emit(l), emit(r), 対応 `f64.*` 命令 |
| `If(cond, then, else_, ty)` | emit(cond), `if <block_type>`, emit(then), `else`, emit(else_), `end` |
| `Block(stmts, final_expr)` | 各 stmt を emit, 最後に final_expr を emit |
| `Call(Global(idx), args)` | 各 arg を emit, `call $wasm_fn_idx` |
| 対応外 | `Err(WasmCodegenError::UnsupportedExpr(...))` |

> **注意**: `if/then/else` の `BlockType` は結果型を指定する必要がある。
> `favnir_type_to_wasm_results(ty)` で取得した型から `BlockType` を構築する。

### IRStmt → WASM instruction

| IRStmt | WASM instructions |
|---|---|
| `Bind(local_idx, expr)` | emit(expr), `local.set local_idx` |
| `Expr(expr)` | emit(expr), （Unit でなければ `drop`） |
| `Chain(...)` | Err(W002) |
| `Yield(...)` | Err(W002) |

### ローカル変数の型収集

WASM は関数先頭で全ローカルを宣言する必要がある。
IR の `Local(idx, ty)` を収集して型マップを作る:

```rust
fn collect_local_types(expr: &IRExpr, map: &mut HashMap<u16, Type>) {
    match expr {
        IRExpr::Local(idx, ty) => { map.entry(*idx).or_insert_with(|| ty.clone()); }
        // 再帰...
    }
}
```

ただし **パラメータ** もローカルに含まれる（idx 0..param_count-1）。
パラメータは `IRFnDef` の型情報から取り、残りを local として宣言する。

### WasmCodegenCtx 構造体

```rust
struct WasmCodegenCtx<'a> {
    globals: &'a [IRGlobal],
    fns: &'a [IRFnDef],
    /// fn_idx（IR）→ WASM function index（import 分のオフセットを含む）
    fn_to_wasm_idx: HashMap<usize, u32>,
    /// builtin 名 → WASM import function index
    builtin_to_wasm_idx: HashMap<String, u32>,
    /// 文字列リテラル → data section のオフセット
    str_to_offset: HashMap<String, u32>,
}
```

---

## Phase 2: 文字列 + IO ホスト import

### 文字列の扱い

WASM linear memory の先頭から文字列リテラルを配置する。
同じ文字列は一度だけ（intern）。

```
data section:
[offset 0 ] "Hello, Favnir!" (14 bytes)
[offset 14] "done"           (4 bytes)
...
```

`Lit(Str(s))` の emit:
```rust
let offset = ctx.str_to_offset[s];
let len = s.len() as i32;
func.instruction(&Instruction::I32Const(offset as i32));
func.instruction(&Instruction::I32Const(len));
```

### WASM 専用ビルトイン: IO.println_int 等

`Debug.show(T)` は String を返すため WASM では W002。
代わりに以下の WASM 専用プリント関数を設ける:

| Favnir 呼び出し（WASM 専用） | host import | 引数 |
|---|---|---|
| `IO.println(s: String)` | `fav_host::io_println` | (i32, i32) |
| `IO.print(s: String)` | `fav_host::io_print` | (i32, i32) |
| `IO.println_int(n: Int)` | `fav_host::io_println_int` | i64 |
| `IO.println_float(f: Float)` | `fav_host::io_println_float` | f64 |
| `IO.println_bool(b: Bool)` | `fav_host::io_println_bool` | i32 |

これらは IR の `FieldAccess(Global("IO"), "println_int")` 等として現れる。
コンパイラが対応するビルトインを登録しておく。

> **実装上の注意**: `IO.println_int` 等は WASM target のときだけ有効なビルトイン。
> `.fvc` ビルド時には `compiler.rs` が未知のビルトインとしてエラーを出す。
> → 対応: `compiler.rs` に `IO.println_int` 等を登録し、`vm.rs` でも
>   `Debug.show + IO.println` の組み合わせにフォールバックする。

### ImportSection の組み立て

使用するホスト関数だけを import する（全部列挙しない）:

```rust
fn collect_used_builtins(ir: &IRProgram) -> HashSet<String> {
    // IR を全走査して FieldAccess(Global("IO"), "println") 等を収集
}
```

WASM function index は import が先頭に来るため、
user function の index = import_count + fn_idx になる。

### wasmtime ホスト関数の登録（wasm_exec.rs）

```rust
fn register_host_functions(linker: &mut Linker<()>) -> Result<(), String> {
    linker.func_wrap("fav_host", "io_println", |mut caller: Caller<'_, ()>, ptr: i32, len: i32| {
        let mem = caller.get_export("memory").and_then(|e| e.into_memory()).unwrap();
        let data = mem.data(&caller);
        let s = std::str::from_utf8(&data[ptr as usize..(ptr + len) as usize])
            .unwrap_or("<invalid utf8>");
        println!("{}", s);
    }).map_err(|e| e.to_string())?;

    linker.func_wrap("fav_host", "io_println_int", |_: Caller<'_, ()>, n: i64| {
        println!("{}", n);
    }).map_err(|e| e.to_string())?;

    linker.func_wrap("fav_host", "io_println_float", |_: Caller<'_, ()>, f: f64| {
        println!("{}", f);
    }).map_err(|e| e.to_string())?;

    linker.func_wrap("fav_host", "io_println_bool", |_: Caller<'_, ()>, b: i32| {
        println!("{}", if b != 0 { "true" } else { "false" });
    }).map_err(|e| e.to_string())?;

    // io_print, io_println_float も同様
    Ok(())
}
```

---

## Phase 3: `fav build --target wasm` CLI

### `src/main.rs` の変更

```rust
Some("build") => {
    let mut out: Option<&str> = None;
    let mut file: Option<&str> = None;
    let mut target: Option<String> = None;
    let mut i = 2usize;
    while i < args.len() {
        match args[i].as_str() {
            "-o"       => { out = Some(&args[i+1]); i += 2; }
            "--target" => { target = Some(args[i+1].clone()); i += 2; }
            other      => { file = Some(other); i += 1; }
        }
    }
    cmd_build(file, out, target.as_deref());
}
```

### `src/driver.rs` の変更

```rust
pub fn cmd_build(file: Option<&str>, out: Option<&str>, target: Option<&str>) {
    match target {
        Some("wasm") => cmd_build_wasm(file, out),
        None | Some("fvc") => cmd_build_fvc(file, out),
        Some(t) => {
            eprintln!("error: unknown target `{}`; valid targets: fvc, wasm", t);
            process::exit(1);
        }
    }
}

fn cmd_build_wasm(file: Option<&str>, out: Option<&str>) {
    use crate::backend::wasm_codegen::wasm_codegen_program;

    let (program, path) = load_and_check_program(file);
    let ir = compile_program(&program);
    let bytes = wasm_codegen_program(&ir).unwrap_or_else(|e| {
        eprintln!("error[{}]: {}", e.code(), e);
        eprintln!("  hint: use `fav build` (without --target wasm) to build a .fvc artifact");
        process::exit(1);
    });
    let out_path = out.map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(&path).with_extension("wasm"));
    std::fs::write(&out_path, &bytes).unwrap_or_else(|e| {
        eprintln!("error: cannot write `{}`: {}", out_path.display(), e);
        process::exit(1);
    });
    println!("built {} (wasm)", out_path.display());
}
```

---

## Phase 4: `fav exec <file.wasm>` via wasmtime

### `cmd_exec` の更新

```rust
pub fn cmd_exec(path: &str, show_info: bool, db_path: Option<&str>) {
    if path.ends_with(".wasm") {
        // W004: --db は .wasm では使えない
        if db_path.is_some() {
            eprintln!("error[W004]: --db cannot be used with .wasm artifacts");
            eprintln!("  Db effect is not supported in WASM MVP (v0.9.0)");
            process::exit(1);
        }
        let bytes = std::fs::read(path).unwrap_or_else(|e| {
            eprintln!("error: cannot read `{}`: {}", path, e);
            process::exit(1);
        });
        if show_info {
            println!("{}", wasm_exec_info(&bytes));
            return;
        }
        wasm_exec_main(&bytes).unwrap_or_else(|e| {
            eprintln!("{}", e);
            process::exit(1);
        });
    } else {
        // 既存 .fvc パス
        fvc_exec(path, show_info, db_path);
    }
}
```

### `wasm_exec_main` の実装（wasm_exec.rs）

```rust
pub fn wasm_exec_main(bytes: &[u8]) -> Result<(), String> {
    let engine = Engine::default();
    let module = Module::new(&engine, bytes)
        .map_err(|e| format!("error: invalid .wasm module: {}", e))?;
    let mut store = Store::new(&engine, ());
    let mut linker = Linker::new(&engine);
    register_host_functions(&mut linker)
        .map_err(|e| format!("error: linker setup failed: {}", e))?;
    let instance = linker.instantiate(&mut store, &module)
        .map_err(|e| format!("error: WASM instantiate failed: {}", e))?;
    let main = instance
        .get_typed_func::<(), ()>(&mut store, "main")
        .map_err(|_| "error: .wasm does not export `main` (expected `() -> Unit`)")?;
    main.call(&mut store, ())
        .map_err(|e| format!("error: WASM runtime error: {}", e))
}
```

### `wasm_exec_info` の実装

```
artifact: .wasm
format: WebAssembly binary
exports: main, memory
imports: fav_host::io_println, fav_host::io_println_int
memory: 1 page (64 KB)
```

`wasmparser` か `wasmtime::Module` の API でメタデータを取得する。

---

## Phase 5: テスト + examples

### `examples/hello_wasm.fav`

`hello.fav` は `"Hello, Favnir!"` という String リテラルを渡すだけなので、
そのまま WASM コンパイル対象になる:

```fav
public fn main() -> Unit !Io {
    IO.println("Hello, Favnir!")
}
```

### `examples/math_wasm.fav`

Int 演算 + `IO.println_int` を使う WASM 専用の例:

```fav
// math_wasm.fav — WASM コンパイル対象の算術例
fn add(a: Int, b: Int) -> Int { a + b }
fn factorial(n: Int) -> Int {
    if n <= 1 { 1 } else { n * factorial(n - 1) }
}
public fn main() -> Unit !Io {
    IO.println_int(add(1, 2));
    IO.println_int(factorial(5))
}
```

### Rust テスト（wasm_codegen.rs 内）

```rust
#[cfg(test)]
mod tests {
    use super::wasm_codegen_program;
    use crate::middle::compiler::compile_program;
    use crate::frontend::parser::Parser;
    use crate::backend::wasm_exec::wasm_exec_main;

    fn run(src: &str) {
        let prog = Parser::parse_str(src, "test.fav").unwrap();
        let ir = compile_program(&prog);
        let bytes = wasm_codegen_program(&ir).expect("codegen");
        wasm_exec_main(&bytes).expect("exec");
    }

    #[test] fn wasm_hello_world() { run(r#"public fn main() -> Unit !Io { IO.println("ok") }"#) }
    #[test] fn wasm_int_arithmetic() { run(r#"... IO.println_int(21 + 21) ..."#) }
    #[test] fn wasm_if_else() { ... }
    #[test] fn wasm_recursive_factorial() { ... }
    #[test] fn wasm_bool_ops() { ... }
    #[test] fn wasm_w001_string_return() { /* Err(W001) を期待 */ }
    #[test] fn wasm_w002_debug_show() { /* Err(W002) を期待 */ }
}
```

---

## 実装上の注意点

### ローカル変数の型収集

WASM は関数先頭で全ローカルを宣言する必要がある。
パラメータ（idx 0..n-1）と bind で導入されるローカル（idx n..）を区別すること。
`Function::new(locals)` に渡す型リストは **パラメータを除いた** ローカルのみ。

### WASM function index のオフセット

import 関数が先頭に来るため、user function の WASM idx は:

```
wasm_fn_idx = import_count + ir_fn_idx
```

import_count は使用するホスト関数の数（可変）。
`WasmCodegenCtx.fn_to_wasm_idx` で管理する。

### 比較演算と BlockType

`if/then/else` の `BlockType` には結果型を指定する必要がある:

```rust
let result_types = favnir_type_to_wasm_results(ty)?;
let block_type = if result_types.is_empty() {
    BlockType::Empty
} else if result_types.len() == 1 {
    BlockType::Result(result_types[0])
} else {
    // multi-value (String は v0.9.0 非対応なので来ないはず)
    unreachable!()
};
```

### `IO.println_int` のコンパイラ登録

`compiler.rs` に WASM 専用ビルトインを追加する:

```rust
// IO namespace
ctx.register_builtin("IO", "println");
ctx.register_builtin("IO", "println_int");    // WASM 専用
ctx.register_builtin("IO", "println_float");  // WASM 専用
ctx.register_builtin("IO", "println_bool");   // WASM 専用
```

`.fvc` VM 側（`vm.rs`）でも対応するハンドラを追加しておくと、
`.fav` ファイルを `.fvc` でもビルドできて一貫性が保てる:

```rust
"IO.println_int" => { println!("{}", n_as_i64); Ok(VMValue::Unit) }
```

### wasmtime feature flags

```toml
wasmtime = { version = "x", default-features = false, features = ["cranelift", "runtime"] }
```

ビルド時間が問題になる場合は CI では `features = ["winch"]` も検討。
