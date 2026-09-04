# Plan: v95.8.0 — `fav sap-mock`

## Step 1: `fav/src/driver.rs` に `SapMockServer` と `cmd_sap_mock` を追加

1. `SapMockServer` 構造体を定義する
   - `port: u16` フィールド（デフォルト 8080）
   - `fixtures: String` フィールド（fixtures ファイルパス）
2. `cmd_sap_mock(server: &SapMockServer)` 関数を定義する
   - 起動メッセージ: `SAP Mock Server listening on http://localhost:<port>`
   - エンドポイント一覧を stdout に出力（GET/POST A_BusinessPartner、POST /$batch）

## Step 2: `fav/src/main.rs` に `Some("sap-mock")` アームを追加

1. `Some("ai")` アームの直前に `Some("sap-mock")` アームを追加する
2. `--port` フラグ（デフォルト 8080）と `--fixtures` フラグ（デフォルト `"runes/sap-odata/mock.fav"`）を解析する
3. `driver::cmd_sap_mock(&driver::SapMockServer { port, fixtures })` を呼ぶ

## Step 3: `fav/src/driver.rs` にテストを追加

1. `mod v95700_tests` の直後に `#[cfg(test)] mod v95800_tests { ... }` を追加する
2. テストは `include_str!("driver.rs")` で driver.rs 自身を読み込んで文字列チェックを行う
   （`cargo test` の実行ディレクトリは `fav/src/` ではなく `fav/` のため、`std::fs::read_to_string("src/driver.rs")` でも可）
3. `sap_mock_server_struct_defined` テスト: `driver.rs` ソース内に `SapMockServer` が含まれることを確認
4. `sap_mock_cmd_defined` テスト: `driver.rs` ソース内に `cmd_sap_mock` が含まれることを確認

## Step 4: `cargo test` で全 pass 確認

- `cargo test 2>&1 | grep "test result"` を実行し、4,182 tests, 0 failures を確認する

## Step 5: CHANGELOG / current.md 更新

1. `CHANGELOG.md` の先頭に `[v95.8.0]` エントリを追加する
2. `versions/current.md` の最新安定版を `v95.8.0` に更新する

## Step 6: tasks.md 更新

- 本バージョンの `tasks.md` を COMPLETE ステータスに更新する
