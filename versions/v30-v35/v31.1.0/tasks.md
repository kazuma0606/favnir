# v31.1.0 — タスクリスト

**ステータス**: COMPLETE

---

## 前提確認（T0）

- [x] `fav/Cargo.toml` の version が `31.0.0` であること
- [x] `cargo test 2>&1 | grep "test result"` を実行して通過件数（2422 passed 想定）を実測確認すること
- [x] `driver.rs` に `mod v311000_tests` が存在しないこと
- [x] v31.0.0 が COMPLETE であること
- [x] `driver.rs::get_help_text()` に E0002/E0003/E0004/E0005/E0006/E0010 が存在しないこと（追加対象）

---

## 実装タスク

- [x] **T1** `fav/Cargo.toml` — version を `31.0.0` → `31.1.0` に更新
- [x] **T2** `fav/src/driver.rs` — `cargo_toml_version_is_31_0_0` をスタブ化
- [x] **T3** `fav/src/driver.rs` — `get_help_text()` に E0002/E0003/E0004/E0005/E0006/E0010 の hint を追加
- [x] **T4** `fav/src/driver.rs` — `v311000_tests`（4 件）を追加（`use super::*` あり）
- [x] **T5** `CHANGELOG.md` — `[v31.1.0]` セクションを先頭に追記
- [x] **T6** `benchmarks/v31.1.0.json` — 新規作成
- [x] **T7** `versions/current.md` — 「最新安定版」欄を v31.1.0 に更新

---

## テスト確認

- [x] **T8** `cargo test --bin fav v311000 2>&1 | tail -8` — 4/4 PASS
- [x] **T9** `cargo test 2>&1 | grep "test result"` — 全件 PASS（2426 passed、0 failures）

---

## 完了処理

- [x] **T10** `benchmarks/v31.1.0.json` の `tests_passed` を実測値で更新（2426 — 初期値と一致）
- [x] **T11** このファイル（tasks.md）を COMPLETE に更新（全チェックボックス `[x]`）

---

## 完了条件チェックリスト（spec.md 対応）

- [x] `Cargo.toml` version = `"31.1.0"`
- [x] `get_help_text("E0002")` 〜 `get_help_text("E0010")` が非空スライスを返す（既存 E0001/E0007/E0008/E0009 含む全コード）
- [x] `cargo test v311000` — 4/4 PASS
- [x] `cargo test` — 全件 PASS（0 failures）
- [x] `CHANGELOG.md` に `[v31.1.0]` セクション
- [x] `benchmarks/v31.1.0.json` 存在
- [x] `benchmarks/v31.1.0.json` の `tests_passed` が実測値で更新されていること（2426）
- [x] `versions/current.md` を v31.1.0 に更新
- [x] tasks.md が COMPLETE

---

## コードレビューチェックリスト

- [x] `v311000_tests` に `use super::*` があること（`get_help_text` は非 pub 関数のため必要）
- [x] `cargo_toml_version_is_31_0_0` が空スタブになっていること（コメント付き）
- [x] `get_help_text("E0002")` が非空スライスを返すこと
- [x] `get_help_text("E0003")` が非空スライスを返すこと
- [x] `get_help_text("E0004")` が非空スライスを返すこと
- [x] `get_help_text("E0005")` が非空スライスを返すこと
- [x] `get_help_text("E0006")` が非空スライスを返すこと
- [x] `get_help_text("E0010")` が非空スライスを返すこと
- [x] 既存の `get_help_text()` アーム（E0001/E0007/E0008/E0009 等）が変更されていないこと
- [x] `format_diagnostic()` に変更がないこと（実装済みのため）
