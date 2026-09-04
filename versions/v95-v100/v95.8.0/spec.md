# Spec: v95.8.0 — `fav sap-mock`

## Background

SAP OData 統合のオフラインテストを可能にするため、`fav sap-mock` コマンドを追加する。
本コマンドは `mock.fav`（v90.3.0〜）に定義された `MockSapClient` を参照しながら、
利用可能な OData エンドポイント一覧を表示するモックサーバーの起動をシミュレートする。

v95.8.0 では stdout に起動メッセージとエンドポイント一覧を出力する stub 実装とする。
実際の HTTP サーバー起動（hyper/axum ベース）は後続バージョンで実施する。

## Goals

1. `driver.rs` に `SapMockServer` 構造体を定義する（port, fixtures フィールド）
2. `driver.rs` に `cmd_sap_mock` 関数を定義する（stdout にモック起動情報を出力）
3. `main.rs` に `Some("sap-mock")` アームを追加して `cmd_sap_mock` を呼ぶ
4. `driver.rs` に `mod v95800_tests`（2 件）を追加する

## CLI 使用例

```
$ fav sap-mock --port 8080 --fixtures runes/sap-odata/mock.fav
SAP Mock Server listening on http://localhost:8080
  GET  /sap/opu/odata/sap/API_BUSINESS_PARTNER/A_BusinessPartner
  POST /sap/opu/odata/sap/API_BUSINESS_PARTNER/A_BusinessPartner
  POST /$batch
```

## Files to Modify

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `fav/src/driver.rs` | 修正 | `SapMockServer` 構造体 + `cmd_sap_mock` 関数 + `mod v95800_tests` |
| `fav/src/main.rs` | 修正 | `Some("sap-mock")` アーム追加 |
| `runes/sap-odata/mock.fav` | 変更なし（参照のみ） | `MockSapClient` の定義を参照するが変更不要 |

## Success Criteria

- `driver.rs` に `SapMockServer` が含まれる
- `driver.rs` に `cmd_sap_mock` が含まれる
- `main.rs` に `sap-mock` が含まれる
- `cargo test` で 4,182 tests, 0 failures
- 本バージョンは stdout 出力の stub 実装であり、実際の HTTP サーバー起動は行わない

## Out of Scope（次バージョン以降）

- 実際の HTTP サーバー起動（hyper/axum による OData エンドポイント実装）
- `mock.fav` の MockSapClient を読み込んでエンドポイントを動的生成
- `--watch` フラグでの自動リロード
