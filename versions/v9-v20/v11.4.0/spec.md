# Favnir v11.4.0 仕様書

作成日: 2026-06-06
テーマ: AWS エフェクト → boto3 変換

---

## 背景と目的

v11.3.0 で IO/Csv/Schema/Json の実変換が完了した。
v11.4.0 では `!AWS` エフェクトを boto3 Python コードに変換する。
現状の `_aws_s3_put_object_raw(...)` プレースホルダーを実際に動作する boto3 呼び出しに置き換え、
`fav transpile` が `.py` と合わせて `pyproject.toml`（boto3 依存付き）を生成するようにする。

**目標**: `analyze.fav` を変換した Python が `uv run python analyze.py` で
実際に S3 に書き込めること（boto3 コードが構文・意味的に正しいこと）。

---

## AWS プリミティブ変換対応表

### S3

| Fav | Python (boto3) | 戻り型 |
|---|---|---|
| `AWS.s3_put_object_raw(bucket, key, body)` | `boto3.client("s3").put_object(Bucket=bucket, Key=key, Body=body.encode("utf-8"))` | `Result<Unit, String>` |
| `AWS.s3_get_object_raw(bucket, key)` | `boto3.client("s3").get_object(Bucket=bucket, Key=key)["Body"].read().decode("utf-8")` | `Result<String, String>` |
| `AWS.s3_list_objects_raw(bucket, prefix)` | `[o["Key"] for o in boto3.client("s3").list_objects_v2(Bucket=bucket, Prefix=prefix).get("Contents", [])]` | `Result<List<String>, String>` |
| `AWS.s3_delete_object_raw(bucket, key)` | `boto3.client("s3").delete_object(Bucket=bucket, Key=key)` | `Result<Unit, String>` |
| `AWS.s3_get_object_base64_raw(bucket, key)` | get_object → base64.b64encode(...) | `Result<String, String>` |
| `AWS.s3_put_bytes_raw(bucket, key, bytes)` | `put_object(Body=bytes(body_list))` | `Result<Unit, String>` |
| `AWS.s3_head_bucket_raw(bucket)` | `head_bucket(Bucket=bucket)` | `Result<Unit, String>` |

### DynamoDB

| Fav | Python (boto3) | 戻り型 |
|---|---|---|
| `AWS.dynamo_scan_raw(table)` | `scan(TableName=table)["Items"]` + DynamoDB 型デシリアライズ | `Result<List<Map>, String>` |
| `AWS.dynamo_get_item_raw(table, key)` | `get_item(TableName=table, Key=...)["Item"]` | `Result<Option<Map>, String>` |
| `AWS.dynamo_put_item_raw(table, item)` | `put_item(TableName=table, Item=...)` | `Result<Unit, String>` |
| `AWS.dynamo_delete_item_raw(table, key)` | `delete_item(TableName=table, Key=...)` | `Result<Unit, String>` |

### SQS

| Fav | Python (boto3) | 戻り型 |
|---|---|---|
| `AWS.sqs_send_message_raw(url, body)` | `send_message(QueueUrl=url, MessageBody=body)` | `Result<String, String>` |
| `AWS.sqs_receive_messages_raw(url, max)` | `receive_message(QueueUrl=url, MaxNumberOfMessages=max)["Messages"]` | `Result<List<Map>, String>` |
| `AWS.sqs_delete_message_raw(url, receipt)` | `delete_message(QueueUrl=url, ReceiptHandle=receipt)` | `Result<Unit, String>` |
| `AWS.sqs_get_queue_url_raw(name)` | `get_queue_url(QueueName=name)["QueueUrl"]` | `Result<String, String>` |

---

## 生成コード設計

### boto3 ヘルパー関数の生成方式

S3/DynamoDB/SQS それぞれにヘルパー関数を生成し、try/except で `Ok`/`Err` ラップする。

```python
# S3 ヘルパー例
def _aws_s3_put_object_raw(bucket: str, key: str, body: str):
    try:
        _s3 = boto3.client("s3")
        _s3.put_object(Bucket=bucket, Key=key, Body=body.encode("utf-8"))
        return Ok(None)
    except Exception as _e:
        return Err(str(_e))

def _aws_s3_get_object_raw(bucket: str, key: str):
    try:
        _s3 = boto3.client("s3")
        _body = _s3.get_object(Bucket=bucket, Key=key)["Body"].read().decode("utf-8")
        return Ok(_body)
    except Exception as _e:
        return Err(str(_e))
```

### DynamoDB 型変換ユーティリティ

DynamoDB の boto3 API は `{"S": "val"}` / `{"N": "123"}` 形式。変換用ヘルパーを生成:

```python
def _dynamo_serialize(d: dict) -> dict:
    result = {}
    for k, v in d.items():
        if isinstance(v, str):   result[k] = {"S": v}
        elif isinstance(v, (int, float)): result[k] = {"N": str(v)}
        elif isinstance(v, bool): result[k] = {"BOOL": v}
        else: result[k] = {"S": str(v)}
    return result

def _dynamo_deserialize(item: dict) -> dict:
    result = {}
    for k, v in item.items():
        if "S" in v:    result[k] = v["S"]
        elif "N" in v:  result[k] = float(v["N"]) if "." in v["N"] else int(v["N"])
        elif "BOOL" in v: result[k] = v["BOOL"]
        else: result[k] = str(v)
    return result
```

### import 追加

| 使用 | 追加 import |
|---|---|
| AWS.s3_* | `import boto3` |
| AWS.dynamo_* | `import boto3` |
| AWS.sqs_* | `import boto3` |
| AWS.s3_get_object_base64_raw | `import base64` |

`Emitter` に `needs_boto3: bool` / `needs_base64: bool` フラグを追加。

---

## pyproject.toml 生成

`cmd_transpile` が `.py` ファイルを生成する際、同ディレクトリに `pyproject.toml` を生成する。
boto3 が必要な場合は依存に追加。

```toml
[project]
name = "transpiled"
version = "0.1.0"
requires-python = ">=3.11"
dependencies = [
    "boto3>=1.34",
]

[build-system]
requires = ["hatchling"]
build-backend = "hatchling.build"
```

boto3 不要な場合は `dependencies = []` のみ生成。

---

## analyze.fav 変換後の動作確認

`write_output` ステージの `AWS.s3_put_object_raw(...)` が boto3 コードになること:

```python
# effects: IO, AWS
def write_output(rows: List[TxnRow]) -> None:
    print(...)
    payload = _schema_to_json_array(rows, "TxnRow")
    def _match_4():
        _m4 = _aws_s3_put_object_raw("favnir-e2e-demo", "airgap/output/summary.json", payload)
        if isinstance(_m4, Err): ...
        ...
    return _match_4()
```

`uv run python -m py_compile analyze.py` が通ること。
