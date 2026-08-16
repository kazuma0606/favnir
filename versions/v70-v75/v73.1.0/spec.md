# v73.1.0 Spec — データコントラクト

Date: 2026-08-13
Status: 計画中

---

## 背景

エンタープライズデータパイプラインでは、ステージ境界のスキーマ・SLA・品質条件を
事前に宣言し、違反を早期検出することが求められる。
v73.1.0 では `contract` キーワードを導入し、入出力スキーマ・SLA・品質条件を
型レベルで宣言できる `DataContract` 構造体と検証ロジックを実装する。

---

## 目標

1. `DataContract` 構造体（input_fields / output_fields / sla / quality）を driver.rs に追加
2. `validate_contract_schema` — 入出力フィールドの型整合性チェック関数
3. `DataContractSla` 構造体 + `check_sla_compliance` — SLA 監視フック
4. 2 件のテスト（`data_contract_schema_mismatch_error` / `data_contract_sla_monitoring`）

---

## 構文・API 例

```favnir
// データコントラクトの宣言（将来のパーサー統合を見据えたランタイム実装）
contract OrderPipelineContract {
    input: {
        order_id: String where String.length(self) > 0
        amount:   PositiveFloat
        status:   "pending" | "paid" | "cancelled"
    }
    output: {
        inserted: Int where self >= 0
        skipped:  Int where self >= 0
    }
    sla: {
        max_latency_ms:  5000
        min_throughput:  1000
        max_error_rate:  0.01
    }
    quality: {
        min_completeness: 0.99
        max_null_ratio:   0.01
    }
}
```

---

## 実装詳細

### `driver.rs` — `DataContractField` 構造体

```rust
pub struct DataContractField {
    pub name: String,
    pub ty: String,         // "String" / "Int" / "Float" / "Bool"
    pub nullable: bool,
}
```

### `driver.rs` — `DataContractSla` 構造体

```rust
pub struct DataContractSla {
    pub max_latency_ms: u64,
    pub min_throughput: u64,     // rows/sec
    pub max_error_rate: f64,     // 0.0〜1.0
}
```

### `driver.rs` — `DataContract` 構造体

```rust
pub struct DataContract {
    pub name: String,
    pub input_fields: Vec<DataContractField>,
    pub output_fields: Vec<DataContractField>,
    pub sla: DataContractSla,
}
```

### `driver.rs` — `validate_contract_schema`

```rust
pub fn validate_contract_schema(
    contract: &DataContract,
    actual_input: &[(&str, &str)],   // (field_name, type_name) のスライス
) -> Result<(), String>
```

`contract.input_fields` の各フィールドが `actual_input` に存在し型が一致していることを確認する。
- フィールドが存在しない → `Err("schema mismatch: field '...' missing in actual input")`
- 型不一致 → `Err("schema mismatch: field '...' expected type '...', got '...'")`
- 全一致 → `Ok(())`

**境界値仕様**: `actual_latency_ms == sla.max_latency_ms` は OK（`>` のみ違反）。`actual_throughput == sla.min_throughput` は OK（`<` のみ違反）。`actual_error_rate == sla.max_error_rate` は OK（`>` のみ違反）。

### `driver.rs` — `check_sla_compliance`

```rust
pub fn check_sla_compliance(
    sla: &DataContractSla,
    actual_latency_ms: u64,
    actual_throughput: u64,
    actual_error_rate: f64,
) -> Result<(), String>
```

SLA 条件（最大レイテンシ・最小スループット・最大エラー率）を確認し、
いずれかを超過していれば `Err(...)` を返す。

---

## テスト

### `v731000_tests` モジュール

```rust
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
    // 正しいスキーマ → Ok
    let ok = validate_contract_schema(&contract, &[("order_id", "String"), ("amount", "Float")]);
    assert!(ok.is_ok(), "valid schema should pass: {:?}", ok);
    // 型不一致 → Err
    let err = validate_contract_schema(&contract, &[("order_id", "String"), ("amount", "Int")]);
    assert!(err.is_err(), "type mismatch should return Err");
    assert!(err.unwrap_err().contains("amount"), "error should mention mismatched field");
}

#[test]
fn data_contract_sla_monitoring() {
    let sla = DataContractSla {
        max_latency_ms: 5000,
        min_throughput: 1000,
        max_error_rate: 0.01,
    };
    // SLA 満足 → Ok
    let ok = check_sla_compliance(&sla, 3000, 1500, 0.005);
    assert!(ok.is_ok(), "SLA within bounds should pass: {:?}", ok);
    // レイテンシ超過 → Err
    let err = check_sla_compliance(&sla, 6000, 1500, 0.005);
    assert!(err.is_err(), "latency violation should return Err");
    assert!(err.unwrap_err().contains("latency"), "error should mention latency");
}
```

---

## 成功基準

- `cargo test v731000` で 2 件 pass
- `cargo test` 全体で 3648 tests pass（3646 + 2）
- `fav/Cargo.toml` のバージョンが `73.1.0` であること
- `DataContract` / `DataContractSla` / `DataContractField` が pub
- `validate_contract_schema` / `check_sla_compliance` が pub

---

## スコープ外

- `contract` キーワードのパース・AST 統合（将来バージョン — Rust パーサー変更は不要）
- `DataContractQuality` 構造体（`min_completeness` / `max_null_ratio`）— v73.2.0 以降
- WASM / サイト MDX 更新（v74.x 以降）

---

## 変更ファイル

- `fav/src/driver.rs` — `DataContractField` / `DataContractSla` / `DataContract` 構造体 + `validate_contract_schema` / `check_sla_compliance` + `v731000_tests` + バージョン更新
- `fav/Cargo.toml` — version `73.0.0` → `73.1.0`
- `CHANGELOG.md` — v73.1.0 エントリ追加
- `versions/current.md` — 進行中バージョンを v73.1.0 に更新
