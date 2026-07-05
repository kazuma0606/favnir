# v30.8.0 — タスクリスト

**ステータス**: COMPLETE

---

## 前提確認（T0）

- [x] `fav/Cargo.toml` の version が `30.7.0` であること
- [x] `cargo test 2>&1 | grep "test result"` が `2412 passed` を含むこと
- [x] `driver.rs` に `mod v308000_tests` が存在しないこと
- [x] `driver.rs` に `cmd_new_list` が存在しないこと
- [x] `main.rs` に `cmd_new_list` が存在しないこと
- [x] v30.7.0 が COMPLETE であること

---

## 実装タスク

- [x] **T1** `fav/Cargo.toml` — version を `30.7.0` → `30.8.0` に更新
- [x] **T2** `fav/src/driver.rs` — `cargo_toml_version_is_30_7_0` をスタブ化
- [x] **T3** `fav/src/driver.rs` — `cmd_new_list` を追加（8 テンプレート全件表示、`fn try_cmd_new` の直前）
- [x] **T4a** `fav/src/main.rs` — `use driver::{ ... cmd_new, ... }` に `cmd_new_list` を追加（87 行付近）
- [x] **T4b** `fav/src/main.rs` — 行 1256 付近の `Some("new")` ハンドラ先頭に `--list` フラグ検出を追加
  - **確認済み**: 行 2043 付近の `Some("new")` は `fav notebook new` のサブコマンドハンドラ — **変更なし**
- [x] **T5** `fav/src/driver.rs` — `v308000_tests`（3 件）を追加
  - テスト 2 は `src.contains("fn cmd_new_list")` で先にガードし、その後 8 テンプレート名をループ確認
- [x] **T6** `CHANGELOG.md` — `[v30.8.0]` セクションを先頭に追記
- [x] **T7** `benchmarks/v30.8.0.json` — 新規作成（tests_passed: 2415）
- [x] **T8** `versions/current.md` — 「最新安定版」欄を v30.8.0 に更新

---

## テスト確認

- [x] **T9** `cargo test --bin fav v308000 2>&1 | tail -8` — 3/3 PASS
- [x] **T10** `cargo test 2>&1 | grep "test result"` — 全件 PASS（2415 passed、0 failures）

---

## 完了処理

- [x] **T11** このファイル（tasks.md）を COMPLETE に更新（全チェックボックス `[x]`）

---

## 完了条件チェックリスト（spec.md 対応）

- [x] `Cargo.toml` version = `"30.8.0"`
- [x] `cmd_new_list` が `driver.rs` に実装されている（8 テンプレート全件表示）
- [x] `main.rs` の `use driver::` リストに `cmd_new_list` が追加されている
- [x] `fav new --list` が正しく一覧を表示する（`main.rs` 行 1256 の更新）
- [x] `cargo test v308000` — 3/3 PASS
- [x] `cargo test` — 全件 PASS（2415 passed）
- [x] `CHANGELOG.md` に `[v30.8.0]` セクション
- [x] `benchmarks/v30.8.0.json` 存在
- [x] `versions/current.md` を v30.8.0 に更新
- [x] tasks.md を COMPLETE に更新

---

## コードレビューチェックリスト

- [x] `cmd_new_list` が `fn try_cmd_new` の直前（`cmd_new` の直後）に配置されていること
- [x] 8 テンプレートすべてが `cmd_new_list` に含まれていること（script / pipeline / lib / postgres-etl / etl-csv-to-db / api-gateway / lambda-scheduled / distributed-etl）
- [x] `main.rs` の `--list` 判定が `fav new --list` の直後（name 取得より前）にあること
- [x] `v308000_tests` に `use super::*` がないこと（include_str! のみ使用）
- [x] `v308000_tests::cmd_new_list_contains_all_templates` が `fn cmd_new_list` の存在を先にガードしていること
- [x] `v308000_tests::cmd_new_list_contains_all_templates` が 8 テンプレート名をループで確認していること
- [x] `v308000_tests` に `benchmark_v30_8_0_exists` テストがあること
- [x] `main.rs` 行 2043 付近の `Some("new")`（`fav notebook new`）が変更されていないことを確認

---

## コードレビュー指摘・対応記録

spec-reviewer 指摘 7 件をすべて spec/plan/tasks に反映:
- [HIGH] main.rs の `use driver::` への `cmd_new_list` 追加を T4a として明記
- [HIGH] 行 2043 `Some("new")` が `fav notebook new` であり変更不要であることを明記
- [HIGH] テスト 2 に `fn cmd_new_list` 存在ガードを追加
- [MED] ロードマップ「テスト 1 件」vs spec「3 件」の差異を spec に説明追記
- [MED] benchmark `tests_passed` を `cargo test` 実行後の実数で更新する旨を plan に注記
- [LOW] `TEMPLATE_GALLERY` との乖離リスクを spec OUT OF SCOPE に明記
- [LOW] MDX 除外を明記
