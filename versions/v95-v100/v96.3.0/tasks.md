# Tasks: v96.3.0 — SAP → Parquet エクスポートパイプライン

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `versions/current.md` の最新安定版が `v96.2.0` であることを確認する
- [x] `fav/src/driver.rs` に `mod v96200_tests` が存在することを確認する（v96.2.0 完了済みの証拠）
- [x] `cargo test 2>&1 | grep "test result"` を実行し、現在のテスト数が 4,192 であることを確認する（着手前ベースライン）
- [x] `fav/Cargo.toml` の version が `96.0.0` であることを確認する（パッチ版のため Cargo.toml version は宣言版 96.0.0 のまま）

## T1: `infra/e2e-demo/sap-odata/pipeline_export.fav` を新規作成

- [x] `export_bp_to_parquet` pipeline を定義する（`!SapOData !Io` エフェクト）
  - `Fetch` stage: `ctx.sap.business_partners(BusinessPartnerFilter {...})` で JP 企業 1,000 件を取得
  - `Write` stage: `ctx.io.write_parquet("output/business_partners.parquet", bps)` で Parquet に書き出し
- [x] ファイル冒頭に `import rune "sap-odata"` を記述する

## T2: `fav/src/driver.rs` に `mod v96300_tests` を追加

- [x] `mod v96200_tests` の直後に `#[cfg(test)] mod v96300_tests { ... }` を追加する
- [x] `pipeline_export_fav_exists` テストを追加する（`pipeline_export.fav` に `export_bp_to_parquet` が含まれる）
- [x] `pipeline_export_uses_write_parquet` テストを追加する（`pipeline_export.fav` に `write_parquet` が含まれる）

## T3: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、4,194 tests, 0 failures であることを確認する

## T4: `CHANGELOG.md` に v96.3.0 エントリを追加

- [x] `CHANGELOG.md` の先頭に `[v96.3.0]` エントリを追加する

## T5: `versions/current.md` 更新

- [x] 最新安定版を `v96.3.0` に更新する（テスト数 4,194）

## T-last: CI 事前確認（T3 の `cargo test` 全 pass 確認後に実施すること）

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
