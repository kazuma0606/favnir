# v43.5.0 タスク — ラムダ引数型推論（Contextual lambda inference）

## ステータス: COMPLETE（2026-07-12）— 2917 tests

---

## T0 — 事前確認

- [x] `cargo test` 2914 / 0 確認
- [x] `Cargo.toml` version = `43.4.0` 確認
- [x] `infer_list_lambda_call` が `fav/self/checker.fav` に存在しないことを確認

---

## T1 — checker.fav — infer_list_lambda_call 追加 + infer_call 分岐追加

- [x] `infer_list_lambda_call` を `infer_call` の直前に挿入
  - `v43.5.0: contextual lambda inference` コメント付き
  - `EArgList` パターンマッチで list_expr → rest_args → lam_expr と分解
  - `unwrap_ty(infer_expr(list_expr, env))` → `type_str_inner` で elem_ty 取得
  - `env_insert(env, param, param_ty)` で lambda param に elem_ty を付与
  - `Result.and_then(infer_expr(body, lam_env), |ret_ty| ...)` で body 評価
  - `fname == "map"` → `wrap_in("List", ret_ty)`、`filter` → `list_ty`
  - 非 ELambda / 非 EArgList のフォールバックは `infer_generic_list`
- [x] `infer_call` の `else` ブランチを変更
  - `(ns == "List") && ((fname == "map") || (fname == "filter"))` → `infer_list_lambda_call(fname, args, env)`
  - それ以外は既存の `bind arg_tys <- infer_arg_tys(args, env)` + namespace 分岐を維持

---

## T2 — driver.rs — v43500_tests 追加

- [x] `v43400_tests` モジュールの直前に `v43500_tests` を挿入
- [x] `cargo_toml_version_is_43_5_0` テスト追加（`Cargo.toml` に `"43.5.0"` を含む）
- [x] `contextual_lambda_map_propagates_elem_type` テスト追加
  - `fn f(xs: List<Int>) -> List<Int> { List.map(xs, |x| x) }` → `run_checker_fav` が `Ok`
- [x] `contextual_lambda_filter_preserves_elem_type` テスト追加
  - `fn g(xs: List<Int>) -> List<Int> { List.filter(xs, |x| x > 0) }` → `run_checker_fav` が `Ok`

---

## T3 — Cargo.toml + v43400_tests スタブ化

- [x] `fav/Cargo.toml` version を `43.4.0` → `43.5.0` に更新
- [x] `v43400_tests::cargo_toml_version_is_43_4_0` の assert を削除してスタブ化

---

## T4 — CHANGELOG.md

- [x] v43.5.0 エントリ追加
  - Added: `infer_list_lambda_call` + `v43500_tests` 3 件
  - Changed: `infer_call` の `ns=="List"` ブランチを `map`/`filter` で分岐、`cargo_toml_version_is_43_4_0` スタブ化

---

## T5 — テスト実行・確認

- [x] `cargo test -j 8 -- --test-threads=8` 実行
- [x] 2917 passed; 0 failed 確認
- [x] `v43500_tests` 3 件 pass 確認

---

## T6 — バージョン管理ドキュメント更新

- [x] `versions/current.md` → v43.5.0 最新安定版（2917 tests）、次版 v43.6.0
- [x] `versions/roadmap/roadmap-v43.1-v44.0.md` → v43.5.0 を `✅ COMPLETE（2026-07-12）`、推定 2917 → 実績 2917 に修正
- [x] `versions/v40-v45/v43.5.0/tasks.md` → COMPLETE、全チェックボックス `[x]`

---

## 実装時の知見

- `type_str_inner(s: String) -> String` と `unwrap_ty(r: Result<String, String>) -> String` は両方 `String` を返す（Result ではない）
  → `bind x <- String_expr` は Favnir で let バインドとして有効（短絡なし）
- `bind param_ty <- if ... { "Unknown" } else { elem_ty }` は正当な Favnir 構文
- `ECollect`（リストリテラル `[1,2,3]`）から `infer_expr` は `"Unknown"` を返すため、リテラル直接の要素型伝播は非対応（既知制限）
- 型付きパラメータ経由（`xs: List<Int>`）では正常に動作
