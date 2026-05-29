# Favnir v8.7.0 Spec

Date: 2026-05-30
Theme: checker.fav の user-defined fn 完全チェック（未定義関数検出 + variant constructor 登録）

---

## 背景

v8.1.0 以降、`fav check` と `fav run` の型チェックは checker.fav 経由になった。
しかし checker.fav には未検出の主要エラーカテゴリが残っている:

| エラー | Rust checker | checker.fav | 状態 |
|---|---|---|---|
| 演算子型ミスマッチ（E0001/E0002） | ✓ | ✓ | 完了 |
| 未宣言エフェクト（E0003） | ✓ | ✓ | 完了 |
| match 非網羅（E0004） | ✓ | ✓ | 完了 |
| 型統一失敗（E0005/E0006） | ✓ | ✓ | 完了 |
| **未定義 user-defined fn 呼び出し** | ✓ | ✗ | **v8.7.0** |
| **ジェネリック fn の引数数不一致** | ✓ | ✗ | **v8.7.0** |

v8.7.0 では上記 2 件を checker.fav に実装する。

---

## 現状分析

### `infer_call_user` の問題

```fav
fn infer_call_user(fname, args, env, state):
    match env_lookup(env, fname):
        None     => Result.ok(inf_result_of("Unknown", state))  ← 未定義でも無視
        Some(ty) => ...
```

`env_lookup` が `None` を返した場合（未定義関数）、エラーを出さずに "Unknown" を返す。

### なぜ今まで `None` が正当だったか

`collect_fn_schemes` は user-defined 関数のみを env に登録する。
**user-defined sum type のバリアントコンストラクター**（e.g. `Blue(5)` の `Blue`）は
関数定義ではないため登録されない。
その結果、`Blue(5)` は `ECall("", "Blue", args)` として `infer_call_user` に渡り、
`None` ケースに落ちていた。

**E0007 を実装するには `collect_variant_constructors` で事前登録が必須。**

### `fn_to_scheme_str` の制限

非ジェネリック関数（型変数なし）は env に戻り型のみ保存される:
```
fn double(x: Int) -> Int → env: "double" → "Int"  (引数情報なし)
```

ジェネリック関数のみスキーム形式で保存:
```
fn identity<A>(x: A) -> A → env: "identity" → "forall|A|A|A"
```

v8.7.0 では**ジェネリック関数の引数数チェックのみ実装**する（非ジェネリック関数の引数チェックは v8.8.0 以降）。

---

## 設計

### 新エラーコード

| コード | 意味 | 例 |
|---|---|---|
| E0007 | 未定義 user-defined 関数呼び出し | `foo(1)` — `foo` が未定義 |
| E0008 | ジェネリック関数の引数数不一致 | `identity(1, 2)` — `identity<A>(x: A)` は 1 引数 |

### Phase A: `collect_variant_constructors`

sum type 定義から variant を env に登録する:

```fav
// バリアント1件を env に登録する
fn register_variant(type_name: String, v: VariantDef, env: List<KVPair>) -> List<KVPair> {
    match v.payload {
        None =>
            // 引数なし: "Red" → "Color"（戻り型のみ）
            env_insert(env, v.name, type_name)
        Some(te) => {
            // 引数あり: "Blue" → "forall||Int|Color"（スキーム）
            bind param_str <- type_expr_to_str(te)
            env_insert(env, v.name, make_fn_scheme_str("", param_str, type_name))
        }
    }
}

// TypeDef の全 variant を登録
fn register_type_variants(td: TypeDef, env: List<KVPair>) -> List<KVPair> {
    if td.is_record { env }   // レコード型はコンストラクターなし
    else { register_variants_inner(td.name, td.variants, env) }
}

fn register_variants_inner(type_name: String, vs: List<VariantDef>, env: List<KVPair>) -> List<KVPair> {
    match List.first(vs) {
        None    => env
        Some(v) => register_variants_inner(type_name, List.drop(vs, 1),
                       register_variant(type_name, v, env))
    }
}

fn collect_variant_constructors(items: List<Item>, env: List<KVPair>) -> List<KVPair> {
    match List.first(items) {
        None => env
        Some(item) => match item {
            IType(td) => collect_variant_constructors(List.drop(items, 1),
                             register_type_variants(td, env))
            _         => collect_variant_constructors(List.drop(items, 1), env)
        }
    }
}
```

`check(prog)` を更新:

```fav
public fn check(prog: Program) -> Result<String, String> {
    bind init_env    <- env_empty()
    bind scheme_env  <- collect_fn_schemes(prog.items, init_env)
    bind full_env    <- collect_variant_constructors(prog.items, scheme_env)
    check_items(prog.items, full_env)
}
```

### Phase B: E0007 — 未定義関数検出

`infer_call_user` の `None` ケースを変更:

```fav
fn infer_call_user(fname, args, env, state):
    match env_lookup(env, fname):
        None =>
            Result.err(fmt_err("E0007",
                String.concat("undefined function: ", fname)))
        Some(ty) => ...  // 変更なし
```

### Phase C: E0008 — ジェネリック関数の引数数チェック

`count_scheme_params(params_semi: String) -> Int` ヘルパーを追加:

```fav
fn count_scheme_params(s: String) -> Int {
    if String.length(s) == 0 { 0 }
    else {
        bind parts <- String.split(s, ";")
        List.length(parts)
    }
}
```

`infer_call_user` でジェネリック関数の引数数を検証:

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

---

## 制約・スコープ外

### 非ジェネリック関数の引数数チェック（スコープ外）

非ジェネリック関数（型変数なし）の env エントリは戻り型のみ（e.g. `"Int"`）。
引数数情報がないため v8.7.0 では引数数チェック不可。
解決には `fn_to_scheme_str` の変更（全関数をスキーム形式で保存）が必要 → v8.8.0 以降。

### 未定義変数（EVar）の検出（スコープ外）

`infer_expr` の `EVar` ケースでも未定義変数を検出できる。
しかしバリアントコンストラクターを引数なしで式として使う場合
（e.g. `bind x <- None` → `EVar("None")`）が誤検知になるリスクがある。
`None` / `Some` 等の組み込みコンストラクターを env に登録する仕組みが必要 → 将来版。

### `infer_call`（非 HM パス）の undefined 検出（スコープ外）

`infer_call` は `infer_expr` から直接呼ばれる（`infer_hm` の fallback パス）。
`infer_call_user` (HM パス) との重複を避けるため v8.7.0 では `infer_call_hm` 経由のみ変更。
