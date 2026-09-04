# Spec: v91.8.0 — `ODataQueryBuilder` + SapQueryClient 統合

Status: 未着手

---

## Background

v91.1〜v91.7 で `SelectClause<T>` / `ExpandClause<T>` / `FilterExpr<T>` / `SalesOrderQuery` / `BusinessPartnerQuery` / `MaterialQuery` / `PurchaseOrderQuery` / `JournalEntryQuery` のすべてのクエリ型を整備した。

v91.8.0 では：
1. **`ODataQueryBuilder<T, Q>`** 共通ラッパー型と `build_url` ヘルパーを `query.fav` に追加する
2. **循環 dep を解消**し、クエリメソッドを持つ新インターフェース **`SapQueryClient`** を実装する

### 循環 dep 解消の設計

`types.fav` に `query.fav` を import すると循環 dep が発生する：
```
query.fav → sales_order.fav → types.fav → (if imports query.fav) → CIRCULAR
```

解消策：`SapClient`（`types.fav`）には手を加えず、**新規ファイル `runes/sap-odata/query_client.fav`** に `SapQueryClient` interface を定義する。

```
query_client.fav → query.fav → entity.fav → types.fav   ✅ 循環なし
```

`SapODataClient`（`client.fav`）と `MockSapClient`（`mock.fav`）が `SapQueryClient` を impl する。

> **AppCtx との統合**: `AppCtx.sap` フィールドは引き続き `SapClient` 型のまま。`ctx.sap.sales_orders_query(q)` 構文は v91.9.0 以降で `AppCtx` を `SapQueryClient` も実装する型に更新した際に使用可能になる。本バージョンでは `SapQueryClient` として明示的に型注釈した変数経由でメソッドを呼び出せる。

---

## Goals

1. `runes/sap-odata/query.fav` に `ODataQueryBuilder<T, Q>` 型と `build_url` ヘルパーを追加する
2. `runes/sap-odata/query_client.fav`（新規）に `SapQueryClient` interface を定義する
3. `client.fav` に `impl SapQueryClient for SapODataClient` を追加する
4. `mock.fav` に `impl SapQueryClient for MockSapClient` を追加する
5. Rust テスト 4 件を `driver.rs` に追加する

---

## Syntax / API Examples

```favnir
-- query.fav に追加: OData URL 生成の共通ラッパー
public type ODataQueryBuilder<T, Q> = {
    query:  Q,
    entity: String    -- エンティティセット名（"A_BusinessPartner", "A_SalesOrder" 等）
}

-- クエリを OData URL に変換するヘルパー（簡易実装: entity のみ結合）
public fn build_url<T, Q>(builder: ODataQueryBuilder<T, Q>, base_url: String) -> String {
    String.concat([base_url, "/", builder.entity])
}

-- query_client.fav（新規）: SapQueryClient interface
interface SapQueryClient {
    fn sales_orders_query(ctx: SapQueryClient, q: SalesOrderQuery) -> Result<List<SalesOrder>, String>
    fn business_partners_query(ctx: SapQueryClient, q: BusinessPartnerQuery) -> Result<List<BusinessPartner>, String>
    fn materials_query(ctx: SapQueryClient, q: MaterialQuery) -> Result<List<Material>, String>
    fn purchase_orders_query(ctx: SapQueryClient, q: PurchaseOrderQuery) -> Result<List<PurchaseOrder>, String>
    fn journal_entries_query(ctx: SapQueryClient, q: JournalEntryQuery) -> Result<List<JournalEntry>, String>
}

-- mock.fav に追加
impl SapQueryClient for MockSapClient {
    fn sales_orders_query(ctx: MockSapClient, q: SalesOrderQuery)
        -> Result<List<SalesOrder>, String> { ctx.sales_orders_result }
    -- ... 他4メソッドも同様
}
```

---

## Files to Modify / Create

| ファイル | 変更内容 |
|---|---|
| `runes/sap-odata/query.fav` | `ODataQueryBuilder<T, Q>` 型・`build_url` 関数を追加 |
| `runes/sap-odata/query_client.fav` | **新規作成**。`SapQueryClient` interface を定義 |
| `runes/sap-odata/client.fav` | `use sap_odata.query_client` 追加、`impl SapQueryClient for SapODataClient` 追加 |
| `runes/sap-odata/mock.fav` | `use sap_odata.query_client` 追加、`impl SapQueryClient for MockSapClient` 追加 |
| `fav/src/driver.rs` | `mod v91800_tests` 追加（2 件） |

---

## Success Criteria

- `cargo test` 全 pass: **4,088 tests, 0 failures**（4,084 + 4）
- `runes/sap-odata/query.fav` に `public type ODataQueryBuilder` が存在する
- `runes/sap-odata/query.fav` に `public fn build_url` が存在する
- `runes/sap-odata/query_client.fav` が新規作成されており `public interface SapQueryClient` を含む
- `runes/sap-odata/client.fav` に `impl SapQueryClient for SapODataClient` が含まれる
- `runes/sap-odata/mock.fav` に `impl SapQueryClient for MockSapClient` が含まれる
- `mod v91800_tests` 内の 4 テストが pass する:
  - `odata_query_builder_type_defined`
  - `build_url_function_defined`
  - `query_client_interface_defined`
  - `client_implements_sap_query_client`

---

## Note

> **CHANGELOG**: v91.8.0 は中間スプリントのため、CHANGELOG.md への記録は v92.0.0 宣言時にまとめて行う。

> **ロードマップのテスト数**: ロードマップ記載の完了条件（4081 + 4 = 4085）は計画値。実測は 4,084 ベース（v91.7.0 の code-reviewer 指摘対応で +1 追加）→ 4,088。ロードマップ一覧表・推移表の実測値反映は v92.0.0 宣言時に実施する。

> **`build_url` の簡易実装**: 本バージョンでは `entity` を URL に結合するのみ（`$filter` / `$select` 等の展開は future work）。完全実装は v92.x.x 以降で対応予定。

> **MDX ドキュメント更新**: `site/content/docs/runes/sap-odata.mdx` の更新は v92.0.0 宣言時にまとめて実施する。
