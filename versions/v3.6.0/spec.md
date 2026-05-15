# Favnir v3.6.0 Language Specification

## Theme: 増分処理（Incremental Processing）

"前回実行からの差分だけ処理する" をパイプライン宣言として書けるようにする。

---

## 1. `!Checkpoint` エフェクト

チェックポイントの読み書きを行う関数に付与するエフェクト。

```favnir
public fn main() -> Unit !Io !Checkpoint {
    bind last <- Checkpoint.last("etl_run")
    // ...
    Checkpoint.save("etl_run", current_ts);
}
```

---

## 2. `Checkpoint` VM プリミティブ

### `Checkpoint.last(name: String) -> Option<String> !Checkpoint`

最後に保存されたチェックポイント値を返す。存在しない場合は `Option.none()`。

```favnir
bind last_ts <- Checkpoint.last("daily_etl")
// last_ts : Option<String>
```

### `Checkpoint.save(name: String, value: String) -> Unit !Checkpoint`

チェックポイント値を保存する。次の実行で `Checkpoint.last` が返す。

```favnir
Checkpoint.save("daily_etl", "2026-05-15T00:00:00Z");
```

### `Checkpoint.reset(name: String) -> Unit !Checkpoint`

チェックポイントを削除する（次の実行はフルスキャン）。

```favnir
Checkpoint.reset("daily_etl");
```

---

## 3. `CheckpointMeta` 組み込み型

```favnir
type CheckpointMeta = {
    name:       String    // チェックポイント名
    value:      String    // 最後の値（空文字 = 未設定）
    updated_at: String    // ISO 8601 タイムスタンプ（未設定時は ""）
}
```

`Checkpoint.meta(name)` で取得可能：

```favnir
bind meta <- Checkpoint.meta("etl_run")
IO.println($"Last run: {meta.updated_at}")
```

---

## 4. `DB.upsert_raw` VM プリミティブ

### `DB.upsert_raw(conn: DbHandle, type_name: String, row: Map<String,String>, key_field: String) -> Unit !Db`

レコードを INSERT OR REPLACE（idempotent）。

```favnir
bind conn <- DB.connect("sqlite://state.db")
DB.upsert_raw(conn, "User", row, "id");
```

`key_field` はプライマリキーとして使用するフィールド名。

---

## 5. `runes/incremental/incremental.fav` 公開 API

| 関数 | シグネチャ | 説明 |
|------|-----------|------|
| `incremental.last` | `String -> Option<String> !Checkpoint` | チェックポイント読み取り |
| `incremental.save` | `(String, String) -> Unit !Checkpoint` | チェックポイント保存 |
| `incremental.reset` | `String -> Unit !Checkpoint` | チェックポイント削除 |
| `incremental.meta` | `String -> CheckpointMeta !Checkpoint` | メタ情報取得 |
| `incremental.run_since` | `(String, String -> List<Map<String,String>>) -> List<Map<String,String>> !Checkpoint !Io` | 前回以降のデータを取得してチェックポイント更新 |
| `incremental.upsert` | `(DbHandle, String, Map<String,String>, String) -> Unit !Db` | idempotent write |

### `incremental.run_since` 詳細

```favnir
// fetch_fn: (last_checkpoint_or_epoch) -> rows
bind new_rows <- incremental.run_since("etl_run", |since|
    DB.query_raw(conn, $"SELECT * FROM events WHERE ts > '{since}'")
)
// チェックポイントは自動更新（現在時刻 ISO 8601）
```

---

## 6. `fav.toml [checkpoint]` 設定

```toml
[checkpoint]
backend = "file"          # "file" | "sqlite"
path = ".fav_checkpoints" # file: ディレクトリ / sqlite: DBファイルパス
```

### file バックエンド（デフォルト）

`.fav_checkpoints/<name>.txt` にチェックポイント値を保存。

### sqlite バックエンド

指定 DB ファイルの `_fav_checkpoints` テーブルに保存。
テーブルは初回アクセス時に自動作成。

```sql
CREATE TABLE IF NOT EXISTS _fav_checkpoints (
    name       TEXT PRIMARY KEY,
    value      TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
```

---

## 7. 典型ワークフロー

```bash
# 初回実行（フルスキャン）
fav run etl.fav

# 2回目以降（差分のみ）
fav run etl.fav

# チェックポイントリセット（再フルスキャン）
fav checkpoint reset etl_run
```

```favnir
// etl.fav
import rune "incremental"

type Event = { id: Int ts: String payload: String }

public fn main() -> Unit !Io !Checkpoint !Db {
    bind conn <- DB.connect("sqlite://events.db")
    bind last <- incremental.last("etl_run")
    bind since <- Option.unwrap_or(last, "1970-01-01T00:00:00Z")
    bind rows <- DB.query_raw(conn, $"SELECT * FROM events WHERE ts > '{since}'")
    IO.println($"Processing {List.length(rows)} new rows...")
    // ... process rows ...
    bind now <- IO.timestamp()
    incremental.save("etl_run", now);
}
```

---

## 8. `fav checkpoint` サブコマンド

```bash
# チェックポイント一覧
fav checkpoint list

# チェックポイント値表示
fav checkpoint show <name>

# チェックポイントリセット
fav checkpoint reset <name>

# チェックポイント手動設定
fav checkpoint set <name> <value>
```

---

## 9. `IO.timestamp()` 追加

```favnir
IO.timestamp() -> String !Io
```

現在時刻を ISO 8601 UTC 文字列で返す（例: `"2026-05-15T12:34:56Z"`）。

---

## Breaking Changes

v3.5.0 との破壊的変更なし。
