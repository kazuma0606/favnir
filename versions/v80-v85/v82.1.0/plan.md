# v82.1.0 実装計画

## 方針

**前提**: v82.0.0 完了済み（3,865 tests pass）。

`test_framework.rs` に新型・関数を追加し、`driver.rs` に `v82100_tests` モジュールを追加する。

---

## 実装ステップ

### Step 1: `ContractFieldType` enum を追加

`fav/src/test_framework.rs` の `// ── v82.1.0` コメントから開始。

```rust
/// パイプライン契約フィールドの型。
#[derive(Debug, Clone, PartialEq)]
pub enum ContractFieldType {
    Str,
    Int,
    Float,
    Bool,
    Nullable(Box<ContractFieldType>),
    List(Box<ContractFieldType>),
}
```

### Step 2: `ContractField` 構造体を追加

```rust
/// IoContract の個々のフィールド定義。
#[derive(Debug, Clone, PartialEq)]
pub struct ContractField {
    pub name: String,
    pub field_type: ContractFieldType,
    pub required: bool,
}
```

### Step 3: `IoContract` 構造体を追加

```rust
/// パイプラインの入出力インターフェース契約。
#[derive(Debug, Clone)]
pub struct IoContract {
    pub name: String,
    pub version: String,
    pub input: Vec<ContractField>,
    pub output: Vec<ContractField>,
}
```

### Step 4: `ContractValidationResult` 構造体を追加

```rust
/// IoContract 検証結果。
#[derive(Debug)]
pub struct ContractValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
}
```

### Step 5: `validate_io_contract` 関数を実装

```rust
/// `actual_input` が `contract.input` の必須フィールドをすべて含むかを検証する。
///
/// - required=true のフィールドが actual_input に存在しない場合は errors に追加
/// - フィールド名の照合（field_type の照合は本バージョンでは非対応）
pub fn validate_io_contract(
    contract: &IoContract,
    actual_input: &[ContractField],
) -> ContractValidationResult {
    let mut errors = Vec::new();
    for expected in &contract.input {
        if expected.required {
            let found = actual_input.iter().any(|f| f.name == expected.name);
            if !found {
                errors.push(format!("missing required field: {}", expected.name));
            }
        }
    }
    ContractValidationResult {
        valid: errors.is_empty(),
        errors,
    }
}
```

### Step 6: テストモジュール追加（driver.rs）

`fav/src/driver.rs` 末尾に `#[cfg(test)] mod v82100_tests` を追加する。

- `io_contract_validates_matching_fields`: 全必須フィールドが存在する場合 `valid=true`・`errors.is_empty()`
- `io_contract_fails_on_missing_required_field`: 必須フィールド欠損の場合 `valid=false`・`errors` に対象フィールド名が含まれる

### Step 7: `cargo test` 全通過確認

3,867 tests pass（+2）、0 failures であることを確認する。

### Step 8: CHANGELOG 更新

`CHANGELOG.md` の先頭に v82.1.0 エントリを追加する。
