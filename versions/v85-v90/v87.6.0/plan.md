# Plan: v87.6.0 — ページネーション基盤

## 実装ステップ

### Step 1: `runes/sap-odata/types.fav` に `PagedResult` 型を追加

`ODataParams` 型の直後に以下を追加:

```favnir
-- ページネーション結果（v87.6.0）
type PagedResult = {
    items:      List<String>,
    next_token: Option<String>
}
```

### Step 2: `runes/sap-odata/client.fav` に pagination 関数を追加

`odata_list()` の直後に以下を追加:

```favnir
-- ページ単位取得（v87.6.0）
public fn odata_list_paged(
    cfg:        SapConfig,
    entity_set: String,
    params:     ODataParams,
    page_size:  Int
) -> Result<PagedResult, String> {
    Result.err("not implemented")
}

-- 全件収集（v87.6.0）
public fn odata_collect_all(
    cfg:        SapConfig,
    entity_set: String,
    params:     ODataParams,
    max_pages:  Int
) -> Result<List<String>, String> {
    Result.err("not implemented")
}
```

### Step 3: `runes/sap-odata/sap_odata.fav` を更新

既存の `odata_list()` ラッパーの直後に以下を追加:

```favnir
public type PagedResult = types.PagedResult
public fn odata_list_paged(cfg: SapConfig, entity_set: String, params: ODataParams, page_size: Int) -> Result<PagedResult, String> {
    client.odata_list_paged(cfg, entity_set, params, page_size)
}
public fn odata_collect_all(cfg: SapConfig, entity_set: String, params: ODataParams, max_pages: Int) -> Result<List<String>, String> {
    client.odata_collect_all(cfg, entity_set, params, max_pages)
}
```

### Step 4: `fav/src/driver.rs` に `mod v87600_tests` を追加

```rust
#[cfg(test)]
mod v87600_tests {
    #[test]
    fn paged_result_type_exists() {
        let content = std::fs::read_to_string("../runes/sap-odata/types.fav")
            .expect("runes/sap-odata/types.fav should exist");
        assert!(content.contains("PagedResult"), "PagedResult type should be defined in types.fav");
    }
    #[test]
    fn odata_list_paged_function_exists() {
        let content = std::fs::read_to_string("../runes/sap-odata/client.fav")
            .expect("runes/sap-odata/client.fav should exist");
        assert!(content.contains("public fn odata_list_paged("), "odata_list_paged function should be defined");
    }
}
```

### Step 5: `cargo test` で全 pass 確認（3985 + 2 = 3987）
