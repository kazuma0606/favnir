# v31.4.0 — タスクリスト

**ステータス**: COMPLETE

---

## 前提確認（T0）

- [x] `fav/Cargo.toml` の version が `31.3.0` であること
- [x] `cargo test 2>&1 | grep "test result"` を実行して通過件数（2433 passed 想定）を実測確認すること
- [x] `driver.rs` に `mod v314000_tests` が存在しないこと
- [x] v31.3.0 が COMPLETE であること
- [x] `cmd_repl()` のプロンプトが `"> "` のままであること（変更対象）
- [x] `add_history()` に上限処理がないこと（追加対象）
- [x] `repl_complete_with_defs()` が存在しないこと（追加対象）

---

## 実装タスク

- [x] **T1** `fav/Cargo.toml` — version を `31.3.0` → `31.4.0` に更新
- [x] **T2** `fav/src/driver.rs` — `cargo_toml_version_is_31_3_0` をスタブ化
- [x] **T3** `fav/src/driver.rs` — `cmd_repl()` のプロンプトを `"favnir> "` に変更
- [x] **T4** `fav/src/driver.rs` — `add_history()` に 100 件上限を追加
- [x] **T5** `fav/src/driver.rs` — `repl_complete_with_defs()` を追加（`repl_complete_prefix()` 直後）
- [x] **T6** `fav/src/driver.rs` — `v314000_tests`（4 件）を追加（`use super::*` あり）
- [x] **T7** `CHANGELOG.md` — `[v31.4.0]` セクションを先頭に追記
- [x] **T8** `benchmarks/v31.4.0.json` — 新規作成
- [x] **T9** `versions/current.md` — 「最新安定版」欄を v31.4.0 に更新

---

## テスト確認

- [x] **T10** `cargo test --bin fav v314000 2>&1 | tail -8` — 4/4 PASS
- [x] **T11** `cargo test 2>&1 | grep "test result"` — 全件 PASS（2437 passed、0 failures）

---

## 完了処理

- [x] **T12** `benchmarks/v31.4.0.json` の `tests_passed` を実測値で更新（2437 — 暫定値と一致）
- [x] **T13** このファイル（tasks.md）を COMPLETE に更新（全チェックボックス `[x]`）

---

## 完了条件チェックリスト（spec.md 対応）

- [x] `Cargo.toml` version = `"31.4.0"`
- [x] REPL プロンプトが `favnir> ` になっている
- [x] `add_history()` が 100 件を超えたとき先頭エントリを削除する
- [x] `repl_complete_with_defs("List.", &[])` が `"List.map"` を含む（BUILTIN_DOCS 委譲パスの検証）
- [x] `repl_complete_with_defs("my", &["my_fn".to_string()])` が `"my_fn"` を含み `"other_fn"` を含まない
- [x] `cargo test v314000` — 4/4 PASS
- [x] `cargo test` — 全件 PASS（0 failures）
- [x] `CHANGELOG.md` に `[v31.4.0]` セクション
- [x] `benchmarks/v31.4.0.json` 存在
- [x] `benchmarks/v31.4.0.json` の `tests_passed` が実測値で更新されていること（2437）
- [x] `versions/current.md` を v31.4.0 に更新
- [x] tasks.md が COMPLETE

---

## コードレビューチェックリスト

- [x] `v314000_tests` に `use super::*` があること（4 件）
- [x] `cargo_toml_version_is_31_3_0` が空スタブになっていること（コメント付き）
- [x] `cmd_repl()` のプロンプトが `"favnir> "` になっていること（`"> "` でないこと）
- [x] `add_history()` の上限が 100 件であること（`> 100` の条件で先頭を削除）
- [x] `repl_complete_with_defs()` が `repl_complete_prefix()` を内部で呼んでいること
- [x] `repl_complete_with_defs()` が重複エントリを追加しないこと（`!result.contains(name)` のチェック）
- [x] `repl_complete_with_defs()` が結果をソートして返すこと
- [x] 既存の `repl_complete_prefix()` シグネチャが変更されていないこと
- [x] site/ MDX が変更されていないこと（OUT OF SCOPE）
