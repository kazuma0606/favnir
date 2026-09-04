# Plan: v96.6.0 — S/4HANA Clean Core REST API wrapper（`CleanCoreClient`）

## Step 1: `runes/sap-odata/clean_core.fav` を新規作成

`CleanCoreClient` レコード型と `CleanCoreClient.get` スタブ関数を定義する。

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

## Step 2: `fav/src/driver.rs` に `mod v96600_tests` を追加

`mod v96500_tests` の直後に追加する。

テスト 1: `clean_core_fav_exists` — `clean_core.fav` に `CleanCoreClient` が含まれることを確認。

テスト 2: `clean_core_fav_has_get_fn` — `clean_core.fav` に `CleanCoreClient.get` が含まれることを確認。

`runes/` 配下のファイルは `std::fs::read_to_string("../runes/sap-odata/clean_core.fav")` で読む
（`include_str!` ではなく `read_to_string`。他の runes テストと同じパターン）:

```rust
let content = std::fs::read_to_string("../runes/sap-odata/clean_core.fav")
    .expect("runes/sap-odata/clean_core.fav should exist");
```

## Step 3: `cargo test` で 4,203 tests, 0 failures を確認

## Step 4: `CHANGELOG.md` に v96.6.0 エントリを追加

## Step 5: `versions/current.md` を v96.6.0 に更新
