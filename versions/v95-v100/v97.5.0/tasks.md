# Tasks: v97.5.0 — SAP BTP Integration Suite connector（`iFlowClient`）

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `versions/current.md` の最新安定版が `v97.4.0` であることを確認する
- [x] `fav/src/driver.rs` に `mod v97400_tests` が存在することを確認する（v97.4.0 完了済みの証拠）
- [x] `cargo test 2>&1 | grep "test result"` を実行し、現在のテスト数が 4,221 であることを確認する（着手前ベースライン）
- [x] `fav/Cargo.toml` の version が `97.0.0` であることを確認する（パッチ版のため Cargo.toml version は宣言版 97.0.0 のまま）

## T1: `runes/sap-odata/iflow.fav` を新規作成

- [x] `IFlowClient` レコード型を定義する（`base_url` / `oauth_url` / `client_id`）
- [x] `IFlowMessage` レコード型を定義する（`headers: List<String>` / `body: String`）
- [x] `iflow_send` スタブ関数を定義する（`iflow_id` を含む固定文字列を返す）

## T2: `fav/src/driver.rs` に `mod v97500_tests` を追加

- [x] `mod v97400_tests` の直後に `#[cfg(test)] mod v97500_tests { ... }` を追加する
- [x] `iflow_fav_exists` テストを追加する
- [x] `iflow_fav_has_iflow_client` テストを追加する

## T3: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、4,223 tests, 0 failures であることを確認する

## T4: `CHANGELOG.md` に v97.5.0 エントリを追加

- [x] `CHANGELOG.md` の先頭に `[v97.5.0]` エントリを追加する

## T5: `versions/current.md` 更新

- [x] `最終更新:` ヘッダーを `v97.5.0` に更新する
- [x] 最新安定版を `v97.5.0` に更新する（テスト数 4,223）

## T-last: CI 事前確認（T3 の `cargo test` 全 pass 確認後・T4/T5 完了後に実施すること。`cargo test` 再実行は不要。Clippy / fmt のみ確認する）

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
