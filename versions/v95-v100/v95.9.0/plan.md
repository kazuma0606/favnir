# Plan: v95.9.0 — 安定化・コードフリーズ

## Step 1: `fav/src/driver.rs` にスプリント総括テストを追加

1. `mod v95800_tests` の直後に `#[cfg(test)] mod v95900_tests { ... }` を追加する
2. `sprint1_sap_mock_registered` テスト: `include_str!("main.rs")` で
   `main.rs` に `"sap-mock"` が含まれることを確認する
   （`main.rs` は `fav/src/` 内のためコンパイル時解決の `include_str!` を使う）
3. `sprint1_rpc_fav_complete` テスト: `std::fs::read_to_string("../runes/sap-odata/rpc.fav")` で
   `FunctionImportParam` / `function_import` / `action_import` の 3 つが含まれることを確認する
   （`rpc.fav` は `fav/src/` 外のため `std::fs::read_to_string` を使う — 既存 v95600_tests と同パターン）

## Step 2: `cargo test` で全 pass 確認

- `cargo test 2>&1 | grep "test result"` を実行し、4,184 tests, 0 failures を確認する

## Step 3: CI 全通過確認

- `cargo clippy --locked -- -D warnings` を実行し pass を確認する
- `./target/debug/fav fmt --check self/compiler.fav` を実行し pass を確認する
- `./target/debug/fav fmt --check self/checker.fav` を実行し pass を確認する

## Step 4: CHANGELOG / current.md 更新

1. `CHANGELOG.md` の先頭に `[v95.9.0]` エントリを追加する
2. `versions/current.md` の最新安定版を `v95.9.0` に更新する

## Step 5: tasks.md 更新

- 本バージョンの `tasks.md` を COMPLETE ステータスに更新する
