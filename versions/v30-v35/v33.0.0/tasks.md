# v33.0.0 — タスクリスト

**ステータス**: COMPLETE

---

## 前提確認（T0）

- [x] `fav/Cargo.toml` の version が `32.9.0` であること
- [x] `benchmarks/v32.9.0.json` の `tests_passed` が 2492 であることを確認
- [x] `cargo test 2>&1 | grep "test result"` を実行して通過件数（2492 passed 想定）を実測確認すること
- [x] `driver.rs` に `mod v330000_tests` が存在しないこと
- [x] v32.9.0 が COMPLETE であること
- [x] `cargo_toml_version_is_32_9_0` が v329000_tests 内に存在すること（スタブ化対象）
- [x] `MILESTONE.md` に `"Language Power"` がまだ含まれていないこと（先行宣言がないことを確認）
- [x] `cargo test --bin fav v329000` が 4/4 PASS であること（前バージョン 4 件 PASS を確認）
- [x] テスト名が v180000_tests（`changelog_has_v17_entries` / `readme_mentions_bounded_generics` / `readme_mentions_package_system` / `docs_generics_exists`）と重複しないこと

---

## 実装タスク

- [x] **T1** `fav/Cargo.toml` — version を `32.9.0` → `33.0.0` に更新
- [x] **T2** `fav/src/driver.rs` — `cargo_toml_version_is_32_9_0` をスタブ化
- [x] **T3** `fav/src/driver.rs` — `v330000_tests`（4 件）を追加
       挿入位置: `v329000_tests` 直後・`// ── v31.7.0 tests` の前
       `use super::*` なし、`include_str!` のみ使用
- [x] **T4** `MILESTONE.md` — v33.0.0「Language Power」セクションを先頭に追加
- [x] **T5** `README.md` — v32.0 行直後に v33.0 マイルストーン宣言を追加
- [x] **T6** `CHANGELOG.md` — `[v33.0.0]` セクションを先頭に追記
- [x] **T7** `benchmarks/v33.0.0.json` — 新規作成（暫定値 2496、実測後に確定）
- [x] **T8** `versions/current.md` — 「最新安定版」欄を v33.0.0 に更新
- [x] **T9** `cargo clean` → `fav/tmp/hello.fav` 復元 → `cargo build` → `cargo test`（クリーンアップ実施）

---

## テスト確認

- [x] **T10** `cargo test --bin fav v330000 2>&1 | tail -8` — 4/4 PASS
- [x] **T11** `cargo test 2>&1 | grep "test result"` — 全件 PASS（2496 passed、0 failures）

---

## 完了処理

- [x] **T12** `benchmarks/v33.0.0.json` の `tests_passed` を実測値（2496）で更新
- [x] **T13** このファイル（tasks.md）を COMPLETE に更新（全チェックボックス `[x]`）

---

## 完了条件チェックリスト（spec.md 対応）

- [x] `Cargo.toml` version = `"33.0.0"`
- [x] `MILESTONE.md` に `"Language Power"` セクションが存在すること
- [x] `README.md` に `"v33.0"` の記述があること
- [x] `cargo test --bin fav v330000` — 4/4 PASS
- [x] `cargo test`（`cargo clean` 後）— 全件 PASS（2496 件、0 failures）
- [x] `CHANGELOG.md` に `[v33.0.0]` セクション
- [x] `benchmarks/v33.0.0.json` 存在かつ `tests_passed` が実測値（2496）
- [x] `benchmarks/v33.0.0.json` の `milestone` フィールドが `"Language Power"` であること
- [x] `versions/current.md` を v33.0.0 に更新
- [x] `tasks.md` が COMPLETE

---

## コードレビューチェックリスト

- [x] `v330000_tests` に `use super::*` が**ない**こと（`include_str!` のみ）
- [x] `cargo_toml_version_is_32_9_0` が空スタブになっていること（コメント付き）
- [x] `milestone_language_power_declared` が v180000_tests のテスト名と異なること
- [x] `readme_mentions_v33_0` が v180000_tests のテスト名と異なること
- [x] 挿入位置が `v329000_tests` 直後・`// ── v31.7.0 tests` の前であること
- [x] MILESTONE.md に v33.0.0「Language Power」セクションが先頭に追加されていること
- [x] README.md に v33.0 マイルストーン宣言が v32.0 行直後に追加されていること
- [x] CHANGELOG.md の日付が正しいこと（2026-07-03）
- [x] `benchmarks/v33.0.0.json` の `milestone` が `"Language Power"` であること
- [x] `versions/current.md` が v33.0.0 に更新されていること
- [x] `cargo clean` が実施されていること（`fav/tmp/hello.fav` 復元済み）
