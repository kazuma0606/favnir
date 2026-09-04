# Tasks: v98.2.0 — `BwQuery<T>` / `BwResult<T>` + `ctx.sap.bw_query()`

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `versions/current.md` の最新安定版が `v98.1.0` であることを確認する
- [x] `fav/src/driver.rs` に `mod v98100_tests` が存在することを確認する（v98.1.0 完了済みの証拠）
- [x] `cargo test -- --test-threads=1 2>&1 | grep "test result"` を実行し、現在のテスト数が 4,237 であることを確認する（着手前ベースライン）
- [x] `fav/Cargo.toml` の version が `98.0.0` であることを確認する（パッチ版のため Cargo.toml version は宣言版 98.0.0 のまま）

## T1: `runes/sap-odata/analytics.fav` に追記

- [x] `BwQuery<T>` ジェネリックレコード型を追記する（`info_provider` / `characteristics` / `key_figures` / `filters`）
- [x] `BwResult<T>` ジェネリックレコード型を追記する（`rows: List<T>` / `total: Int`）
- [x] `bw_query_mock<T>` ヘルパー関数を追記する（`BwResult { rows, total: List.length(rows) }` を返す）
- [x] コメントが `--` スタイルであることを確認する（`//` 不可）

## T2: `fav/src/driver.rs` に `mod v98200_tests` を追加

- [x] `mod v98100_tests` の直後に `mod v98200_tests`（2 テスト）を追加する:
  - `analytics_fav_has_bw_query`: `analytics.fav` に `BwQuery` が含まれることを確認
  - `analytics_fav_has_bw_result`: `analytics.fav` に `BwResult` が含まれることを確認
- [x] `mod v98200_tests` ブロック先頭に `// use super::* は不要（std::fs のみ使用）` という Rust コメントを 1 行追記する

## T3: `cargo test` で全 pass 確認

- [x] `cargo test -- --test-threads=1 2>&1 | grep "test result"` を実行し、4,239 tests, 0 failures であることを確認する

## T4: `CHANGELOG.md` に v98.2.0 エントリを追加

- [x] `CHANGELOG.md` の先頭に `[v98.2.0]` エントリを追加する

## T5: `versions/current.md` 更新

- [x] `最終更新:` ヘッダーを `v98.2.0` に更新する
- [x] 最新安定版を `v98.2.0` に更新する（テスト数 4,239）

<!-- site MDX ドキュメントは v98.8.0 で対応予定（本バージョンはスコープ外） -->
<!-- MILESTONE.md 更新は宣言版（v99.0.0）で対応予定（patch version は対象外） -->

## T-last: CI 事前確認（T3 の `cargo test` 全 pass 確認後・T4/T5 完了後に実施すること。`cargo test` 再実行は不要。Clippy / fmt のみ確認する）

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
