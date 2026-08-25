# Plan: v86.1.0 — BusinessPartner / BusinessPartnerAddress 型定義

## 実装ステップ

### Step 1: `CHANGELOG.md` に v86.1.0 エントリ追加

v86.0.0 エントリの直後（先頭）に v86.1.0 エントリを追加する。

### Step 2: `runes/sap-odata/business_partner.fav` 新規作成

`BusinessPartnerCategory` 列挙型、`BusinessPartner` レコード型、`BusinessPartnerAddress` レコード型を定義する。

```favnir
type BusinessPartnerCategory = Person | Organization | Group

type BusinessPartner = {
    partner_id:   String,
    name:         String,
    category:     BusinessPartnerCategory,
    country:      String,
    language:     String,
    currency:     String,
    created_at:   String,
    addresses:    Option<List<BusinessPartnerAddress>>
}

type BusinessPartnerAddress = {
    address_id:  String,
    street:      String,
    city:        String,
    postal_code: String,
    country:     String,
    region:      Option<String>
}
```

### Step 3: `fav/src/driver.rs` に `mod v86100_tests` 追加

`mod v86000_tests { ... }` の直後に追加する。

```rust
#[cfg(test)]
mod v86100_tests {
    #[test]
    fn business_partner_type_defined_in_rune() {
        let content = std::fs::read_to_string("../runes/sap-odata/business_partner.fav")
            .expect("runes/sap-odata/business_partner.fav should exist");
        assert!(
            content.contains("BusinessPartner"),
            "business_partner.fav should define BusinessPartner type"
        );
    }

    #[test]
    fn business_partner_address_type_defined_in_rune() {
        let content = std::fs::read_to_string("../runes/sap-odata/business_partner.fav")
            .expect("runes/sap-odata/business_partner.fav should exist");
        assert!(
            content.contains("BusinessPartnerAddress"),
            "business_partner.fav should define BusinessPartnerAddress type"
        );
    }
}
```

### Step 4: `cargo test` で全 pass 確認

```
cargo test 2>&1 | grep "test result"
```

期待: `3955 tests, 0 failures`
