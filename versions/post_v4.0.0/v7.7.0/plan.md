# Favnir v7.7.0 実装計画

---

## 変更ファイル一覧

| ファイル | 変更内容 |
|----------|----------|
| `fav/self/checker.fav` | Phase A〜D 全変更（~250 行追加） |
| `fav/src/driver.rs` | `checker_v77_tests` モジュール追加（3 テスト） |
| `site/content/docs/language/self-host-checker.mdx` | 新規作成 |
| `versions/v7.7.0/tasks.md` | 完了状態に更新 |

---

## Phase A: エフェクト追跡

### A-1: FnDef 型定義の更新

`checker.fav` の `FnDef` に `effects: List<String>` を追加:

```favnir
type FnDef = {
    is_public: Bool
    name: String
    params: List<Param>
    ret: TypeExpr
    effects: List<String>
    body: Expr
}
```

既存のテスト内 FnDef リテラルを全更新（`effects: List.empty()` を追加）。

### A-2: `infer_expr_effects`

```favnir
fn ns_to_effect(ns: String) -> String {
    if ns == "IO"       { "IO" }
    else { if ns == "Compiler" { "IO" }
    else { if ns == "Cache"    { "Cache" }
    else { if ns == "Queue"    { "Queue" }
    else { if ns == "Email"    { "Email" }
    else { "" }
    }}}}
}

fn infer_expr_effects(expr: Expr) -> List<String> {
    match expr {
        ECall(ns, fname, args) => {
            bind eff <- ns_to_effect(ns)
            if eff == "" { List.empty() }
            else { List.singleton(eff) }
        }
        EBind(v, val_e, cont_e) =>
            List.concat(infer_expr_effects(val_e), infer_expr_effects(cont_e))
        EIf(cond, then_e, else_e) =>
            List.concat(infer_expr_effects(cond),
            List.concat(infer_expr_effects(then_e), infer_expr_effects(else_e)))
        EBlock(a, b) =>
            List.concat(infer_expr_effects(a), infer_expr_effects(b))
        EMatch(scrut, arms) =>
            List.concat(infer_expr_effects(scrut), infer_arms_effects(arms))
        EArgList(h, t) =>
            List.concat(infer_expr_effects(h), infer_expr_effects(t))
        _ => List.empty()
    }
}

fn infer_arms_effects(arms: Expr) -> List<String> {
    match arms {
        EArmNil => List.empty()
        EArm(pat, body, rest) =>
            List.concat(infer_expr_effects(body), infer_arms_effects(rest))
        _ => List.empty()
    }
}
```

### A-3〜A-5: エフェクトチェック

```favnir
fn eq_str_eff(a: String, b: String) -> Bool { a == b }

fn has_effect(effects: List<String>, eff: String) -> Bool {
    List.any(effects, |e| eq_str_eff(e, eff))
}

fn check_effects_all(declared: List<String>, inferred: List<String>) -> Option<String> {
    match List.first(inferred) {
        None => Option.none()
        Some(eff) => {
            if has_effect(declared, eff) {
                check_effects_all(declared, List.drop(inferred, 1))
            } else {
                Option.some(fmt_err("E0003",
                    String.concat("undeclared effect !", eff)))
            }
        }
    }
}
```

`check_fn_def` に統合:

```favnir
fn check_fn_def(fd: FnDef, env: List<KVPair>) -> Result<String, String> {
    bind param_env <- build_param_env(fd.params, env)
    bind inferred_effs <- infer_expr_effects(fd.body)
    match check_effects_all(fd.effects, inferred_effs) {
        Some(err) => Result.err(err)
        None =>
            Result.and_then(infer_expr(fd.body, param_env), |body_ty|
            Result.ok(fd.name))
    }
}
```

---

## Phase B: builtin 全登録

### B-1〜B-9: `builtin_ret_ty` 拡張

`io_fn` の分岐を拡張し、`compiler_fn` / `cache_fn` / `queue_fn` / `email_fn` を追加。
`builtin_ret_ty` に `Compiler` / `Cache` / `Queue` / `Email` の分岐を追加。

```favnir
fn io_fn(fname: String) -> String {
    if fname == "argv"              { "List" }
    else { if fname == "read_file_raw"     { "Result" }
    else { if fname == "write_stdout_raw"  { "Unit" }
    else { if fname == "write_stderr_raw"  { "Unit" }
    else { if fname == "exit_raw"          { "Unit" }
    else { if fname == "list_dir_raw"      { "Result" }
    else { if fname == "file_stat_raw"     { "Map" }
    else { if fname == "path_join_raw"     { "String" }
    else { if fname == "home_dir_raw"      { "Option" }
    else { if fname == "cwd_raw"           { "String" }
    else { if fname == "is_dir_raw"        { "Bool" }
    else { if fname == "println"           { "Unit" }
    else { "Unknown" }
    }}}}}}}}}}}}
}

fn compiler_fn(fname: String) -> String {
    if fname == "check_raw"        { "Result" }
    else { if fname == "lineage_text_raw" { "String" }
    else { "Unknown" }
    }
}

fn cache_fn(fname: String) -> String {
    if fname == "get_raw"     { "Option" }
    else { if fname == "set_raw"     { "Unit" }
    else { if fname == "del_raw"     { "Unit" }
    else { if fname == "exists_raw"  { "Bool" }
    else { if fname == "del_prefix_raw" { "Unit" }
    else { "Unknown" }
    }}}}}
}

fn queue_fn(fname: String) -> String {
    if fname == "send_raw"   { "Result" }
    else { if fname == "recv_raw"   { "Result" }
    else { if fname == "ack_raw"    { "Result" }
    else { if fname == "delete_raw" { "Result" }
    else { "Unknown" }
    }}}}
}

fn email_fn(fname: String) -> String {
    if fname == "send_raw" { "Result" }
    else { "Unknown" }
}
```

`builtin_ret_ty` に追加:

```favnir
else { if ns == "Compiler" { compiler_fn(fname) }
else { if ns == "Cache"    { cache_fn(fname) }
else { if ns == "Queue"    { queue_fn(fname) }
else { if ns == "Email"    { email_fn(fname) }
else { if ns == "Float"    { float_fn(fname) }
else { "Unknown" }
}}}}}
```

---

## Phase C: エラーコード

### C-1: `fmt_err` 追加（既存コードの直前に配置）

```favnir
fn fmt_err(code: String, msg: String) -> String {
    String.concat(code, String.concat(": ", msg))
}
```

### C-2: 既存エラー文字列を全置換

```favnir
// Before
Result.err(String.concat("type mismatch in arithmetic: ", ...))

// After
Result.err(fmt_err("E0001", String.concat("arithmetic type mismatch: ", ...)))
```

| 変更箇所 | 旧 | 新コード |
|----------|-----|----------|
| `infer_op` arithmetic | "type mismatch in arithmetic" | E0001 |
| `infer_op` logical | "logical operator requires Bool" | E0002 |
| `check_effects_all` | 新規 | E0003 |
| `check_match_exhaustive` | 新規 | E0004 |

---

## Phase D: match 網羅性チェック

### D-1〜D-2: ctor 収集

```favnir
fn collect_arm_ctors(arms: Expr) -> List<String> {
    match arms {
        EArmNil => List.empty()
        EArm(pat, body, rest) => {
            bind ctor <- pat_ctor_name(pat)
            List.push(collect_arm_ctors(rest), ctor)
        }
        _ => List.empty()
    }
}

fn pat_ctor_name(pat: Pat) -> String {
    match pat {
        PWild      => "_"
        PVar(name) => "_"
        PVariant(name) => name
        PVariantP(name, inner) => name
        _ => ""
    }
}

fn eq_str_ctor(a: String, b: String) -> Bool { a == b }

fn list_contains(lst: List<String>, s: String) -> Bool {
    List.any(lst, |x| eq_str_ctor(x, s))
}

fn has_wildcard_ctor(ctors: List<String>) -> Bool {
    list_contains(ctors, "_")
}
```

### D-3〜D-5: 網羅性チェック関数

```favnir
fn check_option_exhaustive(ctors: List<String>) -> Bool {
    if has_wildcard_ctor(ctors) { true }
    else {
        list_contains(ctors, "None") && list_contains(ctors, "Some")
    }
}

fn check_result_exhaustive(ctors: List<String>) -> Bool {
    if has_wildcard_ctor(ctors) { true }
    else {
        list_contains(ctors, "Ok") && list_contains(ctors, "Err")
    }
}

fn check_match_exhaustive(scrut_ty: String, arms: Expr) -> Option<String> {
    bind ctors <- collect_arm_ctors(arms)
    if scrut_ty == "Option" {
        if check_option_exhaustive(ctors) { Option.none() }
        else { Option.some(fmt_err("E0004", "non-exhaustive match on Option")) }
    } else {
    if scrut_ty == "Result" {
        if check_result_exhaustive(ctors) { Option.none() }
        else { Option.some(fmt_err("E0004", "non-exhaustive match on Result")) }
    } else {
        Option.none()
    }}
}
```

### D-6: `infer_expr` の EMatch ハンドラを更新

```favnir
EMatch(scrut, arms) => Result.and_then(infer_expr(scrut, env), |sty|
    match check_match_exhaustive(sty, arms) {
        Some(err) => Result.err(err)
        None => infer_arms(arms, env)
    })
```

---

## Phase E: テスト

### checker.fav 内（末尾に追加）

```favnir
test "fmt_err format" {
    fmt_err("E0001", "bad") == "E0001: bad"
}

test "effect io detected" {
    bind effs <- infer_expr_effects(ECall("IO", "argv", EArgNil))
    List.length(effs) == 1 && List.first(effs) == Some("IO")
}

test "effect cache detected" {
    bind effs <- infer_expr_effects(ECall("Cache", "get_raw", EArgNil))
    List.length(effs) == 1 && List.first(effs) == Some("Cache")
}

test "effect none for pure" {
    bind effs <- infer_expr_effects(ELit(LInt(1)))
    List.length(effs) == 0
}

test "builtin io list_dir_raw" {
    builtin_ret_ty("IO", "list_dir_raw") == "Result"
}

test "builtin cache get_raw" {
    builtin_ret_ty("Cache", "get_raw") == "Option"
}

test "builtin compiler check_raw" {
    builtin_ret_ty("Compiler", "check_raw") == "Result"
}

test "match option exhaustive ok" {
    match check_match_exhaustive("Option",
        EArm(PVariant("None"), ELit(LBool(false)),
        EArm(PVariantP("Some", PVar("v")), ELit(LBool(true)), EArmNil))) {
        None => true
        Some(_) => false
    }
}

test "match option missing some" {
    match check_match_exhaustive("Option",
        EArm(PVariant("None"), ELit(LBool(false)), EArmNil)) {
        None => false
        Some(e) => String.starts_with(e, "E0004")
    }
}
```

### driver.rs（`checker_v77_tests` モジュール）

```rust
mod checker_v77_tests {
    // 共通: checker.fav をロードして public fn を呼ぶヘルパー
    fn run_checker_test(test_name: &str) -> String { ... }

    #[test]
    fn checker_fav_effect_tracking_test() { ... }

    #[test]
    fn checker_fav_builtin_coverage_test() { ... }

    #[test]
    fn checker_fav_exhaustiveness_test() { ... }
}
```

---

## Phase F: ドキュメント

`site/content/docs/language/self-host-checker.mdx`:

- frontmatter: title / order: 7 / category: "言語仕様"
- セクション: アーキテクチャ / エラーコード一覧 / エフェクト追跡の仕組み / match 網羅性 / ロードマップ

---

## 注意点

1. `bind x <- expr` は `expr` が純粋値でも使える（単なる let バインド）
   - `infer_expr_effects(ECall(...))` が `List<String>` を返すので `bind effs <- ...` は不要
   - 直接 `List.singleton(eff)` を返す

2. `infer_arms` のシグネチャ変更に伴い、`EMatch` ハンドラの呼び出しも変更が必要

3. `check_fn_def` で `infer_expr_effects` を呼ぶ際、ネストした Result を避けるため
   純粋関数として設計（`-> List<String>` を返す）

4. FnDef の `effects` フィールド追加後、既存テスト内の FnDef リテラルに
   `effects: List.empty()` を追加する必要がある
