# Spec: v96.6.0 — S/4HANA Clean Core REST API wrapper（`CleanCoreClient`）

## Background

SAP S/4HANA Cloud の "Clean Core" 戦略では、従来の OData v2 に加えて JSON REST API
（SAP API Business Hub 上の REST エンドポイント群）が公開されつつある。
v96.5.0 までは `SapClient` interface が OData 専用であり、JSON REST API への型安全アクセスが不可能だった。

v96.6.0 では `CleanCoreClient` 型を `runes/sap-odata/clean_core.fav` に定義し、
SAP Clean Core REST API への型安全なアクセスを可能にする。

## Goals

1. `runes/sap-odata/clean_core.fav` を新規作成する
   - `CleanCoreClient` レコード型（`base_url: String`, `token: String`）を定義する
   - `CleanCoreClient.get` 関数スタブ（`path: String` を受け取り `String` を返す）を定義する
2. `fav/src/driver.rs` に `mod v96600_tests`（2 テスト）を追加する

## Favnir コード仕様

```favnir
-- runes/sap-odata/clean_core.fav
-- SAP S/4HANA Clean Core REST API wrapper（v96.6.0）

public type CleanCoreClient = {
    base_url: String,
    token:    String
}

-- Clean Core REST API への GET リクエスト（v96.6.0 スタブ）
-- path 例: "/API_BUSINESS_PARTNER/A_BusinessPartner('BP001')"
-- 戻り値は JSON 文字列（完全実装は将来バージョンで行う）
public fn CleanCoreClient.get(client: CleanCoreClient, path: String) -> String {
    String.concat(["GET ", client.base_url, path])
}
```

## CLI 使用例（将来参照用）

```favnir
-- 注: v96.6.0 は String スタブ。将来バージョンでジェネリック形式に拡張予定。
-- ロードマップ参考形: ctx.sap_clean_core.get<BusinessPartnerV2>(path)
bind result <- ctx.sap_clean_core.get(
    "/API_BUSINESS_PARTNER/A_BusinessPartner('BP001')"
)
```

## Success Criteria

- `runes/sap-odata/clean_core.fav` が存在し `CleanCoreClient` を含む
- `runes/sap-odata/clean_core.fav` が `CleanCoreClient.get` を含む
- `cargo test` で 4,203 tests, 0 failures

## Error Codes

新規エラーコードなし。

## Files to Modify

| ファイル | 変更内容 |
|---|---|
| `runes/sap-odata/clean_core.fav` | 新規作成（`CleanCoreClient` 型 + `CleanCoreClient.get` スタブ） |
| `fav/src/driver.rs` | `mod v96600_tests`（2 テスト）を追加 |
