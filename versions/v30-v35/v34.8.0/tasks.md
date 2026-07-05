# v34.8.0 — タスクリスト

**ステータス**: COMPLETE

---

## 前提確認（T0）

- [x] `fav/Cargo.toml` の version が `34.7.0` であること
- [x] `benchmarks/v34.7.0.json` の `tests_passed` が 2571 であることを確認
- [x] `cargo test 2>&1 | grep "test result"` を実行して通過件数（2571 passed 想定）を実測確認すること
- [x] `driver.rs` に `mod v348000_tests` が存在しないこと
- [x] v34.7.0 が COMPLETE であること
- [x] `cargo_toml_version_is_34_7_0` が v347000_tests 内に存在すること（スタブ化対象）
- [x] `cargo test --bin fav v347000` が 5/5 PASS であること
- [x] `MIGRATION.md` が存在しないこと（新規作成対象）
- [x] `cmd_upgrade` が driver.rs に存在しないこと（新規実装対象）

---

## 実装タスク

- [x] **T1** `fav/Cargo.toml` — version を `34.7.0` → `34.8.0` に更新
- [x] **T2** `MIGRATION.md` — !Effect → Capability Context 移行ガイドを新規作成
        （背景 / fav upgrade 手順 / !Effect→ctx 対応表 / 手動移行手順 / Before-After / FAQ）
- [x] **T3** `fav/src/driver.rs` — `pub fn cmd_upgrade(args: &[&str]) -> Result<String, String>` を実装
        （--from-effects + --dry-run / --in-place、フラグなしは Err）
- [x] **T4** `fav/src/main.rs` — `Some("upgrade")` アームを `Some("migrate")` の直後に追加
- [x] **T5** `fav/src/driver.rs` — `cargo_toml_version_is_34_7_0` をスタブ化
- [x] **T6** `fav/src/driver.rs` — `v348000_tests`（5 件）を追加
        挿入位置: `v347000_tests` 直後・`// ── v31.7.0 tests` の前
        `use super::*` あり（`cmd_upgrade` を直接呼ぶため）
- [x] **T7** `CHANGELOG.md` — `[v34.8.0]` セクションを先頭に追記
- [x] **T8** `benchmarks/v34.8.0.json` — 新規作成（`tests_passed`: 2576）
- [x] **T9** `versions/current.md` — 「最新安定版」欄を v34.8.0 に更新

---

## テスト確認

- [x] **T10** `cargo test --bin fav v348000 2>&1 | tail -8` — 5/5 PASS
- [x] **T11** `cargo test 2>&1 | grep "test result"` — 全件 PASS（2576 passed、0 failures）

---

## 完了処理

- [x] **T12** `benchmarks/v34.8.0.json` の `tests_passed` を実測値（2576）で更新
- [x] **T13** このファイル（tasks.md）を COMPLETE に更新（全チェックボックス `[x]`）

---

## 完了条件チェックリスト（spec.md 対応）

- [x] `cargo clean` 不要（x.8.0 のため実施しない）
- [x] `Cargo.toml` version = `"34.8.0"`
- [x] `cargo_toml_version_is_34_7_0` が空スタブになっていること
- [x] `cargo test --bin fav v348000` — 5/5 PASS
- [x] `cargo test` — 全件 PASS（2576 件、0 failures）
- [x] `MIGRATION.md` が存在し `"AppCtx"` と `"fav upgrade"` を含むこと
- [x] `cmd_upgrade(&["--from-effects", "--dry-run"])` が `Ok` を返すこと
- [x] `cmd_upgrade(&[])` が `Err` を返すこと
- [x] `CHANGELOG.md` に `[v34.8.0]` セクション
- [x] `benchmarks/v34.8.0.json` 存在かつ `tests_passed` が実測値（2576）
- [x] `versions/current.md` が v34.8.0 に更新されていること
- [x] `tasks.md` が COMPLETE

---

## コードレビューチェックリスト

- [x] `v348000_tests` に `use super::*` があること（`cmd_upgrade` 関数呼び出しのため必要）
- [x] `migration_guide_exists` の `include_str!` パスが `"../../MIGRATION.md"` であること
- [x] `cargo_toml_version_is_34_7_0` が空スタブになっていること（コメント付き）
- [x] 挿入位置が `v347000_tests` 直後・`// ── v31.7.0 tests` の前であること
- [x] `cmd_upgrade` が `--from-effects` なしで `Err` を返すこと
- [x] main.rs の `Some("upgrade")` アームが `Some("migrate")` の直後にあること
- [x] CHANGELOG.md の日付が正しいこと（2026-07-04）
- [x] `benchmarks/v34.8.0.json` の `milestone` が `"Production Ready"` であること
- [x] `versions/current.md` が v34.8.0 に更新されていること
- [x] `MIGRATION.md` に !Effect→ctx 対応表 / Before-After / fav upgrade 手順が含まれていること
