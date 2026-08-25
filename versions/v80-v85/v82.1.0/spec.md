# v82.1.0 — `IoContract` / `ContractField` 型基盤

Date: 2026-08-20
Status: 計画中

---

## Background

Pipeline Contracts 1.0 スプリント（v82.1〜v83.0）の第 1 版。
パイプラインの入出力インターフェースを型で定義する基盤型を構築する。

`IoContract` はパイプラインが「何を受け取り、何を出力するか」を型として表現し、
実行前の静的検証（`fav verify --contract`）を可能にする。

---

## Goals

1. `ContractFieldType` enum を定義する（`Str` / `Int` / `Float` / `Bool` / `Nullable(Box<ContractFieldType>)` / `List(Box<ContractFieldType>)`）
2. `ContractField` 構造体を定義する（`name: String`, `field_type: ContractFieldType`, `required: bool`）
3. `IoContract` 構造体を定義する（`name: String`, `version: String`, `input: Vec<ContractField>`, `output: Vec<ContractField>`）
4. `ContractValidationResult` 構造体を定義する（`valid: bool`, `errors: Vec<String>`）
5. `validate_io_contract(contract: &IoContract, actual_input: &[ContractField]) -> ContractValidationResult` を実装する

---

## API Examples

```rust
// ContractField の定義
let field_id = ContractField {
    name: "id".into(),
    field_type: ContractFieldType::Int,
    required: true,
};
let field_name = ContractField {
    name: "name".into(),
    field_type: ContractFieldType::Str,
    required: true,
};

// IoContract の定義
let contract = IoContract {
    name: "orders_pipeline".into(),
    version: "1.0.0".into(),
    input: vec![field_id.clone(), field_name.clone()],
    output: vec![field_id.clone()],
};

// 検証: マッチするケース
let actual = vec![field_id.clone(), field_name.clone()];
let result = validate_io_contract(&contract, &actual);
assert!(result.valid);
assert!(result.errors.is_empty());

// 検証: 必須フィールド欠損ケース
let missing = vec![field_id.clone()]; // name が欠損
let result2 = validate_io_contract(&contract, &missing);
assert!(!result2.valid);
assert!(!result2.errors.is_empty());
```

---

## Success Criteria

- `cargo test` 全 pass（3,867 tests = 3,865 + 2）
- 新規テスト 2 件（`v82100_tests` モジュール）:
  - `io_contract_validates_matching_fields`
  - `io_contract_fails_on_missing_required_field`

> **drift 注記**: ロードマップ（roadmap-v82.1-v83.0.md）は 3,853 + 2 = 3,855 と記載しているが、
> v82.0.0 完了時の実績（3,865）を基準とする。

---

## Files to Modify

| ファイル | 変更内容 |
|---|---|
| `fav/src/test_framework.rs` | `IoContract` / `ContractField` / `ContractFieldType` / `ContractValidationResult` / `validate_io_contract` を追加 |
| `fav/src/driver.rs` | `#[cfg(test)] mod v82100_tests` を追加（テスト 2 件） |
| `CHANGELOG.md` | v82.1.0 エントリを先頭に追加 |
