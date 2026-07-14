# v40.7.0 タスクリスト

**ステータス**: COMPLETE
**目標テスト数**: 2835（前バージョン 2832 + 3）
**実績テスト数**: 2835 passed, 0 failed（2026-07-11）

---

## T0 — 事前確認

- [x] `cargo test` が 2832 tests / 0 failures であることを確認
- [x] `fav/Cargo.toml` version が `40.6.0` であることを確認
- [x] `versions/roadmap/roadmap-v40.1-v41.0.md` §v40.7.0 を確認
- [x] `v40600_tests::cargo_toml_version_is_40_6_0` が NOTE コメント付きライブアサーションであることを確認し行番号を記録: 行44416
- [x] NOTE コメントが欠落している場合は実装を中断し報告すること
- [x] `v40600_tests` の閉じ `}` の行番号を確認し記録: 行44433
- [x] `driver.rs` に `v40700_tests` モジュールが存在しないことを確認（今回新規作成）
- [x] `BenchOpts` に `stream` フィールドが存在しないことを確認

---

## T1 — driver.rs: BenchOpts + cmd_bench 更新

- [x] `BenchOpts` 構造体の `json: bool` フィールドの直後に `pub stream: bool` を追加
- [x] `BenchOpts::default()` 構築部に `stream: false` を追加
- [x] `cmd_bench` 関数先頭に `--stream` スタブ分岐を追加

---

## T2 — main.rs: --stream フラグ解析追加 + ヘルプテキスト更新

- [x] `bench` アームの `"--json"` 解析直後に `"--stream"` アームを追加
- [x] ヘルプテキスト（`bench [--runs <n>] ...` の行）に `[--stream]` を追記・説明行を追加

---

## T3 — Cargo.toml バージョン bump

- [x] `fav/Cargo.toml` の `version = "40.6.0"` → `"40.7.0"` に変更

---

## T4 — CHANGELOG.md 更新

- [x] `[v40.7.0]` エントリを `[v40.6.0]` の直後に追加

---

## T5 — driver.rs テストモジュール更新

- [x] `v40600_tests::cargo_toml_version_is_40_6_0` をスタブ化
- [x] `v40700_tests` モジュール（3 テスト）を末尾に追加（`use super::*` 付き）
  - `cargo_toml_version_is_40_7_0`（NOTE コメント付き）
  - `changelog_has_v40_7_0`
  - `bench_opts_has_stream_field`（NOTE コメントなし）

---

## T6 — テスト実行・確認

- [x] `cargo test` 実行
- [x] failures=0 を確認
- [x] テスト数 ≥ 2835 を確認（実績: 2835）
- [x] `v40700_tests` 3 件すべて pass を確認

---

## T7 — バージョン管理ドキュメント更新

- [x] `versions/current.md` を v40.7.0（最新安定版）・v40.8.0（次に切る版）に更新
- [x] `versions/roadmap/roadmap-v40.1-v41.0.md` の v40.7.0 を完了済みにマーク
- [x] `versions/v40-v45/v40.7.0/tasks.md` を COMPLETE ステータスに更新（全チェックボックス [x]）

---

## コードレビュー指摘と対応

**code-reviewer 指摘（実装後対応）:**
- [LOW] `main.rs` ヘルプテキストに `(v40.7.0 stub)` という実装詳細が露出 → `(v40.7.0 stub)` を削除 ✅

**spec-reviewer 指摘（実装前対応）:**
- [LOW] `bench_opts_has_stream_field` の NOTE コメントが不正確 → NOTE コメントを削除（spec.md・tasks.md 修正）
- [LOW] main.rs ヘルプテキスト更新がスコープ外 → T2 に追加・ヘルプテキストを更新
- [LOW] 完了条件 #3・#4 の検証方法が不明 → 完了条件テーブルに「検証方法」列を追加し手動確認と明記

---

## 最終ステータス

- [x] 全タスク完了
- [x] spec-reviewer 指摘対応済み（3 件 → 全対応）
- [x] code-reviewer 指摘対応済み（1 件 → 対応済み）
