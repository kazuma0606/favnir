# Spec: v91.1.0 — `SelectClause<T>` 型定義

## Background

v91.0.0「SAP Ctx 統合 1.0」で `ctx.sap.*` パターンが完成した。
次の課題は OData クエリオプション（`$select` / `$expand` / `$filter`）を**型で表現**することである。

v91.1.0 では、OData `$select` に対応する `SelectClause<T>` 型を定義し、
`runes/sap-odata/query.fav` ファイルを新規作成する（以降のスプリントでここに型を積み上げる）。

## Goals

- `runes/sap-odata/query.fav` を新規作成する
- `SelectClause<T>` 型を定義する（フィールド名リストで OData `$select` を表現）
- `select_fields<T>` ヘルパー関数を追加する
- Rust テスト 2 件を追加する（4,065 → 4,067）

> **注意 — テスト数について**: ロードマップ計画値では v91.0.0 のベースを 4,063 として 4,065 を目標としているが、
> v91.0.0 の実装完了時点での実測値は **4,065**（`cargo test` で確認済み）。
> 本 spec は実測値を基準とし、目標テスト数を **4,067** とする。

## Syntax / API

```favnir
-- フィールド選択を表す型（OData $select に対応）
type SelectClause<T> = {
    fields: List<String>    -- T のフィールド名リスト
}

-- ヘルパー関数: フィールドリストから SelectClause を生成する
fn select_fields<T>(fields: List<String>) -> SelectClause<T> {
    SelectClause { fields: fields }
}

-- 使用例: BusinessPartner の 3 フィールドを選択
bind q <- select_fields<BusinessPartner>(["BusinessPartner", "BusinessPartnerName", "Country"])
-- q.fields == ["BusinessPartner", "BusinessPartnerName", "Country"]
```

## Success Criteria

- `runes/sap-odata/query.fav` が存在する（`odata_query_file_exists` テストで確認）
- `query.fav` に `SelectClause` が含まれる（`select_clause_type_defined` テストで確認）
- `query.fav` に `fn select_fields` が定義されている（コードレビューで確認、専用テストは v91.2.0 以降に追加予定）
- `cargo test` が 4,067 tests, 0 failures で通過する

## Error Codes

- なし（新規型定義のみ、チェッカー変更なし）

## Files to Modify / Create

| ファイル | 操作 | 内容 |
|---|---|---|
| `runes/sap-odata/query.fav` | **新規作成** | `SelectClause<T>` / `select_fields<T>` |
| `fav/src/driver.rs` | 追記 | `mod v91100_tests` 2 件 |
