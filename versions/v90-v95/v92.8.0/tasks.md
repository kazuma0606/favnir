# Tasks: v92.8.0 — サイトドキュメント更新（QueryBuilder<T> パターン）

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、4,112 tests, 0 failures を確認する（着手前ベースライン）
- [x] `fav/src/driver.rs` に `mod v92700_tests` が存在することを確認する（v92.7.0 完了済みの証拠）
- [x] `site/content/docs/runes/sap-odata.mdx` を Read し、現状のセクション構成を確認する
- [x] `fav/tmp/hello.fav` が存在することを確認する

## T1: `sap-odata.mdx` にセクションを追加する

- [x] `## QueryBuilder<T> Fluent API（v92.1.0〜）` セクションを追記する（`query<T>()` + チェーン関数一覧・使用例）
- [x] `## Page<T> によるページネーション（v92.4.0〜）` セクションを追記する（`Page<T>` 型定義・`fetch_all_pages` の説明）
- [x] `## W060 N+1 lint（v92.5.0〜）` セクションを追記する（検出パターンと推奨コード。W060 が正しい）
- [x] `## fetch_all_pages パターン（v92.6.0 デモ）` セクションを追記する（pipeline_query.fav のコード例）
- [x] 全コード例で `bind q` 再束縛（E0018 違反）を避け `q1` / `q2` / `q3` を使っていることを確認する

## T2: `driver.rs` に `mod v92800_tests` を追加する

- [x] `mod v92700_tests { ... }` の直後に `#[cfg(test)] mod v92800_tests { ... }` を追加する
- [x] `docs_sap_odata_mentions_query_builder` テストを実装する（`../site/content/docs/runes/sap-odata.mdx` に `QueryBuilder` 含有確認）
- [x] `docs_sap_odata_mentions_fetch_all_pages` テストを実装する（同 MDX に `fetch_all_pages` 含有確認）

## T3: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、4,114 tests, 0 failures であることを確認する

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する

## T4: tasks.md を COMPLETE に更新する

- [x] 本ファイルの Status を `COMPLETE` に変更する
- [x] T0〜T-last の全チェックボックスを `[x]` にする
