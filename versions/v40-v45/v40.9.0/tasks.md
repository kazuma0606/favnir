# v40.9.0 タスクリスト

**ステータス**: COMPLETE
**目標テスト数**: 2840（前バージョン 2838 + 2）
**実績テスト数**: 2841 passed, 0 failed（2026-07-11）

---

## T0 — 事前確認

- [x] `cargo test` が 2838 tests / 0 failures であることを確認
- [x] `fav/Cargo.toml` version が `40.8.0` であることを確認
- [x] `versions/roadmap/roadmap-v40.1-v41.0.md` §v40.9.0 を確認
- [x] `v40800_tests::cargo_toml_version_is_40_8_0` が NOTE コメント付きライブアサーションであることを確認し行番号を記録: 行44464
- [x] NOTE コメントが欠落している場合は実装を中断し報告すること
- [x] `v40800_tests` の閉じ `}` の行番号を確認し記録: 行44481
- [x] `driver.rs` に `v40900_tests` モジュールが存在しないことを確認（今回新規作成）
- [x] `site/content/docs/streaming-foundations.mdx` が存在しないことを確認

---

## T1 — streaming-foundations.mdx 作成

- [x] `site/content/docs/streaming-foundations.mdx` を新規作成
  - フロントマター（title / description）
  - ウィンドウ関数一覧（tumbling_window / sliding_window / session_window）
  - イベント型・Watermark・late_policy 説明
  - 関連 cookbook リンク
  - `streaming-foundations` という文字列を含めること

---

## T2 — Cargo.toml バージョン bump

- [x] `fav/Cargo.toml` の `version = "40.8.0"` → `"40.9.0"` に変更

---

## T3 — CHANGELOG.md 更新

- [x] `[v40.9.0]` エントリを `[v40.8.0]` の直後に追加

---

## T4 — driver.rs テストモジュール更新

- [x] `v40800_tests::cargo_toml_version_is_40_8_0` をスタブ化
  ```rust
  #[test]
  fn cargo_toml_version_is_40_8_0() {
      // Stubbed: version bumped to 40.9.0 — assertion intentionally removed
  }
  ```
- [x] `v40900_tests` モジュール（2 テスト）を末尾に追加（`use super::*` 不要）
  - `cargo_toml_version_is_40_9_0`（NOTE コメント付き）
  - `changelog_has_v40_9_0`

---

## T5 — テスト実行・確認

- [x] `cargo test` 実行
- [x] failures=0 を確認
- [x] テスト数 ≥ 2840 を確認（実績: 2841）
- [x] `v40900_tests` 3 件すべて pass を確認

---

## T6 — バージョン管理ドキュメント更新

- [x] `versions/current.md` を v40.9.0（最新安定版）・v41.0.0（次に切る版）に更新
- [x] `versions/roadmap/roadmap-v40.1-v41.0.md` の v40.9.0 を完了済みにマーク（完了条件テスト数を実績値 2840 に更新）
- [x] `versions/v40-v45/v40.9.0/tasks.md` を COMPLETE ステータスに更新（全チェックボックス [x]）

---

## コードレビュー指摘と対応

**code-reviewer 指摘（実装後対応）:**
- [HIGH] `v40900_tests` に `streaming_foundations_doc_exists` テストが欠落 → 追加（2 → 3 テスト、2840 → 2841）✅
- [MED] `streaming-foundations.mdx` コード例で `let` を使用（Favnir 言語仕様違反） → `bind x <- expr` に修正（全 8 箇所）✅
- [LOW] CHANGELOG の `v40900_tests` 件数が 2 → 3 に未更新 → 更新 ✅

---

## 最終ステータス

- [x] 全タスク完了
- [x] spec-reviewer 指摘対応済み（[MED] 2件・[LOW] 1件 → 全対応）
- [x] code-reviewer 指摘対応済み（3 件 → 全対応）
