# v25.5.0 仕様書 — mongodb Rune 実質化

## 概要

| 項目 | 内容 |
|---|---|
| バージョン | v25.5.0 |
| フェーズ | Rune Foundation（v25.1〜v26.0） |
| テーマ | mongodb Rune の「動く Rune」5 条件達成 |
| 依存関係 | なし（postgres / mysql と独立した API 体系） |
| 目標テスト数 | 2007 件（+7 件 ≥ ロードマップ最小 5 件） |

---

## 背景と目的

v25.4.0 で mysql Rune を実質化した。次はドキュメント系 NoSQL の代表 MongoDB を実質化する。
JSON / BSON との親和性から、イベントログ・ユーザープロファイル・半構造データに多用される。

既存の `runes/mongodb/mongodb.fav` は v24.5.0 で追加されたスタブのみ（関数定義なし）。

Postgres / MySQL とは独立した API 体系（SQL ではなく JSON フィルタ / パイプライン）を採用する。

---

## 「動く Rune」5 条件

| # | 条件 | 対象 |
|---|---|---|
| 1 | connect | `MONGODB_URL` 環境変数（例: `mongodb://localhost:27017/mydb`）経由で接続確立 |
| 2 | read | `Mongo.find` / `Mongo.find_one` / `Mongo.aggregate` — JSON フィルタ・パイプライン文字列 |
| 3 | write | `Mongo.insert_one` / `Mongo.insert_many` / `Mongo.update_one` / `Mongo.delete_one` |
| 4 | error | `Result<T, String>` 統一、エラーメッセージにコレクション名を含む |
| 5 | test | `v255000_tests` 7 件 PASS + `examples/mongo_events_etl.fav` E2E デモ |

---

## 既存実装の現状

| ファイル | 状態 | 備考 |
|---|---|---|
| `runes/mongodb/mongodb.fav` | スタブのみ（関数なし） | v24.5.0 で追加 |
| `Effect::MongoDB` | **未定義** | v25.5.0 で追加（`ast.rs`） |
| `Mongo.*_raw` primitives | **なし** | v25.5.0 で追加（`vm.rs`） |
| `mongodb` crate | **未追加** | v25.5.0 で `Cargo.toml` に追加 |

---

## 機能仕様

### 型定義

```favnir
// 接続 URL ラッパー型（"mongodb://user:pass@host:port/db" 形式）
// データベース名は URL パスで指定（例: mongodb://localhost:27017/mydb → db = "mydb"）
type MongoConn(String)
```

### 追加関数一覧

| 関数 | シグネチャ | 内容 |
|---|---|---|
| `Mongo.connect` | `(url: String) -> Result<MongoConn, String> !MongoDB` | 接続確立（ping 確認） |
| `Mongo.find` | `(conn: MongoConn, coll: String, filter: String) -> Result<String, String> !MongoDB` | フィルタに一致する全ドキュメントを JSON 配列文字列で返す |
| `Mongo.find_one` | `(conn: MongoConn, coll: String, filter: String) -> Result<String, String> !MongoDB` | 1 件取得（見つからない場合 `Result.err("not_found")`） |
| `Mongo.insert_one` | `(conn: MongoConn, coll: String, doc: String) -> Result<String, String> !MongoDB` | ドキュメント挿入（挿入 ID を文字列で返す） |
| `Mongo.insert_many` | `(conn: MongoConn, coll: String, docs: String) -> Result<Int, String> !MongoDB` | バッチ挿入（挿入件数を返す） |
| `Mongo.update_one` | `(conn: MongoConn, coll: String, filter: String, update: String) -> Result<Int, String> !MongoDB` | 更新（`$set` / `$inc` 等演算子対応、更新件数を返す） |
| `Mongo.delete_one` | `(conn: MongoConn, coll: String, filter: String) -> Result<Int, String> !MongoDB` | 削除（削除件数を返す） |
| `Mongo.aggregate` | `(conn: MongoConn, coll: String, pipeline: String) -> Result<String, String> !MongoDB` | 集計パイプライン（`$match / $group / $sort`）、JSON 配列文字列で返す |

> **filter / update / pipeline / doc / docs 形式**: すべて JSON 文字列。
> - filter 例: `"{\"status\": \"active\"}"`
> - update 例: `"{\"$set\": {\"status\": \"processed\"}}"`
> - pipeline 例: `"[{\"$match\": {\"status\": \"active\"}}, {\"$group\": {\"_id\": \"$type\", \"count\": {\"$sum\": 1}}}]"`
> - docs 例: `"[{\"name\": \"alice\"}, {\"name\": \"bob\"}]"`
>
> **find / find_one の戻り値**: `_id` は `{"$oid": "..."}` 形式で JSON にシリアライズされる。

---

## エフェクト追加仕様（`!MongoDB`）

v25.5.0 で `Effect::MongoDB` を新たに追加する。

| ファイル | 変更内容 |
|---|---|
| `fav/src/ast.rs` | `Effect` enum に `MongoDB` バリアント追加（`MySQL` バリアントの直後） |
| `fav/src/middle/checker.rs` | `ns_to_inferred_effect` / `require_mongodb_effect` / Mongo builtin fns 追加 |
| `fav/src/middle/reachability.rs` | `Effect::*` 網羅的 match に `MongoDB` 追加 |
| `fav/src/middle/ast_lower_checker.rs` | `ast::Effect::*` 網羅的 match に `MongoDB` 追加 |
| `fav/src/emit_python.rs` | `Effect::MongoDB => "MongoDB"` アームを追加 |
| `fav/src/lineage.rs` | `Effect::MongoDB` のリネージ追跡追加（`format_effects` の 2 箇所 + `classify_capability_kind`） |
| `fav/src/lint.rs` | `effect_to_str` 網羅的 match に `Effect::MongoDB` 追加 |
| `fav/src/error_catalog.rs` | E0322「undeclared !MongoDB effect」追加 |
| `fav/src/fmt.rs` | `Effect::MongoDB => Some("!MongoDB".to_string())` 追加 |
| `fav/src/frontend/parser.rs` | `"MongoDB" => Effect::MongoDB` アーム追加（`"MySQL"` の後） |
| `fav/src/driver.rs` | `format_effects` / `effect_json_name` に `MongoDB` アーム追加 |

> **注意**: `Effect::MongoDB` 追加で更新が必要なファイルは合計 11 ファイル。
> `cargo build` で exhaustive match エラーを確認しながら進めること。

---

## MongoDB クライアント実装方針

- `mongodb = { version = "3", default-features = false, features = ["tokio-runtime"] }` を
  `[target.'cfg(not(target_arch = "wasm32"))'.dependencies]` に追加
  （`mongodb = "3"` の `sync` feature は v3 で廃止済み。**tokio-runtime が主方針**。）
- vm.rs では `tokio::runtime::Builder::new_current_thread().enable_all().build()?.block_on(async { ... })` で同期化
  （`tokio = { version = "1", features = ["full"] }` は既に `Cargo.toml` に存在）
- `MONGODB_URL` 環境変数を優先、未設定時は `mongodb://127.0.0.1:27017/test` をフォールバック
- `connect_raw` は URL を `MongoConn` にラップし、`Client::with_uri_str(url)` + ping で接続確認
- データベース名は URL パスから自動抽出（`mongodb://host/dbname` の `dbname` 部分）
- `cfg(not(target_arch = "wasm32"))` ガードを全 Mongo primitive に付与

### URL からデータベース名の抽出

```rust
// URL: mongodb://localhost:27017/mydb → db_name = "mydb"
// URL: mongodb://localhost:27017 → db_name = "test"（デフォルト、ポート番号は除外）
fn extract_mongo_db_name(url: &str) -> String {
    let after_scheme = url
        .strip_prefix("mongodb://")
        .or_else(|| url.strip_prefix("mongodb+srv://"))
        .unwrap_or(url);
    // "host:port/dbname" → dbname
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

> **設計判断: `find_one` の戻り型**
> ロードマップでは `Option<T>` 返却と記述されていたが、Favnir VM の型システムが
> `Option<T>` を直接サポートしないため `Result<String, String>` に統一した。
> 未発見時は `Result.err("not_found")` を返す（PgConn / RedisConn / MySqlConn と同パターン）。

### VM primitives 一覧（8 件）

| primitive 名 | 引数 | 戻り値 |
|---|---|---|
| `Mongo.connect_raw` | `url: String` | `Result<String, String>`（MongoConn ラッパー） |
| `Mongo.find_raw` | `url: String, coll: String, filter_json: String` | `Result<String, String>`（JSON 配列文字列） |
| `Mongo.find_one_raw` | `url: String, coll: String, filter_json: String` | `Result<String, String>`（JSON オブジェクト文字列 / `"not_found"`） |
| `Mongo.insert_one_raw` | `url: String, coll: String, doc_json: String` | `Result<String, String>`（挿入 ID 文字列） |
| `Mongo.insert_many_raw` | `url: String, coll: String, docs_json: String` | `Result<Int, String>`（挿入件数） |
| `Mongo.update_one_raw` | `url: String, coll: String, filter_json: String, update_json: String` | `Result<Int, String>`（更新件数） |
| `Mongo.delete_one_raw` | `url: String, coll: String, filter_json: String` | `Result<Int, String>`（削除件数） |
| `Mongo.aggregate_raw` | `url: String, coll: String, pipeline_json: String` | `Result<String, String>`（JSON 配列文字列） |

> **connect_raw の戻り型**（checker レベル）: `Result<String, String>`。
> `runes/mongodb/mongodb.fav` では `Result<MongoConn, String>` として公開するが、
> `MongoConn(String)` は名目型ラッパーであり checker は String として扱う
> （PgConn / RedisConn / MySqlConn と同じパターン — 意図的な簡略化）。

---

## エラーコード

| コード | 名前 | 説明 |
|---|---|---|
| E0322 | UndeclaredMongoDBEffect | `!MongoDB` エフェクトなしで Mongo 系 Rune を呼び出した場合 |

---

## `examples/mongo_events_etl.fav`

```favnir
import rune "mongodb"

// ── MongoDB を使ったイベント ETL デモ (v25.5.0) ──────────────────────────────
// 前提: docker run -p 27017:27017 mongo:7
// 実行: fav run examples/mongo_events_etl.fav

stage LoadActiveEvents: Unit -> Result<String, String> !MongoDB = |_| {
    bind conn <- Mongo.connect("mongodb://localhost:27017/analytics")
    Mongo.find(conn, "events", "{\"status\": \"active\"}")
}

stage ArchiveEvent: Result<String, String> -> Result<Int, String> !MongoDB = |event_result| {
    bind event_json <- event_result
    bind conn       <- Mongo.connect("mongodb://localhost:27017/analytics")
    bind _          <- Mongo.insert_one(conn, "archive", event_json)
    Mongo.delete_one(conn, "events", "{\"status\": \"active\"}")
}

pipeline EventsETL = LoadActiveEvents |> ArchiveEvent
```

---

## やらないこと（スコープ外）

- トランザクション（`ClientSession` / `with_transaction`）
- インデックス操作（`create_index` / `drop_index`）
- コレクション管理（`create_collection` / `drop_collection`）
- `find[T]` ジェネリクス型推論（JSON 文字列返却のみ、デシリアライズは呼び出し元）
- TLS 接続（`mongodb+srv://` スキーム）
- Atlas Search / ベクトル検索

---

## 完了条件

| # | 条件 |
|---|---|
| 1 | `Mongo.connect` が `runes/mongodb/mongodb.fav` に実装済み |
| 2 | `Mongo.find` / `Mongo.find_one` / `Mongo.aggregate` が実装済み（read 系） |
| 3 | `Mongo.insert_one` / `Mongo.insert_many` / `Mongo.update_one` / `Mongo.delete_one` が実装済み（write 系） |
| 4 | `Mongo.*_raw` VM primitives（8 件）が `fav/src/backend/vm.rs` に存在する |
| 5 | `Effect::MongoDB` が `fav/src/ast.rs` に存在し、E0322 が `error_catalog.rs` に存在する（`cargo build` で exhaustive match 確認済み） |
| 6 | `examples/mongo_events_etl.fav` が存在し `import rune "mongodb"` + `find` + `insert_one` + `delete_one` を含む |
| 7 | `CHANGELOG.md` に `[v25.5.0]` エントリが存在する |
| 8 | `site/content/docs/runes/mongodb.mdx` に新規 API が記載済み |
| 9 | `cargo test v255000` で 7 件すべて PASS |
| 10 | 総テスト数 ≥ 2007 件 |

---

## 検証コマンド

```bash
cd fav && cargo test v255000 -- --test-threads=1
cd fav && cargo test -j 8 -- --test-threads=8 2>&1 | tail -5
```
