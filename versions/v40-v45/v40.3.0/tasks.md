# v40.3.0 タスクリスト

**ステータス**: COMPLETE
**目標テスト数**: 2823（前バージョン 2820 + 3）
**実績テスト数**: 2823 passed, 0 failed（2026-07-11）

---

## T0 — 事前確認

- [x] `cargo test` が 2820 tests / 0 failures であることを確認
- [x] `fav/Cargo.toml` version が `40.2.0` であることを確認
- [x] `versions/roadmap/roadmap-v40.1-v41.0.md` §v40.3.0 を確認
- [x] `v40200_tests::cargo_toml_version_is_40_2_0` が NOTE コメント付きライブアサーションであることを確認し行番号を記録: 行44326
- [x] NOTE コメントが欠落している場合は実装を中断し報告すること
- [x] `v40200_tests` の閉じ `}` の行番号を確認し記録: 行44342
- [x] `driver.rs` に `v40300_tests` モジュールが存在しないことを確認（今回新規作成）

---

## T1 — stream.fav に Event<T> 型定義追加

- [x] `runes/stream/stream.fav` に `Event` 型定義（`value: Any` / `timestamp: Int`）を追加
  - `type Event<T>` のジェネリクス構文が未サポートのため `Any` でスタブ化（TODO 明記）
  - 完全なジェネリクス統合は v43.x 型推論スプリントへ持ち越し
- [x] ヘッダーコメントを v40.3.0 に更新し `Event<T>` 追記

---

## T2 — rune.toml バージョン bump + description 更新

- [x] `runes/stream/rune.toml` の `version = "40.2.0"` → `"40.3.0"` に変更
- [x] `runes/stream/rune.toml` の `description` に `Event<T>(timestamp)` を追記

---

## T3 — Cargo.toml バージョン bump

- [x] `fav/Cargo.toml` の `version = "40.2.0"` → `"40.3.0"` に変更

---

## T4 — CHANGELOG.md 更新

- [x] `[v40.3.0]` エントリを `[v40.2.0]` の直後に追加

---

## T5 — driver.rs 更新

- [x] `v40200_tests::cargo_toml_version_is_40_2_0` をスタブ化
- [x] `v40300_tests` モジュール（3 テスト）を追加
  - `cargo_toml_version_is_40_3_0`
  - `changelog_has_v40_3_0`
  - `stream_fav_has_event_type`（`Event` + `timestamp` の両方を検証）

---

## T6 — テスト実行・確認

- [x] `cargo test` 実行
- [x] failures=0 を確認
- [x] テスト数 ≥ 2823 を確認（実績: 2823）
- [x] `v40300_tests` 3 件すべて pass を確認

---

## T7 — バージョン管理ドキュメント更新

- [x] `versions/current.md` を v40.3.0（最新安定版）・v40.4.0（次に切る版）に更新
- [x] `versions/roadmap/roadmap-v40.1-v41.0.md` の v40.3.0 を完了済みにマーク
  （`roadmap-v40.1-v45.0.md` はマスター概要のため個別バージョン完了マーク不要）
- [x] `versions/v40-v45/v40.3.0/tasks.md` を COMPLETE ステータスに更新（全チェックボックス [x]）

---

## コードレビュー指摘と対応

**code-reviewer** 3 件指摘:
- [MED] CHANGELOG / rune.toml / ヘッダーが `Event<T>` / `value: T` と記載、実装は `Any` — **修正済み**（`Event<T>` → `Event`、`value: T` → `value: Any スタブ` に統一）
- [LOW] `type Event` に `public` なし → **修正済み**（`public type Event` に変更）
- [LOW] テストアサーションが `"Event"` 文字列と緩い → スタブフェーズでは設計上の既知事項として**許容**

**spec-reviewer** 3 件指摘 → 実装前にすべて対応済み:
- [MED] フォールバック時のテスト合否基準未定義 → spec.md に「コメントスタブでも pass 扱い」と明示
- [MED] tasks.md T2 に rune.toml description 更新欠落 → T2 チェックボックス追加・spec.md 変更表更新
- [LOW] 完了条件の手動確認項目が混在 → 「自動検証」「手動確認」2 セクションに分離

---

## 最終ステータス

- [x] 全タスク完了
- [x] spec-reviewer 指摘対応済み（3 件 → 全対応）
- [x] code-reviewer 指摘対応済み（実施済み）
