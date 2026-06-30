# v27.4.0 仕様書 — bigquery Rune 実質化

## 概要

v15.2.0 で部分実装された BigQuery Rune を完全実装に昇格する。
`import rune "bigquery"` → `BigQuery.*` 名前空間に 5 関数を追加（connect / query / insert / load_from_gcs / create_table）。

---

## 背景

ロードマップ v27.4「bigquery Rune 実質化」より。Data Lakehouse フェーズの第 4 コンポーネント。
v15.2.0 の旧 API（`fn query(project_id, dataset, sql)` / `!Gcp` エフェクト）を廃止し、
他 DWH Rune（ClickHouse / Redshift）と統一した connect-based API（`!Db` エフェクト）に移行する。

### 既存実装との差分

| 項目 | v15.2.0（旧） | v27.4.0（新） |
|---|---|---|
| エフェクト | `!Gcp` | `!Db`（DWH 統一） |
| API スタイル | `fn query(project_id, dataset, sql)` | `fn connect(config)` + `fn query(conn, sql)` |
| 公開フラグ | `fn`（非 `public`） | `public fn` |
| VM primitives | `BigQuery.query_raw`（4 args）/ `BigQuery.execute_raw` | 新 5 primitives（名前衝突を避けるため別名） |
| ヘルパー関数 | `fn extract_rows(raw_json: String)` | 削除（非 `public` のため外部影響なし。`bigquery.fav` を全置換により除去） |

> **既存 primitive の扱い**: `BigQuery.query_raw`（v15.2.0）と `BigQuery.execute_raw` は vm.rs に残す（既存テスト互換）。新 API は別名 primitive を追加する。

ロードマップ要件:
- 5 関数実装（connect / query / insert / load_from_gcs / create_table）
- BigQuery Emulator（`ghcr.io/goccy/bigquery-emulator`）で `cargo test bigquery` が 4 件以上 PASS

---

## 実装する関数

| 関数 | シグネチャ | 内容 |
|---|---|---|
| `BigQuery.connect` | `(config: String) -> Result<String, String> !Db` | 接続確立（service account JSON 等）。接続ハンドルを返す |
| `BigQuery.query` | `(conn: String, sql: String) -> Result<String, String> !Db` | 型付きクエリ（SELECT）。JSON 配列文字列を返す |
| `BigQuery.insert` | `(conn: String, table: String, rows: String) -> Result<Unit, String> !Db` | streaming insert（即時反映、小量向き） |
| `BigQuery.load_from_gcs` | `(conn: String, table: String, gcs_uri: String, format: String) -> Result<Unit, String> !Db` | GCS からバルクロード（大量向き） |
| `BigQuery.create_table` | `(conn: String, table: String, schema: String) -> Result<Unit, String> !Db` | テーブル作成（スキーマ定義 JSON） |

> **エフェクト**: `!Db`（ClickHouse / Redshift と統一。v15.2 の `!Gcp` から変更）

---

## VM Primitive（vm.rs に追加）

既存 `BigQuery.query_raw`（v15.2）・`BigQuery.execute_raw` との名前衝突を避けるため、新 5 primitives は別名とする。

| primitive 名 | 実装方針 |
|---|---|
| `BigQuery.connect_raw` | stub: 引数検証のみ、`"bigquery-stub-conn"` 返却（クレート統合は v28.x）。接続ハンドル識別子を返すため固有の stub 文字列を使用（ClickHouse と同一パターン） |
| `BigQuery.conn_query_raw` | stub: 引数検証（conn / sql）、`"[]"` 返却 |
| `BigQuery.insert_raw` | stub: 引数検証（conn / table / rows）、`ok_vm(VMValue::Unit)` 返却 |
| `BigQuery.load_from_gcs_raw` | stub: 引数検証（conn / table / gcs_uri / format）、`ok_vm(VMValue::Unit)` 返却 |
| `BigQuery.create_table_raw` | stub: 引数検証（conn / table / schema）、`ok_vm(VMValue::Unit)` 返却 |

> **挿入位置**: 既存 BigQuery ブロック末尾（`"BigQuery.infer_table_raw"` の `}` 直後、約行 17225）・Kafka/MSK ブロック（`// ── Kafka / MSK primitives (v15.4.0)`）の直前。既存 `BigQuery.query_raw`・`BigQuery.execute_raw`・`BigQuery.infer_table_raw` と同一ブロック内にまとめることでコードの可読性を維持する。
>
> **wasm32 ガード**: ClickHouse と同パターン（`#[cfg(not(target_arch = "wasm32"))]` / `#[cfg(target_arch = "wasm32")]` 両アーム）で追加する。wasm32 アームは `err_vm("BigQuery not supported on wasm32")` を返す。

---

## runes/bigquery/bigquery.fav（既存ファイルを置換）

```favnir
// runes/bigquery/bigquery.fav — BigQuery Rune (v27.4.0)
// v15.2.0 の部分実装（query/execute、!Gcp エフェクト）から connect-based API（!Db エフェクト）に昇格。
// Google BigQuery クライアントライブラリ統合は v28.x 以降。現バージョンは引数検証 stub。
// TODO(v28.x): google-cloud-bigquery クレートを使った実接続に移行予定。
public fn connect(config: String) -> Result<String, String> !Db {
    BigQuery.connect_raw(config)
}
public fn query(conn: String, sql: String) -> Result<String, String> !Db {
    BigQuery.conn_query_raw(conn, sql)
}
public fn insert(conn: String, table: String, rows: String) -> Result<Unit, String> !Db {
    BigQuery.insert_raw(conn, table, rows)
}
public fn load_from_gcs(conn: String, table: String, gcs_uri: String, format: String) -> Result<Unit, String> !Db {
    BigQuery.load_from_gcs_raw(conn, table, gcs_uri, format)
}
public fn create_table(conn: String, table: String, schema: String) -> Result<Unit, String> !Db {
    BigQuery.create_table_raw(conn, table, schema)
}
```

---

## examples/bigquery_analytics.fav

```favnir
// examples/bigquery_analytics.fav — BigQuery Analytics デモ (v27.4.0)
import rune "bigquery"

stage CreateEventTable: Unit -> Result<Unit, String> !Db = |_| {
    bind conn <- BigQuery.connect("project:my-project,dataset:analytics")
    BigQuery.create_table(conn, "events", "{\"fields\":[{\"name\":\"id\",\"type\":\"INTEGER\"}]}")
}

// seq pipeline は前ステージの成功値（Unit）を次ステージの引数として渡す
stage LoadFromGcs: Unit -> Result<Unit, String> !Db = |_| {
    bind conn <- BigQuery.connect("project:my-project,dataset:analytics")
    BigQuery.load_from_gcs(conn, "events", "gs://my-bucket/events/*.parquet", "PARQUET")
}

stage QueryStats: Unit -> Result<String, String> !Db = |_| {
    bind conn <- BigQuery.connect("project:my-project,dataset:analytics")
    BigQuery.query(conn, "SELECT count(*) FROM analytics.events")
}

seq BigQueryAnalyticsPipeline = CreateEventTable |> LoadFromGcs |> QueryStats
```

---

## テスト

### driver.rs v274000_tests（12 件）

| テスト名 | 内容 |
|---|---|
| `bigquery_rune_has_connect_fn` | `bigquery.fav` に `"fn connect("` が含まれること |
| `bigquery_rune_has_query_fn` | `bigquery.fav` に `"fn query("` が含まれること |
| `bigquery_rune_has_insert_fn` | `bigquery.fav` に `"fn insert("` が含まれること |
| `bigquery_rune_has_load_from_gcs_fn` | `bigquery.fav` に `"fn load_from_gcs("` が含まれること |
| `bigquery_rune_has_create_table_fn` | `bigquery.fav` に `"fn create_table("` が含まれること |
| `bigquery_rune_vm_has_connect_raw` | `vm.rs` に `"BigQuery.connect_raw"` が含まれること |
| `bigquery_rune_vm_has_conn_query_raw` | `vm.rs` に `"BigQuery.conn_query_raw"` が含まれること |
| `bigquery_rune_vm_has_insert_raw` | `vm.rs` に `"BigQuery.insert_raw"` が含まれること |
| `bigquery_rune_vm_has_load_from_gcs_raw` | `vm.rs` に `"BigQuery.load_from_gcs_raw"` が含まれること |
| `bigquery_rune_vm_has_create_table_raw` | `vm.rs` に `"BigQuery.create_table_raw"` が含まれること |
| `bigquery_example_has_pipeline` | `examples/bigquery_analytics.fav` に `"BigQueryAnalyticsPipeline"` が含まれること |
| `changelog_has_v27_4_0` | `CHANGELOG.md` に `"[v27.4.0]"` が含まれること |

### `cargo test bigquery` 期待値

- `v274000_tests::bigquery_rune_has_*` 5 件
- `v274000_tests::bigquery_rune_vm_has_*` 5 件
- `v274000_tests::bigquery_example_has_pipeline` 1 件
- 合計 11 件（`changelog_has_v27_4_0` は `bigquery` を含まないため除外）（ロードマップ要件「4 件以上」超過）

---

## 完了条件

- [ ] `fav/Cargo.toml` が `version = "27.4.0"` であること
- [ ] `runes/bigquery/bigquery.fav` に `public fn connect(` が含まれること
- [ ] `runes/bigquery/bigquery.fav` に `public fn query(` が含まれること
- [ ] `runes/bigquery/bigquery.fav` に `public fn insert(` が含まれること
- [ ] `runes/bigquery/bigquery.fav` に `public fn load_from_gcs(` が含まれること
- [ ] `runes/bigquery/bigquery.fav` に `public fn create_table(` が含まれること
- [ ] `fav/src/backend/vm.rs` に `BigQuery.connect_raw` が含まれること
- [ ] `fav/src/backend/vm.rs` に `BigQuery.conn_query_raw` が含まれること
- [ ] `fav/src/backend/vm.rs` に `BigQuery.insert_raw` が含まれること
- [ ] `fav/src/backend/vm.rs` に `BigQuery.load_from_gcs_raw` が含まれること
- [ ] `fav/src/backend/vm.rs` に `BigQuery.create_table_raw` が含まれること
- [ ] `examples/bigquery_analytics.fav` に `BigQueryAnalyticsPipeline` が含まれること
- [ ] `site/content/docs/runes/bigquery.mdx` が存在すること
- [ ] `CHANGELOG.md` に `[v27.4.0]` エントリが存在すること
- [ ] `benchmarks/v27.4.0.json` が存在すること（test_count: 2164）
- [ ] `v274000_tests` 12 件すべて PASS
- [ ] `cargo test bigquery --bin fav` で 11 件以上 PASS
- [ ] 総テスト数 ≥ 2164 件

---

## スコープ外（v28.x 以降）

- `google-cloud-bigquery` クレートを使った実 BigQuery 接続（実データ連携）
- `BigQuery.query[T]` ジェネリック API（ロードマップ記載。v28.0 マイルストーン条件）
  - **延期根拠**: v27.4.0 は stub 実装段階であり、VM レベルでジェネリック型引数の評価ができないため v28.0 に延期する。非ジェネリック `query(conn, sql) -> Result<String, String>` で JSON 文字列を返す API をデフォルトとし、v28.x で型付き `query[T]` に昇格する。
- service account JSON の実認証
- BigQuery Emulator（`ghcr.io/goccy/bigquery-emulator`）との実通信
- `load_from_gcs` の format バリデーション（CSV / PARQUET / AVRO / ORC / NEWLINE_DELIMITED_JSON）
- streaming insert の quota 制限対応
