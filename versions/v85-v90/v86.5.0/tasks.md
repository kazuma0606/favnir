# Tasks: v86.5.0 — update_business_partner() PATCH

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、3,961 tests, 0 failures を確認する
- [x] `fav/src/driver.rs` に `mod v86400_tests` が存在することを確認する（v86.4.0 完了済みの証拠）
- [x] `runes/sap-odata/business_partner.fav` に `create_business_partner` 関数が存在することを確認する

## T1: `CHANGELOG.md` に v86.5.0 エントリを追加

- [x] `CHANGELOG.md` の先頭に v86.5.0 エントリを追加する（テストモジュール追加より先に実施）

## T2: `runes/sap-odata/business_partner.fav` を編集

- [x] `BusinessPartnerPatch` 型を追記する（3 フィールド、すべて Optional）
- [x] `update_business_partner()` 関数シグネチャ + スタブ本体を追記する

## T3: `runes/sap-odata/sap_odata.fav` を編集

- [x] `public type BusinessPartnerPatch` 再エクスポートを追加する
- [x] `update_business_partner()` ラッパーを追加する

## T4: `mod v86500_tests` を追加

- [x] `mod v86400_tests { ... }` の直後に `#[cfg(test)] mod v86500_tests { ... }` を追加する
- [x] `update_business_partner_function_exists` テストを実装する
- [x] `business_partner_patch_type_exists` テストを実装する（レコード型確認 + `Option<String>` assert を含む）

## T5: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、3,963 tests, 0 failures であることを確認する

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する

## 修正事項（code-reviewer 指摘対応）

- [MED] `business_partner_patch_type_exists` の `Option<String>` assert が偽陽性 → `language` フィールド固有チェック（`l.contains("language") && l.contains("Option<String>")`）に強化
- [LOW] `update_business_partner_function_exists` が戻り値型を未検証 → `content.contains("Result<Unit, String>")` assert を追加
