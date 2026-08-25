# Spec: v86.5.0 — update_business_partner() PATCH

## Background

v86.4.0 で `create_business_partner()` POST を実装した。
v86.5.0 では BusinessPartner の部分更新関数 `update_business_partner()` を追加する。
PATCH リクエストで変更したいフィールドのみを指定できる `BusinessPartnerPatch` 型を定義する。

**前提**:
- `SapConfig` は `runes/sap-odata/types.fav` で定義済み
- `business_partner.fav` は `use sap_odata.types` を持つ（v86.2.0 で追加）
- `BusinessPartner` / `BusinessPartnerCategory` は `public` 修飾子付きで定義済み（v86.4.0 で修正）

## Goals

- `BusinessPartnerPatch` レコード型を定義する（3 フィールド、すべて Optional）
- `update_business_partner()` 関数シグネチャをスタブとして定義する
- driver.rs に `mod v86500_tests`（2 件）を追加し、3961 → 3963 tests とする

## 型定義・関数シグネチャ（Favnir 構文）

```favnir
public type BusinessPartnerPatch = {
    name:     Option<String>,
    currency: Option<String>,
    language: Option<String>
}

public fn update_business_partner(
    cfg:        SapConfig,
    partner_id: String,
    patch:      BusinessPartnerPatch
) -> Result<Unit, String>
```

PATCH は変更フィールドのみを送信する。スタブ実装とし、実際の HTTP 呼び出しは後続バージョンで行う。

## ファイル構成

| ファイル | 変更 |
|---|---|
| `runes/sap-odata/business_partner.fav` | `BusinessPartnerPatch` 型 + `update_business_partner()` 関数を追記 |
| `runes/sap-odata/sap_odata.fav` | `BusinessPartnerPatch` 再エクスポート + `update_business_partner()` ラッパーを追加 |
| `fav/src/driver.rs` | `mod v86500_tests` 追加（2 テスト） |
| `CHANGELOG.md` | v86.5.0 エントリ追加 |

## Success Criteria

- `runes/sap-odata/business_partner.fav` が `BusinessPartnerPatch` を含む
- `runes/sap-odata/business_partner.fav` が `update_business_partner` を含む
- `runes/sap-odata/sap_odata.fav` が `BusinessPartnerPatch` の再エクスポートを含む
- `runes/sap-odata/sap_odata.fav` が `update_business_partner` の再エクスポートを含む
- `cargo test 2>&1 | grep "test result"` → 3963 tests, 0 failures

## テスト詳細

```rust
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
    // 3 フィールドがすべて Option<String> であることを確認
    assert!(
        content.contains("Option<String>"),
        "BusinessPartnerPatch fields should be Option<String>"
    );
}
```
