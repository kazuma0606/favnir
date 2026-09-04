# Spec: v94.1.0 — `BatchRequest<T>` 型定義

## Background

OData `$batch` プロトコルは、複数の CRUD 操作を 1 回の HTTP リクエストでまとめて送信する仕組みである。
SAP S/4HANA OData API は `$batch` エンドポイントをサポートしており、大量のエンティティ更新を効率化できる。

v94.1.0 では、`$batch` 操作をモデル化する Favnir 型定義を `runes/sap-odata/batch.fav` に追加する。
実際の HTTP 送信（`ctx.sap.batch()`）は v94.2.0 で実装する。

## Goals

1. `BatchOperation<T>` — バッチ操作の種類（Create / Update / Delete）を ADT で表現する
2. `BatchRequest<T>` — バッチリクエスト型（entity_set + operations）を定義する
3. `BatchResponse<T>` — バッチレスポンス型（succeeded / failed）を定義する
4. `BatchError` — 個別操作の失敗情報を定義する

## Syntax/API Examples

```favnir
-- runes/sap-odata/batch.fav

-- バッチ操作の種類
type BatchOperation<T> =
    | BatchCreate(T)
    | BatchUpdate(String, T)    -- id, entity
    | BatchDelete(String)       -- id

-- バッチリクエスト型
type BatchRequest<T> = {
    entity_set:  String,
    operations:  List<BatchOperation<T>>
}

-- バッチレスポンス型
type BatchResponse<T> = {
    succeeded: List<T>,
    failed:    List<BatchError>
}

-- 個別操作失敗情報
type BatchError = {
    index:   Int,
    message: String
}
```

## Success Criteria

- `runes/sap-odata/batch.fav` が新規作成される
- `batch.fav` に `BatchRequest` が含まれる
- `cargo test 2>&1 | grep "test result"` が 4,144 tests, 0 failures を示す
- `cargo clippy --locked -- -D warnings` が pass する

## Error Codes

なし（型定義のみ、新規エラーコードは不要）

## Files to Modify / Create

| ファイル | 操作 | 内容 |
|---|---|---|
| `runes/sap-odata/batch.fav` | **新規作成** | BatchOperation / BatchRequest / BatchResponse / BatchError 型定義 |
| `fav/src/driver.rs` | **追加** | `mod v94100_tests`（2 件） |
