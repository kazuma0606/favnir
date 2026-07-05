# v31.7.0 — タスクリスト

**ステータス**: COMPLETE

---

## 前提確認（T0）

- [x] `fav/Cargo.toml` の version が `31.6.0` であること
- [x] `cargo test 2>&1 | grep "test result"` を実行して通過件数（2443 passed 想定）を実測確認すること
- [x] `driver.rs` に `mod v317000_tests` が存在しないこと
- [x] v31.6.0 が COMPLETE であること
- [x] `main.rs` の `Some("check")` パーサに `--all` フラグが存在しないこと（追加対象）
- [x] `driver.rs` に `cmd_check_all` および `check_all_files` が存在しないこと（追加対象）

---

## 実装タスク

- [x] **T1** `fav/Cargo.toml` — version を `31.6.0` → `31.7.0` に更新
- [x] **T2** `fav/src/driver.rs` — `cargo_toml_version_is_31_6_0` をスタブ化
- [x] **T3** `fav/src/driver.rs` — `pub(crate) fn check_all_files(dir, json)` を追加（`cmd_check_dir` 直後）
- [x] **T4** `fav/src/driver.rs` — `pub fn cmd_check_all(json)` を追加（`check_all_files` の直後）
- [x] **T5** `fav/src/main.rs` — `all_mode` 変数宣言を追加
- [x] **T6** `fav/src/main.rs` — `"--all"` アームを `match args[i].as_str()` ブロックに追加
- [x] **T7** `fav/src/main.rs` — `} else if all_mode { driver::cmd_check_all(json); }` をディスパッチに追加
- [x] **T8** `fav/src/driver.rs` — `v317000_tests`（3 件）を追加（`use super::*` あり）
- [x] **T9** `CHANGELOG.md` — `[v31.7.0]` セクションを先頭に追記
- [x] **T10** `benchmarks/v31.7.0.json` — 新規作成（実測値 2446）
- [x] **T11** `versions/current.md` — 「最新安定版」欄を v31.7.0 に更新

---

## テスト確認

- [x] **T12** `cargo test --bin fav v317000 2>&1 | tail -8` — 3/3 PASS
- [x] **T13** `cargo test 2>&1 | grep "test result"` — 全件 PASS（2446 passed、0 failures）

---

## 完了処理

- [x] **T14** `benchmarks/v31.7.0.json` の `tests_passed` を実測値で更新（2446 — 暫定値と一致）
- [x] **T15** このファイル（tasks.md）を COMPLETE に更新（全チェックボックス `[x]`）

---

## 完了条件チェックリスト（spec.md 対応）

- [x] `Cargo.toml` version = `"31.7.0"`
- [x] `fav check --all` で fav.toml src ディレクトリを走査してチェックする
- [x] `fav check --all --json` で JSON 形式出力される（エラーのあるファイルが 1 つでも存在すれば exit 1）
- [x] `fav check <file>` 等の既存動作が変わらないこと（`--all` なし時は従来パス）
- [x] `cargo test v317000` — 3/3 PASS
- [x] `cargo test` — 全件 PASS（2446 passed、0 failures）
- [x] `CHANGELOG.md` に `[v31.7.0]` セクション
- [x] `benchmarks/v31.7.0.json` 存在かつ `tests_passed` が実測値（2446）
- [x] `versions/current.md` を v31.7.0 に更新
- [x] `tasks.md` が COMPLETE

---

## コードレビューチェックリスト

- [x] `v317000_tests` に `use super::*` があること（3 件）
- [x] `cargo_toml_version_is_31_6_0` が空スタブになっていること（コメント付き）
- [x] `check_all_files` が `pub(crate)` であること（テストから直接呼び出し可能）
- [x] `check_all_files` が `files.is_empty()` でも panic しないこと（空ディレクトリ → 0 を返す）
- [x] JSON モードで `total_errors` が `ok: false` のファイル数を正しく数えること
- [x] `--all` フラグが `--sample` / `--dir` ディスパッチより後（`else if all_mode`）に処理されること
- [x] 既存 `fav check <file>` / `fav check --dir` パスが変わらないこと
- [x] `FavToml::load` が `Option<FavToml>` を返すため `.ok()` は不使用、`unwrap_or_else(||...)` を使用
- [x] CHANGELOG.md の日付が正しいこと（2026-07-03）
