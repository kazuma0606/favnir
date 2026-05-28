# Favnir v8.0.0 実装計画

---

## 変更ファイル一覧

| ファイル | 変更内容 |
|----------|----------|
| `fav/self/checker.fav` | Phase A〜D 全変更（~220 行追加） |
| `fav/src/driver.rs` | `checker_v80_tests` モジュール追加（3 テスト） |
| `site/content/docs/language/self-host-checker.mdx` | v8.0.0 セクション追記 |
| `versions/v8.0.0/tasks.md` | 完了状態に更新 |

---

## Phase A: 型変数拡張とリストユーティリティ

### A-1: `is_type_var_extended`

`is_type_var`（1 文字大文字のみ）を拡張して 2 文字型変数（`T0`, `T1` 等）も受け入れる。

```favnir
fn is_type_var_extended(s: String) -> Bool {
    if is_type_var(s) { true }
    else {
    if String.length(s) == 2 {
        bind first  <- String.slice(s, 0, 1)
        bind second <- String.slice(s, 1, 2)
        is_type_var(first) &&
        (second == "0" || second == "1" || second == "2" || second == "3" || second == "4" ||
         second == "5" || second == "6" || second == "7" || second == "8" || second == "9")
    } else { false }
    }
}
```

閉じ括弧カウント: 2 if-branch → 末尾 `}` 1 個（`else { false }` + outer `}`）

### A-2: `list_dedup`

```favnir
fn list_dedup_inner(lst: List<String>, seen: List<String>) -> List<String> {
    match List.first(lst) {
        None => List.empty()
        Some(x) => {
            if list_contains(seen, x) {
                list_dedup_inner(List.drop(lst, 1), seen)
            } else {
                List.push(
                    list_dedup_inner(List.drop(lst, 1), List.push(seen, x)),
                    x)
            }
        }
    }
}

fn list_dedup(lst: List<String>) -> List<String> {
    list_dedup_inner(lst, List.empty())
}
```

注意: `List.push` は先頭に追加（prepend）なので最後に `List.push(..., x)` で先頭に付ける。
`list_dedup_inner` が返すリストは逆順になるため、後続処理での利用に注意すること。

実際には vars の順序が問題になる場合は `list_dedup_inner` を末尾結合型に書き直す。
v8.0.0 では型変数の順序は `String.split(vars_csv, ",")` で復元するため、
dedup の返す順序が正しければ十分（`first_char` 順になっていれば OK）。

### A-3: `collect_type_vars_from_te`

TypeExpr を再帰的に走査して型変数名を収集する（重複あり、後で dedup する）。

```favnir
fn collect_type_vars_from_te(te: TypeExpr) -> List<String> {
    match te {
        TeSimple(name) => {
            if is_type_var_extended(name) { List.singleton(name) }
            else { List.empty() }
        }
        TeList(inner)    => collect_type_vars_from_te(inner)
        TeOption(inner)  => collect_type_vars_from_te(inner)
        TeResult(a, b)   => List.concat(collect_type_vars_from_te(a), collect_type_vars_from_te(b))
        TeMap(k, v)      => List.concat(collect_type_vars_from_te(k), collect_type_vars_from_te(v))
        TeFn(a, b)       => List.concat(collect_type_vars_from_te(a), collect_type_vars_from_te(b))
    }
}
```

### A-4: `collect_type_vars_from_params`

```favnir
fn collect_type_vars_from_params_inner(params: List<Param>, acc: List<String>) -> List<String> {
    match List.first(params) {
        None => acc
        Some(p) => collect_type_vars_from_params_inner(
            List.drop(params, 1),
            List.concat(acc, collect_type_vars_from_te(p.ty)))
    }
}

fn collect_type_vars_from_params(params: List<Param>) -> List<String> {
    collect_type_vars_from_params_inner(params, List.empty())
}
```

---

## Phase B: スキーム文字列ヘルパー

スキーム形式: `"forall|A,B|List<A>;Fn|List<B>"`
- `|` でセクション分割（Favnir 型文字列には `|` が出現しないため安全）
- `,` で型変数名を結合
- `;` でパラメータ型を結合（`Map<K,V>` の `,` と混在しないよう）

### B-1: 検出・抽出ヘルパー

```favnir
fn is_fn_scheme_str(s: String) -> Bool {
    String.starts_with(s, "forall|")
}

fn fn_scheme_section(s: String, idx: Int) -> String {
    match List.first(List.drop(String.split(s, "|"), idx)) {
        None     => ""
        Some(v)  => v
    }
}

fn fn_scheme_vars_str(s: String) -> String   { fn_scheme_section(s, 1) }
fn fn_scheme_params_str(s: String) -> String { fn_scheme_section(s, 2) }
fn fn_scheme_ret(s: String) -> String        { fn_scheme_section(s, 3) }
```

### B-2: 構築ヘルパー

```favnir
fn join_strings_inner(lst: List<String>, sep: String, acc: String, first: Bool) -> String {
    match List.first(lst) {
        None => acc
        Some(s) => {
            if first { join_strings_inner(List.drop(lst, 1), sep, s, false) }
            else { join_strings_inner(List.drop(lst, 1), sep,
                       String.concat(acc, String.concat(sep, s)), false) }
        }
    }
}

fn join_strings(lst: List<String>, sep: String) -> String {
    join_strings_inner(lst, sep, "", true)
}

fn make_fn_scheme_str(vars_csv: String, params_semi: String, ret: String) -> String {
    if vars_csv == "" { ret }
    else {
        String.concat("forall|", String.concat(vars_csv,
        String.concat("|", String.concat(params_semi, String.concat("|", ret)))))
    }
}
```

---

## Phase C: スキーム構築と pre-pass

### C-1: `params_to_type_str_list`

```favnir
fn params_to_type_str_list_inner(params: List<Param>, acc: List<String>) -> List<String> {
    match List.first(params) {
        None => acc
        Some(p) => params_to_type_str_list_inner(
            List.drop(params, 1),
            List.concat(acc, List.singleton(type_expr_to_str(p.ty))))
    }
}

fn params_to_type_str_list(params: List<Param>) -> List<String> {
    params_to_type_str_list_inner(params, List.empty())
}
```

### C-2: `fn_to_scheme_str`

```favnir
fn fn_to_scheme_str(fd: FnDef) -> String {
    bind all_vars <- list_dedup(List.concat(
        collect_type_vars_from_params(fd.params),
        collect_type_vars_from_te(fd.ret)))
    if List.length(all_vars) == 0 {
        type_expr_to_str(fd.ret)
    } else {
        bind vars_csv    <- join_strings(all_vars, ",")
        bind param_tys   <- params_to_type_str_list(fd.params)
        bind params_semi <- join_strings(param_tys, ";")
        bind ret_str     <- type_expr_to_str(fd.ret)
        make_fn_scheme_str(vars_csv, params_semi, ret_str)
    }
}
```

### C-3: `collect_fn_schemes`

```favnir
fn collect_fn_schemes(items: List<Item>, env: List<KVPair>) -> List<KVPair> {
    match List.first(items) {
        None => env
        Some(item) => {
            match item {
                IFn(fd) => collect_fn_schemes(
                    List.drop(items, 1),
                    env_insert(env, fd.name, fn_to_scheme_str(fd)))
                _ => collect_fn_schemes(List.drop(items, 1), env)
            }
        }
    }
}
```

### C-4: `check` 関数の更新

```favnir
public fn check(prog: Program) -> Result<String, String> {
    bind init_env   <- env_empty()
    bind scheme_env <- collect_fn_schemes(prog.items, init_env)
    check_items(prog.items, scheme_env)
}
```

---

## Phase D: スキームの具体化と infer_call_hm

### D-1: `build_scheme_subst_inner`

```favnir
fn build_scheme_subst_inner(param_tys: List<String>, arg_tys: List<String>,
                             subst: List<KVPair>) -> Result<List<KVPair>, String> {
    match List.first(param_tys) {
        None => Result.ok(subst)
        Some(pt) => {
            match List.first(arg_tys) {
                None => Result.ok(subst)
                Some(at) => Result.and_then(unify_deep(pt, at, subst), |s2|
                    build_scheme_subst_inner(
                        List.drop(param_tys, 1),
                        List.drop(arg_tys, 1),
                        s2))
            }
        }
    }
}
```

### D-2: `apply_scheme_subst`

```favnir
fn apply_scheme_subst(ret_ty: String, vars: List<String>, subst: List<KVPair>) -> String {
    match List.first(vars) {
        None => ret_ty
        Some(v) => {
            match subst_lookup(subst, v) {
                None      => apply_scheme_subst(ret_ty, List.drop(vars, 1), subst)
                Some(val) => apply_scheme_subst(
                    String.replace(ret_ty, v, val),
                    List.drop(vars, 1),
                    subst)
            }
        }
    }
}
```

### D-3: `instantiate_fn_scheme`

```favnir
fn instantiate_fn_scheme(scheme: String, arg_tys: List<String>, state: InfState)
    -> Result<InfResult, String>
{
    bind vars_str   <- fn_scheme_vars_str(scheme)
    bind params_str <- fn_scheme_params_str(scheme)
    bind ret_ty     <- fn_scheme_ret(scheme)
    bind vars       <- String.split(vars_str, ",")
    bind param_tys  <- String.split(params_str, ";")
    Result.and_then(build_scheme_subst_inner(param_tys, arg_tys, state.subst), |subst2|
        Result.ok(inf_result_of(
            apply_scheme_subst(ret_ty, vars, subst2),
            inf_state_new(subst2, state.counter))))
}
```

### D-4: `infer_call_user`

```favnir
fn infer_call_user(fname: String, args: Expr, env: List<KVPair>, state: InfState)
    -> Result<InfResult, String>
{
    match env_lookup(env, fname) {
        None => Result.ok(inf_result_of("Unknown", state))
        Some(ty) => {
            if is_fn_scheme_str(ty) {
                bind arg_tys <- infer_arg_tys(args, env)
                instantiate_fn_scheme(ty, arg_tys, state)
            } else {
                Result.ok(inf_result_of(ty, state))
            }
        }
    }
}
```

### D-5: `infer_call_hm`

```favnir
fn infer_call_hm(ns: String, fname: String, args: Expr, env: List<KVPair>, state: InfState)
    -> Result<InfResult, String>
{
    if ns == "" {
        infer_call_user(fname, args, env, state)
    } else {
        Result.and_then(infer_call(ns, fname, args, env), |ty|
            Result.ok(inf_result_of(ty, state)))
    }
}
```

### D-6: `infer_hm` の `ECall` ケース追加

```favnir
fn infer_hm(expr: Expr, env: List<KVPair>, state: InfState) -> Result<InfResult, String> {
    match expr {
        ELit(lit) => ...
        EVar(name) => ...
        EBind(vname, val_e, cont_e) => ...
        EIf(cond, then_e, else_e) => ...
        ECall(ns, fname, args) => infer_call_hm(ns, fname, args, env, state)  // ← 追加
        _ => Result.and_then(infer_expr(expr, env), |ty|
            Result.ok(inf_result_of(ty, state)))
    }
}
```

---

## Phase E: テスト（checker.fav 末尾に 10 件追加）

```favnir
test "is_type_var_ext single" {
    is_type_var_extended("A") && is_type_var_extended("Z")
}

test "is_type_var_ext two-char" {
    is_type_var_extended("T0") && is_type_var_extended("A9")
}

test "is_type_var_ext not" {
    is_type_var_extended("Int") == false && is_type_var_extended("AB") == false
}

test "collect_te vars simple" {
    match List.first(collect_type_vars_from_te(TeSimple("A"))) {
        None => false
        Some(v) => v == "A"
    }
}

test "collect_te vars none" {
    List.length(collect_type_vars_from_te(TeSimple("Int"))) == 0
}

test "is_fn_scheme_str true" {
    is_fn_scheme_str("forall|A|List<A>|Option<A>")
}

test "is_fn_scheme_str false" {
    is_fn_scheme_str("Option<Int>") == false
}

test "fn_scheme_ret" {
    fn_scheme_ret("forall|A|List<A>|Option<A>") == "Option<A>"
}

test "instantiate simple" {
    bind s0 <- inf_state_new(subst_empty(), 0)
    match instantiate_fn_scheme("forall|A|A|A", List.push(List.empty(), "Int"), s0) {
        Err(_) => false
        Ok(r)  => r.ty == "Int"
    }
}

test "instantiate nested" {
    bind s0 <- inf_state_new(subst_empty(), 0)
    match instantiate_fn_scheme(
        "forall|A|List<A>|Option<A>",
        List.push(List.empty(), "List<String>"),
        s0) {
        Err(_) => false
        Ok(r)  => r.ty == "Option<String>"
    }
}
```

---

## Phase F: driver.rs 統合テスト

```rust
mod checker_v80_tests {
    // checker_src() / run_checker_inline() は checker_v79_tests と同様

    #[test]
    fn checker_fav_scheme_str() {
        // is_fn_scheme_str("forall|A|List<A>|Option<A>") → true
        let result = run_checker_inline(r#"
public fn main() -> Bool {
    is_fn_scheme_str("forall|A|List<A>|Option<A>")
}
"#);
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn checker_fav_instantiate_scheme() {
        // instantiate_fn_scheme("forall|A|List<A>|Option<A>", ["List<Int>"], s0) → "Option<Int>"
        let result = run_checker_inline(r#"
public fn main() -> String {
    bind s0 <- inf_state_new(subst_empty(), 0)
    match instantiate_fn_scheme(
        "forall|A|List<A>|Option<A>",
        List.push(List.empty(), "List<Int>"),
        s0) {
        Err(_) => "error"
        Ok(r)  => r.ty
    }
}
"#);
        assert_eq!(result, Value::Str("Option<Int>".to_string()));
    }

    #[test]
    fn checker_fav_infer_hm_generic_call() {
        // fn first_elem(xs: List<A>) -> Option<A>; infer_hm(ECall("", "first_elem", EArgList(EVar("xs"), EArgNil)))
        // with env { xs: List<Int>, first_elem: "forall|A|List<A>|Option<A>" }
        // → "Option<Int>"
        let result = run_checker_inline(r#"
public fn main() -> String {
    bind env <- env_insert(
        env_insert(env_empty(), "xs", "List<Int>"),
        "first_elem", "forall|A|List<A>|Option<A>")
    bind s0 <- inf_state_new(subst_empty(), 0)
    match infer_hm(
        ECall("", "first_elem", EArgList(EVar("xs"), EArgNil)),
        env, s0) {
        Err(_) => "error"
        Ok(r)  => r.ty
    }
}
"#);
        assert_eq!(result, Value::Str("Option<Int>".to_string()));
    }
}
```

---

## 実装上の注意点

1. **`list_dedup_inner` の順序**:
   `List.push(list_dedup_inner(...), x)` は先頭に x を付ける。
   再帰呼び出しが「残りのリスト」を処理するため、最終的な順序は元のリストの順序を維持する。
   ただし push=prepend なので `list_dedup` が返すリストの先頭 = 元リストの最後の新要素。
   実際の vars 順序に依存するなら、`List.concat(acc, List.singleton(x))` パターンで
   アキュムレータ型に書き換えること。

2. **`join_strings` の first フラグ**:
   Favnir では `Bool` を関数引数に渡せる。`join_strings_inner` の `first: Bool` で初回は
   sep を付けない。`if first { ... } else { ... }` 構造 → 1 if-branch → 閉じ括弧なし
   （`else { ... }` が self-closing）。

3. **`fn_scheme_section` の `List.drop` + `List.first`**:
   `String.split("forall|A|List<A>|Option<A>", "|")` → `["forall", "A", "List<A>", "Option<A>"]`
   `List.drop(list, 1)` → `["A", "List<A>", "Option<A>"]`
   `List.first(...)` → `Some("A")`
   これは正しい。ただし `String.split` が返すリストの順序に注意（push=prepend だと逆順）。
   実際の実装では `String.split` の返す順序を確認すること。

4. **`String.replace` による型変数置換**:
   `String.replace("Option<A>", "A", "Int")` → `"Option<Int>"` ✓
   `String.replace("Map<A, A>", "A", "String")` → `"Map<String, String>"` ✓
   ただし `String.replace("Aa", "A", "Int")` → `"Inta"` という問題がある。
   v8.0.0 では型変数が単文字 A-Z または 2 文字 T0-Z9 に限定されているため、
   実際の型名（Int, Bool, String 等）との衝突は起きない。

5. **`infer_hm` の `ECall` ケース追加**:
   `ECall` は現在 `_` フォールバックで `infer_expr` に委譲されていた。
   `ECall` を明示的なケースに昇格させることで、スキームに基づく推論が有効になる。
   既存の fallback は変わらず残す（EMatch 等）。

6. **`check` 関数の `collect_fn_schemes` pre-pass**:
   pre-pass で全関数のスキームを env に登録してから `check_items` を呼ぶ。
   これにより前方参照する関数呼び出しでも正しくスキームを参照できる。
