# v43.3.0 仕様書 — ジェネリック型引数推論（Call-site inference）

## 概要

ロードマップ: "checker.fav の型変数単一化（`unify`）に call-site 推論パスを追加"

`fn identity<A>(x: A) -> A { x }` を呼ぶとき、引数の型から `A = Int` / `A = String` を確定させる。

### 現状と問題

`infer_call`（非HMパス / `infer_expr` 経由）に **バグ**:

```favnir
// 問題の箇所 (checker.fav 1852行目付近)
Some(ty) => if is_fn_scheme_str(ty) {
    Result.ok(fn_scheme_ret(ty))  // ← "forall|A|A|A" の ret = "A" をそのまま返す！
```

`fn identity<A>(x: A) -> A { x }` を `infer_call` で評価すると、引数を見ずに `"A"` を返す。
HM パス（`infer_call_user`）は `instantiate_fn_scheme` で正しく推論するが、
`infer_arg_tys` → `infer_expr` → `infer_call` の連鎖では非HMパスが使われるため、
`identity(identity(42))` のようなネスト呼び出しで正しい型が得られない。

### 解決

`infer_call` の `ns == ""` かつ is_fn_scheme_str の分岐を、`infer_call_user` と同様に
`instantiate_fn_scheme` で型変数を解決するよう修正する。

---

## 変更ファイル一覧

| ファイル | 変更内容 |
|---|---|
| `fav/self/checker.fav` | `infer_call` の non-HM ジェネリック推論を修正 |
| `fav/src/driver.rs` | `v43300_tests` 追加（3 件） |
| `fav/Cargo.toml` | version 43.2.0 → 43.3.0 |
| `CHANGELOG.md` | v43.3.0 エントリ追加 |

---

## T1 — `fav/self/checker.fav`

### 変更対象: `infer_call`（1848 行付近）

**変更前:**
```favnir
fn infer_call(ns: String, fname: String, args: Expr, env: List<KVPair>) -> Result<String, String> {
    if ns == "" {
        match env_lookup(env, fname) {
            Some(ty) => if is_fn_scheme_str(ty) {
                Result.ok(fn_scheme_ret(ty))
            } else {
                Result.ok(ty)
            }
            None => Result.ok("Unknown")
        }
    } else {
        bind arg_tys <- infer_arg_tys(args, env);
        ...
    }
}
```

**変更後:**
```favnir
fn infer_call(ns: String, fname: String, args: Expr, env: List<KVPair>) -> Result<String, String> {
    if ns == "" {
        match env_lookup(env, fname) {
            Some(ty) => if is_fn_scheme_str(ty) {
                // v43.3.0: call-site generic instantiation — args drive type var resolution
                bind arg_tys <- infer_arg_tys(args, env);
                bind vars_str <- fn_scheme_vars_str(ty);
                if String.length(vars_str) == 0 {
                    Result.ok(fn_scheme_ret(ty))
                } else {
                    bind state0 <- inf_state_new(subst_empty(), 0);
                    Result.and_then(instantiate_fn_scheme(ty, arg_tys, state0), |r| Result.ok(r.ty))
                }
            } else {
                Result.ok(ty)
            }
            None => Result.ok("Unknown")
        }
    } else {
        bind arg_tys <- infer_arg_tys(args, env);
        ...（既存コードそのまま）
    }
}
```

**注意点:**
- `fn_scheme_vars_str` は `String`（非 Result）を返すが、checker.fav の `bind` は InfState 等の非 Result 値にも適用できる（checker.fav 2247行の `infer_call_user` が既に同パターンで `bind vars_str <- fn_scheme_vars_str(ty)` を使用）
- `inf_state_new(subst_empty(), 0)` は `InfState`（非 Result）を返すが同様に `bind` で受ける（checker.fav 1945行の `check_fn_def` と同パターン）
- `vars_str` が空（非ジェネリック）の場合は従来通り `fn_scheme_ret(ty)` を返す（回帰なし）
- `instantiate_fn_scheme` は `Result<InfResult, String>` を返す → `r.ty` を取り出して `Result.ok`
- 引数個数チェック（E0008）は本パスでは行わない — `infer_call_user` との設計的非対称。引数個数不一致は `instantiate_fn_scheme` のエラーとして伝播する（非HMパスの既存挙動と一致）

---

## T2 — `fav/src/driver.rs` — v43300_tests

`v43200_tests` の直前に挿入（降順慣例）:

```rust
// -- v43300_tests (v43.3.0) -- ジェネリック型引数推論（Call-site inference）--
#[cfg(test)]
mod v43300_tests {
    #[test]
    fn cargo_toml_version_is_43_3_0() {
        // NOTE: この assert は次バージョン bump 時にスタブ化すること
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("43.3.0"), "Cargo.toml must contain version 43.3.0");
    }
    #[test]
    fn call_site_inference_identity_ok() {
        // identity("hello") -> String の呼び出しが型エラーなしに通ること
        use crate::frontend::parser::Parser;
        use crate::middle::ast_lower_checker::lower_program;
        use crate::checker_fav_runner::run_checker_fav;
        let src = r#"
fn identity<A>(x: A) -> A { x }
fn main() -> String { identity("hello") }
"#;
        let prog = Parser::parse_str(src, "v43300_ok.fav").expect("parse");
        let result = run_checker_fav(lower_program(&prog));
        assert!(result.is_ok(), "generic call-site inference should pass: {:?}", result.err());
    }
    #[test]
    fn call_site_inference_wrong_return_e0009() {
        // identity("hello") -> String だが fn が -> Int を宣言 → E0009
        use crate::frontend::parser::Parser;
        use crate::middle::ast_lower_checker::lower_program;
        use crate::checker_fav_runner::run_checker_fav;
        let src = r#"
fn identity<A>(x: A) -> A { x }
fn wrong_return() -> Int { identity("hello") }
"#;
        let prog = Parser::parse_str(src, "v43300_e0009.fav").expect("parse");
        let result = run_checker_fav(lower_program(&prog));
        assert!(result.is_err(), "E0009 expected for return type mismatch");
        let msgs = result.unwrap_err();
        assert!(
            msgs.iter().any(|m| m.contains("E0009")),
            "E0009 expected in errors, got: {:?}", msgs
        );
    }
}
```

---

## 完了条件

- `cargo test` 2910 tests passed, 0 failed（2907 + 3）
- `v43300_tests` 3 件 pass
- `call_site_inference_identity_ok`: `identity("hello") -> String` が E0007/E0009 なし
- `call_site_inference_wrong_return_e0009`: `-> Int { identity("hello") }` が E0009 を返す

---

## 影響範囲

- **`infer_call` の変更**: `ns == ""` かつジェネリック関数の場合のみ変更。非ジェネリック（`vars_str == ""`）は従来通り。
- **HM パスへの影響なし**: `infer_call_hm` → `infer_call_user` は変更なし。
- **`infer_arg_tys` 経由のネスト呼び出し改善**: `identity(identity(42))` が `"A"` ではなく `"Int"` を返すようになる（直接のテストは省略、既存テストで回帰を検出）。
- **`infer_call` は `infer_expr` 経由の評価にのみ影響**: `check_fn_def` の主経路（`infer_hm`）は変更なし。
