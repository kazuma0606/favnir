# v82.1.0 タスクリスト

Status: COMPLETE

---

## T0: 事前確認

- [x] `cargo test` が 3,865 tests pass、0 failures であることを確認する（前提: v82.0.0 完了済み）

## T1: `ContractFieldType` enum 追加

- [x] `fav/src/test_framework.rs` に `ContractFieldType` enum を追加する
  - variants: `Str` / `Int` / `Float` / `Bool` / `Nullable(Box<ContractFieldType>)` / `List(Box<ContractFieldType>)`
  - `#[derive(Debug, Clone, PartialEq)]` を付与する

## T2: `ContractField` 構造体追加

- [x] `fav/src/test_framework.rs` に `ContractField` 構造体を追加する
  - `name: String` / `field_type: ContractFieldType` / `required: bool`
  - `#[derive(Debug, Clone, PartialEq)]` を付与する

## T3: `IoContract` 構造体追加

- [x] `fav/src/test_framework.rs` に `IoContract` 構造体を追加する
  - `name: String` / `version: String` / `input: Vec<ContractField>` / `output: Vec<ContractField>`
  - `#[derive(Debug, Clone)]` を付与する

## T4: `ContractValidationResult` 構造体 + `validate_io_contract` 関数追加

- [x] `ContractValidationResult` 構造体（`valid: bool`, `errors: Vec<String>`）を追加する
- [x] `validate_io_contract(contract: &IoContract, actual_input: &[ContractField]) -> ContractValidationResult` を実装する
  - required=true のフィールドが actual_input にない場合は `"missing required field: {name}"` を errors に追加
  - `valid = errors.is_empty()`

## T5: CHANGELOG 更新

- [x] `CHANGELOG.md` の先頭に v82.1.0 エントリを追加する

## T6: `v82100_tests` テストモジュール追加

- [x] `fav/src/driver.rs` 末尾に `#[cfg(test)] mod v82100_tests` を追加する
  - `io_contract_validates_matching_fields`: 全必須フィールドが存在 → `valid=true`・`errors.is_empty()` ✅
  - `io_contract_fails_on_missing_required_field`: 必須フィールド欠損 → `valid=false`・errors に "name" を含む ✅

## T7: テスト通過確認

- [x] `cargo test` が 3,867 tests pass（+2）、0 failures であることを確認する

## T8: 最終確認（CI チェック）

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する

## コードレビュー指摘と対応

| 優先度 | 指摘 | 対応 |
|---|---|---|
| [LOW] | `IoContract` に `PartialEq` が未実装 | `#[derive(PartialEq)]` を追加 ✅ |
| [LOW] | `detect_anomaly` の NaN 動作が未ドキュメント | doc comment に NaN 注記を追加 ✅ |
