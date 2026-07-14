# v41.7.0 タスクリスト

**ステータス**: COMPLETE
**目標テスト数**: 2867（前バージョン 2865 + 2）
**実績テスト数**: 2867

---

## T0 — 事前確認

- [x] `cargo test` が 2865 tests / 0 failures であることを確認
- [x] `fav/Cargo.toml` version が `41.6.0` であることを確認
- [x] `versions/roadmap/roadmap-v41.1-v42.0.md` §v41.7.0 を確認
- [x] `v41600_tests::cargo_toml_version_is_41_6_0` が NOTE コメント付きライブアサーションであることを確認し行番号を記録: 44650
- [x] NOTE コメントが欠落している場合は実装を中断し報告すること
- [x] `lint.rs` の `lint_program` 関数の構造（最後の check 呼び出し）を確認
- [x] `lint.rs` に `use std::collections::{HashMap, HashSet}` が存在することを確認（line 18）
- [x] `ast.rs` の `Stmt` enum を確認 → `Stmt::Expr(Expr)` が正しい
- [x] `ast.rs` の `Lit` enum を確認 → `Lit::Int(i64)` 単フィールド
- [x] `ast.rs` の `Block` 構造体を確認 → `expr: Box<Expr>`（return_expr は存在しない）
- [x] `ast.rs` の `TypeExpr::Named` の構造を確認 → `Named(String, Vec<TypeExpr>, Span)` 3 フィールド
- [x] **確認済み前提（変更不要）**: `Block.expr: Box<Expr>`（return_expr フィールドは存在しない）
- [x] **確認済み前提（変更不要）**: `Lit::Int(i64)` 単フィールド（`Lit::Int(x, _)` は誤り）
- [x] **確認済み前提（変更不要）**: `TypeExpr::Named(String, Vec<TypeExpr>, Span)` — 3 フィールド
- [x] **確認済み前提（変更不要）**: `Stmt::Expr(Expr::If(...))` が正しいパターン（`Stmt::If` は存在しない）

---

## T1 — lint.rs: `collect_refinement_aliases` 追加

- [x] `// ── tests` ブロックの直前に `collect_refinement_aliases` を追加
- [x] `HashMap` import は `use std::collections::{HashMap, HashSet};`（line 18）で既存 — 追加不要
- [x] `TypeBody::Alias` + `invariants` 非空 + `Closure` + `BinOp` の 4 条件を確認して登録

---

## T2 — lint.rs: `exprs_lit_eq` / `check_w030_cond` / `check_w030_fn` 追加

- [x] `exprs_lit_eq` を追加（`Lit::Int(x)`, `Lit::Float(x)` — 単フィールド形式）
- [x] `check_w030_cond` を追加（`param op literal` と `literal op param` の両パターン）
- [x] `check_w030_fn` を追加（params マップ構築 + stmts 走査 + `fd.body.expr.as_ref()` 走査）
- [x] `TypeExpr::Named(type_name, _, _)` — 3 フィールドでマッチ

---

## T3 — lint.rs: `check_w030_redundant_refinement_guard` + `lint_program` 組み込み

- [x] `check_w030_redundant_refinement_guard` を追加（コメント `// ── W030` 付き）
- [x] `lint_program` に `// v41.7.0: W030` コメントとともに呼び出しを追加
- [x] コンパイルエラー修正: `if_op == *inv_op` → `*if_op == *inv_op`（`BinOp` の `PartialEq` 問題）

---

## T4 — driver.rs テストモジュール更新

- [x] `v41600_tests::cargo_toml_version_is_41_6_0` をスタブ化（"Stubbed: version bumped to 41.7.0"）
- [x] `v41700_tests` モジュール（2 テスト）を追加:
  - `cargo_toml_version_is_41_7_0`（NOTE コメント付き）
  - `lint_w030_redundant_guard_detected`（`lint_program` 使用）

---

## T5 — Cargo.toml バージョン bump

- [x] `version = "41.6.0"` → `"41.7.0"`

---

## T6 — CHANGELOG.md 更新

- [x] `[v41.7.0]` エントリを `[v41.6.0]` の直前に追加

---

## T7 — テスト実行・確認

- [x] `cargo test` 実行
- [x] failures=0 を確認
- [x] テスト数 ≥ 2867 を確認（実績: 2867）
- [x] `v41700_tests` 2 件すべて pass を確認
- [x] 既存テストが壊れていないことを確認

---

## T8 — バージョン管理ドキュメント更新

- [x] `versions/current.md` を v41.7.0（最新安定版）・v41.8.0（次に切る版）に更新
- [x] `versions/roadmap/roadmap-v41.1-v42.0.md` の v41.7.0 を完了済みにマーク
- [x] `versions/v40-v45/v41.7.0/tasks.md` を COMPLETE ステータスに更新（全チェックボックス [x]）
- [x] **MILESTONE.md 更新**: 本バージョンは機能リリース（非マイルストーン宣言）のため不要

---

## コードレビュー指摘と対応

### [実装時修正] `*if_op == *inv_op` の参照外し
- **問題**: `if_op: &BinOp` に対して `inv_op: &BinOp` を `==` で比較する際に `BinOp` の `PartialEq<&BinOp>` が未実装でコンパイルエラー
- **対応**: `*if_op == *inv_op` に修正 ✅

### [HIGH] `f64 ==` 比較問題（`exprs_lit_eq`）
- **指摘**: `NaN != NaN`・`-0.0 == 0.0` 問題
- **対応**: `x.to_bits() == y.to_bits()` に変更 ✅

### [HIGH] "literal op param" 分岐がデッドコード（演算子対称性未考慮）
- **指摘**: `if 0 <= x` のような書き方で W030 が検出されない。`matches!(inv_rhs.as_ref(), Expr::Ident(...))` が典型的な invariant `|v| v >= 0` では false になり、分岐全体がデッドコードだった
- **対応**: `flip_binop` ヘルパーを追加し、`flip_binop(if_op) == Some(inv_op.clone()) && exprs_lit_eq(lhs, inv_rhs)` に変更 ✅

### [LOW] `invariants[0]` コメント欠落
- **指摘**: 複数 invariant がある場合に 2 番目以降が無視されることが未記載
- **対応**: コメント追加 ✅

---

## 最終ステータス

- [x] 全タスク完了
- [x] spec-reviewer 指摘対応済み（plan.md 修正: `Block.return_expr` → `Block.expr`、`Lit::Int(x,_)` → `Lit::Int(x)`、`TypeExpr::Named` 3 フィールド）
- [x] コンパイルエラー修正済み
- [x] code-reviewer 指摘対応済み（f64 to_bits 比較・flip_binop 追加・コメント追加）
