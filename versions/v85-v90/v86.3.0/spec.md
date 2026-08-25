# Spec: v86.3.0 — business_partner_by_id() + $expand

## Background

v86.2.0 で `business_partners()` 一覧取得関数を定義した。
v86.3.0 では単一 BusinessPartner の取得関数 `business_partner_by_id()` を追加し、
`expand_address = true` の場合に `$expand=to_BusinessPartnerAddress` を付与して
住所情報を 1 リクエストで取得できるようにする。

**前提**:
- `SapConfig` は `runes/sap-odata/types.fav` で定義済み
- `BusinessPartner` / `BusinessPartnerAddress` は `runes/sap-odata/business_partner.fav` で定義済み（v86.1.0）

## Goals

- `business_partner_by_id()` 関数シグネチャをスタブとして定義する
- `expand_address: Bool` パラメータで `$expand=to_BusinessPartnerAddress` 付与を表現する
- driver.rs に `mod v86300_tests`（2 件）を追加し、3957 → 3959 tests とする

## 関数シグネチャ（Favnir 構文）

```favnir
public fn business_partner_by_id(
    cfg:            SapConfig,
    partner_id:     String,
    expand_address: Bool
) -> Result<BusinessPartner, String>
```

`expand_address = true` の場合、`$expand=to_BusinessPartnerAddress` クエリパラメータを付与する。
スタブ実装とし、実際の HTTP 呼び出しは後続バージョンで行う。

## ファイル構成

| ファイル | 変更 |
|---|---|
| `runes/sap-odata/business_partner.fav` | `business_partner_by_id()` 関数を追記 |
| `runes/sap-odata/sap_odata.fav` | `business_partner_by_id()` 再エクスポートを追加 |
| `fav/src/driver.rs` | `mod v86300_tests` 追加（2 テスト） |
| `CHANGELOG.md` | v86.3.0 エントリ追加 |

## Success Criteria

- `runes/sap-odata/business_partner.fav` が `business_partner_by_id` を含む
- `runes/sap-odata/business_partner.fav` が `to_BusinessPartnerAddress` を含む
- `runes/sap-odata/sap_odata.fav` が `business_partner_by_id` の再エクスポートを含む
- `cargo test 2>&1 | grep "test result"` → 3959 tests, 0 failures

## テスト詳細

```rust
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
```
