# Plan: v90.1.0 — `SapClient` interface 定義

## 実装ステップ

### Step 1: 既存型の確認

実装前に参照型が存在することを確認する。

1. `runes/sap-odata/business_partner.fav` に `BusinessPartner` / `BusinessPartnerFilter` が定義されていることを確認
2. `runes/sap-odata/sales_order.fav` に `SalesOrder` / `SalesOrderFilter` が定義されていることを確認
3. `runes/sap-odata/material.fav` に `Material` / `MaterialFilter` が定義されていることを確認
4. `runes/sap-odata/journal_entry.fav` に `JournalEntry` / `JournalFilter` が定義されていることを確認

### Step 2: `SapClient` interface を `types.fav` に追加

`runes/sap-odata/types.fav` の末尾に以下を追加する:

```favnir
-- SAP アクセスの統一インターフェース（v90.1.0）
-- ctx.sap フィールドの型として使用する。
-- impl: SapODataClient（本番）/ MockSapClient（テスト・v90.3.0 で実装）
interface SapClient {
    business_partners:      (BusinessPartnerFilter) -> Result<List<BusinessPartner>, String>,
    business_partner_by_id: (String)                -> Result<BusinessPartner, String>,
    sales_orders:           (SalesOrderFilter)      -> Result<List<SalesOrder>, String>,
    materials:              (MaterialFilter)         -> Result<List<Material>, String>,
    journal_entries:        (JournalFilter)          -> Result<List<JournalEntry>, String>
}
```

### Step 3: `driver.rs` に `mod v90100_tests` を追加

`mod v90000_tests { ... }` の直後に以下を挿入する:

```rust
#[cfg(test)]
mod v90100_tests {
    // use super::* は不要（std::fs のみ使用）
    #[test]
    fn sap_client_interface_defined() {
        let content = std::fs::read_to_string("../runes/sap-odata/types.fav")
            .expect("types.fav should exist");
        assert!(
            content.contains("SapClient"),
            "types.fav should define SapClient interface"
        );
    }

    #[test]
    fn sap_client_has_business_partners_method() {
        let content = std::fs::read_to_string("../runes/sap-odata/types.fav")
            .expect("types.fav should exist");
        assert!(
            content.contains("business_partners"),
            "SapClient should have business_partners method"
        );
    }
}
```

### Step 4: `cargo test` で全 pass 確認

```bash
cd fav && cargo test 2>&1 | grep "test result"
```

期待結果: `test result: ok. 4043 passed; 0 failed`

### Step 5: CI 事前確認

```bash
cargo clippy --locked -- -D warnings
./target/debug/fav fmt --check self/compiler.fav
./target/debug/fav fmt --check self/checker.fav
```
