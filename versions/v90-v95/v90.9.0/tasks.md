# Tasks: v90.9.0 — 安定化・コードフリーズ

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、4,059 tests, 0 failures を確認する
- [x] `fav/src/driver.rs` に `mod v90800_tests` が存在することを確認する（v90.8.0 完了済みの証拠）
- [x] `site/content/docs/runes/sap-odata.mdx` に `ctx.sap` が含まれることを確認する（v90.8.0 完了済みの証拠）
- [x] `infra/e2e-demo/sap-odata/pipeline.fav` に `ctx.sap.` が含まれることを確認する（v90.6.0 完了済みの証拠）

## T1: 現状確認

- [x] `pipeline.fav` に 4 シナリオ関数（`sync_business_partners` / `daily_sales_report` / `check_stock_vs_orders` / `outstanding_payables`）と `ctx.sap.` が含まれることを確認する
- [x] `runes/sap-odata/mock.fav` が存在することを確認する

## T2: `mod v90900_tests` を `driver.rs` に追加

- [x] `mod v90800_tests { ... }` の直後に `#[cfg(test)] mod v90900_tests { ... }` を追加する
- [x] `sap_ctx_integration_smoke_all_scenarios` テストを実装する
  - [x] `sync_business_partners` / `daily_sales_report` / `check_stock_vs_orders` / `outstanding_payables` / `ctx.sap.` の 5 文字列すべてをループ確認する
- [x] `sap_ctx_mock_client_in_rune_dir` テストを実装する
  - [x] `runes/sap-odata/mock.fav` が `Path::new(...).exists()` で存在することを確認する

## T3: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、4,061 tests, 0 failures であることを確認する

## T4: `CHANGELOG.md` に v90.9.0 エントリを追加

- [x] `CHANGELOG.md` の先頭（`## [v90.8.0]` の前）に v90.9.0 エントリを追加する
- [x] `v90.9.0`・`安定化`・`コードフリーズ`・テスト数 `4,061` が含まれることを確認する
> 本バージョンは `changelog_has_v90_9_0` Rust テストを含まないため T3 後の追加で問題ない。

> 上記テスト全 pass 後、CI 事前確認（T-last）に進む。

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する

## code-reviewer 指摘対応（完了）

- [x] **[STYLE]** `use std::path::Path;` → `std::path::Path::new(...)` フルパスに変更（他モジュールと統一）
- [x] **[BUG]** `"ctx.sap."` をループから除外（`v90600_tests::pipeline_fav_uses_ctx_sap` で既にカバー済み）。4 シナリオ関数名のみに絞る
- [x] **[BUG]** `sap_ctx_mock_client_in_rune_dir` に `MockSapClient.default` の内容チェックを追加（`v90300_tests::mock_sap_client_file_exists` との差別化 — v90.7.0 実装確認を統合）

## Note

> 実装完了後は本ファイルの Status を `COMPLETE` に変更し、T0〜T-last の全チェックボックスを `[x]` にすること。
