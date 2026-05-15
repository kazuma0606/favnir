# Favnir v3.7.0 Language Specification

## Theme: `http` + `parquet` rune — REST / GraphQL / DWH 出力

"読んで変換してAPIとして公開する" を完成させ、レガシーシステムの非破壊モダン化を実現する。

---

## 1. `!Network` エフェクト

HTTP の送受信を行う関数に付与するエフェクト。
（実装名: `Effect::Network` / キーワード: `!Network`）

```favnir
public fn main() -> Unit !Io !Network {
    bind resp <- Http.get("https://api.example.com/users")
    IO.println(resp.body)
}
```

---

## 2. 組み込み型

### `HttpResponse`

```favnir
type HttpResponse = {
    status:  Int     // HTTP ステータスコード（200, 404, 500 ...）
    body:    String  // レスポンスボディ（テキスト）
    content_type: String  // Content-Type ヘッダー値
}
```

### `HttpError`

```favnir
type HttpError = {
    code:    Int     // エラー種別（0=接続失敗, 1=タイムアウト, 2=ステータスエラー）
    message: String
    status:  Int     // HTTP ステータス（ステータスエラー時のみ有効、それ以外は 0）
}
```

### `ParquetError`

```favnir
type ParquetError = {
    message: String
}
```

---

## 3. `Http` VM プリミティブ

### `Http.get_raw(url: String) -> Result<HttpResponse, HttpError> !Network`

同期 HTTP GET。レスポンスを `HttpResponse` として返す。

```favnir
bind result <- Http.get_raw("https://api.example.com/products")
match result {
    Ok(resp) => IO.println(resp.body)
    Err(e)   => IO.println($"Error: {e.message}")
}
```

### `Http.post_raw(url: String, body: String, content_type: String) -> Result<HttpResponse, HttpError> !Network`

同期 HTTP POST。`content_type` は `"application/json"` 等。

```favnir
bind result <- Http.post_raw(
    "https://api.example.com/ingest",
    "{\"id\": 1}",
    "application/json"
)
```

### `Http.serve_raw(port: Int, routes: List<Map<String, String>>, handler_name: String) -> Unit !Network !Io`

指定ポートで HTTP サーバーを起動（ブロッキング）。
`routes` は `[{ "method": "GET", "path": "/api/v1/products" }]` の形式。
`handler_name` は登録済み Favnir 関数名（文字列）。

実用的な公開 API は `http` rune の `serve` / `serve_graphql` を使う。

---

## 4. `Parquet` VM プリミティブ

### `Parquet.write_raw(path: String, type_name: String, rows: List<Map<String, String>>) -> Result<Unit, ParquetError>`

型名のスキーマで `rows` を Parquet ファイルに書き出す。
フィールドの型は `type_name` で登録されたレコード型から決定。

```favnir
type Product = { id: Int name: String price: Float }

bind rows <- Gen.list_raw("Product", 1000)
bind result <- Parquet.write_raw("output/products.parquet", "Product", rows)
```

### `Parquet.read_raw(path: String) -> Result<List<Map<String, String>>, ParquetError>`

Parquet ファイルを `List<Map<String, String>>` として読み込む。

```favnir
bind result <- Parquet.read_raw("data/snapshot.parquet")
match result {
    Ok(rows) => IO.println($"Loaded {List.length(rows)} rows")
    Err(e)   => IO.println($"Error: {e.message}")
}
```

---

## 5. `runes/http/http.fav` 公開 API

| 関数 | シグネチャ | 説明 |
|------|-----------|------|
| `http.get` | `String -> Result<HttpResponse, HttpError> !Network` | HTTP GET |
| `http.post` | `(String, String) -> Result<HttpResponse, HttpError> !Network` | HTTP POST（body: String） |
| `http.post_json` | `(String, String) -> Result<HttpResponse, HttpError> !Network` | POST with Content-Type: application/json |
| `http.get_body` | `String -> Result<String, HttpError> !Network` | GET してボディ文字列だけ返す |
| `http.ok` | `(Int, String) -> HttpResponse` | レスポンス生成ヘルパー |
| `http.error_response` | `(Int, String) -> HttpResponse` | エラーレスポンス生成ヘルパー |

```favnir
import rune "http"

public fn main() -> Unit !Io !Network {
    // シンプルな GET
    bind body <- http.get_body("https://httpbin.org/get")
    match body {
        Ok(text) => IO.println($"Got: {String.length(text)} bytes")
        Err(e)   => IO.println($"Failed: {e.message}")
    }

    // JSON POST
    bind resp <- http.post_json("https://httpbin.org/post", "{\"key\": \"value\"}")
    match resp {
        Ok(r) => IO.println($"Status: {r.status}")
        Err(e) => IO.println($"Error: {e.message}")
    }
}
```

---

## 6. `runes/parquet/parquet.fav` 公開 API

| 関数 | シグネチャ | 説明 |
|------|-----------|------|
| `parquet.write` | `(String, String, List<Map<String,String>>) -> Result<Unit, ParquetError>` | Parquet 書き込み |
| `parquet.read` | `String -> Result<List<Map<String,String>>, ParquetError>` | Parquet 読み込み |
| `parquet.append` | `(String, String, List<Map<String,String>>) -> Result<Unit, ParquetError>` | 既存ファイルに追記 |
| `parquet.row_count` | `String -> Result<Int, ParquetError>` | 行数だけ返す（データ読み込みなし） |

```favnir
import rune "parquet"

type SensorReading = { device_id: Int ts: String value: Float unit: String }

public fn main() -> Unit !Io {
    // 書き込み
    bind rows <- Gen.list_raw("SensorReading", 10000)
    bind write_result <- parquet.write("output/readings.parquet", "SensorReading", rows)
    match write_result {
        Ok(_)  => IO.println("Written successfully")
        Err(e) => IO.println($"Write error: {e.message}")
    }

    // 読み込み
    bind read_result <- parquet.read("output/readings.parquet")
    match read_result {
        Ok(loaded) => IO.println($"Read {List.length(loaded)} rows")
        Err(e)     => IO.println($"Read error: {e.message}")
    }
}
```

---

## 7. `fav build --graphql` — SDL ファイル生成

`fav build` コマンドの `--graphql` フラグとして実装。

```bash
fav build --graphql src/main.fav --out schema.graphql
fav build --graphql src/main.fav          # stdout に出力
```

Favnir の `type` 定義と `interface` 定義から GraphQL SDL を生成する。
実行は不要。AST を静的に走査して SDL を出力する。

### 型マッピング

| Favnir 型 | GraphQL 型 |
|----------|-----------|
| `Int` | `Int` |
| `Float` | `Float` |
| `String` | `String` |
| `Bool` | `Boolean` |
| `Option<T>` | `T`（nullable） |
| `List<T>` | `[T!]!` |
| `Result<T, E>` | `T`（エラー時は GraphQL errors フィールドへ） |

### interface → Query/Mutation

```favnir
// src/main.fav
type User = { id: Int  name: String  email: String }

interface UserQuerySchema {
    user:  Int  -> Result<User, HttpError>
    users: Unit -> Result<List<User>, HttpError>
}
```

生成される `schema.graphql`:
```graphql
type User {
    id:    Int!
    name:  String!
    email: String!
}

type Query {
    user(id: Int!): User
    users: [User!]!
}
```

---

## 8. 典型ワークフロー

```bash
# Step 1: 外部 API からデータ取得・変換・Parquet 保存
fav run etl.fav

# Step 2: Parquet を読み込んで型定義を推論
fav infer output/result.parquet --out schema/row.fav

# Step 3: GraphQL スキーマを生成
fav build --graphql src/api.fav --out schema.graphql

# Step 4: レガシーシステムを API として公開
fav run serve.fav
```

---

## Breaking Changes

v3.6.0 との破壊的変更なし。

---

## 新規 Cargo 依存

| クレート | 用途 |
|---------|------|
| `ureq = "2"` | HTTP クライアント（軽量、同期） |
| `tiny_http = "0.12"` | HTTP サーバー（シンプル、ブロッキング） |
| `parquet = "52"` | Apache Parquet 読み書き |
| `arrow = { version = "52", features = ["ipc"] }` | Arrow データフォーマット（Parquet 依存） |
