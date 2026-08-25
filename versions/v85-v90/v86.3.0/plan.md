# Plan: v86.3.0 — business_partner_by_id() + $expand

## 実装ステップ

### Step 1: `CHANGELOG.md` に v86.3.0 エントリ追加

v86.2.0 エントリの直前（先頭）に v86.3.0 エントリを追加する。

### Step 2: `runes/sap-odata/business_partner.fav` を編集

既存ファイルの末尾に `business_partner_by_id()` 関数を追記する。

```favnir
-- 単一 BusinessPartner 取得（v86.3.0）
-- expand_address = true の場合 $expand=to_BusinessPartnerAddress を付与する

public fn business_partner_by_id(
    cfg:            SapConfig,
    partner_id:     String,
    expand_address: Bool
) -> Result<BusinessPartner, String> {
    Result.err("not implemented")
}
```

### Step 3: `runes/sap-odata/sap_odata.fav` を編集

`business_partner_by_id()` の再エクスポートを追加する。

```favnir
public fn business_partner_by_id(cfg: SapConfig, partner_id: String, expand_address: Bool) -> Result<BusinessPartner, String> {
    business_partner.business_partner_by_id(cfg, partner_id, expand_address)
}
```

### Step 4: `fav/src/driver.rs` に `mod v86300_tests` 追加

`mod v86200_tests { ... }` の直後に追加する。

```rust
// use super::* 不要（std::fs::read_to_string のみ使用）
#[cfg(test)]
mod v86300_tests {
    #[test]
    fn business_partner_by_id_function_exists() {
        let content = std::fs::read_to_string("../runes/sap-odata/business_partner.fav")
            .expect("runes/sap-odata/business_partner.fav should exist");
        assert!(
            content.contains("fn business_partner_by_id"),
            "business_partner.fav should define business_partner_by_id function"
        );
    }

    #[test]
    fn business_partner_expand_address_in_rune() {
        let content = std::fs::read_to_string("../runes/sap-odata/business_partner.fav")
            .expect("runes/sap-odata/business_partner.fav should exist");
        assert!(
            content.contains("to_BusinessPartnerAddress"),
            "business_partner.fav should reference to_BusinessPartnerAddress expand"
        );
    }
}
```

### Step 5: `cargo test` で全 pass 確認

```
cargo test 2>&1 | grep "test result"
```

期待: `3959 tests, 0 failures`
