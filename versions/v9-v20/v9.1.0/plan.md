# Favnir v9.1.0 実装計画

Date: 2026-05-31

---

## Phase A: `rvm` 独立バイナリ

**変更ファイル**: `fav/Cargo.toml`, `fav/src/vm.rs`, `fav/src/bin/rvm.rs`（新規）

### A-1: `VM_VERSION` 定数を `vm.rs` に追加

```rust
// fav/src/vm.rs — 先頭付近に追加
pub const VM_VERSION: &str = "1.0.0";
```

### A-2: `Cargo.toml` に `[[bin]]` エントリ追加

```toml
# fav/Cargo.toml — 既存の [[bin]] name="fav" の後に追加
[[bin]]
name = "rvm"
path = "src/bin/rvm.rs"
```

### A-3: `fav/src/bin/rvm.rs` 新規作成

```rust
use fav::vm::VM_VERSION;
use fav::driver;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() == 1 {
        eprintln!("Usage: rvm [--version] [--help] [--db <url>] <file.fvc>");
        std::process::exit(1);
    }

    let mut db_url: Option<String> = None;
    let mut file: Option<String> = None;
    let mut i = 1;

    while i < args.len() {
        match args[i].as_str() {
            "--version" => {
                println!("Favnir VM {}", VM_VERSION);
                return;
            }
            "--help" => {
                println!("Usage: rvm [--version] [--help] [--db <url>] <file.fvc>");
                println!("  --version    Print VM version and exit");
                println!("  --db <url>   Database connection URL");
                println!("  <file.fvc>   Bytecode file to execute");
                return;
            }
            "--db" => {
                i += 1;
                if i < args.len() {
                    db_url = Some(args[i].clone());
                }
            }
            f => {
                file = Some(f.to_string());
            }
        }
        i += 1;
    }

    let file = match file {
        Some(f) => f,
        None => {
            eprintln!("rvm: no bytecode file specified");
            std::process::exit(1);
        }
    };

    driver::cmd_exec(&file, db_url.as_deref());
}
```

注: `driver::cmd_exec` は `fav exec` と同じ関数を呼び出す。`pub` 可視性を確認すること。

---

## Phase B: stdlib Favnir 実装

**変更ファイル**:
- `fav/self/stdlib/list_stdlib.fav`（既存に追記）
- `fav/self/stdlib/string_stdlib.fav`（既存に追記）
- `fav/self/stdlib/map_stdlib.fav`（新規）
- `fav/self/stdlib/result_stdlib.fav`（新規）

### B-1: `list_stdlib.fav` — List 11 関数追加

既存の `intersperse` の後に追記:

```favnir
// List.chunk: n 件ずつ分割
fn list_chunk_inner(xs: List<A>, n: Int, acc: List<A>, result: List<List<A>>) -> List<List<A>> {
    match List.first(xs) {
        None =>
            if List.length(acc) == 0 { result }
            else { List.push(result, acc) }
        Some(x) => {
            bind next_acc <- List.push(acc, x)
            if List.length(next_acc) == n {
                list_chunk_inner(List.drop(xs, 1), n, List.empty(), List.push(result, next_acc))
            } else {
                list_chunk_inner(List.drop(xs, 1), n, next_acc, result)
            }
        }
    }
}
public fn chunk(xs: List<A>, n: Int) -> List<List<A>> {
    list_chunk_inner(xs, n, List.empty(), List.empty())
}

// List.flat_map: map + concat
public fn flat_map(f: A -> List<B>, xs: List<A>) -> List<B> {
    List.concat(List.map(xs, f))
}

// List.group_by: キーで分類
// (実装は List.fold ベースで kv list を構築)
fn group_by_insert(key: String, val: A, groups: List<{key: String, values: List<A>}>) -> List<{key: String, values: List<A>}> {
    match List.find(groups, |g| g.key == key) {
        None =>
            List.push(groups, { key: key, values: List.singleton(val) })
        Some(_) =>
            List.map(groups, |g|
                if g.key == key { { key: g.key, values: List.push(g.values, val) } }
                else { g })
    }
}
public fn group_by(f: A -> String, xs: List<A>) -> List<{key: String, values: List<A>}> {
    List.foldl(xs, List.empty(), |acc, x| group_by_insert(f(x), x, acc))
}

// List.zip_with: 2リストを f で合成
fn zip_with_inner(f: A -> B -> C, xs: List<A>, ys: List<B>, acc: List<C>) -> List<C> {
    match List.first(xs) {
        None => acc
        Some(x) => match List.first(ys) {
            None => acc
            Some(y) =>
                zip_with_inner(f, List.drop(xs, 1), List.drop(ys, 1),
                    List.push(acc, f(x)(y)))
        }
    }
}
public fn zip_with(f: A -> B -> C, xs: List<A>, ys: List<B>) -> List<C> {
    zip_with_inner(f, xs, ys, List.empty())
}

// List.take_while: 条件を満たす間 take
fn take_while_inner(pred: A -> Bool, xs: List<A>, acc: List<A>) -> List<A> {
    match List.first(xs) {
        None => acc
        Some(x) =>
            if pred(x) { take_while_inner(pred, List.drop(xs, 1), List.push(acc, x)) }
            else { acc }
    }
}
public fn take_while(pred: A -> Bool, xs: List<A>) -> List<A> {
    take_while_inner(pred, xs, List.empty())
}

// List.drop_while: 条件を満たす間 drop
public fn drop_while(pred: A -> Bool, xs: List<A>) -> List<A> {
    match List.first(xs) {
        None => xs
        Some(x) =>
            if pred(x) { drop_while(pred, List.drop(xs, 1)) }
            else { xs }
    }
}

// List.unique: 順序保持で重複除去
fn unique_inner(xs: List<A>, seen: List<A>, acc: List<A>) -> List<A> {
    match List.first(xs) {
        None => acc
        Some(x) =>
            if List.contains(seen, x) {
                unique_inner(List.drop(xs, 1), seen, acc)
            } else {
                unique_inner(List.drop(xs, 1), List.push(seen, x), List.push(acc, x))
            }
    }
}
public fn unique(xs: List<A>) -> List<A> {
    unique_inner(xs, List.empty(), List.empty())
}

// List.count: 条件を満たす個数
public fn count(pred: A -> Bool, xs: List<A>) -> Int {
    List.length(List.filter(xs, pred))
}

// List.sum: 合計（Int版）
public fn sum(xs: List<Int>) -> Int {
    List.foldl(xs, 0, |acc, x| acc + x)
}

// List.min: 最小値
public fn min(xs: List<Int>) -> Option<Int> {
    match List.first(xs) {
        None => None
        Some(h) => Some(List.foldl(List.drop(xs, 1), h, |acc, x| if x < acc { x } else { acc }))
    }
}

// List.max: 最大値
public fn max(xs: List<Int>) -> Option<Int> {
    match List.first(xs) {
        None => None
        Some(h) => Some(List.foldl(List.drop(xs, 1), h, |acc, x| if x > acc { x } else { acc }))
    }
}
```

### B-2: `string_stdlib.fav` — String 7 関数追加

既存の `capitalize` / `indent` の後に追記:

```favnir
// String.pad_left: 左パディング
public fn pad_left(s: String, width: Int, pad: String) -> String {
    bind len <- String.length(s)
    if len >= width { s }
    else { String.concat(String.repeat(pad, width - len), s) }
}

// String.pad_right: 右パディング
public fn pad_right(s: String, width: Int, pad: String) -> String {
    bind len <- String.length(s)
    if len >= width { s }
    else { String.concat(s, String.repeat(pad, width - len)) }
}

// String.truncate: 末尾省略
public fn truncate(s: String, max_len: Int, suffix: String) -> String {
    bind len <- String.length(s)
    if len <= max_len { s }
    else { String.concat(String.take(s, max_len - String.length(suffix)), suffix) }
}

// String.repeat: 文字列の繰り返し
fn repeat_inner(s: String, n: Int, acc: String) -> String {
    if n <= 0 { acc }
    else { repeat_inner(s, n - 1, String.concat(acc, s)) }
}
public fn repeat(s: String, n: Int) -> String {
    repeat_inner(s, n, "")
}

// String.trim_start: 先頭の空白除去
public fn trim_start(s: String) -> String {
    if String.length(s) == 0 { s }
    else {
        bind first <- String.take(s, 1)
        if first == " " || first == "\t" { trim_start(String.drop(s, 1)) }
        else { s }
    }
}

// String.trim_end: 末尾の空白除去
public fn trim_end(s: String) -> String {
    bind len <- String.length(s)
    if len == 0 { s }
    else {
        bind last <- String.drop(s, len - 1)
        if last == " " || last == "\t" { trim_end(String.take(s, len - 1)) }
        else { s }
    }
}

// String.replace: 部分文字列置換
public fn replace(s: String, from: String, to: String) -> String {
    bind parts <- String.split(s, from)
    String.join(parts, to)
}
```

### B-3: `map_stdlib.fav` — 新規作成

```favnir
namespace Map

// Map.merge_with: 同一キーを f で解決
public fn merge_with(f: A -> A -> A, a: Map<String,A>, b: Map<String,A>) -> Map<String,A> {
    bind b_list <- Map.to_list(b)
    List.foldl(b_list, a, |acc, entry|
        match Map.get(acc, entry.key) {
            None    => Map.set(acc, entry.key, entry.value)
            Some(v) => Map.set(acc, entry.key, f(v)(entry.value))
        })
}

// Map.filter: エントリを絞り込む
public fn filter(pred: String -> A -> Bool, m: Map<String,A>) -> Map<String,A> {
    bind entries <- Map.to_list(m)
    bind kept    <- List.filter(entries, |e| pred(e.key)(e.value))
    Map.from_list(kept)
}

// Map.map_values: 値を変換
public fn map_values(f: A -> B, m: Map<String,A>) -> Map<String,B> {
    bind entries <- Map.to_list(m)
    Map.from_list(List.map(entries, |e| { key: e.key, value: f(e.value) }))
}

// Map.from_list: リストから Map 構築
public fn from_list(entries: List<{key: String, value: A}>) -> Map<String,A> {
    List.foldl(entries, Map.empty(), |acc, e| Map.set(acc, e.key, e.value))
}

// Map.to_list: Map をリスト化（Rust primitive にディスパッチ）
public fn to_list(m: Map<String,A>) -> List<{key: String, value: A}> {
    Map.entries(m)
}
```

### B-4: `result_stdlib.fav` — 新規作成

```favnir
namespace Result

// Result.map_err: エラー側を変換
public fn map_err(f: E -> F, r: Result<A,E>) -> Result<A,F> {
    match r {
        Ok(v)  => Ok(v)
        Err(e) => Err(f(e))
    }
}

// Result.and_then: モナド bind
public fn and_then(f: A -> Result<B,E>, r: Result<A,E>) -> Result<B,E> {
    match r {
        Ok(v)  => f(v)
        Err(e) => Err(e)
    }
}

// Result.all: 全成功 or 最初のエラー
fn all_inner(xs: List<Result<A,E>>, acc: List<A>) -> Result<List<A>,E> {
    match List.first(xs) {
        None    => Ok(acc)
        Some(r) => match r {
            Err(e) => Err(e)
            Ok(v)  => all_inner(List.drop(xs, 1), List.push(acc, v))
        }
    }
}
public fn all(xs: List<Result<A,E>>) -> Result<List<A>,E> {
    all_inner(xs, List.empty())
}

namespace Option

// Option.map: Some 内を変換
public fn map(f: A -> B, o: Option<A>) -> Option<B> {
    match o {
        None    => None
        Some(v) => Some(f(v))
    }
}

// Option.and_then: モナド bind
public fn and_then(f: A -> Option<B>, o: Option<A>) -> Option<B> {
    match o {
        None    => None
        Some(v) => f(v)
    }
}

// Option.unwrap_or: デフォルト値付き unwrap
public fn unwrap_or(default: A, o: Option<A>) -> A {
    match o {
        None    => default
        Some(v) => v
    }
}

// Option.is_some / is_none
public fn is_some(o: Option<A>) -> Bool {
    match o { None => false  Some(_) => true }
}
public fn is_none(o: Option<A>) -> Bool {
    match o { None => true  Some(_) => false }
}
```

---

## Phase C: 型シグネチャ登録

### C-1: `checker.rs` — Rust チェッカーへの追加

**変更ファイル**: `fav/src/middle/checker.rs`

`builtins()` 関数の `list_fns` / `string_fns` 配列に追加:

```rust
// List 追加分
("List.flat_map",   "(A -> List<B>) -> List<A> -> List<B>"),
("List.zip_with",   "(A -> B -> C) -> List<A> -> List<B> -> List<C>"),
("List.take_while", "(A -> Bool) -> List<A> -> List<A>"),
("List.drop_while", "(A -> Bool) -> List<A> -> List<A>"),
("List.unique",     "List<A> -> List<A>"),
("List.count",      "(A -> Bool) -> List<A> -> Int"),
("List.sum",        "List<Int> -> Int"),
("List.min",        "List<Int> -> Option<Int>"),
("List.max",        "List<Int> -> Option<Int>"),
("List.chunk",      "List<A> -> Int -> List<List<A>>"),
("List.group_by",   "(A -> String) -> List<A> -> List<{key: String, values: List<A>}>"),

// String 追加分
("String.pad_left",   "String -> Int -> String -> String"),
("String.pad_right",  "String -> Int -> String -> String"),
("String.truncate",   "String -> Int -> String -> String"),
("String.repeat",     "String -> Int -> String"),
("String.trim_start", "String -> String"),
("String.trim_end",   "String -> String"),
("String.replace",    "String -> String -> String -> String"),

// Map 追加分
("Map.merge_with",  "(A -> A -> A) -> Map<String,A> -> Map<String,A> -> Map<String,A>"),
("Map.filter",      "(String -> A -> Bool) -> Map<String,A> -> Map<String,A>"),
("Map.map_values",  "(A -> B) -> Map<String,A> -> Map<String,B>"),
("Map.from_list",   "List<{key: String, value: A}> -> Map<String,A>"),
("Map.to_list",     "Map<String,A> -> List<{key: String, value: A}>"),

// Result / Option 追加分
("Result.map_err",  "(E -> F) -> Result<A,E> -> Result<A,F>"),
("Result.and_then", "(A -> Result<B,E>) -> Result<A,E> -> Result<B,E>"),
("Result.all",      "List<Result<A,E>> -> Result<List<A>,E>"),
("Option.map",      "(A -> B) -> Option<A> -> Option<B>"),
("Option.and_then", "(A -> Option<B>) -> Option<A> -> Option<B>"),
("Option.unwrap_or","A -> Option<A> -> A"),
("Option.is_some",  "Option<A> -> Bool"),
("Option.is_none",  "Option<A> -> Bool"),
```

### C-2: `checker.fav` — self-hosted チェッカーへの追加

**変更ファイル**: `fav/self/checker.fav`

`builtin_ret_ty` 関数の `list_fn` / `string_fn` マッチに追加:

```favnir
// list_fn の追加エントリ（既存 map/filter/... の後に）
"List.flat_map"   => Some("List<B>")
"List.zip_with"   => Some("List<C>")
"List.take_while" => Some("List<A>")
"List.drop_while" => Some("List<A>")
"List.unique"     => Some("List<A>")
"List.count"      => Some("Int")
"List.sum"        => Some("Int")
"List.min"        => Some("Option<Int>")
"List.max"        => Some("Option<Int>")
"List.chunk"      => Some("List<List<A>>")
"List.group_by"   => Some("List<A>")   // 簡略型

// string_fn の追加エントリ
"String.pad_left"   => Some("String")
"String.pad_right"  => Some("String")
"String.truncate"   => Some("String")
"String.repeat"     => Some("String")
"String.trim_start" => Some("String")
"String.trim_end"   => Some("String")
"String.replace"    => Some("String")

// map_fn（新規ブランチ）
fn is_map_fn(name: String) -> Bool {
    name == "Map.merge_with" || name == "Map.filter" || name == "Map.map_values" ||
    name == "Map.from_list"  || name == "Map.to_list" || name == "Map.empty"   ||
    name == "Map.get"        || name == "Map.set"     || name == "Map.entries"
}
fn map_fn_ret(name: String) -> Option<String> {
    if name == "Map.to_list" || name == "Map.entries" { Some("List<A>") }
    else if name == "Map.get"                          { Some("Option<A>") }
    else if name == "Map.from_list"                    { Some("Map<String,A>") }
    else                                               { Some("Map<String,A>") }
}

// result_fn（新規ブランチ）
fn is_result_fn(name: String) -> Bool {
    name == "Result.map_err"  || name == "Result.and_then" || name == "Result.all" ||
    name == "Option.map"      || name == "Option.and_then" || name == "Option.unwrap_or" ||
    name == "Option.is_some"  || name == "Option.is_none"
}
fn result_fn_ret(name: String) -> Option<String> {
    if name == "Result.all"      { Some("Result<List<A>,E>") }
    else if name == "Option.is_some" || name == "Option.is_none" { Some("Bool") }
    else if name == "Option.unwrap_or" { Some("A") }
    else if name == "Option.map"     { Some("Option<B>") }
    else if name == "Option.and_then"{ Some("Option<B>") }
    else { Some("Result<A,F>") }
}
```

注: `builtin_ret_ty` の末尾に `is_map_fn(name)` / `is_result_fn(name)` のブランチを追加する。

### C-3: `vm.rs` — stdlib_fav_runner へのディスパッチ追加

**変更ファイル**: `fav/src/vm.rs`

```rust
// 既存の intersperse/capitalize/indent ディスパッチの後に追加
"List.flat_map" | "List.zip_with" | "List.take_while" | "List.drop_while" |
"List.unique"   | "List.count"    | "List.sum"         |
"List.min"      | "List.max"      | "List.chunk"       | "List.group_by" => {
    call_list_stdlib(name, args)
}

"String.pad_left"   | "String.pad_right" | "String.truncate" |
"String.repeat"     | "String.trim_start"| "String.trim_end" | "String.replace" => {
    call_string_stdlib(name, args)
}

"Map.merge_with" | "Map.filter" | "Map.map_values" |
"Map.from_list"  | "Map.to_list" => {
    call_map_stdlib(name, args)
}

"Result.map_err"   | "Result.and_then" | "Result.all" |
"Option.map"       | "Option.and_then" | "Option.unwrap_or" |
"Option.is_some"   | "Option.is_none" => {
    call_result_stdlib(name, args)
}
```

新関数 `call_map_stdlib` / `call_result_stdlib` を `stdlib_fav_runner.rs` に追加（`call_list_stdlib` と同パターン）。

---

## Phase D: E0012 — 非ジェネリック関数引数数チェック

**変更ファイル**: `fav/self/checker.fav`

### D-1: `fn_to_scheme_str` の出力形式変更

現在: `env_insert(env, name, ret_ty_str)` → `"ReturnType"`
変更後: `env_insert(env, name, ArgCount:ReturnType)` — e.g. `"1:String"`

```favnir
// 非ジェネリック fn の env 登録形式
fn make_non_generic_scheme(param_count: Int, ret_ty: String) -> String {
    String.concat(Int.to_string(param_count), String.concat(":", ret_ty))
}
```

`collect_fn_schemes` 内で `IFn(fd)` が非ジェネリック（`fd.type_params` が空）の場合:

```favnir
// Before:
env_insert(env, fd.name, ret_str)

// After:
bind param_count <- List.length(fd.params)
bind scheme      <- make_non_generic_scheme(param_count, ret_str)
env_insert(env, fd.name, scheme)
```

### D-2: `is_non_generic_scheme` 判定ヘルパー追加

```favnir
// "数字:" で始まるかどうかで非ジェネリックスキームを判定
fn is_non_generic_scheme(s: String) -> Bool {
    if String.length(s) < 2 { false }
    else {
        bind first <- String.take(s, 1)
        first == "0" || first == "1" || first == "2" || first == "3" ||
        first == "4" || first == "5" || first == "6" || first == "7" ||
        first == "8" || first == "9"
    }
}
```

### D-3: `check_fn_call_arity` 追加

```favnir
fn check_fn_call_arity(scheme: String, fname: String, n_actual: Int) -> Result<String, String> {
    bind colon_idx <- String.index_of(scheme, ":")
    bind expected_str <- String.take(scheme, colon_idx)
    bind ret_ty       <- String.drop(scheme, colon_idx + 1)
    bind n_expected   <- Int.parse(expected_str)
    if n_expected == n_actual { Result.ok(ret_ty) }
    else {
        Result.err(fmt_err("E0012",
            String.concat(fname,
            String.concat(" expects ",
            String.concat(Int.to_string(n_expected),
            String.concat(" argument(s), got ",
            Int.to_string(n_actual)))))))
    }
}
```

### D-4: `infer_call_user` に E0012 チェックを追加

```favnir
// is_fn_scheme_str の前に is_non_generic_scheme チェックを挿入
Some(ty) => {
    if is_non_generic_scheme(ty) {
        bind n_actual <- List.length(infer_arg_tys_raw(args, env))
        check_fn_call_arity(ty, fname, n_actual)
    } else if is_fn_scheme_str(ty) {
        // 既存のジェネリックパス（E0008）
        bind arg_tys <- infer_arg_tys(args, env)
        ...
    } else {
        Result.ok(inf_result_of(ty, state))
    }
}
```

注: 非ジェネリックスキームの戻り型は `check_fn_call_arity` が `ret_ty` 文字列を返す。
`inf_result_of` でラップして `InfResult` にする。

---

## Phase E: マルチパラメータクロージャ self-hosted 対応

**変更ファイル**:
- `fav/self/parser.fav`
- `fav/self/checker.fav`
- `fav/self/compiler.fav`
- `fav/src/middle/ast_lower_checker.rs`

### E-1: `parser.fav` — `ELambda` 型変更

```favnir
// Before:
| ELambda(String, Expr)

// After:
| ELambda(List<String>, Expr)
```

`parse_lambda` 関数を更新:

```favnir
// Before: 単引数
fn parse_lambda(tokens: List<Token>, pos: Int) -> ParseResult<Expr> {
    // |x| body → ELambda("x", body)
}

// After: 複数引数（カンマ区切り）
fn parse_lambda_params(tokens: List<Token>, pos: Int, acc: List<String>) -> ParseResult<List<String>> {
    // | または , を消費しながら Ident を収集
}
fn parse_lambda(tokens: List<Token>, pos: Int) -> ParseResult<Expr> {
    // |x, y, z| body → ELambda(["x", "y", "z"], body)
    bind params_result <- parse_lambda_params(tokens, pos, List.empty())
    bind body_result   <- parse_expr(tokens, params_result.pos)
    ParseResult.ok(ELambda(params_result.value, body_result.value), body_result.pos)
}
```

既存の `ELambda` パターンマッチをすべて `ELambda(params, body)` に更新。

### E-2: `checker.fav` — `ELambda` チェック更新

```favnir
// Before:
ELambda(param, body) =>
    bind param_ty <- fresh_var(state)
    bind body_env <- env_insert(env, param, param_ty)
    infer_hm(body, body_env, ...)

// After:
ELambda(params, body) =>
    bind param_ty <- fresh_var(state)
    // 最初のパラメータのみ環境に追加（カリー化前提）
    // 複数パラメータは compiler.fav でカリー化展開されるため、
    // チェック時は List.first(params) を使う
    match List.first(params) {
        None => Result.err("E: lambda with no params")
        Some(p) =>
            bind body_env <- env_insert(env, p, param_ty)
            infer_hm(body, body_env, ...)
    }
```

注: チェッカーは引数型を完全に追跡するより、まず parser の変更を通すことを優先。
型推論の精度は v9.x で改善。

### E-3: `compiler.fav` — カリー化展開

```favnir
// Before:
ELambda(param, body) =>
    compile_lambda(param, body, env)

// After:
ELambda(params, body) =>
    desugar_multi_lambda(params, body, env)

// カリー化脱糖: |x, y| body → |x| |y| body
fn desugar_multi_lambda(params: List<String>, body: Expr, env: CompileEnv) -> CompileResult {
    match List.length(params) {
        0 => compile_error("lambda with no params")
        1 =>
            bind p <- List.first(params)
            compile_lambda(p.value, body, env)
        _ =>
            bind p    <- List.first(params)
            bind rest <- List.drop(params, 1)
            bind inner <- desugar_multi_lambda(rest, body, env)
            compile_lambda(p.value, inner, env)
    }
}
```

### E-4: `ast_lower_checker.rs` — 複数引数対応

```rust
// Before（単引数のみ）:
Expr::Lambda(param, body) => {
    let lowered_body = lower_expr(body, ctx);
    VMValue::Struct("ELambda", vec![
        VMValue::Str(param.clone()),
        lowered_body,
    ])
}

// After（複数引数対応）:
Expr::Lambda(params, body) => {
    let lowered_body = lower_expr(body, ctx);
    let param_list = VMValue::List(
        params.iter().map(|p| VMValue::Str(p.clone())).collect()
    );
    VMValue::Struct("ELambda", vec![
        param_list,
        lowered_body,
    ])
}
```

注: Rust の `Expr::Lambda` が `Vec<String>` を持つかどうかを確認すること。
`fav/src/ast.rs` の `Lambda` 定義を先に確認し、必要なら `(String, Box<Expr>)` → `(Vec<String>, Box<Expr>)` に変更。

---

## Phase F: テスト追加

**変更ファイル**: `fav/src/driver.rs`（既存テストモジュールに追記）

### F-1: `rvm_version_constant` テスト

```rust
#[test]
fn rvm_version_constant() {
    assert_eq!(fav::vm::VM_VERSION, "1.0.0");
}
```

### F-2: `stdlib_v91_list_tests` — List 新関数

```rust
#[test]
fn stdlib_sum() {
    let src = "public fn main() -> Int { List.sum([1, 2, 3, 4, 5]) }";
    assert_run_output(src, "15");
}

#[test]
fn stdlib_zip_with() {
    let src = r#"
public fn main() -> List<Int> {
    List.zip_with(|x| |y| x + y, [1,2,3], [4,5,6])
}
"#;
    assert_run_output(src, "[5, 7, 9]");
}

#[test]
fn stdlib_take_while() {
    let src = "public fn main() -> List<Int> { List.take_while(|x| x < 3, [1,2,3,4,5]) }";
    assert_run_output(src, "[1, 2]");
}

#[test]
fn stdlib_unique() {
    let src = "public fn main() -> List<Int> { List.unique([1,2,1,3,2]) }";
    assert_run_output(src, "[1, 2, 3]");
}

#[test]
fn stdlib_min_max() {
    let src = r#"
public fn main() -> String {
    bind mn <- List.min([3,1,4,1,5])
    bind mx <- List.max([3,1,4,1,5])
    match mn {
        None => "none"
        Some(v) => match mx {
            None => "none"
            Some(w) => String.concat(Int.to_string(v), String.concat(",", Int.to_string(w)))
        }
    }
}
"#;
    assert_run_output(src, "1,5");
}
```

### F-3: `stdlib_v91_string_tests`

```rust
#[test]
fn stdlib_string_repeat() {
    let src = r#"public fn main() -> String { String.repeat("ab", 3) }"#;
    assert_run_output(src, "ababab");
}

#[test]
fn stdlib_string_replace() {
    let src = r#"public fn main() -> String { String.replace("hello world", "world", "Favnir") }"#;
    assert_run_output(src, "hello Favnir");
}

#[test]
fn stdlib_string_trim() {
    let src = r#"public fn main() -> String { String.trim_start("  hello") }"#;
    assert_run_output(src, "hello");
}
```

### F-4: `e0012_arity_mismatch` テスト

```rust
#[test]
fn e0012_arity_mismatch() {
    let src = r#"
fn greet(name: String) -> String { String.concat("Hello, ", name) }
public fn main() -> String { greet("Alice", "Bob") }
"#;
    let errors = run_check(src);
    assert!(
        errors.iter().any(|e| e.contains("E0012")),
        "expected E0012 for arity mismatch, got: {:?}", errors
    );
}

#[test]
fn e0012_correct_arity_ok() {
    let src = r#"
fn greet(name: String) -> String { String.concat("Hello, ", name) }
public fn main() -> String { greet("Alice") }
"#;
    let errors = run_check(src);
    assert!(errors.is_empty(), "correct arity should not error: {:?}", errors);
}
```

### F-5: `multi_param_closure_tests`

```rust
#[test]
fn multi_param_closure_zip_with() {
    // self-hosted pipeline で |x, y| が動作すること
    let src = r#"
public fn main() -> List<Int> {
    List.zip_with(|x, y| x + y, [1,2,3], [4,5,6])
}
"#;
    assert_run_output_favnir_pipeline(src, "[5, 7, 9]");
}
```

---

## Phase G: 最終確認

```
cargo test rvm_version                  # VM_VERSION 定数
cargo test stdlib_v91                   # stdlib 新関数
cargo test e0012                        # E0012 アリティ
cargo test multi_param_closure          # マルチパラメータクロージャ
cargo test                              # 全件（目標: 1160 件以上）
```

---

## 実装上の注意

### Phase E の優先順位

`ELambda` の変更は parser → checker → compiler → ast_lower の順で実施。
各ステップで `cargo build` が通ることを確認してから次に進む。

### `ast.rs` の `Lambda` 確認

`fav/src/ast.rs` の `Expr::Lambda` が `(String, Box<Expr>)` の場合、
E-4 の前に `(Vec<String>, Box<Expr>)` へ変更し、Rust パーサーも合わせて更新する。
ただし `|x| body`（単引数）は `vec!["x"]` として表現されるため後方互換を維持できる。

### Phase D の `Int.parse` 依存

`check_fn_call_arity` 内で `Int.parse` を使う。
`Int.parse` が checker.fav の `builtin_ret_ty` に登録されているか確認すること。
未登録なら `String.split(scheme, ":")` + `List.first` で文字列比較でも代替可能。

### `Map.entries` primitive の確認

`map_stdlib.fav` の `Map.to_list` は `Map.entries(m)` を呼ぶ。
`vm.rs` に `Map.entries` が Rust primitive として実装されているか確認し、
なければ追加する。

### stdlib のテスト目標

v9.1.0 完了条件のテスト件数 1160 件以上。
内訳: 1136（v9.0.0 ベース）+ 24（新規テスト想定）= 1160。
