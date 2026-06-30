# v25.5.0 実装計画 — mongodb Rune 実質化

## 実装順序

```
Step 0  Cargo.toml: version bump + mongodb crate 追加
Step 1  ast.rs: Effect::MongoDB 追加（MySQL の直後）
Step 2  error_catalog.rs: E0322 追加
Step 3  fmt.rs / lineage.rs / emit_python.rs / lint.rs /
        reachability.rs / ast_lower_checker.rs: Effect::MongoDB 対応（6 ファイル）
Step 4  checker.rs: require_mongodb_effect / ns_to_inferred_effect / Mongo builtin fns 追加
Step 5  parser.rs: "MongoDB" => Effect::MongoDB アーム追加
Step 6  driver.rs: format_effects / effect_json_name に MongoDB アーム追加
Step 7  vm.rs: Mongo.*_raw 8 件追加 + extract_db_name ヘルパー
Step 8  runes/mongodb/mongodb.fav: type MongoConn + 8 関数 全面更新
Step 9  examples/mongo_events_etl.fav: 新規作成
Step 10 site/content/docs/runes/mongodb.mdx: 新規作成
Step 11 CHANGELOG.md: [v25.5.0] エントリ追加
Step 12 benchmarks/v25.5.0.json: 新規作成（test_count: 2007）
Step 13 driver.rs: v255000_tests 7 件追加
Step 14 cargo test v255000: 7 件 PASS 確認
Step 15 cargo test: 総テスト数 ≥ 2007 件 確認
Step 16 spec-reviewer レビュー実施
```

---

## 詳細実装手順

### Step 0 — Cargo.toml

```toml
# [target.'cfg(not(target_arch = "wasm32"))'.dependencies] セクションに追加
# mysql = ... の直後に配置
mongodb = { version = "3", default-features = false, features = ["tokio-runtime"] }
```

バージョンを `25.5.0` に bump:

```toml
version = "25.5.0"
```

> **crate 選定**: `mongodb = "3"` の `sync` feature は v3 で廃止済み。**`tokio-runtime` feature が主方針**。
> vm.rs では `tokio::runtime::Builder::new_current_thread().enable_all().build()?.block_on(async { ... })` で
> 非同期 API を同期化する（`tokio = { version = "1", features = ["full"] }` は既に存在）。

### Step 1 — ast.rs: Effect::MongoDB

```rust
// MySQL, の後、AzureDb の前（ただし MySQL の直後）
MySQL,
/// v25.5.0: MongoDB Rune effect（ドキュメント系 NoSQL 専用）
MongoDB,
AzureDb,
```

### Step 2 — error_catalog.rs: E0322

E0321（MySQL）の後に追加:

```rust
ErrorEntry {
    code: "E0322",
    title: "undeclared !MongoDB effect",
    category: "effects",
    description: "A MongoDB operation was used in a function that does not declare `!MongoDB`.",
    example: "fn run(coll: String) -> Result<String, String> {\n    Mongo.find_raw(conn, coll, \"{}\")  // E0322: !MongoDB not declared\n}",
    fix: "Add `!MongoDB` to the function signature: `fn run(coll: String) -> Result<String, String> !MongoDB`.",
},
```

### Step 3 — 6 ファイル一括更新（Effect::MongoDB 追加）

各ファイルの `Effect::MySQL` アームの直後に `MongoDB` を追加:

| ファイル | 追加場所 | 追加内容 |
|---|---|---|
| `fmt.rs` | `Effect::MySQL =>` の後 | `Effect::MongoDB => Some("!MongoDB".to_string())` |
| `lineage.rs` (format_effects) | `MySQL =>` の後 | `MongoDB => "!MongoDB".into()` |
| `lineage.rs` (classify_capability_kind) | `ast::Effect::MySQL =>` の後 | `ast::Effect::MongoDB => { return ("io".into(), Some("DocStore".into())) }` |
| `emit_python.rs` | `Effect::MySQL =>` の後 | `Effect::MongoDB => "MongoDB"` |
| `lint.rs` | `Effect::MySQL =>` の後 | `Effect::MongoDB => "MongoDB"` |
| `reachability.rs` | `Effect::MySQL =>` の後 | `Effect::MongoDB => { effects_required.insert("MongoDB".to_string()); }` |
| `ast_lower_checker.rs` | `ast::Effect::MySQL =>` の後 | `ast::Effect::MongoDB => "MongoDB".to_string()` |

> **lineage.rs の分類**: MongoDB はドキュメント DB でありリレーショナル DB とは異なる。
> `"io"` / `"DocStore"` に分類することで Postgres / MySQL（`"read"/"DbRead"`）と区別する。

### Step 4 — checker.rs: require_mongodb_effect + builtin fns

```rust
fn require_mongodb_effect(&mut self, span: &Span) {
    if !self.has_effect(|e| matches!(e, Effect::MongoDB)) {
        self.type_error(
            "E0322",
            "Mongo.* call requires `!MongoDB` effect on enclosing fn/stage",
            span,
        );
    }
}
```

`ns_to_inferred_effect` に追加:
```rust
"Mongo" | "MongoDB" => Some(Effect::MongoDB),
```

builtin fns:
```rust
// MongoDB (v25.5.0) — require !MongoDB effect
// connect_raw の戻り型は Result<String, String>（checker レベル）。
// MongoConn(String) は名目型ラッパー — PgConn / RedisConn / MySqlConn と同パターン。
("Mongo", "connect_raw") => {
    self.require_mongodb_effect(span);
    Some(Type::Result(Box::new(Type::String), Box::new(Type::String)))
}
("Mongo", "find_raw") | ("Mongo", "find_one_raw") | ("Mongo", "aggregate_raw") | ("Mongo", "insert_one_raw") => {
    self.require_mongodb_effect(span);
    Some(Type::Result(Box::new(Type::String), Box::new(Type::String)))
}
("Mongo", "insert_many_raw") | ("Mongo", "update_one_raw") | ("Mongo", "delete_one_raw") => {
    self.require_mongodb_effect(span);
    Some(Type::Result(Box::new(Type::Int), Box::new(Type::String)))
}
("Mongo", _) => {
    self.require_mongodb_effect(span);
    Some(Type::Unknown)
}
```

### Step 5 — parser.rs

`"MySQL"` アームの直後に追加:
```rust
"MongoDB" => {
    self.advance();
    Effect::MongoDB
}
```

### Step 6 — driver.rs: format_effects / effect_json_name

`format_effects` の `MySQL` アームの後:
```rust
MySQL => "!MySQL".into(),
MongoDB => "!MongoDB".into(),
```

`effect_json_name` の `MySQL` アームの後:
```rust
ast::Effect::MySQL => "MySQL".into(),
ast::Effect::MongoDB => "MongoDB".into(),
```

### Step 7 — vm.rs: Mongo.*_raw 8 件 + ヘルパー

MySQL primitives セクションの直後に追加。

#### extract_mongo_db_name ヘルパー（vm.rs グローバル関数として追加）

```rust
#[cfg(not(target_arch = "wasm32"))]
fn extract_mongo_db_name(url: &str) -> String {
    // "mongodb://user:pass@host:port/dbname" → "dbname"
    // "mongodb://host:port" → "test"（パスなし・ポート番号のみの場合はデフォルト）
    let after_scheme = url
        .strip_prefix("mongodb://")
        .or_else(|| url.strip_prefix("mongodb+srv://"))
        .unwrap_or(url);
    // "host:port/dbname" → スラッシュ以降を取り出す
    if let Some(slash_pos) = after_scheme.find('/') {
        let db_part = &after_scheme[slash_pos + 1..];
        let db_name = db_part.split('?').next().unwrap_or(db_part);
        if !db_name.is_empty() {
            return db_name.to_string();
        }
    }
    "test".to_string()
}
```

#### Mongo.connect_raw

```rust
"Mongo.connect_raw" => {
    let url = vm_string(args.into_iter().next()...)?;
    let url_clone = url.clone();
    let db_name = extract_mongo_db_name(&url);
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| format!("tokio build error: {}", e))?
        .block_on(async {
            let client = mongodb::Client::with_uri_str(&url_clone)
                .await
                .map_err(|e| format!("Mongo.connect_raw invalid URL: {}", e))?;
            // ping で接続確認
            client.database(&db_name)
                .run_command(mongodb::bson::doc! { "ping": 1 })
                .await
                .map_err(|e| format!("Mongo.connect_raw ping error: {}", e))?;
            Ok::<_, String>(())
        })?;
    Ok(ok_vm(VMValue::Str(url)))
}
```

#### Mongo.find_raw

```rust
"Mongo.find_raw" => {
    // url, coll, filter_json
    let filter_doc: mongodb::bson::Document = serde_json::from_str(&filter_json)
        .and_then(|v| mongodb::bson::to_document(&v)...)
        ...
    let cursor = collection.find(filter_doc).run()?;
    let docs: Vec<serde_json::Value> = cursor.map(|r| bson_to_json(r?)).collect()?;
    Ok(ok_vm(VMValue::Str(serde_json::to_string(&docs)?)))
}
```

#### Mongo.find_one_raw

見つかった場合は JSON オブジェクト文字列、見つからない場合は `Result.err("not_found")` を返す。

#### Mongo.insert_one_raw

`collection.insert_one(doc).run()` → `inserted_id` を `ObjectId` 文字列として返す。

#### Mongo.insert_many_raw

`InsertManyResult` に直接 count フィールドはないため `.inserted_ids.len()` で件数を取得:

```rust
let result = collection.insert_many(bson_docs).await
    .map_err(|e| format!("Mongo.insert_many_raw error on '{}': {}", coll, e))?;
let count = result.inserted_ids.len() as i64;
Ok(ok_vm(VMValue::Int(count)))
```

#### Mongo.update_one_raw

`collection.update_one(filter, update).run()` → `modified_count` を `Int` で返す。

#### Mongo.delete_one_raw

`collection.delete_one(filter).run()` → `deleted_count` を `Int` で返す。

#### Mongo.aggregate_raw

`collection.aggregate(pipeline).run()` → JSON 配列文字列で返す。

#### BSON ↔ JSON 変換ヘルパー

```rust
#[cfg(not(target_arch = "wasm32"))]
fn mongo_bson_to_json(doc: mongodb::bson::Document) -> serde_json::Value {
    // mongodb::bson::Document は serde::Serialize 実装済み
    // ObjectId は serde_json 経由では binary になる可能性があるため、
    // Document の各フィールドを走査し Bson::ObjectId → {"$oid": "hex"} に変換する
    fn bson_to_json_value(b: mongodb::bson::Bson) -> serde_json::Value {
        match b {
            mongodb::bson::Bson::ObjectId(oid) => {
                serde_json::json!({"$oid": oid.to_hex()})
            }
            mongodb::bson::Bson::Document(d) => {
                let map: serde_json::Map<_, _> = d.into_iter()
                    .map(|(k, v)| (k, bson_to_json_value(v)))
                    .collect();
                serde_json::Value::Object(map)
            }
            mongodb::bson::Bson::Array(arr) => {
                serde_json::Value::Array(arr.into_iter().map(bson_to_json_value).collect())
            }
            other => serde_json::to_value(&other).unwrap_or(serde_json::Value::Null),
        }
    }
    bson_to_json_value(mongodb::bson::Bson::Document(doc))
}

#[cfg(not(target_arch = "wasm32"))]
fn mongo_json_to_bson(v: &str) -> Result<mongodb::bson::Document, String> {
    let json: serde_json::Value = serde_json::from_str(v)
        .map_err(|e| format!("JSON parse error: {}", e))?;
    mongodb::bson::to_document(&json)
        .map_err(|e| format!("BSON conversion error: {}", e))
}
```

> **実装上の注意**: `mongodb = "3"` の非同期 API は `collection.find(filter).await` 形式。
> 全 primitive は `tokio::runtime::Builder::new_current_thread().enable_all().build()?.block_on(async { ... })` で同期化する。

### Step 8 — runes/mongodb/mongodb.fav

```favnir
// runes/mongodb/mongodb.fav — MongoDB Rune (v25.5.0)
type MongoConn(String)

public fn connect(url: String) -> Result<MongoConn, String> !MongoDB {
    Mongo.connect_raw(url)
}
public fn find(conn: MongoConn, coll: String, filter: String) -> Result<String, String> !MongoDB {
    Mongo.find_raw(conn, coll, filter)
}
public fn find_one(conn: MongoConn, coll: String, filter: String) -> Result<String, String> !MongoDB {
    Mongo.find_one_raw(conn, coll, filter)
}
public fn insert_one(conn: MongoConn, coll: String, doc: String) -> Result<String, String> !MongoDB {
    Mongo.insert_one_raw(conn, coll, doc)
}
public fn insert_many(conn: MongoConn, coll: String, docs: String) -> Result<Int, String> !MongoDB {
    Mongo.insert_many_raw(conn, coll, docs)
}
public fn update_one(conn: MongoConn, coll: String, filter: String, update: String) -> Result<Int, String> !MongoDB {
    Mongo.update_one_raw(conn, coll, filter, update)
}
public fn delete_one(conn: MongoConn, coll: String, filter: String) -> Result<Int, String> !MongoDB {
    Mongo.delete_one_raw(conn, coll, filter)
}
public fn aggregate(conn: MongoConn, coll: String, pipeline: String) -> Result<String, String> !MongoDB {
    Mongo.aggregate_raw(conn, coll, pipeline)
}
```

### Step 9 — examples/mongo_events_etl.fav

spec.md の Example をそのまま作成。

### Step 10 — site/content/docs/runes/mongodb.mdx

- タイトル: MongoDB Rune
- `!MongoDB` エフェクトの説明
- 全 8 関数の API リファレンス（シグネチャ・説明・例）
- filter / update / pipeline の JSON フォーマット説明
- `find_one` の `"not_found"` エラーパターン
- 接続毎確立のパフォーマンス注記

### Step 11 — CHANGELOG.md

`[v25.5.0]` エントリを先頭に追加（`[v25.4.0]` の前）:

```markdown
## [v25.5.0] — 2026-06-25

### Added
- mongodb Rune 実質化（「動く Rune」5 条件達成）
- `Mongo.connect` / `find` / `find_one` / `insert_one` / `insert_many` / `update_one` / `delete_one` / `aggregate`（8 関数）
- `Effect::MongoDB`（`!MongoDB` エフェクト）追加（11 ファイル更新）
- E0322「undeclared !MongoDB effect」エラーコード追加
- `examples/mongo_events_etl.fav`（イベント ETL デモ）
- `site/content/docs/runes/mongodb.mdx`（API ドキュメント）
- `mongodb = { version = "3", ... }` を native-only 依存に追加
```

### Step 12 — benchmarks/v25.5.0.json

```json
{
  "version": "25.5.0",
  "timestamp": "2026-06-25T00:00:00Z",
  "metrics": {
    "test_count": 2007,
    "compile_hello_ms": 12,
    "compile_etl_ms": 45
  }
}
```

### Step 13 — driver.rs: v255000_tests 7 件

```rust
mod v255000_tests {
    fn mongo_rune_has_connect_fn()          // mongodb.fav に "fn connect" を確認
    fn mongo_rune_has_read_fns()            // find / find_one / aggregate を確認
    fn mongo_rune_has_write_fns()           // insert_one / insert_many / update_one / delete_one を確認
    fn mongo_primitives_exist_in_vm()       // vm.rs に "Mongo.connect_raw" 等を確認
    fn mongo_events_etl_example_exists()    // examples/mongo_events_etl.fav を確認
    fn changelog_has_v25_5_0()              // CHANGELOG.md に v25.5.0 を確認
    fn effect_mongodb_and_e0322_exist()     // ast.rs に "MongoDB," + error_catalog.rs に "E0322" を確認
}
```

---

## 注意事項・既知リスク

| リスク | 対策 |
|---|---|
| `mongodb = "3"` の `sync` feature が存在しない場合 | `tokio::runtime::Builder::new_current_thread().build()?.block_on(async { ... })` で非同期を同期化する |
| `mongodb::bson::to_document` が `serde_json::Value` を正確に変換できない場合 | `mongodb::bson::Deserializer` 経由で変換する、または `bson::doc!` マクロを使う |
| ObjectId の JSON シリアライズ | `mongodb::bson::oid::ObjectId` → `to_hex()` で文字列化する |
| `Effect::MongoDB` の exhaustive match | `cargo build` で漏れを確認。v25.4.0 の実績では 11 ファイル更新が必要 |
| `mongodb` crate と `bson` crate のバージョン競合 | `mongodb = "3"` が `bson = "2"` を内包しているか確認。別途 `bson` を追加する必要がある場合は `bson = { version = "2", features = ["serde_with"] }` を追加 |

---

## ファイル変更一覧

| ファイル | 種別 | 変更内容 |
|---|---|---|
| `fav/Cargo.toml` | 更新 | version bump + mongodb crate 追加 |
| `fav/src/ast.rs` | 更新 | `Effect::MongoDB` 追加 |
| `fav/src/error_catalog.rs` | 更新 | E0322 追加 |
| `fav/src/fmt.rs` | 更新 | `Effect::MongoDB` 表示文字列 |
| `fav/src/lineage.rs` | 更新 | `Effect::MongoDB` 2 箇所追加 |
| `fav/src/emit_python.rs` | 更新 | `Effect::MongoDB` アーム |
| `fav/src/lint.rs` | 更新 | `Effect::MongoDB` アーム |
| `fav/src/middle/reachability.rs` | 更新 | `Effect::MongoDB` アーム |
| `fav/src/middle/ast_lower_checker.rs` | 更新 | `Effect::MongoDB` アーム |
| `fav/src/middle/checker.rs` | 更新 | `require_mongodb_effect` / builtin fns |
| `fav/src/frontend/parser.rs` | 更新 | `"MongoDB" => Effect::MongoDB` |
| `fav/src/driver.rs` | 更新 | `format_effects` / `effect_json_name` + v255000_tests |
| `fav/src/backend/vm.rs` | 更新 | Mongo.*_raw 8 件 + ヘルパー関数 |
| `runes/mongodb/mongodb.fav` | 更新 | 全面更新（type MongoConn + 8 関数） |
| `examples/mongo_events_etl.fav` | 新規 | イベント ETL デモ |
| `CHANGELOG.md` | 更新 | `[v25.5.0]` エントリ |
| `site/content/docs/runes/mongodb.mdx` | 新規 | API ドキュメント |
| `benchmarks/v25.5.0.json` | 新規 | test_count: 2007 |
