# v76.8.0 実装計画 — Provenance contracts

Date: 2026-08-15

---

## Step 1: driver.rs — PiiPolicy enum / ProvenanceContract 構造体追加

`fav/src/driver.rs` の末尾に `// --- v76.8.0: Provenance contracts ---` コメントと型定義を追加する。

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum PiiPolicy {
    MustBeMasked,
    AllowRaw,
    MustBeAbsent,
}

#[derive(Debug, Clone)]
pub struct ProvenanceContract {
    pub allowed_sources: Vec<DataSourceType>,
    pub pii_policy:      PiiPolicy,
}
```

---

## Step 2: driver.rs — validate_provenance_contract 追加

```rust
pub fn validate_provenance_contract(
    contract: &ProvenanceContract,
    tag: &ProvenanceTag,
) -> Result<(), String> {
    if !contract.allowed_sources.is_empty()
        && !contract.allowed_sources.contains(&tag.source.source_type)
    {
        return Err(format!(
            "source type not allowed: {:?} is not in allowed_sources",
            tag.source.source_type
        ));
    }
    match contract.pii_policy {
        PiiPolicy::MustBeMasked if tag.pii => {
            Err("pii policy violated: MustBeMasked requires pii=false".to_string())
        }
        PiiPolicy::MustBeAbsent if tag.pii => {
            Err("pii policy violated: MustBeAbsent requires pii=false".to_string())
        }
        _ => Ok(()),
    }
}
```

---

## Step 3: cargo test（既存テスト通過確認）

`cargo test` を実行し、既存の 3728 テストが引き続き pass することを確認する（v768000_tests 追加前の状態）。

---

## Step 4: CHANGELOG.md 更新

`CHANGELOG.md` の先頭に v76.8.0 エントリを追加する（テストモジュール追加より先）。

---

## Step 5: driver.rs — v768000_tests モジュール追加

```rust
#[cfg(test)]
mod v768000_tests {
    use super::*;  // PiiPolicy, ProvenanceContract, validate_provenance_contract, DataSource, DataSourceType, ProvenanceTag を参照するため必須

    #[test]
    fn provenance_contract_source_violation() { ... }

    #[test]
    fn provenance_contract_pii_violation() { ... }
}
```

---

## Step 6: Cargo.toml バージョン更新

`76.7.0` → `76.8.0`

また、driver.rs 内に存在する `76.7.0` バージョン文字列アサーションを `76.8.0` へ一括置換する（`replace_all: true`）。

---

## Step 7: versions/current.md 更新

進行中バージョンを v76.8.0 に、次に切る版を v76.9.0 に更新する。

---

## Step 8: 最終確認

`cargo test` が 3730 tests all pass であることを確認する。
