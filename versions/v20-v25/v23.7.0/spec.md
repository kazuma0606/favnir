# v23.7.0 — vm.fav Phase 4（stdlib・builtin 呼び出し）

Date: 2026-06-22

## 目標

vm.fav に builtin ディスパッチ層を実装する。
`LoadGlobal` / `Call` オペコードを処理し、`call_builtin` 関数（Favnir 実装）を通じて
Rust ネイティブ builtin を呼び出せるようにする。

これは「**Favnir ↔ Rust の永続的境界**」の確立である。
vm.fav は Favnir コードで書かれた `call_builtin` を通じて Rust VM の builtin を呼び出す。
この層は今後も Rust 側に残る（vm.fav で再実装しない）設計上の意図がある。

---

## スコープ

### Favnir（fav/self/vm.fav への変更）

| 変更種別 | 対象 | 内容 |
|---|---|---|
| 型追加 | `VMVal` | `VMStr(String)` バリアント追加 |
| 関数更新 | `vmval_to_string` | `VMStr(s) => f"VMStr({s})"` アーム追加 |
| シグネチャ変更 | `fn vm_execute` | `globals: Int` パラメータを追加（Mut.map handle） |
| 再帰呼び出し更新 | Phase 1〜3 全アーム | `vm_execute(..., locals, pc)` → `vm_execute(..., locals, globals, pc)` |
| 新関数追加 | `fn call_builtin` | builtin ディスパッチ関数（4 件） |
| 新オペコード追加 | `vm_execute` | `LoadGlobal(idx)` / `Call(argc)` の 2 件 |
| 関数更新 | `fn vm_run` | `globals` を空 Mut.map として生成し `vm_execute` に渡す |
| 新関数追加 | `fn vm_run_named` | globals を引数として受け取るエントリポイント |

### Rust（変更なし）

Phase 4 に必要な Rust primitive は全て実装済み。

| 使用 primitive | 追加バージョン | 用途 |
|---|---|---|
| `Mut.map()` / `Mut.get` / `Mut.set` | v23.3.0 | globals マップ |
| `Bytes.from_hex` | v23.1.0 | テスト用バイトコード構築 |

---

## 型・関数定義

### VMVal 拡張

```favnir
type VMVal =
  | VMInt(Int)
  | VMBool(Bool)
  | VMUnit
  | VMStr(String)    // Phase 4 追加: builtin 名または文字列値
```

### vmval_to_string 拡張

```favnir
fn vmval_to_string(v: VMVal) -> String {
  match v {
    VMInt(n)  => f"VMInt({n})"
    VMBool(b) => f"VMBool({b})"
    VMUnit    => "VMUnit"
    VMStr(s)  => f"VMStr({s})"   // Phase 4 追加
  }
}
```

### call_builtin（新規）

```favnir
// Favnir ↔ Rust の永続的境界。vm.fav からは Favnir 記法で呼び出すが、
// 実行時は Rust VM の vm_call_builtin にディスパッチされる。
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

### vm_execute シグネチャ変更

```favnir
// Phase 4: globals: Int — Mut<Map<Int, VMVal>> の opaque handle
// globals[idx] = VMStr("Namespace.function") のように builtin 名を格納
fn vm_execute(bytecode: Bytes, stack: Int, locals: Int, globals: Int, pc: Int) -> Result<VMVal, String>
```

### Phase 4 対応オペコード（2 件）

| オペコード | バイト | 動作 |
|---|---|---|
| `LoadGlobal(idx)` | `0x12 u16LE` | `globals[idx]` を pop → push |
| `Call(argc)` | `0x15 u16LE` | argc 個の引数 + callee を pop → `call_builtin` ディスパッチ |

> **Call の制限（Phase 4）**: argc=0 と argc=1 のみサポート。
> argc≥2 は `Result.err("Call: argc=N not yet supported")` を返す。
> 多引数対応は Phase 5（v23.8.0）で `collect_args` ヘルパー追加予定。

### vm_run 更新・vm_run_named 追加

```favnir
fn vm_run(bytecode: Bytes) -> Result<VMVal, String> {
  bind stack   <- Mut.list()
  bind locals  <- Mut.map()
  bind globals <- Mut.map()
  vm_execute(bytecode, stack, locals, globals, 0)
}

// Phase 4 追加: globals を外部から渡すエントリポイント
fn vm_run_named(bytecode: Bytes, globals: Int) -> Result<VMVal, String> {
  bind stack  <- Mut.list()
  bind locals <- Mut.map()
  vm_execute(bytecode, stack, locals, globals, 0)
}
```

---

## LoadGlobal / Call オペコード詳細

### LoadGlobal(idx)

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
```

### Call(argc) — argc=0 / argc=1 のみ

```favnir
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
```

---

## 設計上の注意点

| # | 注意点 | 対応 |
|---|---|---|
| 1 | `vm_execute` の再帰呼び出し更新数が多い（約 20〜25 件） | `grep -n "vm_execute(bytecode, stack, locals, dec.next_pc)" fav/self/vm.fav` で一括検索・更新 |
| 2 | `globals: Int` は `Mut.map()` handle | `Mut.get(globals, idx)` の `idx: Int` は `VMValue::Int(idx)` としてキー照合される（locals と同じパターン） |
| 3 | `f"{n}"` は `Int.to_string` の代替として確実に動作 | f-strings は Favnir の基本機能 |
| 4 | `Call(argc)` の引数はスタックトップが最後の引数 | 単引数: pop once; 0引数: callee のみ pop |
| 5 | `GetField` は Phase 4 では `_` フォールスルー | `// GetField は Phase 5 以降で実装予定` コメント追加 |
| 6 | `String.length(s)` / `String.trim(s)` の Favnir 可用性 | checker.rs の `builtin_ret_ty` に `("String", "length") => Type::Int` / `("String", "trim") => Type::String` 登録済み。checker.fav でも 7 箇所以上使用されており利用可能（リスクは低い） |
| 8 | `Call(argc)` の整数リテラル match | `match argc { 0 => ... 1 => ... _ => ... }` は Favnir がサポートする整数リテラルパターン。vm.fav の Div アーム（`match bi { 0 => ... _ => ... }`）で既に使用済み |
| 7 | `vm_run` の後方互換 | `globals <- Mut.map()` で空 globals を生成し既存テストが引き続き通る |

---

## テスト（5 件）

| テスト名 | 内容 | 期待値 |
|---|---|---|
| `version_is_23_7_0` | Cargo.toml に `version = "23.7.0"` | — |
| `vm_fav_phase4_compiles` | vm.fav を parse + build_artifact | エラーなし |
| `vmstr_to_string_variant` | `vmval_to_string(VMStr("hello"))` | `"VMStr(hello)"` |
| `execute_builtin_call` | `LoadGlobal(0)="Int.to_string" + Const(42) + Call(1) + Return` → vm_run_named | `"VMStr(42)"` |
| `changelog_has_v23_7_0` | CHANGELOG.md に `[v23.7.0]` | — |

### バイトコード詳細

**`execute_builtin_call`**: hex `"120000012a0015010016"`

```
globals[0] = VMStr("Int.to_string")   ← vm_run_named に渡す

pc=0: 12 00 00  LoadGlobal(0)   → push globals[0] = VMStr("Int.to_string")
pc=3: 01 2A 00  Const(42)       → push VMInt(42)
pc=6: 15 01 00  Call(1)         → pop VMInt(42), pop VMStr("Int.to_string")
                                   → call_builtin("Int.to_string", [VMInt(42)])
                                   → push VMStr("42")
pc=9: 16        Return           → pop → VMStr("42")

vmval_to_string(VMStr("42")) = "VMStr(42)"
```

---

## ロードマップとの関係

ロードマップ v23.7 は「Rust で実装された builtin を vm.fav から呼び出せるようにする」を示している。
Phase 4 はその核心である `call_builtin` を Favnir で実装し、
Favnir コードが Favnir の語彙（`String.length` / `String.trim` 等）で Rust VM の builtin を委譲できることを確立する。

**ロードマップからの意図的変更点:**

| 項目 | ロードマップ記述 | 実際の実装 | 変更理由 |
|---|---|---|---|
| `call_builtin` シグネチャ | `fn call_builtin(name: String, args: List<VMValue>)` | `fn call_builtin(name: String, args: Int)` | Phase 1〜3（v23.3〜v23.6）で確立した opaque handle パターン（Mut.list を Int として扱う）に統一 |
| ロードマップ v23.6 の成果物 | CallFrame / VMState レコード型 | 制御フロー + ローカル変数（単一フレーム） | v23.5.0 で Favnir 型チェッカーが `Mut<T>` をレコードフィールド型として扱えないことが判明。フェーズ定義を再構成 |
| `vm_run_named` | 記載なし | `fn vm_run_named(bytecode, globals)` を追加 | globals 注入のためのエントリポイントとして必須（テスト可能性のため） |

**Phase 4 で意図的にスコープ外とするもの:**
- `GetField` オペコード（レコード型アクセス）: Phase 5 以降
- `Call(argc >= 2)`: Phase 5 で `collect_args` 追加予定
- `VMList` / `VMRecord` バリアント: Phase 5 以降

---

## 完了条件

- [ ] `VMVal` に `VMStr(String)` バリアントが追加される
- [ ] `vmval_to_string` に `VMStr(s) => f"VMStr({s})"` が追加される
- [ ] `fn call_builtin` が追加される（4 builtin: Int.to_string / String.length / String.trim / Math.abs）
- [ ] `vm_execute` に `globals: Int` パラメータが追加される
- [ ] Phase 1〜3 全再帰呼び出しに `globals` が渡される
- [ ] `LoadGlobal(idx)` / `Call(0)` / `Call(1)` が実装される
- [ ] `vm_run` が空 globals を生成する
- [ ] `vm_run_named(bytecode, globals)` が追加される
- [ ] `cargo test v237000 --bin fav` — 5/5 PASS
- [ ] `cargo test --bin fav` — リグレッションなし（1917 件以上合格）
- [ ] `CHANGELOG.md` に v23.7.0 エントリ
- [ ] `benchmarks/v23.7.0.json` 作成済み
- [ ] `site/content/docs/tools/vm-fav.mdx` に Phase 4 セクション追記
