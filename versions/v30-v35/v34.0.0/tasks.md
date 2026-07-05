# v34.0.0 — タスクリスト

**ステータス**: COMPLETE

---

## 前提確認（T0）

- [x] `fav/Cargo.toml` の version が `33.9.0` であること
- [x] `benchmarks/v33.9.0.json` の `tests_passed` が 2532 であることを確認
- [x] `cargo test 2>&1 | grep "test result"` を実行して通過件数（2532 passed 想定）を実測確認すること
- [x] `driver.rs` に `mod v340000_tests` が存在しないこと
- [x] v33.9.0 が COMPLETE であること
- [x] `cargo_toml_version_is_33_9_0` が v339000_tests 内に存在すること（スタブ化対象）
- [x] `cargo test --bin fav v339000` が 4/4 PASS であること（前バージョン 4 件 PASS を確認）
- [x] `MILESTONE.md` に `Performance & Tooling` が存在しないこと（追加対象）
- [x] `README.md` に `v34` が存在しないこと（追加対象）
- [x] `versions/roadmap/roadmap-v33.1-v34.0.md` の cargo clean 規定を確認済みであること

---

## 実装タスク

- [x] **T0-clean** `fav/` で `cargo clean` を実行し `cargo build` が通ることを確認（20.5 GiB 削減、2m54s）
- [x] **T1** `fav/Cargo.toml` — version を `33.9.0` → `34.0.0` に更新
- [x] **T2** `fav/src/driver.rs` — `cargo_toml_version_is_33_9_0` をスタブ化
- [x] **T3** `fav/src/driver.rs` — `v340000_tests`（4 件）を追加
       挿入位置: `v339000_tests` 直後・`// ── v31.7.0 tests` の前
       `use super::*` なし、import なし（`include_str!` のみ使用）
- [x] **T4** `CHANGELOG.md` — `[v34.0.0]` セクションを先頭に追記
- [x] **T5** `benchmarks/v34.0.0.json` — 新規作成（`milestone`: `"Production Ready"`）
- [x] **T6** `MILESTONE.md` — `v34.0.0 — Performance & Tooling` セクションを先頭に追加
- [x] **T7** `README.md` — v34.0 マイルストーン行を v33.0 行の直後に追記
- [x] **T8** `versions/current.md` — 「最新安定版」欄を v34.0.0 に更新、マイルストーン表も更新

---

## テスト確認

- [x] **T9** `cargo test --bin fav v340000 2>&1 | tail -8` — 4/4 PASS
- [x] **T10** `cargo test 2>&1 | grep "test result"` — 全件 PASS（2536 passed、0 failures）

---

## 完了処理

- [x] **T11** `benchmarks/v34.0.0.json` の `tests_passed` を実測値で更新（2536 確定）
- [x] **T12** このファイル（tasks.md）を COMPLETE に更新（全チェックボックス `[x]`）

---

## 完了条件チェックリスト（spec.md 対応）

- [x] `cargo clean` + `cargo build` が通っていること
- [x] `Cargo.toml` version = `"34.0.0"`
- [x] `cargo_toml_version_is_33_9_0` が空スタブになっていること（他3テストは残存）
- [x] `cargo test --bin fav v340000` — 4/4 PASS
- [x] `cargo test` — 全件 PASS（2536 件、0 failures）
- [x] `CHANGELOG.md` に `[v34.0.0]` セクション
- [x] `MILESTONE.md` に `v34.0.0 — Performance & Tooling` セクション（先頭）
- [x] `README.md` に `v34` 言及
- [x] `benchmarks/v34.0.0.json` 存在かつ `tests_passed` が実測値（2536）
- [x] `benchmarks/v34.0.0.json` の `milestone` フィールドが `"Production Ready"`
- [x] `versions/current.md` を v34.0.0 に更新
- [x] `tasks.md` が COMPLETE

---

## コードレビューチェックリスト

- [x] `v340000_tests` に `use super::*` が**ない**こと
- [x] `v340000_tests` に import 文が**ない**こと（`include_str!` のみ）
- [x] WASM ゲートがないこと（ファイル読み込みのみ）
- [x] `cargo_toml_version_is_33_9_0` が空スタブになっていること（コメント付き）
- [x] `milestone_performance_tooling_declared` で `src.contains("Performance & Tooling")` を assert していること
- [x] `readme_mentions_v34` で `src.contains("v34")` を assert していること
- [x] 挿入位置が `v339000_tests` 直後・`// ── v31.7.0 tests` の前であること
- [x] CHANGELOG.md の日付が正しいこと（2026-07-04）
- [x] `benchmarks/v34.0.0.json` の `milestone` が `"Production Ready"` であること
- [x] `MILESTONE.md` の宣言日が `2026-07-04` であること
- [x] `versions/current.md` が v34.0.0 に更新されていること
