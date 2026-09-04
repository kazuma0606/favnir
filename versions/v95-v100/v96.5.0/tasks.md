# Tasks: v96.5.0 — カスタム OData サービス対応（`--sap-service-name`）

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `versions/current.md` の最新安定版が `v96.4.0` であることを確認する
- [x] `fav/src/driver.rs` に `mod v96400_tests` が存在することを確認する（v96.4.0 完了済みの証拠）
- [x] `cargo test 2>&1 | grep "test result"` を実行し、現在のテスト数が 4,197 であることを確認する（着手前ベースライン）
- [x] `fav/Cargo.toml` の version が `96.0.0` であることを確認する（パッチ版のため Cargo.toml version は宣言版 96.0.0 のまま）

## T1: `fav/src/sap_metadata.rs` に `generate_custom_service_header` を追加

- [x] `generate_custom_service_header(service_name: &str) -> String` 関数を追加する
- [x] 生成フォーマット: `"-- Generated from SAP OData service: {service_name}\n-- Do not edit manually.\n"`

## T2: `fav/src/main.rs` に `--sap-service-name` フラグ解析を追加

- [x] `infer` コマンドハンドラ内に `--sap-service-name` フラグ解析を追加する
- [x] 省略時はデフォルト空文字 `""` を使用する

## T3: `fav/src/driver.rs` に `mod v96500_tests` を追加

- [x] `mod v96400_tests` の直後に `#[cfg(test)] mod v96500_tests { ... }` を追加する
- [x] `sap_metadata_has_custom_service_header` テストを追加する（`sap_metadata.rs` に `generate_custom_service_header` が含まれる）
- [x] `main_has_sap_service_name_flag` テストを追加する（`main.rs` に `--sap-service-name` が含まれる）

## T4: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、4,199 tests, 0 failures であることを確認する

## T5: `CHANGELOG.md` に v96.5.0 エントリを追加

- [x] `CHANGELOG.md` の先頭に `[v96.5.0]` エントリを追加する

## T6: `versions/current.md` 更新

- [x] 最新安定版を `v96.5.0` に更新する（テスト数 4,199）

## T-last: CI 事前確認（T4 の `cargo test` 全 pass 確認後に実施すること）

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
