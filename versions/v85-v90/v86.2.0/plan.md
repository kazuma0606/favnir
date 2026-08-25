# Plan: v86.2.0 — BusinessPartnerFilter + business_partners() クエリ

## 実装ステップ

### Step 1: `CHANGELOG.md` に v86.2.0 エントリ追加

v86.1.0 エントリの直前（先頭）に v86.2.0 エントリを追加する。

### Step 2: `runes/sap-odata/business_partner.fav` を編集

既存ファイルの末尾に `BusinessPartnerFilter` 型と `business_partners()` 関数を追記する。

```favnir
type BusinessPartnerFilter = {
    country:       Option<String>,
    category:      Option<BusinessPartnerCategory>,
    changed_after: Option<String>,
    top:           Option<Int>
}

public fn business_partners(
    cfg:    SapConfig,
    filter: BusinessPartnerFilter
) -> Result<List<BusinessPartner>, String> {
    Result.err("not implemented")
}
```

**注意**: `SapConfig` は `runes/sap-odata/types.fav` で定義済み。
`business_partners()` 関数本体はスタブ実装とし、後続バージョンで HTTP 呼び出しに置き換える。

### Step 3: `fav/src/driver.rs` に `mod v86200_tests` 追加

`mod v86100_tests { ... }` の直後に追加する。

```rust
#[cfg(test)]
mod v86200_tests {
    #[test]
    fn business_partners_function_exists() {
        let content = std::fs::read_to_string("../runes/sap-odata/business_partner.fav")
            .expect("runes/sap-odata/business_partner.fav should exist");
        assert!(
            content.contains("business_partners"),
            "business_partner.fav should define business_partners function"
        );
    }

    #[test]
    fn business_partner_filter_type_exists() {
        let content = std::fs::read_to_string("../runes/sap-odata/business_partner.fav")
            .expect("runes/sap-odata/business_partner.fav should exist");
        assert!(
            content.contains("BusinessPartnerFilter"),
            "business_partner.fav should define BusinessPartnerFilter type"
        );
    }
}
```

### Step 4: `cargo test` で全 pass 確認

```
cargo test 2>&1 | grep "test result"
```

期待: `3957 tests, 0 failures`
