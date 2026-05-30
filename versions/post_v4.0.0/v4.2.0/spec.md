# Favnir v4.2.0 仕様書 — DB / HTTP / gRPC Rune 2.0

作成日: 2026-05-16

## 概要

v4.1.0 で整備したマルチファイル rune 基盤の上に、DB・HTTP・gRPC の 3 本の rune に
**Favnir で書かれた実質的なロジック**を追加する。

v4.0.0〜v4.1.0 の rune は全て VM プリミティブの薄いラッパーだった。
v4.2.0 以降は rune が「Favnir の表現力のデモンストレーション」になる。

---

## 1. DB Rune 2.0

### 1.1 現状（v4.1.0）

```
runes/db/
  db.fav          ← barrel file (use connection.*, use query.*)
  connection.fav  ← connect, close
  query.fav       ← query, query_params, execute, execute_params
```

全関数が VM プリミティブ（`DB.*_raw`）の 1 行ラッパー。

### 1.2 追加するファイルと関数

#### `runes/db/transaction.fav`（新規）

```favnir
// with_transaction: コールバック内で失敗したら自動ロールバック
// f: DbHandle -> Result<List<Map<String, String>>, DbError>
public fn with_transaction(
    conn: DbHandle,
    f: DbHandle -> Result<List<Map<String, String>>, DbError>
) -> Result<List<Map<String, String>>, DbError> !Db {
    bind tx <- DB.begin_tx(conn)
    let result: Result<List<Map<String, String>>, DbError> = f(tx)
    match result {
        Ok(v)  => {
            bind _ <- DB.commit_tx(tx)
            Result.ok(v)
        }
        Err(e) => {
            bind _ <- DB.rollback_tx(tx)
            Result.err(e)
        }
    }
}

// savepoint: ネストしたトランザクション制御（SQLite SAVEPOINT）
public fn savepoint(conn: DbHandle, name: String) -> Result<Unit, DbError> !Db {
    DB.execute_raw(conn, String.concat("SAVEPOINT ", name))
    Result.ok(())
}

public fn release_savepoint(conn: DbHandle, name: String) -> Result<Unit, DbError> !Db {
    DB.execute_raw(conn, String.concat("RELEASE SAVEPOINT ", name))
    Result.ok(())
}

public fn rollback_to_savepoint(conn: DbHandle, name: String) -> Result<Unit, DbError> !Db {
    DB.execute_raw(conn, String.concat("ROLLBACK TO SAVEPOINT ", name))
    Result.ok(())
}
```

> **注意**: Favnir は v4.2.0 時点でジェネリック関数型引数（`<T>` 型変数）を持たないため、
> `with_transaction` の戻り値型は `List<Map<String, String>>` に固定する。
> 型変数の導入は v5.x 以降の言語機能拡張で対応する。

#### `runes/db/query.fav`（既存を拡張）

追加関数:

```favnir
// query_one: 結果の最初の行だけを返す（0行は Err）
public fn query_one(
    handle: DbHandle,
    sql: String
) -> Result<Map<String, String>, DbError> !Db {
    bind rows <- DB.query_raw(handle, sql)
    match List.head(rows) {
        Some(row) => Result.ok(row)
        None      => Result.err(DbError { code: "NOT_FOUND" message: "query returned no rows" })
    }
}

// paginate: LIMIT/OFFSET を自動付与
public fn paginate(
    handle: DbHandle,
    sql: String,
    page: Int,
    size: Int
) -> Result<List<Map<String, String>>, DbError> !Db {
    let offset: Int = page * size
    let paged_sql: String = String.concat(String.concat(String.concat(String.concat(
        sql, " LIMIT "), Debug.show(size)), " OFFSET "), Debug.show(offset))
    DB.query_raw(handle, paged_sql)
}

// batch_insert: List<Map> を反復して INSERT（トランザクション内で実行を推奨）
public fn batch_insert(
    handle: DbHandle,
    sql_template: String,
    rows: List<List<String>>
) -> Result<Int, DbError> !Db {
    List.fold_left(rows, Result.ok(0), |acc, params| {
        match acc {
            Err(e) => Result.err(e)
            Ok(n)  => {
                bind count <- DB.execute_raw_params(handle, sql_template, params)
                Result.ok(n + count)
            }
        }
    })
}
```

#### `runes/db/migration.fav`（新規）

マイグレーション管理を純 Favnir で実装。
VM プリミティブは既存の `DB.query_raw` / `DB.execute_raw` を使う。

```favnir
// ensure_migrations_table: _fav_migrations テーブルを初期化
fn ensure_migrations_table(conn: DbHandle) -> Result<Unit, DbError> !Db {
    bind _ <- DB.execute_raw(conn,
        "CREATE TABLE IF NOT EXISTS _fav_migrations (
             id INTEGER PRIMARY KEY AUTOINCREMENT,
             name TEXT NOT NULL UNIQUE,
             applied_at TEXT NOT NULL
         )")
    Result.ok(())
}

// applied_migrations: 適用済みマイグレーション名のリストを返す
public fn applied_migrations(conn: DbHandle) -> Result<List<String>, DbError> !Db {
    bind _ <- ensure_migrations_table(conn)
    bind rows <- DB.query_raw(conn, "SELECT name FROM _fav_migrations ORDER BY id ASC")
    let names: List<String> = List.map(rows, |row|
        Option.unwrap_or(Map.get(row, "name"), ""))
    Result.ok(names)
}

// mark_applied: マイグレーションを適用済みとして記録
public fn mark_applied(conn: DbHandle, name: String) -> Result<Unit, DbError> !Db {
    bind _ <- DB.execute_raw_params(conn,
        "INSERT INTO _fav_migrations (name, applied_at) VALUES (?, ?)",
        [name, IO.timestamp()])
    Result.ok(())
}
```

#### `runes/db/db.fav`（barrel 更新）

```favnir
use connection.{ connect, close }
use query.{ query, query_params, query_one, paginate, execute, execute_params, batch_insert }
use transaction.{ with_transaction, savepoint, release_savepoint, rollback_to_savepoint }
use migration.{ applied_migrations, mark_applied }
```

### 1.3 VM primitives — 追加なし

v4.2.0 の DB Rune 2.0 では新規 VM プリミティブは不要。
`DB.begin_tx` / `DB.commit_tx` / `DB.rollback_tx` / `DB.execute_in_tx` は v3.x で実装済み。

### 1.4 `fav db migrate` CLI コマンド

`driver.rs` に新規実装する。DB VM プリミティブを使った Rust コード。

```
fav db migrate           # migrations/*.sql を昇順で実行（未適用のみ）
fav db migrate --status  # 適用状態を表形式で表示
fav db migrate --rollback # 最後に適用したマイグレーションをロールバック
```

**ファイル命名規則**: `migrations/001_create_users.sql` （番号プレフィクスで昇順）

**_fav_migrations テーブル**（Rust レベルで確保、rune からも見える）:
```sql
CREATE TABLE IF NOT EXISTS _fav_migrations (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    name       TEXT NOT NULL UNIQUE,
    applied_at TEXT NOT NULL
);
```

**接続文字列**: `fav.toml` の `[database] url` または `--db-url` フラグ。
デフォルト: `sqlite://./fav.db`

---

## 2. HTTP Rune 2.0

### 2.1 現状（v4.1.0）

```
runes/http/
  http.fav       ← barrel (use request.*, use response.*)
  request.fav    ← get, post, post_json, get_body
  response.fav   ← ok, error_response
```

### 2.2 追加するファイルと関数

#### 新規 VM プリミティブ（Rust 追加）

| プリミティブ | 引数 | 戻り値 |
|------------|------|--------|
| `Http.put_raw(url, body, content_type)` | (String, String, String) | `Result<HttpResponse, HttpError> !Network` |
| `Http.delete_raw(url)` | (String) | `Result<HttpResponse, HttpError> !Network` |
| `Http.patch_raw(url, body, content_type)` | (String, String, String) | `Result<HttpResponse, HttpError> !Network` |
| `Http.get_raw_headers(url, headers)` | (String, Map<String,String>) | `Result<HttpResponse, HttpError> !Network` |
| `Http.post_raw_headers(url, body, ct, headers)` | (String, String, String, Map<String,String>) | `Result<HttpResponse, HttpError> !Network` |

#### `runes/http/client.fav`（新規）

```favnir
public fn put(url: String, body: String) -> Result<HttpResponse, HttpError> !Network {
    Http.put_raw(url, body, "application/json")
}

public fn delete(url: String) -> Result<HttpResponse, HttpError> !Network {
    Http.delete_raw(url)
}

public fn patch(url: String, body: String) -> Result<HttpResponse, HttpError> !Network {
    Http.patch_raw(url, body, "application/json")
}

public fn get_with_headers(
    url: String,
    headers: Map<String, String>
) -> Result<HttpResponse, HttpError> !Network {
    Http.get_raw_headers(url, headers)
}

public fn post_with_headers(
    url: String,
    body: String,
    headers: Map<String, String>
) -> Result<HttpResponse, HttpError> !Network {
    Http.post_raw_headers(url, body, "application/json", headers)
}
```

#### `runes/http/retry.fav`（新規）

```favnir
// with_retry: 最大 n 回リトライ。成功したら即返す。
public fn with_retry(
    max_attempts: Int,
    f: Unit -> Result<HttpResponse, HttpError>
) -> Result<HttpResponse, HttpError> !Network {
    fn attempt(n: Int) -> Result<HttpResponse, HttpError> !Network {
        let result: Result<HttpResponse, HttpError> = f(())
        match result {
            Ok(v)  => Result.ok(v)
            Err(e) => if n <= 1 then Result.err(e) else attempt(n - 1)
        }
    }
    attempt(max_attempts)
}

// retry_get: GET を最大 n 回リトライ
public fn retry_get(
    url: String,
    max_attempts: Int
) -> Result<HttpResponse, HttpError> !Network {
    with_retry(max_attempts, |_| Http.get_raw(url))
}

// retry_post: POST を最大 n 回リトライ
public fn retry_post(
    url: String,
    body: String,
    max_attempts: Int
) -> Result<HttpResponse, HttpError> !Network {
    with_retry(max_attempts, |_| Http.post_raw(url, body, "application/json"))
}
```

#### `runes/http/auth.fav`（新規）

認証ヘッダーを Map<String, String> として生成する純粋関数。

```favnir
// bearer: Authorization: Bearer <token> ヘッダーを生成
public fn bearer(token: String) -> Map<String, String> {
    Map.set((), "Authorization", String.concat("Bearer ", token))
}

// basic: Authorization: Basic <base64> ヘッダーを生成
// （Base64 エンコードは VM プリミティブに委譲）
public fn basic(username: String, password: String) -> Map<String, String> {
    let credentials: String = String.concat(String.concat(username, ":"), password)
    let encoded: String = String.base64_encode(credentials)
    Map.set((), "Authorization", String.concat("Basic ", encoded))
}

// api_key: X-Api-Key ヘッダーを生成
public fn api_key(key: String) -> Map<String, String> {
    Map.set((), "X-Api-Key", key)
}
```

> `String.base64_encode` が未実装の場合、`basic` 関数は placeholder として
> `String.concat("Basic ", credentials)` を返す（base64 未変換）。
> 実 base64 は v4.5.0 Auth Rune で追加する。

#### `runes/http/http.fav`（barrel 更新）

```favnir
use request.{ get, post, post_json, get_body }
use response.{ ok, error_response }
use client.{ put, delete, patch, get_with_headers, post_with_headers }
use retry.{ with_retry, retry_get, retry_post }
use auth.{ bearer, basic, api_key }
```

### 2.3 HttpResponse への status フィールド追加

現状の `HttpResponse` 型は `{ body: String status: Int }` となっているか確認する。
`http.ok` が正しく status コードを使っているかを `response.fav` でも補完する。

---

## 3. gRPC Rune 2.0 — フィールド名修正

### 3.1 問題（v4.0.0〜v4.1.0）

```favnir
// 現状: フィールド名が field1/field2（位置番号）
bind resp <- grpc.call("localhost:9090", "/UserService/GetUser", req)
// resp = { "field1": "42", "field2": "Alice" }
//         ↑ 実際は { "id": "42", "name": "Alice" } であるべき
```

原因: `proto_bytes_to_string_map(bytes)` が `type_metas` を参照せず
固定キー `"field1"`, `"field2"`, ... を生成している。

### 3.2 修正方針

#### VM 変更: `proto_bytes_to_string_map` に type_name を追加

`Grpc.decode_raw(type_name, encoded)` は既に `type_name` を受け取っているが、
`proto_bytes_to_string_map` 内では無視されている。

```rust
// vm.rs の修正: type_metas から field names を解決
fn proto_bytes_to_named_map(
    bytes: &[u8],
    type_name: &str,
    type_metas: &HashMap<String, TypeMeta>,
) -> Result<HashMap<String, VMValue>, String> {
    let fields = type_metas
        .get(type_name)
        .map(|m| m.fields.as_slice())
        .unwrap_or(&[]);
    // field_number → field_name の対応を作る
    // 位置 0 = field 1, 位置 1 = field 2, ...
    ...
}
```

#### 新規 VM プリミティブ: `Grpc.call_typed_raw`

```
Grpc.call_typed_raw(response_type_name, host, method, payload)
  -> Result<Map<String, String>, RpcError> !Rpc
```

既存の `Grpc.call_raw` は後方互換のため維持（フィールド名は position-based のまま）。
新規の `Grpc.call_typed_raw` がフィールド名解決を行う。

#### Favnir rune 変更: `runes/grpc/client.fav`

```favnir
// call_typed: フィールド名を保持した呼び出し（v4.2.0 以降推奨）
public fn call_typed(
    response_type: String,
    host: String,
    method: String,
    payload: Map<String, String>
) -> Result<Map<String, String>, RpcError> !Rpc {
    Grpc.call_typed_raw(response_type, host, method, payload)
}
```

#### Favnir rune 変更: `runes/grpc/codec.fav`

`decode` が既存の `Grpc.decode_raw(type_name, encoded)` を使っているため、
`proto_bytes_to_string_map` の修正だけで `grpc.decode` も自動修正される。

### 3.3 後方互換性

- `grpc.call` は `Grpc.call_raw` のまま（`field1`/`field2` を返す）
- `grpc.call_typed` を新規追加（実フィールド名）
- `grpc.decode` は `Grpc.decode_raw` 経由で改善される（破壊的変更に見えるが、
  既存コードで `field1` を使っているテストは修正が必要になる）

> **注意**: `grpc.decode` の修正は既存テストを壊す可能性がある。
> 既存の `grpc.decode` 呼び出しを使っているテストはすべて確認して更新する。

---

## 4. 共通: DB マイグレーション

### 4.1 `fav.toml` 設定

```toml
[database]
url        = "sqlite://./fav.db"
migrations = "migrations"   # migrations ディレクトリ（デフォルト）
```

### 4.2 migration ファイル形式

```
migrations/
  001_create_users.sql
  002_add_orders.sql
  003_add_index.sql
```

各ファイルは単純な SQL（セミコロン区切りで複数ステートメント可）。
ロールバックは `-- @down` セクションで記述（省略可）:

```sql
-- @up
CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT NOT NULL);

-- @down
DROP TABLE IF EXISTS users;
```

---

## 5. エラーコード

| コード | 意味 | 発生箇所 |
|--------|------|---------|
| E0520  | migration ファイルのパースエラー | `fav db migrate` |
| E0521  | DB 接続設定が未定義（fav.toml に `[database]` なし） | `fav db migrate` |
| E0522  | migration の実行失敗 | `fav db migrate` |

---

## 6. 新規 Cargo 依存

追加なし（既存の `rusqlite`, `ureq`, `h2`, `bytes` で対応）。

---

## 7. テスト方針

### DB Rune 2.0

- `db.test.fav` に追加: `with_transaction_commit`, `with_transaction_rollback`, `paginate_basic`, `query_one_found`, `query_one_not_found`, `batch_insert_multiple`, `migration_mark_applied`, `migration_applied_list`（計 8 件追加）
- `driver.rs` 統合テスト: `fav db migrate --status` の出力確認（3 件）

### HTTP Rune 2.0

- `http.test.fav` に追加: `with_retry_succeeds_first_attempt`, `with_retry_exhausts`, `bearer_header_format`, `basic_header_format`, `api_key_header_format`（計 5 件追加）
- VM stdlib テスト: `http_put_raw_err`, `http_delete_raw_err`, `http_patch_raw_err`（3 件）

### gRPC Rune 2.0

- `grpc.test.fav` に追加: `decode_returns_field_names`（1 件）
- VM stdlib テスト: `grpc_call_typed_raw_returns_named_fields`（1 件）

### 合計目標

- 既存 808 件を維持したまま 新規 20 件以上を追加

---

## 8. 完了条件

- [x] `db.with_transaction(conn, f)` が Favnir コードで実装され、commit/rollback が正しく動く
- [x] `db.paginate(conn, sql, page, size)` が正しい LIMIT/OFFSET を発行する
- [x] `db.query_one` が 0 行のとき `Err` を返す
- [x] `http.with_retry(3, f)` が失敗時に最大 3 回リトライする
- [x] `http.put` / `http.delete` / `http.patch` が新規 VM プリミティブ経由で動く
- [x] `http.bearer(token)` が `{ "Authorization": "Bearer <token>" }` を返す
- [x] `grpc.decode(type_name, encoded)` が実フィールド名を返す
- [x] `grpc.call_typed(type_name, host, method, payload)` が実フィールド名で結果を返す
- [x] `fav db migrate` が未適用マイグレーションを昇順で実行する
- [x] 全既存テスト（808 件）がパスする
