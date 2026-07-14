# v43.5.0 仕様書 — ラムダ引数型推論（Contextual lambda inference）

## 概要

ロードマップ: "パイプライン上流の型を下流ラムダの引数型に伝播"

```favnir
// 推論前（明示）
[1, 2, 3] |> List.map(|x: Int| x * 2)

// 推論後（List<Int> から x: Int が伝播）
fn process(xs: List<Int>) -> List<Int> { List.map(xs, |x| x * 2) }
```

---

## 現状と問題

`infer_call` の `ns == "List"` ブランチは `infer_arg_tys(args, env)` で全引数を評価してから
`infer_generic_list(fname, arg_tys)` を呼ぶ。

`infer_arg_tys` でラムダを評価すると `infer_expr` の `ELambda` ケースが呼ばれ、
パラメータは常に `"Unknown"` として扱われる（行 1905）：

```favnir
ELambda({ _0: param, _1: body }) => {
    bind lam_env <- env_insert(env, param, "Unknown");  // ← 型情報なし
    Result.and_then(infer_expr(body, lam_env), |bty| Result.ok("Fn"))
}
```

これにより：
- `List.map(xs, |x| x)` の戻り型が `"List"` になり型情報が失われる
- `|x| x * "hello"` のような型エラーが `x: Unknown` のため検出されない（Unknown * String → Unknown で通過）

---

## 解決

`infer_call` の `ns == "List" && (fname == "map" || fname == "filter")` ケースを
新関数 `infer_list_lambda_call` に分岐し、
**ラムダを評価する前にリストの要素型を確定してラムダパラメータに渡す**。

---

## 変更ファイル一覧

| ファイル | 変更内容 |
|---|---|
| `fav/self/checker.fav` | `infer_list_lambda_call` 追加、`infer_call` の `ns=="List"` ブランチを分岐 |
| `fav/src/driver.rs` | `v43500_tests` 追加（3 件） |
| `fav/Cargo.toml` | version 43.4.0 → 43.5.0 |
| `CHANGELOG.md` | v43.5.0 エントリ追加 |

**`compiler.fav` は変更不要**: compiler.fav は型推論を担わず AST → bytecode 変換のみを行うため、型推論ロジック追加の影響を受けない。

---

## T1 — `fav/self/checker.fav`

### 前提

- スキームフォーマット: `forall|vars|params|ret`（既存）
- `type_str_inner(s: String) -> String` — `List<T>` の内部型を返す（`"List<Int>"` → `"Int"`）。**Result ではなく String を返す**（内部型なし = `""`）
- `unwrap_ty(r: Result<String, String>) -> String` — Result を String に変換（Err → `"Unknown"`）。**Result ではなく String を返す**
- `wrap_in("List", "Int")` = `"List<Int>"` — 外部型でラップ
- `infer_arg_tys` は `List.push` のため arg 順が **逆順** になる（v43.4.0 で判明）
  → ただし本関数は `args: Expr` を直接パターンマッチするため逆順の影響を受けない
- `bind x <- String_expr` は Favnir でlet バインドとして有効（Result を返さない式でも `bind` 構文を使える）。Result を返す式の場合のみ短絡が発生する

### 追加: `infer_list_lambda_call`（`infer_call` の直前に挿入）

```favnir
// v43.5.0: contextual lambda inference — propagate list element type to lambda param
fn infer_list_lambda_call(fname: String, args: Expr, env: List<KVPair>) -> Result<String, String> {
    match args {
        EArgList({ _0: list_expr, _1: rest_args }) => {
            bind list_ty <- unwrap_ty(infer_expr(list_expr, env));
            bind elem_ty <- type_str_inner(list_ty);
            bind param_ty <- if String.length(elem_ty) == 0 { "Unknown" } else { elem_ty };
            match rest_args {
                EArgList({ _0: lam_expr, _1: _ }) => {
                    match lam_expr {
                        ELambda({ _0: param, _1: body }) => {
                            bind lam_env <- env_insert(env, param, param_ty);
                            Result.and_then(infer_expr(body, lam_env), |ret_ty|
                                if fname == "map" {
                                    Result.ok(wrap_in("List", ret_ty))
                                } else {
                                    Result.ok(list_ty)
                                })
                        }
                        _ => {
                            bind arg_tys <- infer_arg_tys(args, env);
                            Result.ok(infer_generic_list(fname, arg_tys))
                        }
                    }
                }
                _ => {
                    bind arg_tys <- infer_arg_tys(args, env);
                    Result.ok(infer_generic_list(fname, arg_tys))
                }
            }
        }
        _ => {
            bind arg_tys <- infer_arg_tys(args, env);
            Result.ok(infer_generic_list(fname, arg_tys))
        }
    }
}
```

### 変更: `infer_call` の `else` ブランチ（ns != "" の分岐）

**変更前:**
```favnir
    } else {
        bind arg_tys <- infer_arg_tys(args, env);
        if ns == "List" {
            Result.ok(infer_generic_list(fname, arg_tys))
        } else {
            if ns == "Option" {
                Result.ok(infer_generic_opt(fname, arg_tys))
            } else {
                if ns == "Result" {
                    Result.ok(infer_generic_res(fname, arg_tys))
                } else {
                    Result.ok(builtin_ret_ty(ns, fname))
                }
            }
        }
    }
```

**変更後:**
```favnir
    } else {
        if (ns == "List") && ((fname == "map") || (fname == "filter")) {
            // v43.5.0: contextual lambda inference — propagate list element type to lambda param
            infer_list_lambda_call(fname, args, env)
        } else {
            bind arg_tys <- infer_arg_tys(args, env);
            if ns == "List" {
                Result.ok(infer_generic_list(fname, arg_tys))
            } else {
                if ns == "Option" {
                    Result.ok(infer_generic_opt(fname, arg_tys))
                } else {
                    if ns == "Result" {
                        Result.ok(infer_generic_res(fname, arg_tys))
                    } else {
                        Result.ok(builtin_ret_ty(ns, fname))
                    }
                }
            }
        }
    }
```

---

## T2 — `fav/src/driver.rs` — v43500_tests

`v43400_tests` モジュールの直前に挿入:

```rust
// -- v43500_tests (v43.5.0) -- ラムダ引数型推論（Contextual lambda inference）--
#[cfg(test)]
mod v43500_tests {
    #[test]
    fn cargo_toml_version_is_43_5_0() {
        // NOTE: この assert は次バージョン bump 時にスタブ化すること
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("43.5.0"), "Cargo.toml must contain version 43.5.0");
    }
    #[test]
    fn contextual_lambda_map_propagates_elem_type() {
        // List<Int> の要素型 Int がラムダパラメータに伝播し List<Int> が返る
        use crate::frontend::parser::Parser;
        use crate::middle::ast_lower_checker::lower_program;
        use crate::checker_fav_runner::run_checker_fav;
        let src = r#"
fn f(xs: List<Int>) -> List<Int> { List.map(xs, |x| x) }
"#;
        let prog = Parser::parse_str(src, "v43500_map.fav").expect("parse");
        let result = run_checker_fav(lower_program(&prog));
        assert!(result.is_ok(), "map with elem type propagation should pass: {:?}", result.err());
    }
    #[test]
    fn contextual_lambda_filter_preserves_elem_type() {
        // List<Int> の要素型 Int がラムダパラメータに伝播し List<Int> が返る
        use crate::frontend::parser::Parser;
        use crate::middle::ast_lower_checker::lower_program;
        use crate::checker_fav_runner::run_checker_fav;
        let src = r#"
fn g(xs: List<Int>) -> List<Int> { List.filter(xs, |x| x > 0) }
"#;
        let prog = Parser::parse_str(src, "v43500_filter.fav").expect("parse");
        let result = run_checker_fav(lower_program(&prog));
        assert!(result.is_ok(), "filter with elem type propagation should pass: {:?}", result.err());
    }
}
```

---

## 完了条件

- `cargo test` 2917 tests passed, 0 failed（2914 + 3）
- `v43500_tests` 3 件 pass
- `contextual_lambda_map_propagates_elem_type`: `fn f(xs: List<Int>) -> List<Int> { List.map(xs, |x| x) }` が型エラーなし
- `contextual_lambda_filter_preserves_elem_type`: `fn g(xs: List<Int>) -> List<Int> { List.filter(xs, |x| x > 0) }` が型エラーなし

---

## 影響範囲

- **`List.map` / `List.filter` のみ変更**: `list_fn_tys` 等の他の List 関数は既存の `infer_generic_list` を使用（回帰なし）
- **ラムダ以外の引数（関数参照等）はフォールバック**: `ELambda` でない場合は `infer_generic_list` に委譲
- **要素型不明（Unknown）の場合は既存挙動**: `type_str_inner` が `""` を返す場合は `param_ty = "Unknown"` で従来通り
- **`infer_call_hm` 経由で呼ばれる**: `check_fn_def` → `infer_hm` → `infer_call_hm` → `infer_call` → `infer_list_lambda_call` のパスで動作
- **`infer_expr` 経由のパスも同様に改善**: `infer_expr` → `infer_call` → `infer_list_lambda_call`
- **v43.4.0 の `instantiate_fn_scheme` への影響なし**: `List.map`/`List.filter` はスキームを経由しない（`ns != ""` のパスは `infer_call_user` を呼ばない）
- **注意**: `[1, 2, 3]` リストリテラルは `ECollect` であり `infer_expr` が `"Unknown"` を返すため、リテラルから直接の要素型伝播は非対応（既知制限）。型付きパラメータ経由（`xs: List<Int>`）では正常に動作する。
