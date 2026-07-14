# v41.9.0 タスクリスト

**ステータス**: COMPLETE
**目標テスト数**: 2870（前バージョン 2868 + 2）
**実績テスト数**: 2870

---

## T0 — 事前確認

- [x] `cargo test` が 2868 tests / 0 failures であることを確認
- [x] `fav/Cargo.toml` version が `41.8.0` であることを確認
- [x] `versions/roadmap/roadmap-v41.1-v42.0.md` §v41.9.0 を確認
- [x] `v41800_tests::cargo_toml_version_is_41_8_0` が NOTE コメント付きライブアサーションであることを確認し行番号を記録: 44650
- [x] NOTE コメントが欠落している場合は実装を中断し報告すること
- [x] `site/content/docs/type-precision.mdx` が存在しないことを確認

---

## T1 — `site/content/docs/type-precision.mdx` 新規作成

- [x] frontmatter（title / description）を追加
- [x] Overview セクション（Type Precision フェーズの目標）を追加
- [x] v41.1〜v41.8 機能一覧テーブルを追加
- [x] v42.0 Preview セクション（宣言文の予告）を追加
- [x] ファイルが "Type Precision" を含むことを確認

---

## T2 — driver.rs テストモジュール更新

- [x] `v41800_tests::cargo_toml_version_is_41_8_0` をスタブ化（"Stubbed: version bumped to 41.9.0"）
- [x] `v41900_tests` モジュール（2 テスト）を `v41800_tests` の直前に追加:
  - `cargo_toml_version_is_41_9_0`（NOTE コメント付き）
  - `type_precision_doc_exists`

---

## T3 — Cargo.toml バージョン bump

- [x] `version = "41.8.0"` → `"41.9.0"`

---

## T4 — CHANGELOG.md 更新

- [x] `[v41.9.0]` エントリを `[v41.8.0]` の直前に追加

---

## T5 — テスト実行・確認

- [x] `cargo test` 実行
- [x] failures=0 を確認
- [x] テスト数 = 2870 を確認（2868 + 2 件）
- [x] `v41900_tests` 2 件 pass を確認
- [x] 既存テストが壊れていないことを確認

---

## T6 — バージョン管理ドキュメント更新

- [x] `versions/current.md` を v41.9.0（最新安定版）・v42.0.0（次に切る版）に更新
- [x] `versions/roadmap/roadmap-v41.1-v42.0.md` の v41.9.0 を完了済みにマーク（`✅ COMPLETE（2026-07-12）` を追記）
- [x] `versions/v40-v45/v41.9.0/tasks.md` を COMPLETE ステータスに更新（全チェックボックス [x]）
- [x] **MILESTONE.md 更新**: 本バージョンはコードフリーズ版（非マイルストーン宣言）のため不要

---

## 最終ステータス

- [x] 全タスク完了
