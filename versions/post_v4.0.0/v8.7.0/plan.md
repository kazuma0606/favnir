# Favnir v8.7.0 実装計画

Date: 2026-05-30

---

## Phase A: `collect_variant_constructors` 追加

**変更ファイル**: `fav/self/checker.fav`

既存の `collect_fn_schemes` の直後に追加。

### A-1: `register_variant`

```fav
fn register_variant(type_name: String, v: VariantDef, env: List<KVPair>) -> List<KVPair> {
    match v.payload {
        None    => env_insert(env, v.name, type_name)
        Some(te) => {
            bind param_str <- type_expr_to_str(te)
            env_insert(env, v.name, make_fn_scheme_str("", param_str, type_name))
        }
    }
}
```

- 引数なしバリアント（`None`, `Red`）→ `env_insert(env, name, type_name)`
- 引数ありバリアント（`Some(te)`）→ `make_fn_scheme_str("", param_str, type_name)` でスキーム登録

### A-2: `register_variants_inner`

```fav
fn register_variants_inner(type_name: String, vs: List<VariantDef>, env: List<KVPair>) -> List<KVPair> {
    match List.first(vs) {
        None    => env
        Some(v) =>
            register_variants_inner(type_name, List.drop(vs, 1),
                register_variant(type_name, v, env))
    }
}
```

### A-3: `collect_variant_constructors`

```fav
fn collect_variant_constructors(items: List<Item>, env: List<KVPair>) -> List<KVPair> {
    match List.first(items) {
        None => env
        Some(item) => match item {
            IType(td) =>
                if td.is_record {
                    collect_variant_constructors(List.drop(items, 1), env)
                } else {
                    collect_variant_constructors(List.drop(items, 1),
                        register_variants_inner(td.name, td.variants, env))
                }
            _ => collect_variant_constructors(List.drop(items, 1), env)
        }
    }
}
```

注: `if td.is_record { ... } else { ... }` の内側で `register_variants_inner` を呼ぶ。

### A-4: `check(prog)` 更新

```fav
public fn check(prog: Program) -> Result<String, String> {
    bind init_env   <- env_empty()
    bind scheme_env <- collect_fn_schemes(prog.items, init_env)
    bind full_env   <- collect_variant_constructors(prog.items, scheme_env)
    check_items(prog.items, full_env)
}
```

---

## Phase B: E0007 — 未定義関数検出

**変更ファイル**: `fav/self/checker.fav`

### B-1: `infer_call_user` の `None` ケース変更

```fav
fn infer_call_user(fname: String, args: Expr, env: List<KVPair>, state: InfState) -> Result<InfResult, String> {
    match env_lookup(env, fname) {
        None =>
            Result.err(fmt_err("E0007",
                String.concat("undefined function: ", fname)))
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

---

## Phase C: E0008 — 引数数チェック

**変更ファイル**: `fav/self/checker.fav`

### C-1: `count_scheme_params` ヘルパー追加

`instantiate_fn_scheme` の直前に追加:

```fav
// Count semicolon-delimited params in a scheme params string.
// Returns 0 for empty string (0-param function).
fn count_scheme_params(s: String) -> Int {
    if String.length(s) == 0 { 0 }
    else {
        bind parts <- String.split(s, ";")
        List.length(parts)
    }
}
```

### C-2: `infer_call_user` にアリティチェック追加

Phase B の変更に追加:

```fav
Some(ty) => {
    if is_fn_scheme_str(ty) {
        bind arg_tys    <- infer_arg_tys(args, env)
        bind params_str <- fn_scheme_params_str(ty)
        bind n_expected <- count_scheme_params(params_str)
        bind n_actual   <- List.length(arg_tys)
        if n_expected != n_actual {
            Result.err(fmt_err("E0008",
                String.concat(fname,
                String.concat(": expected ",
                String.concat(Int.to_string(n_expected),
                String.concat(" args but got ",
                Int.to_string(n_actual)))))))
        } else {
            instantiate_fn_scheme(ty, arg_tys, state)
        }
    } else {
        Result.ok(inf_result_of(ty, state))
    }
}
```

注: Phase B と C は `infer_call_user` への変更をまとめて実施する。

---

## Phase D: 統合テスト

**変更ファイル**: `fav/src/driver.rs`（既存の `checker_fav_wire_tests` に追記）

### D-1: `undefined_fn_e0007`

```rust
#[test]
fn undefined_fn_e0007() {
    // foo は未定義 → E0007
    let src = "public fn main() -> Int { foo(1) }";
    let errors = run_check(src);
    assert!(
        errors.iter().any(|e| e.contains("E0007")),
        "expected E0007 for undefined function, got: {:?}", errors
    );
}
```

### D-2: `variant_constructor_ok`

```rust
#[test]
fn variant_constructor_ok() {
    // MyOpt は user-defined sum type → Some1(x) はコンストラクター → エラーなし
    let src = r#"
type MyOpt = | None1 | Some1(Int)
public fn main() -> MyOpt { Some1(42) }
"#;
    let errors = run_check(src);
    assert!(errors.is_empty(), "variant constructor should not cause E0007: {:?}", errors);
}
```

### D-3: `wrong_arity_e0008`（ジェネリック関数）

```rust
#[test]
fn wrong_arity_e0008() {
    // identity<A> は 1 引数 → 2 引数で呼ぶ → E0008
    let src = r#"
fn identity<A>(x: A) -> A { x }
public fn main() -> Int { identity(1, 2) }
"#;
    let errors = run_check(src);
    assert!(
        errors.iter().any(|e| e.contains("E0008")),
        "expected E0008 for arity mismatch, got: {:?}", errors
    );
}
```

### D-4: 既存テスト確認

```
cargo test checker_fav_wire_tests  # 既存 3 件 + 新規 3 件 = 6 件
cargo test                         # 全件
```

---

## Phase E: 最終確認

- `cargo test` — 1122 tests passing（+ 3 新規 = 1125）
- `fav check self/checker.fav` — checker.fav 自身をチェックしてエラーなし
  （`--legacy-check` フラグで Rust checker を使う）
- tasks.md 完了・commit

---

## 実装ノート

### `make_fn_scheme_str("", param_str, type_name)` の挙動確認

```
make_fn_scheme_str("", "Int", "Color") = ?
```

`make_fn_scheme_str` は `if String.length(vars_csv) == 0 { ret }` で ret だけを返す。
つまり `"Color"` になってしまい、スキーム形式にならない。

**修正が必要**: Phase A で `register_variant`（引数ありバリアント）は
`make_fn_scheme_str` を使わずに直接スキーム文字列を構築する:

```fav
Some(te) => {
    bind param_str <- type_expr_to_str(te)
    // make_fn_scheme_str("", ...) は vars 空のとき ret だけ返すのでここでは直接組み立てる
    bind scheme <- String.concat("forall||",
                   String.concat(param_str,
                   String.concat("|", type_name)))
    env_insert(env, v.name, scheme)
}
```

これで `"Blue"` → `"forall||Int|Color"` が正しく登録される。

### `infer_call_user` で `is_fn_scheme_str` が variant コンストラクターに対して true を返すか

`"forall||Int|Color"` → `String.starts_with(s, "forall|")` → true ✓
`"Color"` (引数なし) → false → `else` ブランチ → `inf_result_of("Color", state)` → 問題なし ✓

### `count_scheme_params` の引数なしコンストラクター

引数ありバリアントのスキーム `"forall||Int|Color"`:
- `fn_scheme_params_str` = `"Int"`
- `count_scheme_params("Int")` = 1
- `identity(1, 2)` のようなケースでアリティミスマッチを正しく検出

### checker.fav 自身のセルフチェック

`collect_variant_constructors` を追加後、checker.fav の定義する型
(`Type`, `Expr`, `Pat`, `Item` 等) の variant が env に登録される。
これにより checker.fav 内での variant コンストラクター使用も型チェックされる。
既存の variant 使用が正しければエラーなし。
