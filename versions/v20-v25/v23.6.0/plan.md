# v23.6.0 実装計画 — vm.fav Phase 3

## 前提確認

v23.6.0 は Rust 側変更なし。fav/self/vm.fav の修正 + テスト + ドキュメントのみ。

### 実装前チェック

```bash
grep -n "fn vm_execute\|fn vm_run" fav/self/vm.fav
# → 現在のシグネチャが (bytecode: Bytes, stack: Int, pc: Int) であること

grep -n "vm_execute(bytecode, stack" fav/self/vm.fav | wc -l
# → Phase 2 の再帰呼び出し件数を確認（10〜12 件程度）

grep -n "version = " fav/Cargo.toml
# → "23.5.0" であること

grep -n "mod v235000_tests\|mod v236000_tests" fav/src/driver.rs | head -3
# → v236000_tests が未存在であること
```

---

## T0: 事前確認 — `&&`/`||` 演算子サポート確認

```bash
grep -n "AmpAmp\|PipePipe\|&&\|||" fav/src/frontend/lexer.rs | head -5
```

`AmpAmp` / `PipePipe` トークンが存在すれば `&&`/`||` は使用可能。

---

## T1: `fav/self/vm.fav` — Phase 3 変更

### T1-1: `vm_execute` / `vm_run` シグネチャ更新

**変更 1**: `fn vm_execute` の定義行を更新

```
旧: fn vm_execute(bytecode: Bytes, stack: Int, pc: Int) -> Result<VMVal, String> {
新: fn vm_execute(bytecode: Bytes, stack: Int, locals: Int, pc: Int) -> Result<VMVal, String> {
```

**変更 2**: Phase 2 の全再帰呼び出しを一括更新（検索→置換）

```
旧: vm_execute(bytecode, stack, dec.next_pc)
新: vm_execute(bytecode, stack, locals, dec.next_pc)
```

注: `Return` アームは再帰なし、変更不要。`Dup` アームの `vm_execute` も同様に更新すること。

**変更 3**: `vm_run` 更新

```favnir
fn vm_run(bytecode: Bytes) -> Result<VMVal, String> {
  bind stack  <- Mut.list()
  bind locals <- Mut.map()
  vm_execute(bytecode, stack, locals, 0)
}
```

---

### T1-2: Phase 3 オペコード追加（`_` アームの直前に挿入）

現在の `_ => Result.err("vm_execute: unimplemented opcode")` の直前に以下を追加:

```favnir
      Jump(off) => {
        vm_execute(bytecode, stack, locals, dec.next_pc + off)
      }
      JumpIfFalse(off) => {
        bind cond_r <- Mut.pop(stack)
        match cond_r {
          err(e) => Result.err(e)
          ok(cond) => match cond {
            VMBool(b) => match b {
              true  => vm_execute(bytecode, stack, locals, dec.next_pc)
              false => vm_execute(bytecode, stack, locals, dec.next_pc + off)
            }
            _ => Result.err("JumpIfFalse: not a Bool")
          }
        }
      }
      LoadLocal(slot) => {
        bind get_r <- Mut.get(locals, slot)
        match get_r {
          err(e) => Result.err(f"LoadLocal: {e}")
          ok(v) => {
            bind push_r <- Mut.push(stack, v)
            match push_r {
              err(e) => Result.err(f"LoadLocal push: {e}")
              ok(_)  => vm_execute(bytecode, stack, locals, dec.next_pc)
            }
          }
        }
      }
      StoreLocal(slot) => {
        bind val_r <- Mut.pop(stack)
        match val_r {
          err(e) => Result.err(e)
          ok(val) => {
            bind set_r <- Mut.set(locals, slot, val)
            match set_r {
              err(e) => Result.err(f"StoreLocal: {e}")
              ok(_)  => vm_execute(bytecode, stack, locals, dec.next_pc)
            }
          }
        }
      }
      Ne => {
        bind b_r <- Mut.pop(stack)
        match b_r {
          err(e) => Result.err(e)
          ok(b) => {
            bind a_r <- Mut.pop(stack)
            match a_r {
              err(e) => Result.err(e)
              ok(a) => match a {
                VMInt(ai) => match b {
                  VMInt(bi) => {
                    bind push_r <- Mut.push(stack, VMBool(ai != bi))
                    match push_r {
                      err(e) => Result.err(e)
                      ok(_)  => vm_execute(bytecode, stack, locals, dec.next_pc)
                    }
                  }
                  _ => Result.err("Ne: type error on b")
                }
                _ => Result.err("Ne: type error on a")
              }
            }
          }
        }
      }
      Lt => {
        bind b_r <- Mut.pop(stack)
        match b_r {
          err(e) => Result.err(e)
          ok(b) => {
            bind a_r <- Mut.pop(stack)
            match a_r {
              err(e) => Result.err(e)
              ok(a) => match a {
                VMInt(ai) => match b {
                  VMInt(bi) => {
                    bind push_r <- Mut.push(stack, VMBool(ai < bi))
                    match push_r {
                      err(e) => Result.err(e)
                      ok(_)  => vm_execute(bytecode, stack, locals, dec.next_pc)
                    }
                  }
                  _ => Result.err("Lt: type error on b")
                }
                _ => Result.err("Lt: type error on a")
              }
            }
          }
        }
      }
      Le => {
        bind b_r <- Mut.pop(stack)
        match b_r {
          err(e) => Result.err(e)
          ok(b) => {
            bind a_r <- Mut.pop(stack)
            match a_r {
              err(e) => Result.err(e)
              ok(a) => match a {
                VMInt(ai) => match b {
                  VMInt(bi) => {
                    bind push_r <- Mut.push(stack, VMBool(ai <= bi))
                    match push_r {
                      err(e) => Result.err(e)
                      ok(_)  => vm_execute(bytecode, stack, locals, dec.next_pc)
                    }
                  }
                  _ => Result.err("Le: type error on b")
                }
                _ => Result.err("Le: type error on a")
              }
            }
          }
        }
      }
      Gt => {
        bind b_r <- Mut.pop(stack)
        match b_r {
          err(e) => Result.err(e)
          ok(b) => {
            bind a_r <- Mut.pop(stack)
            match a_r {
              err(e) => Result.err(e)
              ok(a) => match a {
                VMInt(ai) => match b {
                  VMInt(bi) => {
                    bind push_r <- Mut.push(stack, VMBool(ai > bi))
                    match push_r {
                      err(e) => Result.err(e)
                      ok(_)  => vm_execute(bytecode, stack, locals, dec.next_pc)
                    }
                  }
                  _ => Result.err("Gt: type error on b")
                }
                _ => Result.err("Gt: type error on a")
              }
            }
          }
        }
      }
      Ge => {
        bind b_r <- Mut.pop(stack)
        match b_r {
          err(e) => Result.err(e)
          ok(b) => {
            bind a_r <- Mut.pop(stack)
            match a_r {
              err(e) => Result.err(e)
              ok(a) => match a {
                VMInt(ai) => match b {
                  VMInt(bi) => {
                    bind push_r <- Mut.push(stack, VMBool(ai >= bi))
                    match push_r {
                      err(e) => Result.err(e)
                      ok(_)  => vm_execute(bytecode, stack, locals, dec.next_pc)
                    }
                  }
                  _ => Result.err("Ge: type error on b")
                }
                _ => Result.err("Ge: type error on a")
              }
            }
          }
        }
      }
      And => {
        bind b_r <- Mut.pop(stack)
        match b_r {
          err(e) => Result.err(e)
          ok(b) => {
            bind a_r <- Mut.pop(stack)
            match a_r {
              err(e) => Result.err(e)
              ok(a) => match a {
                VMBool(ab) => match b {
                  VMBool(bb) => {
                    bind push_r <- Mut.push(stack, VMBool(ab && bb))
                    match push_r {
                      err(e) => Result.err(e)
                      ok(_)  => vm_execute(bytecode, stack, locals, dec.next_pc)
                    }
                  }
                  _ => Result.err("And: type error on b")
                }
                _ => Result.err("And: type error on a")
              }
            }
          }
        }
      }
      Or => {
        bind b_r <- Mut.pop(stack)
        match b_r {
          err(e) => Result.err(e)
          ok(b) => {
            bind a_r <- Mut.pop(stack)
            match a_r {
              err(e) => Result.err(e)
              ok(a) => match a {
                VMBool(ab) => match b {
                  VMBool(bb) => {
                    bind push_r <- Mut.push(stack, VMBool(ab || bb))
                    match push_r {
                      err(e) => Result.err(e)
                      ok(_)  => vm_execute(bytecode, stack, locals, dec.next_pc)
                    }
                  }
                  _ => Result.err("Or: type error on b")
                }
                _ => Result.err("Or: type error on a")
              }
            }
          }
        }
      }
      Div => {
        bind b_r <- Mut.pop(stack)
        match b_r {
          err(e) => Result.err(e)
          ok(b) => {
            bind a_r <- Mut.pop(stack)
            match a_r {
              err(e) => Result.err(e)
              ok(a) => match a {
                VMInt(ai) => match b {
                  VMInt(bi) => match bi {
                    0 => Result.err("Div: division by zero")
                    _ => {
                      bind push_r <- Mut.push(stack, VMInt(ai / bi))
                      match push_r {
                        err(e) => Result.err(e)
                        ok(_)  => vm_execute(bytecode, stack, locals, dec.next_pc)
                      }
                    }
                  }
                  _ => Result.err("Div: type error on b")
                }
                _ => Result.err("Div: type error on a")
              }
            }
          }
        }
      }
```

> **`JumpIfFalse` の bool match について**:
> `match b { true => ... false => ... }` は有効。`parser.rs:2511` で `Pattern::Lit(Lit::Bool)` として実装済み。
> フォールバック不要。

> **`Mut.push` の `bind push_r <-` について**:
> v23.5.0 コードレビューの対応として、push エラーも明示的に処理する。

---

## T2: `fav/src/driver.rs` — `v236000_tests` 追加

### T2-1: `v235000_tests::version_is_23_5_0` に `#[ignore]` 追加（T3-1 より前に必須）

```rust
#[test]
#[ignore]
fn version_is_23_5_0() {
```

### T2-2: `v236000_tests` モジュールを `v235000_tests` の直後に追加

```rust
// ── v236000_tests (v23.6.0) — vm.fav Phase 3（制御フロー・ローカル変数）────────
#[cfg(test)]
mod v236000_tests {
    use super::*;

    #[test]
    fn version_is_23_6_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("version = \"23.6.0\""), "Cargo.toml should have version 23.6.0");
    }

    #[test]
    fn vm_fav_phase3_compiles() {
        let src = include_str!("../self/vm.fav");
        let tokens = crate::frontend::lexer::Lexer::new(src, "vm.fav")
            .tokenize().expect("lex vm.fav");
        let prog = crate::frontend::parser::Parser::new(tokens)
            .parse_program().expect("parse vm.fav");
        let _artifact = build_artifact(&prog);
    }

    #[test]
    fn execute_locals() {
        // Const(42) → StoreLocal(0) → LoadLocal(0) → Return → VMInt(42)
        // 01 2A 00  11 00 00  10 00 00  16
        let vm_src = include_str!("../self/vm.fav");
        let src = format!(r#"{}
public fn main() -> String {{
  bind hex_r <- Bytes.from_hex("012a0011000010000016")
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
        assert_eq!(result, crate::value::Value::Str("VMInt(42)".to_string()),
            "locals round-trip: Const(42)+StoreLocal(0)+LoadLocal(0)+Return should be VMInt(42)");
    }

    #[test]
    fn execute_jump() {
        // ConstFalse + JumpIfFalse(6) → skip Const(1)+Jump(3) → Const(2) → Return → VMInt(2)
        // 04  31 06 00  01 01 00  30 03 00  01 02 00  16
        let vm_src = include_str!("../self/vm.fav");
        let src = format!(r#"{}
public fn main() -> String {{
  bind hex_r <- Bytes.from_hex("0431060001010030030001020016")
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
        assert_eq!(result, crate::value::Value::Str("VMInt(2)".to_string()),
            "JumpIfFalse with false condition should reach false path: VMInt(2)");
    }

    #[test]
    fn changelog_has_v23_6_0() {
        let cl = include_str!("../../CHANGELOG.md");
        assert!(cl.contains("[v23.6.0]"), "CHANGELOG.md should have [v23.6.0] entry");
    }
}
```

---

## T3: Cargo.toml + CHANGELOG + benchmarks + docs

> **注意**: T2-1 の `#[ignore]` 追加完了後に Cargo.toml を更新すること。

### T3-1: `fav/Cargo.toml` バージョン更新
```
version = "23.5.0" → "23.6.0"
```

### T3-2: `CHANGELOG.md` 先頭に v23.6.0 エントリ追加

```markdown
## [v23.6.0] — 2026-06-22 — vm.fav Phase 3（制御フロー・ローカル変数）

### Added
- `vm.fav` Phase 3: 制御フロー・ローカル変数
  - `vm_execute` に `locals: Int` パラメータを追加（MutMap による単一フレームのローカル変数）
  - `vm_run` が Mut.map() でローカル変数マップを生成
  - 新オペコード 12 件: Jump / JumpIfFalse / LoadLocal / StoreLocal / Ne / Lt / Le / Gt / Ge / And / Or / Div

### Changed
- `fn vm_execute` シグネチャ: `(bytecode, stack, pc)` → `(bytecode, stack, locals, pc)`
```

### T3-3: `benchmarks/v23.6.0.json` 作成（`test_count` は実行後に更新）

```json
{
  "version": "23.6.0",
  "date": "2026-06-22",
  "test_count": 0,
  "feature": "vm.fav Phase 3（制御フロー・ローカル変数）",
  "metrics": {
    "vm_fav_opcodes_phase3": 12,
    "vm_execute_params": 4,
    "self_hosted_vm_phase": 3
  }
}
```

### T3-4: `site/content/docs/tools/vm-fav.mdx` に Phase 3 セクション追記

フェーズ表を更新:
```
| Phase 3（v23.6.0） | 制御フロー（Jump/JumpIfFalse）・ローカル変数（LoadLocal/StoreLocal）・残余演算 |
```

---

## 実装順序

```
T0（事前確認）
T1-1（シグネチャ更新 + 再帰呼び出し一括更新）
T1-2（Phase 3 オペコード追加）
cargo check → エラー 0 確認
T2-1（#[ignore]）← T3-1 より前に必須
T2-2（tests）    ← T1 完了後
T3-1（version）  ← T2-1 完了後
T3-2〜4（docs）  ← T3-1 完了後
```

---

## リスク対応表

| リスク | 検出方法 | 対応 |
|---|---|---|
| `match b { true => ... false => ... }` がパース不可 | コンパイルエラー | `if b { ... } else { ... }` に変更 |
| `&&`/`||` が Bool 演算子でない | コンパイルエラー | `match a { true => match b { true => true _ => false } _ => false }` で代替 |
| `Mut.set(locals, slot, val)` が slot:Int を受け付けない | 実行時エラー | `Mut.set(locals, f"{slot}", val)` + `Mut.get(locals, f"{slot}")` で String キーに変換（フォールバック） |
| Phase 2 再帰呼び出しの更新漏れ | コンパイルエラー（型エラー）または実行時エラー | `grep -n "vm_execute(bytecode, stack, dec" fav/self/vm.fav` で残りを確認 |
