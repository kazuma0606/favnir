# v40.2.0 タスクリスト

**ステータス**: COMPLETE
**目標テスト数**: 2820（前バージョン 2817 + 3）
**実績テスト数**: 2820 passed, 0 failed（2026-07-11）

---

## T0 — 事前確認

- [x] `cargo test` が 2817 tests / 0 failures であることを確認
- [x] `fav/Cargo.toml` version が `40.1.0` であることを確認
- [x] `versions/roadmap/roadmap-v40.1-v41.0.md` §v40.2.0 を確認
- [x] `v40100_tests::cargo_toml_version_is_40_1_0` が NOTE コメント付きライブアサーションであることを確認し行番号を記録: 行44302
- [x] NOTE コメントが欠落している場合は実装を中断し報告すること
- [x] `v40100_tests` の閉じ `}` の行番号を確認し記録: 行44319
- [x] `driver.rs` に `v40200_tests` モジュールが存在しないことを確認（今回新規作成）

---

## T1 — stream.fav に session_window 追記

- [x] `runes/stream/stream.fav` に `session_window(stream, gap)` スタブ関数を追加
- [x] ヘッダーコメントを v40.2.0 に更新し session_window 追記

---

## T2 — rune.toml バージョン bump

- [x] `runes/stream/rune.toml` の `version = "40.1.0"` → `"40.2.0"` に変更
- [x] description に `session_window` を追記

---

## T3 — Cargo.toml バージョン bump

- [x] `fav/Cargo.toml` の `version = "40.1.0"` → `"40.2.0"` に変更

---

## T4 — CHANGELOG.md 更新

- [x] `[v40.2.0]` エントリを `[v40.1.0]` の直後に追加

---

## T5 — driver.rs 更新

- [x] `v40100_tests::cargo_toml_version_is_40_1_0` をスタブ化
- [x] `v40200_tests` モジュール（3 テスト）を追加
  - `cargo_toml_version_is_40_2_0`
  - `changelog_has_v40_2_0`
  - `stream_rune_has_session_window`

---

## T6 — テスト実行・確認

- [x] `cargo test` 実行
- [x] failures=0 を確認
- [x] テスト数 ≥ 2820 を確認（実績: 2820）
- [x] `v40200_tests` 3 件すべて pass を確認

---

## T7 — バージョン管理ドキュメント更新

- [x] `versions/current.md` を v40.2.0（最新安定版）・v40.3.0（次に切る版）に更新
- [x] `versions/roadmap/roadmap-v40.1-v41.0.md` の v40.2.0 を完了済みにマーク
  （`roadmap-v40.1-v45.0.md` はマスター概要のため個別バージョン完了マーク不要）
- [x] `versions/v40-v45/v40.2.0/tasks.md` を COMPLETE ステータスに更新（全チェックボックス [x]）

---

## コードレビュー指摘と対応

**spec-reviewer** 3 件指摘 → 実装前にすべて対応済み:
- [MED] plan.md 依存グラフに rune.toml → Step 6 の線が欠落 → 修正済み
- [LOW] roadmap-v40.1-v45.0.md 完了マーク方針不明確 → T7 に注記追加
- [LOW] rune.toml version 検証テストなし → 手動確認項目として spec.md に明示

---

## 最終ステータス

- [x] 全タスク完了
- [x] spec-reviewer 指摘対応済み（3 件 → 全対応）
- [x] code-reviewer 指摘対応済み（実施済み）
