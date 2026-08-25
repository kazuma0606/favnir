# Tasks: v87.6.0 — ページネーション基盤（`$top` / `$skip` / `@odata.nextLink`）

Status: COMPLETE

## code-reviewer 指摘と対応

- [MED] ロードマップの `fn` vs 実装 `public fn` の齟齬 → `odata_list_paged` / `odata_collect_all` は `public fn` が正しい（呼び出し元から可視が必要）。実装は正しく、ロードマップ表記が略記であることを確認。
- [MED] `contains("PagedResult")` が広すぎる → `contains("type PagedResult =")` に修正（driver.rs `paged_result_type_exists` テスト）。
- [LOW] `odata_collect_all` テスト未追加 → v87.8.0 の `odata_collect_all` 本実装時にテストを追加する（現版はスタブのみのため省略）。

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、3,985 tests, 0 failures を確認する
- [x] `fav/src/driver.rs` に `mod v87500_tests` が存在することを確認する（v87.5.0 完了済みの証拠）
- [x] `fav/Cargo.toml` の version が `87.0.0` であることを確認する（宣言バージョン v87.0.0 以降はスプリント中も 87.0.0 のまま）

## T1: `runes/sap-odata/types.fav` に `PagedResult` 型を追加

- [x] `ODataParams` 型の直後に `type PagedResult` （`items: List<String>`, `next_token: Option<String>`）を定義する

## T2: `runes/sap-odata/client.fav` に pagination 関数を追加

- [x] `odata_list()` の直後に `public fn odata_list_paged(cfg, entity_set, params, page_size)` スタブを追加する
- [x] `odata_list_paged()` の直後に `public fn odata_collect_all(cfg, entity_set, params, max_pages)` スタブを追加する

## T3: `runes/sap-odata/sap_odata.fav` を更新（re-export）

- [x] `odata_list()` ラッパーの直後に `public type PagedResult = types.PagedResult` を追加する
- [x] `odata_list_paged()` ラッパー関数を追加する
- [x] `odata_collect_all()` ラッパー関数を追加する
- Note: T3 は手作業確認（Rust テストの対象外。ロードマップ完了条件の 2 件テストは types.fav と client.fav を参照）

## T4: `driver.rs` に `mod v87600_tests` を追加

- [x] `mod v87500_tests { ... }` の直後に `#[cfg(test)] mod v87600_tests { ... }` を追加する
- [x] `paged_result_type_exists` テストを実装する（`types.fav` で `PagedResult` を確認）
- [x] `odata_list_paged_function_exists` テストを実装する（`client.fav` で `public fn odata_list_paged(` を確認）

## T5: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、3,987 tests, 0 failures であることを確認する（code-reviewer 対応後も再確認済み）

- Note: CHANGELOG / MILESTONE / site MDX 更新は v88.0.0 宣言バージョンでまとめて実施する（本バージョンは対象外）

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
