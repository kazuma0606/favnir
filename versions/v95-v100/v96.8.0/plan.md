# Plan: v96.8.0 — 接続プール / キャッシュ / リトライ（`RetryPolicy` 型）

## Step 1: `runes/sap-odata/connection.fav` を新規作成

`RetryPolicy` と `SapConnectionPool` のレコード型を定義する。

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

## Step 2: `fav/src/driver.rs` に `mod v96800_tests` を追加

`mod v96700_tests` の直後に追加する。

テスト 1: `connection_fav_has_retry_policy` — `connection.fav` に `RetryPolicy` が含まれることを確認。

テスト 2: `connection_fav_has_sap_connection_pool` — `connection.fav` に `SapConnectionPool` が含まれることを確認。

`runes/` 配下のファイルは `std::fs::read_to_string("../runes/sap-odata/connection.fav")` で読む:

```rust
let content = std::fs::read_to_string("../runes/sap-odata/connection.fav")
    .expect("runes/sap-odata/connection.fav should exist");
```

## Step 3: `cargo test` で 4,207 tests, 0 failures を確認

## Step 4: `CHANGELOG.md` に v96.8.0 エントリを追加

## Step 5: `versions/current.md` を v96.8.0 に更新
