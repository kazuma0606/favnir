# Tasks: v95.2.0 — `ctx.sap.delta_fetch<T>()`

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `versions/current.md` の最新安定版が `v95.0.0` であることを確認する
- [x] `cargo test` を実行し、4,166 tests, 0 failures を確認する（着手前ベースライン）
- [x] `runes/sap-odata/delta.fav` が存在することを確認する（v95.1.0 完了済みの証拠）
- [x] `fav/src/driver.rs` に `mod v95100_tests` が存在することを確認する（v95.1.0 完了済みの証拠）
- [x] `fav/Cargo.toml` の version が `95.0.0` であることを確認する

## T1: `types.fav` — `SapClient` interface に `delta_fetch<T>` を追加する

- [x] `runes/sap-odata/types.fav` の先頭 `use` 宣言に `use sap_odata.delta` を追加する
- [x] `SapClient` interface の `batch` シグネチャの直後に `delta_fetch<T>` シグネチャを追加する
  ```
  fn delta_fetch<T>(ctx: SapClient, entity_set: String, delta_link: Option<String>) -> Result<DeltaResult<T>, String>
  ```

## T2: `client.fav` — `delta_fetch<T>` スタブ実装を追加する

- [x] `runes/sap-odata/client.fav` の先頭 `use` 宣言群に `use sap_odata.delta` を追加する
- [x] `impl SapClient for SapODataClient` ブロックに `delta_fetch<T>` スタブを追加する
  （戻り値: `Result.err("delta_fetch: not yet implemented")`）

## T3: `sap_odata.fav` — `delta_fetch<T>` re-export を追加する

- [x] `runes/sap-odata/sap_odata.fav` の `$delta` セクションに `delta_fetch<T>` ラッパー関数を追加する

## T4: `driver.rs` に `mod v95200_tests` を追加する

- [x] `mod v95100_tests { ... }` の直後に `#[cfg(test)] mod v95200_tests { ... }` を追加する
- [x] `sap_client_interface_has_delta_fetch` テストを追加する（`types.fav` に `"delta_fetch"` が含まれる）
- [x] `client_fav_implements_delta_fetch` テストを追加する（`client.fav` に `"delta_fetch"` が含まれる）

## T5: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、4,168 tests, 0 failures であることを確認する

## T6: tasks.md を更新する

- [x] 本ファイルの Status を `COMPLETE` に変更する
- [x] T0〜T-last の全チェックボックスを `[x]` にする

## T-last: CI 事前確認（T5 の `cargo test` 全 pass 確認後に実施すること）

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
