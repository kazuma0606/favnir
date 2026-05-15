# Favnir v3.3.0 Specification

## Theme: `db` rune — データベース接続

v3.3.0 は SQL データベースへの型安全なアクセスを提供する公式 rune を追加する。
v3.2.0 の `Schema.adapt<T>` をそのまま再利用し、
DB クエリ結果を型付きレコードのリストとして扱えるようにする。

---

## 1. 新規型定義

### 1.1 `DbError`

```favnir
type DbError = {
    code:    String
    message: String
}
```

### 1.2 `DbHandle` (不透明型)

接続ハンドルを表す VM 内部管理の不透明値。
Favnir コードからは `DbHandle` 型として扱い、フィールドには直接アクセスしない。

```favnir
// VM が管理する不透明型。ユーザーが構築することはない。
// DB.connect(...) の戻り値としてのみ取得する。
```

### 1.3 `TxHandle` (不透明型)

トランザクションハンドル。`DB.begin_tx(handle)` の戻り値としてのみ取得する。

---

## 2. エフェクト追加

```
effect Db
```

DB 操作は `!Db` エフェクトを持つ。
`public fn main() -> Unit !Io !Db { ... }` のように複数エフェクトを並記できる。

---

## 3. VM プリミティブ

### 3.1 接続管理

```
DB.connect(conn_str: String) -> Result<DbHandle, DbError> !Db
DB.close(handle: DbHandle) -> Unit !Db
```

**接続文字列フォーマット**:

| ドライバ | 形式 |
|---------|------|
| SQLite ファイル | `"sqlite:path/to/db.sqlite"` |
| SQLite インメモリ | `"sqlite::memory:"` |
| PostgreSQL | `"postgres://user:pass@host:port/dbname"` |

### 3.2 クエリ実行

```
DB.query_raw(handle: DbHandle, sql: String)
    -> Result<List<Map<String, String>>, DbError> !Db
```

- SELECT 結果を文字列マップのリストとして返す
- カラム名をキー、値を文字列に変換（NULL → 空文字列）
- `Schema.adapt<T>` に渡すと型付きリストになる（v3.2.0 からの再利用）

```
DB.execute_raw(handle: DbHandle, sql: String)
    -> Result<Int, DbError> !Db
```

- INSERT / UPDATE / DELETE を実行し、影響行数を返す

### 3.3 パラメータバインド

```
DB.query_raw_params(handle: DbHandle, sql: String, params: List<String>)
    -> Result<List<Map<String, String>>, DbError> !Db

DB.execute_raw_params(handle: DbHandle, sql: String, params: List<String>)
    -> Result<Int, DbError> !Db
```

- `?` プレースホルダーへのパラメータバインド（SQL インジェクション防止）
- params は `List<String>`（VM 側で型変換）

### 3.4 トランザクション

```
DB.begin_tx(handle: DbHandle) -> Result<TxHandle, DbError> !Db
DB.commit_tx(tx: TxHandle)   -> Result<Unit, DbError> !Db
DB.rollback_tx(tx: TxHandle) -> Result<Unit, DbError> !Db
DB.query_in_tx(tx: TxHandle, sql: String)
    -> Result<List<Map<String, String>>, DbError> !Db
DB.execute_in_tx(tx: TxHandle, sql: String)
    -> Result<Int, DbError> !Db
```

---

## 4. `runes/db/db.fav`

Favnir 製 rune。VM プリミティブ上の薄いラッパー。

```favnir
// 接続
public fn connect(conn_str: String) -> Result<DbHandle, DbError> !Db {
    DB.connect(conn_str)
}

// SELECT → 型付きリスト
public fn query<T>(handle: DbHandle, sql: String) -> Result<List<T>, DbError> !Db {
    chain raw <- DB.query_raw(handle, sql)
    bind adapted <- Schema.adapt(raw, type_name_of<T>())
    Result.map_err(adapted, |e| DbError { code: "E_SCHEMA"  message: e.message })
}

// SELECT（パラメータ付き）
public fn query_params<T>(handle: DbHandle, sql: String, params: List<String>)
    -> Result<List<T>, DbError> !Db {
    chain raw <- DB.query_raw_params(handle, sql, params)
    bind adapted <- Schema.adapt(raw, type_name_of<T>())
    Result.map_err(adapted, |e| DbError { code: "E_SCHEMA"  message: e.message })
}

// INSERT / UPDATE / DELETE
public fn execute(handle: DbHandle, sql: String) -> Result<Int, DbError> !Db {
    DB.execute_raw(handle, sql)
}

public fn execute_params(handle: DbHandle, sql: String, params: List<String>)
    -> Result<Int, DbError> !Db {
    DB.execute_raw_params(handle, sql, params)
}

// トランザクション
public fn transaction<T>(
    handle: DbHandle,
    f: TxHandle -> Result<T, DbError>
) -> Result<T, DbError> !Db {
    chain tx <- DB.begin_tx(handle)
    bind result <- f(tx)
    match result {
        Ok(v) => {
            chain _ <- DB.commit_tx(tx)
            Result.ok(v)
        }
        Err(e) => {
            bind _ <- DB.rollback_tx(tx)
            Result.err(e)
        }
    }
}

// 切断
public fn close(handle: DbHandle) -> Unit !Db {
    DB.close(handle)
}
```

---

## 5. 対応ドライバ

| ドライバ | Cargo クレート | 優先度 |
|---------|--------------|--------|
| SQLite  | `rusqlite` (既存) | 高 |
| PostgreSQL | `postgres` | 高 |

MySQL は v3.3.0 スコープ外（将来対応）。

---

## 6. セキュリティ設計

### 6.1 パラメータバインド必須

SQL 文字列に直接ユーザー入力を連結することはアンチパターン。
`DB.query_raw_params` / `DB.execute_raw_params` のプレースホルダーを使う。

### 6.2 L005: 接続情報ハードコード警告

`lint.rs` に L005 を追加。
以下のパターンを検出してリンタ警告を出す:

```
L005: hardcoded db credential
  hint: use Env.get("DB_PASSWORD") instead of string literals for credentials
```

検出パターン:
- `DB.connect("postgres://...user:pass...")`のように接続文字列にパスワードが含まれる場合
- `fav lint` 実行時に報告する

---

## 7. `Env.get` 組み込み

環境変数読み取り用 VM プリミティブ（L005 推奨代替として追加）:

```
Env.get(name: String) -> String !Io
Env.get_or(name: String, default: String) -> String
```

---

## 8. エラーコード追加

| コード | タイトル |
|--------|---------|
| E0601 | db connection failed — 接続文字列が不正またはDBに接続できない |
| E0602 | db query failed — SQL 文法エラーまたは実行時エラー |
| E0603 | db transaction failed — トランザクション開始・コミット失敗 |
| E0604 | db schema mismatch — DB カラムと Favnir 型のフィールドが一致しない |
| E0605 | db driver unsupported — 接続文字列のドライバ種別が未対応 |

---

## 9. 利用例

### SQLite インメモリ CRUD

```favnir
import rune "db"

type User = {
    id:   Int
    name: String
    age:  Int
}

public fn main() -> Unit !Io !Db {
    chain conn <- db.connect("sqlite::memory:")

    bind _ <- db.execute(conn,
        "CREATE TABLE users (id INTEGER, name TEXT, age INTEGER)")

    bind _ <- db.execute_params(conn,
        "INSERT INTO users VALUES (?, ?, ?)",
        ["1", "Alice", "30"])

    chain users <- db.query<User>(conn, "SELECT id, name, age FROM users")
    IO.println($"Found {List.length(users)} users")

    db.close(conn)
}
```

### トランザクション

```favnir
import rune "db"

public fn main() -> Unit !Io !Db {
    chain conn <- db.connect(Env.get_or("DB_URL", "sqlite::memory:"))

    bind result <- db.transaction<Int>(conn, |tx| {
        chain _ <- DB.execute_in_tx(tx, "INSERT INTO events VALUES (1, 'login')")
        chain _ <- DB.execute_in_tx(tx, "INSERT INTO events VALUES (2, 'view')")
        Result.ok(2)
    })

    match result {
        Ok(n)  => IO.println($"Committed {n} events")
        Err(e) => IO.println($"Rolled back: {e.message}")
    }
}
```

### DB + CSV 異種ソース統合（ロードマップ完成例）

```favnir
import rune "db"
import rune "csv"

type MasterRow = { id: Int  name: String }
type TxRow     = { master_id: Int  value: Float }

public fn main() -> Unit !Io !Db !File {
    chain conn    <- db.connect(Env.get("DB_URL"))
    chain masters <- db.query<MasterRow>(conn, "SELECT id, name FROM masters")

    bind csv_text <- File.read("transactions.csv")
    chain txns    <- csv.parse<TxRow>(csv_text)

    // Join: id == master_id
    bind joined <- List.filter_map(masters, |m| {
        bind match_tx <- List.find(txns, |t| t.master_id == m.id)
        Option.map(match_tx, |t| $"{m.name}: {t.value}")
    })

    for row in joined { IO.println(row); }
}
```

---

## 10. 完了条件

- `db.connect("sqlite::memory:")` で接続でき、`db.query<T>` / `db.execute` が動く
- `db.query_params<T>` でプレースホルダーバインドが動く
- `db.transaction<T>` でコミット・ロールバックが動く
- PostgreSQL 接続文字列で `db.connect` が動く
- DB カラムと型フィールドの不一致で E0604 が出る
- `fav lint` で接続文字列にパスワードを直書きすると L005 が出る
- `Env.get` / `Env.get_or` で環境変数が読める
- 既存テストが全て通る

---

## 11. 非ゴール（v3.3.0 スコープ外）

- MySQL / MariaDB 対応（将来）
- コネクションプール（将来）
- ORM / クエリビルダー（Favnir の哲学は生 SQL + 型安全マッピング）
- マイグレーション管理ツール（将来 `fav migrate-db` 候補）
- 非同期 DB ドライバ（`!Async` エフェクト統合は v3.6.0 以降）
