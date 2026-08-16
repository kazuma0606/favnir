# v73.1.0 実装計画 — データコントラクト

Date: 2026-08-13

---

## 実装ステップ

### T0: 事前確認

1. `fav/Cargo.toml` のバージョンが `73.0.0` であることを確認
2. `cargo test` が 3646 tests pass（0 failures）であることを確認
3. `driver.rs` に `v73000_tests` モジュールが存在することを確認
4. `driver.rs` に `v731000_tests` が未存在であることを確認
5. `driver.rs` 内の `"73.0.0"` 文字列件数を grep で確認しておく

---

### T1: `DataContractField` / `DataContractSla` / `DataContract` 構造体追加

`driver.rs` に以下を追加する（既存構造体の近くに配置）:

```rust
pub struct DataContractField {
    pub name: String,
    pub ty: String,
    pub nullable: bool,
}

pub struct DataContractSla {
    pub max_latency_ms: u64,
    pub min_throughput: u64,
    pub max_error_rate: f64,
}

pub struct DataContract {
    pub name: String,
    pub input_fields: Vec<DataContractField>,
    pub output_fields: Vec<DataContractField>,
    pub sla: DataContractSla,
}
```

確認: `cargo build` でエラーがないことを確認。

---

### T2: `validate_contract_schema` 追加

```rust
pub fn validate_contract_schema(
    contract: &DataContract,
    actual_input: &[(&str, &str)],
) -> Result<(), String> {
    for field in &contract.input_fields {
        match actual_input.iter().find(|(n, _)| *n == field.name.as_str()) {
            None => return Err(format!(
                "schema mismatch: field '{}' missing in actual input", field.name
            )),
            Some((_, actual_ty)) if *actual_ty != field.ty.as_str() => return Err(format!(
                "schema mismatch: field '{}' expected type '{}', got '{}'",
                field.name, field.ty, actual_ty
            )),
            _ => {}
        }
    }
    Ok(())
}
```

確認: `cargo build` でエラーがないことを確認。

---

### T3: `check_sla_compliance` 追加

```rust
pub fn check_sla_compliance(
    sla: &DataContractSla,
    actual_latency_ms: u64,
    actual_throughput: u64,
    actual_error_rate: f64,
) -> Result<(), String> {
    if actual_latency_ms > sla.max_latency_ms {
        return Err(format!(
            "SLA violation: latency {}ms exceeds max {}ms",
            actual_latency_ms, sla.max_latency_ms
        ));
    }
    if actual_throughput < sla.min_throughput {
        return Err(format!(
            "SLA violation: throughput {} rows/sec below min {}",
            actual_throughput, sla.min_throughput
        ));
    }
    if actual_error_rate > sla.max_error_rate {
        return Err(format!(
            "SLA violation: error rate {:.4} exceeds max {:.4}",
            actual_error_rate, sla.max_error_rate
        ));
    }
    Ok(())
}
```

確認: `cargo build` でエラーがないことを確認。

---

### T4: `v731000_tests` モジュール追加

`v73000_tests` モジュールの直後に追加:

```rust
#[cfg(test)]
mod v731000_tests {
    use super::{DataContract, DataContractField, DataContractSla,
                validate_contract_schema, check_sla_compliance};

    #[test]
    fn data_contract_schema_mismatch_error() {
        let contract = DataContract {
            name: "OrderContract".to_string(),
            input_fields: vec![
                DataContractField { name: "order_id".to_string(), ty: "String".to_string(), nullable: false },
                DataContractField { name: "amount".to_string(),   ty: "Float".to_string(),  nullable: false },
            ],
            output_fields: vec![],
            sla: DataContractSla { max_latency_ms: 5000, min_throughput: 1000, max_error_rate: 0.01 },
        };
        let ok = validate_contract_schema(&contract, &[("order_id", "String"), ("amount", "Float")]);
        assert!(ok.is_ok(), "valid schema should pass: {:?}", ok);
        let err = validate_contract_schema(&contract, &[("order_id", "String"), ("amount", "Int")]);
        assert!(err.is_err(), "type mismatch should return Err");
        assert!(err.unwrap_err().contains("amount"), "error should mention mismatched field");
        // フィールド欠落ケース
        let missing = validate_contract_schema(&contract, &[("order_id", "String")]);
        assert!(missing.is_err(), "missing field should return Err");
        assert!(missing.unwrap_err().contains("amount"), "error should mention missing field");
    }

    #[test]
    fn data_contract_sla_monitoring() {
        let sla = DataContractSla { max_latency_ms: 5000, min_throughput: 1000, max_error_rate: 0.01 };
        let ok = check_sla_compliance(&sla, 3000, 1500, 0.005);
        assert!(ok.is_ok(), "SLA within bounds should pass: {:?}", ok);
        let err = check_sla_compliance(&sla, 6000, 1500, 0.005);
        assert!(err.is_err(), "latency violation should return Err");
        assert!(err.unwrap_err().contains("latency"), "error should mention latency");
    }
}
```

確認: `cargo test v731000` で 2 件 pass。

---

### T5: バージョン更新

- `fav/Cargo.toml`: `version = "73.0.0"` → `version = "73.1.0"`
- `driver.rs` 内の `version = \"73.0.0\"` を `version = \"73.1.0\"` に replace_all
- エラーメッセージ内の `73.0.0` を `73.1.0` に replace_all
- 残存 `73.0.0` がコメント・セクションヘッダーのみであることを確認
- `cargo build` 後に `fav/Cargo.lock` が `version = "73.1.0"` を含むことを確認

---

### T6: 部分テスト確認

- `cargo test v731000` で 2 件 pass

---

### T7: 全体テスト確認

- `cargo test` 全体で 3648 tests pass（0 failures）

---

### T8: `CHANGELOG.md` 更新

```markdown
## [v73.1.0] — 2026-08-13 — データコントラクト

### Added
- `DataContractField` 構造体（name / ty / nullable）
- `DataContractSla` 構造体（max_latency_ms / min_throughput / max_error_rate）
- `DataContract` 構造体（name / input_fields / output_fields / sla）
- `validate_contract_schema(contract, actual_input)` — 入力フィールドの型整合性チェック
- `check_sla_compliance(sla, latency, throughput, error_rate)` — SLA 監視フック

### Tests
- `data_contract_schema_mismatch_error` — 型不一致を Err として検出することを確認
- `data_contract_sla_monitoring` — レイテンシ超過を Err として検出することを確認
- 合計テスト数: 3648（+2）
```

---

### T9: `versions/current.md` 更新

- 「最終更新」を `2026-08-13 (v73.1.0)` に更新
- 「進行中バージョン」を `v73.1.0` に更新
- 「次に切る版」を `v73.2.0` に更新

---

### T10: 最終確認

- `cargo test v731000` で 2 件 pass
- `cargo test` 全体で 3648 tests pass（0 failures）
- `fav/Cargo.toml` のバージョンが `73.1.0`
- `DataContract` / `DataContractSla` / `DataContractField` が pub で存在する
- `validate_contract_schema` / `check_sla_compliance` が pub で存在する
