# Favnir v3.7.0 Language Specification

## New in v3.7.0: `http` + `parquet` rune

---

## 1. `!Network` エフェクト

HTTP 送受信を行う関数に付与するエフェクト。

```favnir
public fn fetch_users() -> Result<HttpResponse, HttpError> !Network {
    Http.get_raw("https://api.example.com/users")
}
```

`Http.*` を呼び出す関数・ステージには `!Network` が必要。
付与しない場合は型チェッカーがエラーを報告する。

---

## 2. 組み込み型

### `HttpResponse`

```favnir
type HttpResponse = {
    status:       Int     // HTTP ステータスコード
    body:         String  // レスポンスボディ（テキスト）
    content_type: String  // Content-Type ヘッダー値
}
```

### `HttpError`

```favnir
type HttpError = {
    code:    Int     // 0=接続失敗, 1=タイムアウト, 2=HTTPステータスエラー
    message: String
    status:  Int     // HTTPエラー時のステータスコード（それ以外は 0）
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

同期 HTTP GET（`ureq` クレート使用）。

```favnir
bind result <- Http.get_raw("https://httpbin.org/get")
match result {
    Ok(resp) => IO.println($"Status: {resp.status}")
    Err(e)   => IO.println($"Error: {e.message}")
}
```

### `Http.post_raw(url: String, body: String, content_type: String) -> Result<HttpResponse, HttpError> !Network`

同期 HTTP POST。

```favnir
bind result <- Http.post_raw(
    "https://api.example.com/data",
    "{\"key\": \"value\"}",
    "application/json"
)
```

### `Http.serve_raw(port: Int, routes: List<Map<String,String>>, handler_name: String) -> Unit !Network !Io`

指定ポートで HTTP サーバーを起動（`tiny_http` ブロッキングループ）。
`handler_name` に登録済み Favnir 関数名を渡す。ハンドラは `HttpResponse` を返す関数。

---

## 4. `Parquet` VM プリミティブ

### `Parquet.write_raw(path: String, type_name: String, rows: List<Map<String,String>>) -> Result<Unit, ParquetError>`

型スキーマ（`type_metas`）に基づいて Parquet ファイルを書き出す。
`type_name` は同一ソースファイルに `type T = { ... }` で宣言されている必要がある。

```favnir
type Row = { id: Int name: String }

bind result <- Parquet.write_raw("output/rows.parquet", "Row", rows)
```

### `Parquet.read_raw(path: String) -> Result<List<Map<String,String>>, ParquetError>`

Parquet ファイルを読み込む。全フィールドを文字列として返す。

---

## 5. `runes/http/http.fav` 公開 API

| 関数 | シグネチャ | 説明 |
|------|-----------|------|
| `http.get` | `String -> Result<HttpResponse, HttpError> !Network` | HTTP GET |
| `http.post` | `(String, String) -> Result<HttpResponse, HttpError> !Network` | HTTP POST（text/plain） |
| `http.post_json` | `(String, String) -> Result<HttpResponse, HttpError> !Network` | HTTP POST（application/json） |
| `http.get_body` | `String -> Result<String, HttpError> !Network` | GET してボディ文字列だけ返す |
| `http.ok` | `(Int, String) -> HttpResponse` | レスポンス生成ヘルパー（純粋） |
| `http.error_response` | `(Int, String) -> HttpResponse` | エラーレスポンス生成ヘルパー（純粋） |

```favnir
import rune "http"

public fn main() -> Unit !Io !Network {
    bind result <- http.get_body("https://httpbin.org/get")
    match result {
        Ok(body) => IO.println($"Got {String.length(body)} bytes")
        Err(e)   => IO.println($"Failed: {e.message}")
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
| `parquet.row_count` | `String -> Result<Int, ParquetError>` | 行数取得 |

```favnir
import rune "parquet"

type Product = { id: Int name: String price: Float }

public fn main() -> Unit !Io {
    bind rows <- Gen.list_raw("Product", 1000)
    bind write_result <- parquet.write("products.parquet", "Product", rows)
    match write_result {
        Ok(_) => {
            bind count_result <- parquet.row_count("products.parquet")
            IO.println($"Written: {Result.unwrap_or(count_result, 0)} rows")
        }
        Err(e) => IO.println($"Error: {e.message}")
    }
}
```

---

## 7. `fav build --graphql` — SDL 生成

`fav build` コマンドの `--graphql` フラグ。実行せず AST を静的走査して SDL を生成する。

```bash
fav build --graphql src/main.fav --out schema.graphql
fav build --graphql src/main.fav          # stdout に出力
```

### 型マッピング

| Favnir 型 | GraphQL 型 |
|----------|-----------|
| `Int` | `Int!` |
| `Float` | `Float!` |
| `String` | `String!` |
| `Bool` | `Boolean!` |
| `Option<T>` | `T`（nullable） |
| `List<T>` | `[T!]!` |
| `Result<T, E>` | `T`（nullable; エラーは GraphQL errors へ） |

### interface → Query

```favnir
type User = { id: Int  name: String }

interface UserQuerySchema {
    user:  Int  -> Result<User, HttpError>
    users: Unit -> Result<List<User>, HttpError>
}
```

生成 SDL:
```graphql
type User {
    id: Int!
    name: String!
}

type Query {
    user(arg1: Int!): User
    users: [User!]!
}
```

---

## 8. 典型ワークフロー

```bash
# 外部 API → 変換 → Parquet 保存
fav run etl.fav

# 型定義から GraphQL スキーマ生成
fav build --graphql src/api.fav --out schema.graphql

# レガシーデータを API として公開
fav run serve.fav
```
