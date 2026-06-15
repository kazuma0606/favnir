# v18.2.0 — 行多相（Row Polymorphism）タスク

## ステータス: 完了

---

## タスク一覧

### T1: `fav/src/ast.rs` — `TypeConstraint` / `Type::Intersection` / `GenericParam.bounds` 変更

- [x] `TypeConstraint` enum を追加（`Interface(String)` / `HasField { name, ty }`）
- [x] `GenericParam.bounds` を `Vec<String>` → `Vec<TypeConstraint>` に変更
- [x] `Type::Intersection(Box<Type>, Box<Type>)` を `Type` enum に追加
- [x] `Type::display()` に `Intersection` case 追加
- [x] `Type::is_compatible()` に `Intersection` case 追加（保守的に通す）
- [x] `TypeExpr::Intersection(Box<TypeExpr>, Box<TypeExpr>)` を `TypeExpr` に追加

### T2: `fav/src/frontend/parser.rs` — `parse_type_bounds` / `parse_type_expr` 拡張

- [x] `parse_type_bounds` を拡張: `with { field: Type, ... }` → `TypeConstraint::HasField` 複数生成
- [x] 既存の `with Ord` → `TypeConstraint::Interface("Ord")` に変換するよう修正
- [x] `parse_type_expr` に `T & U` 交差型の解析を追加（`TypeExpr::Intersection`）
- [x] `TokenKind::Amp` を型文脈で交差型演算子として扱う

### T3: `fav/src/middle/checker.rs` — 行多相チェック

- [x] `resolve_type_expr` に `TypeExpr::Intersection` → `Type::Intersection` への変換を追加
- [x] `validate_type_expr_arity` に `TypeExpr::Intersection` の case 追加
- [x] `type_implements_bound` を `TypeConstraint` 対応に変更
  - [x] `TypeConstraint::Interface(name)` → 既存の bounds チェック
  - [x] `TypeConstraint::HasField { name, ty }` → `type_has_field` を呼ぶ
- [x] `type_has_field(ty: &Type, field_name: &str) -> bool` を追加
  - [x] `Type::Record(fields)` → フィールド名を確認
  - [x] `Type::Named(name)` → `record_fields` から確認
- [x] E0337 エラーを call-site で発行
  - [x] `check_generic_bounds_at_call`（または該当箇所）に `HasField` チェックを追加

### T4: 波及ファイル更新（`GenericParam.bounds` 型変更対応）

- [x] `fav/src/middle/compiler.rs` — `p.bounds` イテレーション箇所を `TypeConstraint` 対応に修正
  - [x] `bounds.any(|p| !p.bounds.is_empty())` → `TypeConstraint` 対応
  - [x] `p.bounds` を `Vec<String>` として参照している箇所を修正
- [x] `fav/src/fmt.rs` — `GenericParam` の pretty-print で `TypeConstraint` を処理
  - [x] `Interface(name)` → `name` 表示
  - [x] `HasField { name, ty }` → `{ name: ty }` 表示
- [x] `fav/src/emit_python.rs` — bounds 処理を `TypeConstraint` 対応に修正
- [x] `fav/src/lineage.rs` — `GenericParam` bounds 参照箇所（あれば）を修正
- [x] `Type::Intersection` の exhaustive match を以下の全箇所に追加:
  - [x] `checker.rs` の `Type` match 箇所
  - [x] `compiler.rs` の型変換箇所
  - [x] `fmt.rs` の `Type::display()`

### T5: `fav/src/driver.rs` — `v182000_tests` 追加

- [x] `v181000_tests` の `version_is_18_1_0` テストを削除
- [x] `v182000_tests` モジュールを追加（5件）:
  - [x] `version_is_18_2_0`
  - [x] `row_poly_single_field`（`fn f<R with { id: Int }>` が型チェックを通る）
  - [x] `row_poly_different_records`（異なるレコード型に同じ関数を適用）
  - [x] `row_poly_intersection_return`（`-> R & { ts: String }` がパースされる）
  - [x] `row_poly_field_missing`（制約フィールドなしで E0337）

### T6: バージョン更新

- [x] `fav/Cargo.toml` のバージョンを `18.1.0` → `18.2.0` に更新
- [x] `cargo build` で `Cargo.lock` 更新

### T7: `site/content/docs/language/row-polymorphism.mdx` 作成

- [x] `fn f<R with { id: Int }>(row: R)` の基本構文を記載
- [x] 交差型 `R & { field: Type }` の説明（値の位置と型の位置の使い分け）を記載
- [x] 複数フィールド制約の例を記載
- [x] パイプラインでの活用例を記載
- [x] E0337 エラーの説明を記載

---

## テスト（v182000_tests、5件）

| テスト名 | 内容 |
|---|---|
| `version_is_18_2_0` | Cargo.toml に "18.2.0" が含まれる |
| `row_poly_single_field` | `fn f<R with { id: Int }>(row: R) -> Int` が型チェックを通る |
| `row_poly_different_records` | 異なるレコード型に同じ関数を適用できる |
| `row_poly_intersection_return` | `-> R & { ts: String }` がパース・型チェックされる |
| `row_poly_field_missing` | 制約フィールドがない型を渡すと E0337 |

---

## 完了条件チェックリスト

- [x] `fav/Cargo.toml` のバージョンが `18.2.0`
- [x] `TypeConstraint` enum が `ast.rs` に存在する
- [x] `GenericParam.bounds` が `Vec<TypeConstraint>` になっている
- [x] `Type::Intersection` が `checker.rs` に存在する
- [x] `fn f<R with { id: Int }>(row: R)` の型チェックが通る
- [x] 制約フィールドなしで E0337 が発行される
- [x] `-> R & { ts: String }` の交差型がパースされる
- [x] `site/content/docs/language/row-polymorphism.mdx` が存在する
- [x] `cargo test v182000` — 5/5 PASS
- [x] `cargo test` — リグレッションなし

---

## 優先度

T1（ast.rs 基盤）                   ← 最初
T2（parser.rs）                     ← T1 完了後
T3（checker.rs）                    ← T1, T2 完了後
T4（波及ファイル）                   ← T1 完了後（T2, T3 と並列可）
→ T5（v182000_tests）              ← T1〜T4 すべて完了後
T6（バージョン更新）                 ← T5 完了後
T7（ドキュメント）                   ← T6 と並列可

**重要**: T1 の `GenericParam.bounds` 変更はコンパイルエラーを多数発生させる。
T4 の波及ファイル更新で全て解消してからテストを実行する。
