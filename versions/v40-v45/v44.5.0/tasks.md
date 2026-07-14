# v44.5.0 タスク — Back-pressure x `fav policy` 統合

## ステータス: COMPLETE（2026-07-14）— 2955 tests

---

## T0 — 事前確認

- [x] `cargo test` 2953 / 0 確認
- [x] `Cargo.toml` version = `44.4.0` 確認
- [x] `v44500_tests` が `fav/src/driver.rs` に存在しないことを確認
- [x] `collect_stage_max_inflight_annotations` が `fav/src/driver.rs` に存在しないことを確認

---

## T1 — driver.rs: `collect_stage_max_inflight_annotations` 追加

- [x] `collect_annotated_lineage_bindings` の直後（`bare_inner_literal_line` の直前）に追加
  - `Item::TrfDef(td)` を走査
  - `td.max_inflight.is_some()` のステージを収集
  - 返り値: `"<filename>:<line>: <stage_name>: max_inflight=<n>"` 形式

---

## T2 — driver.rs: `v44500_tests` 追加 / スタブ化 / Cargo.toml

- [x] `v44400_tests` の直前に `v44500_tests` を挿入（2 件）
  - `cargo_toml_version_is_44_5_0`
  - `stage_max_inflight_annotation_detected`
- [x] スタブ化: `v44400_tests::cargo_toml_version_is_44_4_0` の `assert!` を削除し `// Stubbed: version bumped to 44.5.0 in v44.5.0.` に置き換える
- [x] `fav/Cargo.toml` version を `44.4.0` → `44.5.0` に更新

---

## T3 — CHANGELOG.md に v44.5.0 エントリ追加

- [x] v44.5.0 エントリを CHANGELOG.md の先頭に追加（`[v44.5.0]` を含む）
  - Back-pressure x `fav policy` 統合の説明
  - `collect_stage_max_inflight_annotations` ヘルパー追加

---

## T4 — テスト実行・確認

- [x] `cargo test -j 8 -- --test-threads=8` 実行
- [x] 2955 passed; 0 failed 確認
- [x] `v44500_tests` 2 件 pass 確認

---

## T5 — バージョン管理ドキュメント更新

- [x] `versions/current.md` → v44.5.0 最新安定版（2955 tests）、次版 v44.6.0
- [x] `versions/roadmap/roadmap-v44.1-v45.0.md` → v44.5.0 を `✅ COMPLETE（2026-07-14）`、推定テスト数を実績に修正
- [x] `versions/v40-v45/v44.5.0/tasks.md` → COMPLETE、全チェックボックス `[x]`
