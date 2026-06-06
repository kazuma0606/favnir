# Favnir v11.6.0 仕様書

作成日: 2026-06-06
テーマ: `!Postgres` → psycopg2 Python トランスパイル

---

## 背景と目的

v11.5.0 で Fav ネイティブ側の `!Postgres` エフェクト + VM プリミティブが完成した。
v11.6.0 では `emit_python.rs` トランスパイラ側に Postgres 対応を追加し、
`Postgres.*` 呼び出しを psycopg2 Python コードに変換できるようにする。

ターゲット: `fav transpile --target python` で生成した `.py` が psycopg2 経由で
PostgreSQL に接続・操作できる状態になること。

---

## 変換対応表（Fav → Python）

| Fav プリミティブ | 生成 Python コード |
|---|---|
| `Postgres.execute_raw(sql, params)` | `_pg_execute(sql, params)` ヘルパー呼び出し |
| `Postgres.query_raw(sql, params)` | `_pg_query(sql, params)` ヘルパー呼び出し |
| `Postgres.infer_table_raw(table)` | `_pg_infer_table(table)` ヘルパー呼び出し |

---

## 生成コード: `_pg_execute` / `_pg_query` ヘルパー

```python
import psycopg2
import psycopg2.extras
import os as _os

def _pg_connect():
    url = _os.environ.get("DATABASE_URL")
    if url:
        return psycopg2.connect(url)
    return psycopg2.connect(
        host=_os.environ.get("PGHOST", "localhost"),
        port=int(_os.environ.get("PGPORT", "5432")),
        dbname=_os.environ.get("PGDATABASE", "postgres"),
        user=_os.environ.get("PGUSER", "postgres"),
        password=_os.environ.get("PGPASSWORD", ""),
    )

def _pg_execute(sql, params_json):
    import json as _json_mod
    params = _json_mod.loads(params_json) if isinstance(params_json, str) else params_json
    conn = _pg_connect()
    try:
        with conn.cursor() as cur:
            cur.execute(sql, params)
        conn.commit()
        return ("Ok", None)
    except Exception as e:
        return ("Err", str(e))
    finally:
        conn.close()

def _pg_query(sql, params_json):
    import json as _json_mod
    params = _json_mod.loads(params_json) if isinstance(params_json, str) else params_json
    conn = _pg_connect()
    try:
        with conn.cursor(cursor_factory=psycopg2.extras.RealDictCursor) as cur:
            cur.execute(sql, params)
            rows = [dict(r) for r in cur.fetchall()]
        return ("Ok", _json_mod.dumps(rows))
    except Exception as e:
        return ("Err", str(e))
    finally:
        conn.close()
```

---

## pyproject.toml 依存追加

`import psycopg2` が生成されている場合、`pyproject.toml` の dependencies に自動追加:

```toml
"psycopg2-binary>=2.9",
```

---

## PyEmitter への変更

### 新規フィールド（`emit_python.rs`）

```rust
needs_psycopg2:      bool,
needs_pg_helpers:    bool,
```

### 2-pass 同期（`copy_flags_from_sub`）

```rust
self.needs_psycopg2   = sub.needs_psycopg2;
self.needs_pg_helpers = sub.needs_pg_helpers;
```

### import ブロック

```rust
if self.needs_psycopg2 {
    self.line("import psycopg2");
    self.line("import psycopg2.extras");
    self.line("import os as _os");
}
```

### helpers emit

```rust
if self.needs_pg_helpers { self.emit_pg_helpers(); }
```

### NS ディスパッチ（emit_call_expr）

```rust
("Postgres", "execute_raw") if a.len() == 2 => {
    self.needs_psycopg2 = true;
    self.needs_pg_helpers = true;
    self.needs_json = true;
    return format!("_pg_execute({}, {})", a[0], a[1])
}
("Postgres", "query_raw") if a.len() == 2 => {
    self.needs_psycopg2 = true;
    self.needs_pg_helpers = true;
    self.needs_json = true;
    return format!("_pg_query({}, {})", a[0], a[1])
}
("Postgres", name) => {
    self.needs_psycopg2 = true;
    self.needs_pg_helpers = true;
    return format!("_pg_{}({})", name, a.join(", "))
}
```

---

## driver.rs — pyproject.toml psycopg2 依存追加

`cmd_transpile` の pyproject.toml 生成ロジックに `psycopg2-binary` を追加:

```rust
let psycopg2_dep = if py_src.contains("import psycopg2") {
    "    \"psycopg2-binary>=2.9\",\n"
} else {
    ""
};
// boto3_dep と psycopg2_dep を両方 dependencies に出力
```

---

## テスト設計（v11600_tests）

| テスト名 | 検証内容 |
|---|---|
| `transpile_postgres_execute_raw` | `Postgres.execute_raw` → `_pg_execute(...)` を含む |
| `transpile_postgres_query_raw` | `Postgres.query_raw` → `_pg_query(...)` を含む |
| `transpile_postgres_imports_psycopg2` | `import psycopg2` が生成される |
| `transpile_postgres_pg_connect_helper` | `_pg_connect` ヘルパーが含まれる |
| `transpile_postgres_pyproject_psycopg2_dep` | pyproject.toml に `psycopg2-binary` が含まれる |
| `transpile_postgres_pipeline_smoke` | `!Postgres` パイプライン → Python 出力が構文的に正しい |

---

## バージョン更新

- `fav/Cargo.toml`: `version = "11.6.0"`
