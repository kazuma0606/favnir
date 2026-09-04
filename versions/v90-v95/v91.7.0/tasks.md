# Tasks: v91.7.0 — `JournalEntryQuery` 実装

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、4,081 tests, 0 failures を確認する
- [x] `fav/src/driver.rs` に `mod v91600_tests` が存在することを確認する（v91.6.0 完了済みの証拠）
- [x] `runes/sap-odata/query.fav` に `public type PurchaseOrderQuery` が含まれることを確認する
- [x] `runes/sap-odata/journal_entry.fav` が `query.fav` を import していないことを確認する（循環 dep チェック）
- [x] `runes/sap-odata/types.fav` が `query.fav` を import していないことを確認する
- [x] `fav/tmp/hello.fav` が存在することを確認する

## T1: `query.fav` に `use sap_odata.journal_entry` を追加

- [x] `use sap_odata.purchase_order` の直後に `use sap_odata.journal_entry` を追記する

## T2: `query.fav` に `JournalEntryQuery` 型を追記

- [x] `purchase_order_query()` 定義の後に `public type JournalEntryQuery = { filter, select, fiscal_year, top, skip }` を追加する（`fiscal_year: Option<String>` を含む点が他クエリ型と異なる）

## T3: `query.fav` に `journal_entry_query()` ビルダーを追記

- [x] `JournalEntryQuery` 型の直後に `public fn journal_entry_query() -> JournalEntryQuery { ... Option.none() ... }` を追加する（`fiscal_year: Option.none()` を含む）

## T4: SapClient interface 拡張（延期）

> **SKIP（v91.8.0 へ延期）**: 循環 dep 制約により、`SapClient` への `journal_entries_query` 追加は v91.8.0 で一括実施する。

- [x] 延期決定を記録する → SKIP

## T5: `driver.rs` に `mod v91700_tests` を追加

- [x] `mod v91600_tests { ... }` の直後に `#[cfg(test)] mod v91700_tests { ... }` を追加する
- [x] `journal_entry_query_type_defined` テストを実装する（`query.fav` に `"public type JournalEntryQuery"` が含まれることを確認）
- [x] `journal_entry_query_builder_defined` テストを実装する（`query.fav` に `"public fn journal_entry_query"` が含まれることを確認）

## T6: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、4,083 tests, 0 failures であることを確認する

## T7: tasks.md を COMPLETE に更新

- [x] 本ファイルの Status を `COMPLETE` に変更する
- [x] T0〜T-last の全チェックボックスを `[x]` にする（T0 の全項目を含む）

## Note

> 実装完了後は本ファイルの Status を `COMPLETE` に変更し、T0〜T-last の全チェックボックスを `[x]` にすること（T0 の全項目を含む）。

> **CHANGELOG**: v91.7.0 は中間スプリントのため、CHANGELOG.md への記録は v92.0.0 宣言時にまとめて行う。

> **ロードマップのテスト数**: ロードマップ記載の完了条件（4075 + 2 = 4077）は計画値。実測は 4,081 ベース（→ 4,083）。ロードマップ修正は v92.0.0 時に実施。

> **MDX ドキュメント更新**: `site/content/docs/runes/sap-odata.mdx` の更新は v92.0.0 宣言時にまとめて実施する。

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
