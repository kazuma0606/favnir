# Spec: v96.8.0 — 接続プール / キャッシュ / リトライ（`RetryPolicy` 型）

## Background

v96.7.0 までの SAP 接続実装には、本番運用に必要な接続管理機能（リトライ・タイムアウト・接続プール）が存在しなかった。
一時的な 503 / 429 エラー時に自動リトライする仕組みがなく、高負荷時の運用に不安があった。

v96.8.0 では `RetryPolicy` 型と `SapConnectionPool` 型を `runes/sap-odata/connection.fav` に追加し、
本番運用に必要な接続管理の型表現を整える。

## Goals

1. `runes/sap-odata/connection.fav` を新規作成する
   - `RetryPolicy` レコード型（`max_attempts`, `backoff_ms`, `retry_on_status`）を定義する
   - `SapConnectionPool` レコード型（`pool_size`, `timeout_ms`, `retry_policy`）を定義する
2. `fav/src/driver.rs` に `mod v96800_tests`（2 テスト）を追加する

## Favnir コード仕様

注: ロードマップでは `type RetryPolicy` と省略表記されているが、Rune 内の公開型定義は `public type` が正しい実装形。

```favnir
-- runes/sap-odata/connection.fav
-- SAP 接続プール / キャッシュ / リトライ設定（v96.8.0）

public type RetryPolicy = {
    max_attempts:    Int,
    backoff_ms:      Int,
    retry_on_status: List<Int>
}

public type SapConnectionPool = {
    pool_size:    Int,
    timeout_ms:   Int,
    retry_policy: RetryPolicy
}
```

## 使用例

```favnir
bind policy <- RetryPolicy {
    max_attempts:    3,
    backoff_ms:      500,
    retry_on_status: [503, 429]
}

bind pool <- SapConnectionPool {
    pool_size:    10,
    timeout_ms:   30000,
    retry_policy: policy
}
```

## Success Criteria

- `runes/sap-odata/connection.fav` が存在し `RetryPolicy` を含む
- `runes/sap-odata/connection.fav` が `SapConnectionPool` を含む
- `cargo test` で 4,207 tests, 0 failures

## Error Codes

新規エラーコードなし。

## Files to Modify

| ファイル | 変更内容 |
|---|---|
| `runes/sap-odata/connection.fav` | 新規作成（`RetryPolicy` 型 + `SapConnectionPool` 型） |
| `fav/src/driver.rs` | `mod v96800_tests`（2 テスト）を追加 |
