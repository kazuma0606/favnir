# v76.7.0 実装計画 — Data product 型

Date: 2026-08-15

---

## Step 1: driver.rs — DataProductSla / ProvenancePolicy / DataProduct 構造体追加

`fav/src/driver.rs` の末尾に `// --- v76.7.0: Data product 型 ---` コメントと構造体を追加する。

```rust
#[derive(Debug, Clone)]
pub struct DataProductSla {
    pub freshness_minutes: u64,
}

#[derive(Debug, Clone)]
pub struct ProvenancePolicy {
    pub require_source_declared: bool,
    pub pii_must_be_masked:      bool,
}

#[derive(Debug, Clone)]
pub struct DataProduct {
    pub name:              String,
    pub owner:             String,
    pub sla:               DataProductSla,
    pub provenance_policy: ProvenancePolicy,
}
```

---

## Step 2: driver.rs — validate_data_product 追加

```rust
pub fn validate_data_product(product: &DataProduct, tag: &ProvenanceTag) -> Result<(), String> {
    if product.provenance_policy.require_source_declared && tag.source.name.is_empty() {
        return Err("source must be declared: source name is empty".to_string());
    }
    if product.provenance_policy.pii_must_be_masked && tag.pii {
        return Err("pii policy violated: pii=true but pii_must_be_masked is required".to_string());
    }
    Ok(())
}
```

---

## Step 3: cargo test（既存テスト通過確認）

`cargo test` を実行し、既存の 3726 テストが引き続き pass することを確認する（v767000_tests 追加前の状態）。

---

## Step 4: CHANGELOG.md 更新

`CHANGELOG.md` の先頭に v76.7.0 エントリを追加する（テストモジュール追加より先）。

---

## Step 5: driver.rs — v767000_tests モジュール追加

```rust
#[cfg(test)]
mod v767000_tests {
    use super::*;  // DataProductSla, ProvenancePolicy, DataProduct, validate_data_product, DataSource, DataSourceType, ProvenanceTag を参照するため必須

    #[test]
    fn data_product_validated() { ... }

    #[test]
    fn data_product_pii_policy_violated() { ... }
}
```

---

## Step 6: Cargo.toml バージョン更新

`76.6.0` → `76.7.0`

また、driver.rs 内に存在する `76.6.0` バージョン文字列アサーションを `76.7.0` へ一括置換する（`replace_all: true`）。

---

## Step 7: versions/current.md 更新

進行中バージョンを v76.7.0 に、次に切る版を v76.8.0 に更新する。

---

## Step 8: 最終確認

`cargo test` が 3728 tests all pass であることを確認する。
