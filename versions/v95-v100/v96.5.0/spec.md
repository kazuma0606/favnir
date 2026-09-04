# Spec: v96.5.0 — カスタム OData サービス対応（`--sap-service-name`）

## Background

v94.x で `fav infer --from sap` による SAP $metadata XML からの型自動生成を実装した。
しかし既存実装は標準 S/4HANA サービス（API_BUSINESS_PARTNER 等）を前提とした設計であり、
顧客独自のカスタム OData サービス（`ZMY_CUSTOM_SRV` 等）への対応が不完全だった。

v96.5.0 では `--sap-service-name` フラグを追加し、任意のカスタム OData サービス名を
指定して型生成できるようにする。

## Goals

1. `fav/src/sap_metadata.rs` に `generate_custom_service_header` 関数を追加する
   - カスタムサービス名を受け取り、生成 Favnir ファイルのヘッダーコメントを返す
2. `fav/src/main.rs` の `infer` コマンドハンドラに `--sap-service-name` フラグ解析を追加する
3. `fav/src/driver.rs` に `mod v96500_tests`（2 テスト）を追加する

## CLI 仕様

```
$ fav infer --from sap \
    --sap-metadata https://my-sap/sap/opu/odata/sap/ZMY_CUSTOM_SRV/$metadata \
    --sap-service-name ZMY_CUSTOM_SRV \
    --output runes/sap-odata/custom_service.fav
```

`--sap-service-name` が省略された場合はデフォルトサービス名（空文字 or `"SAP_ODATA"`）を使用する。

## 実装仕様（Rust）

```rust
/// カスタム OData サービス名からファイルヘッダーコメントを生成する（v96.5.0）
/// service_name: "ZMY_CUSTOM_SRV" 等のサービス名
/// 生成例: "-- Generated from SAP OData service: ZMY_CUSTOM_SRV\n-- Do not edit manually.\n"
pub fn generate_custom_service_header(service_name: &str) -> String {
    format!(
        "-- Generated from SAP OData service: {}\n-- Do not edit manually.\n",
        service_name
    )
}
```

`main.rs` での `--sap-service-name` 解析（`infer` コマンド内）:

```rust
let sap_service_name = args.iter()
    .position(|a| a == "--sap-service-name")
    .and_then(|i| args.get(i + 1))
    .map(|s| s.as_str())
    .unwrap_or("");
```

## Success Criteria

- `fav/src/sap_metadata.rs` に `generate_custom_service_header` 関数が含まれる
- `fav/src/main.rs` に `--sap-service-name` の文字列が含まれる
- `cargo test` で 4,199 tests, 0 failures

## Error Codes

新規エラーコードなし。

## Files to Modify

| ファイル | 変更内容 |
|---|---|
| `fav/src/sap_metadata.rs` | `generate_custom_service_header(service_name: &str) -> String` 関数を追加 |
| `fav/src/main.rs` | `infer` コマンドハンドラに `--sap-service-name` フラグ解析を追加 |
| `fav/src/driver.rs` | `mod v96500_tests`（2 テスト）を追加 |
