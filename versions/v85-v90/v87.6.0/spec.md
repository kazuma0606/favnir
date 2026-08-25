# Spec: v87.6.0 — ページネーション基盤（`$top` / `$skip` / `@odata.nextLink`）

## Background

SAP S/4HANA は 100 万件超の受注を持つケースがある。
現在の `odata_list()` は 1 リクエスト分のレスポンスを返すのみで、
大量データの安全な処理に対応できない。

本バージョンでは OData v4 の `$top` / `$skip` パラメータと
`@odata.nextLink` によるページネーション基盤を実装する。

## Goals

- `runes/sap-odata/types.fav` に `PagedResult` 型を追加する
- `runes/sap-odata/client.fav` に `odata_list_paged()` / `odata_collect_all()` 関数を追加する
  - `client.fav` は `use sap_odata.types` 済みのため `PagedResult` を直接参照できる（`ODataParams` と同パターン）
  - ロードマップのシグネチャ表記は `public` なしだが、`sap_odata.fav` から呼ぶため `public fn` が正しい
- `runes/sap-odata/sap_odata.fav` に re-export を追加する
- Rust テスト 2 件で型・関数の存在を確認する

## Syntax / API

```favnir
-- runes/sap-odata/types.fav に追加（v87.6.0）

-- ページネーション結果（@odata.nextLink 対応）
type PagedResult = {
    items:      List<String>,   -- JSON 行のリスト（各エンティティの JSON 文字列）
    next_token: Option<String>  -- @odata.nextLink から抽出した次ページ URL
}
```

```favnir
-- runes/sap-odata/client.fav に追加（v87.6.0）

-- ページ単位取得（$top / $skip）
-- page_size 件ずつ取得し PagedResult を返す
public fn odata_list_paged(
    cfg:        SapConfig,
    entity_set: String,
    params:     ODataParams,
    page_size:  Int
) -> Result<PagedResult, String> {
    Result.err("not implemented")
}

-- 全件収集（@odata.nextLink を再帰的にたどる）
-- max_pages: 安全装置として最大ページ数を指定（0 以下で無制限）
public fn odata_collect_all(
    cfg:        SapConfig,
    entity_set: String,
    params:     ODataParams,
    max_pages:  Int
) -> Result<List<String>, String> {
    Result.err("not implemented")
}
```

```favnir
-- runes/sap-odata/sap_odata.fav に追加

public type PagedResult = types.PagedResult
public fn odata_list_paged(cfg: SapConfig, entity_set: String, params: ODataParams, page_size: Int) -> Result<PagedResult, String> {
    client.odata_list_paged(cfg, entity_set, params, page_size)
}
public fn odata_collect_all(cfg: SapConfig, entity_set: String, params: ODataParams, max_pages: Int) -> Result<List<String>, String> {
    client.odata_collect_all(cfg, entity_set, params, max_pages)
}
```

## Success Criteria

1. `runes/sap-odata/types.fav` に `PagedResult` 型が定義されている
2. `runes/sap-odata/client.fav` に `public fn odata_list_paged(` が定義されている
3. `runes/sap-odata/sap_odata.fav` に `PagedResult` / `odata_list_paged` / `odata_collect_all` の文字列が含まれている（目視確認。Rust テスト対象外）
4. Rust テスト 2 件が pass: `paged_result_type_exists` / `odata_list_paged_function_exists`
5. `cargo test` 全 pass（3985 + 2 = 3987 tests）

## Files to Modify

- `runes/sap-odata/types.fav` — `PagedResult` 型を追加
- `runes/sap-odata/client.fav` — `odata_list_paged()` / `odata_collect_all()` 関数を追加
- `runes/sap-odata/sap_odata.fav` — `PagedResult` re-export + ラッパー関数 2 件を追加
- `fav/src/driver.rs` — `mod v87600_tests` 追加
