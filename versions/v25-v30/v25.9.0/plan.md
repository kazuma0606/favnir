# v25.9.0 実装計画 — vm.fav Phase 6（CallNamed 実装）

## 実装フェーズ

---

### Phase 1: Cargo.toml バージョン bump

```toml
version = "25.9.0"
```

---

### Phase 2: vm.fav — Opcode 拡張

`fav/self/vm.fav` の `Opcode` 型に `CallNamed(Int, Int)` を追加:

```favnir
type Opcode =
  | ...
  | Call(Int)
  | CallNamed(Int, Int)   // v25.9.0: fn_name_const_idx, argc
  | Return
  | ...
```

`opcode_to_string` にも追加:
```favnir
CallNamed(ni, ac) => f"CallNamed({ni},{ac})"
```

---

### Phase 3: vm.fav — 5バイトデコーダー追加

`decode_byte_with_u16x2_le`: 連続する 2 つの u16 LE オペランドを読む:

```favnir
fn decode_byte_with_u16x2_le(bytes: Bytes, byte: Int, pc: Int) -> Result<DecodeResult, String> {
  bind r1 <- Bytes.read_u16_le(bytes, pc + 1)
  match r1 {
    ok(op1) => {
      bind r2 <- Bytes.read_u16_le(bytes, pc + 3)
      match r2 {
        ok(op2) => match byte {
          0x56 => Result.ok(DecodeResult { op: CallNamed(op1, op2)  next_pc: pc + 5 })
          _    => Result.err(f"decode_byte_with_u16x2_le: unknown opcode: {byte}")
        }
        err(e) => Result.err(e)
      }
    }
    err(e) => Result.err(e)
  }
}
```

`decode_opcode` に `0x56` アームを追加:
```favnir
0x56 => decode_byte_with_u16x2_le(bytes, byte, pc)
```

---

### Phase 4: vm.fav — vm_execute シグネチャ変更

`vm_execute` に `consts: Int` と `program: Int` を追加（末尾に）:

```favnir
fn vm_execute(bytecode: Bytes, stack: Int, locals: Int, globals: Int,
              consts: Int, program: Int, pc: Int) -> Result<VMVal, String>
```

- `consts`: 現在関数の定数プール。`Mut.list` ハンドル（String VMVal のリスト）
- `program`: 全関数マップ。`Mut.map` ハンドル、キー = fn_name、値 = fn_json_str
  - `fn_json_str` はパースして `{"code": "hex...", "consts": [...]}` を取得

**注意**: `vm_execute` の全再帰呼び出し箇所を更新する（多数あり）。以下の 3 パターンを個別に更新：

| パターン | 変更前 | 変更後 |
|---|---|---|
| 通常（大多数） | `vm_execute(bytecode, stack, locals, globals, dec.next_pc)` | `vm_execute(bytecode, stack, locals, globals, consts, program, dec.next_pc)` |
| Jump ハンドラ | `vm_execute(bytecode, stack, locals, globals, dec.next_pc + off)` | `vm_execute(bytecode, stack, locals, globals, consts, program, dec.next_pc + off)` |
| JumpIfFalse 2箇所 | 同上（`dec.next_pc` / `dec.next_pc + off` の 2 バリアント） | それぞれ更新 |
| vm_run 初回 | `vm_execute(bytecode, stack, locals, globals, 0)` | 別途更新（consts/program を空で作成して渡す） |

**HIGH-3 対応**: `Jump` / `JumpIfFalse` ハンドラは `dec.next_pc + off` パターンを使うため、一括置換対象外。個別に更新する。T3 完了後すぐに `cargo build` で型エラーがないことを確認する（T3.5）。

---

### Phase 5: vm.fav — CallNamed ハンドラ実装

**HIGH-2 対応**: `Mut.str_map` / `Mut.get_str` は vm.rs に存在しない。
`program` は `Mut.map`（整数キー）として実装できないため、**線形検索パターン**を主パスとする:
- `program_keys: Int` (Mut.list) — fn_name 文字列のリスト
- `program_vals: Int` (Mut.list) — fn_json 文字列のリスト（インデックス対応）
- `find_fn_in_program(keys, vals, fn_name, i) -> Result<String, String>` — 再帰線形検索

`vm_execute` のシグネチャは以下に変更:
```favnir
fn vm_execute(bytecode: Bytes, stack: Int, locals: Int, globals: Int,
              consts: Int, prog_keys: Int, prog_vals: Int, pc: Int) -> Result<VMVal, String>
```

`CallNamed` ハンドラ（線形検索パターン）:

```favnir
CallNamed(name_idx, argc) => {
  // 1. 現在関数の定数から関数名を取得
  bind name_r <- Mut.get(consts, name_idx)
  match name_r {
    err(e) => Result.err(f"CallNamed: consts[{name_idx}] not found: {e}")
    ok(name_val) => match name_val {
      VMStr(fn_name) => {
        // 2. prog_keys/vals から対象関数の JSON を線形検索
        bind fn_json_r <- find_fn_in_program(prog_keys, prog_vals, fn_name, 0)
        match fn_json_r {
          err(e) => Result.err(f"CallNamed: function '{fn_name}' not in program: {e}")
          ok(fn_json) => {
            // 3. fn_json をパースして code と consts を取得
            bind parsed <- parse_fn_json(fn_json)
            match parsed {
              err(e) => Result.err(f"CallNamed: parse fn_json: {e}")
              ok(fn_def) => {
                // 4. argc 個の引数収集
                bind args_r <- collect_args(stack, argc)
                match args_r {
                  err(e) => Result.err(e)
                  ok(args) => {
                    // 5. fresh locals (args を順に設定)
                    bind fresh_locals <- Mut.map()
                    bind _ <- copy_args_to_locals(args, fresh_locals)
                    // 6. bytes デコード
                    bind bytes_r <- Bytes.from_hex(fn_def.code)
                    match bytes_r {
                      err(e) => Result.err(f"CallNamed: from_hex: {e}")
                      ok(fn_bytes) => {
                        // 7. target 関数の consts list を構築
                        bind fn_consts <- build_consts_list(fn_def.consts)
                        // 8. 再帰実行（線形検索パターン: prog_keys, prog_vals を渡す）
                        bind result_r <- vm_execute(fn_bytes, stack, fresh_locals, globals, fn_consts, prog_keys, prog_vals, 0)
                        match result_r {
                          err(e) => Result.err(e)
                          ok(v) => {
                            bind push_r <- Mut.push(stack, v)
                            match push_r {
                              err(e) => Result.err(e)
                              ok(_)  => vm_execute(bytecode, stack, locals, globals, consts, prog_keys, prog_vals, dec.next_pc)
                            }
                          }
                        }
                      }
                    }
                  }
                }
              }
            }
          }
        }
      }
      _ => Result.err(f"CallNamed: constants[{name_idx}] is not VMStr")
    }
  }
}
```

**補助関数**:
- `parse_fn_json(json: String) -> Result<FnDef, String>` — `{"code":..., "consts":[...]}` をパース（単純文字列解析）
- `copy_args_to_locals(args: Int, locals: Int) -> Result<Unit, String>` — args リストの各要素を locals に番号順で設定
- `build_consts_list(consts_json: String) -> Result<Int, String>` — JSON 配列文字列 → `Mut.list` ハンドル（単純 `,` 分割）
- `find_fn_in_program(keys: Int, vals: Int, name: String, i: Int) -> Result<String, String>` — 線形検索（HIGH-2 対応）
- `build_program_lists(json: String) -> Result<(Int, Int), String>` — JSON → prog_keys, prog_vals 構築

**FnDef 型**:
```favnir
type FnDef = {
  code:   String
  consts: String  // JSON 配列文字列 "[\"fn1\", \"fn2\"]"
}
```

---

### Phase 6: vm.fav — vm_run_program 新エントリポイント

```favnir
// program_json 例:
// {"main":{"code":"hex...","consts":["helper"]},"helper":{"code":"hex...","consts":[]}}
fn vm_run_program(program_json: String) -> Result<VMVal, String> {
  bind prog_lists <- build_program_lists(program_json)  // prog_keys, prog_vals
  let prog_keys = prog_lists.first
  let prog_vals = prog_lists.second
  bind main_json_r <- find_fn_in_program(prog_keys, prog_vals, "main", 0)
  match main_json_r {
    err(e) => Result.err(f"vm_run_program: no 'main' function: {e}")
    ok(main_json) => {
      bind parsed <- parse_fn_json(main_json)
      match parsed {
        err(e) => Result.err(e)
        ok(main_def) => {
          bind bytes_r <- Bytes.from_hex(main_def.code)
          match bytes_r {
            err(e) => Result.err(f"vm_run_program: from_hex: {e}")
            ok(bytes) => {
              bind stack  <- Mut.list()
              bind locals <- Mut.map()
              bind globals <- Mut.map()
              bind consts <- build_consts_list(main_def.consts)
              vm_execute(bytes, stack, locals, globals, consts, prog_keys, prog_vals, 0)
            }
          }
        }
      }
    }
  }
}
```

**補助関数**:
- `build_program_lists(json: String) -> Result<(Int, Int), String>` — JSON → (prog_keys, prog_vals) 線形テーブル（HIGH-2 対応）

---

### Phase 7: driver.rs — build_vm_program_json

FvcArtifact を program JSON にシリアライズする関数:

```rust
pub fn build_vm_program_json(artifact: &FvcArtifact) -> String {
    // artifact.fns: Vec<FnBytecode>
    // artifact.globals: Vec<IRGlobal> — name, kind (0=CompiledFn)
    // 各 FnBytecode: code (Vec<u8>), constants (Vec<Constant>)
    //
    // 出力:
    // {"main": {"code": "hex...", "consts": ["fn_name", ...]}, ...}
    let mut map = serde_json::Map::new();
    for global in &artifact.globals {
        if global.kind == 0 {  // CompiledFn
            let fn_def = &artifact.fns[global.fn_idx];
            let code_hex = fn_def.code.iter().map(|b| format!("{:02x}", b)).collect::<String>();
            let consts_arr: Vec<serde_json::Value> = fn_def.constants.iter()
                .filter_map(|c| match c { Constant::Str(s) => Some(serde_json::Value::String(s.clone())), _ => None })
                .collect();
            let fn_obj = serde_json::json!({ "code": code_hex, "consts": consts_arr });
            map.insert(global.name.clone(), fn_obj);
        }
    }
    serde_json::Value::Object(map).to_string()
}
```

**注意**: `CallNamed` が使う `name_const_idx` は `Constant::Str` のみを対象とするため、
`filter_map` で `Constant::Str` だけを抽出する。インデックスが他の Constant 型（Int 等）と
混在する場合、ズレが生じる。実際は全 `Constant` を JSON 変換する（`Int` も文字列として格納）。

**修正（HIGH-1対応）**: `Constant` は `Int / Float / Str / Name` の 4 バリアント。`CallNamed` が参照するのは `Constant::Name`（関数名）であり `Constant::Str` ではない。全バリアントを変換する:
```rust
let consts_arr: Vec<serde_json::Value> = fn_def.constants.iter()
    .map(|c| match c {
        Constant::Int(n)   => serde_json::Value::String(format!("{}", n)),
        Constant::Float(f) => serde_json::Value::String(format!("{}", f)),
        Constant::Str(s)   => serde_json::Value::String(s.clone()),
        Constant::Name(s)  => serde_json::Value::String(s.clone()),
    })
    .collect();
```

**注意**: `FvcArtifact` の実際の API は `fn_idx_by_name(&name)` メソッドが存在する（vm.rs 行 1844 付近）。実装前に `FvcArtifact` / `FvcGlobal` / `FvcFunction` の正確なフィールド名を vm.rs の定義から確認すること（plan.md の `IRGlobal` / `artifact.fns` / `global.kind == 0` は仮称）。

---

### Phase 8: driver.rs — run_via_vm + CLI

**MED-2 対応**: JSON を Favnir 文字列リテラルとして埋め込む場合、`\r`, `\t`, `{`, `}` 等の
エスケープが複雑になる。既存の `run_with_vm(vm_src, bytecode_hex, globals_entries)` パターンに倣い、
program_json を globals のひとつとして渡す設計を採用する:

```rust
pub fn run_via_vm(vm_src: &str, program_json: &str) -> String {
    // globals[0] = VMStr(program_json) として渡し、
    // vm.fav のラッパーで LoadGlobal(0) → vm_run_program で受け取る
    let wrapper = r#"
  bind globals <- Mut.map()
  // ※ globals[0] に program_json が設定済み（run_with_vm パターン）
  bind prog_r <- Mut.get(globals, 0)
  match prog_r {
    err(e) => e
    ok(prog_val) => match prog_val {
      VMStr(program_json) => {
        bind result <- vm_run_program(program_json)
        match result {
          ok(v)  => vmval_display(v)
          err(e) => e
        }
      }
      _ => "run_via_vm: globals[0] is not VMStr"
    }
  }
"#;
    // 既存 run_with_vm API に program_json を globals[0] として渡す
    run_with_vm(vm_src, "16", &[(0usize, program_json)])  // "16" = Return（ダミー）
}
```

**注意**: `run_with_vm` の `bytecode_hex` は vm.fav 本体が `vm_run_program` を呼ぶラッパーコードに
差し替えるため、ダミーとして `"16"` (Return opcode) を渡す。実際には vm.fav のソース末尾に
`vm_run_program` を呼ぶエントリを追記する形で実装する。
```

**main.rs に追加** (`fav run --vm <path> --compile <src_path>`):
```
if let Some(compile_pos) = args.iter().position(|a| a == "--compile") {
    let src_path = args[compile_pos + 1].as_str();
    let src = std::fs::read_to_string(src_path)?;
    let program = parse(&src);
    let artifact = build_artifact(&program);
    let program_json = build_vm_program_json(&artifact);
    let vm_src = std::fs::read_to_string(vm_path)?;
    let result = run_via_vm(&vm_src, &program_json);
    println!("{}", result);
}
```

---

### Phase 9: v259000_tests (7 件)

```rust
mod v259000_tests {
    fn vm_fav_has_call_named_opcode()       // vm.fav に CallNamed(Int, Int) あり
    fn vm_fav_has_vm_run_program()          // vm.fav に vm_run_program あり
    fn vm_fav_decoder_handles_0x56()        // vm.fav に 0x56 => decode_... あり
    fn build_vm_program_json_smoke()        // build_vm_program_json がパニックしない（hello.fav）
    fn run_via_vm_correct_result()          // multi-function プログラムを vm.fav 経由で実行し正しい値を返す
                                            // ↑ LOW-2 対応: helper_fn(x) = x + x を呼ぶ最小プログラム
                                            //   で返り値が "6" であることを assert
    fn changelog_has_v25_9_0()             // CHANGELOG.md に [v25.9.0] あり
    fn benchmark_v25_9_0_exists()          // benchmarks/v25.9.0.json あり
}
```

---

## 技術メモ

### Mut.str_map は存在しない（HIGH-2 確認済み）
`Mut.str_map()` / `Mut.str_get` / `Mut.str_set` は vm.rs に存在しない（spec-reviewer grep 確認済み）。
**線形検索パターンを主パスとして採用**:
- `prog_keys: Int` (Mut.list) — fn_name 文字列リスト
- `prog_vals: Int` (Mut.list) — fn_json 文字列リスト（インデックス対応）
- `find_fn_in_program(keys, vals, fn_name, i)` — 再帰的線形検索

### vm_execute 再帰呼び出し更新パターン一覧（HIGH-3）
`vm_execute` のシグネチャに `consts: Int, prog_keys: Int, prog_vals: Int` を追加するため、
ファイル内の全再帰呼び出しを更新する。特殊パターンに注意:

| パターン | 箇所 |
|---|---|
| 通常 (`dec.next_pc`) | ≈ 35 箇所（一括置換可） |
| Jump (`dec.next_pc + off`) | vm.fav 行 415 付近 |
| JumpIfFalse × 2 | vm.fav 行 423, 424 付近 |
| vm_run 初回呼び出し | vm.fav 行 736, 742（空リストで consts/prog を渡す） |

T3 完了直後に `cargo build` を実行して漏れを検出する（T3.5 として tasks.md に追加）。

### Constant::Name（HIGH-1 確認済み）
`callNamed` の `name_const_idx` は `Constant::Name(fn_name)` を指す（`Constant::Str` ではない）。
vm.rs 行 2868 の `Constant::Name(name) => name.clone()` が証拠。
`build_vm_program_json` は 4 バリアント（Int / Float / Str / Name）を全変換する。

### FvcArtifact 構造確認（MED-1）
`build_vm_program_json` の実装前に、以下のフィールド・メソッド名を vm.rs から確認:
- `artifact.fns` / `artifact.functions` どちらか
- `global.kind == 0` / `global.fn_idx` の実際のフィールド名
- `fn_idx_by_name(&name)` の利用可否
- `FnBytecode.constants: Vec<Constant>` の実際の型名

### run_via_vm の JSON 渡し方（MED-2）
文字列リテラル埋め込みによるエスケープ問題を避けるため、`run_with_vm` の `globals_entries` パターンを使用。
program_json を globals[0] に `VMStr` として渡し、vm.fav 側の `vm_run_program_entry` が
`LoadGlobal(0)` で取り出して `vm_run_program` を呼ぶ。

### parse_fn_json の制約
`Json.get_arr_field` 等は存在しないため、単純文字列解析で実装。
対象: compiler.fav が出力する関数名（アルファベット・ドット・アンダースコアのみ）。
`"` / `,` を含む関数名は想定外のため、エラーハンドリング不要（範囲外）。
