# v43.3.0 実装計画 — ジェネリック型引数推論（Call-site inference）

## 前提

- v43.2.0 完了（2907 tests）
- `fav/Cargo.toml` version: `43.2.0`
- `infer_call_user` + `instantiate_fn_scheme` は既存実装済み（HM パスで動作）
- `infer_call`（非HMパス）がジェネリック関数で `"A"` を返すバグが存在

---

## タスク順序

```
T0 事前確認
T1 checker.fav — infer_call の call-site instantiation 修正
T2 driver.rs — v43300_tests 追加（v43200_tests の直前）
T3 Cargo.toml — version 43.2.0 → 43.3.0 + v43200_tests スタブ化
T4 CHANGELOG.md — v43.3.0 エントリ追加
T5 cargo test 実行・確認（2910 pass, 0 fail）
T6 バージョン管理ドキュメント更新
```

---

## T0 — 事前確認

1. `cargo test` 2907 / 0 確認
2. `Cargo.toml` version = `43.2.0` 確認
3. `infer_call` に `v43.3.0` コメントがないことを確認

---

## T1 — checker.fav

### 変更対象: `infer_call` の `ns == ""` かつ `is_fn_scheme_str` 分岐

`checker.fav` の `fn infer_call` を以下のように変更する:

**変更前（問題箇所）:**
```favnir
            Some(ty) => if is_fn_scheme_str(ty) {
                Result.ok(fn_scheme_ret(ty))
            } else {
```

**変更後:**
```favnir
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
```

`infer_call` の `else` 以下（`ns != ""` の分岐）は変更なし。

---

## T2 — driver.rs — v43300_tests

`v43200_tests` モジュールの直前に挿入:

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

## T3 — Cargo.toml + v43200_tests スタブ化

```toml
version = "43.3.0"
```

`v43200_tests::cargo_toml_version_is_43_2_0` をスタブ化:

```rust
fn cargo_toml_version_is_43_2_0() {
    // Stubbed: version bumped to 43.3.0 -- assertion intentionally removed
}
```

---

## T4 — CHANGELOG.md

```markdown
## [v43.3.0] — 2026-07-12

### Fixed
- `fav/self/checker.fav`: `infer_call`（非HMパス）でジェネリック関数の呼び出し時に型変数を解決せず生の型変数文字列（例: `"A"`）を返していたバグを修正。`instantiate_fn_scheme` を使ってコールサイトで型変数を確定するよう変更（v43.3.0 call-site generic instantiation）

### Added
- `v43300_tests`: `cargo_toml_version_is_43_3_0` / `call_site_inference_identity_ok` / `call_site_inference_wrong_return_e0009`

### Changed
- `v43200_tests::cargo_toml_version_is_43_2_0` をスタブ化
```

---

## T5 — テスト実行

```bash
cargo test -j 8 -- --test-threads=8 2>&1 | grep "test result"
```

期待: `2910 passed; 0 failed`

---

## T6 — バージョン管理ドキュメント更新

- `versions/current.md` → v43.3.0 最新安定版（2910 tests）、次版 v43.4.0
- `versions/roadmap/roadmap-v43.1-v44.0.md` → v43.3.0 を `✅ COMPLETE（2026-07-12）`、推定 2901 → 実績 2910 に修正
- `versions/v40-v45/v43.3.0/tasks.md` → COMPLETE、全チェックボックス `[x]`
