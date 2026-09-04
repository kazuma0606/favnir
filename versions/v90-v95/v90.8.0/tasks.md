# Tasks: v90.8.0 — サイトドキュメント更新（ctx.sap パターンガイド）

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、4,056 tests, 0 failures を確認する
- [x] `fav/src/driver.rs` に `mod v90700_tests` が存在することを確認する（v90.7.0 完了済みの証拠）
- [x] `runes/ctx/ctx.fav` に `Ctx.mock` が含まれることを確認する（v90.7.0 完了済みの証拠）
- [x] `site/content/docs/runes/sap-odata.mdx` に `sap_config_from_env` が含まれることを確認する（本バージョンで削除するため）

## T1: 現状確認

- [x] `site/content/docs/runes/sap-odata.mdx` を読み込み、旧スタイル（`sap_config_from_env`）が残っていることを確認する

## T2: `sap-odata.mdx` の既存コード例を書き換え

- [x] BusinessPartner セクションのコード例を `ctx.sap.*` スタイルに書き換える
  - [x] `bind cfg <- sap_odata.sap_config_from_env()` 行を削除する
  - [x] `sap_odata.business_partners(cfg, filter)` → `ctx.sap.business_partners(filter)` に変更する
- [x] SalesOrder セクションのコード例を書き換える
  - [x] `bind cfg <- sap_odata.sap_config_from_env()` 行を削除する
  - [x] `sap_odata.sales_orders(cfg, filter)` → `ctx.sap.sales_orders(filter)` に変更する
- [x] Material セクションのコード例を書き換える
  - [x] `bind cfg <- sap_odata.sap_config_from_env()` 行を削除する
  - [x] `sap_odata.materials(cfg, filter)` → `ctx.sap.materials(filter)` に変更する
- [x] JournalEntry セクションのコード例を書き換える
  - [x] `bind cfg <- sap_odata.sap_config_from_env()` 行を削除する
  - [x] `sap_odata.journal_entries(cfg, filter)` → `ctx.sap.journal_entries(filter)` に変更する
- [x] `sap-odata.mdx` に `sap_config_from_env` が含まれないことを確認する

> **実装上の注記**: 初期実装で `## ctx.sap パターン` / `## Ctx.build 自動設定注入` の説明文中に `sap_config_from_env` という文字列が残存していた。code-reviewer [BUG] 指摘を受け、説明文を書き換えて完全除去した。

## T3: 新セクションを追加

- [x] `## ctx.sap パターン` セクションを追加する
  - [x] AppCtx 経由で SAP にアクセスする説明を記載する
  - [x] `fn sync_business_partners(ctx: AppCtx) -> Result<Int, String>` のコード例を含める
- [x] `## MockSapClient でユニットテスト` セクションを追加する
  - [x] `Ctx.mock(MockSapClient { ... })` の使用例を含める
  - [x] `MockSapClient.default()` の使用例を含める
- [x] `## Ctx.build 自動設定注入` セクションを追加する
  - [x] `Ctx.build()` の説明を記載する
  - [x] 本番コードでの `bind ctx <- Ctx.build()` の流れを含める

## T4: `mod v90800_tests` を `driver.rs` に追加

- [x] `mod v90700_tests { ... }` の直後に `#[cfg(test)] mod v90800_tests { ... }` を追加する
- [x] `docs_sap_odata_mentions_ctx_sap` テストを実装する（`sap-odata.mdx` に `ctx.sap` が含まれる）
- [x] `docs_sap_odata_mentions_mock_sap_client` テストを実装する（`sap-odata.mdx` に `MockSapClient` が含まれる）
- [x] `docs_sap_odata_no_sap_config_from_env` テストを追加する（code-reviewer [BUG] 対応: `sap_config_from_env` が含まれないことを検証）

## T5: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、4,059 tests, 0 failures であることを確認する

## T6: `CHANGELOG.md` に v90.8.0 エントリを追加

- [x] `CHANGELOG.md` の先頭（`## [v90.7.0]` の前）に v90.8.0 エントリを追加する
- [x] `v90.8.0`・`ctx.sap`・`MockSapClient`・`Ctx.build`・テスト数 `4,059` が含まれることを確認する
> 本バージョンは `changelog_has_v90_8_0` Rust テストを含まないため T5 後の追加で問題ない。

> 上記テスト全 pass 後、CI 事前確認（T-last）に進む。

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する

## Note

> 実装完了後は本ファイルの Status を `COMPLETE` に変更し、T0〜T-last の全チェックボックスを `[x]` にすること。
