# Spec: v90.2.0 — `AppCtx` に `sap: SapClient` フィールドを追加

## Background

`AppCtx` は Favnir の Capability Context パターンの中心型であり、パイプライン関数が受け取る
依存注入コンテナとして機能する。v13.5.0 で VM に組み込まれた実態（`db_url`・`aws_region`・`s3_bucket` 等）
を持つが、Favnir 言語レベルでの型定義ファイルは存在していなかった。

v90.1.0 で定義した `SapClient` interface を `AppCtx` に統合し、`ctx.sap.business_partners(filter)` という
呼び出しスタイルを宣言する。あわせて `runes/ctx/ctx.fav` に `AppCtx` の公式型定義を作成する。

## Goals

- `runes/ctx/ctx.fav`（新規作成）に `AppCtx` type を定義し、`sap: SapClient` フィールドを含める
- 既存フィールド（`s3`・`db`・`io`）も正式に型定義として記述する
- `driver.rs` に `mod v90200_tests` を追加（2 件）

## Syntax / API

```favnir
// runes/ctx/ctx.fav — AppCtx 型定義（v90.2.0）
// AppCtx は Favnir パイプライン関数の依存注入コンテナ。
// Ctx.build で本番インスタンスを、Ctx.mock でテスト用インスタンスを生成する。
// 実行時は vm.rs の AppCtx プリミティブが実体を提供する。

type AppCtx = {
    s3:  StorageCtx,     // S3 等のオブジェクトストレージアクセス（v13.5.0）
    db:  DbCtx,          // DB アクセス（PostgreSQL / MySQL 等）（v13.5.0）
    io:  IoCtx,          // 標準 IO / ファイル IO（v13.5.0）
    sap: SapClient       // SAP S/4HANA OData アクセス（v90.2.0 追加）
}
```

### 各フィールドの型

| フィールド | 型 | 追加バージョン | 説明 |
|---|---|---|---|
| `s3` | `StorageCtx` | v13.5.0 | S3 等のオブジェクトストレージアクセス |
| `db` | `DbCtx` | v13.5.0 | DB アクセス（`runes/ctx/db.fav` で定義） |
| `io` | `IoCtx` | v13.5.0 | 標準 IO / ファイル IO（`runes/ctx/io.fav` で定義） |
| `sap` | `SapClient` | v90.2.0 | SAP S/4HANA OData アクセス（`runes/sap-odata/types.fav` で定義） |

## Success Criteria

Rust テスト 2 件（合計: 4043 + 2 = **4045**）。
> 実装開始前に T0 で `cargo test` を実行し、ベースラインが 4,043 であることを実測すること。
> 実測値が異なる場合は本 spec のテスト数を実測値に合わせて更新してから実装を開始すること。

1. `app_ctx_has_sap_field`
   - `runes/ctx/ctx.fav` に文字列 `sap` が含まれること
2. `sap_field_type_is_sap_client`
   - `runes/ctx/ctx.fav` に文字列 `sap: SapClient` が含まれること

## Error Codes

本バージョンでは新規エラーコードなし。

## Files to Modify

| ファイル | 変更区分 | 変更内容 |
|---|---|---|
| `runes/ctx/ctx.fav` | **新規作成** | `AppCtx` type 定義（`StorageCtx`・`DbCtx`・`IoCtx`・`SapClient` の 4 フィールド） |
| `fav/src/driver.rs` | 追記 | `mod v90200_tests` を追加（`mod v90100_tests` の直後） |
