# v25.6.0 仕様書 — dynamodb Rune 実質化

## 概要

| 項目 | 内容 |
|---|---|
| バージョン | v25.6.0 |
| フェーズ | Rune Foundation（v25.1〜v26.0） |
| テーマ | dynamodb Rune の「動く Rune」5 条件達成 |
| 依存関係 | なし（aws_post / SigV4 は vm.rs に既存） |
| 目標テスト数 | 2014 件（+7 件 ≥ ロードマップ最小 5 件） |

---

## 背景と目的

v25.5.0 で mongodb Rune を実質化した。次は AWS ユーザーの KV / NoSQL の中心である DynamoDB を実質化する。

既存の `runes/dynamodb/dynamodb.fav` は v24.5.0 で追加されたスタブのみ（関数定義なし）。

vm.rs には `AWS.dynamo_get_item_raw` / `AWS.dynamo_put_item_raw` / `AWS.dynamo_delete_item_raw` /
`AWS.dynamo_query_raw` / `AWS.dynamo_scan_raw`（v4.11.0）と `AWS.dynamo_put_item_cond_raw`（v15.1.0）が存在する。
v25.6.0 ではこれらとは独立した `DynamoDB.*_raw` primitives を追加する
（`Effect::DynamoDB` チェック付き・JSON 文字列 I/O・`DynamoConn` ラッパー）。

---

## 「動く Rune」5 条件

| # | 条件 | 対象 |
|---|---|---|
| 1 | connect | `LOCALSTACK_ENDPOINT` 環境変数（例: `http://localhost:4566`）または AWS エンドポイント経由で接続確立 |
| 2 | read | `DynamoDB.get_item` / `DynamoDB.query` / `DynamoDB.scan` — JSON フィルタ文字列 |
| 3 | write | `DynamoDB.put_item` / `DynamoDB.delete_item` / `DynamoDB.batch_write` / `DynamoDB.transact_write` |
| 4 | error | `Result<T, String>` 統一、エラーメッセージにテーブル名を含む |
| 5 | test | `v256000_tests` 7 件 PASS + `examples/dynamodb_session_store.fav` E2E デモ |

---

## 既存実装の現状

| ファイル | 状態 | 備考 |
|---|---|---|
| `runes/dynamodb/dynamodb.fav` | スタブのみ（関数なし） | v24.5.0 で追加 |
| `Effect::DynamoDB` | **未定義** | v25.6.0 で追加（`ast.rs`） |
| `DynamoDB.*_raw` primitives | **なし** | v25.6.0 で追加（`vm.rs`） |
| `AWS.dynamo_*_raw` | 既存（v4.11.0） | `VMValue::Record` ベース。v25.6.0 では別途 JSON 文字列ベースの `DynamoDB.*_raw` を追加 |

---

## 機能仕様

### 型定義

```favnir
// 接続エンドポイント URL ラッパー型
// "" または "default" → AWS 本番エンドポイント（リージョンは AWS_DEFAULT_REGION 環境変数）
// "http://localhost:4566" → LocalStack
type DynamoConn(String)
```

### 追加関数一覧

| 関数 | シグネチャ | 内容 |
|---|---|---|
| `DynamoDB.connect` | `(endpoint: String) -> Result<DynamoConn, String> !DynamoDB` | エンドポイント確認（ListTables ping） |
| `DynamoDB.get_item` | `(conn: DynamoConn, table: String, key_json: String) -> Result<String, String> !DynamoDB` | GetItem（見つからない場合 `Result.err("not_found")`） |
| `DynamoDB.put_item` | `(conn: DynamoConn, table: String, item_json: String) -> Result<Unit, String> !DynamoDB` | PutItem |
| `DynamoDB.delete_item` | `(conn: DynamoConn, table: String, key_json: String) -> Result<Unit, String> !DynamoDB` | DeleteItem |
| `DynamoDB.query` | `(conn: DynamoConn, table: String, key_cond: String, attr_vals_json: String) -> Result<String, String> !DynamoDB` | Query（KeyConditionExpression + ExpressionAttributeValues） |
| `DynamoDB.scan` | `(conn: DynamoConn, table: String, filter_json: String) -> Result<String, String> !DynamoDB` | Scan（filter_json が `""` の場合は全件スキャン） |
| `DynamoDB.batch_write` | `(conn: DynamoConn, table: String, puts_json: String) -> Result<Int, String> !DynamoDB` | BatchWriteItem（最大 25 件、挿入件数を返す） |
| `DynamoDB.transact_write` | `(conn: DynamoConn, ops_json: String) -> Result<Unit, String> !DynamoDB` | TransactWriteItems |

> **JSON フォーマット**
> - `key_json`: `{"pk": "user123", "sk": "profile"}` — プレーンな JSON オブジェクト
> - `item_json`: `{"pk": "user123", "sk": "profile", "ttl": 1700000000, "data": "..."}` — プレーンな JSON オブジェクト
> - `attr_vals_json`: `{":pk": "user123", ":status": "active"}` — KeyConditionExpression の値
> - `puts_json`: `[{"pk": "user1", "data": "val"}, ...]` — JSON 配列
> - `ops_json`: TransactWriteItems の各操作を含む JSON 配列
>
> **戻り値**: `get_item` は JSON オブジェクト文字列、`query` / `scan` は JSON 配列文字列
> **型変換**: String → `{"S": "val"}`、Number → `{"N": "1"}`、Boolean → `{"BOOL": true}` の変換は vm.rs 内で自動処理

---

## エフェクト追加仕様（`!DynamoDB`）

v25.6.0 で `Effect::DynamoDB` を新たに追加する。

| ファイル | 変更内容 |
|---|---|
| `fav/src/ast.rs` | `Effect` enum に `DynamoDB` バリアント追加（`MongoDB` バリアントの直後） |
| `fav/src/middle/checker.rs` | `ns_to_inferred_effect` / `require_dynamodb_effect` / DynamoDB builtin fns 追加 |
| `fav/src/middle/reachability.rs` | `Effect::*` 網羅的 match に `DynamoDB` 追加 |
| `fav/src/middle/ast_lower_checker.rs` | `ast::Effect::*` 網羅的 match に `DynamoDB` 追加 |
| `fav/src/emit_python.rs` | `Effect::DynamoDB => "DynamoDB"` アームを追加 |
| `fav/src/lineage.rs` | `Effect::DynamoDB` のリネージ追跡追加（`format_effects` 1 箇所 + `classify_capability_kind`） |
| `fav/src/lint.rs` | `effect_to_str` 網羅的 match に `Effect::DynamoDB` 追加 |
| `fav/src/error_catalog.rs` | E0323「undeclared !DynamoDB effect」追加 |
| `fav/src/fmt.rs` | `Effect::DynamoDB => Some("!DynamoDB".to_string())` 追加 |
| `fav/src/frontend/parser.rs` | `"DynamoDB" => Effect::DynamoDB` アーム追加（`"MongoDB"` の後） |
| `fav/src/driver.rs` | `format_effects` / `effect_json_name` に `DynamoDB` アーム追加 |

> **注意**: `Effect::DynamoDB` 追加で更新が必要なファイルは合計 11 ファイル。
> `cargo build` で exhaustive match エラーを確認しながら進めること。

---

## DynamoDB クライアント実装方針

- 既存の `aws_post` / `get_aws_config` / SigV4 signing infrastructure（vm.rs に既存）を再利用
- `DynamoDB.*_raw` primitives は JSON 文字列 I/O（`AWS.dynamo_*_raw` は VMValue::Record I/O）
- エンドポイント決定ロジック:
  1. `DynamoConn` の文字列が空または `"default"` → `AWS_ENDPOINT_URL` → `LOCALSTACK_ENDPOINT` → AWS 本番
  2. それ以外 → `DynamoConn` の文字列をそのまま endpoint として使用
- `cfg(not(target_arch = "wasm32"))` ガードを全 DynamoDB primitive に付与

### JSON ↔ DynamoDB 属性変換ヘルパー

```rust
/// プレーン JSON Value → DynamoDB 属性 JSON 文字列
/// String  → {"S": "val"}
/// Number  → {"N": "1.0"}
/// Boolean → {"BOOL": true}
/// Null    → {"NULL": true}
/// Array   → {"L": [...]}
/// Object  → {"M": {...}}
#[cfg(not(target_arch = "wasm32"))]
fn json_val_to_dynamo_attr_str(v: &serde_json::Value) -> String { ... }

/// プレーン JSON Object → DynamoDB Item JSON 文字列
/// {"pk": "user1", "ttl": 1700} → {"pk":{"S":"user1"},"ttl":{"N":"1700"}}
#[cfg(not(target_arch = "wasm32"))]
fn json_to_dynamo_item_str(v: &serde_json::Value) -> Result<String, String> { ... }

/// DynamoDB Item JSON → プレーン JSON Value
/// {"pk": {"S": "user1"}, "ttl": {"N": "1700"}} → {"pk": "user1", "ttl": 1700}
#[cfg(not(target_arch = "wasm32"))]
fn dynamo_item_to_json_val(item: &serde_json::Value) -> serde_json::Value { ... }
```

### VM primitives 一覧（8 件）

| primitive 名 | 引数 | 戻り値 |
|---|---|---|
| `DynamoDB.connect_raw` | `endpoint: String` | `Result<String, String>`（DynamoConn ラッパー） |
| `DynamoDB.get_item_raw` | `endpoint: String, table: String, key_json: String` | `Result<String, String>`（JSON オブジェクト / `"not_found"`） |
| `DynamoDB.put_item_raw` | `endpoint: String, table: String, item_json: String` | `Result<Unit, String>` |
| `DynamoDB.delete_item_raw` | `endpoint: String, table: String, key_json: String` | `Result<Unit, String>` |
| `DynamoDB.query_raw` | `endpoint: String, table: String, key_cond: String, attr_vals_json: String` | `Result<String, String>`（JSON 配列文字列） |
| `DynamoDB.scan_raw` | `endpoint: String, table: String, filter_json: String` | `Result<String, String>`（JSON 配列文字列） |
| `DynamoDB.batch_write_raw` | `endpoint: String, table: String, puts_json: String` | `Result<Int, String>`（書き込み件数） |
| `DynamoDB.transact_write_raw` | `endpoint: String, ops_json: String` | `Result<Unit, String>` |

> **connect_raw の戻り型**（checker レベル）: `Result<String, String>`。
> `runes/dynamodb/dynamodb.fav` では `Result<DynamoConn, String>` として公開するが、
> `DynamoConn(String)` は名目型ラッパーであり checker は String として扱う
> （PgConn / RedisConn / MySqlConn / MongoConn と同じパターン — 意図的な簡略化）。

---

## エラーコード

| コード | 名前 | 説明 |
|---|---|---|
| E0323 | UndeclaredDynamoDBEffect | `!DynamoDB` エフェクトなしで DynamoDB 系 Rune を呼び出した場合 |

---

## `examples/dynamodb_session_store.fav`

```favnir
import rune "dynamodb"

// ── DynamoDB を使ったセッションストア デモ (v25.6.0) ─────────────────────────
// 前提: docker run -p 4566:4566 localstack/localstack
//       aws --endpoint-url=http://localhost:4566 dynamodb create-table \
//           --table-name sessions --attribute-definitions AttributeName=session_id,AttributeType=S \
//           --key-schema AttributeName=session_id,KeyType=HASH \
//           --billing-mode PAY_PER_REQUEST
// 実行: fav run examples/dynamodb_session_store.fav

stage StoreSession: String -> Result<Unit, String> !DynamoDB = |session_json| {
    bind conn <- DynamoDB.connect("http://localhost:4566")
    DynamoDB.put_item(conn, "sessions", session_json)
}

stage GetSession: String -> Result<String, String> !DynamoDB = |session_id| {
    bind conn <- DynamoDB.connect("http://localhost:4566")
    DynamoDB.get_item(conn, "sessions", "{\"session_id\": \"" + session_id + "\"}")
}

stage DeleteSession: String -> Result<Unit, String> !DynamoDB = |session_id| {
    bind conn <- DynamoDB.connect("http://localhost:4566")
    DynamoDB.delete_item(conn, "sessions", "{\"session_id\": \"" + session_id + "\"}")
}
```

---

## やらないこと（スコープ外）

- GSI（グローバルセカンダリインデックス）作成・管理
- DynamoDB Streams
- TTL 設定（`put_item` の item_json に `ttl` フィールドを含めることで間接的に対応可能）
- ConditionExpression（`AWS.dynamo_put_item_cond_raw` として既存。v25.6.0 では expose しない）
- ページネーション（`LastEvaluatedKey` の自動処理）
- DAX（DynamoDB Accelerator）接続
- 型付きジェネリクス（`DynamoDB.get_item[T]` / `DynamoDB.query[T]`）

> **ロードマップとの差分**: ロードマップ（roadmap-v25.1-v26.0.md）には型付きジェネリクスや GSI 対応が記載されているが、v25.6.0 では「動く Rune」5 条件の達成を最優先とし、これらを v26.x 以降に延期する。JSON 文字列 I/O で動作する基本 API を先に確立することで、データエンジニアがすぐに利用できる状態を優先する。

---

## 完了条件

| # | 条件 |
|---|---|
| 1 | `DynamoDB.connect` が `runes/dynamodb/dynamodb.fav` に実装済み |
| 2 | `DynamoDB.get_item` / `DynamoDB.query` / `DynamoDB.scan` が実装済み（read 系） |
| 3 | `DynamoDB.put_item` / `DynamoDB.delete_item` / `DynamoDB.batch_write` / `DynamoDB.transact_write` が実装済み（write 系） |
| 4 | `DynamoDB.*_raw` VM primitives（8 件）が `fav/src/backend/vm.rs` に存在する |
| 5 | `Effect::DynamoDB` が `fav/src/ast.rs` に存在し、E0323 が `error_catalog.rs` に存在する（`cargo build` で exhaustive match 確認済み） |
| 6 | `examples/dynamodb_session_store.fav` が存在し `import rune "dynamodb"` + `put_item` + `get_item` + `delete_item` を含む |
| 7 | `CHANGELOG.md` に `[v25.6.0]` エントリが存在する |
| 8 | `site/content/docs/runes/dynamodb.mdx` に新規 API が記載済み |
| 9 | `cargo test v256000` で 7 件すべて PASS |
| 10 | 総テスト数 ≥ 2014 件 |

---

## 設計判断

### `DynamoDB.*_raw` vs `AWS.dynamo_*_raw` の共存

既存の `AWS.dynamo_*_raw` primitives（v4.11.0）は `VMValue::Record` ベースの I/O を使用する。
v25.6.0 で追加する `DynamoDB.*_raw` は JSON 文字列 I/O を使用する（MongoDB パターン）。
両者は独立して共存する（既存の AWS.* primitives は変更しない）。

### get_item の "not_found" パターン

DynamoDB の `GetItem` は存在しないキーに対して `Item` フィールドのない成功レスポンスを返す。
Favnir では `Result.err("not_found")` として統一する（MongoDB の `find_one` と同パターン）。

### DynamoConn(String) の checker 互換性

`connect_raw` は `Result<String, String>` を返す。`runes/dynamodb/dynamodb.fav` では `Result<DynamoConn, String>` として公開するが、`DynamoConn(String)` は名目型ラッパーであり checker 内で `String` として扱われるため `fav check` はエラーにならない（v25.5.0 の MongoConn / v10.6.0 の PgConn と同パターン — 意図的な簡略化）。

### connect の毎回 ping 設計

`DynamoDB.connect` は呼び出しのたびに `ListTables` ping を実行する。stage 内で毎回 `connect` を呼ぶと合計リクエスト数が 2 倍になる。これは v25.5.0 の MongoDB と同じ設計制約であり、コネクションプールは v26.x で対応予定（`vm.rs` の primitives ブロック先頭に TODO コメントを追記する）。

### transact_write の ops_json フォーマット

`transact_write_raw` の `ops_json` は **DynamoDB ネイティブ形式**（属性型 JSON）で渡す必要がある。他の操作（`put_item`、`get_item` 等）はプレーン JSON を自動変換するが、`transact_write` はトランザクション内の各操作が異なる DynamoDB Action（`Put` / `Delete` / `Update` / `ConditionCheck`）を混在させるため、ユーザーが DynamoDB TransactItems 形式の JSON を直接構築して渡す。

例:
```json
[
  {"Put": {"TableName": "sessions", "Item": {"session_id": {"S": "abc"}, "ttl": {"N": "1700"}}}},
  {"Delete": {"TableName": "sessions", "Key": {"session_id": {"S": "old"}}}}
]
```

---

## 検証コマンド

```bash
cd fav && cargo test v256000 -- --test-threads=1
cd fav && cargo test -j 8 -- --test-threads=8 2>&1 | tail -5
```
