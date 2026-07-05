# v31.8.0 — タスクリスト

**ステータス**: COMPLETE

---

## 前提確認（T0）

- [x] `fav/Cargo.toml` の version が `31.7.0` であること
- [x] `cargo test 2>&1 | grep "test result"` を実行して通過件数（2446 passed 想定）を実測確認すること
- [x] `driver.rs` に `mod v318000_tests` が存在しないこと
- [x] v31.7.0 が COMPLETE であること
- [x] `driver.rs` に `scaffold_to_src` が存在しないこと（追加対象）
- [x] `cmd_scaffold` の `stage` / `seq` アームが `out_file` のみを参照し、fav.toml auto-write が未実装であること（確認）

---

## 実装タスク

- [x] **T1** `fav/Cargo.toml` — version を `31.7.0` → `31.8.0` に更新
- [x] **T2** `fav/src/driver.rs` — `cargo_toml_version_is_31_7_0` をスタブ化
- [x] **T3** `fav/src/driver.rs` — `pub(crate) fn scaffold_to_src(root, sub, name, content)` を `write_scaffold` 直後に追加
- [x] **T4** `fav/src/driver.rs` — `cmd_scaffold` の `stage` アームを修正（`out_file` None 時に `scaffold_to_src` 呼び出し）
- [x] **T5** `fav/src/driver.rs` — `cmd_scaffold` の `seq` アームを修正（同上）
- [x] **T6** `fav/src/driver.rs` — `v318000_tests`（3 件）を追加（`use super::*` あり）
- [x] **T7** `CHANGELOG.md` — `[v31.8.0]` セクションを先頭に追記
- [x] **T8** `benchmarks/v31.8.0.json` — 新規作成（実測値 2449）
- [x] **T9** `versions/current.md` — 「最新安定版」欄を v31.8.0 に更新

---

## テスト確認

- [x] **T10** `cargo test --bin fav v318000 2>&1 | tail -8` — 3/3 PASS
- [x] **T11** 既存 scaffold テストが引き続き PASS であること — 9/9 PASS
- [x] **T12** `cargo test 2>&1 | grep "test result"` — 全件 PASS（2449 passed、0 failures）

---

## 完了処理

- [x] **T13** `benchmarks/v31.8.0.json` の `tests_passed` を実測値で更新（2449 — 暫定値と一致）
- [x] **T14** このファイル（tasks.md）を COMPLETE に更新（全チェックボックス `[x]`）

---

## 完了条件チェックリスト（spec.md 対応）

- [x] `Cargo.toml` version = `"31.8.0"`
- [x] `fav scaffold stage <Name>` がプロジェクト内で `src/stages.fav` にコードを追記する
- [x] `fav scaffold seq <Name>` がプロジェクト内で `src/pipelines.fav` にコードを追記する
- [x] `fav scaffold stage <Name> --out <file>` の既存動作が変わらないこと
- [x] fav.toml なし環境では従来通り stdout 出力されること
- [x] `cargo test v318000` — 3/3 PASS
- [x] `cargo test` — 全件 PASS（2449 passed、0 failures）
- [x] `CHANGELOG.md` に `[v31.8.0]` セクション
- [x] `benchmarks/v31.8.0.json` 存在かつ `tests_passed` が実測値（2449）
- [x] `versions/current.md` を v31.8.0 に更新
- [x] `tasks.md` が COMPLETE

---

## コードレビューチェックリスト

- [x] `v318000_tests` に `use super::*` があること（3 件）
- [x] `cargo_toml_version_is_31_7_0` が空スタブになっていること（コメント付き）
- [x] `scaffold_to_src` が `pub(crate)` であること（テストから直接呼び出し可能）
- [x] `FavToml::load(root)` が `Option<FavToml>` を返すため `.ok()` は不使用、`unwrap_or_else(||...)` を使用
- [x] `scaffold_to_src` の `filename` が `stage` → `stages.fav`、`seq` → `pipelines.fav` であること
- [x] ファイルが既存の場合は追記（`.append(true)`）、新規の場合は作成（`.create(true)`）
- [x] `rune` / `postgres-etl` アームが変更されていないこと
- [x] `--out` 指定時は `write_scaffold` を従来通り呼ぶこと（後方互換）
- [x] fav.toml が見つからない場合（`find_root` が `None`）は stdout フォールバックすること
- [x] テスト fav.toml が `[project]` セクションを使用していること（`[package]` では src が無視される）
- [x] `path.ends_with("src/stages.fav")` アサートが含まれること
- [x] CHANGELOG.md の日付が正しいこと（2026-07-03）
