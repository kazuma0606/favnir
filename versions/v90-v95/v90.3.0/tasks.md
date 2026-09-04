# Tasks: v90.3.0 — `MockSapClient` 実装

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、4,045 tests, 0 failures を確認する
- [x] `fav/src/driver.rs` に `mod v90200_tests` が存在することを確認する（v90.2.0 完了済みの証拠）
- [x] `runes/sap-odata/types.fav` に `interface SapClient` が含まれることを確認する（v90.1.0 完了済みの証拠）
- [x] `runes/sap-odata/mock.fav` が存在しないことを確認する（本バージョンで新規作成するファイル）

## T1: 前提・パターン確認

- [x] `runes/sap-odata/types.fav` の `interface SapClient` メソッド一覧を確認する（5 メソッド）
- [x] `runes/ctx/mock_db.fav` の `impl X for Y` 構文を確認する（参照パターン）

## T2: `runes/sap-odata/mock.fav` を新規作成

- [x] `runes/sap-odata/mock.fav` を新規作成する
- [x] `//` コメントスタイルでファイルヘッダーを記述する
- [x] `type MockSapClient = { ... }` レコード型を定義する（4 フィールド）
- [x] `impl SapClient for MockSapClient { ... }` ブロックを実装する（5 メソッド）
  - [x] `business_partners`: `self.business_partners_result` を返す
  - [x] `business_partner_by_id`: `Result.err("not implemented")` を返す
  - [x] `sales_orders`: `self.sales_orders_result` を返す
  - [x] `materials`: `self.materials_result` を返す
  - [x] `journal_entries`: `self.journal_entries_result` を返す

## T3: `mod v90300_tests` を `driver.rs` に追加

- [x] `mod v90200_tests { ... }` の直後に `#[cfg(test)] mod v90300_tests { ... }` を追加する
- [x] `mock_sap_client_file_exists` テストを実装する（`mock.fav` が存在することを確認）
- [x] `mock_sap_client_implements_sap_client` テストを実装する（`impl SapClient for MockSapClient` が含まれることを確認）

## T4: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、4,048 tests, 0 failures であることを確認する（code-reviewer 指摘対応でテスト 3 件に増加）

## T5: `CHANGELOG.md` に v90.3.0 エントリを追加

- [x] `CHANGELOG.md` の先頭（`## [v90.2.0]` の前）に v90.3.0 エントリを追加する
- [x] `v90.3.0`・`MockSapClient`・`runes/sap-odata/mock.fav`・テスト数 `4,047` が含まれることを確認する
> 本バージョンは `changelog_has_v90_3_0` Rust テストを含まないため T4 後の追加で問題ない。

> 上記テスト全 pass 後、CI 事前確認（T-last）に進む。

## Note

> 実装完了後は本ファイルの Status を `COMPLETE` に変更し、T0〜T-last の全チェックボックスを `[x]` にすること。

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
