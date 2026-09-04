# Spec: v96.3.0 — SAP → Parquet エクスポートパイプライン

## Background

v96.2.0 で `fav.toml [sap.environments]` マルチ環境設定の基盤が整った。
v96.3.0 では、SAP OData から取得した BusinessPartner エンティティを
Parquet ファイルに書き出し、DuckDB で分析する E2E パイプラインのデモを追加する。

これは「SAP + データレイク」統合のリファレンス実装であり、
v96.4.0 の SAP → Snowflake リアルタイム同期への足がかりとなる。

> **スコープ**: ロードマップのタイトルには「DuckDB」が含まれるが、v96.3.0 では Parquet 書き出しのみを実装する。
> DuckDB でのクエリ（`SELECT * FROM read_parquet(...)` 等）は将来バージョンのスコープとする。

## Goals

1. `infra/e2e-demo/sap-odata/pipeline_export.fav`（新規）を作成する
   - SAP BusinessPartner を取得して Parquet に書き出す `export_bp_to_parquet` pipeline
2. `driver.rs` に `mod v96300_tests`（2 テスト）を追加する

## Pipeline 仕様

```favnir
-- SAP → Parquet エクスポートパイプライン（v96.3.0）
-- BusinessPartner エンティティを SAP OData から取得し、Parquet ファイルに書き出す。
import rune "sap-odata"

pipeline export_bp_to_parquet !SapOData !Io {
    stage Fetch {
        bind bps <- ctx.sap.business_partners(BusinessPartnerFilter {
            country:       Option.some("JP"),
            category:      Option.none(),
            changed_after: Option.none(),
            top:           Option.some(1000)
        })
    }
    |> stage Write {
        bind _ <- ctx.io.write_parquet("output/business_partners.parquet", bps)
    }
}
```

## Success Criteria

- `infra/e2e-demo/sap-odata/pipeline_export.fav` が存在する
- ファイルに `export_bp_to_parquet` と `write_parquet` が含まれる
- `cargo test` で 4,194 tests, 0 failures

## Error Codes

新規エラーコードなし。

## Files to Modify

| ファイル | 変更内容 |
|---|---|
| `infra/e2e-demo/sap-odata/pipeline_export.fav` | 新規作成（SAP → Parquet エクスポートパイプライン） |
| `fav/src/driver.rs` | `mod v96300_tests`（2 テスト）を追加 |
