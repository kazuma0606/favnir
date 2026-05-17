# Favnir v4.11.0 仕様書 — AWS SDK Rune & fav deploy

作成日: 2026-05-17

---

## 概要

Favnir から AWS サービスを直接操作できる **AWS Rune** と、Favnir プログラムを AWS Lambda へデプロイする **`fav deploy`** コマンドを追加する。LocalStack（ローカル開発）と本番 AWS の両方に対応する。

**主な追加機能:**
- `!AWS` エフェクト — AWS API 呼び出しを伴う関数を型レベルで区別
- `runes/aws/` — S3 / SQS / DynamoDB の各 Rune モジュール
- VM プリミティブ — SigV4 署名 + `ureq` による REST API 呼び出し
- `fav.toml [aws]` セクション — region / endpoint_url / credential 設定
- `fav deploy [--env <env>] [--dry-run]` — Lambda へのデプロイ

---

## `!AWS` エフェクト

### 型システムでの扱い

`!AWS` は `Effect::Unknown("AWS")` として表現する（`!Auth`, `!Env` と同じパターン）。

```favnir
// aws Rune の関数はすべて !AWS を要求
public fn get_object(bucket: String, key: String) -> Result<String, String> !AWS {
    AWS.get_object_raw(bucket, key)
}
```

呼び出し側は `!AWS` を宣言する必要がある：

```favnir
import rune "aws"

public fn main() -> Unit !Io !AWS {
    match aws.get_object("my-bucket", "data/input.json") {
        Ok(body) => IO.println(body)
        Err(e)   => IO.println($"error: {e}")
    }
}
```

### checker.rs への追加

- `BUILTIN_EFFECTS` に `"AWS"` を追加
- `require_aws_effect(&mut self, span: &Span)` — E0313 を発行
- `check_test_def` の `current_effects` に `Effect::Unknown("AWS")` を追加

---

## 設定: `fav.toml [aws]`

```toml
[aws]
region       = "ap-northeast-1"     # AWS リージョン
endpoint_url = ""                   # LocalStack: "http://localhost:4566"
profile      = ""                   # AWS named profile（省略時: env vars）
```

### 優先順位（実行時）

1. `fav.toml [aws]` の値
2. `AWS_REGION` 環境変数
3. `AWS_ENDPOINT_URL` 環境変数（LocalStack 用）
4. `AWS_ACCESS_KEY_ID` / `AWS_SECRET_ACCESS_KEY` / `AWS_SESSION_TOKEN`
5. デフォルト region: `us-east-1`

### `AwsConfig` 構造体（vm.rs）

```rust
pub struct AwsConfig {
    pub region: String,
    pub endpoint_url: Option<String>,
    pub access_key: String,
    pub secret_key: String,
    pub session_token: Option<String>,
}
```

`AWS_CONFIG` thread_local + `set_aws_config` で初期化。

---

## VM プリミティブ

### SigV4 署名

`hmac`・`sha2`（既存 Cargo deps）を使用してミニマルな SigV4 署名を実装する。

```rust
fn sign_request(config: &AwsConfig, service: &str, region: &str,
                method: &str, url: &str, body: &str, headers: &[(&str, &str)])
    -> HashMap<String, String>
{
    // 1. Canonical Request 生成
    // 2. StringToSign 生成
    // 3. Signing Key 導出 (HMAC-SHA256 × 4)
    // 4. Authorization ヘッダー生成
}
```

LocalStack の場合（`endpoint_url` が設定されている場合）は署名をスキップし、ダミー credentials を使用。

---

## S3 Rune

### `runes/aws/s3.fav`

```favnir
// オブジェクト取得
public fn get_object(bucket: String, key: String) -> Result<String, String> !AWS {
    AWS.s3_get_object_raw(bucket, key)
}

// オブジェクト保存
public fn put_object(bucket: String, key: String, body: String) -> Result<Unit, String> !AWS {
    AWS.s3_put_object_raw(bucket, key, body)
}

// オブジェクト削除
public fn delete_object(bucket: String, key: String) -> Result<Unit, String> !AWS {
    AWS.s3_delete_object_raw(bucket, key)
}

// オブジェクト一覧
public fn list_objects(bucket: String, prefix: String) -> Result<List<String>, String> !AWS {
    AWS.s3_list_objects_raw(bucket, prefix)
}

// バケット存在確認
public fn bucket_exists(bucket: String) -> Result<Bool, String> !AWS {
    AWS.s3_head_bucket_raw(bucket)
}
```

### S3 API 実装

| primitive | HTTP | エンドポイント |
|-----------|------|--------------|
| `s3_get_object_raw` | GET | `/{bucket}/{key}` |
| `s3_put_object_raw` | PUT | `/{bucket}/{key}` |
| `s3_delete_object_raw` | DELETE | `/{bucket}/{key}` |
| `s3_list_objects_raw` | GET | `/{bucket}?list-type=2&prefix={prefix}` |
| `s3_head_bucket_raw` | HEAD | `/{bucket}` |

レスポンス形式: S3 は XML を返すが、Favnir は String/Bool/List で抽象化する。

---

## SQS Rune

### `runes/aws/sqs.fav`

```favnir
// メッセージ送信
public fn send_message(queue_url: String, body: String) -> Result<String, String> !AWS {
    AWS.sqs_send_message_raw(queue_url, body)
}

// メッセージ受信
public fn receive_messages(queue_url: String, max: Int) -> Result<List<Map<String,String>>, String> !AWS {
    AWS.sqs_receive_messages_raw(queue_url, max)
}

// メッセージ削除（処理完了）
public fn delete_message(queue_url: String, receipt_handle: String) -> Result<Unit, String> !AWS {
    AWS.sqs_delete_message_raw(queue_url, receipt_handle)
}

// キューの URL 取得
public fn get_queue_url(queue_name: String) -> Result<String, String> !AWS {
    AWS.sqs_get_queue_url_raw(queue_name)
}
```

SQS メッセージの Map キー: `message_id`, `body`, `receipt_handle`, `attributes`

---

## DynamoDB Rune

### `runes/aws/dynamodb.fav`

```favnir
// 項目取得
public fn get_item(table: String, key: Map<String,String>) -> Result<Option<Map<String,String>>, String> !AWS {
    AWS.dynamo_get_item_raw(table, key)
}

// 項目保存
public fn put_item(table: String, item: Map<String,String>) -> Result<Unit, String> !AWS {
    AWS.dynamo_put_item_raw(table, item)
}

// 項目削除
public fn delete_item(table: String, key: Map<String,String>) -> Result<Unit, String> !AWS {
    AWS.dynamo_delete_item_raw(table, key)
}

// クエリ（KeyConditionExpression）
public fn query(table: String, condition: String, values: Map<String,String>) -> Result<List<Map<String,String>>, String> !AWS {
    AWS.dynamo_query_raw(table, condition, values)
}

// スキャン
public fn scan(table: String) -> Result<List<Map<String,String>>, String> !AWS {
    AWS.dynamo_scan_raw(table)
}
```

### DynamoDB API 形式

DynamoDB は JSON 形式の REST API（POST to `https://dynamodb.<region>.amazonaws.com/`）。

属性型変換（Favnir `String` ↔ DynamoDB AttributeValue）:
- 全フィールドを `{"S": "value"}` として扱う（v4.11.0 は String のみ対応）
- 数値は文字列として保存（`{"S": "42"}`）

---

## AWS バレルモジュール

### `runes/aws/aws.fav`

```favnir
use s3.*
use sqs.*
use dynamodb.*
```

```favnir
import rune "aws"

fn main() -> Unit !Io !AWS {
    match aws.get_object("my-bucket", "test.txt") {
        Ok(body) => IO.println(body)
        Err(e)   => IO.println($"Error: {e}")
    }
}
```

---

## `fav deploy` コマンド

### 概要

Favnir プロジェクトを AWS Lambda へデプロイする。

```
fav deploy [--env <env>] [--function <name>] [--dry-run] [--region <region>]
```

| オプション | デフォルト | 説明 |
|-----------|-----------|------|
| `--env` | `production` | デプロイ環境（fav.toml の `[[deploy.env]]` に対応） |
| `--function` | プロジェクト名 | Lambda 関数名 |
| `--dry-run` | false | 実際のデプロイをせず、ステップを表示 |
| `--region` | fav.toml の region | AWS リージョン |

### `fav.toml [deploy]` セクション

```toml
[deploy]
runtime = "provided.al2"   # Lambda カスタムランタイム
handler = "bootstrap"      # Lambda ハンドラ名
memory  = 256              # Lambda メモリ (MB)
timeout = 30               # Lambda タイムアウト (秒)
s3_bucket = "my-deploy-bucket"   # デプロイ用 S3 バケット
role_arn  = "arn:aws:iam::123456789:role/my-lambda-role"

[[deploy.env]]
name    = "production"
region  = "ap-northeast-1"
```

### デプロイフロー

```
1. cargo build --release (Favnir VM バイナリ)
   → v4.11.0 では実際ビルドせず scaffold のみ

2. .fav ファイルを zip 圧縮 → /tmp/<project>-<timestamp>.zip

3. S3 に zip をアップロード
   → PUT s3://<s3_bucket>/deploys/<project>/<timestamp>.zip

4. Lambda 関数の作成または更新
   → CreateFunction (初回) / UpdateFunctionCode (更新)

5. 完了メッセージ表示
```

### `--dry-run` 出力例

```
[deploy] Project: myapp v0.1.0
[deploy] Function: myapp
[deploy] Region: ap-northeast-1
[deploy] Runtime: provided.al2
[deploy] Memory: 256 MB, Timeout: 30s
[deploy] Step 1: Package .fav files → /tmp/myapp-20260517.zip (DRY RUN)
[deploy] Step 2: Upload to s3://my-deploy-bucket/deploys/myapp/20260517.zip (DRY RUN)
[deploy] Step 3: Update Lambda function 'myapp' (DRY RUN)
[deploy] Done (dry run — no changes made)
```

---

## テスト方針

### ユニットテスト (`fav/src/backend/vm_stdlib_tests.rs`)

| テスト | 内容 |
|--------|------|
| `aws_s3_get_object_raw_returns_err_on_bad_host` | 無効ホストで Err を返す |
| `aws_s3_put_object_raw_returns_err_on_bad_host` | 無効ホストで Err を返す |
| `aws_s3_list_objects_raw_returns_err_on_bad_host` | 無効ホストで Err を返す |
| `aws_sqs_send_message_raw_returns_err_on_bad_host` | 無効ホストで Err を返す |
| `aws_sqs_receive_messages_raw_returns_err_on_bad_host` | 無効ホストで Err を返す |
| `aws_dynamo_get_item_raw_returns_err_on_bad_host` | 無効ホストで Err を返す |
| `aws_dynamo_put_item_raw_returns_err_on_bad_host` | 無効ホストで Err を返す |
| `aws_dynamo_scan_raw_returns_err_on_bad_host` | 無効ホストで Err を返す |
| `aws_config_respects_endpoint_url` | endpoint_url が URL に反映される |

### 統合テスト (`fav/src/driver.rs`)

| テスト | 内容 |
|--------|------|
| `aws_rune_test_file_passes` | `runes/aws/aws.test.fav` の全テストが pass |
| `aws_s3_get_returns_err_in_favnir_source` | Favnir ソースから S3 Err を扱える |
| `aws_sqs_send_returns_err_in_favnir_source` | Favnir ソースから SQS Err を扱える |
| `aws_dynamo_scan_returns_err_in_favnir_source` | Favnir ソースから DynamoDB Err を扱える |
| `deploy_dry_run_prints_steps` | `--dry-run` が全ステップを表示する |

### Rune テスト (`runes/aws/aws.test.fav`)

```favnir
test "s3 get returns error on bad host" {
    match aws.get_object("nonexistent", "key") {
        Err(_) => true
        Ok(_)  => false
    }
}

test "sqs send returns error on bad host" {
    match aws.send_message("https://invalid", "hello") {
        Err(_) => true
        Ok(_)  => false
    }
}

test "dynamo scan returns error on bad host" {
    match aws.scan("nonexistent") {
        Err(_) => true
        Ok(_)  => false
    }
}
```

---

## 既知の制約

- v4.11.0 は SigV4 署名を実装するが、LocalStack での動作を主に検証する（本番 AWS はベストエフォート）
- DynamoDB は String 型属性のみ対応（Number/Binary/Set は将来）
- Lambda デプロイは scaffold のみ（実際の Lambda 実行ランタイムは v5.0.0 で対応）
- `fav deploy` は `fvc` アーティファクト（バイトコード）を Lambda に配置するのではなく、`.fav` ソースをパッケージとして送る（v4.11.0 の制限）
- SQS の属性（MessageAttributes, DelaySeconds 等）は v4.11.0 では未対応
- S3 の Multipart Upload は未対応（最大 ~5MB のオブジェクトまで）
