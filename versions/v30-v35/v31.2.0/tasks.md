# v31.2.0 — タスクリスト

**ステータス**: COMPLETE

---

## 前提確認（T0）

- [x] `fav/Cargo.toml` の version が `31.1.0` であること
- [x] `cargo test 2>&1 | grep "test result"` を実行して通過件数（2426 passed 想定）を実測確認すること
- [x] `driver.rs` に `mod v312000_tests` が存在しないこと
- [x] v31.1.0 が COMPLETE であること
- [x] `driver.rs` に `levenshtein()` / `suggest_similar()` 関数が存在しないこと（追加対象）
- [x] `get_help_text()` に E0011/E0012/E0016/E0017/E0019 が存在しないこと（追加対象）

---

## 実装タスク

- [x] **T1** `fav/Cargo.toml` — version を `31.1.0` → `31.2.0` に更新
- [x] **T2** `fav/src/driver.rs` — `cargo_toml_version_is_31_1_0` をスタブ化
- [x] **T3** `fav/src/driver.rs` — `levenshtein()` 関数を追加（`get_help_text()` 直前）
- [x] **T4** `fav/src/driver.rs` — `suggest_similar()` 関数を追加（`levenshtein()` 直後）
- [x] **T5** `fav/src/driver.rs` — `get_help_text()` に E0011/E0012/E0016/E0017/E0019 を追加
- [x] **T6** `fav/src/driver.rs` — `v312000_tests`（4 件）を追加（`use super::*` あり）
- [x] **T7** `CHANGELOG.md` — `[v31.2.0]` セクションを先頭に追記
- [x] **T8** `benchmarks/v31.2.0.json` — 新規作成
- [x] **T9** `versions/current.md` — 「最新安定版」欄を v31.2.0 に更新

---

## テスト確認

- [x] **T10** `cargo test --bin fav v312000 2>&1 | tail -8` — 4/4 PASS
- [x] **T11** `cargo test 2>&1 | grep "test result"` — 全件 PASS（2430 passed、0 failures）

---

## 完了処理

- [x] **T12** `benchmarks/v31.2.0.json` の `tests_passed` を実測値で更新（2430 — 初期値と一致）
- [x] **T13** このファイル（tasks.md）を COMPLETE に更新（全チェックボックス `[x]`）

---

## 完了条件チェックリスト（spec.md 対応）

- [x] `Cargo.toml` version = `"31.2.0"`
- [x] `levenshtein()` と `suggest_similar()` が `driver.rs` に追加されている
- [x] `get_help_text("E0011")` が非空スライスを返す
- [x] `get_help_text("E0012")` が非空スライスを返す
- [x] `get_help_text("E0016")` が非空スライスを返す
- [x] `get_help_text("E0017")` が非空スライスを返す
- [x] `get_help_text("E0019")` が非空スライスを返す
- [x] `suggest_similar("user_id", ...)` が `order_id`（距離 3）を除外すること
- [x] `cargo test v312000` — 4/4 PASS
- [x] `cargo test` — 全件 PASS（0 failures）
- [x] `CHANGELOG.md` に `[v31.2.0]` セクション
- [x] `benchmarks/v31.2.0.json` 存在
- [x] `benchmarks/v31.2.0.json` の `tests_passed` が実測値で更新されていること（2430）
- [x] `versions/current.md` を v31.2.0 に更新
- [x] tasks.md が COMPLETE

---

## コードレビューチェックリスト

- [x] `v312000_tests` に `use super::*` があること
- [x] `cargo_toml_version_is_31_1_0` が空スタブになっていること（コメント付き）
- [x] `levenshtein("kitten", "sitting") == 3` が正しく動作すること
- [x] `levenshtein("abc", "abc") == 0` が正しく動作すること
- [x] `suggest_similar()` の返却件数が最大 3 件であること（`truncate(3)`）
- [x] `suggest_similar()` が距離 > 2 の候補を除外すること
- [x] 既存の `get_help_text()` アームが変更されていないこと
- [x] `levenshtein()` / `suggest_similar()` が `#[cfg(test)]` 外に配置されていること（ユーティリティ関数）
- [x] checker.fav への統合がないこと（OUT OF SCOPE）
- [x] site/ MDX が変更されていないこと（OUT OF SCOPE）
- [x] `suggest_similar()` が距離順ソートを行っていないこと（現バージョンは入力順）
