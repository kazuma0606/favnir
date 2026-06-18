# Favnir v11.4.0 実装計画

作成日: 2026-06-06

---

## 実装順序

```
Phase A: Emitter に boto3 フラグ追加
    ↓
Phase B: emit_apply の ("AWS", name) を実変換に置き換え
    ↓
Phase C: S3 ヘルパー関数 emit
    ↓
Phase D: DynamoDB ヘルパー関数 emit
    ↓
Phase E: SQS ヘルパー関数 emit
    ↓
Phase F: emit_prelude / emit_program 更新
    ↓
Phase G: cmd_transpile に pyproject.toml 生成を追加
    ↓
Phase H: テスト（v11400_tests）
    ↓
Phase I: バージョン更新・コミット
```

---

## Phase A — Emitter フラグ追加

`emit_python.rs` の `Emitter` 構造体に追加:

```rust
needs_boto3:          bool,
needs_base64:         bool,
needs_aws_s3:         bool,
needs_aws_dynamo:     bool,
needs_aws_sqs:        bool,
```

`Emitter::new()` で全て `false` に初期化。

---

## Phase B — emit_apply 更新

現在の `("AWS", name)` フォールバック:
```rust
("AWS", name) => {
    return format!("_aws_{}({})", name, a.join(", "))
}
```

これを個別ケースに展開:

```rust
// S3
("AWS", "s3_put_object_raw") if a.len() == 3 => {
    self.needs_boto3 = true; self.needs_aws_s3 = true;
    return format!("_aws_s3_put_object_raw({}, {}, {})", a[0], a[1], a[2])
}
("AWS", "s3_get_object_raw") if a.len() == 2 => {
    self.needs_boto3 = true; self.needs_aws_s3 = true;
    return format!("_aws_s3_get_object_raw({}, {})", a[0], a[1])
}
("AWS", "s3_list_objects_raw") if a.len() == 2 => {
    self.needs_boto3 = true; self.needs_aws_s3 = true;
    return format!("_aws_s3_list_objects_raw({}, {})", a[0], a[1])
}
("AWS", "s3_delete_object_raw") if a.len() == 2 => {
    self.needs_boto3 = true; self.needs_aws_s3 = true;
    return format!("_aws_s3_delete_object_raw({}, {})", a[0], a[1])
}
("AWS", "s3_get_object_base64_raw") if a.len() == 2 => {
    self.needs_boto3 = true; self.needs_aws_s3 = true; self.needs_base64 = true;
    return format!("_aws_s3_get_object_base64_raw({}, {})", a[0], a[1])
}
("AWS", "s3_put_bytes_raw") if a.len() == 3 => {
    self.needs_boto3 = true; self.needs_aws_s3 = true;
    return format!("_aws_s3_put_bytes_raw({}, {}, {})", a[0], a[1], a[2])
}
("AWS", "s3_head_bucket_raw") if a.len() == 1 => {
    self.needs_boto3 = true; self.needs_aws_s3 = true;
    return format!("_aws_s3_head_bucket_raw({})", a[0])
}
// DynamoDB
("AWS", "dynamo_scan_raw") if a.len() == 1 => {
    self.needs_boto3 = true; self.needs_aws_dynamo = true;
    return format!("_aws_dynamo_scan_raw({})", a[0])
}
("AWS", "dynamo_get_item_raw") if a.len() == 2 => {
    self.needs_boto3 = true; self.needs_aws_dynamo = true;
    return format!("_aws_dynamo_get_item_raw({}, {})", a[0], a[1])
}
("AWS", "dynamo_put_item_raw") if a.len() == 2 => {
    self.needs_boto3 = true; self.needs_aws_dynamo = true;
    return format!("_aws_dynamo_put_item_raw({}, {})", a[0], a[1])
}
("AWS", "dynamo_delete_item_raw") if a.len() == 2 => {
    self.needs_boto3 = true; self.needs_aws_dynamo = true;
    return format!("_aws_dynamo_delete_item_raw({}, {})", a[0], a[1])
}
("AWS", "dynamo_query_raw") if a.len() == 2 => {
    self.needs_boto3 = true; self.needs_aws_dynamo = true;
    return format!("_aws_dynamo_query_raw({}, {})", a[0], a[1])
}
// SQS
("AWS", "sqs_send_message_raw") if a.len() == 2 => {
    self.needs_boto3 = true; self.needs_aws_sqs = true;
    return format!("_aws_sqs_send_message_raw({}, {})", a[0], a[1])
}
("AWS", "sqs_receive_messages_raw") if a.len() == 2 => {
    self.needs_boto3 = true; self.needs_aws_sqs = true;
    return format!("_aws_sqs_receive_messages_raw({}, {})", a[0], a[1])
}
("AWS", "sqs_delete_message_raw") if a.len() == 2 => {
    self.needs_boto3 = true; self.needs_aws_sqs = true;
    return format!("_aws_sqs_delete_message_raw({}, {})", a[0], a[1])
}
("AWS", "sqs_get_queue_url_raw") if a.len() == 1 => {
    self.needs_boto3 = true; self.needs_aws_sqs = true;
    return format!("_aws_sqs_get_queue_url_raw({})", a[0])
}
// フォールバック
("AWS", name) => {
    self.needs_boto3 = true;
    return format!("_aws_{}({})", name, a.join(", "))
}
```

---

## Phase C — S3 ヘルパー関数

`emit_aws_s3_helpers()` メソッド追加:

```python
def _aws_s3_put_object_raw(bucket: str, key: str, body: str):
    try:
        boto3.client("s3").put_object(Bucket=bucket, Key=key, Body=body.encode("utf-8"))
        return Ok(None)
    except Exception as _e:
        return Err(str(_e))

def _aws_s3_get_object_raw(bucket: str, key: str):
    try:
        _body = boto3.client("s3").get_object(Bucket=bucket, Key=key)["Body"].read().decode("utf-8")
        return Ok(_body)
    except Exception as _e:
        return Err(str(_e))

def _aws_s3_list_objects_raw(bucket: str, prefix: str):
    try:
        _resp = boto3.client("s3").list_objects_v2(Bucket=bucket, Prefix=prefix)
        return Ok([_o["Key"] for _o in _resp.get("Contents", [])])
    except Exception as _e:
        return Err(str(_e))

def _aws_s3_delete_object_raw(bucket: str, key: str):
    try:
        boto3.client("s3").delete_object(Bucket=bucket, Key=key)
        return Ok(None)
    except Exception as _e:
        return Err(str(_e))

def _aws_s3_head_bucket_raw(bucket: str):
    try:
        boto3.client("s3").head_bucket(Bucket=bucket)
        return Ok(None)
    except Exception as _e:
        return Err(str(_e))
```

---

## Phase D — DynamoDB ヘルパー関数

`emit_aws_dynamo_helpers()` メソッド追加。
DynamoDB 型変換ユーティリティ（`_dynamo_serialize` / `_dynamo_deserialize`）も生成:

```python
def _dynamo_serialize(d: dict) -> dict:
    result = {}
    for k, v in d.items():
        if isinstance(v, bool):           result[k] = {"BOOL": v}
        elif isinstance(v, (int, float)): result[k] = {"N": str(v)}
        else:                             result[k] = {"S": str(v)}
    return result

def _dynamo_deserialize(item: dict) -> dict:
    result = {}
    for k, v in item.items():
        if "S" in v:     result[k] = v["S"]
        elif "N" in v:   result[k] = float(v["N"]) if "." in v["N"] else int(v["N"])
        elif "BOOL" in v: result[k] = v["BOOL"]
        else:            result[k] = str(v)
    return result

def _aws_dynamo_scan_raw(table: str):
    try:
        _items = boto3.client("dynamodb").scan(TableName=table)["Items"]
        return Ok([_dynamo_deserialize(_i) for _i in _items])
    except Exception as _e:
        return Err(str(_e))

# ... get_item / put_item / delete_item / query
```

---

## Phase E — SQS ヘルパー関数

`emit_aws_sqs_helpers()` メソッド追加:

```python
def _aws_sqs_send_message_raw(url: str, body: str):
    try:
        _resp = boto3.client("sqs").send_message(QueueUrl=url, MessageBody=body)
        return Ok(_resp.get("MessageId", ""))
    except Exception as _e:
        return Err(str(_e))

def _aws_sqs_receive_messages_raw(url: str, max_count: int):
    try:
        _resp = boto3.client("sqs").receive_message(QueueUrl=url, MaxNumberOfMessages=max_count)
        return Ok(_resp.get("Messages", []))
    except Exception as _e:
        return Err(str(_e))

def _aws_sqs_delete_message_raw(url: str, receipt: str):
    try:
        boto3.client("sqs").delete_message(QueueUrl=url, ReceiptHandle=receipt)
        return Ok(None)
    except Exception as _e:
        return Err(str(_e))

def _aws_sqs_get_queue_url_raw(name: str):
    try:
        return Ok(boto3.client("sqs").get_queue_url(QueueName=name)["QueueUrl"])
    except Exception as _e:
        return Err(str(_e))
```

---

## Phase F — emit_prelude / emit_program 更新

`emit_prelude` に追加:
```rust
if self.needs_boto3  { self.line("import boto3"); }
if self.needs_base64 { self.line("import base64 as _base64_mod"); }
```

`emit_program` の Phase 7 にヘルパー呼び出し追加:
```rust
if self.needs_aws_s3     { self.emit_aws_s3_helpers(); }
if self.needs_aws_dynamo { self.emit_aws_dynamo_helpers(); }
if self.needs_aws_sqs    { self.emit_aws_sqs_helpers(); }
```

フラグのコピー（Phase 3）にも `needs_boto3` / `needs_base64` / `needs_aws_*` を追加。

---

## Phase G — pyproject.toml 生成

`driver.rs` の `cmd_transpile` に追加:

```rust
// Python 生成後、pyproject.toml を出力ファイルと同ディレクトリに生成
let py_dir = std::path::Path::new(&out).parent().unwrap_or(std::path::Path::new("."));
let pyproject_path = py_dir.join("pyproject.toml");

// 既存の pyproject.toml がなければ生成
if !pyproject_path.exists() {
    let boto3_dep = if py_src.contains("import boto3") {
        "    \"boto3>=1.34\",\n"
    } else {
        ""
    };
    let content = format!(
        "[project]\nname = \"transpiled\"\nversion = \"0.1.0\"\nrequires-python = \">=3.11\"\ndependencies = [\n{}]\n\n[build-system]\nrequires = [\"hatchling\"]\nbuild-backend = \"hatchling.build\"\n",
        boto3_dep
    );
    let _ = std::fs::write(&pyproject_path, &content);
    println!("generated: {}", pyproject_path.display());
}
```

---

## Phase H — テスト（6件）

| テスト名 | 検証内容 |
|---|---|
| `transpile_aws_s3_put` | `AWS.s3_put_object_raw` → `_aws_s3_put_object_raw` + `import boto3` |
| `transpile_aws_s3_get` | `AWS.s3_get_object_raw` → `_aws_s3_get_object_raw` ヘルパー |
| `transpile_aws_s3_list` | `AWS.s3_list_objects_raw` → `list_objects_v2` ヘルパー |
| `transpile_aws_dynamo_scan` | `AWS.dynamo_scan_raw` → `_dynamo_deserialize` + `_aws_dynamo_scan_raw` |
| `transpile_aws_sqs_send` | `AWS.sqs_send_message_raw` → `_aws_sqs_send_message_raw` ヘルパー |
| `transpile_analyze_fav_aws_smoke` | `analyze.fav` 全体 → boto3 コード含む + `py_compile` 通過 |

---

## Phase I — バージョン更新

- `fav/Cargo.toml` version → `"11.4.0"`
- `cargo build` で `Cargo.lock` 更新
- `git commit & push` — CI 確認
