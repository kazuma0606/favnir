# Tasks: v89.3.0 — シナリオ 4: 購買→支払サイクル照合

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、4,023 tests, 0 failures を確認する
- [x] `fav/src/driver.rs` に `mod v89200_tests` が存在することを確認する（v89.2.0 完了済みの証拠）
- [x] `fav/Cargo.toml` の version が `89.0.0` であることを確認する（v90.0.0 宣言バージョンまでバンプしない設計のため、89.0.0 が正しい）
- [x] `infra/e2e-demo/sap-odata/pipeline.fav` にシナリオ 1〜3（`sync_business_partners` / `daily_sales_report` / `check_stock_vs_orders`）が存在することを確認する

## T1: `infra/e2e-demo/sap-odata/pipeline.fav` にシナリオ 4 を追記

- [x] `check_stock_vs_orders` 関数の直後に `-- シナリオ 4: 購買→支払サイクル照合（v89.3.0）` コメントを追加する
- [x] `outstanding_payables(ctx: AppCtx) -> Result<List<OutstandingPayable>, String>` 関数を実装する
  - `sap_odata.purchase_orders` で `PartiallyDelivered` の発注を取得
  - `sap_odata.journal_entries` で 2026 年度の会計伝票を取得
  - `sap_odata.match_unposted_orders` で突き合わせ
  - `Json.encode` して S3 に保存（`payables/outstanding.json`）

## T2: `driver.rs` に `mod v89300_tests` を追加

- [x] `mod v89200_tests { ... }` の直後に `#[cfg(test)] mod v89300_tests { ... }` を追加する
- [x] `sap_e2e_pipeline_contains_outstanding_payables` テストを実装する（`pipeline.fav` に `"outstanding_payables"` を確認）
- [x] `sap_e2e_pipeline_has_all_four_scenarios` テストを実装する（4 関数名すべてを確認）

## T3: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、4,025 tests, 0 failures であることを確認する

> 上記テスト全 pass 後、CI 事前確認（T-last）に進む。

## Note

> 実装完了後は本ファイルの Status を `COMPLETE` に変更し、T0〜T-last の全チェックボックスを `[x]` にすること。



CHANGELOG / MILESTONE / site MDX 更新は v90.0.0 宣言バージョンでまとめて実施する（本バージョンは対象外）。

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
