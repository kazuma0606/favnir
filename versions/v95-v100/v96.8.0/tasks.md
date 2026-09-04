# Tasks: v96.8.0 — 接続プール / キャッシュ / リトライ（`RetryPolicy` 型）

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `versions/current.md` の最新安定版が `v96.7.0` であることを確認する
- [x] `fav/src/driver.rs` に `mod v96700_tests` が存在することを確認する（v96.7.0 完了済みの証拠）
- [x] `cargo test 2>&1 | grep "test result"` を実行し、現在のテスト数が 4,205 であることを確認する（着手前ベースライン）
- [x] `fav/Cargo.toml` の version が `96.0.0` であることを確認する（パッチ版のため Cargo.toml version は宣言版 96.0.0 のまま）

## T1: `runes/sap-odata/connection.fav` を新規作成

- [x] ファイル冒頭にコメントヘッダーを記述する（ファイル名・バージョン）
- [x] `public type RetryPolicy = { max_attempts: Int, backoff_ms: Int, retry_on_status: List<Int> }` を定義する
- [x] `public type SapConnectionPool = { pool_size: Int, timeout_ms: Int, retry_policy: RetryPolicy }` を定義する

## T2: `fav/src/driver.rs` に `mod v96800_tests` を追加

- [x] `mod v96700_tests` の直後に `#[cfg(test)] mod v96800_tests { ... }` を追加する
- [x] `connection_fav_has_retry_policy` テストを追加する（`connection.fav` に `RetryPolicy` が含まれる）
- [x] `connection_fav_has_sap_connection_pool` テストを追加する（`connection.fav` に `SapConnectionPool` が含まれる）
- [x] テスト内のファイルパスが `std::fs::read_to_string("../runes/sap-odata/connection.fav")` であることを確認する

## T3: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、4,207 tests, 0 failures であることを確認する

## T4: `CHANGELOG.md` に v96.8.0 エントリを追加

- [x] `CHANGELOG.md` の先頭に `[v96.8.0]` エントリを追加する

## T5: `versions/current.md` 更新

- [x] 最新安定版を `v96.8.0` に更新する（テスト数 4,207）

## T-last: CI 事前確認（T3 の `cargo test` 全 pass 確認後に実施すること）

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
