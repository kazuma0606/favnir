# v25.8.0 実装計画 — elasticsearch Rune 実質化

## 実装順序

### Phase 1: Cargo.toml バンプ（T0）

```toml
[package]
version = "25.8.0"
# ureq = "2" は既存 native-only deps に存在 — 追加 crate 不要
```

---

### Phase 2: ast.rs — `Effect::Elasticsearch` 追加（T1）

`DynamoDB,` の直後に追加:

```rust
/// v25.8.0: Elasticsearch Rune effect（全文検索・ベクトル検索）
Elasticsearch,
```

---

### Phase 3: error_catalog.rs — E0324 追加（T2）

E0323（DynamoDB）エントリの閉じ `},` の**直後、E0365 エントリの前**に挿入:

```rust
ErrorEntry {
    code: "E0324",
    title: "undeclared !Elasticsearch effect",
    category: "effects",
    description: "An Elasticsearch operation was used in a function that does not declare `!Elasticsearch`.",
    example: "fn run(idx: String) -> Result<String, String> {\n    ES.search_raw(url, idx, \"{}\")  // E0324: !Elasticsearch not declared\n}",
    fix: "Add `!Elasticsearch` to the function signature: `fn run(idx: String) -> Result<String, String> !Elasticsearch`.",
},
```

---

### Phase 4: exhaustive match 更新 6 ファイル（T3）

`Effect::Elasticsearch` を追加した後、以下 6 ファイルの `match Effect::...` に追加:

#### `fav/src/fmt.rs`
```rust
Effect::Elasticsearch => Some("!Elasticsearch".to_string()),
```
（`Effect::DynamoDB` アームの直後）

#### `fav/src/emit_python.rs`
```rust
Effect::Elasticsearch => "Elasticsearch",
```

#### `fav/src/lint.rs`
```rust
Effect::Elasticsearch => "Elasticsearch",
```

#### `fav/src/middle/reachability.rs`
```rust
Effect::Elasticsearch => {
    effects_required.insert("Elasticsearch".to_string());
}
```

#### `fav/src/middle/ast_lower_checker.rs`
```rust
ast::Effect::Elasticsearch => "Elasticsearch".to_string(),
```

#### `fav/src/lineage.rs`
```rust
// format_effects の match（DynamoDB => "!DynamoDB".into(), の直後に追加）:
Elasticsearch => "!Elasticsearch".into(),
// classify_capability_kind の match（ast::Effect::DynamoDB アームの直後に追加）:
ast::Effect::Elasticsearch => { return ("io".into(), Some("Search".into())) }
```

---

### Phase 5: checker.rs 更新（T4）

#### `require_elasticsearch_effect` 追加（`require_dynamodb_effect` の直後）

```rust
fn require_elasticsearch_effect(&mut self, span: &Span) {
    if !self.has_effect(|e| matches!(e, Effect::Elasticsearch)) {
        self.type_error(
            "E0324",
            "ES.* call requires `!Elasticsearch` effect on enclosing fn/stage",
            span,
        );
    }
}
```

#### `ns_to_inferred_effect` に追加

```rust
"ES" | "Elasticsearch" => Some(Effect::Elasticsearch),
```
（`"DynamoDB"` アームの直後）

#### ES builtin fns 追加（DynamoDB ブロックの直後）

```rust
// Elasticsearch (v25.8.0) — require !Elasticsearch effect
// connect_raw の戻り型は Result<String, String>（checker レベル）。
// ESConn(String) は名目型ラッパー — checker は String として扱う（DynamoConn / KafkaConn と同パターン）。
("ES", "connect_raw") => {
    self.require_elasticsearch_effect(span);
    Some(Type::Result(Box::new(Type::String), Box::new(Type::String)))
}
("ES", "index_raw") | ("ES", "search_raw") | ("ES", "knn_search_raw") => {
    self.require_elasticsearch_effect(span);
    Some(Type::Result(Box::new(Type::String), Box::new(Type::String)))
}
("ES", "index_with_id_raw") | ("ES", "delete_raw") | ("ES", "create_index_raw") => {
    self.require_elasticsearch_effect(span);
    Some(Type::Result(Box::new(Type::Unit), Box::new(Type::String)))
}
("ES", "bulk_raw") => {
    self.require_elasticsearch_effect(span);
    Some(Type::Result(Box::new(Type::Int), Box::new(Type::String)))
}
("ES", _) => {
    self.require_elasticsearch_effect(span);
    Some(Type::Unknown)
}
```

---

### Phase 6: parser.rs 更新（T5）

`"DynamoDB" =>` アームの直後に追加:

```rust
"Elasticsearch" => {
    self.advance();
    Effect::Elasticsearch
}
```

---

### Phase 7: driver.rs 更新 — format_effects / effect_json_name（T6）

#### `format_effects` の match:
```rust
DynamoDB => "!DynamoDB".into(),
Elasticsearch => "!Elasticsearch".into(),
```

#### `effect_json_name` の match:
```rust
ast::Effect::DynamoDB => "DynamoDB".into(),
ast::Effect::Elasticsearch => "Elasticsearch".into(),
```

---

### Phase 8: cargo build チェック（T7）

```bash
cd fav && cargo build
```

exhaustive match エラーなし確認。

---

### Phase 9: vm.rs 更新（T8）

#### ヘルパー関数（DynamoDB helpers ブロックの後に追加）

```rust
// ── Elasticsearch helpers (v25.8.0) ───────────────────────────────────────────
// TODO(v26.x): コネクションプール（現在は毎回 GET / ping）

/// ESConn の URL を解決（空 → ELASTICSEARCH_URL 環境変数 → http://localhost:9200）
fn get_es_url(conn_url: &str) -> String {
    let s = conn_url.trim().to_string();
    if s.is_empty() {
        std::env::var("ELASTICSEARCH_URL")
            .unwrap_or_else(|_| "http://localhost:9200".to_string())
    } else {
        s
    }
}

/// Elasticsearch への HTTP リクエスト共通ヘルパー（ureq 使用）
/// ureq v2 の API: `ureq::request(method, url)` でメソッドを文字列指定（vm.rs 既存パターン）。
/// base64 は `base64::engine::general_purpose::STANDARD.encode(...)` インスタンスメソッド呼び出し。
fn es_http(method: &str, url: &str, body_opt: Option<&str>) -> Result<String, String> {
    use base64::Engine as _;
    let api_key = std::env::var("ELASTICSEARCH_API_KEY").ok();
    let username = std::env::var("ELASTICSEARCH_USERNAME").ok();
    let password = std::env::var("ELASTICSEARCH_PASSWORD").ok();

    let mut req = ureq::request(method, url).set("Content-Type", "application/json");
    if let Some(key) = &api_key {
        req = req.set("Authorization", &format!("ApiKey {key}"));
    } else if let (Some(u), Some(p)) = (&username, &password) {
        let encoded = base64::engine::general_purpose::STANDARD.encode(format!("{u}:{p}"));
        req = req.set("Authorization", &format!("Basic {encoded}"));
    }

    let resp = match body_opt {
        Some(body) => req.send_string(body).map_err(|e| e.to_string())?,
        None       => req.call().map_err(|e| e.to_string())?,
    };
    resp.into_string().map_err(|e| e.to_string())
}

/// es_http の ndjson バリアント（bulk 用）
fn es_http_ndjson(url: &str, body: &str) -> Result<String, String> {
    use base64::Engine as _;
    let api_key = std::env::var("ELASTICSEARCH_API_KEY").ok();
    let username = std::env::var("ELASTICSEARCH_USERNAME").ok();
    let password = std::env::var("ELASTICSEARCH_PASSWORD").ok();

    let mut req = ureq::post(url).set("Content-Type", "application/x-ndjson");
    if let Some(key) = &api_key {
        req = req.set("Authorization", &format!("ApiKey {key}"));
    } else if let (Some(u), Some(p)) = (&username, &password) {
        let encoded = base64::engine::general_purpose::STANDARD.encode(format!("{u}:{p}"));
        req = req.set("Authorization", &format!("Basic {encoded}"));
    }
    req.send_string(body)
        .map_err(|e| e.to_string())?
        .into_string()
        .map_err(|e| e.to_string())
}
```

#### VM primitives（既存 DynamoDB.transact_write_raw の後に追加）

```rust
// ── Elasticsearch primitives (v25.8.0) ────────────────────────────────────────
#[cfg(not(target_arch = "wasm32"))]
"ES.connect_raw" => {
    // (url: String) -> Result<String, String>
    let mut it = args.into_iter();
    let url_arg = vm_string(it.next().ok_or("ES.connect_raw: missing url")?, "ES.connect_raw")?;
    let url = get_es_url(&url_arg);
    match es_http("GET", &url, None) {
        Ok(_)  => Ok(ok_vm(VMValue::Str(url))),
        Err(e) => Ok(err_vm(VMValue::Str(format!("ES.connect_raw: ping failed: {e}")))),
    }
}
#[cfg(target_arch = "wasm32")]
"ES.connect_raw" => Ok(err_vm(VMValue::Str("Elasticsearch not supported on wasm32".into()))),

#[cfg(not(target_arch = "wasm32"))]
"ES.index_raw" => {
    // (url: String, index: String, doc_json: String) -> Result<String, String>
    let mut it = args.into_iter();
    let url_arg  = vm_string(it.next().ok_or("ES.index_raw: missing url")?,     "ES.index_raw")?;
    let index    = vm_string(it.next().ok_or("ES.index_raw: missing index")?,   "ES.index_raw")?;
    let doc_json = vm_string(it.next().ok_or("ES.index_raw: missing doc_json")?, "ES.index_raw")?;
    let url = get_es_url(&url_arg);
    let endpoint = format!("{url}/{index}/_doc");
    match es_http("POST", &endpoint, Some(&doc_json)) {
        Ok(resp) => {
            let v: serde_json::Value = serde_json::from_str(&resp)
                .map_err(|e| format!("ES.index_raw: parse: {e}"))?;
            let id = v["_id"].as_str().unwrap_or("").to_string();
            Ok(ok_vm(VMValue::Str(id)))
        }
        Err(e) => Ok(err_vm(VMValue::Str(format!("ES.index_raw: index={index}: {e}")))),
    }
}
#[cfg(target_arch = "wasm32")]
"ES.index_raw" => Ok(err_vm(VMValue::Str("Elasticsearch not supported on wasm32".into()))),

#[cfg(not(target_arch = "wasm32"))]
"ES.index_with_id_raw" => {
    // (url: String, index: String, id: String, doc_json: String) -> Result<Unit, String>
    let mut it = args.into_iter();
    let url_arg  = vm_string(it.next().ok_or("ES.index_with_id_raw: missing url")?,     "ES.index_with_id_raw")?;
    let index    = vm_string(it.next().ok_or("ES.index_with_id_raw: missing index")?,   "ES.index_with_id_raw")?;
    let id       = vm_string(it.next().ok_or("ES.index_with_id_raw: missing id")?,      "ES.index_with_id_raw")?;
    let doc_json = vm_string(it.next().ok_or("ES.index_with_id_raw: missing doc_json")?, "ES.index_with_id_raw")?;
    let url = get_es_url(&url_arg);
    let endpoint = format!("{url}/{index}/_doc/{id}");
    match es_http("PUT", &endpoint, Some(&doc_json)) {
        Ok(_)  => Ok(ok_vm(VMValue::Unit)),
        Err(e) => Ok(err_vm(VMValue::Str(format!("ES.index_with_id_raw: index={index}: {e}")))),
    }
}
#[cfg(target_arch = "wasm32")]
"ES.index_with_id_raw" => Ok(err_vm(VMValue::Str("Elasticsearch not supported on wasm32".into()))),

#[cfg(not(target_arch = "wasm32"))]
"ES.search_raw" => {
    // (url: String, index: String, query_json: String) -> Result<String, String>
    let mut it = args.into_iter();
    let url_arg    = vm_string(it.next().ok_or("ES.search_raw: missing url")?,       "ES.search_raw")?;
    let index      = vm_string(it.next().ok_or("ES.search_raw: missing index")?,     "ES.search_raw")?;
    let query_json = vm_string(it.next().ok_or("ES.search_raw: missing query_json")?, "ES.search_raw")?;
    let url = get_es_url(&url_arg);
    let endpoint = format!("{url}/{index}/_search");
    let body = if query_json.is_empty() { "{}".to_string() } else { query_json };
    match es_http("POST", &endpoint, Some(&body)) {
        Ok(resp) => {
            let v: serde_json::Value = serde_json::from_str(&resp)
                .map_err(|e| format!("ES.search_raw: parse: {e}"))?;
            // _source が null の hit は除外（_source: false 設定時の防御）
            let hits: Vec<serde_json::Value> = v["hits"]["hits"]
                .as_array()
                .unwrap_or(&vec![])
                .iter()
                .filter_map(|h| {
                    let src = &h["_source"];
                    if src.is_null() { None } else { Some(src.clone()) }
                })
                .collect();
            let s = serde_json::to_string(&hits)
                .map_err(|e| format!("ES.search_raw: serialize: {e}"))?;
            Ok(ok_vm(VMValue::Str(s)))
        }
        Err(e) => Ok(err_vm(VMValue::Str(format!("ES.search_raw: index={index}: {e}")))),
    }
}
#[cfg(target_arch = "wasm32")]
"ES.search_raw" => Ok(err_vm(VMValue::Str("Elasticsearch not supported on wasm32".into()))),

#[cfg(not(target_arch = "wasm32"))]
"ES.bulk_raw" => {
    // (url: String, index: String, docs_json: String) -> Result<Int, String>
    let mut it = args.into_iter();
    let url_arg   = vm_string(it.next().ok_or("ES.bulk_raw: missing url")?,       "ES.bulk_raw")?;
    let index     = vm_string(it.next().ok_or("ES.bulk_raw: missing index")?,     "ES.bulk_raw")?;
    let docs_json = vm_string(it.next().ok_or("ES.bulk_raw: missing docs_json")?, "ES.bulk_raw")?;
    let url = get_es_url(&url_arg);
    // JSON 配列 → NDJSON に変換
    let docs: Vec<serde_json::Value> = match serde_json::from_str(&docs_json) {
        Ok(serde_json::Value::Array(arr)) => arr,
        Ok(_) => return Ok(err_vm(VMValue::Str("ES.bulk_raw: docs_json must be a JSON array".into()))),
        Err(e) => return Ok(err_vm(VMValue::Str(format!("ES.bulk_raw: parse: {e}")))),
    };
    let count = docs.len();
    let mut ndjson = String::new();
    for doc in &docs {
        ndjson.push_str(&format!("{{\"index\":{{\"_index\":\"{index}\"}}}}\n"));
        ndjson.push_str(&doc.to_string());
        ndjson.push('\n');
    }
    let endpoint = format!("{url}/_bulk");
    match es_http_ndjson(&endpoint, &ndjson) {
        Ok(resp) => {
            let v: serde_json::Value = serde_json::from_str(&resp)
                .map_err(|e| format!("ES.bulk_raw: parse: {e}"))?;
            if v["errors"].as_bool().unwrap_or(false) {
                return Ok(err_vm(VMValue::Str(format!("ES.bulk_raw: index={index}: bulk had errors"))));
            }
            Ok(ok_vm(VMValue::Int(count as i64)))
        }
        Err(e) => Ok(err_vm(VMValue::Str(format!("ES.bulk_raw: index={index}: {e}")))),
    }
}
#[cfg(target_arch = "wasm32")]
"ES.bulk_raw" => Ok(err_vm(VMValue::Str("Elasticsearch not supported on wasm32".into()))),

#[cfg(not(target_arch = "wasm32"))]
"ES.delete_raw" => {
    // (url: String, index: String, id: String) -> Result<Unit, String>
    let mut it = args.into_iter();
    let url_arg = vm_string(it.next().ok_or("ES.delete_raw: missing url")?,   "ES.delete_raw")?;
    let index   = vm_string(it.next().ok_or("ES.delete_raw: missing index")?, "ES.delete_raw")?;
    let id      = vm_string(it.next().ok_or("ES.delete_raw: missing id")?,    "ES.delete_raw")?;
    let url = get_es_url(&url_arg);
    let endpoint = format!("{url}/{index}/_doc/{id}");
    match es_http("DELETE", &endpoint, None) {
        Ok(_)  => Ok(ok_vm(VMValue::Unit)),
        Err(e) => Ok(err_vm(VMValue::Str(format!("ES.delete_raw: index={index}: {e}")))),
    }
}
#[cfg(target_arch = "wasm32")]
"ES.delete_raw" => Ok(err_vm(VMValue::Str("Elasticsearch not supported on wasm32".into()))),

#[cfg(not(target_arch = "wasm32"))]
"ES.knn_search_raw" => {
    // (url: String, index: String, knn_json: String) -> Result<String, String>
    // knn_json: {"field": "embedding", "query_vector": [...], "k": 10, "num_candidates": 100}
    let mut it = args.into_iter();
    let url_arg  = vm_string(it.next().ok_or("ES.knn_search_raw: missing url")?,      "ES.knn_search_raw")?;
    let index    = vm_string(it.next().ok_or("ES.knn_search_raw: missing index")?,    "ES.knn_search_raw")?;
    let knn_json = vm_string(it.next().ok_or("ES.knn_search_raw: missing knn_json")?, "ES.knn_search_raw")?;
    let url = get_es_url(&url_arg);
    let endpoint = format!("{url}/{index}/_search");
    let body = format!("{{\"knn\": {knn_json}}}");
    match es_http("POST", &endpoint, Some(&body)) {
        Ok(resp) => {
            let v: serde_json::Value = serde_json::from_str(&resp)
                .map_err(|e| format!("ES.knn_search_raw: parse: {e}"))?;
            // _source が null の hit は除外（_source: false 設定時の防御）
            let hits: Vec<serde_json::Value> = v["hits"]["hits"]
                .as_array()
                .unwrap_or(&vec![])
                .iter()
                .filter_map(|h| {
                    let src = &h["_source"];
                    if src.is_null() { None } else { Some(src.clone()) }
                })
                .collect();
            let s = serde_json::to_string(&hits)
                .map_err(|e| format!("ES.knn_search_raw: serialize: {e}"))?;
            Ok(ok_vm(VMValue::Str(s)))
        }
        Err(e) => Ok(err_vm(VMValue::Str(format!("ES.knn_search_raw: index={index}: {e}")))),
    }
}
#[cfg(target_arch = "wasm32")]
"ES.knn_search_raw" => Ok(err_vm(VMValue::Str("Elasticsearch not supported on wasm32".into()))),

#[cfg(not(target_arch = "wasm32"))]
"ES.create_index_raw" => {
    // (url: String, index: String, mapping_json: String) -> Result<Unit, String>
    let mut it = args.into_iter();
    let url_arg      = vm_string(it.next().ok_or("ES.create_index_raw: missing url")?,         "ES.create_index_raw")?;
    let index        = vm_string(it.next().ok_or("ES.create_index_raw: missing index")?,       "ES.create_index_raw")?;
    let mapping_json = vm_string(it.next().ok_or("ES.create_index_raw: missing mapping_json")?, "ES.create_index_raw")?;
    let url = get_es_url(&url_arg);
    let endpoint = format!("{url}/{index}");
    let body = if mapping_json.is_empty() { "{}".to_string() } else { mapping_json };
    match es_http("PUT", &endpoint, Some(&body)) {
        Ok(_)  => Ok(ok_vm(VMValue::Unit)),
        Err(e) => Ok(err_vm(VMValue::Str(format!("ES.create_index_raw: index={index}: {e}")))),
    }
}
#[cfg(target_arch = "wasm32")]
"ES.create_index_raw" => Ok(err_vm(VMValue::Str("Elasticsearch not supported on wasm32".into()))),
```

---

### Phase 10: runes/elasticsearch/elasticsearch.fav 作成（T9）

```favnir
// runes/elasticsearch/elasticsearch.fav — Elasticsearch Rune (v25.8.0)
//
// 使い方:
//   import rune "elasticsearch"
//
// 環境変数:
//   ELASTICSEARCH_URL      — ベース URL（例: "http://localhost:9200"）
//   ELASTICSEARCH_API_KEY  — API キー認証（省略可）
//   ELASTICSEARCH_USERNAME / ELASTICSEARCH_PASSWORD — Basic 認証（省略可）
//
// ローカル開発:
//   docker run -p 9200:9200 \
//       -e "discovery.type=single-node" \
//       -e "xpack.security.enabled=false" \
//       elasticsearch:8.11.0

// ES ベース URL ラッパー型
// "" -> ELASTICSEARCH_URL 環境変数 -> "http://localhost:9200"
type ESConn(String)

// Elasticsearch への接続確認（GET / ping）
public fn connect(url: String) -> Result<ESConn, String> !Elasticsearch {
    ES.connect_raw(url)
}

// ドキュメントをインデックスする（ID 自動生成、生成された _id を返す）
public fn index(conn: ESConn, index: String, doc_json: String) -> Result<String, String> !Elasticsearch {
    ES.index_raw(conn, index, doc_json)
}

// ドキュメントをインデックスする（ID 指定）
public fn index_with_id(conn: ESConn, index: String, id: String, doc_json: String) -> Result<Unit, String> !Elasticsearch {
    ES.index_with_id_raw(conn, index, id, doc_json)
}

// 検索クエリを実行し、hits._source の JSON 配列文字列を返す
public fn search(conn: ESConn, index: String, query_json: String) -> Result<String, String> !Elasticsearch {
    ES.search_raw(conn, index, query_json)
}

// 複数ドキュメントをバルクインデックスし、インデックス件数を返す
public fn bulk(conn: ESConn, index: String, docs_json: String) -> Result<Int, String> !Elasticsearch {
    ES.bulk_raw(conn, index, docs_json)
}

// ドキュメントを削除する
public fn delete(conn: ESConn, index: String, id: String) -> Result<Unit, String> !Elasticsearch {
    ES.delete_raw(conn, index, id)
}

// ベクトル近傍検索（ES 8.x+ kNN）、hits._source の JSON 配列文字列を返す
// knn_json: {"field": "embedding", "query_vector": [...], "k": 10, "num_candidates": 100}
public fn knn_search(conn: ESConn, index: String, knn_json: String) -> Result<String, String> !Elasticsearch {
    ES.knn_search_raw(conn, index, knn_json)
}

// インデックスを作成する（mapping_json が "" の場合はデフォルトマッピング）
public fn create_index(conn: ESConn, index: String, mapping_json: String) -> Result<Unit, String> !Elasticsearch {
    ES.create_index_raw(conn, index, mapping_json)
}
```

---

### Phase 11: E2E デモ作成（T10）

`examples/elasticsearch_logs_etl.fav` — spec.md の内容どおり作成。

---

### Phase 12: ドキュメント作成（T11）

`site/content/docs/runes/elasticsearch.mdx` — DynamoDB / MongoDB mdx を参考に全 API 記載。

---

### Phase 13: CHANGELOG 更新（T12）

`CHANGELOG.md` に `[v25.8.0]` エントリ追加。

---

### Phase 14: ベンチマーク + テスト（T13〜T15）

`benchmarks/v25.8.0.json`:

```json
{
  "version": "25.8.0",
  "test_count": 2028,
  "timestamp": "2026-06-25"
}
```

`fav/src/driver.rs` に `v258000_tests` モジュール（7 件）:

```rust
#[cfg(test)]
mod v258000_tests {
    /// ast.rs に Effect::Elasticsearch、error_catalog.rs に E0324、
    /// checker.rs に require_elasticsearch_effect が存在することを一括確認
    #[test]
    fn effect_elasticsearch_and_e0324_exist() {
        let ast_src = include_str!("ast.rs");
        assert!(ast_src.contains("Elasticsearch,"), "Effect::Elasticsearch missing in ast.rs");
        let cat_src = include_str!("error_catalog.rs");
        assert!(cat_src.contains("E0324"), "E0324 missing in error_catalog.rs");
        let chk_src = include_str!("middle/checker.rs");
        assert!(chk_src.contains("require_elasticsearch_effect"), "require_elasticsearch_effect missing in checker.rs");
    }

    #[test]
    fn es_connect_raw_in_vm() {
        let src = include_str!("backend/vm.rs");
        assert!(src.contains("\"ES.connect_raw\""), "ES.connect_raw missing in vm.rs");
    }

    #[test]
    fn es_index_raw_in_vm() {
        let src = include_str!("backend/vm.rs");
        assert!(src.contains("\"ES.index_raw\""), "ES.index_raw missing in vm.rs");
    }

    #[test]
    fn es_search_raw_in_vm() {
        let src = include_str!("backend/vm.rs");
        assert!(src.contains("\"ES.search_raw\""), "ES.search_raw missing in vm.rs");
    }

    #[test]
    fn es_bulk_raw_in_vm() {
        let src = include_str!("backend/vm.rs");
        assert!(src.contains("\"ES.bulk_raw\""), "ES.bulk_raw missing in vm.rs");
    }

    #[test]
    fn es_rune_file_has_connect_and_search() {
        let src = include_str!("../../runes/elasticsearch/elasticsearch.fav");
        assert!(src.contains("fn connect"), "connect missing in elasticsearch.fav");
        assert!(src.contains("fn search"), "search missing in elasticsearch.fav");
        assert!(src.contains("type ESConn"), "ESConn missing in elasticsearch.fav");
    }

    #[test]
    fn es_logs_etl_example_exists() {
        let src = include_str!("../../examples/elasticsearch_logs_etl.fav");
        assert!(src.contains("import rune \"elasticsearch\""), "import rune missing in example");
        assert!(src.contains("index"), "index missing in example");
        assert!(src.contains("search"), "search missing in example");
    }
}
```

---

## 注意事項

- `ureq::delete(url)` は ureq v2 に存在する（`ureq::request("DELETE", url)` でも可）
- `base64::Engine::encode(...)` は `use base64::Engine;` が必要（既存 vm.rs に存在）
- `es_http_ndjson` で `Content-Type: application/x-ndjson` が必須（`application/json` では bulk が 400 になる）
- `GET /` レスポンスは `{"name": "...", "cluster_name": "...", "version": {...}}` 形式 — `Ok(_)` で ping 成功とみなす
- `Effect::Elasticsearch` を追加後 `cargo build` で exhaustive match を確認してから vm.rs を編集する（T7 の位置）
- lineage.rs の `classify_capability_kind` は `("io", "Search")` — DynamoDB の `"KvStore"` とは異なるカテゴリ
- E0324 の挿入位置: E0323（DynamoDB）エントリの直後
- `ESConn` から primitive への引数渡し: Rune の `conn: ESConn` が VM で `VMValue::Str`（ブローカーアドレス）として渡される（DynamoConn と同パターン）
