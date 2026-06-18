# v14.4.0 Spec — AWS Rune 正式パッケージング (runes/aws/)

Date: 2026-06-12

---

## 目的

v14.2.0 で追加した `AwsCtx` 型と既存の `runes/aws/` を **統合** する。

`runes/aws/s3.fav`（v4.11.0）は `put_object(bucket, key, body)` のようにバケット名を
毎回明示する設計だった。v14.4.0 では `AwsCtx` からバケット名・リージョンを取り出せる
**ctx-aware ラッパー**を追加し、CrossCloud パイプラインで自然に使えるようにする。

あわせて AWS Secrets Manager への接続（`secrets_get`）を新規追加する。

---

## 現状（v14.3.0 時点）

| リソース | 状態 |
|---|---|
| `runes/aws/s3.fav` | `get_object(bucket, key)` / `put_object(bucket, key, body)` ─ バケット名を明示 |
| `runes/aws/sqs.fav` | `send_message(queue_url, body)` / `receive_messages(queue_url, max)` など |
| `runes/aws/s3_storage.fav` | `S3Storage(String)` — `StorageRead`/`StorageWrite` impl（v13.2.0） |
| `runes/aws/dynamodb.fav` | DynamoDB 操作（v4.11.0） |
| `runes/aws/aws.fav` | バレル: `use s3.*` / `use sqs.*` / `use dynamodb.*` |
| `Ctx.aws_get_field_raw` | **未実装**（azure_get_field_raw 相当） |
| `AWS.secrets_get_raw` | **未実装** |

---

## ユーザー体験（Before / After）

### Before（v14.3.0 まで）

```fav
// バケット名をハードコードまたは別途変数で管理
bind result <- aws.put_object("my-data-bucket", "output/result.json", body)
```

### After（v14.4.0）

```toml
# fav.toml
[aws]
region    = "${AWS_REGION}"
s3_bucket = "${S3_BUCKET}"
```

```fav
import rune "ctx"
import rune "aws"

public fn migrate(aws_ctx: AwsCtx, azure_ctx: AzureCtx) -> Result<Unit, String> !AWS !AzureDb {
    // バケット名は AwsCtx から自動取得
    bind _  <- aws.s3_put(aws_ctx, "proof/migrate.json", proof_json)

    // Secrets Manager から接続文字列を取得
    bind secret <- aws.secrets_get(aws_ctx, "prod/azure-postgres-url")
    AzurePostgres.execute_raw(secret, "INSERT INTO ...", "[]")
}
```

---

## スコープ

### In Scope

| 項目 | 内容 |
|---|---|
| `Ctx.aws_get_field_raw` VM primitive | `AwsCtx` JSON から `region`/`s3_bucket`/`db_url` を取り出す |
| `AWS.secrets_get_raw` VM primitive | Secrets Manager GetSecretValue API（ureq + SigV4） |
| checker.rs 登録 | `Ctx.aws_get_field_raw` → `String`、`AWS.secrets_get_raw` → `Result<String, String>` |
| `runes/aws/secrets.fav` | `secrets_get(ctx: AwsCtx, secret_name) -> Result<String, String> !AWS` |
| `runes/aws/s3.fav` 拡張 | ctx-aware wrappers: `s3_put(ctx, key, body)` / `s3_get(ctx, key)` / `s3_delete(ctx, key)` / `s3_list(ctx, prefix)` |
| `runes/aws/aws.fav` 更新 | `use secrets.*` 追加 |
| `rune.toml` 更新 | description / バージョン更新 |
| `v144000_tests` | 3 件のテスト |
| バージョン `14.4.0` | `fav/Cargo.toml` バンプ |

### Out of Scope

- `AwsStorageCtx` / `AwsQueueCtx` 独立型（`AwsCtx` で代替。v15.x で検討）
- `aws-sdk-*` crate 追加（既存 ureq + SigV4 で実装）
- SQS ctx-aware ラッパー（queue_url は ctx に含まれないため v15.x 以降）
- Azure Blob Storage（v14.5.0）
- CrossCloud E2E デモ（v15.0.0）

---

## 関数設計

### `runes/aws/secrets.fav`（新規）

```fav
// secrets.fav — AWS Secrets Manager (v14.4.0)
import rune "ctx"

public fn secrets_get(ctx: AwsCtx, secret_name: String) -> Result<String, String> !AWS {
    let region = Ctx.aws_get_field_raw(ctx, "region")
    AWS.secrets_get_raw(region, secret_name)
}
```

### `runes/aws/s3.fav` への追加（ctx-aware wrappers）

```fav
// 既存の put_object(bucket, key, body) はそのまま残す
// ctx-aware ラッパーを追加

import rune "ctx"

public fn s3_put(ctx: AwsCtx, key: String, body: String) -> Result<Unit, String> !AWS {
    let bucket = Ctx.aws_get_field_raw(ctx, "s3_bucket")
    AWS.s3_put_object_raw(bucket, key, body)
}

public fn s3_get(ctx: AwsCtx, key: String) -> Result<String, String> !AWS {
    let bucket = Ctx.aws_get_field_raw(ctx, "s3_bucket")
    AWS.s3_get_object_raw(bucket, key)
}

public fn s3_delete(ctx: AwsCtx, key: String) -> Result<Unit, String> !AWS {
    let bucket = Ctx.aws_get_field_raw(ctx, "s3_bucket")
    AWS.s3_delete_object_raw(bucket, key)
}

public fn s3_list(ctx: AwsCtx, prefix: String) -> Result<List<String>, String> !AWS {
    let bucket = Ctx.aws_get_field_raw(ctx, "s3_bucket")
    AWS.s3_list_objects_raw(bucket, prefix)
}
```

---

## VM Primitive 設計

### `Ctx.aws_get_field_raw(ctx_str, field) -> String`

`Ctx.azure_get_field_raw` と同パターン:
```rust
"Ctx.aws_get_field_raw" => {
    // args[0]: AwsCtx JSON string (e.g. "ok({...})" or raw JSON)
    // args[1]: field name ("region", "s3_bucket", "db_url")
    let json_str = ctx_str.trim_start_matches("ok(").trim_end_matches(')');
    // serde_json parse → get field → return as String
}
```

### `AWS.secrets_get_raw(region, secret_name) -> Result<String, String>`

```rust
"AWS.secrets_get_raw" => {
    // POST https://secretsmanager.{region}.amazonaws.com/
    // Action: GetSecretValue
    // SigV4 署名: service="secretsmanager", ureq + aws_sigv4 helper（既存パターン）
    // 返り値: ok(secret_string) or err(message)
}
```

---

## 完了条件

| 確認項目 | 目標 |
|---|---|
| `cargo test v144000` 全 3 件パス | ✅ |
| `cargo test` 全件パス（リグレッションなし） | ✅ |
| `CARGO_PKG_VERSION == "14.4.0"` | ✅ |
| `aws.secrets_get(ctx, "name")` が型チェックをパス | ✅ |
| `aws.s3_put(ctx, key, body)` が型チェックをパス | ✅ |
| `Ctx.aws_get_field_raw` が E0007 を出さない | ✅ |
| `AWS.secrets_get_raw` が E0007 を出さない | ✅ |
