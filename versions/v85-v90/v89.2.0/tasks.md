# Tasks: v89.2.0 — `OutstandingPayable` 型 + `match_unposted_orders()`

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、4,021 tests, 0 failures を確認する
- [x] `fav/src/driver.rs` に `mod v89100_tests` が存在することを確認する（v89.1.0 完了済みの証拠）
- [x] `fav/Cargo.toml` の version が `89.0.0` であることを確認する（v90.0.0 宣言バージョンまでバンプしない設計のため、89.0.0 が正しい）
- [x] `runes/sap-odata/journal_entry.fav` に `journal_entries` 関数が存在することを確認する（追記先の確認）

## T1: `runes/sap-odata/journal_entry.fav` に追記

- [x] ファイル先頭の use ブロック（`use sap_odata.types` の直後）に `use sap_odata.purchase_order` を追加する（`PurchaseOrder` 参照のため）
- [x] `OutstandingPayable` 型を定義する（6 フィールド）
- [x] `match_unposted_orders(pos: List<PurchaseOrder>, journals: List<JournalEntry>) -> Result<List<OutstandingPayable>, String>` をスタブ実装する（`Result.err("not implemented")`）

## T2: `runes/sap-odata/sap_odata.fav` に re-export を追加

- [x] `public type OutstandingPayable = journal_entry.OutstandingPayable` を追加する
- [x] `public fn match_unposted_orders(...)` ラッパー関数を追加する

## T3: `driver.rs` に `mod v89200_tests` を追加

- [x] `mod v89100_tests { ... }` の直後に `#[cfg(test)] mod v89200_tests { ... }` を追加する
- [x] `outstanding_payable_type_exists` テストを実装する（`journal_entry.fav` に `"OutstandingPayable"` を確認）
- [x] `match_unposted_orders_function_exists` テストを実装する（`journal_entry.fav` に `"public fn match_unposted_orders("` を確認）

## T4: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、4,023 tests, 0 failures であることを確認する

## Note

CHANGELOG / MILESTONE / site MDX 更新は v90.0.0 宣言バージョンでまとめて実施する（本バージョンは対象外）。

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する

## spec-reviewer 指摘対応

- [MED] ロードマップの `JournalEntryFilter` → `OutstandingPayable 型 + match_unposted_orders()` に修正（バージョン一覧 + セクション見出し + 実装ファイル欄）
- [MED] ロードマップの実装ファイル欄に `sap_odata.fav` を追記
- [MED] `use sap_odata.purchase_order` の追記位置をファイル先頭 use ブロックと明記（spec / plan / tasks）
- [LOW] spec の Files 表に `PurchaseOrder` は re-export 済みの旨を注記、plan に Step 5（CI 確認）を追加
