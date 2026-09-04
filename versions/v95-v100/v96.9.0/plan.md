# Plan: v96.9.0 — 安定化・コードフリーズ

## Step 1: `fav/src/driver.rs` に `mod v96900_tests` を追加

`mod v96800_tests` の直後に追加する。

テスト 1: `v96_sprint_new_rune_files_present` — v96.x.0 で追加した 3 Rune ファイル
（`clean_core.fav`、`cross_system.fav`、`connection.fav`）がすべて存在することを確認。

テスト 2: `v96_sprint_connection_fav_has_retry_on_status` — `connection.fav` に
`retry_on_status` フィールドが含まれることを確認。

```rust
let clean_core = std::fs::read_to_string("../runes/sap-odata/clean_core.fav")
    .expect("runes/sap-odata/clean_core.fav should exist");
let cross_system = std::fs::read_to_string("../runes/sap-odata/cross_system.fav")
    .expect("runes/sap-odata/cross_system.fav should exist");
let connection = std::fs::read_to_string("../runes/sap-odata/connection.fav")
    .expect("runes/sap-odata/connection.fav should exist");
```

## Step 2: `cargo test` で 4,209 tests, 0 failures を確認

## Step 3: CI チェック全通過を確認

- `cargo clippy --locked -- -D warnings`
- `./target/debug/fav fmt --check self/compiler.fav`
- `./target/debug/fav fmt --check self/checker.fav`

## Step 4: `CHANGELOG.md` に v96.9.0 エントリを追加

## Step 5: `versions/current.md` を v96.9.0 に更新
