# v43.3.0 タスク — ジェネリック型引数推論（Call-site inference）

## ステータス: COMPLETE（2026-07-12）— 2910 tests

---

## T0 — 事前確認

- [x] `cargo test` 2907 / 0 確認
- [x] `Cargo.toml` version = `43.2.0` 確認
- [x] `infer_call` に `v43.3.0` コメントがないことを確認

---

## T1 — checker.fav — infer_call の call-site instantiation 修正

- [x] `fav/self/checker.fav` の `infer_call`（ns == "" かつ is_fn_scheme_str 分岐）を修正
- [x] `v43.3.0: call-site generic instantiation` コメント追加
- [x] `infer_arg_tys` → `fn_scheme_vars_str` → `inf_state_new` → `instantiate_fn_scheme` の順で呼び出し
- [x] `vars_str` が空の場合は従来通り `fn_scheme_ret(ty)` を返す（回帰なし）

---

## T2 — driver.rs — v43300_tests 追加

- [x] `v43200_tests` モジュールの直前に `v43300_tests` を挿入
- [x] `cargo_toml_version_is_43_3_0` テスト追加
- [x] `call_site_inference_identity_ok` テスト追加（`identity("hello") -> String` が型エラーなし）
- [x] `call_site_inference_wrong_return_e0009` テスト追加（`-> Int { identity("hello") }` が E0009）

---

## T3 — Cargo.toml + v43200_tests スタブ化

- [x] `fav/Cargo.toml` version を `43.2.0` → `43.3.0` に更新
- [x] `v43200_tests::cargo_toml_version_is_43_2_0` をスタブ化（assert 削除）

---

## T4 — CHANGELOG.md

- [x] v43.3.0 エントリ追加（Fixed: infer_call ジェネリック推論バグ修正、Added: v43300_tests 3 件）

---

## T5 — テスト実行・確認

- [x] `cargo test -j 8 -- --test-threads=8` 実行
- [x] 2910 passed; 0 failed 確認
- [x] `v43300_tests` 3 件 pass 確認

---

## T6 — バージョン管理ドキュメント更新

- [x] `versions/current.md` → v43.3.0 最新安定版（2910 tests）、次版 v43.4.0
- [x] `versions/roadmap/roadmap-v43.1-v44.0.md` → v43.3.0 を `✅ COMPLETE（2026-07-12）`、推定 2910 → 実績 2910 に修正
- [x] `versions/v40-v45/v43.3.0/tasks.md` → COMPLETE、全チェックボックス `[x]`

---

## コードレビュー指摘

（実装後に code-reviewer を実行する）
