# Favnir v7.8.0 実装計画

---

## 変更ファイル一覧

| ファイル | 変更内容 |
|----------|----------|
| `fav/self/checker.fav` | Phase A〜D 全変更（~195 行追加） |
| `fav/src/driver.rs` | `checker_v78_tests` モジュール追加（3 テスト） |
| `site/content/docs/language/self-host-checker.mdx` | v7.8.0 セクション追記 |
| `versions/v7.8.0/tasks.md` | 完了状態に更新 |

---

## Phase A: 型変数と置換環境

### A-1: `TyVar` 追加

`Type` sum type に追加：

```favnir
type Type =
  | TyInt
  | TyFloat
  | TyBool
  | TyString
  | TyUnit
  | TyList(Type)
  | TyOption(Type)
  | TyResult(Type, Type)
  | TyMap(Type, Type)
  | TyFn(Type, Type)
  | TyUnknown
  | TyVar(String)   // 追加
```

`type_to_str` に追加：
```favnir
TyVar(name) => name
```

### A-2: Subst ヘルパー（KVPair 流用）

```favnir
fn subst_empty() -> List<KVPair> { env_empty() }

fn subst_insert(subst: List<KVPair>, name: String, ty: String) -> List<KVPair> {
    env_insert(subst, name, ty)
}

fn subst_lookup(subst: List<KVPair>, name: String) -> Option<String> {
    env_lookup(subst, name)
}
```

### A-3: `is_type_var`

```favnir
fn is_type_var(s: String) -> Bool {
    if String.length(s) == 1 {
        bind c <- String.slice(s, 0, 1)
        c == "A" || c == "B" || c == "C" || c == "D" || c == "E" ||
        c == "F" || c == "G" || c == "H" || c == "I" || c == "J" ||
        c == "K" || c == "L" || c == "M" || c == "N" || c == "O" ||
        c == "P" || c == "Q" || c == "R" || c == "S" || c == "T" ||
        c == "U" || c == "V" || c == "W" || c == "X" || c == "Y" || c == "Z"
    } else { false }
}
```

### A-4: `apply_subst`

```favnir
fn apply_subst(subst: List<KVPair>, ty: String) -> String {
    if is_type_var(ty) {
        match subst_lookup(subst, ty) {
            Some(resolved) => resolved
            None => ty
        }
    } else { ty }
}
```

### A-5: `type_str_outer` / `type_str_inner`

```favnir
fn type_str_outer(s: String) -> String {
    if String.contains(s, "<") {
        match List.first(String.split(s, "<")) {
            Some(outer) => outer
            None => s
        }
    } else { s }
}

fn type_str_inner(s: String) -> String {
    if String.contains(s, "<") {
        bind parts <- String.split(s, "<")
        match List.first(List.drop(parts, 1)) {
            None => ""
            Some(rest) => String.slice(rest, 0, String.length(rest) - 1)
        }
    } else { "" }
}
```

---

## Phase B: 基本単一化

```favnir
fn unify(t1: String, t2: String, subst: List<KVPair>) -> Result<List<KVPair>, String> {
    if t1 == "Unknown" || t2 == "Unknown" { Result.ok(subst) }
    else {
    if t1 == t2 { Result.ok(subst) }
    else {
    if is_type_var(t1) {
        match subst_lookup(subst, t1) {
            None    => Result.ok(subst_insert(subst, t1, t2))
            Some(existing) =>
                if existing == t2 { Result.ok(subst) }
                else { Result.err(fmt_err("E0005",
                    String.concat("conflict: ", String.concat(t1,
                    String.concat(" was ", String.concat(existing,
                    String.concat(", got ", t2))))))) }
        }
    } else {
    if is_type_var(t2) {
        match subst_lookup(subst, t2) {
            None    => Result.ok(subst_insert(subst, t2, t1))
            Some(existing) =>
                if existing == t1 { Result.ok(subst) }
                else { Result.err(fmt_err("E0005",
                    String.concat("conflict: ", String.concat(t2,
                    String.concat(" was ", String.concat(existing,
                    String.concat(", got ", t1))))))) }
        }
    } else {
        Result.err(fmt_err("E0005",
            String.concat("cannot unify ", String.concat(t1,
            String.concat(" with ", t2)))))
    }}}}
}
```

エラーコードに `E0005` を追加（型不整合）。

---

## Phase C: ジェネリクス対応 builtin

### C-1: `infer_arg_tys`

```favnir
fn unwrap_ty(r: Result<String, String>) -> String {
    match r {
        Ok(ty) => ty
        Err(_) => "Unknown"
    }
}

fn infer_arg_tys(args: Expr, env: List<KVPair>) -> List<String> {
    match args {
        EArgNil => List.empty()
        EArgList(h, t) => {
            bind hty <- unwrap_ty(infer_expr(h, env))
            List.push(infer_arg_tys(t, env), hty)
        }
        _ => List.empty()
    }
}
```

`List.push(rest, head)` でリストを構築するため、引数順は逆になる。
`infer_generic_*` 関数では `List.first(arg_tys)` が最初の引数を返す。

### C-2: `infer_generic_list`

```favnir
fn wrap_in(outer: String, inner: String) -> String {
    if inner == "" || inner == "Unknown" { outer }
    else { String.concat(outer, String.concat("<", String.concat(inner, ">"))) }
}

fn infer_generic_list(fname: String, arg_tys: List<String>) -> String {
    bind first_arg <- match List.first(arg_tys) { Some(t) => t   None => "" }
    bind inner     <- type_str_inner(first_arg)
    if fname == "first"      { wrap_in("Option", inner) }
    else { if fname == "find"       { wrap_in("Option", inner) }
    else { if fname == "filter"     { wrap_in("List", inner) }
    else { if fname == "push"       { wrap_in("List", inner) }
    else { if fname == "drop"       { wrap_in("List", inner) }
    else { if fname == "take"       { wrap_in("List", inner) }
    else { if fname == "take_while" { wrap_in("List", inner) }
    else { if fname == "drop_while" { wrap_in("List", inner) }
    else { if fname == "concat"     { wrap_in("List", inner) }
    else { if fname == "singleton"  { wrap_in("List", first_arg) }
    else { if fname == "map"        { "List" }
    else { list_fn(fname) }
    }}}}}}}}}}}
}
```

`singleton` は要素型（inner ではなく first_arg）でラップする。
`map` は引数型だけでは戻り型を決定できないので bare `"List"` を返す（HM は v7.9.0）。

### C-3: `infer_generic_opt`

```favnir
fn infer_generic_opt(fname: String, arg_tys: List<String>) -> String {
    bind first_arg <- match List.first(arg_tys) { Some(t) => t   None => "" }
    bind inner     <- type_str_inner(first_arg)
    if fname == "and_then" { "Option" }
    else { opt_fn(fname) }
}
```

### C-4: `infer_generic_res`

```favnir
fn infer_generic_res(fname: String, arg_tys: List<String>) -> String {
    if fname == "and_then" { "Result" }
    else { res_fn(fname) }
}
```

### C-5: `infer_call` 更新

```favnir
fn infer_call(ns: String, fname: String, args: Expr, env: List<KVPair>) -> Result<String, String> {
    if ns == "" {
        match env_lookup(env, fname) {
            Some(ty) => Result.ok(ty)
            None => Result.ok("Unknown")
        }
    } else {
        bind arg_tys <- infer_arg_tys(args, env)
        if ns == "List"   { Result.ok(infer_generic_list(fname, arg_tys)) }
        else { if ns == "Option" { Result.ok(infer_generic_opt(fname, arg_tys)) }
        else { if ns == "Result" { Result.ok(infer_generic_res(fname, arg_tys)) }
        else { Result.ok(builtin_ret_ty(ns, fname)) }
        }}
    }
}
```

---

## Phase D: ユーザー定義ジェネリクス型（基本）

### D-1: `TypeDef` に `type_params` 追加

```favnir
type TypeDef = {
    name: String
    is_record: Bool
    type_params: List<String>
    variants: List<VariantDef>
    fields: List<Param>
}
```

### D-2: `type_param_env`

```favnir
fn type_param_env_inner(params: List<String>, args: List<String>, acc: List<KVPair>) -> List<KVPair> {
    match List.first(params) {
        None => acc
        Some(p) => {
            match List.first(args) {
                None => acc
                Some(a) => type_param_env_inner(
                    List.drop(params, 1),
                    List.drop(args, 1),
                    subst_insert(acc, p, a))
            }
        }
    }
}

fn type_param_env(params: List<String>, args: List<String>) -> List<KVPair> {
    type_param_env_inner(params, args, subst_empty())
}
```

### D-3: `check_item` のジェネリクス型登録

`IType(td)` で型パラメータ数を `env` に記録する（簡易追跡）。

---

## Phase E: テスト（checker.fav 末尾に追加）

```favnir
test "is_type_var upper" {
    is_type_var("A") && is_type_var("Z")
}

test "is_type_var not" {
    is_type_var("Int") == false && is_type_var("a") == false
}

test "apply_subst resolves" {
    bind subst <- subst_insert(subst_empty(), "A", "Int")
    apply_subst(subst, "A") == "Int"
}

test "apply_subst no match" {
    apply_subst(subst_empty(), "String") == "String"
}

test "unify same" {
    match unify("Int", "Int", subst_empty()) {
        Err(_) => false
        Ok(s)  => List.length(s) == 0
    }
}

test "unify var left" {
    match unify("A", "Bool", subst_empty()) {
        Err(_) => false
        Ok(s)  => match subst_lookup(s, "A") {
            Some(t) => t == "Bool"
            None    => false
        }
    }
}

test "unify conflict" {
    match unify("Int", "String", subst_empty()) {
        Err(e) => String.starts_with(e, "E0005")
        Ok(_)  => false
    }
}

test "type_str_inner list" {
    type_str_inner("List<Int>") == "Int"
}

test "type_str_inner option" {
    type_str_inner("Option<String>") == "String"
}

test "generic list first" {
    infer_generic_list("first", List.push(List.empty(), "List<Int>")) == "Option<Int>"
}
```

---

## Phase F: driver.rs 統合テスト

```rust
mod checker_v78_tests {
    // run_checker_inline は checker_v77_tests と同様のヘルパー

    #[test]
    fn checker_fav_generic_list_first() {
        // infer_generic_list("first", ["List<Int>"]) → "Option<Int>"
        let result = run_checker_inline(r#"
public fn main() -> String {
    infer_generic_list("first", List.push(List.empty(), "List<Int>"))
}
"#);
        assert_eq!(result, Value::Str("Option<Int>".to_string()));
    }

    #[test]
    fn checker_fav_unify_var() {
        // unify("A", "Bool", subst_empty()) → Ok, lookup A → "Bool"
        let result = run_checker_inline(r#"
public fn main() -> String {
    match unify("A", "Bool", subst_empty()) {
        Err(e) => "error"
        Ok(s)  => match subst_lookup(s, "A") {
            None    => "none"
            Some(t) => t
        }
    }
}
"#);
        assert_eq!(result, Value::Str("Bool".to_string()));
    }

    #[test]
    fn checker_fav_type_str_inner() {
        // type_str_inner("Option<String>") → "String"
        let result = run_checker_inline(r#"
public fn main() -> String {
    type_str_inner("Option<String>")
}
"#);
        assert_eq!(result, Value::Str("String".to_string()));
    }
}
```

---

## 実装上の注意点

1. `infer_arg_tys` で `List.push(rest, hty)` を使うと引数が逆順になる。
   `infer_generic_list` / `opt` / `res` では `List.first(arg_tys)` で最初の引数にアクセスできる。

2. `wrap_in(outer, inner)` — inner が空文字 `""` や `"Unknown"` のときは bare 型名（`"Option"`, `"List"`）を返す。
   これにより型情報がないときは v7.7.0 と同じ挙動になる。

3. `type_str_inner("List<Int>")` の実装では `String.split(s, "<")` → `["List", "Int>"]` → 先頭を drop → `"Int>"` を取得 → `String.slice(rest, 0, len - 1)` で `>` を除去。
   ただし `String.split` が区切り文字を含まない場合（`"List"` 等）は `inner = ""` を返す。

4. `TypeDef.type_params` の追加後、既存の `IType(td)` ハンドラは変更なし（フィールドを増やすだけ）。

5. `is_type_var` の判定は 1 文字大文字のみ。複数文字の型変数（`"T1"`, `"Elem"` 等）は v7.9.0 で拡張。

6. `unify` は flat 型（`"Int"`, `"String"`, `"A"`）のみ対応。
   `"List<A>"` の中の `A` を単一化するネスト対応は v7.9.0 へ持ち越し。
