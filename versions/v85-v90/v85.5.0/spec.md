# Spec: v85.5.0 — OData v4 HTTP クライアント基盤（`odata_get` / `odata_list`）

## Background

v85.4.0 で `runes/sap-odata/` の骨格（`rune.toml` / `sap_odata.fav`）を作成した。
本バージョンでは SAP OData v4 の GET / LIST クエリを実行する HTTP クライアント基盤を実装する。
`runes/sap-odata/client.fav` に `odata_get` / `odata_list` 関数を定義し、
`runes/sap-odata/types.fav` に `ODataParams` 型を追加する。

ロードマップ: `versions/roadmap/roadmap-v85.1-v86.0.md`（v85.5.0 セクション）

## Goals

- `runes/sap-odata/types.fav` に `ODataParams` 型を追加する
- `runes/sap-odata/client.fav` に `odata_get` / `odata_list` 関数を実装する
- `runes/sap-odata/sap_odata.fav` を更新して `client.fav` を use し `odata_list` を re-export する
- Rust テスト 2 件を追加して **3,941 tests** を達成する

## Files to Create / Modify

| ファイル | 操作 | 内容 |
|---|---|---|
| `runes/sap-odata/types.fav` | 追記 | `ODataParams` 型定義を追加 |
| `runes/sap-odata/client.fav` | 新規作成 | `odata_get` / `odata_list` 関数 |
| `runes/sap-odata/sap_odata.fav` | 更新 | `use sap_odata.client` + `odata_list` re-export |
| `fav/src/driver.rs` | 追記 | `mod v85500_tests`（テスト 2 件） |

## `ODataParams` 型定義（`types.fav` に追記）

```favnir
-- OData v4 クエリパラメータ
type ODataParams = {
    filter:  Option<String>,
    select:  Option<String>,
    expand:  Option<String>,
    top:     Option<Int>,
    skip:    Option<Int>,
    orderby: Option<String>
}
```

## `client.fav` 設計

```favnir
-- SAP OData v4 HTTP クライアント基盤（v85.5.0）
-- Basic 認証ヘッダーで SAP S/4HANA OData v4 エンドポイントに接続する。

use sap_odata.types

-- 単一エンティティ取得（GET /EntitySet('key')）
-- 戻り値は JSON 文字列（後続バージョンで型パース関数を追加する）
public fn odata_get(cfg: SapConfig, entity_set: String, key: String) -> Result<String, String> {
    bind url <- Result.ok(String.concat([cfg.base_url, "/", entity_set, "('", key, "')"]))
    Http.get_with_headers(url, [
        ("Authorization", basic_auth_header(cfg.username, cfg.password)),
        ("sap-client", cfg.client),
        ("Accept", "application/json")
    ])
}

-- コレクション取得（GET /EntitySet?$filter=...&$top=...）
-- 戻り値は JSON 文字列（OData v4 レスポンス形式）
public fn odata_list(cfg: SapConfig, entity_set: String, params: ODataParams) -> Result<String, String> {
    bind url <- Result.ok(String.concat([cfg.base_url, "/", entity_set, build_query_string(params)]))
    Http.get_with_headers(url, [
        ("Authorization", basic_auth_header(cfg.username, cfg.password)),
        ("sap-client", cfg.client),
        ("Accept", "application/json")
    ])
}

-- Basic 認証ヘッダー値を生成する（内部ヘルパー）
-- String.concat は String を直接返す（Result ではない）ため bind 不要
fn basic_auth_header(username: String, password: String) -> String {
    String.concat(["Basic ", Base64.encode(String.concat([username, ":", password]))])
}

-- ODataParams からクエリ文字列を組み立てる（内部ヘルパー）
fn build_query_string(params: ODataParams) -> String {
    ""
}
```

## `sap_odata.fav` 更新内容

`use sap_odata.client` を追加し、`odata_list` を public に re-export する。

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

## Success Criteria

- `cargo test` が **3,941 tests**, 0 failures
- `odata_list_function_exists_in_rune`:
  - `runes/sap-odata/sap_odata.fav` に `odata_list` が含まれる（ファイル内容チェック）
- `odata_params_type_exists`:
  - `runes/sap-odata/types.fav` に `ODataParams` が含まれる（ファイル内容チェック）

## Error Codes

新規エラーコードなし。

## 注記

- `build_query_string` の実装は骨格のみ（空文字列返し）— v85.9.0 安定化バージョンで実装予定
- `x-csrf-token` フェッチは POST/PATCH 操作用のため本バージョンでは対象外（v86.x 系で追加）
- テストはファイル内容の文字列チェック（HTTP 実行は不要）
- テストのファイルパス: `../runes/sap-odata/sap_odata.fav`（`cargo test` は `fav/` をカレントとして実行）
- MILESTONE.md / README.md の更新は v86.0.0 宣言バージョンで実施する
