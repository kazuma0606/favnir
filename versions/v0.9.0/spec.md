# Favnir v0.9.0 仕様書 — WASM Backend

更新日: 2026-04-30（Codex レビュー反映）

---

## 概要

v0.9.0 のテーマは **WASM backend**。

現在の Favnir は `.fvc` バイトコード + 独自スタック VM で動く。
v0.9.0 では typed IR から WebAssembly binary を生成し、
wasmtime を使って実行できるようにする。

目標はポータビリティと sandbox 実行。
`.wasm` artifact は browser・edge・任意の WASM ランタイムで動かせる。

---

## 追加する機能

### 1. `fav build --target wasm`

```
fav build --target wasm [-o <file.wasm>] [file]
```

- 型検査 → IR コンパイル → WASM codegen の順に処理
- 出力は `.wasm` バイナリ（デフォルト名: `<stem>.wasm`）
- `--target` を省略した場合は従来どおり `.fvc` を生成

### 2. `fav exec <file.wasm>`

```
fav exec [--info] <artifact>
```

- `.fvc` と `.wasm` の両方に対応（拡張子で自動判別）
- `.wasm` の場合は wasmtime エンジンで実行
- `--info` で WASM モジュールのメタデータを表示
- **`--db` は `.wasm` に対して使用不可**（明示エラー）。Db effect は v0.9.0 では非対応。

### 3. WASM codegen モジュール

```
src/backend/wasm_codegen.rs
```

- `fn wasm_codegen_program(ir: &IRProgram) -> Result<Vec<u8>, WasmCodegenError>`
- `wasm-encoder` crate で WASM binary を組み立てる

### 4. WASM executor モジュール

```
src/backend/wasm_exec.rs
```

- `fn wasm_exec_main(bytes: &[u8]) -> Result<(), String>`
- `wasmtime` crate でホスト関数を登録し `main` を呼び出す
- 戻り値は `()` のみ（→ `main` は `Unit !Io` 専用、下記参照）

---

## サポートする Favnir サブセット（v0.9.0 MVP）

WASM codegen が対応するのは以下のサブセット。
対応外の構造を含む関数をコンパイルしようとした場合は `WasmCodegenError` を返す。

### エントリポイントの制約

**WASM でコンパイルできる `main` は以下のシグネチャに限定する**:

```fav
public fn main() -> Unit !Io
```

- 引数なし・戻り値 `Unit` のみサポート
- WASM export `"main"` は `() -> ()` (void)
- 他のシグネチャ（例: `-> Int`）の `main` は W003 エラー

### 対応する型（パラメータ・戻り値別）

| Favnir 型 | WASM 型 | パラメータ | 戻り値 | 備考 |
|---|---|---|---|---|
| `Int` | `i64` | ✓ | ✓ | 符号付き 64 bit |
| `Float` | `f64` | ✓ | ✓ | IEEE 754 倍精度 |
| `Bool` | `i32` | ✓ | ✓ | 0 = false, 1 = true |
| `Unit` | なし | ✓ | ✓ | 戻り値なし（void） |
| `String` | `(i32, i32)` | ✓（リテラルのみ） | **✗** | 戻り値は W001 エラー |

> **String の扱い**: WASM は multi-value return を持つが、String を関数戻り値として扱うと
> `Vec<ValType>` の管理が複雑になる。v0.9.0 MVP では String を**戻り値に使えない**制約を設ける。
> 文字列リテラルを `IO.println` 等のホスト関数に渡すことはできる。

### 対応する式・文

| 構造 | 備考 |
|---|---|
| 整数リテラル | `i64.const` |
| 浮動小数点リテラル | `f64.const` |
| 真偽値リテラル | `i32.const 0/1` |
| 文字列リテラル | data section に配置; (i32 ptr, i32 len) を emit |
| `Unit` リテラル | 値なし |
| 算術 BinOp (+/-/*//) | `i64.add` / `i64.sub` / `i64.mul` / `i64.div_s` |
| 比較 BinOp (==/ !=/</>/<=/>=) | `i64.eq` 等 → i32 |
| `if cond { } else { }` | WASM `if/then/else` block |
| `bind x <- expr` | WASM `local.set` |
| ローカル変数参照 | `local.get` |
| 直接関数呼び出し `f(a, b)` | `call $fn_idx` |
| IO ビルトイン呼び出し | ホスト import 経由（下記参照） |
| ブロック `{ stmt* expr }` | 順次実行 |

### 対応する定義

| 構造 | 備考 |
|---|---|
| `fn name(params) -> RetTy { body }` | params・戻り値は Int/Float/Bool/Unit のみ（String 戻り値は W001） |
| `public fn main() -> Unit !Io { }` | エントリポイント |

### v0.9.0 では対応しない（→ v1.0.0 以降）

- `String` を関数の戻り値として使う
- `Debug.show(T) -> String`（String を返すため W002）
- `List<T>` / `Map<V>` （ヒープ上のコレクション）
- `type` による record / sum variant
- `Option<T>` / `Result<T, E>`（variant に依存）
- クロージャ（関数テーブルが必要）
- `trf` / `flw`（高階関数機構に依存）
- `chain` / `collect` / `yield`
- `Db`, `Network`, `File` effect

---

## WASM ホスト関数（v0.9.0）

`Debug.show` は String を返すため WASM では非対応（W002）。
代わりに以下の WASM 専用プリント関数を提供する。
これらは `.fvc` VM では通常の IO.println に統合されているが、
WASM 上では専用の host import として実装する。

| Favnir 呼び出し | host import name | パラメータ | 動作 |
|---|---|---|---|
| `IO.println(s)` | `fav_host::io_println` | `(i32 ptr, i32 len)` | 文字列を stdout に出力 + 改行 |
| `IO.print(s)` | `fav_host::io_print` | `(i32 ptr, i32 len)` | 文字列を stdout に出力（改行なし） |
| `IO.println_int(n)` | `fav_host::io_println_int` | `(i64)` | 整数を stdout に出力 + 改行 |
| `IO.println_float(f)` | `fav_host::io_println_float` | `(f64)` | 浮動小数点を stdout に出力 + 改行 |
| `IO.println_bool(b)` | `fav_host::io_println_bool` | `(i32)` | `true`/`false` を stdout に出力 + 改行 |

> `IO.println_int` / `IO.println_float` / `IO.println_bool` は WASM 専用ビルトイン。
> `.fvc` VM では「`Debug.show` → `IO.println`」のパターンを使う。

---

## エラーコード

| コード | 意味 | 例 |
|---|---|---|
| `W001` | WASM 非対応型（戻り値として String など） | `fn greet() -> String` |
| `W002` | WASM 非対応構造（Debug.show, List, closure など） | `Debug.show(42)` |
| `W003` | main のシグネチャが WASM 非対応 | `fn main() -> Int` |
| `W004` | `.wasm` に対して `--db` が指定された | `fav exec --db x.db f.wasm` |

エラー出力形式:

```
error[W001]: WASM codegen does not support String as a return type (fn `greet`)
  hint: use `fav build` (without --target wasm) to build a .fvc artifact
```

---

## WASM モジュール構造

生成する `.wasm` モジュールは以下の構造を持つ:

```
(module
  ;; --- imports (ホスト関数) ---
  (import "fav_host" "io_println"       (func $io_println      (param i32 i32)))
  (import "fav_host" "io_println_int"   (func $io_println_int  (param i64)))
  (import "fav_host" "io_println_float" (func $io_println_float (param f64)))
  (import "fav_host" "io_println_bool"  (func $io_println_bool  (param i32)))

  ;; --- memory ---
  (memory (export "memory") 1)      ;; 64KB

  ;; --- data (string literals) ---
  (data (i32.const 0) "Hello, Favnir!")

  ;; --- user functions ---
  (func $factorial (param i64) (result i64)
    ...)

  ;; --- main ---
  (func $main
    i32.const 0    ;; ptr
    i32.const 14   ;; len
    call $io_println)

  (export "main" (func $main))
)
```

---

## artifact フォーマット拡張

`fav exec` は拡張子で判別する:

| 拡張子 | 実行方式 |
|---|---|
| `.fvc` | 独自スタック VM（`--db` 対応） |
| `.wasm` | wasmtime（`--db` は W004 エラー） |

---

## 完了条件

- `fav build --target wasm examples/hello.fav` で `hello.wasm` が生成できる
- `fav exec hello.wasm` で `Hello, Favnir!` が出力される
- `fav exec --db x.db hello.wasm` が `W004` エラーを出す
- `fav build --target wasm` で String 戻り値の fn が `W001` エラーを出す
- `fav build --target wasm` で Debug.show 呼び出しが `W002` エラーを出す
- `cargo test` が全通過
- `Cargo.toml` バージョンが `"0.9.0"`
