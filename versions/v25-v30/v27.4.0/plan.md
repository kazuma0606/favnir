# v27.4.0 実装計画 — bigquery Rune 実質化

## 実装順序

### Phase 1: 既存ファイル確認
- `runes/bigquery/bigquery.fav`（v15.2.0 旧実装）の内容確認
- `fav/src/backend/vm.rs` の既存 `BigQuery.*` primitive（`BigQuery.query_raw` / `BigQuery.execute_raw`）の位置確認
- ClickHouse ブロック末尾の位置確認（挿入ポイント）

### Phase 2: VM primitive 追加（vm.rs）
挿入位置: 既存 BigQuery ブロック末尾（`"BigQuery.infer_table_raw"` の `}` 直後、約行 17225）、Kafka/MSK ブロック（`// ── Kafka / MSK primitives (v15.4.0)`）直前。既存 `BigQuery.query_raw`/`execute_raw`/`infer_table_raw` と同一ブロック内にまとめる。
各 primitive は ClickHouse と同パターンで `#[cfg(not(target_arch = "wasm32"))]` / `#[cfg(target_arch = "wasm32")]` 両アームを追加する。

新 5 primitives（既存 `BigQuery.query_raw` / `BigQuery.execute_raw` は削除しない）:
1. `BigQuery.connect_raw` — `config: String` 検証、`"bigquery-stub-conn"` 返却
2. `BigQuery.conn_query_raw` — `conn: String` / `sql: String` 検証、`"[]"` 返却
3. `BigQuery.insert_raw` — `conn` / `table` / `rows` 検証、`ok_vm(VMValue::Unit)` 返却
4. `BigQuery.load_from_gcs_raw` — `conn` / `table` / `gcs_uri` / `format` 検証、`ok_vm(VMValue::Unit)` 返却
5. `BigQuery.create_table_raw` — `conn` / `table` / `schema` 検証、`ok_vm(VMValue::Unit)` 返却

### Phase 3: runes/bigquery/bigquery.fav 置換
v15.2.0 の旧実装（`!Gcp` エフェクト、非 `public` fn）を新 API（`!Db` エフェクト、`public fn`、5 関数）に全置換。

### Phase 4: examples/bigquery_analytics.fav 新規作成
`BigQueryAnalyticsPipeline = CreateEventTable |> LoadFromGcs |> QueryStats`

### Phase 5: ドキュメント
`site/content/docs/runes/bigquery.mdx` 新規作成（clickhouse.mdx を参考に）

### Phase 6: CHANGELOG.md 更新
`[v27.4.0]` エントリを先頭に追加

### Phase 7: driver.rs テスト追加（12 件）
`v274000_tests` を `v273000_tests` の直後に追加

### Phase 8: Cargo.toml バージョン bump
`version = "27.4.0"`

### Phase 9: benchmarks/v27.4.0.json 新規作成
`{"version":"27.4.0","test_count":2164,"timestamp":"2026-06-27"}`

## 依存関係

```
Phase 1（確認）
    → Phase 2（vm.rs）
    → Phase 3（runes）
    → Phase 4（examples）
    → Phase 5（docs）
    → Phase 6（CHANGELOG）
    → Phase 7（driver.rs）
    → Phase 8（Cargo.toml）
    → Phase 9（benchmarks）
```

## テスト数計算

| バージョン | テスト数 |
|---|---|
| v27.3.0 完了後 | 2152 |
| v274000_tests 追加 | +12 |
| **v27.4.0 合計** | **2164** |

## リスク・注意点

- `BigQuery.query_raw`（4 引数: v15.2.0）と `BigQuery.execute_raw` は vm.rs に**残す**（既存テスト互換）
- 新 5 primitives は別名（`conn_query_raw` 等）を使い名前衝突を回避
- `runes/bigquery/bigquery.fav` は**全置換**（追記ではなく上書き）
- `!Gcp` エフェクトを `!Db` に変更（DWH 統一方針）
- `connect_raw` の TODO: v28.x で google-cloud-bigquery クレートの統合時に `_config` を実接続に渡す旨を明記
