# Favnir v4.11.0 実装計画 — AWS SDK Rune & fav deploy

作成日: 2026-05-17

---

## Phase 0: バージョン更新

- `fav/Cargo.toml` の version を `"4.11.0"` に変更
- `fav/src/main.rs` のヘルプ文字列・バージョン表示を `4.11.0` に更新

---

## Phase 1: `!AWS` エフェクト (checker.rs)

```rust
// checker.rs の BUILTIN_EFFECTS に追加
const BUILTIN_EFFECTS: &[&str] = &[
    "Io", "File", "Db", "Http", "Grpc", "Auth", "Env", "DuckDb", "AWS",
];

// require_aws_effect 追加
fn require_aws_effect(&mut self, span: &Span) {
    if !self.current_effects.contains(&Effect::Unknown("AWS".to_string())) {
        self.errors.push(TypeError::new(
            "E0313",
            "AWS operations require the `!AWS` effect",
            span.clone(),
        ));
    }
}

// check_builtin_apply の ("AWS", _) アーム
("AWS", method) => {
    self.require_aws_effect(span);
    match method {
        "s3_get_object_raw" => Type::App("Result".into(), vec![
            Type::Str, Type::Str]),
        "s3_put_object_raw" => Type::App("Result".into(), vec![
            Type::Unit, Type::Str]),
        "s3_delete_object_raw" => Type::App("Result".into(), vec![
            Type::Unit, Type::Str]),
        "s3_list_objects_raw" => Type::App("Result".into(), vec![
            Type::App("List".into(), vec![Type::Str]), Type::Str]),
        "s3_head_bucket_raw" => Type::App("Result".into(), vec![
            Type::Bool, Type::Str]),
        "sqs_send_message_raw" => Type::App("Result".into(), vec![
            Type::Str, Type::Str]),
        "sqs_receive_messages_raw" => Type::App("Result".into(), vec![
            Type::App("List".into(), vec![
                Type::App("Map".into(), vec![Type::Str, Type::Str])
            ]), Type::Str]),
        "sqs_delete_message_raw" => Type::App("Result".into(), vec![
            Type::Unit, Type::Str]),
        "sqs_get_queue_url_raw" => Type::App("Result".into(), vec![
            Type::Str, Type::Str]),
        "dynamo_get_item_raw" => Type::App("Result".into(), vec![
            Type::App("Option".into(), vec![
                Type::App("Map".into(), vec![Type::Str, Type::Str])
            ]), Type::Str]),
        "dynamo_put_item_raw" => Type::App("Result".into(), vec![
            Type::Unit, Type::Str]),
        "dynamo_delete_item_raw" => Type::App("Result".into(), vec![
            Type::Unit, Type::Str]),
        "dynamo_query_raw" => Type::App("Result".into(), vec![
            Type::App("List".into(), vec![
                Type::App("Map".into(), vec![Type::Str, Type::Str])
            ]), Type::Str]),
        "dynamo_scan_raw" => Type::App("Result".into(), vec![
            Type::App("List".into(), vec![
                Type::App("Map".into(), vec![Type::Str, Type::Str])
            ]), Type::Str]),
        _ => Type::Unknown,
    }
}
```

`check_test_def` の `current_effects` に `Effect::Unknown("AWS".to_string())` を追加。

---

## Phase 2: `AwsConfig` + thread_local (vm.rs)

```rust
#[derive(Debug, Clone, Default)]
pub struct AwsConfig {
    pub region: String,
    pub endpoint_url: Option<String>,
    pub access_key: String,
    pub secret_key: String,
    pub session_token: Option<String>,
}

thread_local! {
    static AWS_CONFIG: RefCell<AwsConfig> = RefCell::new(AwsConfig::default());
}

pub fn set_aws_config(cfg: AwsConfig) {
    AWS_CONFIG.with(|c| *c.borrow_mut() = cfg);
}

fn get_aws_config() -> AwsConfig {
    AWS_CONFIG.with(|c| c.borrow().clone())
}
```

`AwsConfig::from_env()` ヘルパー:
```rust
impl AwsConfig {
    pub fn from_env() -> Self {
        Self {
            region: std::env::var("AWS_REGION")
                .or_else(|_| std::env::var("AWS_DEFAULT_REGION"))
                .unwrap_or_else(|_| "us-east-1".to_string()),
            endpoint_url: std::env::var("AWS_ENDPOINT_URL").ok(),
            access_key: std::env::var("AWS_ACCESS_KEY_ID")
                .unwrap_or_else(|_| "test".to_string()),
            secret_key: std::env::var("AWS_SECRET_ACCESS_KEY")
                .unwrap_or_else(|_| "test".to_string()),
            session_token: std::env::var("AWS_SESSION_TOKEN").ok(),
        }
    }
}
```

---

## Phase 3: SigV4 署名ミニ実装 (vm.rs)

```rust
use sha2::{Digest, Sha256};
use hmac::{Hmac, Mac};

type HmacSha256 = Hmac<Sha256>;

fn sha256_hex(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

fn hmac_sha256_bytes(key: &[u8], data: &[u8]) -> Vec<u8> {
    let mut mac = HmacSha256::new_from_slice(key).expect("hmac");
    mac.update(data);
    mac.finalize().into_bytes().to_vec()
}

fn sigv4_signing_key(secret: &str, date: &str, region: &str, service: &str) -> Vec<u8> {
    let k_secret = format!("AWS4{}", secret);
    let k_date = hmac_sha256_bytes(k_secret.as_bytes(), date.as_bytes());
    let k_region = hmac_sha256_bytes(&k_date, region.as_bytes());
    let k_service = hmac_sha256_bytes(&k_region, service.as_bytes());
    hmac_sha256_bytes(&k_service, b"aws4_request")
}

pub struct SignedHeaders {
    pub authorization: String,
    pub x_amz_date: String,
    pub x_amz_content_sha256: String,
    pub x_amz_security_token: Option<String>,
}

pub fn sigv4_sign(
    config: &AwsConfig,
    service: &str,
    method: &str,
    url: &str,
    body: &[u8],
) -> SignedHeaders {
    // LocalStack や endpoint_url 設定時はダミー認証
    if config.endpoint_url.is_some() {
        let now = "20240101T000000Z";
        return SignedHeaders {
            authorization: "AWS4-HMAC-SHA256 Credential=test/20240101/us-east-1/s3/aws4_request, SignedHeaders=host;x-amz-date, Signature=dummy".into(),
            x_amz_date: now.into(),
            x_amz_content_sha256: sha256_hex(body),
            x_amz_security_token: None,
        };
    }

    // 実 AWS: 正規 SigV4 署名
    let now = chrono::Utc::now();
    let amz_date = now.format("%Y%m%dT%H%M%SZ").to_string();
    let date_stamp = now.format("%Y%m%d").to_string();

    let body_hash = sha256_hex(body);
    let parsed_url = url::Url::parse(url).unwrap();
    let host = parsed_url.host_str().unwrap_or("");
    let path = parsed_url.path();
    let query = parsed_url.query().unwrap_or("");

    let canonical_headers = format!(
        "host:{}\nx-amz-content-sha256:{}\nx-amz-date:{}\n",
        host, body_hash, amz_date
    );
    let signed_headers = "host;x-amz-content-sha256;x-amz-date";
    let canonical_request = format!(
        "{}\n{}\n{}\n{}\n{}\n{}",
        method, path, query, canonical_headers, signed_headers, body_hash
    );
    let credential_scope = format!(
        "{}/{}/{}/aws4_request",
        date_stamp, config.region, service
    );
    let string_to_sign = format!(
        "AWS4-HMAC-SHA256\n{}\n{}\n{}",
        amz_date,
        credential_scope,
        sha256_hex(canonical_request.as_bytes())
    );
    let signing_key = sigv4_signing_key(&config.secret_key, &date_stamp, &config.region, service);
    let signature = hex::encode(hmac_sha256_bytes(&signing_key, string_to_sign.as_bytes()));
    let authorization = format!(
        "AWS4-HMAC-SHA256 Credential={}/{}, SignedHeaders={}, Signature={}",
        config.access_key, credential_scope, signed_headers, signature
    );

    SignedHeaders {
        authorization,
        x_amz_date: amz_date,
        x_amz_content_sha256: body_hash,
        x_amz_security_token: config.session_token.clone(),
    }
}
```

> **注**: `url` crate は新規依存が必要。または `url.parse()` を手動実装してもよい。v4.11.0 では `ureq` の URL 構築を使い、url crate は追加しない。

---

## Phase 4: S3 プリミティブ (vm.rs)

### URL 構築ヘルパー

```rust
fn s3_base_url(config: &AwsConfig, bucket: &str) -> String {
    if let Some(endpoint) = &config.endpoint_url {
        format!("{}/{}", endpoint.trim_end_matches('/'), bucket)
    } else {
        format!("https://{}.s3.{}.amazonaws.com", bucket, config.region)
    }
}
```

### `s3_get_object_raw(bucket, key)`

```rust
("AWS", "s3_get_object_raw") => {
    let key = /* args[1] */;
    let bucket = /* args[0] */;
    let config = get_aws_config();
    let url = format!("{}/{}", s3_base_url(&config, &bucket), key);
    let headers = sigv4_sign(&config, "s3", "GET", &url, b"");
    match ureq::get(&url)
        .set("Authorization", &headers.authorization)
        .set("x-amz-date", &headers.x_amz_date)
        .set("x-amz-content-sha256", &headers.x_amz_content_sha256)
        .call()
    {
        Ok(resp) => {
            let body = resp.into_string().unwrap_or_default();
            Ok(ok_vm(Value::Str(body)))
        }
        Err(e) => Ok(err_vm(&e.to_string())),
    }
}
```

### `s3_put_object_raw(bucket, key, body)`

PUT リクエスト。ボディは文字列。

### `s3_delete_object_raw(bucket, key)`

DELETE リクエスト。成功時は `Ok(Unit)`。

### `s3_list_objects_raw(bucket, prefix)`

GET `?list-type=2&prefix=<prefix>` → XML パース → `<Key>` 要素を抽出 → `List<String>`。

XML パースは正規表現または簡易文字列検索（`xml` クレート不使用）:
```rust
fn extract_xml_tags(xml: &str, tag: &str) -> Vec<String> {
    let open = format!("<{}>", tag);
    let close = format!("</{}>", tag);
    let mut results = Vec::new();
    let mut start = 0;
    while let Some(pos) = xml[start..].find(&open) {
        let abs = start + pos + open.len();
        if let Some(end) = xml[abs..].find(&close) {
            results.push(xml[abs..abs + end].to_string());
            start = abs + end + close.len();
        } else {
            break;
        }
    }
    results
}
```

### `s3_head_bucket_raw(bucket)`

HEAD リクエスト。200 → `Ok(Bool(true))`、404 → `Ok(Bool(false))`、その他エラー → `Err`。

---

## Phase 5: SQS プリミティブ (vm.rs)

SQS は HTTPS + クエリ文字列パラメーター（または JSON）形式の REST API。

### エンドポイント

```rust
fn sqs_base_url(config: &AwsConfig) -> String {
    if let Some(ep) = &config.endpoint_url {
        format!("{}", ep.trim_end_matches('/'))
    } else {
        format!("https://sqs.{}.amazonaws.com", config.region)
    }
}
```

### `sqs_send_message_raw(queue_url, body)`

```
POST <queue_url>
Action=SendMessage&MessageBody=<url_encoded_body>&Version=2012-11-05
```

レスポンス XML から `<MessageId>` を抽出して返す。

### `sqs_receive_messages_raw(queue_url, max_count)`

```
POST <queue_url>
Action=ReceiveMessage&MaxNumberOfMessages=<max>&Version=2012-11-05
```

レスポンス XML から `<Message>` ブロックを抽出:
- `<MessageId>` → `message_id`
- `<Body>` → `body`
- `<ReceiptHandle>` → `receipt_handle`

各メッセージを `Map<String,String>` にして `List` で返す。

### `sqs_delete_message_raw(queue_url, receipt_handle)`

```
POST <queue_url>
Action=DeleteMessage&ReceiptHandle=<encoded>&Version=2012-11-05
```

### `sqs_get_queue_url_raw(queue_name)`

```
POST https://sqs.<region>.amazonaws.com/
Action=GetQueueUrl&QueueName=<queue_name>&Version=2012-11-05
```

レスポンス XML から `<QueueUrl>` を抽出。

---

## Phase 6: DynamoDB プリミティブ (vm.rs)

DynamoDB は JSON ボディの REST API。

```rust
fn dynamo_url(config: &AwsConfig) -> String {
    if let Some(ep) = &config.endpoint_url {
        ep.trim_end_matches('/').to_string()
    } else {
        format!("https://dynamodb.{}.amazonaws.com", config.region)
    }
}
```

### 属性変換ヘルパー

```rust
fn map_to_dynamo_item(m: &HashMap<String, Value>) -> serde_json::Value {
    let mut item = serde_json::Map::new();
    for (k, v) in m {
        let attr_val = serde_json::json!({ "S": v.display() });
        item.insert(k.clone(), attr_val);
    }
    serde_json::Value::Object(item)
}

fn dynamo_item_to_map(item: &serde_json::Value) -> HashMap<String, Value> {
    let mut m = HashMap::new();
    if let serde_json::Value::Object(obj) = item {
        for (k, v) in obj {
            let s = v.get("S").and_then(|s| s.as_str()).unwrap_or("").to_string();
            m.insert(k.clone(), Value::Str(s));
        }
    }
    m
}
```

### `dynamo_get_item_raw(table, key_map)`

```json
POST /
X-Amz-Target: DynamoDB_20120810.GetItem
{ "TableName": "<table>", "Key": { ... } }
```

レスポンスの `Item` フィールドが存在しない場合 → `Ok(Variant("none", None))`。

### `dynamo_put_item_raw(table, item_map)`

```json
POST /
X-Amz-Target: DynamoDB_20120810.PutItem
{ "TableName": "<table>", "Item": { ... } }
```

### `dynamo_delete_item_raw(table, key_map)`

```json
POST /
X-Amz-Target: DynamoDB_20120810.DeleteItem
{ "TableName": "<table>", "Key": { ... } }
```

### `dynamo_query_raw(table, condition, values_map)`

```json
POST /
X-Amz-Target: DynamoDB_20120810.Query
{
  "TableName": "<table>",
  "KeyConditionExpression": "<condition>",
  "ExpressionAttributeValues": {
    ":val": { "S": "..." }
  }
}
```

### `dynamo_scan_raw(table)`

```json
POST /
X-Amz-Target: DynamoDB_20120810.Scan
{ "TableName": "<table>" }
```

---

## Phase 7: toml.rs — `[aws]` セクション

```rust
#[derive(Debug, Clone, Default)]
pub struct AwsTomlConfig {
    pub region: Option<String>,
    pub endpoint_url: Option<String>,
    pub profile: Option<String>,
}

// FavToml に追加:
pub aws: Option<AwsTomlConfig>,
```

`FavToml` を使用する箇所（checker.rs ×2, resolver.rs ×2, driver.rs ×1）に `aws: None` を追加。

---

## Phase 8: Rune ファイル

### `runes/aws/s3.fav`
### `runes/aws/sqs.fav`
### `runes/aws/dynamodb.fav`
### `runes/aws/aws.fav` (barrel)

```favnir
use s3.*
use sqs.*
use dynamodb.*
```

### `runes/aws/aws.test.fav`

bad-host テスト × 7件（上記 spec 参照）。

---

## Phase 9: `fav deploy` — driver.rs

```rust
pub fn cmd_deploy(
    env: Option<&str>,
    function_name: Option<&str>,
    region: Option<&str>,
    dry_run: bool,
) {
    use crate::toml::FavToml;

    let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let root = FavToml::find_root(&cwd).unwrap_or_else(|| {
        eprintln!("error: no fav.toml found");
        process::exit(1);
    });
    let toml = FavToml::load(&root).unwrap_or_else(|| {
        eprintln!("error: could not read fav.toml");
        process::exit(1);
    });

    let project_name = &toml.name;
    let func_name = function_name.unwrap_or(project_name);
    let deploy_cfg = toml.deploy.as_ref();
    let use_region = region
        .or_else(|| deploy_cfg.and_then(|d| d.region.as_deref()))
        .unwrap_or("us-east-1");
    let s3_bucket = deploy_cfg.and_then(|d| d.s3_bucket.as_deref()).unwrap_or("");
    let memory = deploy_cfg.map(|d| d.memory).unwrap_or(256);
    let timeout = deploy_cfg.map(|d| d.timeout).unwrap_or(30);
    let role_arn = deploy_cfg.and_then(|d| d.role_arn.as_deref()).unwrap_or("");

    println!("[deploy] Project: {} v{}", project_name, toml.version);
    println!("[deploy] Function: {}", func_name);
    println!("[deploy] Region: {}", use_region);
    println!("[deploy] Memory: {} MB, Timeout: {}s", memory, timeout);

    // Step 1: Package
    let timestamp = chrono::Utc::now().format("%Y%m%d%H%M%S");
    let zip_path = format!("/tmp/{}-{}.zip", project_name, timestamp);
    println!("[deploy] Step 1: Package → {}{}", zip_path, if dry_run { " (DRY RUN)" } else { "" });
    if !dry_run {
        package_project(&root, &zip_path);
    }

    // Step 2: Upload to S3
    if s3_bucket.is_empty() {
        if !dry_run {
            eprintln!("error: [deploy] s3_bucket is required in fav.toml");
            process::exit(1);
        }
    }
    let s3_key = format!("deploys/{}/{}.zip", project_name, timestamp);
    println!("[deploy] Step 2: Upload to s3://{}/{}{}", s3_bucket, s3_key, if dry_run { " (DRY RUN)" } else { "" });
    if !dry_run {
        upload_to_s3(s3_bucket, &s3_key, &zip_path);
    }

    // Step 3: Lambda update
    println!("[deploy] Step 3: Update Lambda '{}'{}", func_name, if dry_run { " (DRY RUN)" } else { "" });
    if !dry_run {
        update_lambda(func_name, s3_bucket, &s3_key, use_region, role_arn);
    }

    if dry_run {
        println!("[deploy] Done (dry run — no changes made)");
    } else {
        println!("[deploy] Done");
    }
}
```

### `package_project`

```rust
fn package_project(root: &std::path::Path, zip_path: &str) {
    // 簡易実装: src/ 以下の .fav ファイルを zip に格納
    // zip クレートを追加するか、シェルコマンドを使うか
    // v4.11.0: zip クレート追加 (zip = "0.6")
    use zip::write::FileOptions;
    let zip_file = std::fs::File::create(zip_path).expect("create zip");
    let mut zip = zip::ZipWriter::new(zip_file);
    let options = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    // .fav ファイルを再帰的に追加
    for entry in walkdir::WalkDir::new(root.join("src")) {
        let entry = entry.expect("walkdir");
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("fav") {
            let rel = path.strip_prefix(root).expect("strip").to_string_lossy();
            zip.start_file(rel.as_ref(), options).expect("start_file");
            let content = std::fs::read(path).expect("read");
            zip.write_all(&content).expect("write");
        }
    }
    // fav.toml も追加
    let toml_path = root.join("fav.toml");
    if toml_path.exists() {
        zip.start_file("fav.toml", options).expect("start fav.toml");
        let content = std::fs::read(toml_path).expect("read toml");
        zip.write_all(&content).expect("write toml");
    }
    zip.finish().expect("zip finish");
}
```

### `upload_to_s3` / `update_lambda`

AWS API を ureq + SigV4 で呼び出す（S3 PUT + Lambda UpdateFunctionCode API）。

---

## Phase 10: CLI 配線 (main.rs)

```rust
Some("deploy") => {
    let mut env: Option<String> = None;
    let mut function_name: Option<String> = None;
    let mut region: Option<String> = None;
    let mut dry_run = false;
    let mut i = 2usize;
    while i < args.len() {
        match args[i].as_str() {
            "--env" => { env = Some(args.get(i+1)...); i += 2; }
            "--function" => { function_name = Some(args.get(i+1)...); i += 2; }
            "--region" => { region = Some(args.get(i+1)...); i += 2; }
            "--dry-run" => { dry_run = true; i += 1; }
            other => { eprintln!("error: unexpected deploy argument `{}`", other); ... }
        }
    }
    cmd_deploy(env.as_deref(), function_name.as_deref(), region.as_deref(), dry_run);
}
```

HELP テキスト:
```
    deploy [--env <name>] [--function <name>] [--region <r>] [--dry-run]
                  Deploy to AWS Lambda (packages .fav files and uploads to S3/Lambda).
```

---

## Phase 11: テスト

### vm_stdlib_tests.rs

bad-host テスト: AwsConfig に `endpoint_url = Some("http://invalid.host:9999")` を設定し、各プリミティブが `Err` を返すことを確認。

### driver.rs — `aws_tests` モジュール

Favnir ソースから aws.get_object 等を呼び出し、Err を正しく処理できることを確認。

---

## Cargo.toml 変更

```toml
# 新規追加
zip = { version = "0.6", default-features = false, features = ["deflate"] }
```

> **注**: `hex` クレートは SigV4 で必要だが、`sha2`/`hmac` がすでに `hex` encoding を `format!("{:x}", ...)` で代替できる。追加不要。

---

## 実装メモ

- **`aws: None` を FavToml リテラル 5 箇所に追加**: checker.rs ×2, resolver.rs ×2, driver.rs ×1
- **LocalStack 使用時の認証**: `endpoint_url` が Some の場合、SigV4 署名を省略してダミー Authorization ヘッダーを送る
- **SQS の URL エンコード**: `ureq::request("POST", url).send_form(&[...])` を使用
- **DynamoDB の `X-Amz-Target` ヘッダー**: `ureq::request("POST", url).set("X-Amz-Target", "DynamoDB_20120810.PutItem")`
- **S3 の `Content-MD5`**: v4.11.0 では省略（LocalStack は不要、本番 AWS は任意）
- **`hex` encoding**: `format!("{:02x}", b)` のコレクターで代替 (`.iter().map(|b| format!("{:02x}", b)).collect::<String>()`)
- **`deploy` の zip サイズ制限**: Lambda への直接アップロードは 50MB まで。S3 経由なら 250MB まで対応。v4.11.0 は S3 経由のみ
