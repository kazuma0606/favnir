# v82.5.0 — スキーマから契約自動生成（`infer_contract`）

Date: 2026-08-20
Status: 計画中

---

## Background

Pipeline Contracts 1.0 スプリントの第 5 版。
v80.7.0 で定義した `SchemaSnapshot`（列名・型名・nullable の集合）から
`IoContract` を自動生成する。これにより既存パイプラインの実測スキーマを
型付き契約に昇格させ、手書きなしで契約駆動開発を始めることができる。

また、`merge_contracts` で手動定義の契約で自動生成分を上書きする仕組みを提供し、
`format_contract_as_toml` でファイル出力用の文字列に変換する。

---

## Goals

1. `infer_field_type_from_str(type_name: &str) -> ContractFieldType` を実装する
   - `"Int"` → `ContractFieldType::Int`
   - `"Float"` → `ContractFieldType::Float`
   - `"Bool"` → `ContractFieldType::Bool`
   - `"Str"` またはその他 → `ContractFieldType::Str`（デフォルト）
2. `infer_contract_from_schema(schema: &SchemaSnapshot, name: &str, version: &str) -> IoContract` を実装する
   - 各 `ColumnSnapshot` を `ContractField` に変換する
     - `nullable: true` → `ContractFieldType::Nullable(Box::new(base_type))`、`required: false`
     - `nullable: false` → base_type をそのまま使用、`required: true`
   - 変換したフィールドを `input` に設定し、`output: vec![]` とする
3. `merge_contracts(base: &IoContract, override_: &IoContract) -> IoContract` を実装する
   - `override_` の input/output フィールドが `base` の同名フィールドを上書きする
   - どちらにしかないフィールドはそのまま残す
   - `name` / `version` は `override_` の値を使用する
4. `format_contract_as_toml(contract: &IoContract) -> String` を実装する
   - TOML ライクな文字列を返す（実際の TOML パーサーは使わない）

---

## API Examples（Rust テストコード）

```rust
// infer_field_type_from_str
assert!(matches!(infer_field_type_from_str("Int"), ContractFieldType::Int));
assert!(matches!(infer_field_type_from_str("Float"), ContractFieldType::Float));
assert!(matches!(infer_field_type_from_str("unknown"), ContractFieldType::Str));

// infer_contract_from_schema
let schema = SchemaSnapshot {
    pipeline_name: "orders".into(),
    columns: vec![
        ColumnSnapshot { name: "id".into(), type_name: "Int".into(), nullable: false },
        ColumnSnapshot { name: "note".into(), type_name: "Str".into(), nullable: true },
    ],
};
let contract = infer_contract_from_schema(&schema, "orders_contract", "1.0.0");
assert_eq!(contract.name, "orders_contract");
assert_eq!(contract.input.len(), 2);
assert_eq!(contract.input[0].field_type, ContractFieldType::Int);
assert!(contract.input[0].required);
assert!(matches!(contract.input[1].field_type, ContractFieldType::Nullable(_)));
assert!(!contract.input[1].required);

// format_contract_as_toml
let toml_str = format_contract_as_toml(&contract);
assert!(toml_str.contains("orders_contract"));
assert!(toml_str.contains("id"));

// merge_contracts: override_ が base を上書き
let base = IoContract { name: "base".into(), version: "1.0.0".into(), input: vec![
    ContractField { name: "id".into(), field_type: ContractFieldType::Str, required: true },
], output: vec![] };
let override_ = IoContract { name: "merged".into(), version: "2.0.0".into(), input: vec![
    ContractField { name: "id".into(), field_type: ContractFieldType::Int, required: true }, // 上書き
], output: vec![] };
let merged = merge_contracts(&base, &override_);
assert_eq!(merged.name, "merged");
assert_eq!(merged.input[0].field_type, ContractFieldType::Int); // override_ が優先
```

### `format_contract_as_toml` の出力形式

```toml
[contract]
name = "orders_contract"
version = "1.0.0"

[[input]]
name = "id"
type = "Int"
required = true

[[input]]
name = "note"
type = "Nullable(Str)"
required = false
```

- `ContractFieldType::Nullable(inner)` の型名は `"Nullable({inner_type_name})"` 形式で出力する

### `merge_contracts` のマージロジック

- 同じロジックを `input` と `output` の両方に独立して適用する
- 同名フィールドは `override_` の値を優先（`input` / `output` それぞれ独立して処理）
- どちらにしかないフィールドはそのまま残す（union）
- 順序: base の順を維持し、override_ にしかないフィールドを末尾に追加
- `name` / `version` は `override_` の値を使用

---

## Success Criteria

- `cargo test` 全 pass（3,875 tests = 3,873 + 2）
- 新規テスト 2 件（`v82500_tests` モジュール）:
  - `contract_inferred_from_schema`
  - `contract_formatted_as_toml`

---

## Files to Modify

| ファイル | 変更内容 |
|---|---|
| `fav/src/test_framework.rs` | `infer_field_type_from_str` / `infer_contract_from_schema` / `merge_contracts` / `format_contract_as_toml` を追加 |
| `fav/src/driver.rs` | `#[cfg(test)] mod v82500_tests` を追加（テスト 2 件） |
| `CHANGELOG.md` | v82.5.0 エントリを先頭に追加 |
