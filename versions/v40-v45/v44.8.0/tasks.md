# v44.8.0 タスク — パフォーマンス最終調整

## ステータス: COMPLETE（2026-07-15）— 2960 tests

---

## T0 — 事前確認

- [x] `cargo test` 2958 / 0 確認
- [x] `Cargo.toml` version = `44.7.0` 確認
- [x] `v44800_tests` が `fav/src/driver.rs` に存在しないことを確認
- [x] `collect_bench_stream_notes` が `fav/src/driver.rs` に存在しないことを確認
- [x] `v44700_tests::cargo_toml_version_is_44_7_0` がまだスタブ化されていないこと（`assert!` 行が残っていること）を確認

---

## T1 — driver.rs: `collect_bench_stream_notes` 追加

- [x] `collect_stage_max_inflight_annotations` の直後（`bare_inner_literal_line` の直前）に追加
  - `changelog.lines()` を走査して `contains("bench --stream")` の行を収集
  - 返り値: マッチした行のトリム済み文字列リスト `Vec<String>`

---

## T2 — driver.rs: `v44800_tests` 追加 / スタブ化 / Cargo.toml

- [x] `v44700_tests` の直前（上の行）に `v44800_tests` を挿入（2 件）
  - `cargo_toml_version_is_44_8_0`
  - `bench_stream_result_recorded_in_changelog`
- [x] スタブ化: `v44700_tests::cargo_toml_version_is_44_7_0` の `assert!` を削除し `// Stubbed: version bumped to 44.8.0 in v44.8.0.` に置き換える
- [x] `fav/Cargo.toml` version を `44.7.0` → `44.8.0` に更新

---

## T3 — CHANGELOG.md に v44.8.0 エントリ追加（`bench --stream` 記述必須）

- [x] v44.8.0 エントリを CHANGELOG.md の先頭に追加（`[v44.8.0]` を含む）
  - `### Performance` セクションに `bench --stream` 記述を含める（テストのアサート条件）
  - `collect_bench_stream_notes` ヘルパー追加の説明

---

## T4 — テスト実行・確認

- [x] `cargo test -j 8 -- --test-threads=8` 実行
- [x] 2960 passed; 0 failed 確認
- [x] `v44800_tests` 2 件 pass 確認

---

## T5 — バージョン管理ドキュメント更新

- [x] `versions/current.md` → v44.8.0 最新安定版（2960 tests）、次版 v44.9.0
- [x] `versions/roadmap/roadmap-v44.1-v45.0.md` → v44.8.0 を `✅ COMPLETE（2026-07-15）`、実績テスト数を記録（`v41.0 比で改善` 文言は実装前に将来版スコープに修正済み）
- [x] `versions/v40-v45/v44.8.0/tasks.md` → COMPLETE、全チェックボックス `[x]`
