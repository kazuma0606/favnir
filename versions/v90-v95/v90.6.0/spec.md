# Spec: v90.6.0 — `pipeline.fav` を `ctx.sap.*` で書き換え

## Background

v90.5.0 で `sap_odata.fav` の主要 4 関数が `ctx: AppCtx` スタイルに移行した。
しかし `infra/e2e-demo/sap-odata/pipeline.fav` の全 4 シナリオはいまだに旧スタイルを使っている。

```favnir
-- 変更前（旧スタイル）
fn sync_business_partners(ctx: AppCtx) -> Result<Int, String> {
    bind cfg      <- sap_odata.sap_config_from_env()
    bind partners <- sap_odata.business_partners(cfg, BusinessPartnerFilter { ... })
    bind _        <- ctx.s3.put_object(...)
```

`Ctx.build()` が `AppCtx.sap` に `SapODataClient` を自動注入するため、パイプライン関数内で
`sap_config_from_env()` を明示呼び出しする必要はなくなった。
`pipeline.fav` を `ctx.sap.*` スタイルへ書き換えることで、DI コンテナの設計意図が完成する。

### 各シナリオの変更方針

| シナリオ | 使用 SAP 関数 | `ctx.sap.*` 委譲可否 |
|---|---|---|
| 1: sync_business_partners | `business_partners` | ✓ `ctx.sap.business_partners(filter)` |
| 2: daily_sales_report | `sales_orders` | ✓ `ctx.sap.sales_orders(filter)` |
| 3: check_stock_vs_orders | `sales_orders` / `materials` | ✓ 両方委譲可 |
| 4: outstanding_payables | `journal_entries`（簡略化） | ✓ `ctx.sap.journal_entries(filter)` |

シナリオ 4 の旧実装は `purchase_orders`（`SapClient` interface 外）と `journal_entries` を組み合わせていた。
`purchase_orders` は `SapClient` interface に未定義のため、`sap_config_from_env()` なしでは `cfg` を渡せない。
v90.6.0 では `purchase_orders` の呼び出しを除去し、`journal_entries` のみを `ctx.sap.*` 経由で使用する簡略版に書き換える。
`purchase_orders` への対応は `SapClient` interface 拡張（v91.x.x 予定）で行う。

## Goals

1. `pipeline.fav` の全 4 シナリオから `bind cfg <- sap_odata.sap_config_from_env()` を削除する
2. `SapClient` interface 対応の関数（`business_partners` / `sales_orders` / `materials` / `journal_entries`）を `ctx.sap.*` スタイルに書き換える
3. シナリオ 4 の `purchase_orders`（SapClient 外）を除去し、`journal_entries` のみの簡略版に変更する
4. Rust テスト 2 件を追加して構造を保証する

## Syntax / API

```favnir
-- 変更後（シナリオ 1）
fn sync_business_partners(ctx: AppCtx) -> Result<Int, String> {
    bind partners <- ctx.sap.business_partners(BusinessPartnerFilter {
        country:       Option.some("JP"),
        category:      Option.none(),
        changed_after: Option.some("2026-08-01"),
        top:           Option.some(500)
    })
    bind json     <- Json.encode(partners)
    bind _        <- ctx.s3.put_object("favnir-sap-demo", "partners/latest.json", json)
    Result.ok(List.length(partners))
}

-- 変更後（シナリオ 4 — purchase_orders を除去し journal_entries のみに簡略化）
fn outstanding_payables(ctx: AppCtx) -> Result<List<JournalEntry>, String> {
    bind journals <- ctx.sap.journal_entries(JournalFilter {
        fiscal_year:       Option.some(2026),
        posting_date_from: Option.none(),
        company_code:      Option.none(),
        reference:         Option.none(),
        top:               Option.none()
    })
    bind json     <- Json.encode(journals)
    bind _        <- ctx.s3.put_object("favnir-sap-demo", "payables/outstanding.json", json)
    Result.ok(journals)
}
```

## Success Criteria

- `pipeline.fav` に `ctx.sap.` が含まれる（`ctx.sap.*` スタイルへの移行を確認）
- `pipeline.fav` に `sap_config_from_env` が含まれない（明示呼び出しの完全除去を確認）
- driver.rs の `pipeline_fav_uses_ctx_sap` テストが pass する
- driver.rs の `pipeline_fav_no_explicit_cfg` テストが pass する
- `cargo test` で 4,054 tests, 0 failures

## Files to Modify

| ファイル | 操作 |
|---|---|
| `infra/e2e-demo/sap-odata/pipeline.fav` | 全 4 シナリオを `ctx.sap.*` スタイルに書き換え |
| `fav/src/driver.rs` | `mod v90600_tests` 追加 |
| `CHANGELOG.md` | v90.6.0 エントリ追加 |
