# v23.8.0 — vm.fav Phase 5（GetField・collect_args・hello.fav 実行）

Date: 2026-06-22

## 目標

vm.fav に以下の 3 本柱を追加し、`LoadGlobal + GetField + Call(N)` の完全なビルトイン呼び出しシーケンスと、
シンプルな Favnir プログラム（hello.fav 相当）の VM 実行を実現する。

1. **`GetField(idx)`** — namespace + field_name を VMStr 結合し builtin 名を解決
2. **`collect_args`** — スタックから N 個の引数を正順で収集するヘルパー
3. **`Call(argc)` 汎用化** — collect_args を使った単一実装で任意の argc を処理
4. **`vmval_display`** — VMVal の値をユーザー向け表示形式に変換（VMStr → s、VMInt → n）
5. **`call_builtin` 拡張** — `String.concat(a, b)`（2 引数 builtin の例）

---

## スコープ

### Favnir（fav/self/vm.fav への変更）

| 変更種別 | 対象 | 内容 |
|---|---|---|
| 新関数追加 | `fn collect_args_rec` | スタックから N 個の引数を再帰的に収集（acc に push） |
| 新関数追加 | `fn collect_args` | `collect_args_rec` の公開 API |
| オペコード追加 | `GetField(idx)` | pop namespace VMStr + globals[idx] = field VMStr → push `ns.field` VMStr |
| リファクタリング | `Call(argc)` | `match argc { 0/1/_ }` を `collect_args` 利用の単一実装に置換 |
| 新関数追加 | `fn vmval_display` | VMStr(s) → s、VMInt(n) → n、VMBool → true/false、VMUnit → "" |
| call_builtin 拡張 | `"String.concat"` | 2 引数 builtin の実装例（collect_args の引数順を検証） |

### Rust（変更なし）

Phase 5 に必要な Rust primitive は全て実装済み。

---

## 型・関数定義

### collect_args_rec / collect_args

```favnir
// スタックから n 個の引数を LIFO 順で acc に push する。
// 収集後の acc のトップが「ソースコード上の最初の引数」になる。
// （スタック: callee, arg1, arg2 の順でプッシュ → arg2 が先に pop される）
// collect_args_rec は pop した arg2 を acc に push し、次に arg1 を push する。
// 結果: acc = [arg2, arg1] → Mut.pop(acc) で arg1 を先に取り出せる = ソース順）
fn collect_args_rec(stack: Int, n: Int, acc: Int) -> Result<Int, String> {
  match n {
    0 => Result.ok(acc)
    _ => {
      bind v_r <- Mut.pop(stack)
      match v_r {
        err(e) => Result.err(f"collect_args: {e}")
        ok(v) => {
          bind _ <- Mut.push(acc, v)
          collect_args_rec(stack, n - 1, acc)
        }
      }
    }
  }
}

fn collect_args(stack: Int, n: Int) -> Result<Int, String> {
  bind acc <- Mut.list()
  collect_args_rec(stack, n, acc)
}
```

> **引数順の不変条件:**
> `collect_args` の結果 `args` に対して `Mut.pop(args)` を繰り返すと、
> ソースコード上の arg1、arg2、... の順で取り出せる。
> `call_builtin` 内では常に「第 1 引数を最初に pop」する記法を使う。

### GetField(idx) オペコード

```favnir
// GetField: Namespace.Field 名を動的に合成する
// スタック前: ..., VMStr("String")
// 操作: pop VMStr(ns), globals[idx] = VMStr(field), push VMStr("String.trim")
// スタック後: ..., VMStr("String.trim")
GetField(idx) => {
  bind ns_r <- Mut.pop(stack)
  match ns_r {
    err(e) => Result.err(f"GetField: stack underflow: {e}")
    ok(ns) => match ns {
      VMStr(ns_name) => {
        bind field_r <- Mut.get(globals, idx)
        match field_r {
          err(e) => Result.err(f"GetField: globals[{idx}] not found: {e}")
          ok(field) => match field {
            VMStr(field_name) => {
              bind push_r <- Mut.push(stack, VMStr(String.concat(ns_name, String.concat(".", field_name))))
              match push_r {
                err(e) => Result.err(e)
                ok(_)  => vm_execute(bytecode, stack, locals, globals, dec.next_pc)
              }
            }
            _ => Result.err("GetField: globals[idx] is not VMStr")
          }
        }
      }
      _ => Result.err("GetField: top of stack is not VMStr")
    }
  }
}
```

### Call(argc) — collect_args を使った汎用実装

```favnir
// 既存の match argc { 0 => ... 1 => ... _ => err } を置換
Call(argc) => {
  bind args_r <- collect_args(stack, argc)
  match args_r {
    err(e) => Result.err(e)
    ok(args) => {
      bind callee_r <- Mut.pop(stack)
      match callee_r {
        err(e) => Result.err(e)
        ok(callee) => match callee {
          VMStr(name) => {
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
          _ => Result.err("Call: callee is not a VMStr")
        }
      }
    }
  }
}
```

### vmval_display（新規）

```favnir
// ユーザー向け表示形式。VMStr は引用符なし。vmval_to_string はデバッグ用。
fn vmval_display(v: VMVal) -> String {
  match v {
    VMInt(n)  => f"{n}"
    VMBool(b) => f"{b}"
    VMUnit    => ""
    VMStr(s)  => s
  }
}
```

### call_builtin 拡張 — String.concat

```favnir
// 引数は collect_args 収集後のソース順（第 1 引数が Mut.pop で先に取れる）
"String.concat" => {
  bind a_r <- Mut.pop(args)
  match a_r {
    err(e) => Result.err(e)
    ok(a) => {
      bind b_r <- Mut.pop(args)
      match b_r {
        err(e) => Result.err(e)
        ok(b) => match a {
          VMStr(sa) => match b {
            VMStr(sb) => Result.ok(VMStr(String.concat(sa, sb)))
            _ => Result.err("String.concat: arg2 not VMStr")
          }
          _ => Result.err("String.concat: arg1 not VMStr")
        }
      }
    }
  }
}
```

---

## 引数順の不変条件（設計ノート）

```
Favnir ソース: String.concat("hello", " world")
コンパイル後のスタック操作:
  1. LoadGlobal(ns_idx="String")  → push VMStr("String")
  2. GetField(field_idx="concat") → pop "String" + push VMStr("String.concat")
  3. LoadGlobal(arg1_idx)         → push VMStr("hello")
  4. LoadGlobal(arg2_idx)         → push VMStr(" world")
  5. Call(2)                      → collect_args(2)

collect_args(2) 動作:
  step1: pop VMStr(" world") → acc = [" world"]
  step2: pop VMStr("hello")  → acc = [" world", "hello"]  ← "hello" が top

call_builtin("String.concat", acc):
  Mut.pop(acc) → "hello"  (arg1: ソース順で第 1 引数)
  Mut.pop(acc) → " world" (arg2: ソース順で第 2 引数)
  String.concat("hello", " world") → "hello world" ✓
```

---

## 設計上の注意点

| # | 注意点 | 対応 |
|---|---|---|
| 1 | `collect_args_rec` は整数リテラル `match n { 0 => ... _ => ... }` を使う | Favnir でサポート済み（vm.fav Div アームで確認済み） |
| 2 | `Call(argc)` の既存コメント（積み順説明）を新実装に合わせて更新 | collect_args 実装のコメントで一元管理 |
| 3 | `GetField` の `String.concat(ns_name, String.concat(".", field_name))` | f-string `f"{ns_name}.{field_name}"` では String に引用符が付くため String.concat を使う |
| 4 | `vmval_display` の `VMStr(s) => s` | f-string でなく直接 `s` を返す（引用符なし） |
| 5 | `collect_args_rec` の `bind _ <- Mut.push(acc, v)` | Mut.push は Result を返すが acc への push 失敗は実質ありえない；エラーハンドリングは省略可だが `bind _ <- ` で sequencing する |

---

## テスト（6 件）

| テスト名 | 内容 | 期待値 |
|---|---|---|
| `version_is_23_8_0` | Cargo.toml に `version = "23.8.0"` | — |
| `vm_fav_phase5_compiles` | vm.fav を parse + build_artifact | エラーなし |
| `execute_hello_via_vm` | `LoadGlobal(0)="hello" + Return` → `vmval_display` | `"hello"` |
| `execute_getfield_call` | `LoadGlobal(0)="String" + GetField(1)="trim" + LoadGlobal(2)=" hi " + Call(1) + Return` → `vmval_display` | `"hi"` |
| `execute_string_concat` | `LoadGlobal(0)="String" + GetField(1)="concat" + LoadGlobal(2)="hello" + LoadGlobal(3)=" world" + Call(2) + Return` → `vmval_display` | `"hello world"` |
| `changelog_has_v23_8_0` | CHANGELOG.md に `[v23.8.0]` | — |

### バイトコード詳細

**`execute_hello_via_vm`**: hex `"12000016"`

```
globals[0] = VMStr("hello")

pc=0: 12 00 00  LoadGlobal(0)  → push VMStr("hello")
pc=3: 16        Return          → VMStr("hello")

vmval_display(VMStr("hello")) = "hello"
```

**`execute_getfield_call`**: hex `"12000040010012020015010016"`

```
globals[0] = VMStr("String")
globals[1] = VMStr("trim")
globals[2] = VMStr(" hi ")

pc=0:  12 00 00  LoadGlobal(0)  → push VMStr("String")
pc=3:  40 01 00  GetField(1)    → pop "String" + get "trim" → push VMStr("String.trim")
pc=6:  12 02 00  LoadGlobal(2)  → push VMStr(" hi ")
pc=9:  15 01 00  Call(1)        → collect_args(1) → args=[VMStr(" hi ")]
                                   pop callee VMStr("String.trim")
                                   call_builtin("String.trim", ...) → VMStr("hi")
                                   push VMStr("hi")
pc=12: 16        Return          → VMStr("hi")

vmval_display(VMStr("hi")) = "hi"
```

**`execute_string_concat`**: hex `"12000040010012020012030015020016"`

```
globals[0] = VMStr("String")
globals[1] = VMStr("concat")
globals[2] = VMStr("hello")
globals[3] = VMStr(" world")

pc=0:  12 00 00  LoadGlobal(0)  → push VMStr("String")
pc=3:  40 01 00  GetField(1)    → pop "String" + get "concat" → push VMStr("String.concat")
pc=6:  12 02 00  LoadGlobal(2)  → push VMStr("hello")
pc=9:  12 03 00  LoadGlobal(3)  → push VMStr(" world")
pc=12: 15 02 00  Call(2)        → collect_args(2)
                                   step1: pop VMStr(" world") → acc=[" world"]
                                   step2: pop VMStr("hello")  → acc=[" world", "hello"]
                                   pop callee VMStr("String.concat")
                                   call_builtin("String.concat", acc):
                                     Mut.pop(acc) → "hello"  (arg1)
                                     Mut.pop(acc) → " world" (arg2)
                                     String.concat("hello", " world") → "hello world"
pc=15: 16        Return          → VMStr("hello world")

vmval_display(VMStr("hello world")) = "hello world"
```

> このテストにより collect_args の LIFO 反転ロジック（argc=2）が正しく機能することを実証する。

---

## ロードマップとの関係

ロードマップ v23.8 は「vm.fav で vm.fav 自体を実行できることを検証する」と定義している。
Phase 5 では `LoadGlobal + GetField + Call(N)` のシーケンスを完成させることで、
任意のビルトイン呼び出しチェーンが vm.fav 上で動作する基盤を確立する。

**ロードマップ完了条件との対応:**

| ロードマップ完了条件 | v23.8.0 での対応 |
|---|---|
| `fav run --vm=self/vm.fav hello.fav` が動作する | `execute_hello_via_vm` テストで等価実証（CLI フラグは v24.0 で実装） |
| vm.fav での実行結果が Rust VM と一致（500 件以上） | v24.0 マイルストーン宣言で達成目標；v23.8.0 は基盤を完成させる |
| `fav test self/vm.fav` 全件 PASS | v23.8.0 の 6 件テストで確認 |

**スコープ外（v24.0 で実装）:**
- `fav run --vm=<path>` CLI フラグ（main.rs の実装）
- 500 件超テストの vm.fav 経由実行
- ユーザー定義関数の Call ディスパッチ（非ビルトイン）

---

## 完了条件

- [ ] `fn collect_args_rec` が追加される
- [ ] `fn collect_args` が追加される
- [ ] `Call(argc)` ハンドラが `collect_args` を使った単一実装に置換される
- [ ] `GetField(idx)` が実装される（namespace + "." + field_name）
- [ ] `fn vmval_display` が追加される
- [ ] `call_builtin` に `"String.concat"` が追加される
- [ ] `v237000_tests::version_is_23_7_0` が削除済み（`#[ignore]` 蓄積を防ぐ方針）
- [ ] `cargo test v238000 --bin fav` — 6/6 PASS
- [ ] `cargo test --bin fav` — リグレッションなし（1921 件以上合格）
- [ ] `CHANGELOG.md` に v23.8.0 エントリ
- [ ] `benchmarks/v23.8.0.json` 作成済み
- [ ] `site/content/docs/tools/vm-fav.mdx` に Phase 5 セクション追記
