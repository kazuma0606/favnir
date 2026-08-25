# Plan: v85.5.0 — OData v4 HTTP クライアント基盤（`odata_get` / `odata_list`）

## Step 1: 前提確認

- `cargo test` を実行し、3,939 tests, 0 failures を確認する
- `fav/src/driver.rs` に `mod v85400_tests` が存在することを確認する（v85.4.0 完了済みの証拠）
- `runes/sap-odata/sap_odata.fav` と `runes/sap-odata/types.fav` が存在することを確認する

## Step 2: `runes/sap-odata/types.fav` に `ODataParams` 型を追加

既存ファイルの末尾に以下を追記する。

```favnir
-- OData v4 クエリパラメータ（v85.5.0）
type ODataParams = {
    filter:  Option<String>,
    select:  Option<String>,
    expand:  Option<String>,
    top:     Option<Int>,
    skip:    Option<Int>,
    orderby: Option<String>
}
```

## Step 3: `runes/sap-odata/client.fav` を新規作成

```favnir
-- SAP OData v4 HTTP クライアント基盤（v85.5.0）
use sap_odata.types

public fn odata_get(cfg: SapConfig, entity_set: String, key: String) -> Result<String, String> {
    bind url <- Result.ok(String.concat([cfg.base_url, "/", entity_set, "('", key, "')"]))
    Http.get_with_headers(url, [
        ("Authorization", basic_auth_header(cfg.username, cfg.password)),
        ("sap-client", cfg.client),
        ("Accept", "application/json")
    ])
}

public fn odata_list(cfg: SapConfig, entity_set: String, params: ODataParams) -> Result<String, String> {
    bind url <- Result.ok(String.concat([cfg.base_url, "/", entity_set, build_query_string(params)]))
    Http.get_with_headers(url, [
        ("Authorization", basic_auth_header(cfg.username, cfg.password)),
        ("sap-client", cfg.client),
        ("Accept", "application/json")
    ])
}

-- String.concat は String を直接返す（Result ではない）ため bind 不要
fn basic_auth_header(username: String, password: String) -> String {
    String.concat(["Basic ", Base64.encode(String.concat([username, ":", password]))])
}

-- v85.9.0 安定化バージョンで本実装予定のため、本バージョンでは骨格のみ（空文字列返し）
fn build_query_string(params: ODataParams) -> String {
    ""
}
```

## Step 4: `runes/sap-odata/sap_odata.fav` を更新

v85.4.0 の内容を以下で上書きする（`client.fav` を use し `odata_get` / `odata_list` / `ODataParams` を re-export）。

```favnir
-- sap-odata Rune エントリポイント（v85.5.0）
use sap_odata.types
use sap_odata.client

public type SapConfig   = types.SapConfig
public type ODataParams = types.ODataParams
public fn sap_config_from_env() -> Result<SapConfig, String> {
    types.sap_config_from_env()
}
public fn odata_get(cfg: SapConfig, entity_set: String, key: String) -> Result<String, String> {
    client.odata_get(cfg, entity_set, key)
}
public fn odata_list(cfg: SapConfig, entity_set: String, params: ODataParams) -> Result<String, String> {
    client.odata_list(cfg, entity_set, params)
}
```

## Step 5: `fav/src/driver.rs` に `mod v85500_tests` を追加

`mod v85400_tests { ... }` の直後に追加する。

```rust
#[cfg(test)]
mod v85500_tests {
    #[test]
    fn odata_list_function_exists_in_rune() {
        let content = std::fs::read_to_string("../runes/sap-odata/sap_odata.fav")
            .expect("runes/sap-odata/sap_odata.fav should exist");
        assert!(
            content.contains("odata_list"),
            "sap_odata.fav should define odata_list"
        );
    }

    #[test]
    fn odata_params_type_exists() {
        let content = std::fs::read_to_string("../runes/sap-odata/types.fav")
            .expect("runes/sap-odata/types.fav should exist");
        assert!(
            content.contains("ODataParams"),
            "types.fav should define ODataParams type"
        );
    }
}
```

## Step 6: `cargo test` で全 pass 確認

```
cargo test 2>&1 | grep "test result"
# 期待: 3941 tests, 0 failures
```

## Step 7: CHANGELOG 更新

`CHANGELOG.md` の先頭に v85.5.0 エントリを追加する。

## Step 8: CI 事前確認

以下はすべて `fav/` ディレクトリで実行する。

```
cargo clippy --locked -- -D warnings
./target/debug/fav fmt --check self/compiler.fav
./target/debug/fav fmt --check self/checker.fav
```
