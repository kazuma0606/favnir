# v25.6.0 実装計画 — dynamodb Rune 実質化

## 実装順序

### Phase 1: Cargo.toml バンプ（T0）

```toml
[package]
version = "25.6.0"

# DynamoDB は既存の aws_post / SigV4 (reqwest + hmac + sha2) を再利用
# 追加 crate は不要
```

> **注意**: DynamoDB HTTP API は既存の `aws_post` ヘルパーで呼び出す。新規依存 crate は不要。

---

### Phase 2: Effect::DynamoDB 追加（T1〜T7）

`ast.rs` から始めて `cargo build` の exhaustive match エラーを潰していく。

**T1: `fav/src/ast.rs`**

```rust
// MongoDB の直後に追加
/// v25.6.0: DynamoDB Rune effect（AWS NoSQL KV）
DynamoDB,
```

**T2: `fav/src/error_catalog.rs`**

```rust
ErrorEntry {
    code: "E0323",
    title: "undeclared !DynamoDB effect",
    category: "effects",
    description: "A DynamoDB operation was used in a function that does not declare `!DynamoDB`.",
    example: "fn run(table: String) -> Result<String, String> {\n    DynamoDB.get_item_raw(conn, table, \"{}\")  // E0323: !DynamoDB not declared\n}",
    fix: "Add `!DynamoDB` to the function signature: `fn run(table: String) -> Result<String, String> !DynamoDB`.",
},
```

**T3: 6 ファイル一括更新**

| ファイル | 追加内容 |
|---|---|
| `fav/src/fmt.rs` | `Effect::DynamoDB => Some("!DynamoDB".to_string()),`（MongoDB アームの直後） |
| `fav/src/emit_python.rs` | `Effect::DynamoDB => "DynamoDB",`（MongoDB アームの直後） |
| `fav/src/lint.rs` | `Effect::DynamoDB => "DynamoDB",`（MongoDB アームの直後） |
| `fav/src/middle/reachability.rs` | `Effect::DynamoDB => { effects_required.insert("DynamoDB".to_string()); }` |
| `fav/src/middle/ast_lower_checker.rs` | `ast::Effect::DynamoDB => "DynamoDB".to_string(),` |
| `fav/src/lineage.rs` | format_effects に `DynamoDB => "!DynamoDB".into(),`、classify_capability_kind に `ast::Effect::DynamoDB => { return ("io".into(), Some("KvStore".into())) }` |

**T4: `fav/src/middle/checker.rs`**

```rust
fn require_dynamodb_effect(&mut self, span: &Span) {
    if !self.has_effect(|e| matches!(e, Effect::DynamoDB)) {
        self.type_error("E0323", "DynamoDB.* call requires `!DynamoDB` effect on enclosing fn/stage", span);
    }
}
```

`ns_to_inferred_effect` に追加:

```rust
"DynamoDB" => Some(Effect::DynamoDB),
```

DynamoDB builtin fns（checker.rs の builtin_fn_ret_ty 相当の場所）:

```rust
("DynamoDB", "connect_raw") => {
    self.require_dynamodb_effect(span);
    Some(Type::Result(Box::new(Type::String), Box::new(Type::String)))
}
("DynamoDB", "get_item_raw") | ("DynamoDB", "query_raw") | ("DynamoDB", "scan_raw") => {
    self.require_dynamodb_effect(span);
    Some(Type::Result(Box::new(Type::String), Box::new(Type::String)))
}
("DynamoDB", "put_item_raw") | ("DynamoDB", "delete_item_raw") | ("DynamoDB", "transact_write_raw") => {
    self.require_dynamodb_effect(span);
    Some(Type::Result(Box::new(Type::Unit), Box::new(Type::String)))
}
("DynamoDB", "batch_write_raw") => {
    self.require_dynamodb_effect(span);
    Some(Type::Result(Box::new(Type::Int), Box::new(Type::String)))
}
("DynamoDB", _) => {
    self.require_dynamodb_effect(span);
    Some(Type::Unknown)
}
```

**T5: `fav/src/frontend/parser.rs`**

```rust
// "MongoDB" アームの直後
"DynamoDB" => {
    self.advance();
    Effect::DynamoDB
}
```

**T6: `fav/src/driver.rs`**

`format_effects` 内:

```rust
DynamoDB => "!DynamoDB".into(),
```

`effect_json_name` 内:

```rust
ast::Effect::DynamoDB => "DynamoDB".into(),
```

**T7: `cargo build` で exhaustive match 確認**

---

### Phase 3: VM primitives 追加（T8）

`fav/src/backend/vm.rs` の MongoDB primitives ブロックの直後に追加。

#### DynamoDB ヘルパー関数（3 件）

```rust
// ── DynamoDB helpers (v25.6.0) ──────────────────────────────────────────────

/// プレーン JSON Value → DynamoDB 属性型 JSON 文字列
/// String  → {"S": "val"}
/// Number  → {"N": "1.0"}
/// Boolean → {"BOOL": true}
/// Null    → {"NULL": true}
/// Array   → {"L": [...]}
/// Object  → {"M": {...}}
#[cfg(not(target_arch = "wasm32"))]
fn json_val_to_dynamo_attr(v: &serde_json::Value) -> serde_json::Value {
    match v {
        serde_json::Value::String(s) => serde_json::json!({"S": s}),
        serde_json::Value::Number(n) => serde_json::json!({"N": n.to_string()}),
        serde_json::Value::Bool(b) => serde_json::json!({"BOOL": b}),
        serde_json::Value::Null => serde_json::json!({"NULL": true}),
        serde_json::Value::Array(arr) => {
            let items: Vec<serde_json::Value> = arr.iter().map(json_val_to_dynamo_attr).collect();
            serde_json::json!({"L": items})
        }
        serde_json::Value::Object(obj) => {
            let mut m = serde_json::Map::new();
            for (k, val) in obj {
                m.insert(k.clone(), json_val_to_dynamo_attr(val));
            }
            serde_json::json!({"M": m})
        }
    }
}

/// プレーン JSON オブジェクト → DynamoDB Item JSON 文字列
/// {"pk": "user1", "ttl": 1700} → {"pk":{"S":"user1"},"ttl":{"N":"1700"}}
#[cfg(not(target_arch = "wasm32"))]
fn json_to_dynamo_item(v: &serde_json::Value) -> Result<serde_json::Map<String, serde_json::Value>, String> {
    let obj = v.as_object().ok_or_else(|| "DynamoDB: expected JSON object for item/key".to_string())?;
    let mut item = serde_json::Map::new();
    for (k, val) in obj {
        item.insert(k.clone(), json_val_to_dynamo_attr(val));
    }
    Ok(item)
}

/// DynamoDB 属性値 JSON → プレーン JSON Value
/// {"S": "user1"} → "user1"
/// {"N": "1700"}  → 1700
#[cfg(not(target_arch = "wasm32"))]
fn dynamo_attr_to_json(v: &serde_json::Value) -> serde_json::Value {
    if let Some(obj) = v.as_object() {
        if let Some(s) = obj.get("S").and_then(|x| x.as_str()) {
            return serde_json::Value::String(s.to_string());
        }
        if let Some(n) = obj.get("N").and_then(|x| x.as_str()) {
            if let Ok(i) = n.parse::<i64>() { return serde_json::json!(i); }
            if let Ok(f) = n.parse::<f64>() {
                return serde_json::Number::from_f64(f)
                    .map(serde_json::Value::Number)
                    .unwrap_or(serde_json::Value::String(n.to_string()));
            }
        }
        if let Some(b) = obj.get("BOOL").and_then(|x| x.as_bool()) {
            return serde_json::Value::Bool(b);
        }
        if obj.get("NULL").is_some() {
            return serde_json::Value::Null;
        }
        if let Some(arr) = obj.get("L").and_then(|x| x.as_array()) {
            return serde_json::Value::Array(arr.iter().map(dynamo_attr_to_json).collect());
        }
        if let Some(m) = obj.get("M").and_then(|x| x.as_object()) {
            let mut out = serde_json::Map::new();
            for (k, val) in m { out.insert(k.clone(), dynamo_attr_to_json(val)); }
            return serde_json::Value::Object(out);
        }
    }
    v.clone()
}

/// DynamoDB Item (属性型 JSON オブジェクト) → プレーン JSON オブジェクト
#[cfg(not(target_arch = "wasm32"))]
fn dynamo_item_to_plain_json(item: &serde_json::Value) -> serde_json::Value {
    if let Some(obj) = item.as_object() {
        let mut out = serde_json::Map::new();
        for (k, v) in obj { out.insert(k.clone(), dynamo_attr_to_json(v)); }
        serde_json::Value::Object(out)
    } else {
        item.clone()
    }
}
```

#### DynamoDB.*_raw 8 件の実装概要

各 primitive は以下の共通パターン:

> **`aws_post` 実際のシグネチャ（vm.rs）**:
> ```rust
> fn aws_post(config: &AwsConfig, service: &str, url: &str, body: &str, content_type: &str, amz_target: Option<&str>) -> Result<String, String>
> ```
> 戻り値は `Result<String, String>` — `String` はレスポンス body（HTTP エラーは `Err` に変換済み）。`resp.status` フィールドは存在しない。

エンドポイント決定ロジック（`get_dynamo_endpoint` ヘルパー — `get_aws_config()` の `AwsConfig` を受け取る）:

```rust
/// DynamoConn 文字列（endpoint）から実際の DynamoDB endpoint URL を決定する
/// - 空または "default" → config.endpoint_url → LOCALSTACK_ENDPOINT → AWS 本番
/// - それ以外 → 文字列をそのまま使用
#[cfg(not(target_arch = "wasm32"))]
fn get_dynamo_endpoint(conn_endpoint: &str, config: &AwsConfig) -> String {
    if conn_endpoint.is_empty() || conn_endpoint == "default" {
        config.endpoint_url.as_deref()
            .map(|s| s.to_string())
            .or_else(|| std::env::var("LOCALSTACK_ENDPOINT").ok())
            .unwrap_or_else(|| format!("https://dynamodb.{}.amazonaws.com", config.region))
    } else {
        conn_endpoint.to_string()
    }
}
```

`connect_raw` の実装概要:

```rust
#[cfg(not(target_arch = "wasm32"))]
"DynamoDB.connect_raw" => {
    let mut it = args.into_iter();
    let endpoint_str = vm_string(it.next().ok_or("connect_raw: missing endpoint")?, "DynamoDB.connect_raw")?;
    let config = get_aws_config();
    let url = get_dynamo_endpoint(&endpoint_str, &config);
    // ListTables ping（接続確認）
    // TODO(v26.x): コネクションプール（現在は毎回 ListTables ping）
    match aws_post(&config, "dynamodb", &url, "{}", "application/x-amz-json-1.0", Some("DynamoDB_20120810.ListTables")) {
        Ok(_) => Ok(ok_vm(VMValue::Str(endpoint_str))),
        Err(e) => Ok(err_vm(VMValue::Str(format!("DynamoDB.connect_raw: ping failed: {}", e)))),
    }
}
```

各 primitive の AWS DynamoDB Action:

| primitive | Action | 主 body キー |
|---|---|---|
| `connect_raw` | `ListTables` | `{}` |
| `get_item_raw` | `GetItem` | `TableName` + `Key` |
| `put_item_raw` | `PutItem` | `TableName` + `Item` |
| `delete_item_raw` | `DeleteItem` | `TableName` + `Key` |
| `query_raw` | `Query` | `TableName` + `KeyConditionExpression` + `ExpressionAttributeValues` |
| `scan_raw` | `Scan` | `TableName` + `FilterExpression`（省略可）|
| `batch_write_raw` | `BatchWriteItem` | `RequestItems: {table: [{PutRequest: {Item: ...}}, ...]}` |
| `transact_write_raw` | `TransactWriteItems` | `TransactItems` |

`get_item_raw` の not_found パターン（MongoDB の find_one_raw と同パターン）:

```rust
// aws_post は Result<String, String>（String = response body）
let resp_str = aws_post(&config, "dynamodb", &url, &body, "application/x-amz-json-1.0", Some("DynamoDB_20120810.GetItem"))
    .map_err(|e| format!("DynamoDB.get_item_raw: table={}: {}", table, e))?;
let resp_json: serde_json::Value = serde_json::from_str(&resp_str)
    .map_err(|e| format!("DynamoDB.get_item_raw: JSON parse: {}", e))?;
// "Item" フィールドがない → not_found（DynamoDB の仕様）
if let Some(item) = resp_json.get("Item") {
    let plain = dynamo_item_to_plain_json(item);
    let s = serde_json::to_string(&plain).map_err(|e| format!("DynamoDB.get_item_raw: serialize: {}", e))?;
    Ok(ok_vm(VMValue::Str(s)))
} else {
    Ok(err_vm(VMValue::Str("not_found".into())))
}
```

`query_raw` / `scan_raw` の戻り値（JSON 配列文字列）:

```rust
let resp_str = aws_post(&config, "dynamodb", &url, &body, "application/x-amz-json-1.0", Some("DynamoDB_20120810.Query"))
    .map_err(|e| format!("DynamoDB.query_raw: table={}: {}", table, e))?;
let resp_json: serde_json::Value = serde_json::from_str(&resp_str)
    .map_err(|e| format!("DynamoDB.query_raw: JSON parse: {}", e))?;
let items = resp_json.get("Items").and_then(|x| x.as_array()).cloned().unwrap_or_default();
let plain: Vec<serde_json::Value> = items.iter().map(dynamo_item_to_plain_json).collect();
Ok(ok_vm(VMValue::Str(serde_json::to_string(&plain).map_err(|e| e.to_string())?)))
```

`scan_raw` の `filter_json` が空の場合:

```rust
// filter_json が空 → FilterExpression を含めない（全件スキャン）
let body = if filter_json.is_empty() {
    format!(r#"{{"TableName":"{}"}}"#, table)
} else {
    let filter_doc: serde_json::Value = serde_json::from_str(&filter_json)
        .map_err(|e| format!("DynamoDB.scan_raw: filter JSON parse: {}", e))?;
    format!(r#"{{"TableName":"{}","FilterExpression":{}}}"#, table, filter_doc)
};
```

`batch_write_raw` — 最大 25 件制限チェック（クロージャ内 `?` は `collect::<Result<_,_>>()?` パターン）:

```rust
let puts: Vec<serde_json::Value> = serde_json::from_str(&puts_json)
    .map_err(|e| format!("DynamoDB.batch_write_raw: JSON parse: {}", e))?;
if puts.len() > 25 {
    return Ok(err_vm(VMValue::Str(format!("DynamoDB.batch_write_raw: max 25 items per batch, got {}", puts.len()))));
}
let count = puts.len();
// クロージャ内で ? を使えないため collect::<Result<_,_>>()? パターン
let requests: Vec<serde_json::Value> = puts.iter()
    .map(|item| json_to_dynamo_item(item).map(|m| serde_json::json!({"PutRequest": {"Item": m}})))
    .collect::<Result<Vec<_>, _>>()?;
let body = serde_json::json!({"RequestItems": {table: requests}}).to_string();
match aws_post(&config, "dynamodb", &url, &body, "application/x-amz-json-1.0", Some("DynamoDB_20120810.BatchWriteItem")) {
    Ok(_) => Ok(ok_vm(VMValue::Int(count as i64))),
    Err(e) => Ok(err_vm(VMValue::Str(format!("DynamoDB.batch_write_raw: table={}: {}", table, e)))),
}
```

---

### Phase 4: runes/dynamodb/dynamodb.fav 更新（T9）

```favnir
// DynamoDB Rune (v25.6.0) — 接続エンドポイント URL ラッパー
type DynamoConn(String)

// "" または "default" → AWS 本番（AWS_ENDPOINT_URL / LOCALSTACK_ENDPOINT 環境変数を参照）
// "http://localhost:4566" → LocalStack
public fn connect(endpoint: String) -> Result<DynamoConn, String> !DynamoDB {
    DynamoDB.connect_raw(endpoint)
}

public fn get_item(conn: DynamoConn, table: String, key_json: String) -> Result<String, String> !DynamoDB {
    DynamoDB.get_item_raw(conn, table, key_json)
}

public fn put_item(conn: DynamoConn, table: String, item_json: String) -> Result<Unit, String> !DynamoDB {
    DynamoDB.put_item_raw(conn, table, item_json)
}

public fn delete_item(conn: DynamoConn, table: String, key_json: String) -> Result<Unit, String> !DynamoDB {
    DynamoDB.delete_item_raw(conn, table, key_json)
}

public fn query(conn: DynamoConn, table: String, key_cond: String, attr_vals_json: String) -> Result<String, String> !DynamoDB {
    DynamoDB.query_raw(conn, table, key_cond, attr_vals_json)
}

public fn scan(conn: DynamoConn, table: String, filter_json: String) -> Result<String, String> !DynamoDB {
    DynamoDB.scan_raw(conn, table, filter_json)
}

public fn batch_write(conn: DynamoConn, table: String, puts_json: String) -> Result<Int, String> !DynamoDB {
    DynamoDB.batch_write_raw(conn, table, puts_json)
}

public fn transact_write(conn: DynamoConn, ops_json: String) -> Result<Unit, String> !DynamoDB {
    DynamoDB.transact_write_raw(conn, ops_json)
}
```

---

### Phase 5: E2E デモ作成（T10）

`examples/dynamodb_session_store.fav` — spec.md の内容どおり作成。

---

### Phase 6: ドキュメント作成（T11）

`site/content/docs/runes/dynamodb.mdx` — MongoDB mdx を参考に全 API 記載。

---

### Phase 7: CHANGELOG 更新（T12）

`CHANGELOG.md` に `[v25.6.0]` エントリ追加。

---

### Phase 8: ベンチマーク + テスト（T13〜T15）

`benchmarks/v25.6.0.json`:

```json
{
  "version": "25.6.0",
  "test_count": 2014,
  "timestamp": "2026-06-25"
}
```

`fav/src/driver.rs` に `v256000_tests` モジュール（7 件）:

> **テスト設計**: v25.5.0 パターンに倣い、ast.rs + error_catalog.rs + checker.rs を 1 テストに統合（`effect_dynamodb_and_e0323_exist`）し、`transact_write_raw` の存在確認を追加して 7 件に収める。

```rust
#[cfg(test)]
mod v256000_tests {
    /// ast.rs に Effect::DynamoDB、error_catalog.rs に E0323、
    /// checker.rs に require_dynamodb_effect が存在することを一括確認（v25.5.0 パターン）
    #[test]
    fn effect_dynamodb_and_e0323_exist() {
        let ast_src = include_str!("ast.rs");
        assert!(ast_src.contains("DynamoDB,"), "Effect::DynamoDB missing in ast.rs");
        let cat_src = include_str!("error_catalog.rs");
        assert!(cat_src.contains("E0323"), "E0323 missing in error_catalog.rs");
        let chk_src = include_str!("middle/checker.rs");
        assert!(chk_src.contains("require_dynamodb_effect"), "require_dynamodb_effect missing in checker.rs");
    }
    #[test]
    fn dynamodb_connect_raw_in_vm() {
        let src = include_str!("backend/vm.rs");
        assert!(src.contains("DynamoDB.connect_raw"), "DynamoDB.connect_raw missing in vm.rs");
    }
    #[test]
    fn dynamodb_get_item_raw_in_vm() {
        let src = include_str!("backend/vm.rs");
        assert!(src.contains("DynamoDB.get_item_raw"), "DynamoDB.get_item_raw missing in vm.rs");
    }
    #[test]
    fn dynamodb_batch_write_raw_in_vm() {
        let src = include_str!("backend/vm.rs");
        assert!(src.contains("DynamoDB.batch_write_raw"), "DynamoDB.batch_write_raw missing in vm.rs");
    }
    #[test]
    fn dynamodb_transact_write_raw_in_vm() {
        let src = include_str!("backend/vm.rs");
        assert!(src.contains("DynamoDB.transact_write_raw"), "DynamoDB.transact_write_raw missing in vm.rs");
    }
    #[test]
    fn dynamodb_rune_file_has_connect() {
        let src = include_str!("../../runes/dynamodb/dynamodb.fav");
        assert!(src.contains("fn connect"), "connect missing in dynamodb.fav");
    }
    #[test]
    fn dynamodb_session_store_example_exists() {
        let src = include_str!("../../examples/dynamodb_session_store.fav");
        assert!(src.contains("import rune \"dynamodb\""), "import rune missing in example");
        assert!(src.contains("put_item"), "put_item missing in example");
        assert!(src.contains("get_item"), "get_item missing in example");
    }
}
```

---

## 注意事項

- `aws_post` ヘルパーは既存（vm.rs）。引数シグネチャ・戻り型を事前に確認してから呼び出す。
- DynamoDB HTTP API の `X-Amz-Target` ヘッダは `DynamoDB_20120810.<Action>` 形式。
- `batch_write_raw` の ops 組み立ては `json_to_dynamo_item` でアイテムを変換してから `PutRequest` にラップ。
- `transact_write_raw` は `ops_json` をそのまま `TransactItems` として渡す（ユーザー側で DynamoDB 形式の JSON を構築する責任）。
- `cfg(not(target_arch = "wasm32"))` ガードを全 DynamoDB primitive と helper に付与。
