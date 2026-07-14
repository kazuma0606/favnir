# v43.5.0 実装計画 — ラムダ引数型推論（Contextual lambda inference）

## 前提

- v43.4.0 完了（2914 tests）
- `fav/Cargo.toml` version: `43.4.0`
- `infer_call` の `ns == "List"` ブランチは `infer_arg_tys` → `infer_generic_list` の固定パス
- `ELambda` 評価時のパラメータ型は常に `"Unknown"`
- `bind _ <-` は Result の短絡が効かない → `Result.and_then` ネストを使うこと（v43.4.0 で判明）

---

## タスク順序

```
T0 事前確認
T1 checker.fav — infer_list_lambda_call 追加 + infer_call 分岐追加
T2 driver.rs — v43500_tests 追加（v43400_tests の直前）
T3 Cargo.toml — version 43.4.0 → 43.5.0 + v43400_tests スタブ化
T4 CHANGELOG.md — v43.5.0 エントリ追加
T5 cargo test 実行・確認（2917 pass, 0 fail）
T6 バージョン管理ドキュメント更新
```

---

## T0 — 事前確認

1. `cargo test` 2914 / 0 確認
2. `Cargo.toml` version = `43.4.0` 確認
3. `infer_list_lambda_call` が checker.fav に存在しないことを確認

---

## T1 — checker.fav

### 挿入位置: `infer_call`（1848行）の直前

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

### 変更: `infer_call` の `else` ブランチ（1858行付近）

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

## T2 — driver.rs — v43500_tests

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

## T3 — Cargo.toml + v43400_tests スタブ化

```toml
version = "43.5.0"
```

`v43400_tests::cargo_toml_version_is_43_4_0` をスタブ化:

```rust
fn cargo_toml_version_is_43_4_0() {
    // Stubbed: version bumped to 43.5.0 -- assertion intentionally removed
}
```

---

## T4 — CHANGELOG.md

```markdown
## [v43.5.0] — 2026-07-12

### Added
- `fav/self/checker.fav`: `infer_list_lambda_call` — `List.map` / `List.filter` 呼び出し時にラムダパラメータへリスト要素型を伝播（contextual lambda inference）
- `v43500_tests`: `cargo_toml_version_is_43_5_0` / `contextual_lambda_map_propagates_elem_type` / `contextual_lambda_filter_preserves_elem_type`

### Changed
- `fav/self/checker.fav`: `infer_call` の `ns=="List"` ブランチを `map`/`filter` で `infer_list_lambda_call` に分岐
- `v43400_tests::cargo_toml_version_is_43_4_0` をスタブ化
```

---

## T5 — テスト実行

```bash
cargo test -j 8 -- --test-threads=8 2>&1 | grep "test result"
```

期待: `2917 passed; 0 failed`

---

## T6 — バージョン管理ドキュメント更新

- `versions/current.md` → v43.5.0 最新安定版（2917 tests）、次版 v43.6.0
- `versions/roadmap/roadmap-v43.1-v44.0.md` → v43.5.0 を `✅ COMPLETE（2026-07-12）`、推定 2916 → 実績 2917 に修正
- `versions/v40-v45/v43.5.0/tasks.md` → COMPLETE、全チェックボックス `[x]`
