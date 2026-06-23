# v23.7.0 実装計画 — vm.fav Phase 4（stdlib・builtin 呼び出し）

## 前提確認

v23.7.0 は Rust 側変更なし。fav/self/vm.fav の修正 + テスト + ドキュメントのみ。

### 実装前チェック

```bash
grep -n "fn vm_execute\|fn vm_run" fav/self/vm.fav
# → 現在のシグネチャが (bytecode: Bytes, stack: Int, locals: Int, pc: Int) であること

grep -n "vm_execute(bytecode, stack, locals, dec.next_pc)" fav/self/vm.fav | wc -l
# → Phase 1〜3 の再帰呼び出し件数を確認（20〜25 件程度）

grep -n "version = " fav/Cargo.toml
# → "23.6.0" であること

grep -n "mod v236000_tests\|mod v237000_tests" fav/src/driver.rs | head -3
# → v237000_tests が未存在であること

grep -n "VMStr\|call_builtin\|vm_run_named" fav/self/vm.fav
# → 全て 0 件であること（未実装を確認）
```

---

## T0: 事前確認

```bash
# String.length / String.trim が Favnir primitives として使用可能か確認
grep -rn "String\.length\|String\.trim" fav/self/ | head -10
# → checker.fav で使用されていれば利用可能（checker.rs builtin_ret_ty にも登録済み）

# LoadGlobal オペコードバイト確認（0x12）
grep -n "LoadGlobal" fav/src/backend/codegen.rs | head -5
# → "LoadGlobal = 0x12" を確認

# Const(n) のオペランドが直接値であることを確認（定数プールインデックスでないこと）
grep -n "Const\b\|emit_u16\|LoadConst" fav/src/backend/codegen.rs | head -10
# → Phase 2/3 の既存テスト（execute_locals: Const(42) → VMInt(42)）が通ることで実証済みだが念のため確認

# vm-fav.mdx が Phase 3 セクション追記済みであること
grep -n "Phase 3\|Phase 4" site/content/docs/tools/vm-fav.mdx | head -5
```

---

## T1: `fav/self/vm.fav` — Phase 4 変更

### T1-1: `VMVal` 型に `VMStr(String)` を追加

**変更前:**
```favnir
type VMVal =
  | VMInt(Int)
  | VMBool(Bool)
  | VMUnit
```

**変更後:**
```favnir
type VMVal =
  | VMInt(Int)
  | VMBool(Bool)
  | VMUnit
  | VMStr(String)
```

### T1-2: `vmval_to_string` に `VMStr` アームを追加

**変更前:**
```favnir
fn vmval_to_string(v: VMVal) -> String {
  match v {
    VMInt(n)  => f"VMInt({n})"
    VMBool(b) => f"VMBool({b})"
    VMUnit    => "VMUnit"
  }
}
```

**変更後:**
```favnir
fn vmval_to_string(v: VMVal) -> String {
  match v {
    VMInt(n)  => f"VMInt({n})"
    VMBool(b) => f"VMBool({b})"
    VMUnit    => "VMUnit"
    VMStr(s)  => f"VMStr({s})"
  }
}
```

### T1-3: `fn call_builtin` を `vmval_to_string` の直後に追加

```favnir
// vm.fav Phase 4: Favnir ↔ Rust の永続的境界。
// vm.fav からは Favnir 記法で呼び出すが、実行時は Rust VM の vm_call_builtin にディスパッチされる。
// args: Int — Mut.list() の opaque handle（既に積まれた引数を pop して使う）
fn call_builtin(name: String, args: Int) -> Result<VMVal, String> {
  match name {
    "Int.to_string" => {
      bind v_r <- Mut.pop(args)
      match v_r {
        err(e) => Result.err(e)
        ok(v) => match v {
          VMInt(n) => Result.ok(VMStr(f"{n}"))
          _ => Result.err("Int.to_string: expected VMInt")
        }
      }
    }
    "String.length" => {
      bind v_r <- Mut.pop(args)
      match v_r {
        err(e) => Result.err(e)
        ok(v) => match v {
          VMStr(s) => Result.ok(VMInt(String.length(s)))
          _ => Result.err("String.length: expected VMStr")
        }
      }
    }
    "String.trim" => {
      bind v_r <- Mut.pop(args)
      match v_r {
        err(e) => Result.err(e)
        ok(v) => match v {
          VMStr(s) => Result.ok(VMStr(String.trim(s)))
          _ => Result.err("String.trim: expected VMStr")
        }
      }
    }
    "Math.abs" => {
      bind v_r <- Mut.pop(args)
      match v_r {
        err(e) => Result.err(e)
        ok(v) => match v {
          VMInt(n) => if n < 0 {
            Result.ok(VMInt(0 - n))
          } else {
            Result.ok(VMInt(n))
          }
          _ => Result.err("Math.abs: expected VMInt")
        }
      }
    }
    _ => Result.err(f"call_builtin: unknown builtin: {name}")
  }
}
```

> **注意**: `String.length(s)` / `String.trim(s)` が checker.fav で型エラーになる場合:
> - `String.length` の代替: `s` に対して `String.split(s, "")` の件数を数えるなど → 複雑なので、発生したら spec-reviewer に相談
> - `String.trim` の代替: `String.trim(s)` で型エラーなら当 builtin を `_ => Result.err(...)` に一時的に置き換える
> - T0 の事前確認でこれらの利用可否を確認しておくこと

### T1-4: `fn vm_execute` シグネチャに `globals: Int` を追加

**変更前:**
```favnir
fn vm_execute(bytecode: Bytes, stack: Int, locals: Int, pc: Int) -> Result<VMVal, String> {
```

**変更後:**
```favnir
fn vm_execute(bytecode: Bytes, stack: Int, locals: Int, globals: Int, pc: Int) -> Result<VMVal, String> {
```

### T1-5: Phase 1〜3 全再帰呼び出しを一括更新

**検索パターン（更新前）:**
```
vm_execute(bytecode, stack, locals, dec.next_pc)
```

**置換パターン（更新後）:**
```
vm_execute(bytecode, stack, locals, globals, dec.next_pc)
```

更新対象アーム（約 20〜25 件）:
- `ConstUnit`, `ConstTrue`, `ConstFalse`, `Const(n)` — 各 1 件
- `Pop ok` アーム — 1 件
- `Dup ok → push ok` アーム — 1 件
- `Add, Sub, Mul, Eq, Ne, Lt, Le, Gt, Ge, And, Or` — 各 1 件（push ok アーム内）
- `Jump(off)` — 1 件
- `JumpIfFalse true` アーム — 1 件
- `JumpIfFalse false` アーム — 1 件
- `LoadLocal ok → push ok` アーム — 1 件
- `StoreLocal ok → set ok` アーム — 1 件
- `Div ok → push ok` アーム — 1 件

**確認コマンド（更新後）:**
```bash
grep -n "vm_execute(bytecode, stack, locals, dec.next_pc)" fav/self/vm.fav
# → 0 件であること（更新漏れなし）
```

### T1-6: Phase 4 オペコード追加（`_` アームの直前に挿入）

現在の `// LoadGlobal / Call / GetField は Phase 4 以降で実装予定` コメントと
`_ => Result.err("vm_execute: unimplemented opcode")` の間に以下を追加:

```favnir
      LoadGlobal(idx) => {
        bind g_r <- Mut.get(globals, idx)
        match g_r {
          err(e) => Result.err(f"LoadGlobal: {e}")
          ok(v) => {
            bind push_r <- Mut.push(stack, v)
            match push_r {
              err(e) => Result.err(e)
              ok(_)  => vm_execute(bytecode, stack, locals, globals, dec.next_pc)
            }
          }
        }
      }
      Call(argc) => match argc {
        0 => {
          bind callee_r <- Mut.pop(stack)
          match callee_r {
            err(e) => Result.err(e)
            ok(callee) => match callee {
              VMStr(name) => {
                bind args <- Mut.list()
                bind res_r <- call_builtin(name, args)
                match res_r {
                  err(e) => Result.err(e)
                  ok(v) => {
                    bind push_r <- Mut.push(stack, v)
                    match push_r {
                      err(e) => Result.err(e)
                      ok(_)  => vm_execute(bytecode, stack, locals, globals, dec.next_pc)
                    }
                  }
                }
              }
              _ => Result.err("Call(0): callee is not a VMStr")
            }
          }
        }
        1 => {
          bind arg_r <- Mut.pop(stack)
          match arg_r {
            err(e) => Result.err(e)
            ok(arg) => {
              bind callee_r <- Mut.pop(stack)
              match callee_r {
                err(e) => Result.err(e)
                ok(callee) => match callee {
                  VMStr(name) => {
                    bind args <- Mut.list()
                    bind _ <- Mut.push(args, arg)
                    bind res_r <- call_builtin(name, args)
                    match res_r {
                      err(e) => Result.err(e)
                      ok(v) => {
                        bind push_r <- Mut.push(stack, v)
                        match push_r {
                          err(e) => Result.err(e)
                          ok(_)  => vm_execute(bytecode, stack, locals, globals, dec.next_pc)
                        }
                      }
                    }
                  }
                  _ => Result.err("Call(1): callee is not a VMStr")
                }
              }
            }
          }
        }
        _ => Result.err(f"Call: argc={argc} not yet supported in Phase 4")
      }
      // GetField は Phase 5 以降で実装予定
```

> **コメント更新**: 既存コメント `// LoadGlobal / Call / GetField は Phase 4 以降で実装予定` を
> `// GetField は Phase 5 以降で実装予定` に変更すること。

### T1-7: `fn vm_run` 更新 + `fn vm_run_named` 追加

**vm_run 更新（globals を空 Mut.map として生成）:**

```favnir
fn vm_run(bytecode: Bytes) -> Result<VMVal, String> {
  bind stack   <- Mut.list()
  bind locals  <- Mut.map()
  bind globals <- Mut.map()
  vm_execute(bytecode, stack, locals, globals, 0)
}
```

**vm_run_named 追加（vm_run の直後に追加）:**

```favnir
fn vm_run_named(bytecode: Bytes, globals: Int) -> Result<VMVal, String> {
  bind stack  <- Mut.list()
  bind locals <- Mut.map()
  vm_execute(bytecode, stack, locals, globals, 0)
}
```

---

## T1 事後確認

```bash
cargo check --bin fav
# → エラー 0 であること

grep -n "vm_execute(bytecode, stack, locals, dec.next_pc)" fav/self/vm.fav
# → 0 件（更新漏れなし）

grep -n "fn vm_execute\|fn vm_run\|fn vm_run_named\|fn call_builtin\|VMStr" fav/self/vm.fav | head -20
# → 期待するシグネチャが存在すること

# 後方互換性確認: 既存テストが引き続き通ること
cargo test v236000 --bin fav
# → 5/5 PASS であること（vm_run の後方互換性確認）
```

---

## T2: `fav/src/driver.rs` — `v237000_tests` 追加

### T2-1: `v236000_tests::version_is_23_6_0` に `#[ignore]` 追加（T3-1 より前に必須）

```rust
#[test]
#[ignore]
fn version_is_23_6_0() {
```

### T2-2: `v237000_tests` モジュールを `v236000_tests` の直後に追加

```rust
// ── v237000_tests (v23.7.0) — vm.fav Phase 4（stdlib・builtin 呼び出し）──────
#[cfg(test)]
mod v237000_tests {
    use super::*;

    #[test]
    fn version_is_23_7_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("version = \"23.7.0\""), "Cargo.toml should have version 23.7.0");
    }

    #[test]
    fn vm_fav_phase4_compiles() {
        let src = include_str!("../self/vm.fav");
        let tokens = crate::frontend::lexer::Lexer::new(src, "vm.fav")
            .tokenize().expect("lex vm.fav");
        let prog = crate::frontend::parser::Parser::new(tokens)
            .parse_program().expect("parse vm.fav");
        let _artifact = build_artifact(&prog);
    }

    #[test]
    fn vmstr_to_string_variant() {
        // VMStr バリアントが追加され vmval_to_string が正しく動作することを確認
        let vm_src = include_str!("../self/vm.fav");
        let src = format!(r#"{}
public fn main() -> String {{
  vmval_to_string(VMStr("hello"))
}}"#, vm_src);
        let tokens = crate::frontend::lexer::Lexer::new(&src, "vm_test.fav")
            .tokenize().expect("lex");
        let prog = crate::frontend::parser::Parser::new(tokens)
            .parse_program().expect("parse");
        let artifact = build_artifact(&prog);
        let result = exec_artifact_main(&artifact, None).expect("exec");
        assert_eq!(result, crate::value::Value::Str("VMStr(hello)".to_string()),
            "vmval_to_string(VMStr(\"hello\")) should return \"VMStr(hello)\"");
    }

    #[test]
    fn execute_builtin_call() {
        // LoadGlobal(0)="Int.to_string" + Const(42) + Call(1) + Return
        // Bytecode hex: "120000012a0015010016"
        //   12 00 00  LoadGlobal(0)   → push globals[0] = VMStr("Int.to_string")
        //   01 2A 00  Const(42)       → push VMInt(42)
        //   15 01 00  Call(1)         → pop VMInt(42), pop VMStr("Int.to_string")
        //                                → call_builtin → push VMStr("42")
        //   16        Return           → VMStr("42")
        let vm_src = include_str!("../self/vm.fav");
        let src = format!(r#"{}
public fn main() -> String {{
  bind globals <- Mut.map()
  bind _ <- Mut.set(globals, 0, VMStr("Int.to_string"))
  bind hex_r <- Bytes.from_hex("120000012a0015010016")
  match hex_r {{
    ok(bytes) => {{
      bind run_r <- vm_run_named(bytes, globals)
      match run_r {{
        ok(v)  => vmval_to_string(v)
        err(e) => e
      }}
    }}
    err(e) => e
  }}
}}"#, vm_src);
        let tokens = crate::frontend::lexer::Lexer::new(&src, "vm_test.fav")
            .tokenize().expect("lex");
        let prog = crate::frontend::parser::Parser::new(tokens)
            .parse_program().expect("parse");
        let artifact = build_artifact(&prog);
        let result = exec_artifact_main(&artifact, None).expect("exec");
        assert_eq!(result, crate::value::Value::Str("VMStr(42)".to_string()),
            "LoadGlobal+Const+Call(1)+Return should call Int.to_string(42) -> VMStr(42)");
    }

    #[test]
    fn changelog_has_v23_7_0() {
        let cl = include_str!("../../CHANGELOG.md");
        assert!(cl.contains("[v23.7.0]"), "CHANGELOG.md should have [v23.7.0] entry");
    }
}
```

---

## T3: Cargo.toml + CHANGELOG + benchmarks + docs

> **注意**: T2-1 の `#[ignore]` 追加完了後に Cargo.toml を更新すること。

### T3-1: `fav/Cargo.toml` バージョン更新

```
version = "23.6.0" → "23.7.0"
```

### T3-2: `CHANGELOG.md` 先頭に v23.7.0 エントリ追加

```markdown
## [v23.7.0] — 2026-06-22 — vm.fav Phase 4（stdlib・builtin 呼び出し）

### Added
- `vm.fav` Phase 4: stdlib・builtin 呼び出し
  - `VMVal` に `VMStr(String)` バリアントを追加
  - `fn call_builtin(name: String, args: Int) -> Result<VMVal, String>` 実装（4 builtin: Int.to_string / String.length / String.trim / Math.abs）
  - `LoadGlobal(idx)` オペコード: globals マップから値を lookup してスタックに push
  - `Call(0)` / `Call(1)` オペコード: builtin ディスパッチ（Favnir ↔ Rust の永続的境界）
  - `fn vm_run_named(bytecode: Bytes, globals: Int) -> Result<VMVal, String>` 追加

### Changed
- `fn vm_execute` シグネチャ: `(bytecode, stack, locals, pc)` → `(bytecode, stack, locals, globals, pc)`
- `fn vm_run` が空 globals マップを生成するよう更新
```

### T3-3: `benchmarks/v23.7.0.json` 作成

```json
{
  "version": "23.7.0",
  "date": "2026-06-22",
  "test_count": 0,
  "feature": "vm.fav Phase 4（stdlib・builtin 呼び出し）",
  "metrics": {
    "vm_fav_phase": 4,
    "call_builtin_count": 4,
    "vm_execute_params": 5,
    "new_opcodes": 2,
    "vmval_variants": 4
  }
}
```

### T3-4: `site/content/docs/tools/vm-fav.mdx` に Phase 4 セクション追記

フェーズ表を更新:
```
| Phase 4（v23.7.0） | builtin ディスパッチ（LoadGlobal / Call / call_builtin）・VMStr 追加 |
```

---

## 実装順序

```
T0（事前確認）
  └─ String.length / String.trim の可用性確認
T1-1（VMStr 追加）
T1-2（vmval_to_string 更新）
T1-3（call_builtin 追加）
T1-4（vm_execute シグネチャ変更）
T1-5（再帰呼び出し一括更新）← 更新漏れを cargo check で検出
T1-6（LoadGlobal / Call 追加）
T1-7（vm_run 更新 + vm_run_named 追加）
cargo check → エラー 0 確認
T2-1（#[ignore]）← T3-1 より前に必須
T2-2（tests）   ← T1 完了後
cargo test v237000 → 5/5 PASS 確認
T3-1（version） ← T2-1 完了後
T3-2〜4（docs） ← T3-1 完了後
cargo test --bin fav → リグレッションなし確認
```

---

## リスク対応表

| リスク | 検出方法 | 対応 |
|---|---|---|
| 再帰呼び出しの更新漏れ | `cargo check` でシグネチャ不一致エラー | `grep -n "vm_execute(bytecode, stack, locals, dec.next_pc)" fav/self/vm.fav` で残りを確認 |
| `String.length(s)` / `String.trim(s)` が checker.fav で型エラー | コンパイル失敗 or 型エラー | `String.length` → `Bytes.to_utf8` 迂回は不可。`_ => Result.err("not implemented")` で一時無効化し後続フェーズで対応 |
| `f"{n}"` が空文字列を返す | テスト失敗 | `Int.to_string(n)` (vm_call_builtin 経由) に変更。ただし checker.fav への登録を先に確認 |
| `Call(argc)` の `match argc { 0 => ... 1 => ... _ => ... }` が Int パターン不可 | パースエラー | Favnir は整数リテラルの match をサポート（v23.6.0 の `match bi { 0 => ... _ => ... }` で確認済み） |
| `Mut.push(args, arg)` で push 後の順序 | テスト失敗 | Call(1) は arg を 1 件だけ push → call_builtin が pop → 順序問題なし |
| `Mut.set(globals, 0, VMStr(...))` のキー型が LoadGlobal と不一致 | テスト失敗（LoadGlobal が globals[0] を取れない） | Mut.map のキーは VMValue::Int(n) として照合。`0` は Int リテラル → VMValue::Int(0)。LoadGlobal(0) の idx も VMValue::Int(0) として照合されるため一致する（locals の Mut.set と同じパターン） |
