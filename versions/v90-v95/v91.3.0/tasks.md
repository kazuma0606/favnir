# Tasks: v91.3.0 — `FilterExpr<T>` 型定義

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、4,070 tests, 0 failures を確認する
- [x] `fav/src/driver.rs` に `mod v91200_tests` が存在することを確認する（v91.2.0 完了済みの証拠）
- [x] `runes/sap-odata/query.fav` に `ExpandClause` が含まれることを確認する
- [x] `fav/tmp/hello.fav` が存在することを確認する

## T1: `runes/sap-odata/query.fav` に `FilterExpr<T>` を追記

- [x] `public type FilterExpr<T> = | Eq(...) | Gt(...) | Lt(...) | And(...) | Or(...)` を追加する

## T2: `runes/sap-odata/query.fav` に `filter_to_odata_string<T>` を追記

- [x] `public fn filter_to_odata_string<T>(expr: FilterExpr<T>) -> String { match expr { ... } }` を追加する

## T3: `driver.rs` に `mod v91300_tests` を追加

- [x] `mod v91200_tests { ... }` の直後に `#[cfg(test)] mod v91300_tests { ... }` を追加する
- [x] `filter_expr_type_defined` テストを実装する（`query.fav` に `"FilterExpr"` が含まれることを確認）
- [x] `filter_to_odata_string_defined` テストを実装する（`query.fav` に `"filter_to_odata_string"` が含まれることを確認）

## T4: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "passed"` を実行し、4,072 tests, 0 failures であることを確認する

> 上記テスト全 pass 後、CI 事前確認（T-last）に進む。

## Note

> 実装完了後は本ファイルの Status を `COMPLETE` に変更し、T0〜T-last の全チェックボックスを `[x]` にすること。

> **CHANGELOG について**: v91.3.0 は中間スプリントのため、CHANGELOG.md への記録は **v92.0.0 宣言時にまとめて行う**。

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
