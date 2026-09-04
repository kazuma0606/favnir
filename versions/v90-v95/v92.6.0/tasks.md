# Tasks: v92.6.0 — QueryBuilder<T> E2E テストパイプライン

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、4,107 tests, 0 failures を確認する（着手前ベースライン）
- [x] `fav/src/driver.rs` に `mod v92500_tests` が存在することを確認する（v92.5.0 完了済みの証拠）
- [x] `runes/sap-odata/query_builder.fav` に `public fn fetch_all_pages` が含まれることを確認する（v92.4.0 完了済みの証拠）
- [x] `fav/tmp/hello.fav` が存在することを確認する

## T1: `pipeline_query.fav` を新規作成する

- [x] `infra/e2e-demo/sap-odata/pipeline_query.fav` を新規作成する
- [x] `import rune "sap-odata"` を先頭に記述する
- [x] `fn sync_business_partners_paged(ctx: AppCtx) -> Result<String, String>` を実装する
- [x] `bind q1 / q2 / q3` パターン（E0018 回避）で QueryBuilder チェーンを記述する
- [x] `fetch_all_pages` を呼び出すコードを含める（fetcher はスタブ `fn(c, b) { Result.err("...") }`）

## T2: `driver.rs` に `mod v92600_tests` を追加する

- [x] `mod v92500_tests { ... }` の直後に `#[cfg(test)] mod v92600_tests { ... }` を追加する
- [x] `pipeline_query_fav_exists` テストを実装する（`std::path::Path::new("../infra/...").exists()` でファイル存在確認）
- [x] `pipeline_query_uses_fetch_all_pages` テストを実装する（`fetch_all_pages` 含有確認）

## T3: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、4,109 tests, 0 failures であることを確認する

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する

## T4: tasks.md を COMPLETE に更新する

- [x] 本ファイルの Status を `COMPLETE` に変更する
- [x] T0〜T-last の全チェックボックスを `[x]` にする
