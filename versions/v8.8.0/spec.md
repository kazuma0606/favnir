# Favnir v8.8.0 Spec

Date: 2026-05-30
Theme: checker.fav — 非ジェネリック関数の引数数チェック（全関数スキーム化）

---

## 背景

v8.7.0 で E0008（引数数不一致）を実装したが、**ジェネリック関数のみ**が対象だった。
非ジェネリック関数（型変数なし）は env に戻り型のみ保存されており、`is_fn_scheme_str` が
false を返すため引数数チェックが行われない。

```
// v8.7.0 時点
fn double(x: Int) -> Int    → env: "double" → "Int"            ← 戻り型のみ
fn identity(x: A) -> A      → env: "identity" → "forall|A|A|A" ← スキーム形式
```

v8.8.0 では `fn_to_scheme_str` を変更し、**全ての user-defined 関数を統一したスキーム形式で保存**する。
これにより非ジェネリック関数でも引数数チェックが可能になる。

---

## 設計

### スキーム形式の統一

`make_fn_scheme_str` の早期リターン条件を削除し、常に `"forall|<vars>|<params>|<ret>"` を生成:

```
// Before (v8.7.0):
fn double(x: Int) -> Int     → "Int"                   (戻り型のみ)
fn greet(a: String) -> Unit  → "Unit"
fn f() -> Int                → "Int"
fn identity(x: A) -> A       → "forall|A|A|A"          (スキーム)

// After (v8.8.0):
fn double(x: Int) -> Int     → "forall||Int|Int"        (vars="", params="Int", ret="Int")
fn greet(a: String) -> Unit  → "forall||String|Unit"
fn f() -> Int                → "forall|||Int"           (vars="", params="", ret="Int")
fn identity(x: A) -> A       → "forall|A|A|A"           (変更なし)
```

`fn_scheme_section(s, idx)` は `"|"` で分割してインデックスを取得するため、
空セクション（`""`）も正しく処理される:
- `"forall|||Int".split("|") = ["forall", "", "", "Int"]` → section 3 = `"Int"` ✓
- `"forall||Int|Int".split("|") = ["forall", "", "Int", "Int"]` → section 2 = `"Int"` ✓

---

### 変更 1: `make_fn_scheme_str` — 常にスキーム形式を生成

```fav
// Before:
fn make_fn_scheme_str(vars_csv: String, params_semi: String, ret: String) -> String {
    if String.length(vars_csv) == 0 { ret }   ← 削除
    else { ... }

// After:
fn make_fn_scheme_str(vars_csv: String, params_semi: String, ret: String) -> String {
    String.concat("forall|",
    String.concat(vars_csv,
    String.concat("|",
    String.concat(params_semi,
    String.concat("|", ret)))))
}
```

### 変更 2: `fn_to_scheme_str` — 常に `make_fn_scheme_str` を使用

```fav
// Before:
fn fn_to_scheme_str(fd: FnDef) -> String {
    bind all_vars <- ...
    if List.length(all_vars) == 0 {
        type_expr_to_str(fd.ret)     ← 削除
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

### 変更 3: `register_variant` — `make_fn_scheme_str` を使用

v8.7.0 では `make_fn_scheme_str("", param_str, type_name)` が戻り値のみを返す問題を回避するため
直接文字列連結していた。この制約が解消されるので `make_fn_scheme_str` を使うよう簡略化:

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

### 変更 4: `infer_call`（非 HM パス）— 戻り型を正しく返す

`infer_call` は `infer_expr` から呼ばれる非 HM パス。
変更後、`env_lookup` が返す `ty` はスキーム文字列になるため、
戻り型を正しく抽出する必要がある:

```fav
fn infer_call(ns: String, fname: String, args: Expr, env: List<KVPair>) -> Result<String, String> {
    if ns == "" {
        match env_lookup(env, fname) {
            Some(ty) =>
                // スキーム形式なら戻り型を抽出; そうでなければ（lambda 等）そのまま返す
                if is_fn_scheme_str(ty) { Result.ok(fn_scheme_ret(ty)) }
                else { Result.ok(ty) }
            None => Result.ok("Unknown")
        }
    } else {
        // 既存の namespaced call 処理（変更なし）
        bind arg_tys <- infer_arg_tys(args, env)
        ...
    }
}
```

`infer_call` は arity チェックを行わない（E0007/E0008 は `infer_call_user` の HM パスで実施）。

---

## 動作確認

### 非ジェネリック関数の引数数チェック（新規）

```
fn add(a: Int, b: Int) -> Int { a + b }
add(1)          → E0008: add: expected 2 args but got 1
add(1, 2, 3)    → E0008: add: expected 2 args but got 3
add(1, 2)       → OK
```

### 0 引数関数の呼び出し

```
fn get_const() -> Int { 42 }
get_const()     → OK  (params_str = "", count = 0, n_actual = 0)
get_const(1)    → E0008: get_const: expected 0 args but got 1
```

### ジェネリック関数（変更なし）

```
fn identity(x: A) -> A { x }
identity(1, 2)  → E0008 (v8.7.0 から既存)
identity(1)     → OK
```

### 既存テスト

- `checker_fav_wire_valid_fn` — `add(1, 2)` が OK ✓
- `checker_fav_wire_generic_fn` — `identity(42)` が OK ✓
- `checker_fav_wire_self_check` — checker.fav 自身の self-check が通る ✓
- `checker_v87_tests` (3件) — `wrong_arity_e0008` が非ジェネリック関数にも適用されることを確認

---

## 注意事項

### `String.split("", sep)` の挙動

`params_semi = ""` のとき `String.split("", ";") = [""]`（空文字列1要素のリスト）。
`instantiate_fn_scheme` では `build_scheme_subst_inner([""], [], subst)` → `arg_tys` が空なので
早期リターン → `subst` を返す。問題なし。

### `vars = [""]` の `apply_scheme_subst` への影響

型変数なしの scheme では `vars_str = ""` → `String.split("", ",") = [""]`。
`apply_scheme_subst(ret_ty, [""], subst)` は `subst_lookup(subst, "")` → `None` →
`ret_ty` をそのまま返す。問題なし。

### `infer_call`（非 HM）vs `infer_call_user`（HM）の役割分担

- `infer_call` — 非 HM パス。戻り型を返すのみ、arity チェックなし
- `infer_call_user` — HM パス（`check_fn_def` → `infer_hm` 経由）。E0007/E0008 を検出

`check_fn_def` は `infer_hm` を使うため、関数ボディの型チェックは常に HM パスを通る。
`infer_call` が呼ばれるのは `infer_hm` の `_ =>` fallback（EBinOp 等）内の
ネストしたサブ式チェックが `infer_expr` → `infer_call` を辿るケースのみ。
これらは型推論の精度向上のみに使われており、arity チェックを省略しても安全。
