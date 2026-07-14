# v40.4.0 タスクリスト

**ステータス**: COMPLETE
**目標テスト数**: 2826（前バージョン 2823 + 3）
**実績テスト数**: 2826 passed, 0 failed（2026-07-11）

---

## T0 — 事前確認

- [x] `cargo test` が 2823 tests / 0 failures であることを確認
- [x] `fav/Cargo.toml` version が `40.3.0` であることを確認
- [x] `versions/roadmap/roadmap-v40.1-v41.0.md` §v40.4.0 を確認
- [x] `v40300_tests::cargo_toml_version_is_40_3_0` が NOTE コメント付きライブアサーションであることを確認し行番号を記録: 行44349
- [x] NOTE コメントが欠落している場合は実装を中断し報告すること
- [x] `v40300_tests` の閉じ `}` の行番号を確認し記録: 行44366
- [x] `driver.rs` に `v40400_tests` モジュールが存在しないことを確認（今回新規作成）

---

## T1 — stream.fav に with_late_policy 追記

- [x] `runes/stream/stream.fav` に `with_late_policy(stream, tolerance, policy)` スタブ関数を追加
  - スタブは全イベント通過（`Stream.filter(stream, fn(e) { true })`）
  - `public` キーワード付きで宣言
- [x] ヘッダーコメントを更新し `with_late_policy` 追記

---

## T2 — rune.toml バージョン bump + description 更新

- [x] `runes/stream/rune.toml` の `version = "40.3.0"` → `"40.4.0"` に変更
- [x] `runes/stream/rune.toml` の `description` に `with_late_policy` を追記

---

## T3 — Cargo.toml バージョン bump

- [x] `fav/Cargo.toml` の `version = "40.3.0"` → `"40.4.0"` に変更

---

## T4 — CHANGELOG.md 更新

- [x] `[v40.4.0]` エントリを `[v40.3.0]` の直後に追加

---

## T5 — driver.rs 更新

- [x] `v40300_tests::cargo_toml_version_is_40_3_0` をスタブ化
- [x] `v40400_tests` モジュール（3 テスト）を追加
  - `cargo_toml_version_is_40_4_0`
  - `changelog_has_v40_4_0`
  - `stream_fav_has_late_policy`

---

## T6 — テスト実行・確認

- [x] `cargo test` 実行
- [x] failures=0 を確認
- [x] テスト数 ≥ 2826 を確認（実績: 2826）
- [x] `v40400_tests` 3 件すべて pass を確認

---

## T7 — バージョン管理ドキュメント更新

- [x] `versions/current.md` を v40.4.0（最新安定版）・v40.5.0（次に切る版）に更新
- [x] `versions/roadmap/roadmap-v40.1-v41.0.md` の v40.4.0 を完了済みにマーク
  （`roadmap-v40.1-v45.0.md` はマスター概要のため個別バージョン完了マーク不要）
- [x] `versions/v40-v45/v40.4.0/tasks.md` を COMPLETE ステータスに更新（全チェックボックス [x]）

---

## コードレビュー指摘と対応

**code-reviewer** 1 件指摘:
- [LOW] stream.fav ヘッダーの `(v40.3.0)` が旧バージョンのまま → **修正済み**（`v40.4.0` に更新）

**spec-reviewer** 3 件指摘 → 実装前にすべて対応済み:
- [MED] 完了条件テーブルに番号欠番 → 自動検証 1〜5 連番、手動確認 M-1 に分離
- [LOW] `late_tolerance` vs `tolerance` 用語差 → spec.md に注記追加
- [LOW] plan.md 依存グラフ「T0 手動確認」コメントが誤解を招く → 修正済み

---

## 最終ステータス

- [x] 全タスク完了
- [x] spec-reviewer 指摘対応済み（3 件 → 全対応）
- [x] code-reviewer 指摘対応済み（実施済み）
