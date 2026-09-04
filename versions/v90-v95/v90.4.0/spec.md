# Spec: v90.4.0 — `Ctx.build` に SAP 設定注入を統合

## Background

v90.3.0 までで `SapClient` interface と `MockSapClient` を定義した。
本番用の実装として `SapODataClient` 型が必要であり、また `AppCtx` の構築（`Ctx.build`）に SAP 設定注入を組み込む必要がある。

現状の課題:
- `ctx.fav` ヘッダコメントに `Ctx.build` への言及があるが、関数が未実装
- `SapODataClient`（実 HTTP クライアント型）が未定義で、`SapClient` interface の本番実装がない
- ユーザーコードが `sap_config_from_env()` を手動で呼び出す必要があり、DI コンテナとしての `AppCtx` の設計意図に反する

`runes/sap-odata/client.fav` にはすでに `SapConfig` を受け取る HTTP 関数（`odata_get` / `odata_list`）が存在する。
`SapODataClient` はこれらを内部的に使用し、`SapClient` interface を満たす。

## Goals

1. `runes/sap-odata/` に `SapODataClient` 型を追加し、`impl SapClient for SapODataClient` を実装する
2. `runes/ctx/ctx.fav` に `Ctx.build` 関数を追加し、`sap_config_from_env()` を呼び出して `sap` フィールドを初期化する
3. Rust テスト 2 件を追加して構造を保証する

## Syntax / API

### SapODataClient（`runes/sap-odata/client.fav` に追加）

```favnir
-- 本番用 SAP OData クライアント型（v90.4.0）
-- SapClient interface の本番実装。SapConfig を保持し HTTP 経由で SAP S/4HANA にアクセスする。
type SapODataClient = {
    config: SapConfig
}

impl SapClient for SapODataClient {
    fn business_partners(ctx: SapODataClient, filter: BusinessPartnerFilter)
        -> Result<List<BusinessPartner>, String> {
        business_partners(ctx.config, filter)
    }
    fn business_partner_by_id(ctx: SapODataClient, id: String)
        -> Result<BusinessPartner, String> {
        business_partner_by_id(ctx.config, id)
    }
    fn sales_orders(ctx: SapODataClient, filter: SalesOrderFilter)
        -> Result<List<SalesOrder>, String> {
        sales_orders(ctx.config, filter)
    }
    fn materials(ctx: SapODataClient, filter: MaterialFilter)
        -> Result<List<Material>, String> {
        materials(ctx.config, filter)
    }
    fn journal_entries(ctx: SapODataClient, filter: JournalFilter)
        -> Result<List<JournalEntry>, String> {
        journal_entries(ctx.config, filter)
    }
}
```

### Ctx.build（`runes/ctx/ctx.fav` に追加）

```favnir
// AppCtx を本番設定で構築する（v90.4.0）
// 環境変数から各設定を読み込み、DI コンテナとして AppCtx を初期化する。
// SAP 設定は sap_config_from_env() で取得し SapODataClient に注入する。
public fn Ctx.build() -> Result<AppCtx, String> {
    bind sap_cfg <- sap_config_from_env()
    Result.ok(AppCtx {
        sap: SapODataClient { config: sap_cfg }
    })
}
```

## Success Criteria

- `runes/sap-odata/client.fav` に `impl SapClient for SapODataClient` が含まれる
- `runes/ctx/ctx.fav` に `Ctx.build` 関数と `sap` フィールドの初期化が含まれる
- driver.rs の `ctx_build_integrates_sap` テストが pass する
- driver.rs の `sap_odata_client_impl_exists` テストが pass する
- `cargo test` で 4050 tests, 0 failures

## Files to Modify / Create

| ファイル | 操作 |
|---|---|
| `runes/sap-odata/client.fav` | `SapODataClient` 型 + `impl SapClient for SapODataClient` を追加 |
| `runes/ctx/ctx.fav` | `Ctx.build` 関数を追加 |
| `fav/src/driver.rs` | `mod v90400_tests` 追加 |
| `CHANGELOG.md` | v90.4.0 エントリ追加 |
