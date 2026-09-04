# Tasks: v92.1.0 — `QueryBuilder<T>` 型定義

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、4,094 tests, 0 failures を確認する（着手前ベースライン）
- [x] `fav/src/driver.rs` に `mod v92000_tests` が存在することを確認する（v92.0.0 完了済みの証拠）
- [x] `runes/sap-odata/query.fav` に `ODataQueryBuilder` が含まれることを確認する（v91.8.0 完了済みの証拠）
- [x] `runes/sap-odata/query.fav` に `SelectClause` / `ExpandClause` / `FilterExpr` が含まれることを確認する（`query_builder.fav` のインポート前提）
- [x] `fav/tmp/hello.fav` が存在することを確認する

## T1: `runes/sap-odata/query_builder.fav` を新規作成する

- [x] `public type QueryBuilder<T>` を定義する（6 フィールド: select_clause / expand_clause / filter_expr / top_n / skip_n / order_by）
- [x] `public fn query<T>() -> QueryBuilder<T>` を定義する（全フィールド `Option.none()` で初期化）
- [x] ファイル先頭に `use sap_odata.query` を記述する

## T2: `driver.rs` に `mod v92100_tests` を追加する

- [x] `mod v92000_tests { ... }` の直後に `#[cfg(test)] mod v92100_tests { ... }` を追加する
- [x] `query_builder_file_exists` テストを実装する（`../runes/sap-odata/query_builder.fav` の存在確認）
- [x] `query_builder_type_defined` テストを実装する（`public type QueryBuilder` を含む確認）

## T3: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、4,096 tests, 0 failures であることを確認する

## T4: tasks.md を COMPLETE に更新する

- [x] 本ファイルの Status を `COMPLETE` に変更する
- [x] T0〜T-last の全チェックボックスを `[x]` にする

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
