# Favnir v6.6.0 Tasks

Date: 2026-05-27

## Goal

`T.validate` を完全実装する。
`one_of` 制約追加・`TypeName.validate` VM dispatch・`Validate.rows_raw` 追加・統合テスト 10 件。

## Phase A — `one_of` 制約の追加

- [x] A-1: `fav/src/schemas.rs` — `FieldConstraints` に `pub one_of: Option<Vec<String>>` を追加
- [x] A-2: `schemas.rs` — `serde(Deserialize)` 由来で YAML の `one_of: [...]` が自動パースされる
- [x] A-3: `fav/src/backend/vm.rs` — `validate_record_inner` に `one_of` 違反チェックを追加
- [x] A-4: `cargo test validate` 通過確認

## Phase B — `TypeName.validate` VM dispatch

- [x] B-1: `vm.rs` — `Validate.run_raw` ロジックを `fn validate_record_inner` に抽出（共通化）
- [x] B-2: `vm.rs` — `GetField` / `GetFieldC` に `VMValue::VariantCtor(ns)` → `VMValue::Builtin("ns.field")` 変換を追加
- [x] B-3: `vm.rs` の `call_builtin` に `name.ends_with(".validate")` による動的 dispatch を追加
- [x] B-4: テスト: `Order.validate(raw)` 構文が Ok / Err を正しく返すことを確認（D-3/D-4 で確認済み）

## Phase C — `Validate.rows_raw` 追加

- [x] C-1: `vm.rs` — `Validate.rows_raw(type_name, rows)` builtin を追加（違反があれば最初の Err を返す）
- [x] C-2: `checker.rs` — `Validate.rows_raw` の型シグネチャを追加
- [x] C-3: 後方互換: スキーマなし型は従来通り Ok を返す
- [x] C-4: Note: `duckdb.query<T>` の自動バリデーション（rune 層変更が必要）は v6.7.0 以降に持ち越し

## Phase D — 統合テスト（10 件）

`fav/src/backend/vm_stdlib_tests.rs` に追加:

- [x] D-1: `validate_one_of_valid` — one_of 許容値 → Ok
- [x] D-2: `validate_one_of_violation` — one_of 違反値 → Err
- [x] D-3: `validate_type_dot_validate_ok` — `Order.validate(raw)` 構文 → Ok
- [x] D-4: `validate_type_dot_validate_err` — `Order.validate(raw)` 構文 → Err
- [x] D-5: `validate_required_field_missing` — nullable でないフィールド欠落 → Err
- [x] D-6: `validate_nullable_field_ok` — nullable フィールドが None → Ok
- [x] D-7: `validate_pattern_valid` — pattern 制約の許容値 → Ok
- [x] D-8: `validate_multi_constraint_all_pass` — 複数制約すべて通過 → Ok
- [x] D-9: `validate_multi_constraint_partial_fail` — 複数制約の一部失敗 → Err
- [x] D-10: `validate_rows_raw_schema_violation` — Validate.rows_raw + 違反データ → Err

## Phase E — ドキュメント更新

- [x] E-1: `site/content/docs/language/schema.mdx` — `T.validate` の "preview" Note を削除
- [x] E-2: `site/content/docs/language/schema.mdx` — `Order.validate(raw)` 構文の例に更新、`Validate.rows_raw` 追記
- [x] E-3: `site/content/docs/stdlib/infer.mdx` — `Validate.rows_raw` を使ったランタイム検証の説明を追記

## Phase F — 最終検証

- [x] F-1: `cargo test` 全テスト通過（1043 件 = 1033 件 + 新規 10 件）
- [x] F-2: このファイルを完了状態に更新

## 完了条件まとめ

- `one_of` 制約が schemas.rs / vm.rs で動作する ✓
- `TypeName.validate(record)` が VM で実行でき `Result<T, List<ValidationError>>` を返す ✓
- `Validate.rows_raw(type_name, rows)` が追加され一括検証が可能 ✓
- 統合テスト 10 件すべて通過（計 1043 件） ✓
- `schema.mdx` から "preview" 表記が消えている ✓
