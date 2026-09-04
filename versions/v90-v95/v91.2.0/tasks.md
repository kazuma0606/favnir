# Tasks: v91.2.0 — `ExpandClause<T>` 型定義

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、4,067 tests, 0 failures を確認する
- [x] `fav/src/driver.rs` に `mod v91100_tests` が存在することを確認する（v91.1.0 完了済みの証拠）
- [x] `runes/sap-odata/query.fav` に `SelectClause` が含まれることを確認する
- [x] `fav/tmp/hello.fav` が存在することを確認する

## T1: `runes/sap-odata/query.fav` に `ExpandClause<T>` を追記

- [x] `public type ExpandClause<T> = { navigation_properties: List<String> }` を追加する
- [x] `public fn expand_nav<T>(navigation_properties: List<String>) -> ExpandClause<T>` を追加する

## T2: `driver.rs` に `mod v91200_tests` を追加

- [x] `mod v91100_tests { ... }` の直後に `#[cfg(test)] mod v91200_tests { ... }` を追加する
- [x] `expand_clause_type_defined` テストを実装する（`query.fav` に `"ExpandClause"` が含まれることを確認）
- [x] `expand_nav_function_defined` テストを実装する（`query.fav` に `"expand_nav"` が含まれることを確認）

## T3: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "passed"` を実行し、4,069 tests, 0 failures であることを確認する

> 上記テスト全 pass 後、CI 事前確認（T-last）に進む。

## Note

> 実装完了後は本ファイルの Status を `COMPLETE` に変更し、T0〜T-last の全チェックボックスを `[x]` にすること。

> **CHANGELOG について**: v91.2.0 は中間スプリントのため、CHANGELOG.md への記録は **v92.0.0 宣言時にまとめて行う**。

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
