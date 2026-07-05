# v32.0.0 — タスクリスト

**ステータス**: COMPLETE

---

## 前提確認（T0）

- [x] `fav/Cargo.toml` の version が `31.9.0` であること
- [x] `cargo test 2>&1 | grep "test result"` を実行して通過件数（2452 passed 想定）を実測確認すること
- [x] `driver.rs` に `mod v320000_tests` が存在しないこと
- [x] v31.9.0 が COMPLETE であること
- [x] `MILESTONE.md` に `"Language Polish"` が存在しないこと（追加対象）
- [x] `README.md` に `"v32.0"` が存在しないこと（追加対象）
- [x] v31.1〜v31.9 が全て COMPLETE であること（マイルストーン宣言の前提）

---

## 実装タスク

- [x] **T1** `fav/Cargo.toml` — version を `31.9.0` → `32.0.0` に更新
- [x] **T2** `fav/src/driver.rs` — `cargo_toml_version_is_31_9_0` をスタブ化
- [x] **T3** `fav/src/driver.rs` — `v320000_tests`（4 件）を追加（`use super::*` **なし**）
- [x] **T4** `MILESTONE.md` — `v32.0.0 — Language Polish` セクションを先頭に追記
- [x] **T5** `README.md` — v32.0 マイルストーン行を v31.0 行の直後に追加
- [x] **T6** `CHANGELOG.md` — `[v32.0.0]` セクションを先頭に追記
- [x] **T7** `benchmarks/v32.0.0.json` — 新規作成（実測値 2456）
- [x] **T8** `versions/current.md` — 「最新安定版」欄を v32.0.0 に更新、マイルストーンテーブルの `v32.0 — Language Polish` 行を `planned` → `**完了**` に変更

---

## テスト確認

- [x] **T9** `cargo test --bin fav v320000 2>&1 | tail -8` — 4/4 PASS
- [x] **T10** `cargo test 2>&1 | grep "test result"` — 全件 PASS（暫定 2456、実測値で確認、0 failures）

---

## cargo clean クリーンアップ（マイルストーン版必須）

- [x] **T11** `cargo clean` を実行（21.8GiB 削除）
- [x] **T12** `fav/tmp/hello.fav` 確認 — cargo clean 後も残存（削除されなかった）
- [x] **T13** `cargo build` が成功すること（fav v32.0.0 ビルド完了）
- [x] **T14** `cargo test 2>&1 | grep "test result"` — cargo clean 後も全件 PASS（0 failures）

---

## 完了処理

- [x] **T15** `benchmarks/v32.0.0.json` の `tests_passed` を実測値で更新（2456 — 暫定値と一致）
- [x] **T16** このファイル（tasks.md）を COMPLETE に更新（全チェックボックス `[x]`）

---

## 完了条件チェックリスト（spec.md 対応）

- [x] `Cargo.toml` version = `"32.0.0"`
- [x] `MILESTONE.md` に `"Language Polish"` セクションが存在すること
- [x] `README.md` に `"v32.0"` の記述があること
- [x] `cargo test --bin fav v320000` — 4/4 PASS
- [x] `cargo test`（`cargo clean` 後）— 全件 PASS（実測値 2456、0 failures）
- [x] `CHANGELOG.md` に `[v32.0.0]` セクション
- [x] `benchmarks/v32.0.0.json` 存在かつ `tests_passed` が実測値（2456）
- [x] `versions/current.md` を v32.0.0 に更新
- [x] `tasks.md` が COMPLETE

---

## コードレビューチェックリスト

- [x] `v320000_tests` に `use super::*` が**ない**こと（include_str! のみ使用）
- [x] `cargo_toml_version_is_31_9_0` が空スタブになっていること（コメント付き）
- [x] `MILESTONE.md` の宣言日が正しいこと（2026-07-03）
- [x] `MILESTONE.md` の達成コンポーネント一覧が v31.1〜v31.9 の全 9 件を網羅すること
- [x] `README.md` の追加行が v31.0 行の直後に配置されること
- [x] `benchmarks/v32.0.0.json` の `milestone` フィールドが `"Language Polish"` であること
- [x] `versions/current.md` のマイルストーンテーブルで v32.0 が `**完了**` になること
- [x] CHANGELOG.md の日付が正しいこと（2026-07-03）
