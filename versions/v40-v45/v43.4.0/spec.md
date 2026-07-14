# v43.4.0 仕様書 — ジェネリック推論: 曖昧ケース検出（E0412）

## 概要

ロードマップ: "複数の型変数が競合する場合に E0412 ambiguous type variable を報告"

`fn f<A>(x: A, y: A) -> A { x }` を `f(1, "hello")` のように呼ぶと、
`A` が `Int` と `String` の両方に束縛されようとする。これを E0412 として報告する。

### 現状と問題

`instantiate_fn_scheme` → `build_scheme_subst_inner` → `unify_deep` の連鎖で、
型変数が既に別の型に束縛されている場合 E0005 (`conflict`) が返される。
E0005 はジェネリック関数の型変数競合に特化したメッセージではなく、
ユーザーにとって分かりにくい。

### 解決

`instantiate_fn_scheme` の冒頭で **コールサイトの型変数競合を事前に検出** し、
E0005 より前に E0412 `ambiguous type variable` を返す。

---

## 変更ファイル一覧

| ファイル | 変更内容 |
|---|---|
| `fav/self/checker.fav` | `check_scheme_var_ambiguity` + `check_scheme_var_ambiguity_inner` 追加、`instantiate_fn_scheme` に pre-check 挿入 |
| `fav/src/error_catalog.rs` | E0412 エントリ追加 |
| `fav/src/driver.rs` | `v43400_tests` 追加（4 件） |
| `fav/Cargo.toml` | version 43.3.0 → 43.4.0 |
| `CHANGELOG.md` | v43.4.0 エントリ追加 |

---

## T1 — `fav/self/checker.fav`

**前提**: スキームフォーマットは `forall|vars_csv|params_semi|ret`。`|` はフォーマット区切り文字であり、型名（`Int`, `String`, `A` 等）には `|` は含まれない。`fn_scheme_params_str` は `;` 区切りのパラメータ文字列を返す（例: `A;A`）。

### 追加: `check_scheme_var_ambiguity_inner`（`instantiate_fn_scheme` の直前）

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

### 変更: `instantiate_fn_scheme` — pre-check 挿入

**変更前:**
```favnir
fn instantiate_fn_scheme(scheme: String, arg_tys: List<String>, state: InfState) -> Result<InfResult, String> {
    bind vars_str <- fn_scheme_vars_str(scheme);
    bind params_str <- fn_scheme_params_str(scheme);
    bind ret_ty <- fn_scheme_ret(scheme);
    bind vars <- String.split(vars_str, ",");
    bind param_tys <- String.split(params_str, ";");
    Result.and_then(build_scheme_subst_inner(param_tys, arg_tys, state.subst), |subst2| Result.ok(inf_result_of(apply_scheme_subst(ret_ty, vars, subst2), inf_state_new(subst2, state.counter))))
}
```

**変更後:**
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

## T2 — `fav/src/error_catalog.rs` — E0412 追加

`// ── E0412〜E0419: 予約（型推論拡張 v43.3.0+ 用） ──────────────────────────────` コメントを以下に置き換える（末尾ダッシュ列含む完全一致）:

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

## T3 — `fav/src/driver.rs` — v43400_tests

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

## 完了条件

- `cargo test` 2914 tests passed, 0 failed（2910 + 4）
- `v43400_tests` 4 件 pass
- `e0412_conflicting_type_vars`: `f(1, "hello")` が E0412 を返す（他関数の型チェックは継続される — フェイル・ファスト停止なし）
- `e0412_no_conflict_ok`: `f(1, 2)` が型エラーなしで pass（pre-check 挿入後の回帰確認）
- `e0412_in_error_catalog`: E0412 エントリ存在確認

---

## 影響範囲

- **`instantiate_fn_scheme` の変更**: pre-check が先に実行されるため、型変数競合ケースのみ E0412 に変わる。競合がなければ従来通り `build_scheme_subst_inner` が実行される。
- **非ジェネリック関数への影響なし**: `is_fn_scheme_str(ty)` が false のパスは変更なし。
- **両呼び出しパスに一貫して適用**: `instantiate_fn_scheme` を呼ぶすべてのパス（`infer_call`（行 1859）の非HMパス、`infer_call_user`（行 2259）のHMパス）で E0412 が報告されるようになる。
- **引数数不一致（E0008）との優先順位**: `check_scheme_var_ambiguity` は `arg_tys` が尽きた時点でスキップする（`None => Result.ok("ok")`）。引数数不一致は `infer_call_user` の E0008 チェックで別途検出されるため E0008 が優先される。
- **E0412 検出後の型チェック継続**: E0412 は当該関数呼び出し式の型推論を中断するが、`check_items` は他の関数定義の型チェックを継続する（フェイル・ファスト停止なし）。
- **`check_scheme_var_ambiguity` は型変数（単一大文字 A-Z）のみ対象**: `is_type_var` の判定に従う。
