# Spec: v94.2.0 — `ChangeSet` + `ctx.sap.batch()` 実装

## Background

v94.1.0 で `BatchRequest<T>` / `BatchResponse<T>` / `BatchOperation<T>` / `BatchError` の型定義を追加した。
v94.2.0 では OData `$batch` のトランザクション境界を表す `ChangeSet<T>` 型と、
`SapClient` interface への `batch` メソッドを追加する。

`ChangeSet` は OData `$batch` multipart body において `--changeset_<id>` で区切られる
アトミック操作グループを Favnir の型として表現する。

## Goals

1. `ChangeSet<T>` — トランザクション内操作グループを `batch.fav` に追加する
2. `batch_request_builder` — `BatchRequest` を構築するヘルパー関数を `batch.fav` に追加する
3. `SapClient` interface に `batch` メソッドを追加する（`types.fav`）

## Syntax/API Examples

```favnir
-- runes/sap-odata/batch.fav への追加

-- ChangeSet: $batch 内でアトミックに実行される操作グループ
public type ChangeSet<T> = {
    operations: List<BatchOperation<T>>
}

-- BatchRequest を組み立てるヘルパー
public fn batch_request_builder<T>(entity_set: String, ops: List<BatchOperation<T>>) -> BatchRequest<T> {
    BatchRequest {
        entity_set: entity_set,
        operations: ops
    }
}
```

```favnir
-- runes/sap-odata/types.fav の SapClient interface への追加

interface SapClient {
    -- ... 既存メソッド ...
    fn batch(ctx: SapClient, req: BatchRequest<String>) -> Result<BatchResponse<String>, String>
}
```

```favnir
-- 使用例（ユーザーコード）
fn bulk_create_business_partners(
    ctx: AppCtx,
    bps: List<BusinessPartner>
) -> Result<BatchResponse<BusinessPartner>, String> {
    bind ops <- Result.ok(List.map(bps, fn(bp) { BatchCreate(bp) }))
    bind req <- Result.ok(batch_request_builder("A_BusinessPartner", ops))
    ctx.sap.batch(req)
}
```

## Success Criteria

- `runes/sap-odata/batch.fav` に `ChangeSet` が含まれる
- `runes/sap-odata/types.fav` の `SapClient` interface に `batch` メソッドが含まれる
- `cargo test 2>&1 | grep "test result"` が 4,146 tests, 0 failures を示す（着手前: 4,144）
- `cargo clippy --locked -- -D warnings` が pass する

## Error Codes

なし（型定義・interface 拡張のみ、新規エラーコードは不要）

## Files to Modify / Create

| ファイル | 操作 | 内容 |
|---|---|---|
| `runes/sap-odata/batch.fav` | **追記** | `ChangeSet<T>` 型 + `batch_request_builder` ヘルパー関数 |
| `runes/sap-odata/types.fav` | **追記** | `SapClient` interface に `batch` メソッドを追加 |
| `fav/src/driver.rs` | **追加** | `mod v94200_tests`（2 件） |
| `CHANGELOG.md` | **追記** | v94.2.0 エントリ |
