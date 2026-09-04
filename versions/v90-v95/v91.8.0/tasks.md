# Tasks: v91.8.0 — `ODataQueryBuilder` + SapQueryClient 統合

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、4,084 tests, 0 failures を確認する（着手前ベースライン）
- [x] `fav/src/driver.rs` に `mod v91700_tests` が存在することを確認する（v91.7.0 完了済みの証拠）
- [x] `runes/sap-odata/query.fav` に `public type JournalEntryQuery` が含まれることを確認する
- [x] `runes/sap-odata/types.fav` が `query.fav` を import していないことを確認する（循環 dep チェック）
- [x] `fav/tmp/hello.fav` が存在することを確認する

## T1: `query.fav` に `ODataQueryBuilder` 型と `build_url` 関数を追加

- [x] `journal_entry_query()` 定義の後に `public type ODataQueryBuilder<T, Q> = { query: Q, entity: String }` を追加する
- [x] `ODataQueryBuilder` 型の直後に `public fn build_url<T, Q>(builder: ODataQueryBuilder<T, Q>, base_url: String) -> String { ... }` を追加する

## T2: `runes/sap-odata/query_client.fav` を新規作成

- [x] `runes/sap-odata/query_client.fav` を作成する
- [x] ファイル先頭に `use sap_odata.query` および各エンティティ use 文を追加する
- [x] `public interface SapQueryClient` を定義し、5 つのクエリメソッド（`sales_orders_query` / `business_partners_query` / `materials_query` / `purchase_orders_query` / `journal_entries_query`）を宣言する

## T3: `client.fav` に `impl SapQueryClient for SapODataClient` を追加

- [x] `client.fav` に `use sap_odata.query_client` を追加する
- [x] `impl SapQueryClient for SapODataClient` ブロックを追加し、5 メソッドすべてに `Result.err("not yet implemented")` スタブを実装する

## T4: `mock.fav` に `impl SapQueryClient for MockSapClient` を追加

- [x] `mock.fav` に `use sap_odata.query_client` を追加する
- [x] `impl SapQueryClient for MockSapClient` ブロックを追加し、既存の `_result` フィールドを返すスタブを実装する（`purchase_orders_result` が未定義の場合は `Result.err("not implemented")` を返す）

## T5: `driver.rs` に `mod v91800_tests` を追加

- [x] `mod v91700_tests { ... }` の直後に `#[cfg(test)] mod v91800_tests { ... }` を追加する
- [x] `odata_query_builder_type_defined` テストを実装する（`query.fav` に `"public type ODataQueryBuilder"` が含まれることを確認）
- [x] `build_url_function_defined` テストを実装する（`query.fav` に `"public fn build_url"` が含まれることを確認）
- [x] `query_client_interface_defined` テストを実装する（`query_client.fav` に `"public interface SapQueryClient"` が含まれることを確認）
- [x] `client_implements_sap_query_client` テストを実装する（`client.fav` に `"impl SapQueryClient for SapODataClient"` が含まれることを確認）

## T6: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、4,088 tests, 0 failures であることを確認する

## T7: tasks.md を COMPLETE に更新

- [x] 本ファイルの Status を `COMPLETE` に変更する
- [x] T0〜T-last の全チェックボックスを `[x]` にする（T0 の全項目を含む）

## Note

> 実装完了後は本ファイルの Status を `COMPLETE` に変更し、T0〜T-last の全チェックボックスを `[x]` にすること（T0 の全項目を含む）。

> **CHANGELOG**: v91.8.0 は中間スプリントのため、CHANGELOG.md への記録は v92.0.0 宣言時にまとめて行う。

> **ロードマップのテスト数**: ロードマップ記載の完了条件（4081 + 4 = 4085）は計画値。実測は 4,084 ベース（→ 4,088）。推移表・一覧表の実測値反映は v92.0.0 cleanup 時に実施。

> **ロードマップ設計変更**: ロードマップ v91.8.0 セクションは `SapClient` 直接拡張から `SapQueryClient`（`query_client.fav`）設計変更の注記を追加済み。`build_url` 簡易実装の注記も追加済み。

> **`build_url` 簡易実装**: 本バージョンでは entity 結合のみ。$filter/$select 等の URL 展開は将来バージョンで対応。

> **MDX ドキュメント更新**: `site/content/docs/runes/sap-odata.mdx` の更新は v92.0.0 宣言時にまとめて実施する。

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
