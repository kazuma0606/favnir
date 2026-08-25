# Spec: v86.1.0 — BusinessPartner / BusinessPartnerAddress 型定義

## Background

v86.0.0（SAP Foundation 1.0）で `sap-odata` Rune の基盤（SapTomlConfig・接続設定）を確立した。
v86.1.0 では SAP Master Data の中核型である `BusinessPartner` と `BusinessPartnerAddress` を
`runes/sap-odata/business_partner.fav` に定義し、型安全な SAP BP 参照の出発点を作る。

## Goals

- `BusinessPartnerCategory`（列挙型）を定義する（Person | Organization | Group）
- `BusinessPartner` レコード型を定義する（8 フィールド）
- `BusinessPartnerAddress` レコード型を定義する（6 フィールド）
- driver.rs に `mod v86100_tests`（2 件）を追加し、3953 → 3955 tests とする

## 型定義（Favnir 構文）

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

## ファイル構成

| ファイル | 変更 |
|---|---|
| `runes/sap-odata/business_partner.fav` | 新規作成（3 型定義） |
| `fav/src/driver.rs` | `mod v86100_tests` 追加（2 テスト） |
| `CHANGELOG.md` | v86.1.0 エントリ追加 |

## Success Criteria

- `runes/sap-odata/business_partner.fav` が存在し `BusinessPartnerCategory` を含む
- `runes/sap-odata/business_partner.fav` が `BusinessPartner` を含む
- `runes/sap-odata/business_partner.fav` が `BusinessPartnerAddress` を含む
- `cargo test 2>&1 | grep "test result"` → 3955 tests, 0 failures

## テスト詳細

```rust
#[test]
fn business_partner_type_defined_in_rune() {
    // runes/sap-odata/business_partner.fav に BusinessPartner 型が定義されている
    let content = std::fs::read_to_string("../runes/sap-odata/business_partner.fav")
        .expect("runes/sap-odata/business_partner.fav should exist");
    assert!(
        content.contains("BusinessPartner"),
        "business_partner.fav should define BusinessPartner type"
    );
}

#[test]
fn business_partner_address_type_defined_in_rune() {
    // 同ファイルに BusinessPartnerAddress 型が定義されている
    let content = std::fs::read_to_string("../runes/sap-odata/business_partner.fav")
        .expect("runes/sap-odata/business_partner.fav should exist");
    assert!(
        content.contains("BusinessPartnerAddress"),
        "business_partner.fav should define BusinessPartnerAddress type"
    );
}
```
