# Plan: v86.5.0 — update_business_partner() PATCH

## 実装ステップ

### Step 1: `CHANGELOG.md` に v86.5.0 エントリ追加

v86.4.0 エントリの直前（先頭）に v86.5.0 エントリを追加する。

### Step 2: `runes/sap-odata/business_partner.fav` を編集

既存ファイルの末尾に `BusinessPartnerPatch` 型と `update_business_partner()` 関数を追記する。

```favnir
-- BusinessPartner 部分更新（v86.5.0）
-- PATCH リクエストで変更したいフィールドのみを指定する

public type BusinessPartnerPatch = {
    name:     Option<String>,
    currency: Option<String>,
    language: Option<String>
}

public fn update_business_partner(
    cfg:        SapConfig,
    partner_id: String,
    patch:      BusinessPartnerPatch
) -> Result<Unit, String> {
    Result.err("not implemented")
}
```

### Step 3: `runes/sap-odata/sap_odata.fav` を編集

`BusinessPartnerPatch` の再エクスポートと `update_business_partner()` ラッパーを追加する。

```favnir
public type BusinessPartnerPatch = business_partner.BusinessPartnerPatch
public fn update_business_partner(cfg: SapConfig, partner_id: String, patch: BusinessPartnerPatch) -> Result<Unit, String> {
    business_partner.update_business_partner(cfg, partner_id, patch)
}
```

### Step 4: `fav/src/driver.rs` に `mod v86500_tests` 追加

`mod v86400_tests { ... }` の直後に追加する。

```rust
// use super::* 不要（std::fs::read_to_string のみ使用）
#[cfg(test)]
mod v86500_tests {
    #[test]
    fn update_business_partner_function_exists() {
        let content = std::fs::read_to_string("../runes/sap-odata/business_partner.fav")
            .expect("runes/sap-odata/business_partner.fav should exist");
        assert!(
            content.contains("fn update_business_partner"),
            "business_partner.fav should define update_business_partner function"
        );
    }

    #[test]
    fn business_partner_patch_type_exists() {
        let content = std::fs::read_to_string("../runes/sap-odata/business_partner.fav")
            .expect("runes/sap-odata/business_partner.fav should exist");
        assert!(
            content.contains("BusinessPartnerPatch"),
            "business_partner.fav should define BusinessPartnerPatch type"
        );
        assert!(
            content.lines().any(|l| l.contains("BusinessPartnerPatch") && l.contains("{")),
            "BusinessPartnerPatch should be a record type with fields"
        );
        assert!(
            content.contains("Option<String>"),
            "BusinessPartnerPatch fields should be Option<String>"
        );
    }
}
```

### Step 5: `cargo test` で全 pass 確認

```
cargo test 2>&1 | grep "test result"
```

期待: `3963 tests, 0 failures`
