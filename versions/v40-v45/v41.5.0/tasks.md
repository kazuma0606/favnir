# v41.5.0 タスクリスト

**ステータス**: COMPLETE
**目標テスト数**: 2862（前バージョン 2859 + 3）
**実績テスト数**: 2862

---

## T0 — 事前確認

- [x] `cargo test` が 2859 tests / 0 failures であることを確認
- [x] `fav/Cargo.toml` version が `41.4.0` であることを確認
- [x] `versions/roadmap/roadmap-v41.1-v42.0.md` §v41.5.0 を確認
- [x] `v41400_tests::cargo_toml_version_is_41_4_0` が NOTE コメント付きライブアサーションであることを確認し行番号を記録: ___
- [x] NOTE コメントが欠落している場合は実装を中断し報告すること
- [x] `ast_lower_checker.rs` の `RecordSpread` lowering 行番号を確認: ___ （参考: 381行付近）
- [x] `ast_lower_checker.rs` の `lower_te` の `RecordType` 行番号を確認: ___ （参考: 153行付近）
- [x] `checker.fav` の `ERecordLit(String, Expr)` 行番号を確認: ___ （参考: 45行付近）
- [x] `checker.fav` の `TeFn(TypeExpr, TypeExpr)` 行番号を確認: ___ （参考: 63行付近）
- [x] `checker.fav` の `infer_expr` 内 `ERecordLit` ケース行番号を確認: ___ （参考: 1863行付近）
- [x] `checker.fav` の `type_expr_to_str` の `TeFn` ケース行番号を確認
- [x] `checker.fav` の `collect_type_vars_from_te` の `TeMap` ケース行番号を確認
- [x] `lower_field_list` 関数が `ast_lower_checker.rs` に存在することを確認（ERecordSpread で使用）
- [x] `v0` / `v2` ヘルパーが `ast_lower_checker.rs` に存在することを確認

---

## T1 — ast_lower_checker.rs: RecordSpread lowering 修正

- [x] `RecordSpread(_, _, _)` のアームを以下に変更:
  ```rust
  ast::Expr::RecordSpread(base, updates, _) => {
      // v41.5.0: lower to ERecordSpread(base, fields)
      v2("ERecordSpread", lower_expr(base), lower_field_list(updates))
  }
  ```

---

## T2 — ast_lower_checker.rs: RecordType lowering 変更

- [x] `lower_te` の `RecordType` アームを以下に変更:
  ```rust
  ast::TypeExpr::RecordType(_, _) => v0("TeRecord"),
  ```
- [x] `te_to_string` ヘルパー（Rust側文字列変換）の `RecordType` アームは **変更しない**（`"Any"` のまま）

---

## T3 — checker.fav: ERecordSpread バリアント追加

- [x] `ERecordLit(String, Expr)` の直後に追加:
  ```favnir
  | ERecordSpread(Expr, Expr)  // v41.5.0: (base_expr, field_list)
  ```

---

## T4 — checker.fav: TeRecord バリアント追加

- [x] `TeFn(TypeExpr, TypeExpr)` の直後（**末尾**）に追加（`TeIntersection` は存在しない）:
  ```favnir
  | TeRecord          // v41.5.0: { field: Type } 型アノテーション
  ```

---

## T5 — checker.fav: infer_expr に ERecordSpread ケース追加

- [x] `ERecordLit` ケースの直後に追加:
  ```favnir
  ERecordSpread({ _0: base, _1: fields }) =>
      // v41.5.0: base 式を評価して E0001 等を伝播させる（型精度は "Unknown" で近似）
      Result.and_then(infer_expr(base, env), |_bty| Result.ok("Unknown"))
  ```
- [x] `infer_hm` が `_ =>` で `infer_expr` にフォールスルーするため、`infer_expr` に ERecordSpread ケースがあれば十分（`infer_hm` への追加不要）

---

## T6 — checker.fav: type_expr_to_str に TeRecord ケース追加（必須・漏れると実行時エラー）

- [x] 末尾に追加:
  ```favnir
  TeRecord => "Any"
  ```

---

## T7 — checker.fav: collect_type_vars_from_te に TeRecord ケース追加（必須・漏れると実行時エラー）

- [x] 末尾に追加:
  ```favnir
  TeRecord => List.empty()
  ```

---

## T8 — driver.rs テストモジュール更新

- [x] `v41400_tests::cargo_toml_version_is_41_4_0` をスタブ化
- [x] `v41500_tests` モジュール（3 テスト）を末尾に追加
  - `cargo_toml_version_is_41_5_0`
  - `changelog_has_v41_5_0`
  - `record_spread_parseable` — `{ ...u, active: true }` (三点ドット) を使用

---

## T9 — Cargo.toml バージョン bump

- [x] `version = "41.4.0"` → `"41.5.0"`

---

## T10 — CHANGELOG.md 更新

- [x] `[v41.5.0]` エントリを `[v41.4.0]` の直前に追加

---

## T11 — テスト実行・確認

- [x] `cargo test` 実行
- [x] failures=0 を確認
- [x] テスト数 ≥ 2862 を確認（実績: 2862）
- [x] `v41500_tests` 3 件すべて pass を確認
- [x] 既存の Option/Result exhaustiveness テストが壊れていないことを確認

---

## T12 — バージョン管理ドキュメント更新

- [x] `versions/current.md` を v41.5.0（最新安定版）・v41.6.0（次に切る版）に更新
- [x] `versions/roadmap/roadmap-v41.1-v42.0.md` の v41.5.0 を完了済みにマーク
- [x] `versions/v40-v45/v41.5.0/tasks.md` を COMPLETE ステータスに更新（全チェックボックス [x]）

---

## コードレビュー指摘と対応

### [MED] TeIntersection が checker.fav TypeExpr に未定義（既存バグ）
- **指摘**: `ast_lower_checker.rs` L152 は `v2("TeIntersection", ...)` を生成するが `checker.fav` の `TypeExpr` に `TeIntersection` variant が存在しない
- **対応**: v41.5.0 の変更範囲外の既存バグ。v42.0 以降で対応予定として記録

### [LOW] ERecordSpread の base 式を評価せず E0001 等を握り潰し
- **指摘**: `infer_expr` の ERecordSpread ケースが `Result.ok("Unknown")` を即時返却し base 式のエラーを伝播しない
- **対応**: `Result.and_then(infer_expr(base, env), |_bty| Result.ok("Unknown"))` に修正 ✅

### [LOW] check_rebind / check_w006_expr が ERecordSpread 内部を走査しない
- **指摘**: ERecordSpread が `_ =>` フォールスルーのため base 式の rebind / W006 を検出できない
- **対応**: 両関数に `ERecordSpread({ _0: base, _1: _fields }) =>` ケースを追加 ✅

---

## 最終ステータス

- [x] 全タスク完了
- [x] spec-reviewer 指摘対応済み
- [x] code-reviewer 指摘対応済み
