# Tasks: v92.2.0 — `.select` / `.expand` / `.filter` チェーン実装

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、4,097 tests, 0 failures を確認する（着手前ベースライン）
- [x] `fav/src/driver.rs` に `mod v92100_tests` が存在することを確認する（v92.1.0 完了済みの証拠）
- [x] `runes/sap-odata/query_builder.fav` に `public type QueryBuilder` が含まれることを確認する（v92.1.0 完了済みの証拠）
- [x] `runes/sap-odata/query_builder.fav` に `public fn query` が含まれることを確認する（v92.1.0 完了済みの証拠）
- [x] `runes/sap-odata/query.fav` の `SelectClause<T>`・`ExpandClause<T>` のフィールド名を確認する（`fields` / `navigation_properties`）
- [x] `fav/tmp/hello.fav` が存在することを確認する

## T1: `query_builder.fav` に 3 関数を追加する

- [x] `with_select<T>(builder, fields)` を追加する（`select_clause` フィールドを更新）
- [x] `with_expand<T>(builder, nav_props)` を追加する（`expand_clause` フィールドを更新）
- [x] `with_filter<T>(builder, expr)` を追加する（`filter_expr` フィールドを更新）
- [x] 各関数に `public` 修飾子を付ける

## T2: `driver.rs` に `mod v92200_tests` を追加する

- [x] `mod v92100_tests { ... }` の直後に `#[cfg(test)] mod v92200_tests { ... }` を追加する
- [x] `with_select_function_defined` テストを実装する
- [x] `with_expand_function_defined` テストを実装する
- [x] `with_filter_function_defined` テストを実装する

## T3: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、4,100 tests, 0 failures であることを確認する

## T4: tasks.md を COMPLETE に更新する

- [x] 本ファイルの Status を `COMPLETE` に変更する
- [x] T0〜T-last の全チェックボックスを `[x]` にする

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
