# v40.8.0 タスクリスト

**ステータス**: COMPLETE
**目標テスト数**: 2838（前バージョン 2835 + 3）
**実績テスト数**: 2838 passed, 0 failed（2026-07-11）

---

## T0 — 事前確認

- [x] `cargo test` が 2835 tests / 0 failures であることを確認
- [x] `fav/Cargo.toml` version が `40.7.0` であることを確認
- [x] `versions/roadmap/roadmap-v40.1-v41.0.md` §v40.8.0 を確認
- [x] `v40700_tests::cargo_toml_version_is_40_7_0` が NOTE コメント付きライブアサーションであることを確認し行番号を記録: 行44444
- [x] NOTE コメントが欠落している場合は実装を中断し報告すること
- [x] `v40700_tests` の閉じ `}` の行番号を確認し記録: 行44461
- [x] `driver.rs` に `v40800_tests` モジュールが存在しないことを確認（今回新規作成）
- [x] `site/content/cookbook/window-aggregation.mdx` が存在しないことを確認
- [x] `site/content/cookbook/kafka-streaming.mdx` が存在しないことを確認

---

## T1 — window-aggregation.mdx 作成

- [x] `site/content/cookbook/window-aggregation.mdx` を新規作成
  - フロントマター（title / description）
  - タンブリングウィンドウを使ったコード例（`tumbling_window` を含む）
  - 関連 Rune セクション

---

## T2 — kafka-streaming.mdx 作成

- [x] `site/content/cookbook/kafka-streaming.mdx` を新規作成
  - フロントマター（title / description）
  - `consume_windowed` を使ったコード例
  - 関連 Rune セクション

---

## T3 — Cargo.toml バージョン bump

- [x] `fav/Cargo.toml` の `version = "40.7.0"` → `"40.8.0"` に変更

---

## T4 — CHANGELOG.md 更新

- [x] `[v40.8.0]` エントリを `[v40.7.0]` の直後に追加

---

## T5 — driver.rs テストモジュール更新

- [x] `v40700_tests::cargo_toml_version_is_40_7_0` をスタブ化
  ```rust
  #[test]
  fn cargo_toml_version_is_40_7_0() {
      // Stubbed: version bumped to 40.8.0 — assertion intentionally removed
  }
  ```
- [x] `v40800_tests` モジュール（3 テスト）を末尾に追加（`use super::*` 不要）
  - `cargo_toml_version_is_40_8_0`（NOTE コメント付き）
  - `changelog_has_v40_8_0`
  - `cookbook_window_aggregation_exists`（`tumbling_window` を含むことを検証）

---

## T6 — テスト実行・確認

- [x] `cargo test` 実行
- [x] failures=0 を確認
- [x] テスト数 ≥ 2838 を確認（実績: 2838）
- [x] `v40800_tests` 3 件すべて pass を確認

---

## T7 — バージョン管理ドキュメント更新

- [x] `versions/current.md` を v40.8.0（最新安定版）・v40.9.0（次に切る版）に更新
- [x] `versions/roadmap/roadmap-v40.1-v41.0.md` の v40.8.0 を完了済みにマーク
  （`roadmap-v40.1-v45.0.md` はマスター概要のため個別バージョン完了マーク不要）
- [x] `versions/v40-v45/v40.8.0/tasks.md` を COMPLETE ステータスに更新（全チェックボックス [x]）

---

## コードレビュー指摘と対応

**code-reviewer 指摘（実装後対応）:**
- [LOW] `window-aggregation.mdx` コード例に `import kafka` / `import db` が欠落 → 追加 ✅
- [LOW] `kafka-streaming.mdx` コード例に `import db` が欠落 → 追加 ✅

---

## 最終ステータス

- [x] 全タスク完了
- [x] spec-reviewer 指摘対応済み（[HIGH] include_str! パス `../../../` → `../../` 修正、[LOW] kafka no-test 理由修正、plan.md パス説明修正）
- [x] code-reviewer 指摘対応済み（2 件 → 全対応）
