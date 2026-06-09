# Favnir v13.2.0 仕様書

Date: 2026-06-09
Theme: DbRead / DbWrite / StorageRead / StorageWrite interface 実装

---

## 概要

v13.1.0 で確立した `interface` 継承基盤の上に、
データ操作に関わる 4 つの capability interface を言語に組み込む。

この版での実装スコープ：
1. `DbRead` / `DbWrite` / `StorageRead` / `StorageWrite` を組み込み interface として登録
2. `ctx.db.query(...)` 形式のメソッド呼び出し構文（チェーン型フィールドアクセス）の型チェック
3. E0020: capability interface の型不一致エラー
4. `runes/postgres/` / `runes/aws/` / `runes/snowflake/` に Favnir レベルの実装型を追加
5. W009: 旧 `Postgres.*` / `AWS.*` / `Snowflake.*` 直接呼び出しへの deprecated 警告

`HttpClient` / `Io` / `Env` は **v13.3.0**。
コンテキスト型（`LoadCtx` / `WriteCtx`）の充足チェックは **v13.4.0**。

---

## 機能 1: 組み込み capability interface 定義

### インターフェース仕様

```favnir
// DB 読み取り（SELECT 相当）
interface DbRead {
    query:   (sql: String, params: List<String>) -> Result<String, String>
    query1:  (sql: String, params: List<String>) -> Result<String, String>
}

// DB 書き込み（INSERT / UPDATE / DELETE 相当）
interface DbWrite {
    execute: (sql: String, params: List<String>) -> Result<Int, String>
}

// Storage 読み取り（S3 GetObject 等）
interface StorageRead {
    get:    (bucket: String, key: String) -> Result<String, String>
    list:   (bucket: String, prefix: String) -> Result<List<String>, String>
}

// Storage 書き込み（S3 PutObject 等）
interface StorageWrite {
    put:    (bucket: String, key: String, body: String) -> Result<Unit, String>
    delete: (bucket: String, key: String) -> Result<Unit, String>
}
```

### 戻り値型の設計方針

`DbRead.query` の戻り値は `Result<String, String>`（JSON エンコードされた行リスト）とする。
既存の `Postgres.query_raw` / `Snowflake.query_raw` と戻り値型を合わせることで、
移行時の型変換コストをゼロにする。

`StorageRead.get` も同様に `Result<String, String>`（オブジェクト本文）。

### 登録方法

`fav/src/middle/checker.rs` の `InterfaceRegistry::new()` で
これら 4 interface をハードコードして事前登録する：

```rust
fn builtin_interfaces() -> Vec<InterfaceDef> {
    vec![
        InterfaceDef {
            name: "DbRead".into(),
            super_interface: None,
            methods: {
                let mut m = HashMap::new();
                m.insert("query".into(), Type::Fn(vec![
                    Type::String, Type::List(Box::new(Type::String))
                ], Box::new(Type::Result(Box::new(Type::String), Box::new(Type::String)))));
                m.insert("query1".into(), /* 同型 */);
                m
            },
        },
        // DbWrite, StorageRead, StorageWrite も同様
    ]
}
```

`checker.fav` / `compiler.fav` にも組み込み interface として型情報を文字列定数で保持する。

---

## 機能 2: `ctx.db.query(...)` メソッド呼び出しの型チェック

### 構文

```favnir
// ctx: LoadCtx（LoadCtx: CommonCtx、db: DbRead を持つ）
bind rows <- ctx.db.query("SELECT * FROM users WHERE id = $1", List.of(user_id))
```

`ctx.db.query(...)` は AST 上では：

```
Apply(
  FieldAccess(FieldAccess(Ident("ctx"), "db"), "query"),
  [Literal(sql), Literal(params)]
)
```

### 型チェック規則

1. `ctx` の型を解決 → 例: `LoadCtx`（interface 型）
2. `LoadCtx` のフィールド `db` の型を解決 → `DbRead`（v13.1.0 の継承チェーン解決を使用）
3. `DbRead` のメソッド `query` の型を `InterfaceRegistry` から解決
   → `(String, List<String>) -> Result<String, String>`
4. 引数の型を検証し、戻り値型として `Result<String, String>` を返す

### 実装方針

`checker.rs` の `infer_expr` で `Apply(FieldAccess(FieldAccess(...), method_name), args)` パターンを検出し、
最内の式の型が interface 型なら capability method call として処理する。

既存の `NS.fn(args)` パターン（`Apply(FieldAccess(Ident(ns), fn), args)` 形式）とは
ネストの深さで区別する。

---

## 機能 3: E0020 — capability interface 型不一致

### エラー形式

```
E0020: type `String` does not implement interface `DbRead`
  --> pipeline.fav:8:20
   |
 8 | bind rows <- ctx.db.query(sql, params)
   |              ^^^^^^^^^^^^ expected `DbRead`, found `String`
   |
   = help: pass a value that implements `DbRead`
   = help: available implementations: PostgresDb, SnowflakeDb, MockDb
```

### 検出タイミング

- 関数引数の型チェック時に、期待型が interface 型で実際の型がそれを実装していない場合
- フィールドアクセス `ctx.db` で `ctx` の型に `db` フィールドが存在しない場合（既存 E0021 相当）

---

## 機能 4: Rune 実装型

### PostgresDb（`runes/postgres/postgres_db.fav`）

```favnir
// PostgresDb は DbRead と DbWrite を実装する Postgres 接続ラッパー
type PostgresDb(String)  // 接続文字列を保持

impl DbRead for PostgresDb {
    fn query(db: PostgresDb, sql: String, params: List<String>) -> Result<String, String> {
        Postgres.query_raw(sql, params)
    }
    fn query1(db: PostgresDb, sql: String, params: List<String>) -> Result<String, String> {
        Postgres.query_raw(sql, params)
    }
}

impl DbWrite for PostgresDb {
    fn execute(db: PostgresDb, sql: String, params: List<String>) -> Result<Int, String> {
        Postgres.execute_raw(sql, params)
    }
}
```

### S3Storage（`runes/aws/s3_storage.fav`）

```favnir
type S3Storage(String)  // バケット名または設定文字列

impl StorageRead for S3Storage {
    fn get(s: S3Storage, bucket: String, key: String) -> Result<String, String> {
        AWS.s3_get_object_raw(bucket, key)
    }
    fn list(s: S3Storage, bucket: String, prefix: String) -> Result<List<String>, String> {
        AWS.s3_list_objects_raw(bucket, prefix)
    }
}

impl StorageWrite for S3Storage {
    fn put(s: S3Storage, bucket: String, key: String, body: String) -> Result<Unit, String> {
        AWS.s3_put_object_raw(bucket, key, body)
    }
    fn delete(s: S3Storage, bucket: String, key: String) -> Result<Unit, String> {
        AWS.s3_delete_object_raw(bucket, key)
    }
}
```

### SnowflakeDb（`runes/snowflake/snowflake_db.fav`）

```favnir
type SnowflakeDb(String)

impl DbRead for SnowflakeDb {
    fn query(db: SnowflakeDb, sql: String, params: List<String>) -> Result<String, String> {
        Snowflake.query_raw(sql)
    }
    fn query1(db: SnowflakeDb, sql: String, params: List<String>) -> Result<String, String> {
        Snowflake.query_raw(sql)
    }
}

impl DbWrite for SnowflakeDb {
    fn execute(db: SnowflakeDb, sql: String, params: List<String>) -> Result<Int, String> {
        Snowflake.execute_raw(sql)
    }
}
```

### MockDb（`runes/ctx/mock_db.fav`）

テスト用モック実装。`seed` で行データを渡せる。

```favnir
type MockDb(List<String>)  // シードされた JSON 行リスト

fn MockDb.empty() -> MockDb { MockDb(List.empty()) }
fn MockDb.seed(rows: List<String>) -> MockDb { MockDb(rows) }

impl DbRead for MockDb {
    fn query(db: MockDb, sql: String, params: List<String>) -> Result<String, String> {
        // シードデータを JSON 配列として返す
        Result.ok(Json.encode_raw(db))
    }
    fn query1(db: MockDb, sql: String, params: List<String>) -> Result<String, String> {
        Result.ok(Json.encode_raw(db))
    }
}

impl DbWrite for MockDb {
    fn execute(db: MockDb, sql: String, params: List<String>) -> Result<Int, String> {
        Result.ok(0)
    }
}
```

---

## 機能 5: W009 — 旧 Rune 直接呼び出し deprecated 警告

### 警告形式

```
W009: direct Rune call is deprecated — use capability interface instead
  --> pipeline.fav:10:10
   |
10 | bind rows <- Postgres.query_raw(sql, params)
   |              ^^^^^^^^^^^^^^^^^^^^^ deprecated
   |
   = help: migrate to `chain rows <- ctx.db.query(sql, params)`
   = note: direct Rune calls will be an error in v14.0
```

### 対象

| 旧呼び出し | 推奨移行先 |
|---|---|
| `Postgres.query_raw(...)` | `ctx.db.query(...)` |
| `Postgres.execute_raw(...)` | `ctx.db.execute(...)` |
| `AWS.s3_put_object_raw(...)` | `ctx.storage.put(...)` |
| `AWS.s3_get_object_raw(...)` | `ctx.storage.get(...)` |
| `AWS.s3_list_objects_raw(...)` | `ctx.storage.list(...)` |
| `Snowflake.query_raw(...)` | `ctx.db.query(...)` |
| `Snowflake.execute_raw(...)` | `ctx.db.execute(...)` |

### 検出方法

`check_ambient_effects` と同じく AST walk で `Apply(FieldAccess(Ident(ns), fn), ...)` を検出。
対象 namespace / 関数名のペアを定数テーブルで管理。

W009 は `fav check --ambient` フラグ時のみ出力（W008 と同様）。
`fav lint` には含めない。

---

## テストケース

| テスト名 | 内容 |
|---|---|
| `version_is_13_2_0` | `CARGO_PKG_VERSION == "13.2.0"` |
| `db_read_interface_registered` | `DbRead` が built-in interface として checker に登録されている |
| `db_read_interface_type_check` | `ctx.db.query(sql, params)` が型チェックを通る |
| `db_write_rejects_read_ctx` | `DbRead` 引数に `DbWrite` しか持たない ctx を渡す → E0020 |
| `storage_read_interface_registered` | `StorageRead` が登録されている |
| `storage_write_put_type_check` | `ctx.storage.put(bucket, key, body)` の型チェック |
| `postgres_db_rune_compiles` | `runes/postgres/postgres_db.fav` がコンパイルエラーなし |
| `mock_db_rune_compiles` | `runes/ctx/mock_db.fav` がコンパイルエラーなし |
| `w009_postgres_direct_deprecated` | `Postgres.query_raw(...)` が `--ambient` フラグで W009 |
| `w009_no_flag_no_warning` | `--ambient` なしでは W009 は出ない |

---

## 完了条件

- [ ] `DbRead` / `DbWrite` / `StorageRead` / `StorageWrite` が `checker.rs` に事前登録される
- [ ] `ctx.db.query(sql, params)` が型チェックを通る（`ctx` が `DbRead` フィールドを持つ場合）
- [ ] E0020 が `DbRead` を実装していない型に対して検出される
- [ ] `runes/postgres/postgres_db.fav` が `fav check` でエラーなし
- [ ] `runes/aws/s3_storage.fav` が `fav check` でエラーなし
- [ ] `runes/snowflake/snowflake_db.fav` が `fav check` でエラーなし
- [ ] `runes/ctx/mock_db.fav` が `fav check` でエラーなし
- [ ] `fav check --ambient` で W009 が出力される
- [ ] `self/compiler.fav` / `self/checker.fav` が `fav check` でエラーなし
- [ ] `cargo test` 全通過

---

## 非目標

- `HttpClient` / `Io` / `Env` interface の実装（v13.3.0）
- `LoadCtx` / `WriteCtx` によるステージ別 capability 充足チェック（v13.4.0）
- `AppCtx` 具象型と `Ctx.build` / `Ctx.mock` Rune（v13.5.0）
- `seq` pipeline での ctx 型推論（v13.7.0）
- 旧 `!Effect` 記法の廃止（v13.10.0）
- `DynamoDb` の完全実装（DynamoDB VM primitive が未実装のため stub のみ）
