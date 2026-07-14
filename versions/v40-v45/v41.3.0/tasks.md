# v41.3.0 タスクリスト

**ステータス**: COMPLETE
**目標テスト数**: 2856（前バージョン 2853 + 3）
**実績テスト数**: 2856

---

## T0 — 事前確認

- [x] `cargo test` が 2853 tests / 0 failures であることを確認
- [x] `fav/Cargo.toml` version が `41.2.0` であることを確認
- [x] `versions/roadmap/roadmap-v41.1-v42.0.md` §v41.3.0 を確認
- [x] `v41200_tests::cargo_toml_version_is_41_2_0` が NOTE コメント付きライブアサーションであることを確認し行番号を記録
- [x] NOTE コメントが欠落している場合は実装を中断し報告すること
- [x] `v41200_tests` の閉じ `}` の行番号を確認し記録
- [x] `driver.rs` に `v41300_tests` モジュールが存在しないことを確認（今回新規作成）
- [x] `parser.rs` の式側 `LParen` 分岐の行番号を確認し記録（2936 行付近）
- [x] `parser.rs` の `parse_pattern` 内 `LParen` 分岐の行番号を確認し記録（2578 行付近）
- [x] `(expr)` が現在グルーピング括弧として動作していることを確認（変更前確認）
- [x] `(p)` がパターンとして現在パースエラーになることを確認（`()` unit のみサポート）
- [x] `Parser::parse_str` が driver.rs の既存テスト（v41100_tests）で使用済みであることを確認
- [x] `Expr::RecordConstruct` のバリアント定義（String, Vec<(String, Expr)>, Span の 3 引数）を確認
- [x] `Variant::Tuple` が ast.rs に存在することを確認（型定義用 enum — Pattern とは別）

---

## T1 — parser.rs: 式側タプルデシュガー

- [x] 式側 `LParen` 分岐を修正
  - `()` → unit `Lit::Unit`（既存動作維持）
  - `(expr)` → グルーピング括弧（既存動作維持）
  - `(a, b, ...)` → `RecordConstruct("__tuple__", [("_0", a), ("_1", b), ...])` ← 新規
  - 末尾カンマ `(a, b,)` 許容

---

## T2 — parser.rs: パターン側タプルデシュガー

- [x] パターン側 `LParen` 分岐を修正
  - `()` → unit `Lit::Unit`（既存動作維持）
  - `(pat)` → グルーピング括弧（`pat` を返す）← 新規（既存は unit のみ）
  - `(p1, p2, ...)` → `Pattern::Record([Alias("_0", p1), Alias("_1", p2), ...])` ← 新規
  - 末尾カンマ許容

---

## T3 — checker.fav: デシュガー設計コメント追加

- [x] ファイル**末尾**に設計コメントを追加（spec §3 の文面に従う）
  - コードレビュー指摘 [LOW] 対応: `ast_lower_checker.rs` → `checker.rs の lower_pattern` に修正

---

## T4 — Cargo.toml バージョン bump

- [x] `fav/Cargo.toml` の `version = "41.2.0"` → `"41.3.0"` に変更

---

## T5 — CHANGELOG.md 更新

- [x] `[v41.3.0]` エントリを `[v41.2.0]` の直後に追加

---

## T6 — driver.rs テストモジュール更新

- [x] `v41200_tests::cargo_toml_version_is_41_2_0` をスタブ化
- [x] `v41300_tests` モジュール（3 テスト）を末尾に追加
  - `cargo_toml_version_is_41_3_0`（NOTE コメント付き）
  - `changelog_has_v41_3_0`
  - `tuple_pattern_match_parseable`（`=>` FatArrow を使用）

---

## T7 — テスト実行・確認

- [x] `cargo test` 実行
- [x] failures=0 を確認
- [x] テスト数 ≥ 2856 を確認（実績: 2856）
- [x] `v41300_tests` 3 件すべて pass を確認
- [x] `(a, b)` 式と `(p1, p2)` パターンが既存テストを壊していないことを確認
- [x] `fav check` で E0102 が出ることは既知の制限（スコープ外）であることを確認

---

## T8 — バージョン管理ドキュメント更新

- [x] `versions/current.md` を v41.3.0（最新安定版）・v41.4.0（次に切る版）に更新
- [x] `versions/roadmap/roadmap-v41.1-v42.0.md` の v41.3.0 を完了済みにマーク
- [x] `versions/v40-v45/v41.3.0/tasks.md` を COMPLETE ステータスに更新（全チェックボックス [x]）

---

## コードレビュー指摘と対応

| 優先度 | 内容 | 対応 |
|---|---|---|
| [MED] | `emit_python.rs` の `RecordConstruct("__tuple__", ...)` が Python の `__tuple__(a, b)` に変換されてしまう | `emit_python.rs` 行 462–468 に `ty_name == "__tuple__"` 分岐を追加し Python tuple リテラル `(a, b)` を出力するよう修正 |
| [LOW] | `checker.fav` コメントが存在しないファイル名 `ast_lower_checker.rs` を参照 | `checker.rs の lower_pattern` に修正 |

---

## 最終ステータス

- [x] 全タスク完了
- [x] spec-reviewer 指摘対応済み（[HIGH] 3件・[MED] 3件・[LOW] 2件 → 全対応）
- [x] code-reviewer 指摘対応済み（[MED] 1件・[LOW] 1件 → 全対応）
