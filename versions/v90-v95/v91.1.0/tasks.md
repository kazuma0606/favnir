# Tasks: v91.1.0 — `SelectClause<T>` 型定義

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、4,065 tests, 0 failures を確認する
- [x] `fav/src/driver.rs` に `mod v91000_tests` が存在することを確認する（v91.0.0 完了済みの証拠）
- [x] `fav/Cargo.toml` の version が `91.0.0` であることを確認する
- [x] `runes/sap-odata/query.fav` がまだ存在しないことを確認する
- [x] `fav/tmp/hello.fav` が存在することを確認する（cargo clean 後に消えないことを確認）

## T1: `runes/sap-odata/query.fav` 新規作成

- [x] `runes/sap-odata/query.fav` を新規作成する
- [x] ファイル先頭にモジュール説明コメント（`-- SAP OData クエリ型定義 v91.1.0〜`）を記述する
- [x] `public type SelectClause<T> = { fields: List<String> }` を定義する
- [x] `public fn select_fields<T>(fields: List<String>) -> SelectClause<T>` を定義する

## T2: `driver.rs` に `mod v91100_tests` を追加

- [x] `mod v91000_tests { ... }` の直後に `#[cfg(test)] mod v91100_tests { ... }` を追加する
- [x] `odata_query_file_exists` テストを実装する（`Path::new("../runes/sap-odata/query.fav").exists()` を確認）
- [x] `select_clause_type_defined` テストを実装する（`query.fav` に `"SelectClause"` が含まれることを確認）

## T3: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、4,067 tests, 0 failures であることを確認する

> 上記テスト全 pass 後、CI 事前確認（T-last）に進む。

## Note

> 実装完了後は本ファイルの Status を `COMPLETE` に変更し、T0〜T-last の全チェックボックスを `[x]` にすること。

> **CHANGELOG について**: v91.1.0 は中間スプリントのため、CHANGELOG.md への記録は **v92.0.0 宣言時にまとめて行う**。
> v91.1.0 単体での CHANGELOG 更新は不要。

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
