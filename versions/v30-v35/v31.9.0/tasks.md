# v31.9.0 — タスクリスト

**ステータス**: COMPLETE

---

## 前提確認（T0）

- [x] `fav/Cargo.toml` の version が `31.8.0` であること
- [x] `cargo test 2>&1 | grep "test result"` を実行して通過件数（2449 passed 想定）を実測確認すること
- [x] `driver.rs` に `mod v319000_tests` が存在しないこと
- [x] v31.8.0 が COMPLETE であること
- [x] `driver.rs:12020` の `add_history` が空行チェックなしであること（追加対象）
- [x] `check_all_files`（driver.rs:4153〜）が `files.is_empty()` チェックなしであること（追加対象）
- [x] struct 名が `ReplSession`（`ReplState` ではない）であることを driver.rs:11996 で確認すること

---

## 実装タスク

- [x] **T1** `fav/Cargo.toml` — version を `31.8.0` → `31.9.0` に更新
- [x] **T2** `fav/src/driver.rs` — `cargo_toml_version_is_31_8_0` をスタブ化
- [x] **T3** `fav/src/driver.rs` — `add_history` に `line.trim().is_empty()` チェックを追加
- [x] **T4** `fav/src/driver.rs` — `check_all_files` に `files.is_empty() && !json` チェックを追加
- [x] **T5** `fav/src/driver.rs` — `v319000_tests`（3 件）を追加（`use super::*` あり、`ReplSession` 使用）
- [x] **T6** `CHANGELOG.md` — `[v31.9.0]` セクションを先頭に追記
- [x] **T7** `benchmarks/v31.9.0.json` — 新規作成（実測値 2452）
- [x] **T8** `versions/current.md` — 「最新安定版」欄を v31.9.0 に更新

---

## テスト確認

- [x] **T9** `cargo test --bin fav v319000 2>&1 | tail -8` — 3/3 PASS
- [x] **T10** 既存 REPL テストが引き続き PASS であること
- [x] **T11** `cargo test 2>&1 | grep "test result"` — 全件 PASS（2452 passed、0 failures）

---

## 完了処理

- [x] **T12** `benchmarks/v31.9.0.json` の `tests_passed` を実測値で更新（2452 — 暫定値と一致）
- [x] **T13** このファイル（tasks.md）を COMPLETE に更新（全チェックボックス `[x]`）

---

## 完了条件チェックリスト（spec.md 対応）

- [x] `Cargo.toml` version = `"31.9.0"`
- [x] `cargo_toml_version_is_31_8_0` が空スタブになっていること
- [x] `ReplSession::add_history` が空行・空白行をスキップする
- [x] `check_all_files` が非 JSON モードでファイルゼロ件のとき警告を出力する
- [x] `cargo test --bin fav v319000` — 3/3 PASS
- [x] `cargo test` — 全件 PASS（2452 passed、0 failures）
- [x] `CHANGELOG.md` に `[v31.9.0]` セクション
- [x] `benchmarks/v31.9.0.json` 存在かつ `tests_passed` が実測値（2452）
- [x] `versions/current.md` を v31.9.0 に更新
- [x] `tasks.md` が COMPLETE
- [x] site/ MDX 更新: 対象外（バグ修正パッチのみ）

---

## コードレビューチェックリスト

- [x] `v319000_tests` に `use super::*` があること（3 件）
- [x] `cargo_toml_version_is_31_8_0` が空スタブになっていること（コメント付き）
- [x] テスト 3 が `ReplSession::new()` を使っていること（`ReplState` ではない）
- [x] `add_history` の早期 return が `line.trim().is_empty()` 条件であること
- [x] `check_all_files` の空チェックが `!json` 条件付きであること（JSON モードは変更なし）
- [x] 空チェックメッセージが `eprintln!` であること（`println!` ではなく stderr）
- [x] `check_all_files` の JSON モード（空配列出力）に変更がないこと
- [x] CHANGELOG.md の日付が正しいこと（2026-07-03）
