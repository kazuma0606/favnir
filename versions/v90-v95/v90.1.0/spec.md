# Spec: v90.1.0 — `SapClient` interface 定義

## Background

v90.0.0（SAP Integration 1.0）では SAP アクセスに `sap_odata.business_partners(cfg, filter)` 形式を採用しており、
`cfg: SapConfig` を明示的に渡す必要がある。v13.x.x で確立した `DbRead` / `HttpClient` 等の
`ctx.field.method()` パターンに SAP を統合するため、まず `ctx.sap` フィールドの型となる
`SapClient` interface を定義する。

本バージョンは SAP Advanced Era（v90.1〜v95.0）の最初のステップであり、
後続の `AppCtx.sap` フィールド追加（v90.2.0）・`MockSapClient` 実装（v90.3.0）の基盤となる。

## Goals

- `runes/sap-odata/types.fav` に `SapClient` interface を定義する
- インターフェースには SAP S/4HANA の主要エンティティ 5 種のアクセス関数を含める
- `interface` キーワードと `->` 関数型シグネチャを使った Favnir 標準形式で記述する

## Syntax / API

```favnir
interface SapClient {
    business_partners:    (BusinessPartnerFilter) -> Result<List<BusinessPartner>, String>,
    business_partner_by_id: (String)              -> Result<BusinessPartner, String>,
    sales_orders:         (SalesOrderFilter)      -> Result<List<SalesOrder>, String>,
    materials:            (MaterialFilter)         -> Result<List<Material>, String>,
    journal_entries:      (JournalFilter)          -> Result<List<JournalEntry>, String>
}
```

### 各メソッドの説明

| メソッド | 引数型 | 戻り型 | 説明 |
|---|---|---|---|
| `business_partners` | `BusinessPartnerFilter` | `Result<List<BusinessPartner>, String>` | 取引先一覧取得 |
| `business_partner_by_id` | `String`（ID） | `Result<BusinessPartner, String>` | 取引先単件取得 |
| `sales_orders` | `SalesOrderFilter` | `Result<List<SalesOrder>, String>` | 受注一覧取得 |
| `materials` | `MaterialFilter` | `Result<List<Material>, String>` | 資材一覧取得 |
| `journal_entries` | `JournalFilter` | `Result<List<JournalEntry>, String>` | 仕訳一覧取得 |

### 参照型（既存定義）

以下の型は各 `.fav` ファイルに既に定義済み（本バージョンでは変更しない）:
- `BusinessPartner` / `BusinessPartnerFilter` — `runes/sap-odata/business_partner.fav`
- `SalesOrder` / `SalesOrderFilter` — `runes/sap-odata/sales_order.fav`
- `Material` / `MaterialFilter` — `runes/sap-odata/material.fav`
- `JournalEntry` / `JournalFilter` — `runes/sap-odata/journal_entry.fav`

## Success Criteria

Rust テスト 2 件（合計: 4041 + 2 = **4043**）。
> 実装開始前に T0 で `cargo test` を実行し、ベースラインが 4,041 であることを実測すること。
> 実測値が異なる場合は本 spec のテスト数を実測値に合わせて更新してから実装を開始すること。

1. `sap_client_interface_defined`
   - `runes/sap-odata/types.fav` に文字列 `SapClient` が含まれること
2. `sap_client_has_business_partners_method`
   - `runes/sap-odata/types.fav` に文字列 `business_partners` が含まれること

## Error Codes

本バージョンでは新規エラーコードなし。

## Files to Modify

| ファイル | 変更内容 |
|---|---|
| `runes/sap-odata/types.fav` | `SapClient` interface を末尾に追加 |
| `fav/src/driver.rs` | `mod v90100_tests` を追加（`mod v90000_tests` の直後） |
