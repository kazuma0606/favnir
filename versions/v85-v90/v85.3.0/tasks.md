# Tasks: v85.3.0 — Docker Compose — SAP OData モックサーバー構築

Status: COMPLETE

> MILESTONE.md / README.md / `site/content/docs/` の更新は v86.0.0 宣言バージョンで実施する。
> 本バージョンはインフラファイル（Docker Compose / モックデータ / スクリプト）の新規作成と Rust テスト追加のみ。

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、3,935 tests, 0 failures を確認する
- [x] `fav/src/driver.rs` に `mod v85200_tests` が存在することを確認する（v85.2.0 完了済みの証拠）
- [x] `infra/e2e-demo/` ディレクトリが存在することを確認する

## T1: ディレクトリ作成

- [x] `infra/e2e-demo/sap-odata/mock/` ディレクトリを作成する

## T2: `docker-compose.yml` を作成

- [x] `infra/e2e-demo/sap-odata/docker-compose.yml` を作成する
  - `sap-mock` サービス（node:20-alpine, `@sap-ux/mockserver-main`, port 4004）
  - `favnir-runner` サービス（ghcr.io/favnir/fav:latest, SAP_* env vars）

## T3: モックデータ JSON を作成

- [x] `infra/e2e-demo/sap-odata/mock/BusinessPartnerCollection.json` を作成する（OData v4 形式, 10 件）
- [x] `infra/e2e-demo/sap-odata/mock/SalesOrderCollection.json` を作成する（OData v4 形式, 10 件）

## T4: README と起動スクリプトを作成

- [x] `infra/e2e-demo/sap-odata/README.md` を作成する（起動手順・前提条件）
- [x] `scripts/start-sap-mock.sh` を作成する（`docker compose up -d` 実行）
- [x] `chmod +x scripts/start-sap-mock.sh` で実行権限を付与する

## T5: `mod v85300_tests` を追加

- [x] `mod v85200_tests { ... }` の直後に `#[cfg(test)] mod v85300_tests { ... }` を追加する
- [x] `sap_mock_docker_compose_exists` テストを実装する
  - `std::path::Path::new("../infra/e2e-demo/sap-odata/docker-compose.yml").exists()` を確認
- [x] `sap_mock_data_business_partner_exists` テストを実装する
  - `std::path::Path::new("../infra/e2e-demo/sap-odata/mock/BusinessPartnerCollection.json").exists()` を確認

## T6: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、3,937 tests, 0 failures であることを確認する

## T7: CHANGELOG 更新

- [x] `CHANGELOG.md` の先頭に v85.3.0 エントリを追加する

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
