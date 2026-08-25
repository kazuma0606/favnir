# Plan: v86.4.0 — create_business_partner() POST

## 実装ステップ

### Step 1: `CHANGELOG.md` に v86.4.0 エントリ追加

v86.3.0 エントリの直前（先頭）に v86.4.0 エントリを追加する。

### Step 2: `runes/sap-odata/business_partner.fav` を編集

既存ファイルの末尾に `NewBusinessPartner` 型と `create_business_partner()` 関数を追記する。

```favnir
-- BusinessPartner 新規作成（v86.4.0）
-- POST 前に x-csrf-token を取得しリクエストヘッダーに付与する

public type NewBusinessPartner = {
    name:     String,
    category: BusinessPartnerCategory,
    country:  String,
    currency: String
}

public fn create_business_partner(
    cfg:  SapConfig,
    body: NewBusinessPartner
) -> Result<BusinessPartner, String> {
    Result.err("not implemented")
}
```

### Step 3: `runes/sap-odata/sap_odata.fav` を編集

`NewBusinessPartner` の再エクスポートと `create_business_partner()` ラッパーを追加する。

```favnir
public type NewBusinessPartner = business_partner.NewBusinessPartner
public fn create_business_partner(cfg: SapConfig, body: NewBusinessPartner) -> Result<BusinessPartner, String> {
    business_partner.create_business_partner(cfg, body)
}
```

### Step 4: `fav/src/driver.rs` に `mod v86400_tests` 追加

`mod v86300_tests { ... }` の直後に追加する。

```rust
// use super::* 不要（std::fs::read_to_string のみ使用）
#[cfg(test)]
mod v86400_tests {
    #[test]
    fn create_business_partner_function_exists() {
        let content = std::fs::read_to_string("../runes/sap-odata/business_partner.fav")
            .expect("runes/sap-odata/business_partner.fav should exist");
        assert!(
            content.contains("fn create_business_partner"),
            "business_partner.fav should define create_business_partner function"
        );
    }

    #[test]
    fn new_business_partner_type_exists() {
        let content = std::fs::read_to_string("../runes/sap-odata/business_partner.fav")
            .expect("runes/sap-odata/business_partner.fav should exist");
        assert!(
            content.contains("NewBusinessPartner"),
            "business_partner.fav should define NewBusinessPartner type"
        );
    }
}
```

### Step 5: `cargo test` で全 pass 確認

```
cargo test 2>&1 | grep "test result"
```

期待: `3961 tests, 0 failures`
