# Plan: v91.8.0 — `ODataQueryBuilder` + SapQueryClient 統合

## 実装ステップ

### Step 0: 着手前チェック

- `cargo test` で 4,084 tests, 0 failures を確認
- `fav/src/driver.rs` に `mod v91700_tests` が存在することを確認
- `runes/sap-odata/query.fav` に `public type JournalEntryQuery` が含まれることを確認
- `fav/tmp/hello.fav` が存在することを確認

### Step 1: `query.fav` に `ODataQueryBuilder` と `build_url` を追加

`journal_entry_query()` 定義の後に追加する：

```favnir
-- OData クエリ型の共通ラッパー（v91.8.0）
-- entity: SAP エンティティセット名（例: "A_BusinessPartner", "A_SalesOrder"）
-- T: エンティティ型（ファントム）、Q: クエリオプション型
public type ODataQueryBuilder<T, Q> = {
    query:  Q,
    entity: String
}

-- クエリを OData ベース URL に結合するヘルパー（簡易実装）
-- フル OData URL 生成（$filter/$select/$expand 展開）は将来バージョンで対応予定
public fn build_url<T, Q>(builder: ODataQueryBuilder<T, Q>, base_url: String) -> String {
    String.concat([base_url, "/", builder.entity])
}
```

### Step 2: `query_client.fav` を新規作成

`runes/sap-odata/query_client.fav` を作成する。循環 dep なしで `SapQueryClient` interface を定義できる：

```favnir
-- SAP OData クエリメソッド interface（v91.8.0）
-- 循環 dep 解消策: SapClient（types.fav）とは別インターフェースとして定義。
-- query.fav → entity.fav → types.fav の依存チェーンに乗っており循環なし。

use sap_odata.query
use sap_odata.sales_order
use sap_odata.business_partner
use sap_odata.material
use sap_odata.purchase_order
use sap_odata.journal_entry

public interface SapQueryClient {
    fn sales_orders_query(ctx: SapQueryClient, q: SalesOrderQuery) -> Result<List<SalesOrder>, String>
    fn business_partners_query(ctx: SapQueryClient, q: BusinessPartnerQuery) -> Result<List<BusinessPartner>, String>
    fn materials_query(ctx: SapQueryClient, q: MaterialQuery) -> Result<List<Material>, String>
    fn purchase_orders_query(ctx: SapQueryClient, q: PurchaseOrderQuery) -> Result<List<PurchaseOrder>, String>
    fn journal_entries_query(ctx: SapQueryClient, q: JournalEntryQuery) -> Result<List<JournalEntry>, String>
}
```

### Step 3: `client.fav` に `impl SapQueryClient for SapODataClient` を追加

```favnir
use sap_odata.query_client

impl SapQueryClient for SapODataClient {
    fn sales_orders_query(ctx: SapODataClient, q: SalesOrderQuery)
        -> Result<List<SalesOrder>, String> {
        Result.err("sales_orders_query: not yet implemented")
    }
    fn business_partners_query(ctx: SapODataClient, q: BusinessPartnerQuery)
        -> Result<List<BusinessPartner>, String> {
        Result.err("business_partners_query: not yet implemented")
    }
    fn materials_query(ctx: SapODataClient, q: MaterialQuery)
        -> Result<List<Material>, String> {
        Result.err("materials_query: not yet implemented")
    }
    fn purchase_orders_query(ctx: SapODataClient, q: PurchaseOrderQuery)
        -> Result<List<PurchaseOrder>, String> {
        Result.err("purchase_orders_query: not yet implemented")
    }
    fn journal_entries_query(ctx: SapODataClient, q: JournalEntryQuery)
        -> Result<List<JournalEntry>, String> {
        Result.err("journal_entries_query: not yet implemented")
    }
}
```

### Step 4: `mock.fav` に `impl SapQueryClient for MockSapClient` を追加

```favnir
use sap_odata.query_client

impl SapQueryClient for MockSapClient {
    fn sales_orders_query(ctx: MockSapClient, q: SalesOrderQuery)
        -> Result<List<SalesOrder>, String> { ctx.sales_orders_result }
    fn business_partners_query(ctx: MockSapClient, q: BusinessPartnerQuery)
        -> Result<List<BusinessPartner>, String> { ctx.business_partners_result }
    fn materials_query(ctx: MockSapClient, q: MaterialQuery)
        -> Result<List<Material>, String> { ctx.materials_result }
    fn purchase_orders_query(ctx: MockSapClient, q: PurchaseOrderQuery)
        -> Result<List<PurchaseOrder>, String> { Result.err("not implemented") }
    fn journal_entries_query(ctx: MockSapClient, q: JournalEntryQuery)
        -> Result<List<JournalEntry>, String> { ctx.journal_entries_result }
}
```

> **設計判断**: `MockSapClient` 型定義への `purchase_orders_result` フィールド追加は行わない。`PurchaseOrder` のモックは `Result.err("not implemented")` で返す（他バージョンとの一貫性・スコープ管理のため）。`MockSapClient` 型定義自体（`mock.fav` の先頭）は変更しない。

### Step 5: `driver.rs` に `mod v91800_tests` を追加

`mod v91700_tests { ... }` の直後に追加：

```rust
#[cfg(test)]
mod v91800_tests {
    #[test]
    fn odata_query_builder_type_defined() {
        let content = std::fs::read_to_string("../runes/sap-odata/query.fav")
            .expect("runes/sap-odata/query.fav should exist");
        assert!(
            content.contains("public type ODataQueryBuilder"),
            "query.fav should define public type ODataQueryBuilder"
        );
    }
    #[test]
    fn build_url_function_defined() {
        let content = std::fs::read_to_string("../runes/sap-odata/query.fav")
            .expect("runes/sap-odata/query.fav should exist");
        assert!(
            content.contains("public fn build_url"),
            "query.fav should define public fn build_url"
        );
    }
    #[test]
    fn query_client_interface_defined() {
        let content = std::fs::read_to_string("../runes/sap-odata/query_client.fav")
            .expect("runes/sap-odata/query_client.fav should exist");
        assert!(
            content.contains("public interface SapQueryClient"),
            "query_client.fav should define public interface SapQueryClient"
        );
    }
    #[test]
    fn client_implements_sap_query_client() {
        let content = std::fs::read_to_string("../runes/sap-odata/client.fav")
            .expect("runes/sap-odata/client.fav should exist");
        assert!(
            content.contains("impl SapQueryClient for SapODataClient"),
            "client.fav should contain impl SapQueryClient for SapODataClient"
        );
    }
}
```

### Step 6: `cargo test` で全 pass 確認

```bash
cargo test 2>&1 | grep "test result"
```

期待: `4088 passed; 0 failed`

### Step 7: CI 事前確認

```bash
cargo clippy --locked -- -D warnings
./target/debug/fav fmt --check self/compiler.fav
./target/debug/fav fmt --check self/checker.fav
```

---

## 依存順序

```
Step 0（チェック）
  → Step 1（query.fav: ODataQueryBuilder + build_url）
  → Step 2（query_client.fav 新規作成）
  → Step 3（client.fav: impl SapQueryClient）
  → Step 4（mock.fav: impl SapQueryClient）
  → Step 5（driver.rs: テスト追加）
  → Step 6（cargo test）
  → Step 7（CI 事前確認）
```
