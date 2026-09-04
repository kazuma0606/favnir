# Spec: v96.4.0 — SAP → Snowflake リアルタイム同期

## Background

v96.3.0 で SAP → Parquet のエクスポートパイプラインを実装した。
v96.4.0 では v11.0 の Snowflake 統合と接続し、SAP から取得した BusinessPartner データを
Snowflake テーブルに直接ロードする E2E パイプラインのデモを追加する。

SAP × Snowflake の組み合わせは「SAP Multi-system 1.0」スプリントの中核であり、
v96.7.0 の Cross-system 型安全 JOIN の前提となる。

## Goals

1. `infra/e2e-demo/sap-odata/pipeline_snowflake_sync.fav`（新規）を作成する
   - SAP BusinessPartner を Snowflake にロードする `sync_bp_to_snowflake` pipeline
2. `driver.rs` に `mod v96400_tests`（2 テスト）を追加する

## Pipeline 仕様

```favnir
-- SAP → Snowflake リアルタイム同期パイプライン（v96.4.0）
-- BusinessPartner エンティティを SAP OData から取得し、Snowflake テーブルにロードする。
-- v11.0 Snowflake 統合（ctx.snowflake.execute_raw）と接続する。
import rune "sap-odata"
import rune "snowflake"

-- BusinessPartner を Snowflake 行形式（JSON 文字列）に変換するヘルパー
fn bp_to_snowflake_row(bp: BusinessPartner) -> String {
    Json.encode(bp)
}

pipeline sync_bp_to_snowflake !SapOData !Snowflake {
    stage Fetch {
        bind bps <- ctx.sap.business_partners(BusinessPartnerFilter {
            country:       Option.some("JP"),
            category:      Option.none(),
            changed_after: Option.none(),
            top:           Option.some(500)
        })
    }
    |> stage Load {
        bind rows <- List.map(bps, fn(bp) { bp_to_snowflake_row(bp) })
        bind _    <- ctx.snowflake.execute_raw(
            "INSERT INTO SAP_BUSINESS_PARTNERS SELECT * FROM VALUES ?",
            rows
        )
    }
}
```

> **注意**: `ctx.snowflake.execute_raw` は v11.0 で追加した Snowflake Rune の実行プリミティブ。
> `bp_to_snowflake_row` は JSON 変換ヘルパー（スタブとして `Json.encode` を使用）。

## Success Criteria

- `infra/e2e-demo/sap-odata/pipeline_snowflake_sync.fav` が存在する
- ファイルに `sync_bp_to_snowflake` と `execute_raw` が含まれる
- ファイルに `bp_to_snowflake_row` ヘルパー関数の定義が含まれる
- `cargo test` で 4,197 tests, 0 failures

## Error Codes

新規エラーコードなし。

## Files to Modify

| ファイル | 変更内容 |
|---|---|
| `infra/e2e-demo/sap-odata/pipeline_snowflake_sync.fav` | 新規作成（SAP → Snowflake 同期パイプライン） |
| `fav/src/driver.rs` | `mod v96400_tests`（2 テスト）を追加 |
