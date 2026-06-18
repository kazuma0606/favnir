# v18.4.0 仕様書 — Schema Types（スキーマ型）

## 概要

BigQuery / Snowflake / Postgres のテーブルスキーマや JSON Schema ファイルを
**型として直接インポート**できるようにする。

`type UsersRow = schema "file:schemas/users.json"` と書くだけで、
スキーマファイルから Favnir のレコード型が自動生成される。
`fav check` / `fav build` 実行時にスキーマを取得し `.fav/schema-cache/` にキャッシュする。

---

## 構文

```fav
// JSON Schema ファイルから型を生成（v18.4.0 の主要対象）
type UsersRow = schema "file:schemas/users.json"
// → { id: Int, name: String, email: String, created_at: String }

// BigQuery テーブルから型を生成（将来拡張用、v18.4.0 では parse のみ）
type UsersBq = schema "bigquery:my-project.my_dataset.users"

// Postgres テーブルから型を生成（将来拡張用、v18.4.0 では parse のみ）
type OrderRow = schema "postgres:orders"

// Snowflake テーブルから型を生成（将来拡張用、v18.4.0 では parse のみ）
type ProductRow = schema "snowflake:MY_DB.MY_SCHEMA.PRODUCTS"
```

### `schema` キーワードの位置

`schema` は `type` 宣言の右辺にのみ使用する。
`parse_alias_decl` の `parse_type_expr` 呼び出しで `"schema"` ident を検出し、
`TypeExpr::Schema(source_uri, span)` を返す。

---

## スキーマ URI のフォーマット

| プレフィックス | ソース | v18.4.0 での動作 |
|---|---|---|
| `file:path` | JSON Schema ファイル | 実際にファイルを読み込んで型生成 |
| `bigquery:project.dataset.table` | BigQuery | parse のみ（接続なし） |
| `postgres:table` | PostgreSQL | parse のみ（接続なし） |
| `snowflake:DB.SCHEMA.TABLE` | Snowflake | parse のみ（接続なし） |

v18.4.0 では `file:` ソースのみを完全実装し、他は `Type::Unknown` として扱う。

---

## JSON Schema の型マッピング

| JSON Schema type | Favnir 型 |
|---|---|
| `"type": "integer"` | `Int` |
| `"type": "number"` | `Float` |
| `"type": "string"` | `String` |
| `"type": "boolean"` | `Bool` |
| `"type": "array"` | `List<T>` |
| `"type": "object"` | レコード型（再帰） |
| `"type": "null"` / nullable | `Option<T>` |

---

## スキーマキャッシュ

スキーマ取得結果は `.fav/schema-cache/` にキャッシュされる：

```
.fav/
  schema-cache/
    file__schemas__users.json     # キャッシュ済みスキーマ（JSON フォーマット）
    bigquery__my-project__...     # 将来の DB 接続キャッシュ
```

キャッシュ形式: `{ "fields": [{ "name": "id", "type": "Int" }, ...] }`

`fav check --refresh-schemas` でキャッシュを破棄して再取得する。

---

## AST 変更

### `TypeExpr::Schema` の追加

```rust
pub enum TypeExpr {
    // ... 既存 variants ...
    /// `schema "source:identifier"` — schema type import (v18.4.0)
    Schema(String, Span),
}
```

---

## エラーコード

| コード | 説明 |
|---|---|
| E0338 | Schema ファイルが見つからない（`file:` ソースで対象ファイルが存在しない） |
| E0339 | Schema ファイルの形式が無効（JSON パースエラーなど） |

---

## CLI

```bash
# キャッシュを破棄して再取得
fav check --refresh-schemas

# スキーマ型を含むファイルを通常どおりチェック
fav check src/main.fav
```

---

## テスト一覧（v184000_tests、5件）

| テスト名 | 内容 |
|---|---|
| `version_is_18_4_0` | Cargo.toml に "18.4.0" が含まれる |
| `schema_type_parses` | `type X = schema "file:..."` が AST として解析される |
| `schema_cache_creates` | スキーマキャッシュファイルが `.fav/schema-cache/` に生成される |
| `schema_file_source` | `schema "file:path.json"` から型フィールドが生成される |
| `schema_type_checks` | スキーマ型のフィールドアクセスが型チェックされる |

---

## 完了条件

- [ ] `TypeExpr::Schema(String, Span)` が `ast.rs` に存在する
- [ ] `parse_alias_decl` で `schema "..."` が解析される
- [ ] `file:` URI からスキーマが読み込まれてレコード型が生成される
- [ ] スキーマキャッシュが `.fav/schema-cache/` に書き込まれる
- [ ] スキーマ型のフィールドが型チェック（`record_fields`）に登録される
- [ ] 存在しないフィールドへのアクセスが型エラーになる
- [ ] E0338 / E0339 エラーコードが定義される
- [ ] `fav check --refresh-schemas` フラグが動作する
- [ ] `site/content/docs/language/schema-types.mdx` が存在する
- [ ] `cargo test v184000` — 5/5 PASS
- [ ] `cargo test` — リグレッションなし
