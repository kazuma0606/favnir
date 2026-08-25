# Spec: v86.8.0 — Rune Registry デプロイ（sap-odata）

## Background

v86.1.0〜v86.7.0 で SAP OData Rune の型・関数・テスト・E2E パイプラインが揃った。
`runes/sap-odata/rune.toml` のバージョンは `86.8.0` 以前の値のままであり、
Rune Registry（Lambda + DynamoDB + S3）にはまだ `sap-odata` が登録されていない。

v86.8.0 では `rune.toml` のバージョンを `86.8.0` に最新化し、
既存の `deploy-registry` スキルを用いて sap-odata Rune を Registry に登録する。
DynamoDB への登録・S3 へのアップロードを手動で確認し、tasks.md のチェックとして記録する。
`import rune "sap-odata"` の実行動作確認は v86.9.0 安定化スプリントで実施する。

## Goals

1. `runes/sap-odata/rune.toml` のバージョンを `86.8.0` に更新する
2. `deploy-registry` スキルで sap-odata Rune を Lambda Rune Registry にデプロイする
3. `rune.toml` の整合性（version/entry）を Rust テストで確認する

## Scope

### `rune.toml` 更新内容

```toml
[rune]
name        = "sap-odata"
version     = "86.8.0"
entry       = "sap_odata.fav"
description = "SAP S/4HANA OData v4 クライアント — ctx パターンで型安全な SAP データアクセスを提供"
```

### Rust テスト（`mod v86800_tests`）

- `sap_odata_rune_version_matches_cargo`: `rune.toml` の `version` フィールドが `86.` で始まることを確認
- `sap_odata_rune_entry_file_is_sap_odata_fav`: `rune.toml` の `entry` フィールドが `sap_odata.fav` であることを確認

### デプロイ手順

`deploy-registry` スキルを使用して `sap-odata` Rune をデプロイする（実施はユーザーが手動で行う）。
デプロイ後、以下の 2 点を手動確認する:
- DynamoDB (`favnir-rune-registry`) に `name = "sap-odata"` のエントリが存在すること
- S3 (`favnir-rune-packages`) に `sap-odata/` 配下の `.fav` ファイルが存在すること

## Files to Modify

| ファイル | 操作 |
|---|---|
| `CHANGELOG.md` | v86.8.0 エントリ追加（先頭） |
| `runes/sap-odata/rune.toml` | version を `86.8.0` に更新 |
| `fav/src/driver.rs` | `mod v86800_tests` 追加 |

## Success Criteria

- `runes/sap-odata/rune.toml` の `version` が `86.8.0` である
- `runes/sap-odata/rune.toml` の `entry` が `sap_odata.fav` である
- `deploy-registry` スキルが終了コード 0 で完了する
- DynamoDB および S3 への登録が手動確認で確認できる
- `cargo test 2>&1 | grep "test result"` が 3969 tests, 0 failures を出力する
