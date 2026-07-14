# v44.7.0 タスク — ドキュメントサイト Precision & Flow 概要ページ

## ステータス: COMPLETE（2026-07-15）— 2958 tests

---

## T0 — 事前確認

- [x] `cargo test` 2956 / 0 確認
- [x] `Cargo.toml` version = `44.6.0` 確認
- [x] `v44700_tests` が `fav/src/driver.rs` に存在しないことを確認
- [x] `site/content/docs/precision-and-flow.mdx` が存在しないことを確認
- [x] `v44600_tests::cargo_toml_version_is_44_6_0` がまだスタブ化されていないこと（`assert!` 行が残っていること）を確認

---

## T1 — MDX ファイル作成

- [x] `site/content/docs/precision-and-flow.mdx` 作成
  - タイトル: `Precision & Flow` を含む
  - Refinement type / CEP / Opaque type / 型注釈 lineage / Back-pressure / E2E デモの 6 セクション
  - 各セクションに Favnir コードスニペット

---

## T2 — driver.rs: `v44700_tests` 追加 / スタブ化 / Cargo.toml

- [x] `v44600_tests` の直前（上の行）に `v44700_tests` を挿入（2 件）
  - `cargo_toml_version_is_44_7_0`
  - `precision_and_flow_doc_exists`
- [x] スタブ化: `v44600_tests::cargo_toml_version_is_44_6_0` の `assert!` を削除し `// Stubbed: version bumped to 44.7.0 in v44.7.0.` に置き換える
- [x] `fav/Cargo.toml` version を `44.6.0` → `44.7.0` に更新

---

## T3 — CHANGELOG.md に v44.7.0 エントリ追加

- [x] v44.7.0 エントリを CHANGELOG.md の先頭に追加（`[v44.7.0]` を含む）
  - ドキュメントサイト Precision & Flow 概要ページの説明
  - `site/content/docs/precision-and-flow.mdx` 成果物

---

## T4 — テスト実行・確認

- [x] `cargo test -j 8 -- --test-threads=8` 実行
- [x] 2958 passed; 0 failed 確認
- [x] `v44700_tests` 2 件 pass 確認

---

## T5 — バージョン管理ドキュメント更新

- [x] `versions/current.md` → v44.7.0 最新安定版（2958 tests）、次版 v44.8.0
- [x] `versions/roadmap/roadmap-v44.1-v45.0.md` → v44.7.0 を `✅ COMPLETE（2026-07-15）`、推定テスト数を実績に修正
- [x] `versions/v40-v45/v44.7.0/tasks.md` → COMPLETE、全チェックボックス `[x]`
