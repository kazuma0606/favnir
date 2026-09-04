# Tasks: v92.9.0 — 安定化・コードフリーズ

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、4,114 tests, 0 failures を確認する（着手前ベースライン）
- [x] `fav/src/driver.rs` に `mod v92800_tests` が存在することを確認する（v92.8.0 完了済みの証拠）
- [x] `runes/sap-odata/query_builder.fav` が存在することを確認する
- [x] `fav/tmp/hello.fav` が存在することを確認する

## T1: `driver.rs` に `mod v92900_tests` を追加する

- [x] `mod v92800_tests { ... }` の直後に `#[cfg(test)] mod v92900_tests { ... }` を追加する
- [x] `query_builder_smoke_all_chains` テストを実装する（`query_builder.fav` に全 6 チェーン関数が含まれる）
- [x] `query_builder_page_type_in_rune_dir` テストを実装する（`query_builder.fav` が存在し `Page` が含まれる）

## T2: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、4,116 tests, 0 failures であることを確認する

## T3: ドキュメント・バージョン管理

- [x] `CHANGELOG.md` に v92.9.0 エントリを追加する
- [x] `versions/current.md` を v92.9.0 に更新する

## T4: ロードマップ実測値の反映

- [x] `roadmap-v92.1-v93.0.md` のテスト数推移表を実測値（v92.8.0: 4,114、v92.9.0: 4,116）に更新する

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する

## T5: tasks.md を COMPLETE に更新する

- [x] 本ファイルの Status を `COMPLETE` に変更する
- [x] T0〜T-last の全チェックボックスを `[x]` にする
