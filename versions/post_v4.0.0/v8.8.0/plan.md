# Favnir v8.8.0 実装計画

Date: 2026-05-30

---

## Phase A: `make_fn_scheme_str` の変更

**変更ファイル**: `fav/self/checker.fav`

`if String.length(vars_csv) == 0 { ret }` の早期リターンを削除し、
常にスキーム文字列を生成する:

```fav
// Before:
fn make_fn_scheme_str(vars_csv: String, params_semi: String, ret: String) -> String {
    if String.length(vars_csv) == 0 { ret }
    else {
        String.concat("forall|",
        String.concat(vars_csv, ...))
    }
}

// After:
fn make_fn_scheme_str(vars_csv: String, params_semi: String, ret: String) -> String {
    String.concat("forall|",
    String.concat(vars_csv,
    String.concat("|",
    String.concat(params_semi,
    String.concat("|", ret)))))
}
```

---

## Phase B: `fn_to_scheme_str` の変更

**変更ファイル**: `fav/self/checker.fav`

`if List.length(all_vars) == 0` の分岐を削除し、常に full path を通す:

```fav
// Before:
fn fn_to_scheme_str(fd: FnDef) -> String {
    bind all_vars <- ...
    if List.length(all_vars) == 0 {
        type_expr_to_str(fd.ret)
    } else {
        bind vars_csv    <- String.join(all_vars, ",")
        bind param_types <- params_to_type_str_list(fd.params)
        bind params_semi <- String.join(param_types, ";")
        bind ret_str     <- type_expr_to_str(fd.ret)
        make_fn_scheme_str(vars_csv, params_semi, ret_str)
    }
}

// After:
fn fn_to_scheme_str(fd: FnDef) -> String {
    bind all_vars    <- list_dedup(List.concat(
                           collect_type_vars_from_params(fd.params),
                           collect_type_vars_from_te(fd.ret)))
    bind vars_csv    <- String.join(all_vars, ",")
    bind param_types <- params_to_type_str_list(fd.params)
    bind params_semi <- String.join(param_types, ";")
    bind ret_str     <- type_expr_to_str(fd.ret)
    make_fn_scheme_str(vars_csv, params_semi, ret_str)
}
```

---

## Phase C: `register_variant` の簡略化

**変更ファイル**: `fav/self/checker.fav`

v8.7.0 で直接文字列連結していた部分を `make_fn_scheme_str` を使うよう変更:

```fav
// Before (v8.7.0):
Some(te) => {
    bind param_str <- type_expr_to_str(te)
    bind scheme    <- String.concat("forall||",
                      String.concat(param_str,
                      String.concat("|", type_name)))
    env_insert(env, v.name, scheme)
}

// After (v8.8.0):
Some(te) => {
    bind param_str <- type_expr_to_str(te)
    env_insert(env, v.name, make_fn_scheme_str("", param_str, type_name))
}
```

---

## Phase D: `infer_call` の戻り型修正

**変更ファイル**: `fav/self/checker.fav`

`ns == ""` のとき、`env_lookup` が返す ty がスキーム文字列になるため、
戻り型を正しく抽出するよう変更:

```fav
fn infer_call(ns: String, fname: String, args: Expr, env: List<KVPair>) -> Result<String, String> {
    if ns == "" {
        match env_lookup(env, fname) {
            Some(ty) =>
                if is_fn_scheme_str(ty) { Result.ok(fn_scheme_ret(ty)) }
                else { Result.ok(ty) }
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

## Phase E: 統合テスト

**変更ファイル**: `fav/src/driver.rs`

`checker_v87_tests` モジュールに追加テストを追記（または新モジュール `checker_v88_tests`）:

### E-1: `nongeneric_wrong_arity_e0008`

```rust
#[test]
fn nongeneric_wrong_arity_e0008() {
    // 非ジェネリック関数の引数数不一致 → E0008
    let errors = check_errors(r#"
fn add(a: Int, b: Int) -> Int { a + b }
public fn main() -> Int { add(1) }
"#);
    assert!(
        errors.iter().any(|e| e.contains("E0008")),
        "expected E0008 for non-generic fn arity mismatch, got: {:?}", errors
    );
}
```

### E-2: `zero_param_fn_correct_call`

```rust
#[test]
fn zero_param_fn_correct_call() {
    // 0 引数関数の正しい呼び出し → エラーなし
    let errors = check_errors(r#"
fn get_val() -> Int { 42 }
public fn main() -> Int { get_val() }
"#);
    assert!(errors.is_empty(), "0-param correct call should pass: {:?}", errors);
}
```

### E-3: `zero_param_fn_wrong_arity_e0008`

```rust
#[test]
fn zero_param_fn_wrong_arity_e0008() {
    // 0 引数関数に引数を渡す → E0008
    let errors = check_errors(r#"
fn get_val() -> Int { 42 }
public fn main() -> Int { get_val(1) }
"#);
    assert!(
        errors.iter().any(|e| e.contains("E0008")),
        "expected E0008 for zero-param fn with args, got: {:?}", errors
    );
}
```

### E-4: 既存テスト全件通過確認

```
cargo test checker_fav
cargo test
```

---

## Phase F: 最終確認

- `cargo test` — 1128 tests passing（+3 新規）
- `checker_fav_wire_self_check` が大スタックで通ること
- tasks.md 完了・commit

---

## 実装ノート

### Phase A → B → C → D → E の順序

A（`make_fn_scheme_str`）変更後に B（`fn_to_scheme_str`）を変更すると
非ジェネリック関数がスキーム形式になる。この時点でビルドは通るが
`infer_call` が "forall||Int|Int" をそのまま返すため、一部のテストが失敗する可能性がある。
D（`infer_call` 修正）まで実施してから cargo test を実行すること。

### `checker_fav_wire_self_check` の注意

checker.fav 自身をチェックすると、全ての user-defined fn に対して E0008 チェックが走る。
checker.fav 内の関数呼び出しが全て正しいアリティであれば通るはず。
もし意外な E0007/E0008 が出た場合は `infer_call_user` の `None` ケースに注目する
（lambda param が "Unknown" 型で呼ばれるケースは `is_fn_scheme_str("Unknown") = false` で
スキップされるため問題ない）。

### `String.join([], ",")` の挙動

型変数なし関数で `all_vars = []` のとき `String.join([], ",")` は `""` を返す。
これが `make_fn_scheme_str` の `vars_csv` になる → `"forall||..."` 形式 ✓。

### 既存の `is_fn_scheme_str` テスト

`test "is_fn_scheme_str true"` は `"forall|A|List<A>|Option<A>"` でテストしており、
`make_fn_scheme_str` の変更とは無関係。引き続き通る。
