# v41.0.0 タスクリスト

**ステータス**: COMPLETE
**目標テスト数**: 2845（前バージョン 2841 + 4）
**実績テスト数**: 2845 passed, 0 failed（2026-07-11）

---

## T0 — 事前確認

- [x] `cargo test` が 2841 tests / 0 failures であることを確認
- [x] `fav/Cargo.toml` version が `40.9.0` であることを確認
- [x] `versions/roadmap/roadmap-v40.1-v41.0.md` §v41.0.0 を確認
- [x] `v40900_tests::cargo_toml_version_is_40_9_0` が NOTE コメント付きライブアサーションであることを確認し行番号を記録: 行44484
- [x] NOTE コメントが欠落している場合は実装を中断し報告すること
- [x] `v40900_tests` の閉じ `}` の行番号を確認し記録: 行44501
- [x] `driver.rs` に `v41000_tests` モジュールが存在しないことを確認（今回新規作成）
- [x] `site/content/docs/streaming-foundations.mdx` が存在することを確認（v40.9.0 の成果物）
- [x] `MILESTONE.md` に `Streaming Foundations` が含まれないことを確認
- [x] `README.md` に `Streaming Foundations` が含まれないことを確認

---

## T1 — MILESTONE.md 更新

- [x] v41.0.0 エントリを v40.0.0 エントリの直前（先頭）に追加
  - 宣言文（`tumbling_window` / `sliding_window` / `session_window` / `Event<T>` の言及）
  - `Streaming Foundations` という文字列を含めること
  - 達成コンポーネント表（v40.1〜v40.9 の 9 件）
  - 宣言日: 2026-07-11

---

## T2 — README.md 更新

- [x] README.md に `Streaming Foundations`（v41.0）の記述を追加

---

## T3 — Cargo.toml バージョン bump

- [x] `fav/Cargo.toml` の `version = "40.9.0"` → `"41.0.0"` に変更

---

## T4 — CHANGELOG.md 更新

- [x] `[v41.0.0]` エントリを `[v40.9.0]` の直後に追加

---

## T5 — driver.rs テストモジュール更新

- [x] `v40900_tests::cargo_toml_version_is_40_9_0` をスタブ化
  ```rust
  #[test]
  fn cargo_toml_version_is_40_9_0() {
      // Stubbed: version bumped to 41.0.0 — assertion intentionally removed
  }
  ```
- [x] `v41000_tests` モジュール（4 テスト）を末尾に追加（`use super::*` 不要）
  - `cargo_toml_version_is_41_0_0`（NOTE コメント付き）
  - `changelog_has_v41_0_0`
  - `milestone_has_streaming_foundations`
  - `readme_mentions_streaming_foundations`

---

## T6 — テスト実行・確認（クリーンアップ前）

- [x] `cargo test` 実行
- [x] failures=0 を確認
- [x] テスト数 ≥ 2845 を確認（実績: 2845）
- [x] `v41000_tests` 4 件すべて pass を確認

---

## T7 — ★cargo clean + hello.fav 復元 + cargo test 再実行

- [x] `cargo clean` を実行（24.8 GiB 削除）
- [x] `fav/tmp/hello.fav` を確認（cargo clean では削除されないことを確認）
- [x] `cargo test` を再実行し 2845 passed / 0 failed を確認

---

## T8 — バージョン管理ドキュメント更新

- [x] `versions/current.md` を v41.0.0（最新安定版）・v41.1.0（次に切る版）に更新
- [x] `versions/roadmap/roadmap-v40.1-v41.0.md` の v41.0.0 を完了済みにマーク
- [x] `versions/v40-v45/v41.0.0/tasks.md` を COMPLETE ステータスに更新（全チェックボックス [x]）

---

## コードレビュー指摘と対応

**code-reviewer 指摘（実装後対応）:**
- [MED] README.md の v41.0 宣言行直後に v40.0（Enterprise Governance）の説明文が誤配置 → v40.0 説明を正しい位置（v40.0 行の直後）に移動し、v41.0 固有の説明（`tumbling_window` / `Event<T>` / `consume_windowed`）を追加 ✅
- [LOW] README.md v40.0 行に本文説明が欠落（[MED] と同根） → [MED] 修正で同時解消 ✅

---

## 最終ステータス

- [x] 全タスク完了
- [x] spec-reviewer 指摘対応済み（[HIGH] 1件・[MED] 1件・[LOW] 2件 → 全対応）
- [x] code-reviewer 指摘対応済み（2 件 → 全対応）
