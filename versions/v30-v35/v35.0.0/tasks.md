# v35.0.0 — タスクリスト

**ステータス**: COMPLETE

---

## 前提確認（T0）

- [x] `fav/Cargo.toml` の version が `34.9.0` であること
- [x] `benchmarks/v34.9.0.json` の `tests_passed` が 2581 であることを確認
- [x] `cargo test 2>&1 | grep "test result"` を実行して通過件数（2581 passed 想定）を実測確認すること
- [x] `driver.rs` に `mod v350000_tests` が存在しないこと
- [x] v34.9.0 が COMPLETE であること
- [x] `cargo_toml_version_is_34_9_0` が v349000_tests 内に存在すること（スタブ化対象）
- [x] `cargo test --bin fav v349000` が 5/5 PASS であること
- [x] `MILESTONE.md` に `Production Ready` が存在しないこと（追加対象）
- [x] `README.md` に `v35` が存在しないこと（追加対象）
- [x] `versions/roadmap/roadmap-v34.1-v35.0.md` の cargo clean 規定を確認済みであること
- [x] `examples/real-world-etl/README.md` が存在すること（v34.1 で作成済み）

---

## 実装タスク

- [x] **T0-clean** `fav/` で `cargo clean` を実行し `cargo build` が通ることを確認（24.2 GiB 削減、3m51s）
- [x] **T1** `fav/Cargo.toml` — version を `34.9.0` → `35.0.0` に更新
- [x] **T2** `fav/src/driver.rs` — `cargo_toml_version_is_34_9_0` をスタブ化
- [x] **T3** `fav/src/driver.rs` — `v350000_tests`（5 件）を追加
       挿入位置: `v349000_tests` 直後・`// ── v31.7.0 tests` の前
       `use super::*` なし、import なし（`include_str!` のみ使用）
- [x] **T4** `CHANGELOG.md` — `[v35.0.0]` セクションを先頭に追記
- [x] **T5** `MILESTONE.md` — `v35.0.0 — Production Ready` セクションを先頭に追加
- [x] **T6** `README.md` — v35.0 マイルストーン行を v34.0 行の直後に追記
- [x] **T7** `benchmarks/v35.0.0.json` — 新規作成（`tests_passed`: 2586）
- [x] **T8** `versions/current.md` — 「最新安定版」欄を v35.0.0 に更新、マイルストーン表も更新

---

## テスト確認

- [x] **T9** `cargo test --bin fav v350000 2>&1 | tail -8` — 5/5 PASS
- [x] **T10** `cargo test 2>&1 | grep "test result"` — 全件 PASS（2586 passed、0 failures）
- [x] **T11** `cargo clippy --locked -- -D warnings` — warnings なし（Finished のみ）
- [x] **T12** `./target/debug/fav lint ... self/compiler.fav` — ok
- [x] **T13** `./target/debug/fav lint ... self/checker.fav` — ok

---

## 完了処理

- [x] **T14** `benchmarks/v35.0.0.json` の `tests_passed` を実測値（2586）で確定（暫定値と一致）
- [x] **T15** このファイル（tasks.md）を COMPLETE に更新（全チェックボックス `[x]`）

---

## 完了条件チェックリスト（spec.md 対応）

- [x] `cargo clean` + `cargo build` が通っていること（24.2 GiB 削減）
- [x] `Cargo.toml` version = `"35.0.0"`
- [x] `cargo_toml_version_is_34_9_0` が空スタブになっていること
- [x] `cargo test --bin fav v350000` — 5/5 PASS
- [x] `cargo test` — 全件 PASS（2586 件、0 failures）
- [x] `cargo clippy --locked -- -D warnings` — PASS
- [x] `fav lint` (compiler.fav / checker.fav) — PASS
- [x] `CHANGELOG.md` に `[v35.0.0]` セクション
- [x] `MILESTONE.md` に `v35.0.0 — Production Ready` セクション（先頭）
- [x] `README.md` に `v35` 言及
- [x] `benchmarks/v35.0.0.json` 存在かつ `tests_passed` が実測値（2586）
- [x] `benchmarks/v35.0.0.json` の `milestone` フィールドが `"Production Ready"`
- [x] `versions/current.md` を v35.0.0 に更新
- [x] `tasks.md` が COMPLETE

---

## コードレビューチェックリスト

- [x] `v350000_tests` に `use super::*` が**ない**こと
- [x] `v350000_tests` に import 文が**ない**こと（`include_str!` のみ）
- [x] WASM ゲートがないこと（ファイル読み込みのみ）
- [x] `cargo_toml_version_is_34_9_0` が空スタブになっていること（コメント付き）
- [x] `milestone_production_ready_declared` で `src.contains("Production Ready")` を assert していること
- [x] `readme_mentions_v35` で `src.contains("v35")` を assert していること
- [x] `real_world_etl_example_exists` で `include_str!("../../examples/real-world-etl/README.md")` を使用していること
- [x] 挿入位置が `v349000_tests` 直後・`// ── v31.7.0 tests` の前であること
- [x] CHANGELOG.md の日付が正しいこと（2026-07-04）
- [x] `benchmarks/v35.0.0.json` の `milestone` が `"Production Ready"` であること
- [x] `MILESTONE.md` の宣言日が `2026-07-04` であること
- [x] `versions/current.md` の「次に切る版」が `未定` になっていること
- [x] `benchmarks/v35.0.0.json` の `tests_failed` が `0` であること
- [x] `real_world_etl_example_exists` の assert 文字列が `"30 分"` であること
