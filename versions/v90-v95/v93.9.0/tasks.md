# Tasks: v93.9.0 — 安定化・コードフリーズ

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、4,136 tests, 0 failures を確認する（着手前ベースライン）
- [x] `fav/src/driver.rs` に `mod v93800_tests` が存在することを確認する（v93.8.0 完了済みの証拠）
- [x] `fav/src/sap_metadata.rs` に `entity_type_to_favnir` と `enum_type_to_favnir` が存在することを確認する
- [x] `fav/tmp/hello.fav` が存在することを確認する

## T1: `driver.rs` に `mod v93900_tests` を追加する

- [x] `mod v93800_tests { ... }` の直後に `#[cfg(test)] mod v93900_tests { ... }` を追加する
- [x] `sap_metadata_smoke_url_and_file_cli` テストを実装する（`cli.fav` に `from sap` と `metadata-file` の両方が含まれる）
- [x] `sap_metadata_parser_handles_entity_and_enum` テストを実装する（`sap_metadata.rs` に `entity_type_to_favnir` と `enum_type_to_favnir` の両方が含まれる）

## T2: `cargo build` でコンパイル確認

- [x] `cargo build` を実行し、コンパイルエラーがないことを確認する

## T3: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、4,138 tests, 0 failures であることを確認する

## T4: CHANGELOG.md を更新する

- [x] `CHANGELOG.md` に v93.9.0 のエントリを追加する

## T5: ロードマップ本文を確認する

- [x] `versions/roadmap/roadmap-v93.1-v94.0.md` の v93.9.0 本文が `4136 + 2 = 4138` になっていることを確認する（v93.7.0 T6b で修正済み）

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する

## T7: tasks.md を COMPLETE に更新する

- [x] 本ファイルの Status を `COMPLETE` に変更する
- [x] T0〜T-last の全チェックボックスを `[x]` にする
