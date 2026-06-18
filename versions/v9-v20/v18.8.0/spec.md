# v18.8.0 Spec — 型駆動 API 生成（`fav generate api` / `fav serve`）

Date: 2026-06-16

## 概要

型定義から REST（OpenAPI 3.0）と GraphQL API スキーマを自動生成する。
`#[api(...)]` アノテーションを `fn` に付けるだけで、手書き OpenAPI の手間を排除する。

データパイプラインに HTTP エンドポイントを追加する際、Favnir の型情報をそのままスキーマに変換することで「型チェッカーがそのまま API ドキュメントになる」状態を実現する。

---

## 構文

### `#[api(...)]` アノテーション

```favnir
#[api(method = "GET", path = "/users/:id")]
fn get_user(id: Int) -> Result<User, String> !Db {
  bind rows <- Postgres.query_raw(f"SELECT * FROM users WHERE id = {id}", [])
  match rows {
    [user] => Result.ok(user)
    _      => Result.err("not found")
  }
}

#[api(method = "POST", path = "/orders")]
fn create_order(req: CreateOrderRequest) -> Result<Order, String> !Db {
  bind id <- Postgres.execute_raw("INSERT INTO orders ...", [])
  Result.ok({ id: id, amount: req.amount })
}

#[api(method = "GET", path = "/pipeline/status")]
fn get_pipeline_status() -> Result<PipelineStatus, String> {
  Result.ok({ running: true, last_run: "2026-01-01" })
}
```

- `method`: HTTP メソッド（"GET" / "POST" / "PUT" / "DELETE" / "PATCH"）
- `path`: URL パス（`:param` 形式でパスパラメータを宣言）
- `#[api(...)]` のない `fn` は API に含まれない

---

## CLI コマンド

### `fav generate api`

```bash
# OpenAPI 3.0 YAML 生成
fav generate api --format openapi src/api.fav --out api.yaml

# OpenAPI 3.0 JSON 生成
fav generate api --format openapi --json src/api.fav --out api.json

# GraphQL SDL 生成
fav generate api --format graphql src/api.fav --out schema.graphql

# 型チェック込みで生成（スキーマ型との整合性確認）
fav generate api --format openapi --check-schemas src/api.fav --out api.yaml
```

### `fav serve`

```bash
# 開発用 HTTP サーバー起動（デフォルト: ポート 8080）
fav serve src/api.fav --port 8080

# ランダムポートで起動（テスト用）
fav serve src/api.fav --port 0
```

---

## 生成される OpenAPI の例

入力:

```favnir
type User = { id: Int, name: String, email: String }

#[api(method = "GET", path = "/users/:id")]
fn get_user(id: Int) -> Result<User, String> !Db { ... }
```

出力（`api.yaml`）:

```yaml
openapi: "3.0.0"
info:
  title: "Favnir API"
  version: "1.0.0"
paths:
  /users/{id}:
    get:
      parameters:
        - name: id
          in: path
          required: true
          schema:
            type: integer
      responses:
        "200":
          description: "Success"
          content:
            application/json:
              schema:
                $ref: "#/components/schemas/User"
        "400":
          description: "Error"
          content:
            application/json:
              schema:
                type: object
                properties:
                  error:
                    type: string
components:
  schemas:
    User:
      type: object
      properties:
        id:
          type: integer
        name:
          type: string
        email:
          type: string
```

---

## 生成される GraphQL SDL の例

```graphql
type User {
  id: Int!
  name: String!
  email: String!
}

type Query {
  get_user(id: Int!): User
}
```

---

## 型マッピング

### Favnir → OpenAPI

| Favnir 型 | OpenAPI type/format |
|---|---|
| `Int` | `{ type: integer }` |
| `Float` | `{ type: number }` |
| `String` | `{ type: string }` |
| `Bool` | `{ type: boolean }` |
| `List<T>` | `{ type: array, items: <T のスキーマ> }` |
| レコード型 | `{ type: object, properties: { ... } }` |
| `Result<T, E>` | 200: `T` のスキーマ、400: `{ error: string }` |
| 名前付き型（`User` 等） | `$ref: "#/components/schemas/User"` |

### Favnir → GraphQL SDL

既存の `graphql_type_from_type_expr_nonnull` ロジックを再利用。

---

## `fav serve` の動作

`tiny_http` を使った軽量 HTTP サーバー（開発用途のみ）:

1. ソースをコンパイルして `#[api]` アノテーション付き関数を収集
2. 各エンドポイントをルートテーブルに登録
3. HTTP リクエストを受信 → ルートマッチ → パラメータ抽出 → VM で関数実行 → JSON レスポンス

パスパラメータ（`:id`）は `String` として抽出し、関数の引数型に合わせてキャスト。

---

## AST 変更

### `ApiAnnotation` 構造体（新規）

```rust
// ast.rs に追加
#[derive(Debug, Clone)]
pub struct ApiAnnotation {
    pub method: String,  // "GET", "POST", etc.
    pub path: String,    // "/users/:id" etc.
    pub span: Span,
}
```

### `FnDef` への `api_annotation` フィールド追加

```rust
pub struct FnDef {
    // ... 既存フィールド ...
    pub api_annotation: Option<ApiAnnotation>,  // v18.8.0
}
```

---

## Parser 変更

### `parse_api_annotation` メソッド（新規）

`fn` キーワードの直前に `#[api(method = "...", path = "...")]` が来る場合にパース:

```
'#' '[' 'api' '(' 'method' '=' Str(m) ',' 'path' '=' Str(p) ')' ']'
```

- `parse_item` の冒頭で `#[api(...)` を検出
- `FnDef` に `ApiAnnotation` を付与
- `fn` 以外のアイテムへの `#[api]` は警告（無視）

---

## Checker 変更

チェッカーには変更なし（`api_annotation` は型チェック対象外）。
ただし将来的には `--check-schemas` で API 型とスキーマ型の整合性を確認する。

---

## テスト（v188000_tests、5件）

| テスト名 | 内容 |
|---|---|
| `version_is_18_8_0` | Cargo.toml に "18.8.0" が含まれる |
| `api_annotation_parses` | `#[api(method = "GET", path = "/users/:id")] fn ...` が `ApiAnnotation { method: "GET", path: "/users/:id" }` としてパースされる |
| `openapi_generates` | `#[api(...)]` 付き fn から OpenAPI JSON 文字列が生成される（`paths` / `components` が含まれる） |
| `graphql_generates` | `#[api(...)]` 付き fn から GraphQL SDL 文字列が生成される（`type Query` が含まれる） |
| `serve_routes_request` | ルートテーブルが正しく構築される（GET /users/:id がマッチする） |

---

## 完了条件（PASS=5）

1. `#[api(method = "GET", path = "/users/:id")]` アノテーションが解析される
2. `fav generate api --format openapi` が有効な OpenAPI 3.0 JSON/YAML を生成する
3. `fav generate api --format graphql` が有効な GraphQL SDL を生成する
4. ルートテーブルが正しく構築され、`:param` パスパラメータが認識される
5. `fav serve` コマンドが CLI で認識される

---

## スコープ外（v19.x 以降）

- OpenAPI セキュリティスキーマ（API Key / OAuth2）
- WebSocket エンドポイント
- gRPC から REST への変換
- `fav deploy` との統合（Lambda/ECS へのデプロイ）
- 本番用 HTTP サーバー（`fav serve` は開発用のみ）
- スキーマ型（v18.4）との深い整合性チェック（`--check-schemas`）の完全実装
