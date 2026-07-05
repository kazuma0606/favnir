# v32.8.0 — タスクリスト

**ステータス**: COMPLETE

---

## 前提確認（T0）

- [x] `fav/Cargo.toml` の version が `32.7.0` であること
- [x] `benchmarks/v32.7.0.json` の `tests_passed` が 2484 であることを確認
- [x] `cargo test 2>&1 | grep "test result"` を実行して通過件数（2484 passed 想定）を実測確認すること
- [x] `driver.rs` に `mod v328000_tests` が存在しないこと
- [x] v32.7.0 が COMPLETE であること
- [x] `ApiAnnotation` struct が ast.rs に存在すること（v18.8.0 実装済み）
- [x] `collect_api_fns` / `build_openapi_json` / `build_route_table` / `match_route` が driver.rs に存在すること
- [x] `cargo_toml_version_is_32_7_0` が v327000_tests 内に存在すること（スタブ化対象）
- [x] `cargo test --bin fav v327000` が 4/4 PASS であること（前バージョンテスト動作確認）
- [x] `cargo test --bin fav v188000` が PASS であること（`api_annotation_parses`・`openapi_generates`・`graphql_generates`・`serve_routes_request` 4 件アクティブ PASS を確認）
- [x] テスト名が v188000_tests（`api_annotation_parses` / `openapi_generates` / `graphql_generates` / `serve_routes_request`）と重複しないこと

---

## 実装タスク

- [x] **T1** `fav/Cargo.toml` — version を `32.7.0` → `32.8.0` に更新
- [x] **T2** `fav/src/driver.rs` — `cargo_toml_version_is_32_7_0` をスタブ化
- [x] **T3** `fav/src/driver.rs` — `v328000_tests`（4 件）を追加
       挿入位置: `v327000_tests` 直後・`// ── v31.7.0 tests` の前
       `use super::*` なし、`use crate::frontend::parser::Parser` を使用
       ルートテーブル・OpenAPI は `super::collect_api_fns` / `super::build_openapi_json` で呼び出す
- [x] **T4** `CHANGELOG.md` — `[v32.8.0]` セクションを先頭に追記
- [x] **T5** `benchmarks/v32.8.0.json` — 新規作成（暫定値 2488、実測後に確定）
- [x] **T6** `versions/current.md` — 「最新安定版」欄を v32.8.0 に更新

---

## テスト確認

- [x] **T7** `cargo test --bin fav v328000 2>&1 | tail -8` — 4/4 PASS
- [x] **T8** `cargo test 2>&1 | grep "test result"` — 全件 PASS（2488 passed、0 failures）

---

## 完了処理

- [x] **T9** `benchmarks/v32.8.0.json` の `tests_passed` を実測値で更新（2488 — 暫定値と一致）
- [x] **T10** このファイル（tasks.md）を COMPLETE に更新（全チェックボックス `[x]`）

---

## 完了条件チェックリスト（spec.md 対応）

- [x] `Cargo.toml` version = `"32.8.0"`
- [x] `cargo_toml_version_is_32_7_0` が空スタブになっていること
- [x] `api_ann_get_items_path_parses` テストが PASS
- [x] `api_ann_openapi_items_path_exists` テストが PASS
- [x] `cargo test --bin fav v328000` — 4/4 PASS
- [x] `cargo test` — 全件 PASS（2488 件、0 failures）
- [x] `CHANGELOG.md` に `[v32.8.0]` セクション
- [x] `benchmarks/v32.8.0.json` 存在かつ `tests_passed` が実測値
- [x] `benchmarks/v32.8.0.json` の `milestone` フィールドが `"Language Power"` であること
- [x] `versions/current.md` を v32.8.0 に更新
- [x] `tasks.md` が COMPLETE
- [x] site/ MDX 更新: 対象外（型駆動 API 生成は v18.8.0 で完成済み）

---

## コードレビューチェックリスト

- [x] `v328000_tests` に `use super::*` が**ない**こと（`use crate::...` + `super::` 関数呼び出しで完結）
- [x] `cargo_toml_version_is_32_7_0` が空スタブになっていること（コメント付き）
- [x] `api_ann_get_items_path_parses` が v188000_tests のテスト名と異なること
- [x] `api_ann_openapi_items_path_exists` が v188000_tests のテスト名と異なること
- [x] テスト 3 が `/items/:id` パスのパースを検証していること（v188000_tests は `/users/:id`）
- [x] テスト 4 が OpenAPI paths に `/items/{id}` が含まれることを検証していること
- [x] 挿入位置が `v327000_tests` 直後・`// ── v31.7.0 tests` の前であること
- [x] CHANGELOG.md の日付が正しいこと（2026-07-03）
- [x] `benchmarks/v32.8.0.json` の `milestone` が `"Language Power"` であること
- [x] `versions/current.md` が v32.8.0 に更新されていること
