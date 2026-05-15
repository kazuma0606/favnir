# Favnir v3.3.0 Language Specification

## Overview

Favnir v3.3.0 adds the official `db` rune for SQL database access, `Env.get` / `Env.get_or`
environment variable primitives, the `!Db` effect, opaque `DbHandle` / `TxHandle` types, a new
`DbError` record type, error codes E0601–E0605, and lint rule L008 for hardcoded credentials.

---

## New Runtime Types

### `DbError`

```favnir
type DbError = {
    code:    String
    message: String
}
```

Standard error payload returned by all `DB.*` and `db.*` functions when a database operation fails.

### `DbHandle` (opaque)

An opaque connection handle obtained from `DB.connect`. Favnir code cannot construct a `DbHandle`
directly — it is only obtained as the `Ok` payload of `DB.connect`.

### `TxHandle` (opaque)

An opaque transaction handle obtained from `DB.begin_tx`. Shares the same underlying connection as
its parent `DbHandle`.

---

## New Effect: `!Db`

```favnir
effect Db
```

All `DB.*` primitive calls require the enclosing function to declare `!Db`. Multiple effects can
be combined:

```favnir
public fn main() -> Unit !Io !Db {
    chain conn <- DB.connect("sqlite::memory:")
    ...
}
```

The `Db` effect is already a member of `BUILTIN_EFFECTS`.

---

## DB VM Primitives

### Connection Management

| Primitive | Signature | Description |
|-----------|-----------|-------------|
| `DB.connect` | `String -> Result<DbHandle, DbError> !Db` | Open a DB connection. |
| `DB.close`   | `DbHandle -> Unit !Db` | Close and drop the connection. |

Connection string formats:

| Driver | Format |
|--------|--------|
| SQLite file | `"sqlite:path/to/db.sqlite"` |
| SQLite in-memory | `"sqlite::memory:"` |
| PostgreSQL | `"postgres://user:pass@host:port/dbname"` |

### Query Execution

| Primitive | Signature | Description |
|-----------|-----------|-------------|
| `DB.query_raw` | `(DbHandle, String) -> Result<List<Map<String,String>>, DbError> !Db` | SELECT → string-map rows. |
| `DB.execute_raw` | `(DbHandle, String) -> Result<Int, DbError> !Db` | INSERT/UPDATE/DELETE → affected rows. |
| `DB.query_raw_params` | `(DbHandle, String, List<String>) -> Result<List<Map<String,String>>, DbError> !Db` | SELECT with `?` parameter binding. |
| `DB.execute_raw_params` | `(DbHandle, String, List<String>) -> Result<Int, DbError> !Db` | DML with `?` parameter binding. |

`NULL` values are returned as empty strings `""`.

### Transactions

| Primitive | Signature | Description |
|-----------|-----------|-------------|
| `DB.begin_tx`    | `DbHandle -> Result<TxHandle, DbError> !Db` | Begin a transaction. |
| `DB.commit_tx`   | `TxHandle -> Result<Unit, DbError> !Db` | Commit. |
| `DB.rollback_tx` | `TxHandle -> Result<Unit, DbError> !Db` | Rollback. |
| `DB.query_in_tx` | `(TxHandle, String) -> Result<List<Map<String,String>>, DbError> !Db` | SELECT inside transaction. |
| `DB.execute_in_tx` | `(TxHandle, String) -> Result<Int, DbError> !Db` | DML inside transaction. |

Transactions use raw `BEGIN` / `COMMIT` / `ROLLBACK` SQL to avoid lifetime issues.

---

## `Env.*` Primitives

| Primitive | Signature | Description |
|-----------|-----------|-------------|
| `Env.get`    | `String -> Result<String, DbError>` | Read env var; `Err` if not set. |
| `Env.get_or` | `(String, String) -> String` | Read env var with default; pure (no effect). |

`Env.get_or` is effect-free because it never fails.

---

## `runes/db` Rune

Located at `<repo_root>/runes/db/db.fav`. Import with:

```favnir
import rune "db"
```

### Public API

| Function | Signature |
|----------|-----------|
| `db.connect` | `String -> Result<DbHandle, DbError> !Db` |
| `db.query` | `(DbHandle, String) -> Result<List<Map<String,String>>, DbError> !Db` |
| `db.query_params` | `(DbHandle, String, List<String>) -> Result<List<Map<String,String>>, DbError> !Db` |
| `db.execute` | `(DbHandle, String) -> Result<Int, DbError> !Db` |
| `db.execute_params` | `(DbHandle, String, List<String>) -> Result<Int, DbError> !Db` |
| `db.close` | `DbHandle -> Unit !Db` |

---

## Error Codes (E06xx)

| Code | Title | Description |
|------|-------|-------------|
| E0601 | db connection failed | Connection string invalid or DB unreachable. |
| E0602 | db query failed | SQL syntax error or runtime error. |
| E0603 | db transaction failed | Transaction begin/commit/rollback failed. |
| E0604 | db schema mismatch | DB column cannot be mapped to Favnir field type. |
| E0605 | db driver unsupported | Connection string driver not compiled in. |

---

## Lint Rule L008

```
L008: hardcoded db credential
  hint: use Env.get("DB_URL") instead of string literals for credentials
```

**Detection pattern**: `DB.connect` or `db.connect` called with a string literal that contains
both `"://"` and `"@"` (indicating a URL with credentials).

```favnir
// L008 triggered:
bind _ <- DB.connect("postgres://user:secret@localhost/db")

// No warning:
bind url <- Env.get_or("DB_URL", "sqlite::memory:")
bind _ <- DB.connect(url)
```

---

## Usage Examples

### SQLite CRUD

```favnir
import rune "db"

type User = { id: Int  name: String  age: Int }

public fn main() -> Unit !Io !Db {
    chain conn <- db.connect("sqlite::memory:")

    bind _ <- db.execute(conn,
        "CREATE TABLE users (id INTEGER, name TEXT, age INTEGER)")

    bind _ <- db.execute_params(conn,
        "INSERT INTO users VALUES (?, ?, ?)",
        collect { yield "1"; yield "Alice"; yield "30"; () })

    bind rows <- db.query(conn, "SELECT id, name, age FROM users")
    match rows {
        Ok(rs) => IO.println($"Found {List.length(rs)} users")
        Err(e) => IO.println($"Error: {e.message}")
    }

    db.close(conn)
}
```

### Transaction

```favnir
import rune "db"

public fn main() -> Unit !Io !Db {
    chain conn <- db.connect(Env.get_or("DB_URL", "sqlite::memory:"))
    chain tx   <- DB.begin_tx(conn)
    bind _     <- DB.execute_in_tx(tx, "INSERT INTO events VALUES (1, 'login')")
    bind _     <- DB.execute_in_tx(tx, "INSERT INTO events VALUES (2, 'view')")
    bind _     <- DB.commit_tx(tx)
    IO.println("Committed 2 events")
}
```

---

## Supported Drivers

| Driver | Cargo crate | Status |
|--------|-------------|--------|
| SQLite | `rusqlite` (bundled, existing) | Supported |
| PostgreSQL | `postgres = "0.19"` (optional feature `postgres_integration`) | Stub in v3.3.0 (returns E0605) |

MySQL support is out of scope for v3.3.0.

---

## Breaking Changes

None. All v3.2.0 code compiles without modification.
