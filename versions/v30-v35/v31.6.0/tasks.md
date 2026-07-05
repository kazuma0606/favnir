# v31.6.0 — タスクリスト

**ステータス**: COMPLETE

---

## 前提確認（T0）

- [x] `fav/Cargo.toml` の version が `31.5.0` であること
- [x] `cargo test 2>&1 | grep "test result"` を実行して通過件数（2440 passed 想定）を実測確認すること
- [x] `driver.rs` に `mod v316000_tests` が存在しないこと
- [x] v31.5.0 が COMPLETE であること
- [x] `main.rs` の `Some("test")` パーサに `--watch` フラグが存在しないこと（追加対象）
- [x] `cmd_watch` が `"test"` を有効な cmd 値としてサポートしていること（実装済み確認）

---

## 実装タスク

- [x] **T1** `fav/Cargo.toml` — version を `31.5.0` → `31.6.0` に更新
- [x] **T2** `fav/src/driver.rs` — `cargo_toml_version_is_31_5_0` をスタブ化
- [x] **T3** `fav/src/main.rs` — `watch_mode` / `watch_dirs` 変数宣言を追加
- [x] **T4** `fav/src/main.rs` — `"--watch"` アームを `match args[i].as_str()` ブロックに追加
- [x] **T5** `fav/src/main.rs` — `watch_mode` チェック + `cmd_watch(file_for_watch, "test", &dir_refs, 80)` 呼び出しを追加（`cmd_test` より前）
- [x] **T6** `fav/src/driver.rs` — `v316000_tests`（3 件）を追加（`use super::*` あり）
- [x] **T7** `CHANGELOG.md` — `[v31.6.0]` セクションを先頭に追記
- [x] **T8** `benchmarks/v31.6.0.json` — 新規作成（暫定値 2443、**T12 で必ず実測値に更新すること**）
- [x] **T9** `versions/current.md` — 「最新安定版」欄を v31.6.0 に更新

---

## テスト確認

- [x] **T10** `cargo test --bin fav v316000 2>&1 | tail -8` — 3/3 PASS
- [x] **T11** `cargo test 2>&1 | grep "test result"` — 全件 PASS（2443 passed、0 failures）

---

## 完了処理

- [x] **T12** `benchmarks/v31.6.0.json` の `tests_passed` を実測値で更新（2443 — 暫定値と一致）
- [x] **T13** このファイル（tasks.md）を COMPLETE に更新（全チェックボックス `[x]`）

---

## 完了条件チェックリスト（spec.md 対応）

- [x] `Cargo.toml` version = `"31.6.0"`
- [x] `fav test --watch src/` が `cmd_watch(_, "test", _, 80)` を呼び出す
- [x] `fav test <file>` の既存動作が変わらないこと（`--watch` なし時は `cmd_test` が呼ばれる）
- [x] `cargo test v316000` — 3/3 PASS
- [x] `cargo test` — 全件 PASS（2443 passed、0 failures）
- [x] `CHANGELOG.md` に `[v31.6.0]` セクション
- [x] `benchmarks/v31.6.0.json` 存在かつ `tests_passed` が実測値（2443）
- [x] `versions/current.md` を v31.6.0 に更新
- [x] `tasks.md` が COMPLETE

---

## コードレビューチェックリスト

- [x] `v316000_tests` に `use super::*` があること（3 件）
- [x] `cargo_toml_version_is_31_5_0` が空スタブになっていること（コメント付き）
- [x] `--watch` フラグが `cmd_test(...)` より前に処理されること
- [x] `watch_mode` が `false` のとき既存の `cmd_test` パスが変わらないこと
- [x] `file` がディレクトリのとき `file_for_watch = None` + `extra_dirs` に渡していること（`cmd_watch(Some("dir/"), ...)` は NG）
- [x] `"--watch"` アームが `i += 1` のみ（値なし）であること
- [x] `collect_watch_paths_finds_fav_files` テストが一時ディレクトリで `.fav` ファイルを収集できることを確認すること
- [x] CHANGELOG.md の日付が正しいこと（2026-07-03）
