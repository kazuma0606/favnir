# Tasks: v95.8.0 — `fav sap-mock`

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `versions/current.md` の最新安定版が `v95.7.0` であることを確認する
- [x] `runes/sap-odata/batch.fav` に `BatchItemResult` が存在することを確認する（v95.7.0 完了済みの証拠）
- [x] `fav/src/driver.rs` に `mod v95700_tests` が存在することを確認する（v95.7.0 完了済みの証拠）
- [x] `cargo test 2>&1 | grep "test result"` を実行し、現在のテスト数が 4,180 であることを確認する（着手前ベースライン）
- [x] `fav/Cargo.toml` の version が `95.0.0` であることを確認する
  （スプリント内バージョンでは Cargo.toml を更新しない）

## T1: `driver.rs` に `SapMockServer` と `cmd_sap_mock` を追加

- [x] `SapMockServer` 構造体を追加する（`port: u16`、`fixtures: String` フィールド）
- [x] `cmd_sap_mock(server: &SapMockServer)` 関数を追加する
  （起動メッセージ + OData エンドポイント一覧を stdout に出力）

## T2: `main.rs` に `Some("sap-mock")` アームを追加

- [x] `Some("sap-mock")` アームを追加する（`Some("ai")` の直前）
- [x] `--port` フラグ（デフォルト 8080）と `--fixtures` フラグ（デフォルト `"runes/sap-odata/mock.fav"`）を解析する
- [x] `driver::cmd_sap_mock(&driver::SapMockServer { port, fixtures })` を呼び出す

## T3: `driver.rs` にテストを追加

- [x] `mod v95700_tests` の直後に `#[cfg(test)] mod v95800_tests { ... }` を追加する
- [x] `sap_mock_server_struct_defined` テストを追加する（`driver.rs` に `SapMockServer` が含まれる）
- [x] `sap_mock_cmd_defined` テストを追加する（`driver.rs` に `cmd_sap_mock` が含まれる）

## T4: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、4,182 tests, 0 failures であることを確認する

## T5: CHANGELOG / current.md 更新

- [x] `CHANGELOG.md` の先頭に `[v95.8.0]` エントリを追加する
- [x] `versions/current.md` の最新安定版を `v95.8.0` に更新する

## T6: tasks.md 更新

- [x] 本ファイルの Status を `COMPLETE` に変更する
- [x] T0〜T-last の全チェックボックスを `[x]` にする

## T-last: CI 事前確認（T4 の `cargo test` 全 pass 確認後に実施すること）

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
