# v43.4.0 タスク — ジェネリック推論: 曖昧ケース検出（E0412）

## ステータス: COMPLETE（2026-07-12）— 2914 tests

---

## T0 — 事前確認

- [x] `cargo test` 2910 / 0 確認
- [x] `Cargo.toml` version = `43.3.0` 確認
- [x] `instantiate_fn_scheme` に `v43.4.0` コメントがないことを確認

---

## T1 — checker.fav — check_scheme_var_ambiguity 追加 + instantiate_fn_scheme pre-check

- [x] `check_scheme_var_ambiguity_inner` を `instantiate_fn_scheme` 直前に追加
- [x] `check_scheme_var_ambiguity` ラッパー関数追加
- [x] `instantiate_fn_scheme` を `Result.and_then(check_scheme_var_ambiguity(...), |_ambiguity_ok| ...)` に変更
- [x] `v43.4.0: detect ambiguous type variable conflict` コメント追加
- [x] 実装時判明: `bind _ <-` が短絡しないため `Result.and_then` のネスト形式に変更

---

## T2 — error_catalog.rs — E0412 追加

- [x] `// -- E0412-E0419: 予約` コメントを E0412 エントリ + 残余予約コメントに置き換え

---

## T3 — driver.rs — v43400_tests 追加

- [x] `v43300_tests` モジュールの直前に `v43400_tests` を挿入
- [x] `cargo_toml_version_is_43_4_0` テスト追加
- [x] `e0412_in_error_catalog` テスト追加
- [x] `e0412_conflicting_type_vars` テスト追加（`f(1, "hello")` → E0412）
- [x] `e0412_no_conflict_ok` テスト追加（`f(1, 2)` → ok、回帰確認）

---

## T4 — Cargo.toml + v43300_tests スタブ化

- [x] `fav/Cargo.toml` version を `43.3.0` → `43.4.0` に更新
- [x] `v43300_tests::cargo_toml_version_is_43_3_0` をスタブ化（assert 削除）

---

## T5 — CHANGELOG.md

- [x] v43.4.0 エントリ追加（Fixed: E0412 検出、Added: E0412 catalog + v43400_tests 4 件）

---

## T6 — テスト実行・確認

- [x] `cargo test -j 8 -- --test-threads=8` 実行
- [x] 2914 passed; 0 failed 確認
- [x] `v43400_tests` 4 件 pass 確認

---

## T7 — バージョン管理ドキュメント更新

- [x] `versions/current.md` → v43.4.0 最新安定版（2914 tests）、次版 v43.5.0
- [x] `versions/roadmap/roadmap-v43.1-v44.0.md` → v43.4.0 を `✅ COMPLETE（2026-07-12）`、推定 2914 → 実績 2914 に修正
- [x] `versions/v40-v45/v43.4.0/tasks.md` → COMPLETE、全チェックボックス `[x]`
- [x] `site/` MDX 更新は v43.13.0 でまとめて行う（本バージョンはスキップ）

---

## コードレビュー指摘

実装時の発見:
- `bind _ <- check_scheme_var_ambiguity(...)` は `_` がワイルドカード扱いとなり Result の短絡が効かない（E0005 が漏れた）
- `Result.and_then(check_scheme_var_ambiguity(...), |_ambiguity_ok| ...)` に変更して解決
- spec.md の `bind _ <-` パターンは Favnir では使えないことが判明 → 今後は `Result.and_then` ネストを使うこと

コードレビュー対応:
- [BUG] `is_type_var(pt)` → `is_type_var_extended(pt)` に変更（T0/A9 等の2文字型変数を検出できなかった）
- [BUG][コメントのみ] 引数長不一致ケースへの到達は E0008 ガードにより実運用上なし（コード変更不要）
- [STYLE] `bind`/`Result.and_then` 混在はコミットブロッカーではないため対応なし
