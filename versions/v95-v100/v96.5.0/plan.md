# Plan: v96.5.0 — カスタム OData サービス対応（`--sap-service-name`）

## Step 1: `fav/src/sap_metadata.rs` に `generate_custom_service_header` 関数を追加

`generate_custom_service_header(service_name: &str) -> String` を追加する。

```rust
/// カスタム OData サービス名からファイルヘッダーコメントを生成する（v96.5.0）
/// service_name: "ZMY_CUSTOM_SRV" 等のサービス名
pub fn generate_custom_service_header(service_name: &str) -> String {
    format!(
        "-- Generated from SAP OData service: {}\n-- Do not edit manually.\n",
        service_name
    )
}
```

## Step 2: `fav/src/main.rs` に `--sap-service-name` フラグ解析を追加

`infer` コマンドハンドラ内の `--sap-metadata` 解析の近くに追加する。

```rust
let sap_service_name = args.iter()
    .position(|a| a == "--sap-service-name")
    .and_then(|i| args.get(i + 1))
    .map(|s| s.as_str())
    .unwrap_or("");
```

## Step 3: `fav/src/driver.rs` に `mod v96500_tests` を追加

`mod v96400_tests` の直後に追加する。

テスト 1: `sap_metadata_has_custom_service_header` — `sap_metadata.rs` に `generate_custom_service_header` が含まれることを確認。

テスト 2: `main_has_sap_service_name_flag` — `main.rs` に `--sap-service-name` が含まれることを確認。

## Step 4: `cargo test` で 4,199 tests, 0 failures を確認

## Step 5: `CHANGELOG.md` に v96.5.0 エントリを追加

## Step 6: `versions/current.md` を v96.5.0 に更新
