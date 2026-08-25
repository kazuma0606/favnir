# Tasks: v86.1.0 — BusinessPartner / BusinessPartnerAddress 型定義

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、3,953 tests, 0 failures を確認する
- [x] `fav/src/driver.rs` に `mod v86000_tests` が存在することを確認する（v86.0.0 完了済みの証拠）
- [x] `runes/sap-odata/` ディレクトリが存在することを確認する

## T1: `CHANGELOG.md` に v86.1.0 エントリを追加

- [x] `CHANGELOG.md` の先頭に v86.1.0 エントリを追加する（テストモジュール追加より先に実施）

## T2: `runes/sap-odata/business_partner.fav` 新規作成

- [x] `BusinessPartnerCategory` 列挙型を定義する（Person | Organization | Group）
- [x] `BusinessPartner` レコード型を定義する（8 フィールド）
- [x] `BusinessPartnerAddress` レコード型を定義する（6 フィールド）

## T3: `mod v86100_tests` を追加

- [x] `mod v86000_tests { ... }` の直後に `#[cfg(test)] mod v86100_tests { ... }` を追加する
- [x] `business_partner_type_defined_in_rune` テストを実装する
- [x] `business_partner_address_type_defined_in_rune` テストを実装する

## T4: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、3,955 tests, 0 failures であることを確認する

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する

## 修正事項（code-reviewer 指摘対応）

- [MED] `business_partner.fav` の型定義順を `BusinessPartnerAddress` → `BusinessPartnerCategory` → `BusinessPartner` に変更（前方参照回避）
- [MED] Cargo.toml バージョン: スプリント版（x.1.0〜x.9.0）は宣言版（x.0.0）のみ更新する慣例のため変更なし
- [STYLE] `business_partner.fav` 先頭に `-- BusinessPartner 型定義（v86.1.0）` コメントを追加
- [LOW] `read_to_string` 重複: v85900_tests と同じパターン（外部 rune ファイルは `read_to_string`）のため変更なし
- [LOW] `BusinessPartnerCategory` テスト: ロードマップ指定の 2 件（3955 tests）のため追加なし
