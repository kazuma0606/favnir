# Tasks: v92.4.0 — `Page<T>` 型 + `fetch_all_pages` 実装

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、4,103 tests, 0 failures を確認する（着手前ベースライン）
- [x] `fav/src/driver.rs` に `mod v92300_tests` が存在することを確認する（v92.3.0 完了済みの証拠）
- [x] `runes/sap-odata/query_builder.fav` に `public fn with_top` が含まれることを確認する（v92.3.0 完了済みの証拠）
- [x] `runes/sap-odata/query_builder.fav` に `public fn with_skip` が含まれることを確認する（v92.3.0 完了済みの証拠）
- [x] `runes/sap-odata/query_builder.fav` に `public fn with_order_by` が含まれることを確認する（v92.3.0 完了済みの証拠）
- [x] `fav/tmp/hello.fav` が存在することを確認する

## T1: `query_builder.fav` に `Page<T>` と `fetch_all_pages<T>` を追加する

- [x] `public type Page<T>` を定義する（3 フィールド: items / next_link / total）
- [x] `public fn fetch_all_pages<T>` を定義する（スタブ: `Result.err("not yet implemented")`）
- [x] `fetch_all_pages` の引数に `fetcher` 関数型パラメータを含める

## T2: `driver.rs` に `mod v92400_tests` を追加する

- [x] `mod v92300_tests { ... }` の直後に `#[cfg(test)] mod v92400_tests { ... }` を追加する
- [x] `page_type_defined` テストを実装する（`public type Page` 含有確認）
- [x] `fetch_all_pages_function_defined` テストを実装する（`public fn fetch_all_pages` 含有確認）

## T3: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、4,105 tests, 0 failures であることを確認する
- [x] `roadmap-v92.1-v93.0.md` の v92.4.0 エントリが `*_page` 延期・実測テスト数 4105 を反映済みであることを確認する（実装前に更新済みのため確認のみ）

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する

## T4: tasks.md を COMPLETE に更新する

- [x] 本ファイルの Status を `COMPLETE` に変更する
- [x] T0〜T-last の全チェックボックスを `[x]` にする
