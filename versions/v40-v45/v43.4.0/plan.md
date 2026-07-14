# v43.4.0 実装計画 — ジェネリック推論: 曖昧ケース検出（E0412）

## 前提

- v43.3.0 完了（2910 tests）
- `fav/Cargo.toml` version: `43.3.0`
- `instantiate_fn_scheme` は実装済み（`build_scheme_subst_inner` → `unify_deep`）
- 型変数競合は現状 E0005 として返るが、E0412 としてユーザーに伝えるべき

---

## タスク順序

```
T0 事前確認
T1 checker.fav — check_scheme_var_ambiguity 追加 + instantiate_fn_scheme pre-check 挿入
T2 error_catalog.rs — E0412 エントリ追加
T3 driver.rs — v43400_tests 追加（v43300_tests の直前）
T4 Cargo.toml — version 43.3.0 → 43.4.0 + v43300_tests スタブ化
T5 CHANGELOG.md — v43.4.0 エントリ追加
T6 cargo test 実行・確認（2913 pass, 0 fail）
T7 バージョン管理ドキュメント更新
```

---

## T0 — 事前確認

1. `cargo test` 2910 / 0 確認
2. `Cargo.toml` version = `43.3.0` 確認
3. `instantiate_fn_scheme` に `v43.4.0` コメントがないことを確認

---

## T1 — checker.fav

### 挿入位置: `instantiate_fn_scheme`（2225行）の直前

以下の 2 関数を追加する:

```favnir
// v43.4.0: E0412 — detect conflicting type variable bindings at call-site
fn check_scheme_var_ambiguity_inner(param_tys: List<String>, arg_tys: List<String>, seen: List<KVPair>) -> Result<String, String> {
    match List.first(param_tys) {
        None => Result.ok("ok")
        Some(pt) => {
            match List.first(arg_tys) {
                None => Result.ok("ok")
                Some(at) => {
                    if is_type_var(pt) {
                        match subst_lookup(seen, pt) {
                            None => check_scheme_var_ambiguity_inner(List.drop(param_tys, 1), List.drop(arg_tys, 1), subst_insert(seen, pt, at))
                            Some(prev) => if prev == at {
                                check_scheme_var_ambiguity_inner(List.drop(param_tys, 1), List.drop(arg_tys, 1), seen)
                            } else {
                                Result.err(fmt_err("E0412", String.concat("ambiguous type variable ", String.concat(pt, String.concat(": bound to both ", String.concat(prev, String.concat(" and ", at)))))))
                            }
                        }
                    } else {
                        check_scheme_var_ambiguity_inner(List.drop(param_tys, 1), List.drop(arg_tys, 1), seen)
                    }
                }
            }
        }
    }
}

fn check_scheme_var_ambiguity(param_tys: List<String>, arg_tys: List<String>) -> Result<String, String> {
    check_scheme_var_ambiguity_inner(param_tys, arg_tys, subst_empty())
}
```

### 変更: `instantiate_fn_scheme` に pre-check 挿入

```favnir
fn instantiate_fn_scheme(scheme: String, arg_tys: List<String>, state: InfState) -> Result<InfResult, String> {
    bind vars_str <- fn_scheme_vars_str(scheme);
    bind params_str <- fn_scheme_params_str(scheme);
    bind ret_ty <- fn_scheme_ret(scheme);
    bind vars <- String.split(vars_str, ",");
    bind param_tys <- String.split(params_str, ";");
    // v43.4.0: detect ambiguous type variable conflict before building substitution
    bind _ <- check_scheme_var_ambiguity(param_tys, arg_tys);
    Result.and_then(build_scheme_subst_inner(param_tys, arg_tys, state.subst), |subst2| Result.ok(inf_result_of(apply_scheme_subst(ret_ty, vars, subst2), inf_state_new(subst2, state.counter))))
}
```

---

## T2 — error_catalog.rs

`// ── E0412〜E0419: 予約（型推論拡張 v43.3.0+ 用） ──────────────────────────────` を以下に置き換える（末尾ダッシュ列含む完全一致）:

```rust
// ── E0412: 型変数競合 (v43.4.0) ────────────────────────────────────────────
ErrorEntry {
    code: "E0412",
    title: "ambiguous type variable",
    category: "types",
    description: "A type variable in a generic function is bound to conflicting types at the call site. The same type variable appears in multiple parameter positions but the corresponding arguments have different types.",
    example: "fn f<A>(x: A, y: A) -> A { x }\nfn bad() -> Int { f(1, \"hello\") }  // A = Int AND A = String → E0412",
    fix: "Ensure all arguments corresponding to the same type variable have the same type.",
},
// ── E0413〜E0419: 予約（型推論拡張 v43.5.0+ 用） ──────────────────────────────
```

---

## T3 — driver.rs — v43400_tests

`v43300_tests` モジュールの直前に挿入:

```rust
// -- v43400_tests (v43.4.0) -- ジェネリック推論: 曖昧ケース検出（E0412）--
#[cfg(test)]
mod v43400_tests {
    #[test]
    fn cargo_toml_version_is_43_4_0() {
        // NOTE: この assert は次バージョン bump 時にスタブ化すること
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("43.4.0"), "Cargo.toml must contain version 43.4.0");
    }
    #[test]
    fn e0412_in_error_catalog() {
        let catalog = include_str!("error_catalog.rs");
        assert!(catalog.contains("E0412"), "E0412 must be in error_catalog.rs");
        assert!(catalog.contains("ambiguous type variable"), "E0412 title must be present");
    }
    #[test]
    fn e0412_conflicting_type_vars() {
        // fn f<A>(x: A, y: A) -> A { x } called with f(1, "hello") → E0412
        use crate::frontend::parser::Parser;
        use crate::middle::ast_lower_checker::lower_program;
        use crate::checker_fav_runner::run_checker_fav;
        let src = r#"
fn f<A>(x: A, y: A) -> A { x }
fn bad() -> Int { f(1, "hello") }
"#;
        let prog = Parser::parse_str(src, "v43400_e0412.fav").expect("parse");
        let result = run_checker_fav(lower_program(&prog));
        assert!(result.is_err(), "E0412 expected for conflicting type vars");
        let msgs = result.unwrap_err();
        assert!(
            msgs.iter().any(|m| m.contains("E0412")),
            "E0412 expected in errors, got: {:?}", msgs
        );
    }
    #[test]
    fn e0412_no_conflict_ok() {
        // pre-check 挿入後の正常系回帰確認: f(1, 2) は A=Int 同士で競合なし
        use crate::frontend::parser::Parser;
        use crate::middle::ast_lower_checker::lower_program;
        use crate::checker_fav_runner::run_checker_fav;
        let src = r#"
fn f<A>(x: A, y: A) -> A { x }
fn ok_call() -> Int { f(1, 2) }
"#;
        let prog = Parser::parse_str(src, "v43400_ok.fav").expect("parse");
        let result = run_checker_fav(lower_program(&prog));
        assert!(result.is_ok(), "no conflict: f(1, 2) should pass: {:?}", result.err());
    }
}
```

---

## T4 — Cargo.toml + v43300_tests スタブ化

```toml
version = "43.4.0"
```

`v43300_tests::cargo_toml_version_is_43_3_0` をスタブ化:

```rust
fn cargo_toml_version_is_43_3_0() {
    // Stubbed: version bumped to 43.4.0 -- assertion intentionally removed
}
```

---

## T5 — CHANGELOG.md

```markdown
## [v43.4.0] — 2026-07-12

### Fixed
- `fav/self/checker.fav`: ジェネリック関数呼び出し時に同一型変数が複数引数で異なる型に束縛された場合、E0005 ではなく E0412 `ambiguous type variable` を報告するよう修正（`check_scheme_var_ambiguity` pre-check を `instantiate_fn_scheme` に追加）

### Added
- `fav/src/error_catalog.rs`: E0412（ambiguous type variable）エントリ追加
- `v43400_tests`: `cargo_toml_version_is_43_4_0` / `e0412_in_error_catalog` / `e0412_conflicting_type_vars`

### Changed
- `v43300_tests::cargo_toml_version_is_43_3_0` をスタブ化
```

---

## T6 — テスト実行

```bash
cargo test -j 8 -- --test-threads=8 2>&1 | grep "test result"
```

期待: `2914 passed; 0 failed`

---

## T7 — バージョン管理ドキュメント更新

- `versions/current.md` → v43.4.0 最新安定版（2914 tests）、次版 v43.5.0
- `versions/roadmap/roadmap-v43.1-v44.0.md` → v43.4.0 を `✅ COMPLETE（2026-07-12）`、推定 2913 → 実績 2914 に修正
- `versions/v40-v45/v43.4.0/tasks.md` → COMPLETE、全チェックボックス `[x]`
