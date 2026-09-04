# Spec: v94.9.0 — 安定化・コードフリーズ

## Background

v94.1〜v94.8 で実装した SAP Advanced Era（Sprint 5）の全機能を通しで点検する最終安定化スプリント。
新機能の追加は行わず、バグ修正と成果物の存在確認テストの追加のみを実施する。

本バージョン完了後、v95.0.0（SAP Advanced 1.0 宣言）に移行する。

## Goals

1. SAP Advanced Era 全 4 スプリントの成果物が揃っていることを Rust テストで確認する
   - `runes/sap-odata/batch.fav`（$batch 型定義）
   - `runes/sap-odata/query_builder.fav`（QueryBuilder<T>）
   - `fav/src/sap_metadata.rs`（Metadata Infer）
   - `infra/lambda/sap-sync/main.tf`（SnapStart Lambda Terraform）
2. SAP Advanced Era ドキュメント完成の確認テストを追加する
   - `site/content/docs/guides/sap-integration.mdx` が存在する
3. バグ修正のみ受け入れる（新機能追加なし）

## Success Criteria

1. `driver.rs` に `mod v94900_tests` が追加され、2 件のテストが全て pass する:
   - `sap_advanced_smoke_all_features`: batch.fav / query_builder.fav / sap_metadata.rs / main.tf が全て存在する
   - `sap_advanced_era_doc_complete`: `site/content/docs/guides/sap-integration.mdx` が存在する
2. `cargo test` で 4,160 tests（+2）、0 failures
3. `cargo clippy --locked -- -D warnings` pass
4. `fav fmt --check` pass（compiler.fav / checker.fav）

## Files to Modify

| ファイル | 操作 | 内容 |
|---|---|---|
| `fav/src/driver.rs` | **更新** | `mod v94900_tests`（テスト 2 件）追加 |
| `CHANGELOG.md` | **更新** | v94.9.0 エントリ追加 |
