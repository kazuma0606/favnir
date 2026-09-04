# Spec: v90.3.0 — `MockSapClient` 実装

## Background

v90.1.0 で `SapClient` interface を定義し、v90.2.0 で `AppCtx` に `sap: SapClient` フィールドを追加した。
テストコードでは本番の HTTP クライアントを使わず、スタブ（mock）を使って依存を制御したい。

v13.5.0 で導入された `MockDb` / `MockStorage` パターンと同じ設計で `MockSapClient` を実装する。

`business_partner_by_id`（単一エンティティ取得）は固定レスポンスフィールドを持たず、常に `Result.err("not implemented")` を返す。
テストシナリオでは `business_partners` のフィルタ結果から特定の BP を参照できるため、単一取得の mock 化は省略する設計とする。

### MockDb パターン（参照実装）

`runes/ctx/mock_db.fav` の実装:
```favnir
// runes/ctx/mock_db.fav
type MockDb(List<String>)
public fn MockDb.empty() -> MockDb { MockDb(List.empty()) }
public fn MockDb.seed(rows: List<String>) -> MockDb { MockDb(rows) }
impl DbRead for MockDb {
    fn query(db: MockDb, sql: String, params: List<String>) -> Result<String, String> {
        Result.ok(Json.encode_raw(db))
    }
}
```

`MockSapClient` も同じ方針で:
- レコード型（`type MockSapClient = { ... }`）でテスト用固定レスポンスを保持
- `impl SapClient for MockSapClient` で各メソッドは保持した Result をそのまま返す

## Goals

1. `runes/sap-odata/mock.fav` に `MockSapClient` 型を定義する
2. `impl SapClient for MockSapClient` を実装する
3. Rust テスト 2 件を追加して構造を保証する

## Syntax / API

```favnir
// runes/sap-odata/mock.fav
// MockSapClient — テスト用 SapClient スタブ（v90.3.0）
// SapClient interface のすべてのメソッドを固定レスポンスで返す。
// テストパイプラインで ctx.sap の代替として使用する。

type MockSapClient = {
    business_partners_result: Result<List<BusinessPartner>, String>,
    sales_orders_result:      Result<List<SalesOrder>, String>,
    materials_result:         Result<List<Material>, String>,
    journal_entries_result:   Result<List<JournalEntry>, String>
}

impl SapClient for MockSapClient {
    fn business_partners(self: MockSapClient, filter: BusinessPartnerFilter)
        -> Result<List<BusinessPartner>, String> { self.business_partners_result }
    fn business_partner_by_id(self: MockSapClient, id: String)
        -> Result<BusinessPartner, String> { Result.err("not implemented") }
    fn sales_orders(self: MockSapClient, filter: SalesOrderFilter)
        -> Result<List<SalesOrder>, String> { self.sales_orders_result }
    fn materials(self: MockSapClient, filter: MaterialFilter)
        -> Result<List<Material>, String> { self.materials_result }
    fn journal_entries(self: MockSapClient, filter: JournalFilter)
        -> Result<List<JournalEntry>, String> { self.journal_entries_result }
}
```

## Success Criteria

- `runes/sap-odata/mock.fav` が存在する
- `mock.fav` に `impl SapClient for MockSapClient` が含まれる
- `impl SapClient for MockSapClient` ブロックに 5 メソッド（`business_partners` / `business_partner_by_id` / `sales_orders` / `materials` / `journal_entries`）がすべて含まれる
- `cargo test` で 4047 tests, 0 failures

## Files to Modify / Create

| ファイル | 操作 |
|---|---|
| `runes/sap-odata/mock.fav` | 新規作成（MockSapClient + impl） |
| `fav/src/driver.rs` | `mod v90300_tests` 追加 |
| `CHANGELOG.md` | v90.3.0 エントリ追加 |
