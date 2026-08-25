# Tasks: v89.9.0 — 安定化・コードフリーズ

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、4,035 tests, 0 failures を確認する
- [x] `fav/src/driver.rs` に `mod v89800_tests` が存在することを確認する（v89.8.0 完了済みの証拠）
- [x] `fav/Cargo.toml` の version が `89.0.0` であることを確認する
- [x] `runes/sap-odata/` ディレクトリが存在することを確認する（`sap_integration_rune_registry_deployed` テストの前提）
  - テストは `fav/` cwd から `"../runes/sap-odata"` として参照するため、実際のパスは `C:\Users\yoshi\favnir\runes\sap-odata\`

## T1: `runes/sap-odata/` ディレクトリの存在確認

- [x] `ls ../runes/sap-odata/` を実行して存在を確認する
- [x] 存在しない場合は作成する（空ディレクトリでもテストは pass する）

## T2: `mod v89900_tests` を `driver.rs` に追加

- [x] `mod v89800_tests { ... }` の直後に `#[cfg(test)] mod v89900_tests { ... }` を追加する
- [x] `sap_all_four_scenarios_in_pipeline` テストを実装する（4 シナリオ全ての関数名が `pipeline.fav` に存在することを確認）
- [x] `sap_integration_rune_registry_deployed` テストを実装する（`"../runes/sap-odata"` ディレクトリの存在確認）

## T3: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、4,037 tests, 0 failures であることを確認する

> 上記テスト全 pass 後、CI 事前確認（T-last）に進む。

## Note

> 実装完了後は本ファイルの Status を `COMPLETE` に変更し、T0〜T-last の全チェックボックスを `[x]` にすること。

CHANGELOG / MILESTONE 更新は v90.0.0 宣言バージョンでまとめて実施する（本バージョンは対象外）。

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
