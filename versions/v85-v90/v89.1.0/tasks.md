# Tasks: v89.1.0 — `JournalEntry` / `JournalEntryItem` 型定義 + `journal_entries()`

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、4,019 tests, 0 failures を確認する
- [x] `fav/src/driver.rs` に `mod v88900_tests` が存在することを確認する（v88.9.0 完了済みの証拠）
- [x] `fav/src/driver.rs` に `mod v89000_tests` が存在することを確認する（v89.0.0 完了済みの証拠）
- [x] `fav/Cargo.toml` の version が `89.0.0` であることを確認する（v90.0.0 宣言バージョンまでバンプしない設計のため、89.0.0 が正しい）
- [x] `runes/sap-odata/sap_odata.fav` に `use sap_odata.stock` が存在することを確認する（re-export 追加先の確認）

## T1: `runes/sap-odata/journal_entry.fav` を新規作成

- [x] `DebitCredit` 型（`Debit | Credit`）を定義する
- [x] `JournalEntryItem` 型を定義する（6 フィールド）
- [x] `JournalEntry` 型を定義する（7 フィールド、`items` は `Option<List<JournalEntryItem>>`）
- [x] `JournalFilter` 型を定義する（5 フィールド、全 `Option`）
- [x] `journal_entries(cfg: SapConfig, filter: JournalFilter) -> Result<List<JournalEntry>, String>` をスタブ実装する（`Result.err("not implemented")`）

## T2: `runes/sap-odata/sap_odata.fav` に re-export を追加

- [x] `use sap_odata.journal_entry` を追加する
- [x] `public type DebitCredit` / `JournalEntryItem` / `JournalEntry` / `JournalFilter` の re-export を追加する
- [x] `public fn journal_entries(...)` ラッパー関数を追加する

## T3: `driver.rs` に `mod v89100_tests` を追加

- [x] `mod v89000_tests { ... }` の直後に `#[cfg(test)] mod v89100_tests { ... }` を追加する
- [x] `journal_entry_type_defined_in_rune` テストを実装する（`journal_entry.fav` に `"JournalEntry"` を確認）
- [x] `journal_entries_function_exists` テストを実装する（`journal_entry.fav` に `"public fn journal_entries("` を確認）

## T4: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、4,021 tests, 0 failures であることを確認する

## Note

CHANGELOG / MILESTONE / site MDX 更新は v90.0.0 宣言バージョンでまとめて実施する（本バージョンは対象外）。

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
