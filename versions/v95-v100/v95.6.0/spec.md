# Spec: v95.6.0 — Function Import / Action Import

## Background

OData v2/v4 の Function Import（冪等な GET 操作）と Action Import（副作用あり POST 操作）は、
エンティティ CRUD 以外の RPC スタイル呼び出しを可能にするメカニズム。

本バージョンでは `runes/sap-odata/rpc.fav` を新規作成し、
`function_import<T>` と `action_import` のスタブ関数を定義する。
ロードマップの `ctx.sap.function_import<T>` は SapClient interface への追加が必要だが、
v95.6.0 では既存スタイルに合わせた `cfg: SapConfig` 形式のスタブとして実装し、
ctx パターンへの移行は Out of Scope とする。

## Goals

1. `runes/sap-odata/rpc.fav` を新規作成する
2. `FunctionImportParam` 型エイリアスを定義する（`(String, String)` タプル）
3. `function_import<T>` スタブ関数を定義する
4. `action_import` スタブ関数を定義する
5. `fav/src/driver.rs` に `mod v95600_tests`（2 件）を追加する

## Syntax / API Examples

```favnir
-- Function Import パラメータ型
type FunctionImportParam = (String, String)

-- Function Import: 冪等な操作（GET）
-- NOTE: v95.6.0 は cfg スタイルのスタブ。ctx.sap.function_import<T>() への移行は後続バージョン。
bind result <- function_import<ReleaseResult>(cfg, "A_SalesOrder_Release", [
    ("SalesOrder", "0000000001")
])

-- Action Import: 副作用あり操作（POST）
bind _ <- action_import(cfg, "A_BusinessPartner_SetBlocked", [
    ("BusinessPartner", "BP001"),
    ("BusinessPartnerIsBlocked", "true")
])
```

## Files to Modify

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `runes/sap-odata/rpc.fav` | 新規作成 | `FunctionImportParam` 型 + `function_import<T>` / `action_import` スタブ |
| `fav/src/driver.rs` | 修正 | `mod v95600_tests`（2 件）追加 |

## Success Criteria

- `runes/sap-odata/rpc.fav` が存在する
- `rpc.fav` に `FunctionImportParam` が含まれる
- `rpc.fav` に `function_import` が含まれる
- `rpc.fav` に `action_import` が含まれる
- `cargo test` で 4,177 tests, 0 failures

## Out of Scope（次バージョン以降）

- `SapClient` interface への `function_import<T>` / `action_import` 追加（後続バージョン）
- `sap_odata.fav` への re-export 追加（後続バージョン）
- 実際の OData Function Import / Action Import HTTP 実装（後続バージョン）
