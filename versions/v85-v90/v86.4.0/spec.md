# Spec: v86.4.0 — create_business_partner() POST

## Background

v86.3.0 で `business_partner_by_id()` 単一取得を実装した。
v86.4.0 では BusinessPartner の新規作成関数 `create_business_partner()` を追加する。
POST リクエスト前に `x-csrf-token` を取得し、ヘッダーに付与する設計とする。

**前提**:
- `SapConfig` は `runes/sap-odata/types.fav` で定義済み
- `BusinessPartner` / `BusinessPartnerCategory` は `runes/sap-odata/business_partner.fav` で定義済み（v86.1.0）
- `business_partner.fav` は `use sap_odata.types` を持つ（v86.2.0 で追加）

## Goals

- `NewBusinessPartner` レコード型を定義する（4 フィールド）
- `create_business_partner()` 関数シグネチャをスタブとして定義する
- `x-csrf-token` 取得の設計意図をコメントで明記する
- driver.rs に `mod v86400_tests`（2 件）を追加し、3959 → 3961 tests とする

## 型定義・関数シグネチャ（Favnir 構文）

```favnir
public type NewBusinessPartner = {
    name:     String,
    category: BusinessPartnerCategory,
    country:  String,
    currency: String
}

public fn create_business_partner(
    cfg:  SapConfig,
    body: NewBusinessPartner
) -> Result<BusinessPartner, String>
```

POST 前に `x-csrf-token` を取得し、リクエストヘッダーに付与する。
スタブ実装とし、実際の HTTP 呼び出しは後続バージョンで行う。

## ファイル構成

| ファイル | 変更 |
|---|---|
| `runes/sap-odata/business_partner.fav` | `NewBusinessPartner` 型 + `create_business_partner()` 関数を追記 |
| `runes/sap-odata/sap_odata.fav` | `NewBusinessPartner` 再エクスポート + `create_business_partner()` ラッパーを追加 |
| `fav/src/driver.rs` | `mod v86400_tests` 追加（2 テスト） |
| `CHANGELOG.md` | v86.4.0 エントリ追加 |

## Success Criteria

- `runes/sap-odata/business_partner.fav` が `NewBusinessPartner` を含む
- `runes/sap-odata/business_partner.fav` が `NewBusinessPartner` 固有フィールド `currency` を含む
- `runes/sap-odata/business_partner.fav` が `create_business_partner` を含む
- `runes/sap-odata/sap_odata.fav` が `create_business_partner` の再エクスポートを含む
- `cargo test 2>&1 | grep "test result"` → 3961 tests, 0 failures

## テスト詳細

```rust
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
    // NewBusinessPartner 型の開始行（`NewBusinessPartner = {`）が存在することを確認
    assert!(
        content.lines().any(|l| l.contains("NewBusinessPartner") && l.contains("{")),
        "NewBusinessPartner should be a record type with fields"
    );
}
```
