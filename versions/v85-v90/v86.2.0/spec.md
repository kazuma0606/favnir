# Spec: v86.2.0 — BusinessPartnerFilter + business_partners() クエリ

## Background

v86.1.0 で `BusinessPartner` / `BusinessPartnerAddress` / `BusinessPartnerCategory` を
`runes/sap-odata/business_partner.fav` に定義した。
v86.2.0 では同ファイルに一覧取得用の `BusinessPartnerFilter` 型と `business_partners()` 関数を追記し、
SAP BP クエリの基礎を実装する。

**前提**: `SapConfig` は `runes/sap-odata/types.fav` で定義済み。
`BusinessPartnerCategory` は `runes/sap-odata/business_partner.fav` で定義済み（v86.1.0）。

## Goals

- `BusinessPartnerFilter` レコード型を定義する（4 フィールド、すべて Optional）
- `business_partners()` 関数シグネチャを定義する（`SapConfig` + `BusinessPartnerFilter` → `Result<List<BusinessPartner>, String>`）
- driver.rs に `mod v86200_tests`（2 件）を追加し、3955 → 3957 tests とする

## 型定義・関数シグネチャ（Favnir 構文）

```favnir
type BusinessPartnerFilter = {
    country:       Option<String>,
    category:      Option<BusinessPartnerCategory>,
    changed_after: Option<String>,   -- ISO8601 日付文字列
    top:           Option<Int>
}

public fn business_partners(
    cfg:    SapConfig,
    filter: BusinessPartnerFilter
) -> Result<List<BusinessPartner>, String>
```

**注意**: `business_partners()` はシグネチャ（スタブ）として定義する。
実際の HTTP 呼び出し実装は後続バージョンで行う。

## ファイル構成

| ファイル | 変更 |
|---|---|
| `runes/sap-odata/business_partner.fav` | `BusinessPartnerFilter` 型 + `business_partners()` 関数を追記（v86.1.0 で作成済み） |
| `fav/src/driver.rs` | `mod v86200_tests` 追加（2 テスト） |
| `CHANGELOG.md` | v86.2.0 エントリ追加 |

## Success Criteria

- `runes/sap-odata/business_partner.fav` が `BusinessPartnerFilter` を含む
- `runes/sap-odata/business_partner.fav` が `business_partners` を含む
- `cargo test 2>&1 | grep "test result"` → 3957 tests, 0 failures

## テスト詳細

```rust
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
```
