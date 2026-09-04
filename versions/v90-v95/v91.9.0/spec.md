# Spec: v91.9.0 — 安定化・コードフリーズ

Status: 未着手

---

## Background

v91.1〜v91.8 で SAP OData Query 基盤の全構成要素を整備した：

| バージョン | 内容 |
|---|---|
| v91.1〜v91.3 | `SelectClause<T>` / `ExpandClause<T>` / `FilterExpr<T>` |
| v91.4〜v91.7 | `SalesOrderQuery` / `BusinessPartnerQuery` / `MaterialQuery` / `PurchaseOrderQuery` / `JournalEntryQuery` |
| v91.8 | `ODataQueryBuilder<T, Q>` / `build_url` / `SapQueryClient` interface |

v91.9.0 は v91.1〜v91.8 の全機能を通しで確認する**最終安定化スプリント**。
新機能追加はなし。バグ修正のみ受け入れる。

---

## Goals

1. `query.fav` に全クエリ型（5 型）・全ヘルパー関数が揃っていることを Rust テストで担保する
2. `filter_to_odata_string` が存在することを Rust テストで再確認する（v91.3.0 の実装を新規テスト名で担保）
3. `driver.rs` に `mod v91900_tests`（2 件）を追加する

---

## Syntax / API Examples

v91.9.0 では新規 API の追加はなし。以下はスモークテストが確認する既存 API の代表例：

```favnir
-- query.fav に存在するすべてのクエリ型
bind so_q  <- sales_order_query()          -- SalesOrderQuery
bind bp_q  <- business_partner_query()     -- BusinessPartnerQuery
bind mat_q <- material_query()             -- MaterialQuery
bind po_q  <- purchase_order_query()       -- PurchaseOrderQuery
bind je_q  <- journal_entry_query()        -- JournalEntryQuery

-- FilterExpr のシリアライズ（filter_to_odata_string）
bind expr <- FilterExpr.Eq("Country", "JP")
bind s    <- filter_to_odata_string<BusinessPartner>(expr)
-- s = "Country eq 'JP'"
```

---

## Files to Modify / Create

| ファイル | 変更内容 |
|---|---|
| `fav/src/driver.rs` | `mod v91900_tests` 追加（2 件） |

新規ファイル作成なし。

---

## Success Criteria

- `cargo test` 全 pass: **4,090 tests, 0 failures**（4,088 + 2）
- `mod v91900_tests` 内の 2 テストが pass する:
  - `odata_query_smoke_all_query_types`: `query.fav` に 5 クエリ型すべてが含まれる
  - `odata_filter_expr_serializable`: `query.fav` に `filter_to_odata_string` が含まれる

---

## Note

> **CHANGELOG**: v91.9.0 は中間スプリントのため、CHANGELOG.md への記録は v92.0.0 宣言時にまとめて行う。

> **ロードマップのテスト数**: ロードマップ記載の完了条件（4088 + 2 = 4090）は更新済み。一覧表・推移表の実測値反映は v92.0.0 宣言時に実施する。

> **v92.0.0 cleanup 引き継ぎ事項**: v92.0.0 tasks.md 作成時に「ロードマップの `PurchaseOrderQuery` 欠落修正済み確認」および「推移表の全実測値反映」を tasks に含めること。

> **バグ修正のみ**: v91.9.0 は安定化スプリントのため、スモークテスト 2 件以外の新規追加は行わない。
