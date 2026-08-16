# v76.7.0 仕様書 — Data product 型

Date: 2026-08-15
Status: 計画中

---

## Background

データ製品（Data Product）をファーストクラス型として表現し、データメッシュアーキテクチャの基盤を提供する。`DataProduct` 構造体でデータ製品のメタデータ（名前・オーナー・SLA・来歴ポリシー）を管理し、`validate_data_product` で来歴タグとポリシーの整合性を検証する。

---

## Goals

1. `DataProductSla` 構造体（freshness_minutes: u64）を追加する
2. `ProvenancePolicy` 構造体（require_source_declared: bool, pii_must_be_masked: bool）を追加する
3. `DataProduct` 構造体（name: String, owner: String, sla: DataProductSla, provenance_policy: ProvenancePolicy）を追加する
4. `validate_data_product(product: &DataProduct, tag: &ProvenanceTag) -> Result<(), String>` を追加する
5. Rust テスト 2 件を追加し 3728 tests に到達する

---

## 型・関数仕様

### `DataProductSla` 構造体

```rust
#[derive(Debug, Clone)]
pub struct DataProductSla {
    pub freshness_minutes: u64,
}
```

---

### `ProvenancePolicy` 構造体

```rust
#[derive(Debug, Clone)]
pub struct ProvenancePolicy {
    pub require_source_declared: bool,
    pub pii_must_be_masked:      bool,
}
```

---

### `DataProduct` 構造体

```rust
#[derive(Debug, Clone)]
pub struct DataProduct {
    pub name:              String,
    pub owner:             String,
    pub sla:               DataProductSla,
    pub provenance_policy: ProvenancePolicy,
}
```

---

### `validate_data_product`

```rust
pub fn validate_data_product(product: &DataProduct, tag: &ProvenanceTag) -> Result<(), String>
```

**検証ルール:**

1. `require_source_declared = true` の場合: `tag.source.name` が空文字でないこと
   - 違反時: `Err("source must be declared: source name is empty".to_string())`

2. `pii_must_be_masked = true` の場合: `tag.pii` が `false` であること（PII がマスク済みであることを意味する）
   - 違反時: `Err("pii policy violated: pii=true but pii_must_be_masked is required".to_string())`

3. 両方違反している場合は、ルール 1（source_declared）を優先してエラーを返す。この優先順位の検証は本バージョンのテストスコープ外（テスト総数 3728 +2 を維持するため）

4. すべて満たせば `Ok(())`

---

## テスト仕様

### `data_product_validated`

```rust
let src = DataSource {
    name:        "crm".to_string(),
    uri:         "snowflake://crm/users".to_string(),
    source_type: DataSourceType::Snowflake,
};
let tag = ProvenanceTag {
    source:     src,
    transforms: vec!["mask_pii".to_string()],
    pii:        false,
};
let product = DataProduct {
    name:  "customer-360".to_string(),
    owner: "data-platform-team".to_string(),
    sla:   DataProductSla { freshness_minutes: 60 },
    provenance_policy: ProvenancePolicy {
        require_source_declared: true,
        pii_must_be_masked:      true,
    },
};
// source.name = "crm"（非空）、pii = false → 両ポリシーを満たす
assert!(validate_data_product(&product, &tag).is_ok());
```

### `data_product_pii_policy_violated`

```rust
let src = DataSource {
    name:        "crm".to_string(),
    uri:         "snowflake://crm/users".to_string(),
    source_type: DataSourceType::Snowflake,
};
let pii_tag = ProvenanceTag {
    source:     src,
    transforms: vec![],
    pii:        true,  // PII がマスクされていない
};
let product = DataProduct {
    name:  "customer-360".to_string(),
    owner: "data-platform-team".to_string(),
    sla:   DataProductSla { freshness_minutes: 60 },
    provenance_policy: ProvenancePolicy {
        require_source_declared: false,
        pii_must_be_masked:      true,
    },
};
let result = validate_data_product(&product, &pii_tag);
assert!(result.is_err());
let msg = result.unwrap_err();
assert!(msg.contains("pii"), "error message should mention pii: {}", msg);
```

---

## Success Criteria

- `DataProductSla` / `ProvenancePolicy` / `DataProduct` 構造体が定義されている
- `validate_data_product` が require_source_declared・pii_must_be_masked ポリシーを正しく検証する
- ポリシー違反時に `Err(String)` を返し、メッセージに違反種別が含まれる
- `data_product_validated` が pass
- `data_product_pii_policy_violated` が pass
- `cargo test` が 3728 tests all pass
- `driver.rs` 内の `76.6.0` バージョン文字列アサーションがすべて `76.7.0` に更新されている
- `CHANGELOG.md` の先頭に v76.7.0 エントリが存在する

---

## 変更ファイル

- `fav/src/driver.rs` — `DataProductSla`, `ProvenancePolicy`, `DataProduct`, `validate_data_product`, `v767000_tests` を追加
- `CHANGELOG.md` — v76.7.0 エントリを追加
- `versions/current.md` — 進行中バージョンを更新
- `fav/Cargo.toml` — バージョンを `76.6.0` → `76.7.0` に更新

---

## 依存（既実装）

- `DataSource` 構造体（v76.1.0）
- `DataSourceType` enum（v76.1.0）
- `ProvenanceTag` 構造体（v76.1.0）

---

## 対象外

- `DataProduct.validate(product)` の Favnir 言語構文統合（将来バージョン）
- SLA（freshness_minutes）の実際の鮮度チェック（将来バージョン）
- `ProvenancePolicy` の allowed_sources フィルタリング（v76.8.0 の ProvenanceContract で対応）
