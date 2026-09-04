# Tasks: v92.7.0 — QueryBuilder<T> ベンチマーク（--sap-query）

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、4,109 tests, 0 failures を確認する（着手前ベースライン）
- [x] `fav/src/driver.rs` に `mod v92600_tests` が存在することを確認する（v92.6.0 完了済みの証拠）
- [x] `fav/src/driver.rs` の `BenchOpts` 構造体を Read し、既存フィールドを確認する
- [x] `fav/tmp/hello.fav` が存在することを確認する

## T1: `BenchOpts` に `sap_query` フィールドを追加する

- [x] `pub struct BenchOpts` に `pub sap_query: bool,` を追加する（v92.7.0 コメント付き）
- [x] `impl Default for BenchOpts` に `sap_query: false,` を追加する

## T2: `bench_sap_query` 関数を追加する

- [x] `cmd_bench_all` 関数の直後に `pub fn bench_sap_query() -> String` を追加する
- [x] 関数内に `fetch_all_pages` への言及（コメントまたは文字列）を含める
- [x] QueryBuilder チェーン速度（`chain_iters = 10_000`）の計測ループを実装する
- [x] `fetch_all_pages` スタブオーバーヘッド計測ループを実装する
- [x] フォーマット済み結果文字列を返す

## T3: `cmd_bench` に `--sap-query` 分岐を追加する

- [x] `cmd_bench` 関数内の `opts.all` チェックの前に `if opts.sap_query { ... }` 分岐を追加する
- [x] 分岐内で `bench_sap_query()` を呼び出し結果を表示して `true` を返す

## T4: `driver.rs` に `mod v92700_tests` を追加する

- [x] `mod v92600_tests { ... }` の直後に `#[cfg(test)] mod v92700_tests { ... }` を追加する
- [x] `bench_sap_query_flag_defined` テストを実装する（`src/driver.rs` に `bench_sap_query` 含有確認）
- [x] `bench_sap_query_measures_pagination` テストを実装する（`src/driver.rs` に `fetch_all_pages` 含有確認）

## T5: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、4,111 tests, 0 failures であることを確認する

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する

## T6: tasks.md を COMPLETE に更新する

- [x] 本ファイルの Status を `COMPLETE` に変更する
- [x] T0〜T-last の全チェックボックスを `[x]` にする
