# Tasks: v91.0.0 — SAP Ctx 統合 1.0 宣言 ★クリーンアップ

Status: IN PROGRESS

## T0: 着手前チェックリスト

- [ ] `cargo test` を実行し、4,061 tests, 0 failures を確認する
- [ ] `fav/src/driver.rs` に `mod v90900_tests` が存在することを確認する（v90.9.0 完了済みの証拠）
- [ ] `runes/sap-odata/sap_odata.fav` に `deprecated` が含まれることを確認する（本バージョンで削除するため）
- [ ] `infra/e2e-demo/sap-odata/pipeline.fav` に `ctx.sap.` が含まれることを確認する（v90.6.0 完了済みの証拠）

## T1: 現状確認

- [ ] `runes/sap-odata/sap_odata.fav` の deprecated 関数（4 件）を確認する
- [ ] 各個別 Rune ファイル（`business_partner.fav` / `sales_order.fav` / `material.fav` / `journal_entry.fav`）の `cfg: SapConfig` 受け取り関数 variants を確認する

## T2: `cargo clean`

- [ ] `cargo clean` を実行してビルドキャッシュを削除する
- [ ] `fav/tmp/hello.fav` が消えていないことを確認する（消えた場合は復元する）

## T3: `Cargo.toml` バージョン更新

- [ ] `fav/Cargo.toml` の `version = "90.0.0"` を `version = "91.0.0"` に変更する

## T4: `CHANGELOG.md` に v91.0.0 エントリを追加

- [ ] `## [v90.9.0]` の前に v91.0.0 エントリを追加する
- [ ] `SAP Ctx 統合 1.0` / `ctx.sap` / `4,065` が含まれることを確認する
> `changelog_has_v91_0_0` テストより先に追加すること（T10 のテスト追加前に実施）

## T5: `MILESTONE.md` を更新

- [ ] `v91.0 — SAP Ctx 統合 1.0` のエントリを **完了** に更新する
- [ ] `SAP Ctx 統合 1.0` という文字列が含まれることを確認する
> `milestone_has_sap_ctx_integration` テストより先に実施すること

## T6: `README.md` に `ctx.sap` 言及を追加

- [ ] README の SAP 関連セクションに `ctx.sap` パターンへの言及を追加する
- [ ] `ctx.sap` が README に含まれることを確認する
> `readme_mentions_ctx_sap` テストより先に実施すること

## T7: `versions/current.md` を v91.0.0 に更新

- [ ] 「最新安定版」を `v91.0.0 — SAP Ctx 統合 1.0 宣言` に更新する
- [ ] テスト数 `4,065` を記載する

## T8: `driver.rs` の `cargo_toml_version` テストを更新

- [ ] `cargo_toml_version_is_90_0_0` を `cargo_toml_version_is_91_0_0` に更新する（`replace_all: true`）

## T9: deprecated 関数を削除

- [ ] `runes/sap-odata/sap_odata.fav` の deprecated 4 関数を削除する
  - [ ] `business_partners_cfg` を削除する
  - [ ] `sales_orders_cfg` を削除する
  - [ ] `materials_cfg` を削除する
  - [ ] `journal_entries_cfg` を削除する
  - [ ] `sap_odata.fav` に `deprecated` が含まれないことを確認する
- [ ] `runes/sap-odata/business_partner.fav` の `cfg: SapConfig` 受け取り関数を削除する（実態確認後）
- [ ] `runes/sap-odata/sales_order.fav` の `cfg: SapConfig` 受け取り関数を削除する（実態確認後）
- [ ] `runes/sap-odata/material.fav` の `cfg: SapConfig` 受け取り関数を削除する（実態確認後）
- [ ] `runes/sap-odata/journal_entry.fav` の `cfg: SapConfig` 受け取り関数を削除する（実態確認後）

## T10: `mod v91000_tests` を `driver.rs` に追加

- [ ] `mod v90900_tests { ... }` の直後に `#[cfg(test)] mod v91000_tests { ... }` を追加する
- [ ] `cargo_toml_version_is_91_0_0` テストを実装する
- [ ] `changelog_has_v91_0_0` テストを実装する
- [ ] `milestone_has_sap_ctx_integration` テストを実装する
- [ ] `readme_mentions_ctx_sap` テストを実装する

## T11: `cargo test` で全 pass 確認

- [ ] `cargo test 2>&1 | grep "test result"` を実行し、4,065 tests, 0 failures であることを確認する

> 上記テスト全 pass 後、CI 事前確認（T-last）に進む。

## T-last: CI 事前確認

- [ ] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [ ] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [ ] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する

## Note

> 実装完了後は本ファイルの Status を `COMPLETE` に変更し、T0〜T-last の全チェックボックスを `[x]` にすること。
