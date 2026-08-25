# v82.5.0 実装計画

## 方針

**前提**: v82.4.0 完了済み（3,873 tests pass）。

`test_framework.rs` にスキーマ→契約変換関数を追加し、`driver.rs` に `v82500_tests` を追加する。
`SchemaSnapshot` / `ColumnSnapshot` は v80.7.0 で定義済み（line 376〜387）。
`IoContract` / `ContractField` / `ContractFieldType` は v82.1.0 で定義済み。

---

## 実装ステップ

### Step 1: `infer_field_type_from_str` を追加

`fav/src/test_framework.rs` の v82.4.0 セクション末尾に続けて追加する。

```rust
// ── v82.5.0: infer_contract / merge_contracts / format_contract_as_toml ───────

/// 型名文字列から `ContractFieldType` を推論する。
///
/// - `"Int"` → `Int`、`"Float"` → `Float`、`"Bool"` → `Bool`
/// - それ以外（`"Str"` / 未知の型）→ `Str`（デフォルト）
pub fn infer_field_type_from_str(type_name: &str) -> ContractFieldType {
    match type_name {
        "Int" => ContractFieldType::Int,
        "Float" => ContractFieldType::Float,
        "Bool" => ContractFieldType::Bool,
        _ => ContractFieldType::Str,
    }
}
```

### Step 2: `infer_contract_from_schema` を追加

```rust
/// `SchemaSnapshot` から `IoContract` を自動生成する。
///
/// - `nullable: false` の列 → base type、`required: true`
/// - `nullable: true` の列 → `Nullable(base_type)`、`required: false`
/// - すべての列を `input` に設定し、`output` は空とする
pub fn infer_contract_from_schema(
    schema: &SchemaSnapshot,
    name: &str,
    version: &str,
) -> IoContract {
    let input = schema
        .columns
        .iter()
        .map(|col| {
            let base_type = infer_field_type_from_str(&col.type_name);
            let field_type = if col.nullable {
                ContractFieldType::Nullable(Box::new(base_type))
            } else {
                base_type
            };
            ContractField {
                name: col.name.clone(),
                field_type,
                required: !col.nullable,
            }
        })
        .collect();
    IoContract {
        name: name.into(),
        version: version.into(),
        input,
        output: vec![],
    }
}
```

### Step 3: `merge_contracts` を追加

```rust
/// 2 つの `IoContract` をマージする。`override_` が `base` を上書きする。
///
/// - 同名フィールドは `override_` の値を使用
/// - `base` にしかないフィールドはそのまま残す（base の順を維持）
/// - `override_` にしかないフィールドは末尾に追加
/// - `name` / `version` は `override_` の値を使用
pub fn merge_contracts(base: &IoContract, override_: &IoContract) -> IoContract {
    let merge_fields = |base_fields: &[ContractField], override_fields: &[ContractField]| {
        let mut result: Vec<ContractField> = base_fields
            .iter()
            .map(|bf| {
                override_fields
                    .iter()
                    .find(|of| of.name == bf.name)
                    .cloned()
                    .unwrap_or_else(|| bf.clone())
            })
            .collect();
        for of in override_fields {
            if !base_fields.iter().any(|bf| bf.name == of.name) {
                result.push(of.clone());
            }
        }
        result
    };
    IoContract {
        name: override_.name.clone(),
        version: override_.version.clone(),
        input: merge_fields(&base.input, &override_.input),
        output: merge_fields(&base.output, &override_.output),
    }
}
```

### Step 4: `format_contract_as_toml` を追加

`ContractFieldType` を型名文字列に変換するヘルパー `field_type_to_str` を内部関数として使う。

```rust
/// `IoContract` を TOML ライクな文字列に変換する（toml クレート不使用）。
pub fn format_contract_as_toml(contract: &IoContract) -> String {
    fn field_type_to_str(ft: &ContractFieldType) -> String {
        match ft {
            ContractFieldType::Str => "Str".into(),
            ContractFieldType::Int => "Int".into(),
            ContractFieldType::Float => "Float".into(),
            ContractFieldType::Bool => "Bool".into(),
            ContractFieldType::Nullable(inner) => format!("Nullable({})", field_type_to_str(inner)),
            ContractFieldType::List(inner) => format!("List({})", field_type_to_str(inner)),
        }
    }
    let mut lines = vec![
        "[contract]".into(),
        format!("name = \"{}\"", contract.name),
        format!("version = \"{}\"", contract.version),
    ];
    for field in &contract.input {
        lines.push(String::new());
        lines.push("[[input]]".into());
        lines.push(format!("name = \"{}\"", field.name));
        lines.push(format!("type = \"{}\"", field_type_to_str(&field.field_type)));
        lines.push(format!("required = {}", field.required));
    }
    for field in &contract.output {
        lines.push(String::new());
        lines.push("[[output]]".into());
        lines.push(format!("name = \"{}\"", field.name));
        lines.push(format!("type = \"{}\"", field_type_to_str(&field.field_type)));
        lines.push(format!("required = {}", field.required));
    }
    lines.join("\n")
}
```

### Step 5: CHANGELOG 更新

`CHANGELOG.md` の先頭に v82.5.0 エントリを追加する。

### Step 6: `v82500_tests` テストモジュール追加（driver.rs）

`fav/src/driver.rs` 末尾に `#[cfg(test)] mod v82500_tests` を追加する。

- `contract_inferred_from_schema`:
  - `SchemaSnapshot` に nullable/non-nullable 列を含めて `infer_contract_from_schema` を呼ぶ
  - input フィールド数・型・required が正しいことを確認
- `contract_formatted_as_toml`:
  - `format_contract_as_toml` の出力にコントラクト名・フィールド名・型名が含まれることを確認

### Step 7: `cargo test` 全通過確認

3,875 tests pass（+2）、0 failures であることを確認する。
