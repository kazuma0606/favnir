# Tasks: v86.2.0 — BusinessPartnerFilter + business_partners() クエリ

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、3,955 tests, 0 failures を確認する
- [x] `fav/src/driver.rs` に `mod v86100_tests` が存在することを確認する（v86.1.0 完了済みの証拠）
- [x] `runes/sap-odata/business_partner.fav` が存在することを確認する

## T1: `CHANGELOG.md` に v86.2.0 エントリを追加

- [x] `CHANGELOG.md` の先頭に v86.2.0 エントリを追加する（テストモジュール追加より先に実施）

## T2: `runes/sap-odata/business_partner.fav` を編集

- [x] `BusinessPartnerFilter` 型を追記する（4 フィールド、すべて Optional）
- [x] `business_partners()` 関数シグネチャ + スタブ本体を追記する

## T3: `mod v86200_tests` を追加

- [x] `mod v86100_tests { ... }` の直後に `#[cfg(test)] mod v86200_tests { ... }` を追加する
- [x] `business_partners_function_exists` テストを実装する
- [x] `business_partner_filter_type_exists` テストを実装する

## T4: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、3,957 tests, 0 failures であることを確認する

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する

## 修正事項（code-reviewer 指摘対応）

- [HIGH] `business_partner.fav` に `use sap_odata.types` を追加（`SapConfig` 参照を明示化）
- [MED] `BusinessPartnerFilter` に `public` 修飾子を追加
- [MED] `sap_odata.fav` に `use sap_odata.business_partner` + `BusinessPartnerFilter` 再エクスポート + `business_partners()` ラッパーを追加
- [LOW] `business_partners_function_exists` テストの assertion を `"fn business_partners"` に強化
