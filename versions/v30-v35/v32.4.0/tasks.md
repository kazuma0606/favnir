# v32.4.0 — タスクリスト

**ステータス**: COMPLETE

---

## 前提確認（T0）

- [x] `fav/Cargo.toml` の version が `32.3.0` であること
- [x] `benchmarks/v32.3.0.json` の `tests_passed` が 2468 であることを確認
- [x] `cargo test 2>&1 | grep "test result"` を実行して通過件数（2468 passed 想定）を実測確認すること
- [x] `driver.rs` に `mod v324000_tests` が存在しないこと
- [x] v32.3.0 が COMPLETE であること
- [x] `TypeExpr::Schema(String, Span)` が ast.rs:157 付近に存在すること
- [x] `register_schema_types` が checker.rs:7732 付近に存在すること
- [x] `cargo_toml_version_is_32_3_0` が v323000_tests 内に存在すること（スタブ化対象）
- [x] `cargo test --bin fav v323000` が 4/4 PASS であること（前バージョンテスト動作確認）
- [x] `site/content/docs/` 以下にスキーマ型関連 MDX（例: `schema-types.mdx` 等）が存在することを確認（存在しない場合は別タスクで対応、v32.4.0 は対象外とする）

---

## 実装タスク

- [x] **T1** `fav/Cargo.toml` — version を `32.3.0` → `32.4.0` に更新
- [x] **T2** `fav/src/driver.rs` — `cargo_toml_version_is_32_3_0` をスタブ化
- [x] **T3** `fav/src/driver.rs` — `v324000_tests`（4 件）を追加
       挿入位置: `v323000_tests` 直後・`// ── v31.7.0 tests` の前
       `use super::*` なし、`use crate::frontend::parser::Parser; use crate::ast::{Item, TypeBody, TypeExpr};`
       ※ `check_errors` は不要（パーサー中心テスト）
- [x] **T4** `CHANGELOG.md` — `[v32.4.0]` セクションを先頭に追記
- [x] **T5** `benchmarks/v32.4.0.json` — 新規作成（暫定値 2472、実測後に確定）
- [x] **T6** `versions/current.md` — 「最新安定版」欄を v32.4.0 に更新

---

## テスト確認

- [x] **T7** `cargo test --bin fav v324000 2>&1 | tail -8` — 4/4 PASS
- [x] **T8** `cargo test 2>&1 | grep "test result"` — 全件 PASS（2472 passed、0 failures）

---

## 完了処理

- [x] **T9** `benchmarks/v32.4.0.json` の `tests_passed` を実測値で更新（2472 — 暫定値と一致）
- [x] **T10** このファイル（tasks.md）を COMPLETE に更新（全チェックボックス `[x]`）

---

## 完了条件チェックリスト（spec.md 対応）

- [x] `Cargo.toml` version = `"32.4.0"`
- [x] `cargo_toml_version_is_32_3_0` が空スタブになっていること
- [x] `schema_alias_parses` テストが PASS
- [x] `schema_type_ast_is_schema_expr` テストが PASS
- [x] `cargo test --bin fav v324000` — 4/4 PASS
- [x] `cargo test` — 全件 PASS（0 failures）
- [x] `CHANGELOG.md` に `[v32.4.0]` セクション
- [x] `benchmarks/v32.4.0.json` 存在かつ `tests_passed` が実測値
- [x] `benchmarks/v32.4.0.json` の `milestone` フィールドが `"Language Power"` であること
- [x] `versions/current.md` を v32.4.0 に更新
- [x] `tasks.md` が COMPLETE

---

## コードレビューチェックリスト

- [x] `v324000_tests` に `use super::*` が**ない**こと
- [x] `use crate::frontend::parser::Parser; use crate::ast::{Item, TypeBody, TypeExpr};` が使用されていること
- [x] `check_errors` が定義されていない（パーサー中心テストのため不要）
- [x] `cargo_toml_version_is_32_3_0` が空スタブになっていること（コメント付き）
- [x] テスト 3 が `schema "file:..."` 構文のパース成功を検証していること
- [x] テスト 4 が `TypeBody::Alias(TypeExpr::Schema(..))` の AST 構造を検証していること
- [x] 挿入位置が `v323000_tests` 直後・`// ── v31.7.0 tests` の前であること
- [x] CHANGELOG.md の日付が正しいこと（2026-07-03）
- [x] `benchmarks/v32.4.0.json` の `milestone` が `"Language Power"` であること
- [x] `versions/current.md` が v32.4.0 に更新されていること
