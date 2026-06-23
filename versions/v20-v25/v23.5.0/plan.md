# v23.5.0 実装計画 — vm.fav Phase 2

## 前提確認

v23.5.0 は Rust 側変更なし。Favnir コード追加 + テスト + ドキュメントのみ。

### 実装前チェック

```bash
grep -n "Mut\.push\|Mut\.pop\|Mut\.peek\|Mut\.len" fav/src/backend/vm.rs | head -10
# → すべて実装済みであること（v23.3.0）

grep -n "version = " fav/Cargo.toml
# → "23.4.0" であること（T3-1 で "23.5.0" に更新）

grep -n "mod v234000_tests\|mod v235000_tests" fav/src/driver.rs | head -5
# → v235000_tests が未存在であること
```

---

## T0: 事前確認 — `bind _` パターンのサポート

```bash
grep -n "Pat::Wildcard\|Wildcard\|\"_\"" fav/src/frontend/parser.rs | head -10
```

`_` が bind パターンとして使える場合: そのまま実装。

> 注: v23.4.0 の調査では `bind r <- E` 形式が確認済み。`bind _ <- E` が使えない場合は
> `bind _discard <- Mut.push(...)` など named pattern で代替する。

---

## T1: `fav/self/vm.fav` — Phase 2 追記

vm.fav のファイル末尾（`opcode_to_string` 以降）に以下を追記する。

### T1-1: `type VMVal` + `fn vmval_to_string`

```favnir
// v23.5.0: vm.fav Phase 2（スタックベース実行ループ）
//
// 設計ノート:
//   - stack は Mut.list() が返す opaque handle。Favnir 型宣言は Int。
//     (Type::Unknown は型チェッカーで全型に unify するため Int 宣言で問題なし)
//   - Mut.push は Result<Unit, String> を返すため bind _ <- Mut.push(...) で sequencing。
//   - Const(n): Phase 2 では u16 オペランドを整数値として push（定数プール省略）。

type VMVal =
  | VMInt(Int)
  | VMBool(Bool)
  | VMUnit

fn vmval_to_string(v: VMVal) -> String {
  match v {
    VMInt(n)  => f"VMInt({n})"
    VMBool(b) => f"VMBool({b})"
    VMUnit    => "VMUnit"
  }
}
```

### T1-2: `fn vm_execute`

```favnir
fn vm_execute(bytecode: Bytes, stack: Int, pc: Int) -> Result<VMVal, String> {
  bind dec_r <- decode_opcode(bytecode, pc)
  match dec_r {
    err(e) => Result.err(e)
    ok(dec) => match dec.op {
      ConstUnit => {
        bind _ <- Mut.push(stack, VMUnit)
        vm_execute(bytecode, stack, dec.next_pc)
      }
      ConstTrue => {
        bind _ <- Mut.push(stack, VMBool(true))
        vm_execute(bytecode, stack, dec.next_pc)
      }
      ConstFalse => {
        bind _ <- Mut.push(stack, VMBool(false))
        vm_execute(bytecode, stack, dec.next_pc)
      }
      Const(n) => {
        bind _ <- Mut.push(stack, VMInt(n))
        vm_execute(bytecode, stack, dec.next_pc)
      }
      Pop => {
        bind _ <- Mut.pop(stack)
        vm_execute(bytecode, stack, dec.next_pc)
      }
      Dup => {
        bind top_r <- Mut.peek(stack)
        match top_r {
          err(e) => Result.err(e)
          ok(v) => {
            bind _ <- Mut.push(stack, v)
            vm_execute(bytecode, stack, dec.next_pc)
          }
        }
      }
      Return => {
        bind top_r <- Mut.pop(stack)
        match top_r {
          ok(v)  => Result.ok(v)
          err(e) => Result.err(f"Return: {e}")
        }
      }
      Add => {
        bind b_r <- Mut.pop(stack)
        bind a_r <- Mut.pop(stack)
        match b_r {
          err(e) => Result.err(e)
          ok(b) => match a_r {
            err(e) => Result.err(e)
            ok(a) => match a {
              VMInt(ai) => match b {
                VMInt(bi) => {
                  bind _ <- Mut.push(stack, VMInt(ai + bi))
                  vm_execute(bytecode, stack, dec.next_pc)
                }
                _ => Result.err("Add: type error")
              }
              _ => Result.err("Add: type error on a")
            }
          }
        }
      }
      Sub => {
        bind b_r <- Mut.pop(stack)
        bind a_r <- Mut.pop(stack)
        match b_r {
          err(e) => Result.err(e)
          ok(b) => match a_r {
            err(e) => Result.err(e)
            ok(a) => match a {
              VMInt(ai) => match b {
                VMInt(bi) => {
                  bind _ <- Mut.push(stack, VMInt(ai - bi))
                  vm_execute(bytecode, stack, dec.next_pc)
                }
                _ => Result.err("Sub: type error")
              }
              _ => Result.err("Sub: type error on a")
            }
          }
        }
      }
      Mul => {
        bind b_r <- Mut.pop(stack)
        bind a_r <- Mut.pop(stack)
        match b_r {
          err(e) => Result.err(e)
          ok(b) => match a_r {
            err(e) => Result.err(e)
            ok(a) => match a {
              VMInt(ai) => match b {
                VMInt(bi) => {
                  bind _ <- Mut.push(stack, VMInt(ai * bi))
                  vm_execute(bytecode, stack, dec.next_pc)
                }
                _ => Result.err("Mul: type error")
              }
              _ => Result.err("Mul: type error on a")
            }
          }
        }
      }
      Eq => {
        bind b_r <- Mut.pop(stack)
        bind a_r <- Mut.pop(stack)
        match b_r {
          err(e) => Result.err(e)
          ok(b) => match a_r {
            err(e) => Result.err(e)
            ok(a) => match a {
              VMInt(ai) => match b {
                VMInt(bi) => {
                  bind _ <- Mut.push(stack, VMBool(ai == bi))
                  vm_execute(bytecode, stack, dec.next_pc)
                }
                _ => Result.err("Eq: type error")
              }
              _ => Result.err("Eq: unsupported type")
            }
          }
        }
      }
      _ => Result.err("vm_execute: unimplemented opcode")
    }
  }
}
```

### T1-3: `fn vm_run`

```favnir
fn vm_run(bytecode: Bytes) -> Result<VMVal, String> {
  bind stack <- Mut.list()
  vm_execute(bytecode, stack, 0)
}
```

### T1-4 (フォールバック): `bind _` が使えない場合

`bind _ <- Mut.push(...)` が Parser エラーになる場合は `bind _skip <-` 等の named pattern で代替:

```favnir
ConstUnit => {
  bind _skip <- Mut.push(stack, VMUnit)
  vm_execute(bytecode, stack, dec.next_pc)
}
```

### T1-5 (フォールバック): `bind x <- Mut.list()` が動作しない場合

`Mut.list()` が `Result<MutList, String>` を返す場合（ありえないが念のため）:

```bash
grep -n '"Mut.list"' fav/src/backend/vm.rs
# → Ok(VMValue::MutList(...)) を直接返すことを確認
```

---

## T2: `fav/src/driver.rs` — `v235000_tests` 追加

### T2-1: `v234000_tests::version_is_23_4_0` に `#[ignore]` 追加（T3-1 より前に必ず実施）

```rust
#[test]
#[ignore]
fn version_is_23_4_0() {
```

### T2-2: `v235000_tests` モジュールを `v234000_tests` の直後に追加

v23.4.0 の `decode_const_opcode` / `vm_fav_compiles` テストと同じパターンを使う:
- `Lexer::new(src, "vm_test.fav").tokenize()`
- `Parser::new(tokens).parse_program()`
- `build_artifact(&prog)` ← 引数は `&Program` のみ（`src` は不要）
- `exec_artifact_main(&artifact, None)` → `Result<Value, String>`
- `assert_eq!(result, crate::value::Value::Str("VMUnit".to_string()), ...)` ← `Value::Str` で比較
- `include_str!("../Cargo.toml")` ← driver.rs からの相対パス（`../../` ではない）

```rust
#[cfg(test)]
mod v235000_tests {
    use super::*;

    #[test]
    fn version_is_23_5_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("version = \"23.5.0\""), "Cargo.toml should have version 23.5.0");
    }

    #[test]
    fn vm_fav_phase2_compiles() {
        let src = include_str!("../self/vm.fav");
        let tokens = crate::frontend::lexer::Lexer::new(src, "vm.fav")
            .tokenize().expect("lex vm.fav");
        let prog = crate::frontend::parser::Parser::new(tokens)
            .parse_program().expect("parse vm.fav");
        let _artifact = build_artifact(&prog);
    }

    #[test]
    fn execute_const_unit() {
        // bytecode: ConstUnit(0x02) + Return(0x16)
        let vm_src = include_str!("../self/vm.fav");
        let src = format!(r#"{}
public fn main() -> String {{
  bind hex_r <- Bytes.from_hex("0216")
  match hex_r {{
    ok(bytes) => {{
      bind run_r <- vm_run(bytes)
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
        assert_eq!(result, crate::value::Value::Str("VMUnit".to_string()),
            "vm_run([ConstUnit, Return]) should return VMUnit");
    }

    #[test]
    fn execute_add() {
        // bytecode: Const(3) + Const(4) + Add + Return
        // 01 03 00  01 04 00  20  16
        let vm_src = include_str!("../self/vm.fav");
        let src = format!(r#"{}
public fn main() -> String {{
  bind hex_r <- Bytes.from_hex("0103000104002016")
  match hex_r {{
    ok(bytes) => {{
      bind run_r <- vm_run(bytes)
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
        assert_eq!(result, crate::value::Value::Str("VMInt(7)".to_string()),
            "vm_run([Const(3), Const(4), Add, Return]) should return VMInt(7)");
    }

    #[test]
    fn changelog_has_v23_5_0() {
        let cl = include_str!("../../CHANGELOG.md");
        assert!(cl.contains("[v23.5.0]"), "CHANGELOG.md should have [v23.5.0] entry");
    }
}
```

---

## T3: Cargo.toml + CHANGELOG + benchmarks + docs

> **注意**: T2-1 の `#[ignore]` 追加完了後に Cargo.toml を更新すること。

### T3-1: `fav/Cargo.toml` バージョン更新

```
version = "23.4.0" → "23.5.0"
```

### T3-2: `CHANGELOG.md` に v23.5.0 エントリ追加（先頭）

```markdown
## [v23.5.0] — 2026-06-22

### Added
- `vm.fav` Phase 2: スタックベース実行ループ
  - `type VMVal` (VMInt / VMBool / VMUnit)
  - `fn vm_execute` — 11 オペコード dispatch（ConstUnit/ConstTrue/ConstFalse/Const/Pop/Dup/Return/Add/Sub/Mul/Eq）
  - `fn vm_run(bytecode: Bytes) -> Result<VMVal, String>` エントリポイント
  - `fn vmval_to_string` デバッグ用文字列化
```

### T3-3: `benchmarks/v23.5.0.json` 作成

```json
{
  "version": "23.5.0",
  "date": "2026-06-22",
  "test_count": 0,
  "feature": "vm.fav Phase 2（スタックベース実行ループ）",
  "metrics": {
    "vm_fav_opcodes_phase2": 11,
    "vmval_variants": 3,
    "self_hosted_vm_phase": 2
  }
}
```

`test_count` は `cargo test --bin fav` 実行後に実際の値に更新する（`0` は仮値）。

### T3-4: `site/content/docs/tools/vm-fav.mdx` に Phase 2 セクション追記

既存 MDX ファイルの末尾に追記:

```mdx
## Phase 2: スタックベース実行ループ（v23.5.0）

### VMVal 型

```favnir
type VMVal =
  | VMInt(Int)
  | VMBool(Bool)
  | VMUnit
```

### 実行エントリポイント

```favnir
// バイトコードを実行して結果を返す
bind hex_r <- Bytes.from_hex("0103000104002016")
match hex_r {
  ok(bytes) => {
    bind run_r <- vm_run(bytes)
    match run_r {
      ok(v)  => vmval_to_string(v)  // => "VMInt(7)"
      err(e) => e
    }
  }
  err(e) => e
}
```

### 対応オペコード（Phase 2）

| オペコード | 動作 |
|---|---|
| ConstUnit / ConstTrue / ConstFalse | 定数 push |
| Const(n) | VMInt(n) として push |
| Pop / Dup | スタック操作 |
| Return | top を Result.ok で返す |
| Add / Sub / Mul | 整数演算 |
| Eq | 整数比較 → VMBool |
```

---

## 実装順序

```
T0（事前確認）
T1（vm.fav 追記）
T2-1（#[ignore]）← T3-1 より前に必須
T2-2（tests）    ← T1 完了後
T3-1（version）  ← T2-1 完了後
T3-2〜4（docs）  ← T3-1 完了後
```

---

## リスク対応表

| リスク | 検出方法 | 対応 |
|---|---|---|
| `bind _ <- ...` が Parse エラー | `cargo test v235000` 実行時のエラーメッセージ | `bind _skip <-` に変更 |
| `VMVal` バリアントが match で認識されない | コンパイルエラー | 型名確認、`VMInt(ai)` → `VMVal.VMInt(ai)` 等 |
| `Mut.push(stack, VMUnit)` 型エラー | checker エラー | `Unknown` が `VMVal` に unify されない場合はラッパー関数で回避 |
| 再帰 `vm_execute` がスタックオーバーフロー | 実行時 | Phase 2 はテスト用の短いバイトコードのみ → 問題なし |
| `dec.op` のフィールドアクセス | コンパイルエラー | `DecodeResult` の `op` フィールドを確認 |
