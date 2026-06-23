# v23.8.0 実装計画 — vm.fav Phase 5（GetField・collect_args・hello.fav 実行）

## 前提確認

v23.8.0 は Rust 側変更なし。fav/self/vm.fav の修正 + テスト + ドキュメントのみ。

### 実装前チェック

```bash
grep -n "fn vm_execute\|fn vm_run\|fn vm_run_named\|fn call_builtin" fav/self/vm.fav
# → 現在のシグネチャが (bytecode, stack, locals, globals, pc) であること

grep -n "GetField\|collect_args\|vmval_display" fav/self/vm.fav
# → 全て 0 件であること（未実装を確認）

grep -n "version = " fav/Cargo.toml
# → "23.7.0" であること

grep -n "mod v237000_tests\|mod v238000_tests" fav/src/driver.rs | head -3
# → v238000_tests が未存在であること

grep -n "Phase 5\|GetField" site/content/docs/tools/vm-fav.mdx | head -5
# → Phase 5 未存在・GetField が Phase 5 行に記載されていること確認
```

---

## T0: 事前確認

```bash
# GetField のオペコードバイト確認（0x40）
grep -n "GetField" fav/src/backend/codegen.rs | head -5
# → "GetField = 0x40" を確認

# vm.fav Phase 5 で使う String.concat が利用可能か確認
grep -rn "String\.concat" fav/self/ | head -5
# → checker.fav で使用されていれば利用可能

# vm-fav.mdx Phase 4 セクション追記済み・Phase 5 未存在確認
grep -n "Phase 4\|Phase 5" site/content/docs/tools/vm-fav.mdx | head -5
```

---

## T1: `fav/self/vm.fav` — Phase 5 変更

### T1-1: `fn collect_args_rec` を `call_builtin` の直後に追加

`call_builtin` 関数（約215行目）の直後、`fn vm_execute` の直前に挿入する。

```favnir
// Phase 5: collect_args_rec — スタックから n 個の引数を LIFO 順で acc に push する。
// 収集後の acc トップが「ソースコード上の第 1 引数」になる。
// 例: push callee, push arg1, push arg2, Call(2)
//   step1: pop arg2 → acc=[arg2]
//   step2: pop arg1 → acc=[arg2, arg1]  ← arg1 が top
//   → Mut.pop(acc) で arg1 が先に取れる = ソース順
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

> **引数順の不変条件:** `collect_args` の結果 `args` に対して `Mut.pop(args)` を繰り返すと、
> ソースコード上の arg1、arg2、... の順で取り出せる。

### T1-2: `fn vmval_display` を `vmval_to_string` の直後に追加

```favnir
// ユーザー向け表示形式。VMStr は引用符なし、VMInt は数値文字列。
// vmval_to_string はデバッグ用（VMStr("hello") 形式）。
fn vmval_display(v: VMVal) -> String {
  match v {
    VMInt(n)  => f"{n}"
    VMBool(b) => f"{b}"
    VMUnit    => ""
    VMStr(s)  => s
  }
}
```

### T1-3: `call_builtin` に `"String.concat"` を追加

`call_builtin` の `_ => Result.err(...)` アームの直前に追加する。

```favnir
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

### T1-4: `vm_execute` の `GetField` と `Call(argc)` を実装

#### GetField(idx) — `// GetField は Phase 5 以降で実装予定` コメントを置換

**現在（変更前）:**
```favnir
      // GetField は Phase 5 以降で実装予定
      _ => Result.err("vm_execute: unimplemented opcode")
```

**変更後:**
```favnir
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
      _ => Result.err("vm_execute: unimplemented opcode")
```

> **注意**: `GetField` の `String.concat(ns_name, String.concat(".", field_name))` は f-string 不可。
> f-string でネームスペース文字列を埋め込むと `"String"."trim"` のように引用符が付く。

#### Call(argc) — collect_args を使った汎用実装に置換

**現在（変更前）:**
```favnir
      Call(argc) => match argc {
        0 => {
          // ... (Phase 4 実装)
        }
        1 => {
          // ... (Phase 4 実装)
        }
        _ => Result.err(f"Call: argc={argc} not yet supported in Phase 4")
      }
```

**変更後（`Call(argc)` アーム全体を以下に置き換え）:**
```favnir
      // Call: collect_args でスタックから argc 個の引数をソース順で収集
      // （push callee → push arg1 → ... → push argN → Call(N)）
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

---

## T1 事後確認

```bash
cargo check --bin fav
# → エラー 0 であること

grep -n "fn collect_args_rec\|fn collect_args\|fn vmval_display\|GetField\|String\.concat" fav/self/vm.fav | head -20
# → 追加された関数・実装が存在すること

grep -n "Phase 4 以降\|not yet supported" fav/self/vm.fav
# → 0 件（古いコメントが残っていないこと）

# 後方互換性確認
cargo test v237000 --bin fav
# → 5/5 PASS であること
```

---

## T2: `fav/src/driver.rs` — `v238000_tests` 追加

### T2-1: `v237000_tests::version_is_23_7_0` を削除（T3-1 より前に必須）

`#[ignore]` を追加し続けると不要テストが蓄積するため、代わりに前バージョンの `version_is_X_Y_Z` テストを**削除**する方針に変更。

`v237000_tests` モジュール内の以下のテスト関数ごと削除する:

```rust
    #[test]
    fn version_is_23_7_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("version = \"23.7.0\""), "Cargo.toml should have version 23.7.0");
    }
```

### T2-2: `v238000_tests` モジュールを `v237000_tests` の直後に追加（6 件）

```rust
// ── v238000_tests (v23.8.0) — vm.fav Phase 5（GetField・collect_args・hello.fav 実行）──────
#[cfg(test)]
mod v238000_tests {
    use super::*;

    #[test]
    fn version_is_23_8_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("version = \"23.8.0\""), "Cargo.toml should have version 23.8.0");
    }

    #[test]
    fn vm_fav_phase5_compiles() {
        let src = include_str!("../self/vm.fav");
        let tokens = crate::frontend::lexer::Lexer::new(src, "vm.fav")
            .tokenize().expect("lex vm.fav");
        let prog = crate::frontend::parser::Parser::new(tokens)
            .parse_program().expect("parse vm.fav");
        let _artifact = build_artifact(&prog);
    }

    #[test]
    fn execute_hello_via_vm() {
        // LoadGlobal(0)=VMStr("hello") + Return
        // Bytecode hex: "12000016"
        //   12 00 00  LoadGlobal(0)  → push VMStr("hello")
        //   16        Return          → VMStr("hello")
        // vmval_display(VMStr("hello")) = "hello"
        let vm_src = include_str!("../self/vm.fav");
        let src = format!(r#"{}
public fn main() -> String {{
  bind globals <- Mut.map()
  bind _ <- Mut.set(globals, 0, VMStr("hello"))
  bind hex_r <- Bytes.from_hex("12000016")
  match hex_r {{
    ok(bytes) => {{
      bind run_r <- vm_run_named(bytes, globals)
      match run_r {{
        ok(v)  => vmval_display(v)
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
        assert_eq!(result, crate::value::Value::Str("hello".to_string()),
            "vmval_display(VMStr(\"hello\")) should return \"hello\"");
    }

    #[test]
    fn execute_getfield_call() {
        // LoadGlobal(0)="String" + GetField(1)="trim" + LoadGlobal(2)=" hi " + Call(1) + Return
        // Bytecode hex: "12000040010012020015010016"
        //   12 00 00  LoadGlobal(0)  → push VMStr("String")
        //   40 01 00  GetField(1)    → pop "String" + globals[1]="trim" → push VMStr("String.trim")
        //   12 02 00  LoadGlobal(2)  → push VMStr(" hi ")
        //   15 01 00  Call(1)        → collect_args(1) → args=[VMStr(" hi ")]
        //                               pop callee VMStr("String.trim")
        //                               call_builtin("String.trim", args) → VMStr("hi")
        //   16        Return          → VMStr("hi")
        // vmval_display(VMStr("hi")) = "hi"
        let vm_src = include_str!("../self/vm.fav");
        let src = format!(r#"{}
public fn main() -> String {{
  bind globals <- Mut.map()
  bind _ <- Mut.set(globals, 0, VMStr("String"))
  bind _ <- Mut.set(globals, 1, VMStr("trim"))
  bind _ <- Mut.set(globals, 2, VMStr(" hi "))
  bind hex_r <- Bytes.from_hex("12000040010012020015010016")
  match hex_r {{
    ok(bytes) => {{
      bind run_r <- vm_run_named(bytes, globals)
      match run_r {{
        ok(v)  => vmval_display(v)
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
        assert_eq!(result, crate::value::Value::Str("hi".to_string()),
            "GetField(\"trim\") + Call(1) should call String.trim(\" hi \") -> \"hi\"");
    }

    #[test]
    fn execute_string_concat() {
        // LoadGlobal(0)="String" + GetField(1)="concat" + LoadGlobal(2)="hello" + LoadGlobal(3)=" world" + Call(2) + Return
        // Bytecode hex: "12000040010012020012030015020016"
        //   12 00 00  LoadGlobal(0)  → push VMStr("String")
        //   40 01 00  GetField(1)    → pop "String" + globals[1]="concat" → push VMStr("String.concat")
        //   12 02 00  LoadGlobal(2)  → push VMStr("hello")
        //   12 03 00  LoadGlobal(3)  → push VMStr(" world")
        //   15 02 00  Call(2)        → collect_args(2)
        //                               step1: pop " world" → acc=[" world"]
        //                               step2: pop "hello"  → acc=[" world", "hello"]  ← "hello" is top
        //                               pop callee "String.concat"
        //                               call_builtin: pop "hello" (arg1), pop " world" (arg2)
        //                               String.concat("hello", " world") → "hello world"
        //   16        Return          → VMStr("hello world")
        let vm_src = include_str!("../self/vm.fav");
        let src = format!(r#"{}
public fn main() -> String {{
  bind globals <- Mut.map()
  bind _ <- Mut.set(globals, 0, VMStr("String"))
  bind _ <- Mut.set(globals, 1, VMStr("concat"))
  bind _ <- Mut.set(globals, 2, VMStr("hello"))
  bind _ <- Mut.set(globals, 3, VMStr(" world"))
  bind hex_r <- Bytes.from_hex("12000040010012020012030015020016")
  match hex_r {{
    ok(bytes) => {{
      bind run_r <- vm_run_named(bytes, globals)
      match run_r {{
        ok(v)  => vmval_display(v)
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
        assert_eq!(result, crate::value::Value::Str("hello world".to_string()),
            "collect_args(2) should preserve source order: String.concat(\"hello\", \" world\") -> \"hello world\"");
    }

    #[test]
    fn changelog_has_v23_8_0() {
        let cl = include_str!("../../CHANGELOG.md");
        assert!(cl.contains("[v23.8.0]"), "CHANGELOG.md should have [v23.8.0] entry");
    }
}
```

---

## T3: Cargo.toml + CHANGELOG + benchmarks + docs

> **注意**: T2-1 の `#[ignore]` 追加完了後に Cargo.toml を更新すること。

### T3-1: `fav/Cargo.toml` バージョン更新

```
version = "23.7.0" → "23.8.0"
```

### T3-2: `CHANGELOG.md` 先頭に v23.8.0 エントリ追加

```markdown
## [v23.8.0] — 2026-06-22 — vm.fav Phase 5（GetField・collect_args・hello.fav 実行）

### Added
- `vm.fav` Phase 5: GetField・多引数 Call・vmval_display
  - `fn collect_args_rec(stack: Int, n: Int, acc: Int) -> Result<Int, String>` 追加
  - `fn collect_args(stack: Int, n: Int) -> Result<Int, String>` 追加
  - `GetField(idx)` オペコード: namespace VMStr + globals[idx]=field VMStr → push "ns.field" VMStr
  - `Call(argc)` ハンドラを `collect_args` 利用の汎用実装に置換（任意の argc に対応）
  - `fn vmval_display(v: VMVal) -> String` 追加（ユーザー向け表示: VMStr は引用符なし）
  - `call_builtin` に `"String.concat"` 追加（2 引数 builtin の例）

### Notes
- `LoadGlobal + GetField + Call(N)` シーケンス完成: 任意の builtin 呼び出しチェーンが vm.fav 上で動作
- `fav run --vm=<path>` CLI フラグは v24.0 で実装予定
```

### T3-3: `benchmarks/v23.8.0.json` 作成

```json
{
  "version": "23.8.0",
  "date": "2026-06-22",
  "test_count": 0,
  "feature": "vm.fav Phase 5（GetField・collect_args・hello.fav 実行）",
  "metrics": {
    "vm_fav_phase": 5,
    "call_builtin_count": 5,
    "new_opcodes": 1,
    "collect_args_helpers": 2,
    "vmval_variants": 4
  }
}
```

> `test_count` は `cargo test --bin fav` 実行後に実際の件数で更新すること。

### T3-4: `site/content/docs/tools/vm-fav.mdx` に Phase 5 セクション追記

フェーズ表の `Phase 5` 行を更新:

**変更前:**
```
| Phase 5 | GetField・多引数 Call・VMRecord・完全セルフホスト VM |
```

**変更後:**
```
| Phase 5（v23.8.0） | GetField・collect_args・vmval_display・String.concat builtin |
| Phase 6 | VMRecord・完全セルフホスト VM |
```

また、フェーズ 4 セクションの後に以下の Phase 5 セクションを追記:

```markdown
## Phase 5（v23.8.0）— GetField・collect_args・hello.fav 実行

Phase 5 では `LoadGlobal + GetField + Call(N)` の完全なビルトイン呼び出しシーケンスを実現します。

```favnir
// collect_args: スタックから N 個の引数をソース順で収集
fn collect_args(stack: Int, n: Int) -> Result<Int, String>

// GetField: Namespace.Field 名を動的に合成
// スタック前: ..., VMStr("String")
// スタック後: ..., VMStr("String.trim")
GetField(idx) => pop VMStr(ns) + globals[idx]=VMStr(field) → push VMStr("ns.field")

// vmval_display: ユーザー向け表示（VMStr は引用符なし）
fn vmval_display(v: VMVal) -> String
```

### 対応オペコード（Phase 5: 1 件）

| オペコード | バイト | 動作 |
|---|---|---|
| `GetField(idx)` | `0x40 u16LE` | pop VMStr(ns) + globals[idx]=VMStr(field) → push VMStr("ns.field") |

### 引数順の不変条件

```
Favnir ソース: String.trim(" hi ")
コンパイル後:
  1. LoadGlobal(0)="String" → push VMStr("String")
  2. GetField(1)="trim"    → push VMStr("String.trim")
  3. LoadGlobal(2)=" hi "  → push VMStr(" hi ")
  4. Call(1)               → collect_args(1) → args=[VMStr(" hi ")]
                              pop callee "String.trim"
                              call_builtin("String.trim", args) → VMStr("hi")
```
```

---

## 実装順序

```
T0（事前確認）
  └─ GetField=0x40, String.concat 可用性確認
T1-1（collect_args_rec / collect_args 追加） ← call_builtin の直後、vm_execute の直前
T1-2（vmval_display 追加）                   ← vmval_to_string の直後
T1-3（call_builtin に String.concat 追加）   ← _ アームの直前
T1-4a（GetField 実装）                       ← // GetField は Phase 5 以降 コメントを置換
T1-4b（Call(argc) 汎用化）                   ← 既存 match argc { 0/1/_ } を置換
cargo check → エラー 0 確認
T2-1（version_is_23_7_0 削除）               ← T3-1 より前に必須
T2-2（tests）                                ← T1 完了後
cargo test v238000 → 5/5 PASS 確認
T3-1（version） ← T2-1 完了後
T3-2〜4（docs）← T3-1 完了後
cargo test --bin fav → リグレッションなし確認（1921 件以上）
```

---

## リスク対応表

| リスク | 検出方法 | 対応 |
|---|---|---|
| `collect_args_rec` の `match n { 0 => ... _ => ... }` が Int パターン不可 | パースエラー | Favnir は整数リテラルの match をサポート（Div アームの `match bi { 0 => ... _ => ... }` で確認済み） |
| `GetField` の `String.concat(ns, String.concat(".", field))` が型エラー | cargo check 失敗 | f-string で String に引用符が付く問題を確認済み。String.concat 固定 |
| `Call(argc)` 置換で既存 Call(0)/Call(1) テストが壊れる | cargo test v237000 失敗 | collect_args(stack, 0) が空リスト返却 → call_builtin(name, []) は Int.to_string などで pop 失敗 → テストコードが Call(1) のため問題なし |
| `bind _ <- Mut.push(acc, v)` での sequencing | コンパイルエラー | `Mut.push` は `Result<Unit, String>` 返却。`bind _ <-` で sequencing（v23.7.0 と同パターン） |
| `vmval_display` の `VMStr(s) => s` が型エラー | checker 型エラー | s は String 型 → String を返す関数の末尾式として有効 |
| `GetField` f-string での idx interpolation | 文字列に整数が入らない | `f"GetField: globals[{idx}] not found: {e}"` は Int の f-string 展開 → 有効 |
