# Tasks: v94.7.0 — E2E デモ更新（$batch + SnapStart 完全デモ）

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、4,154 tests, 0 failures を確認する（着手前ベースライン）
- [x] `fav/src/driver.rs` に `mod v94600_tests` が存在することを確認する（v94.6.0 完了済みの証拠）
- [x] `runes/sap-odata/README.md` が存在することを確認する（v94.6.0 完了済みの証拠）

## T1: `infra/e2e-demo/sap-odata/pipeline_advanced.fav` を新規作成する

- [x] `infra/e2e-demo/sap-odata/pipeline_advanced.fav` を新規作成する
- [x] 先頭に `import rune "sap-odata"` と `import rune "s3"` を記載する
- [x] `advanced_sap_pipeline` 関数を実装する（シナリオ 5: $batch + QueryBuilder）
  - [x] `ctx.sap.business_partners(filter)` でデータ取得
  - [x] `ctx.s3.put_object(...)` で S3 バックアップ
  - [x] `List.map(bps, fn(bp) { BatchUpdate(...) })` でバッチ操作リスト作成
  - [x] `batch_request_builder("A_BusinessPartner", ops)` でリクエスト構築
  - [x] `ctx.sap.batch(req)` でバッチ送信（テスト要件: `"ctx.sap.batch"` が含まれること）
- [x] `bind x <- pure_fn()` に `Result.ok()` を使っていないことを確認する
- [x] `--` コメント形式を使用する（既存 pipeline.fav に合わせる）

## T2: `driver.rs` に `mod v94700_tests` を追加する

- [x] `mod v94600_tests { ... }` の直後に `#[cfg(test)] mod v94700_tests { ... }` を追加する（2 テスト）
- [x] `pipeline_advanced_fav_exists`: `"../infra/e2e-demo/sap-odata/pipeline_advanced.fav"` が存在することを確認する
- [x] `pipeline_advanced_uses_batch`: ファイルに `"ctx.sap.batch"` が含まれることを確認する

## T3: `CHANGELOG.md` に v94.7.0 エントリを追記する

- [x] `CHANGELOG.md` の先頭に v94.7.0 エントリを追加する

## T4: `cargo build` でコンパイル確認

- [x] `cargo build` を実行し、コンパイルエラーがないことを確認する

## T5: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、4,156 tests, 0 failures であることを確認する

## T6: tasks.md を COMPLETE に更新する

- [x] 本ファイルの Status を `COMPLETE` に変更する
- [x] T0〜T-last の全チェックボックスを `[x]` にする

## T-last: CI 事前確認（T5 の `cargo test` 全 pass 確認後に実施すること）

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
