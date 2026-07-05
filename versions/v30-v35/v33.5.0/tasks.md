# v33.5.0 — タスクリスト

**ステータス**: COMPLETE

---

## 前提確認（T0）

- [x] `fav/Cargo.toml` の version が `33.4.0` であること
- [x] `benchmarks/v33.4.0.json` の `tests_passed` が 2512 であることを確認
- [x] `cargo test 2>&1 | grep "test result"` を実行して通過件数（2512 passed 想定）を実測確認すること
- [x] `driver.rs` に `mod v335000_tests` が存在しないこと
- [x] v33.4.0 が COMPLETE であること
- [x] `cargo_toml_version_is_33_4_0` が v334000_tests 内に存在すること（スタブ化対象）
- [x] `cargo test --bin fav v334000` が 4/4 PASS であること（前バージョン 4 件 PASS を確認）
- [x] `fav/src/driver.rs` に `cmd_compile_to_bytes` / `cmd_run_precompiled_bytes` が存在すること（v19.7.0 実装確認）
- [x] `fav/src/backend/artifact.rs` に `FvcArtifact` / `FavcMeta` が存在すること
- [x] `v197000_tests` のテスト名（`compile_produces_favc` / `precompiled_runs` / `precompiled_same_output` / `favc_version_check`）と v335000_tests のテスト名が重複しないこと

---

## 実装タスク

- [x] **T1** `fav/Cargo.toml` — version を `33.4.0` → `33.5.0` に更新
- [x] **T2** `fav/src/driver.rs` — `cargo_toml_version_is_33_4_0` をスタブ化
- [x] **T3** `fav/src/driver.rs` — `v335000_tests`（4 件）を追加
       挿入位置: `v334000_tests` 直後・`// ── v31.7.0 tests` の前
       `use super::*` なし、必要な import のみ明示
- [x] **T4** `CHANGELOG.md` — `[v33.5.0]` セクションを先頭に追記
- [x] **T5** `benchmarks/v33.5.0.json` — 新規作成（暫定値 2516、実測後に確定）
- [x] **T6** `versions/current.md` — 「最新安定版」欄を v33.5.0 に更新

---

## テスト確認

- [x] **T7** `cargo test --bin fav v335000 2>&1 | tail -8` — 4/4 PASS
- [x] **T8** `cargo test 2>&1 | grep "test result"` — 全件 PASS（2516 passed、0 failures）

---

## 完了処理

- [x] **T9** `benchmarks/v33.5.0.json` の `tests_passed` を実測値で更新（2516 確定）
- [x] **T10** このファイル（tasks.md）を COMPLETE に更新（全チェックボックス `[x]`）

---

## 完了条件チェックリスト（spec.md 対応）

- [x] `Cargo.toml` version = `"33.5.0"`
- [x] `cargo_toml_version_is_33_4_0` が空スタブになっていること
- [x] `cargo test --bin fav v335000` — 4/4 PASS
- [x] `cargo test` — 全件 PASS（2516 件、0 failures）
- [x] `CHANGELOG.md` に `[v33.5.0]` セクション
- [x] `benchmarks/v33.5.0.json` 存在かつ `tests_passed` が実測値
- [x] `benchmarks/v33.5.0.json` の `milestone` フィールドが `"Performance & Tooling"` であること
- [x] `versions/current.md` を v33.5.0 に更新
- [x] `tasks.md` が COMPLETE

---

## コードレビューチェックリスト

- [x] `v335000_tests` に `use super::*` が**ない**こと
- [x] `cargo_toml_version_is_33_4_0` が空スタブになっていること（コメント付き）
- [x] `favc_meta_source_hash_is_nonzero` / `favc_different_sources_differ` が v197000_tests のテスト名と異なること
- [x] `favc_meta_source_hash_is_nonzero` で `artifact.meta.is_some()` を先に `assert!` し、その後 `expect` で取り出して `meta.source_hash != [0u8; 32]` を assert していること
- [x] `favc_different_sources_differ` で `bytes_a != bytes_b` を assert していること
- [x] 挿入位置が `v334000_tests` 直後・`// ── v31.7.0 tests` の前であること
- [x] CHANGELOG.md の日付が正しいこと（2026-07-04）
- [x] `benchmarks/v33.5.0.json` の `milestone` が `"Performance & Tooling"` であること
- [x] `versions/current.md` が v33.5.0 に更新されていること
