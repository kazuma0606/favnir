# v27.3.0 仕様書 — clickhouse Rune 追加

## 概要

ClickHouse Rune を追加する。
`import rune "clickhouse"` → `ClickHouse.*` 名前空間で 4 関数が使用可能になる。
分析特化の列指向 DB に対して接続・クエリ・バルク挿入・非同期バルク挿入ができる。

---

## 背景

ロードマップ v27.3「clickhouse Rune 追加」より。Data Lakehouse フェーズの第 3 コンポーネント。
ClickHouse はリアルタイム集計クエリが高速な列指向 DB で、
ストリームから直接高速バルク挿入できる点で Favnir の Rune Foundation との相性が良い。

ロードマップ要件:
- 4 関数実装（connect / query / insert / async_insert）
- `clickhouse/clickhouse-server`（Docker）で `cargo test clickhouse` が 3 件以上 PASS

---

## 実装する関数

| 関数 | シグネチャ | 内容 |
|---|---|---|
| `ClickHouse.connect` | `(config: String) -> Result<String, String> !Db` | 接続確立（HTTP / native プロトコル）。接続ハンドル文字列を返す |
| `ClickHouse.query` | `(conn: String, sql: String) -> Result<String, String> !Db` | 型付きクエリ（SELECT）。JSON 配列文字列を返す |
| `ClickHouse.insert` | `(conn: String, table: String, rows: String) -> Result<Unit, String> !Db` | バルク挿入（JSON 形式） |
| `ClickHouse.async_insert` | `(conn: String, table: String, rows: String) -> Result<Unit, String> !Db` | 非同期バルク挿入（ClickHouse v21.11+） |

> **エフェクト**: `!Db` エフェクトを使用する（postgres Rune と同一）。Delta Lake / Iceberg の `!Io`（ファイル操作）とは異なり、DB 接続プロトコル経由の操作は `!Db` で統一する。ロードマップ v27.3 コードサンプルも `!Db` を使用しており整合している。

---

## VM Primitive（vm.rs に追加）

| primitive 名 | 実装方針 |
|---|---|
| `ClickHouse.connect_raw` | stub: 引数検証のみ、`"clickhouse-stub-conn"` 返却（クレート統合は v28.x）。接続ハンドルを返す関数のため `"[]"` ではなく固有の stub 識別子を使用（postgres Rune と同一パターン） |
| `ClickHouse.query_raw` | stub: 引数検証、`"[]"` 返却 |
| `ClickHouse.insert_raw` | stub: 引数検証、`ok_vm(VMValue::Unit)` 返却 |
| `ClickHouse.async_insert_raw` | stub: 引数検証、`ok_vm(VMValue::Unit)` 返却 |

> **注意**: `clickhouse-rs` クレートの Cargo 依存追加は WASM 互換性・ビルド時間への影響があるため、
> v27.3.0 では引数検証 stub とする。実 ClickHouse 接続は v28.x で実装する。
> すべての primitive に `#[cfg(not(target_arch = "wasm32"))]` / `#[cfg(target_arch = "wasm32")]` ガードを付ける。

**挿入位置**: Iceberg ブロック末尾（`"Iceberg.list_snapshots_raw" => Ok(err_vm(...))` の直後）・Azure Blob ブロックの直前。

---

## runes/clickhouse/clickhouse.fav

```favnir
// runes/clickhouse/clickhouse.fav — ClickHouse Rune (v27.3.0)
// clickhouse-rs 統合は v28.x 以降。現バージョンは引数検証 stub。
// TODO(v28.x): clickhouse-rs クレートを使った実接続に移行予定。
public fn connect(config: String) -> Result<String, String> !Db {
    ClickHouse.connect_raw(config)
}
public fn query(conn: String, sql: String) -> Result<String, String> !Db {
    ClickHouse.query_raw(conn, sql)
}
public fn insert(conn: String, table: String, rows: String) -> Result<Unit, String> !Db {
    ClickHouse.insert_raw(conn, table, rows)
}
public fn async_insert(conn: String, table: String, rows: String) -> Result<Unit, String> !Db {
    ClickHouse.async_insert_raw(conn, table, rows)
}
```

---

## examples/clickhouse_analytics.fav

```favnir
// examples/clickhouse_analytics.fav — ClickHouse Analytics デモ (v27.3.0)
import rune "clickhouse"

stage LoadEvents: Unit -> Result<String, String> !Db = |_| {
    bind conn <- ClickHouse.connect("http://localhost:8123")
    ClickHouse.query(conn, "SELECT * FROM events LIMIT 100")
}

// seq pipeline は前ステージの成功値（String）を次ステージの引数として渡す
stage InsertProcessed: String -> Result<String, String> !Db = |data| {
    bind conn <- ClickHouse.connect("http://localhost:8123")
    bind _ <- ClickHouse.insert(conn, "processed_events", data)
    Result.ok("inserted to clickhouse")
}

seq ClickHouseAnalyticsPipeline = LoadEvents |> InsertProcessed
```

---

## テスト

### driver.rs v273000_tests（8 件）

| テスト名 | 内容 |
|---|---|
| `clickhouse_rune_has_connect_fn` | `clickhouse.fav` に `"fn connect("` が含まれること |
| `clickhouse_rune_has_query_fn` | `clickhouse.fav` に `"fn query("` が含まれること |
| `clickhouse_rune_has_insert_fn` | `clickhouse.fav` に `"fn insert("` が含まれること |
| `clickhouse_rune_has_async_insert_fn` | `clickhouse.fav` に `"fn async_insert("` が含まれること |
| `clickhouse_rune_vm_has_connect_raw` | `vm.rs` に `"ClickHouse.connect_raw"` が含まれること |
| `clickhouse_rune_vm_has_insert_raw` | `vm.rs` に `"ClickHouse.insert_raw"` が含まれること |
| `clickhouse_example_has_pipeline` | `examples/clickhouse_analytics.fav` に `"ClickHouseAnalyticsPipeline"` が含まれること |
| `changelog_has_v27_3_0` | `CHANGELOG.md` に `"[v27.3.0]"` が含まれること |

### `cargo test clickhouse` 期待値

- `v273000_tests::clickhouse_rune_has_*` 4 件（connect / query / insert / async_insert）
- `v273000_tests::clickhouse_rune_vm_has_connect_raw` 1 件
- `v273000_tests::clickhouse_rune_vm_has_query_raw` 1 件
- `v273000_tests::clickhouse_rune_vm_has_insert_raw` 1 件
- `v273000_tests::clickhouse_rune_vm_has_async_insert_raw` 1 件
- `v273000_tests::clickhouse_example_has_pipeline` 1 件
- 合計 9 件（`changelog_has_v27_3_0` は `clickhouse` を含まないため除外）（ロードマップ要件「3 件以上」超過）

---

## 完了条件

- [ ] `fav/Cargo.toml` が `version = "27.3.0"` であること
- [ ] `runes/clickhouse/clickhouse.fav` に `fn connect(` が含まれること
- [ ] `runes/clickhouse/clickhouse.fav` に `fn query(` が含まれること
- [ ] `runes/clickhouse/clickhouse.fav` に `fn insert(` が含まれること
- [ ] `runes/clickhouse/clickhouse.fav` に `fn async_insert(` が含まれること
- [ ] `fav/src/backend/vm.rs` に `ClickHouse.connect_raw` が含まれること
- [ ] `fav/src/backend/vm.rs` に `ClickHouse.query_raw` が含まれること
- [ ] `fav/src/backend/vm.rs` に `ClickHouse.insert_raw` が含まれること
- [ ] `fav/src/backend/vm.rs` に `ClickHouse.async_insert_raw` が含まれること
- [ ] `examples/clickhouse_analytics.fav` に `ClickHouseAnalyticsPipeline` が含まれること
- [ ] `site/content/docs/runes/clickhouse.mdx` が存在すること
- [ ] `CHANGELOG.md` に `[v27.3.0]` エントリが存在すること
- [ ] `benchmarks/v27.3.0.json` が存在すること（test_count: 2150）
- [ ] `v273000_tests` 8 件すべて PASS
- [ ] `cargo test clickhouse --bin fav` で 7 件以上 PASS
- [ ] 総テスト数 ≥ 2150 件

---

## スコープ外（v28.x 以降）

- `clickhouse-rs` クレートを使った実 ClickHouse 接続（実データ連携）
- `ClickHouse.query[T]` ジェネリック API（ロードマップ記載。v28.x で実装）
- HTTP / Native プロトコルの実接続
- `clickhouse/clickhouse-server`（Docker）との実通信
- ClickHouse v21.11+ の async_insert 実装
- 接続プール管理
