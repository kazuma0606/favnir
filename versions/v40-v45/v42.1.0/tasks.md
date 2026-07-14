# v42.1.0 タスクリスト

**ステータス**: COMPLETE
**目標テスト数**: 2877（前バージョン 2874 + 3）
**実績テスト数**: 2877（v42100_tests 3/3 PASS）

---

## T0 — 事前確認

- [x] `cargo test` が 2874 tests / 0 failures であることを確認
- [x] `fav/Cargo.toml` version が `42.0.0` であることを確認
- [x] `versions/roadmap/roadmap-v42.1-v43.0.md` §v42.1.0 を確認
- [x] `v42000_tests::cargo_toml_version_is_42_0_0` が NOTE コメント付きライブアサーションであることを確認し行番号を記録
- [x] NOTE コメントが欠落している場合は実装を中断し報告すること
- [x] `v42000_tests` の閉じ `}` の行番号を確認し記録（`v42100_tests` の挿入位置特定のため）
- [x] `ast.rs` に `CepPatternDef` が存在しないことを確認
- [x] `parser.rs` に `parse_cep_pattern_def` が存在しないことを確認
- [x] `TokenKind::Int` が `i64` を保持していることを確認
- [x] `Span::merge()` が存在しないことを確認（`parse_schema_def` が `span_from` を使っているパターンで代用）
- [x] `Parser::parse_str` が `pub` であることを確認（既存テストが同パターンを使用のため問題ない見込み）
- [x] `versions/current.md` の現在の記述形式を確認

---

## T1 — `ast.rs` 更新

- [x] `CepClause` 構造体を `SchemaDef` の直後に追加（`event: String` / `within_secs: Option<i64>` / `span: Span`）
- [x] `CepPatternDef` 構造体を `CepClause` の直後に追加（`name: String` / `body: Vec<CepClause>` / `span: Span`）
- [x] `Item` enum に `CepPatternDef(CepPatternDef)` バリアントを `SchemaDef` の直後に追加
- [x] `Item::span()` の exhaustive match に `Item::CepPatternDef(c) => &c.span` を追加
- [x] `cargo build` でコンパイルエラーがないことを確認（fmt.rs / driver.rs 等の exhaustive match が壊れる前兆）

---

## T2 — `parser.rs` 更新

- [x] `parse_cep_pattern_def()` 関数を `parse_schema_def()` の直後に追加
  - `cep` consume → `pattern` keyword check → name ident → `{` → clause ループ → `}`
  - clause: event ident + optional `within N`
  - `within` 後の整数を `within_secs: Some(n)` に格納
- [x] `parse_item()` に `"cep"` dispatch を追加（`"schema"` ブロックの付近）
- [x] エラーメッセージの item リストに `cep` を追加
- [x] `cargo test cep_pattern_parseable` が pass することを確認

---

## T3 — `checker.rs` スタブ

- [x] Pass 1（line 2368 付近）: `| Item::CepPatternDef(..)` を `SchemaDef` の行に追加
- [x] Pass 2（line 2411 付近）: `Item::CepPatternDef(_) => {}` を `SchemaDef` の直後に追加

---

## T4 — `fmt.rs` スタブ

- [x] exhaustive match の `Item::SchemaDef` arm の直後に `Item::CepPatternDef(cd) => format!(...)` を追加

---

## T5 — `driver.rs` 更新

- [x] line 13898 付近の `Item::SchemaDef(..) => {}` の直後に `Item::CepPatternDef(..) => {}` を追加
- [x] `v42000_tests::cargo_toml_version_is_42_0_0` をスタブ化（"Stubbed: version bumped to 42.1.0"）
- [x] `v42100_tests` モジュール（3 テスト）を `v42000_tests` の直前に追加:
  - `cargo_toml_version_is_42_1_0`（NOTE コメント付き）
  - `cep_pattern_parseable`
  - `cep_pattern_fields_correct`

---

## T6 — `checker.fav` 設計コメント

- [x] `fav/self/checker.fav` の末尾に CEP 設計コメントブロック追加（v42.3.0 向け）

---

## T7 — Cargo.toml バージョン bump

- [x] `version = "42.0.0"` → `"42.1.0"`

---

## T8 — CHANGELOG.md 更新

- [x] `[v42.1.0]` エントリを `[v42.0.0]` の直前に追加

---

## T9 — テスト実行・確認

- [x] `cargo test` 実行
- [x] failures=0 を確認
- [x] テスト数 = 2877 を確認（2874 + 3 件）
- [x] `v42100_tests` 3 件 pass を確認
- [x] 既存テストが壊れていないことを確認

---

## T10 — バージョン管理ドキュメント更新

- [x] `versions/current.md` を v42.1.0（最新安定版）・v42.2.0（次に切る版）に更新
- [x] `versions/roadmap/roadmap-v42.1-v43.0.md` の v42.1.0 を完了済みにマーク（`✅ COMPLETE（2026-07-12）` を追記）
- [x] `versions/v40-v45/v42.1.0/tasks.md` を COMPLETE ステータスに更新（全チェックボックス [x]）
- [x] **MILESTONE.md 更新**: 本バージョンは機能リリース（非マイルストーン宣言）のため不要

---

## 最終ステータス

- [x] 全タスク完了

## コードレビュー指摘・対応記録

- spec-reviewer [HIGH] x3: plan.md の `parse_cep_pattern_def()` 実装に 3 つのバグ（`advance()` の戻り値型誤解、`Span::merge()` 非存在、`peek()` が `Option` でない）→ plan.md を正しい実装で修正済み
- spec-reviewer [MED]: `peek().clone()` → match で `TokenKind::Int(n)` 取得のパターン明示 → plan.md に記載済み
