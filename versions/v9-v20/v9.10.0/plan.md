# Favnir v9.10.0 Plan

Date: 2026-06-02
Theme: `fav repl`

---

## Phase A: `vm.rs` — `Debug.show_raw` 追加

### A-1: `display_vmvalue` ヘルパー関数

`vm.rs` に `VMValue` を人間可読な文字列に変換する内部関数を追加する。

```rust
fn display_vmvalue(v: &VMValue) -> String {
    match v {
        VMValue::Int(n)     => n.to_string(),
        VMValue::Float(f)   => format_float(*f),
        VMValue::Str(s)     => format!("\"{}\"", s.replace('"', "\\\"")),
        VMValue::Bool(b)    => b.to_string(),
        VMValue::Unit       => "()".to_string(),
        VMValue::List(xs)   => {
            let items: Vec<String> = xs.iter().map(display_vmvalue).collect();
            format!("[{}]", items.join(", "))
        }
        VMValue::Record(fields) => {
            let pairs: Vec<String> = fields.iter()
                .map(|(k, v)| format!("{}: {}", k, display_vmvalue(v))).collect();
            format!("{{{}}}", pairs.join(", "))
        }
        VMValue::Variant(tag, None)        => tag.clone(),
        VMValue::Variant(tag, Some(inner)) => {
            // normalize lowercase tags: "ok" -> "Ok", "some" -> "Some" etc.
            let display_tag = capitalize_tag(tag);
            format!("{}({})", display_tag, display_vmvalue(inner))
        }
        VMValue::Closure(..) => "<fn>".to_string(),
        _                    => "<value>".to_string(),
    }
}
```

variant タグの正規化（`"ok"` → `"Ok"` など）は既存の normalize_variant_tag の逆操作。

### A-2: `Debug.show_raw` primitive 追加

```rust
"Debug.show_raw" => {
    // 引数: v: Unknown → 任意の VMValue
    let v = args.into_iter().next()
        .ok_or_else(|| "Debug.show_raw: missing argument".to_string())?;
    Ok(VMValue::Str(display_vmvalue(&v)))
}
```

`IO.argv` の前あたりに追加。`Unknown` 型を受け取るため型チェックは不要。

### A-3: `checker.fav` に `Debug` namespace エントリを追加

```favnir
fn debug_fn(fname: String) -> String {
    if fname == "show_raw" { "String" }
    else { "Unknown" }
}
```

`builtin_ret_ty` に `else if ns == "Debug" { debug_fn(fname) }` を追加。
`ns_to_effect` に `else if ns == "Debug" { "IO" }` を追加（表示は副作用なしだが念のため IO に）。

---

## Phase B: `driver.rs` — `cmd_repl` 実装

### B-1: `ReplSession` 構造体

```rust
struct ReplSession {
    /// 累積した定義のソーステキスト（fn/stage/type/seq）
    definitions: String,
    /// 定義済み名前のリスト（:env 表示用）
    def_names: Vec<String>,
}

impl ReplSession {
    fn new() -> Self { ... }
    fn reset(&mut self) { ... }
    fn add_definition(&mut self, src: &str, name: &str) { ... }
}
```

### B-2: `is_definition(line: &str) -> bool`

先頭トークンが `fn` / `public fn` / `stage` / `seq` / `type` / `effect` かどうかで判定。

### B-3: `extract_def_name(line: &str) -> Option<String>`

定義行から名前を取り出す（`:env` および `defined: <name>` 表示用）:
- `fn foo(...)` → `"foo"`
- `stage Bar: ...` → `"Bar"`
- `type Baz = ...` → `"Baz"`

正規表現不使用、先頭トークン列の単純パースで十分。

### B-4: `build_eval_source(session: &ReplSession, expr: &str) -> String`

```rust
fn build_eval_source(session: &ReplSession, expr: &str) -> String {
    format!(
        "{}\nfn main() -> Unit = IO.println(Debug.show_raw({}))\n",
        session.definitions,
        expr
    )
}
```

### B-5: `handle_repl_input(line: &str, session: &mut ReplSession)`

1. `is_definition(line)` が true → `handle_definition`
2. そうでなければ → `handle_expression`

**`handle_definition`**:
1. `session.definitions + line` をコンパイル（型チェックのみ）: `check_source_str`
2. エラーなし → `session.add_definition(line, name)` して `defined: <name>` を表示
3. エラー → エラーメッセージを表示、セッションは変更しない

**`handle_expression`**:
1. `build_eval_source(session, expr)` でソース生成
2. Favnir pipeline（checker.fav + compiler.fav）でコンパイル: `compile_src_str_to_bytes`
3. `run_fvc_bytes` で実行（stdout に結果が出力される）
4. エラーはキャッチして表示、セッションは変更しない

### B-6: `handle_type_cmd(expr: &str, session: &ReplSession)`

```rust
fn handle_type_cmd(expr: &str, session: &ReplSession) {
    let probe_src = format!(
        "{}\nfn _type_probe_() -> ___PROBE___ = {}\n",
        session.definitions, expr
    );
    // checker.fav を実行して E0009 のエラーメッセージから型を抽出
    match check_source_str(&probe_src) {
        Ok(_) => println!("(unknown)"),  // チェックが通るなら型不明
        Err(msg) => {
            // E0009: _type_probe_: return type mismatch: declared ___PROBE___, inferred <TYPE>
            if let Some(ty) = extract_inferred_type(&msg) {
                println!("{}", ty);
            } else {
                println!("(unknown)");
            }
        }
    }
}

fn extract_inferred_type(err_msg: &str) -> Option<String> {
    // E0009 のフォーマット: "... inferred: <TYPE>" を探す
    // checker.fav の E0009 メッセージ形式に依存
    err_msg.find("inferred: ")
        .map(|i| err_msg[i + 10..].split_whitespace().next()?.to_string())
}
```

型抽出が失敗しても `(unknown)` にフォールバックするため、壊れない。

### B-7: `print_repl_help()`

```
Commands:
  :help          show this help
  :quit / :q     exit the REPL
  :reset         clear all session definitions
  :env           show accumulated definitions
  :type <expr>   show the type of an expression

Enter expressions to evaluate, or fn/stage/type definitions to add to the session.
```

### B-8: `cmd_repl()` メインループ

stdin のノーマルライン読み取り（`BufRead::read_line`）。
`rustyline` 等の外部クレートは使用しない（v9.10.0 スコープ外）。

### B-9: `v9100_tests` モジュール

最低 3 件:

- `repl_eval_arithmetic` — `build_eval_source` が正しいソースを生成し、コンパイル・実行が成功する
- `repl_definition_accumulates` — 関数定義を追加後、その関数を呼ぶ式が評価できる
- `repl_error_recovery` — コンパイルエラーが発生してもセッション定義が変化しない

---

## Phase C: `main.rs` — `repl` サブコマンド dispatch

```rust
"repl" => {
    cmd_repl();
}
```

`cmd_repl` を use に追加。引数・フラグなし（v9.10.0 では `--db` 等は未対応）。

---

## Phase D: `cli.fav` — `CmdRepl` ルーティング

### D-1: `CliCmd` に `| CmdRepl` を追加

引数なし（unit variant）。

### D-2: `parse_repl_cmd(args: List<String>) -> CliCmd`

```favnir
fn parse_repl_cmd(args: List<String>) -> CliCmd {
    CmdRepl
}
```

### D-3: `parse_named_cmd` に `"repl"` 分岐を追加

```favnir
else if cmd == "repl" { parse_repl_cmd(rest) }
```

### D-4: `main` の match に `CmdRepl` arm を追加

```favnir
CmdRepl => run_repl()
```

### D-5: `run_repl() -> Unit !IO`

```favnir
fn run_repl() -> Unit !IO =
    IO.println("Use 'fav repl' to start the REPL.")
```

`fav repl` は Rust の `cmd_repl()` が直接処理するため、
cli.fav の `run_repl` は通常呼ばれない（main.rs で先に dispatch する）。
cli.fav 経由（`fav run cli.fav repl` のようなケース）のフォールバック用。

### D-6: `run_help` に repl 行を追加

```
  repl             start interactive REPL
```

---

## Phase E: テスト + バージョン更新

### E-1: `cargo test v9100` — 3 件通過確認

### E-2: セルフチェック

```
cargo test checker_fav_wire_self_check
cargo test bootstrap
cargo test
```

### E-3: バージョン更新

- `fav/Cargo.toml`: `"9.9.0"` → `"9.10.0"`
- `fav/self/cli.fav`: `run_version` → `"9.10.0"`
- `memory/MEMORY.md`: v9.10.0 完了記録

### E-4: commit

---

## 実装順序の依存関係

```
A (Debug.show_raw + checker.fav)
  └── B (driver.rs cmd_repl — show_raw を使う)
        └── C (main.rs dispatch)
              └── D (cli.fav CmdRepl)
                    └── E (tests + version)
```

A は独立して先行実装可能。B-5 の `check_source_str` / `compile_src_str_to_bytes` は
v8.11.0 以降で実装済みのため再利用。

---

## リスクと注意点

### `:type` の精度

checker.fav の E0009 メッセージフォーマット:

```
E0009: _type_probe_: return type mismatch: declared ___PROBE___, inferred <TYPE>
```

このフォーマットが安定していれば型抽出が機能する。
フォーマットが変わると `(unknown)` にフォールバックするだけなので壊れない。
v9.10.0 では「動けばよい」レベルで可。

### テール再帰なし

driver.rs のメインループは Rust の `loop {}` なので
スタックオーバーフローの心配はない。
watch_loop（cli.fav の再帰）とは異なりここは Rust ループで十分。

### `Debug.show_raw` の型

`Unknown` 型の引数を取るため、checker.fav / checker.rs に特別扱いが必要かもしれない。
checker.fav の `builtin_ret_ty` に `Debug` エントリを追加して `"String"` を返せば、
型チェックは通る（引数型は `Unknown` なので任意の値を渡せる）。

### 標準ライブラリのない式

`1 + 2` は内部ではバイナリ演算子であり、`List.map` 等の stdlib 関数は
Favnir の stdlib rune から提供される。
REPL のビルドソースには stdlib import を暗黙的に含める必要があるか確認する。
→ 既存の `compile_src_str_to_bytes` が stdlib を自動結合するなら問題なし。
