# Tasks: v90.6.0 — `pipeline.fav` を `ctx.sap.*` で書き換え

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、4,052 tests, 0 failures を確認する
- [x] `fav/src/driver.rs` に `mod v90500_tests` が存在することを確認する（v90.5.0 完了済みの証拠）
- [x] `runes/sap-odata/sap_odata.fav` に `ctx.sap.business_partners` が含まれることを確認する（v90.5.0 完了済みの証拠）
- [x] `infra/e2e-demo/sap-odata/pipeline.fav` に `sap_config_from_env` が含まれることを確認する（本バージョンで削除するため）

## T1: 現状確認

- [x] `pipeline.fav` の全 4 シナリオ構造を確認する
- [x] `bind cfg <- sap_odata.sap_config_from_env()` が 4 件あることを確認する

## T2: `pipeline.fav` を書き換え

- [x] シナリオ 1（`sync_business_partners`）を書き換える
  - [x] `bind cfg <- sap_odata.sap_config_from_env()` を削除する
  - [x] `sap_odata.business_partners(cfg, filter)` → `ctx.sap.business_partners(filter)` に書き換える
- [x] シナリオ 2（`daily_sales_report`）を書き換える
  - [x] `bind cfg <- sap_odata.sap_config_from_env()` を削除する
  - [x] `sap_odata.sales_orders(cfg, filter)` → `ctx.sap.sales_orders(filter)` に書き換える
- [x] シナリオ 3（`check_stock_vs_orders`）を書き換える
  - [x] `bind cfg <- sap_odata.sap_config_from_env()` を削除する
  - [x] `sap_odata.sales_orders(cfg, filter)` → `ctx.sap.sales_orders(filter)` に書き換える
  - [x] `sap_odata.materials(cfg, filter)` → `ctx.sap.materials(filter)` に書き換える
- [x] シナリオ 4（`outstanding_payables`）を書き換える
  - [x] `bind cfg <- sap_odata.sap_config_from_env()` を削除する
  - [x] `sap_odata.purchase_orders(cfg, filter)` の呼び出しを除去する（`SapClient` interface 外、v91.x.x で対応予定）
  - [x] `sap_odata.journal_entries(cfg, filter)` → `ctx.sap.journal_entries(filter)` に書き換える
  - [x] 戻り値型を `List<JournalEntry>` に変更し `match_unposted_orders` の呼び出しも除去する
- [x] `pipeline.fav` に `sap_config_from_env` が含まれないことを確認する

> **実装上の注記**: pipeline.fav のコメントに `sap_config_from_env` という文字列を含めると `pipeline_fav_no_explicit_cfg` テストが失敗するため、コメント文言を「cfg 明示取得を廃止」に変更した。また `v89900_tests::sap_all_four_scenarios_in_pipeline` が旧スタイル（`sap_odata.*`）を期待していたため、新スタイル（`ctx.sap.*`）に合わせて更新した。

## T3: `mod v90600_tests` を `driver.rs` に追加

- [x] `mod v90500_tests { ... }` の直後に `#[cfg(test)] mod v90600_tests { ... }` を追加する
- [x] `pipeline_fav_uses_ctx_sap` テストを実装する（`pipeline.fav` に `ctx.sap.` が含まれることを確認）
- [x] `pipeline_fav_no_explicit_cfg` テストを実装する（`pipeline.fav` に `sap_config_from_env` が含まれないことを確認）

## T4: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、4,054 tests, 0 failures であることを確認する

## T5: `CHANGELOG.md` に v90.6.0 エントリを追加

- [x] `CHANGELOG.md` の先頭（`## [v90.5.0]` の前）に v90.6.0 エントリを追加する
- [x] `v90.6.0`・`ctx.sap.*`・`sap_config_from_env 削除`・テスト数 `4,054` が含まれることを確認する
> 本バージョンは `changelog_has_v90_6_0` Rust テストを含まないため T4 後の追加で問題ない。

> 上記テスト全 pass 後、CI 事前確認（T-last）に進む。

## Note

> 実装完了後は本ファイルの Status を `COMPLETE` に変更し、T0〜T-last の全チェックボックスを `[x]` にすること。

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する

## code-reviewer 指摘対応（完了）

- [x] **[BUG]** `outstanding_payables` 戻り値型変更（`List<OutstandingPayable>` → `List<JournalEntry>`）の呼び出し元確認
  - `infra/e2e-demo/sap-odata/` に `main.fav` 等の呼び出し元ファイルは存在しない（`pipeline.fav` のみ）
  - `purchase_orders` は SapClient interface 外のため除去は仕様どおり、実害なし
- [x] **[STYLE]** `pipeline_fav_uses_ctx_sap` が `"ctx.sap."` のみチェック → `v89900_tests::sap_all_four_scenarios_in_pipeline` が全 4 メソッド個別カバー済み、non-blocking
