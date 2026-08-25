# Tasks: v86.3.0 — business_partner_by_id() + $expand

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、3,957 tests, 0 failures を確認する
- [x] `fav/src/driver.rs` に `mod v86200_tests` が存在することを確認する（v86.2.0 完了済みの証拠）
- [x] `runes/sap-odata/business_partner.fav` に `business_partners` 関数が存在することを確認する

## T1: `CHANGELOG.md` に v86.3.0 エントリを追加

- [x] `CHANGELOG.md` の先頭に v86.3.0 エントリを追加する（テストモジュール追加より先に実施）

## T2: `runes/sap-odata/business_partner.fav` を編集

- [x] `business_partner_by_id()` 関数シグネチャ + スタブ本体を追記する
- [x] `$expand=to_BusinessPartnerAddress` への言及をコメントに含める

## T3: `runes/sap-odata/sap_odata.fav` を編集

- [x] `business_partner_by_id()` の再エクスポートを追加する

## T4: `mod v86300_tests` を追加

- [x] `mod v86200_tests { ... }` の直後に `#[cfg(test)] mod v86300_tests { ... }` を追加する
- [x] `business_partner_by_id_function_exists` テストを実装する
- [x] `business_partner_expand_address_in_rune` テストを実装する

## T5: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、3,959 tests, 0 failures であることを確認する

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する

## 修正事項（code-reviewer 指摘対応）

- [MED] `sap_odata.fav` に `public type BusinessPartner` / `BusinessPartnerAddress` / `BusinessPartnerCategory` 再エクスポートを追加（v86.2.0 から潜在していた `BusinessPartner` 再エクスポート欠落を修正）
- [LOW] `v86300_tests` の `#[cfg(test)]` 欠落: 実際は 68551 行目に存在（reviewer の誤認、対応不要）
- [LOW] `BusinessPartner` / `BusinessPartnerAddress` の `public` 修飾子: v86.1.0 からの設計、`sap_odata.fav` の再エクスポートで外部公開済みのため対応なし
