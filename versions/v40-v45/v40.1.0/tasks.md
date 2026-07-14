# v40.1.0 タスクリスト

**ステータス**: COMPLETE
**目標テスト数**: 2817（前バージョン 2814 + 3）
**実績テスト数**: 2817 passed, 0 failed（2026-07-11）

---

## T0 — 事前確認

- [x] `cargo test` が 2814 tests / 0 failures であることを確認
- [x] `fav/Cargo.toml` version が `40.0.0` であることを確認
- [x] `versions/roadmap/roadmap-v40.1-v41.0.md` §v40.1.0 を確認
- [x] `v40000_tests::cargo_toml_version_is_40_0_0` が NOTE コメント付きライブアサーションであることを確認し行番号を記録: 行44273
- [x] NOTE コメントが欠落している場合は実装を中断し報告すること
- [x] `v40000_tests` の閉じ `}` の行番号を確認し記録: 行44295
- [x] `driver.rs` に `v40100_tests` モジュールが存在しないことを確認（今回新規作成）

---

## T1 — runes/stream/rune.toml 作成

- [x] `runes/stream/` ディレクトリ確認（既存）
- [x] `runes/stream/rune.toml` 作成
  - `[rune]` セクション: name=stream, version=40.1.0, entry=stream.fav

---

## T2 — runes/stream/stream.fav 更新

- [x] `runes/stream/stream.fav` に `tumbling_window(stream, size)` 関数スタブ追加
- [x] `runes/stream/stream.fav` に `sliding_window(stream, size, step)` 関数スタブ追加
- [x] 既存関数（map/filter/flat_map/window/merge/split）は変更なし

---

## T3 — Cargo.toml バージョン bump

- [x] `fav/Cargo.toml` の `version = "40.0.0"` → `"40.1.0"` に変更

---

## T4 — CHANGELOG.md 更新

- [x] `[v40.1.0]` エントリを `[v40.0.0]` の直後に追加

---

## T5 — driver.rs 更新

- [x] `v40000_tests::cargo_toml_version_is_40_0_0` をスタブ化
- [x] `v40100_tests` モジュール（3 テスト）を追加
  - `cargo_toml_version_is_40_1_0`
  - `changelog_has_v40_1_0`
  - `stream_rune_has_window_functions`

---

## T6 — テスト実行・確認

- [x] `cargo test` 実行
- [x] failures=0 を確認
- [x] テスト数 ≥ 2817 を確認（実績: 2817）
- [x] `v40100_tests` 3 件すべて pass を確認

---

## T7 — バージョン管理ドキュメント更新

- [x] `versions/current.md` を v40.1.0（最新安定版）・v40.2.0（次に切る版）に更新
- [x] `versions/roadmap/roadmap-v40.1-v41.0.md` の v40.1.0 を完了済みにマーク
- [x] `versions/v40-v45/v40.1.0/tasks.md` を COMPLETE ステータスに更新（全チェックボックス [x]）

---

## コードレビュー指摘と対応

**spec-reviewer** 5 件指摘 → 実装前にすべて対応済み:

**code-reviewer** 3 件 [LOW] 指摘:
- [LOW] stream.fav ヘッダーコメントのバージョンが v26.4.0 のまま → **修正済み**（v40.1.0 + tumbling_window/sliding_window 追記）
- [LOW] sliding_window の `step` 引数が未使用 → スタブ設計上の既知事項（TODO コメントで明記済み）として許容
- [LOW] rune.toml に `effects` フィールド未記載 → llm/rune.toml 等も同様に省略しており現仕様では不要。許容

spec-reviewer 5 件指摘 → 実装前にすべて対応済み:
- [MED] 引数名 seconds vs size → spec.md に注釈追記
- [MED] rune.toml フォーマット統一 → 既存 Rune フォーマット（[rune] + 4 フィールド）に準拠
- [MED] ジェネリクス A のコンパイルリスク → 型なし関数スタブで回避（既存 stream.fav スタイル踏襲）
- [LOW] T0 NOTE コメント確認欠落 → 追加済み
- [LOW] versions/current.md 更新タスク欠落 → T7 追加済み

---

## 最終ステータス

- [x] 全タスク完了
- [x] spec-reviewer 指摘対応済み（5 件 → 全対応）
- [x] code-reviewer 指摘対応済み（実施済み）
