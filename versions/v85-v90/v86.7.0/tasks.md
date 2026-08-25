# Tasks: v86.7.0 — SAP OData テスト拡充（BusinessPartner CRUD テスト）

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、3,965 tests, 0 failures を確認する
- [x] `fav/src/driver.rs` に `mod v86600_tests` が存在することを確認する（v86.6.0 完了済みの証拠）
- [x] `runes/sap-odata/sap_odata.test.fav` が存在することを確認する（v85.4.0 で作成済み）

## T1: `CHANGELOG.md` に v86.7.0 エントリを追加

- [x] `CHANGELOG.md` の先頭に v86.7.0 エントリを追加する（テストモジュール追加より先に実施）

## T2: `runes/sap-odata/sap_odata.test.fav` に CRUD テスト追加

- [x] 既存の `test_sap_config_fields_exist` の後に `test_business_partner_create` を追加する
- [x] `test_business_partner_read` を追加する
- [x] `test_business_partner_update` を追加する
- [x] `test_business_partner_list` を追加する

## T3: `scripts/test-with-mock.sh` を新規作成

- [x] `scripts/test-with-mock.sh` をスタブスクリプトとして作成する
- [x] 実行権限を付与する（`chmod +x`）
- [x] `bash scripts/test-with-mock.sh` を実行して "PASS" が出力されることを確認する

## T4: `mod v86700_tests` を追加

- [x] `mod v86600_tests { ... }` の直後に `#[cfg(test)] mod v86700_tests { ... }` を追加する
- [x] `sap_odata_test_fav_exists` テストを実装する
- [x] `sap_odata_test_contains_business_partner_tests` テストを実装する

## T5: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、3,967 tests, 0 failures であることを確認する

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する

## 修正事項（code-reviewer 指摘対応）

- [MED] `sap_odata.test.fav` を `fn test_xxx() -> Bool` 構文から `test "..." { }` 構文に変換（他の rune テストと統一）+ `import "sap-odata"` を追加
- [LOW] `scripts/test-with-mock.sh` の git 実行権限を `100644` → `100755` に修正（`git update-index --chmod=+x`）
- [LOW] Rust テストのアサーション文字列を `test "business_partner_create"` 構文に合わせて更新
