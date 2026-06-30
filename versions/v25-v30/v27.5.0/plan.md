# v27.5.0 実装計画 — redshift Rune 追加

## 実装順序

### Phase 1: 事前確認
- `fav/Cargo.toml` が `27.4.0` であること
- `vm.rs` に `Redshift.*` primitive がないこと
- `runes/redshift/` が存在しないこと
- `cargo test redshift --bin fav` のベースライン件数を記録（0 件であること）

### Phase 2: Cargo.toml バージョン bump
`version = "27.5.0"`

### Phase 3: VM primitive 追加（vm.rs）
挿入位置: ClickHouse ブロック末尾（`"ClickHouse.async_insert_raw" => Ok(err_vm(...))` の wasm32 アーム直後、行 17878 付近）、Azure Blob Storage ブロック（`// ── Azure Blob Storage primitives (v14.5.0)`）直前。
各 primitive は `#[cfg(not(target_arch = "wasm32"))]` / `#[cfg(target_arch = "wasm32")]` 両アームで追加する。

新 5 primitives:
1. `Redshift.connect_raw` — `config: String` 検証、`"redshift-stub-conn"` 返却
2. `Redshift.query_raw` — `conn: String` / `sql: String` 検証、`"[]"` 返却
3. `Redshift.execute_raw` — `conn` / `sql` / `params` 検証、`ok_vm(VMValue::Int(0))` 返却
4. `Redshift.copy_from_s3_raw` — `conn` / `table` / `s3_uri` / `opts` 検証、`ok_vm(VMValue::Unit)` 返却
5. `Redshift.unload_to_s3_raw` — `conn` / `query` / `s3_uri` / `opts` 検証、`ok_vm(VMValue::Unit)` 返却

### Phase 4: runes/redshift/redshift.fav 新規作成
5 関数（connect / query / execute / copy_from_s3 / unload_to_s3）、すべて `public fn` + `!Db` エフェクト。

### Phase 5: examples/redshift_analytics.fav 新規作成
`RedshiftAnalyticsPipeline = LoadFromS3 |> QuerySummary |> UnloadToS3`

### Phase 6: ドキュメント
`site/content/docs/runes/redshift.mdx` 新規作成（clickhouse.mdx / bigquery.mdx を参考に）

### Phase 7: CHANGELOG.md 更新
`[v27.5.0]` エントリを先頭に追加

### Phase 8: benchmarks/v27.5.0.json 新規作成
`{"version":"27.5.0","test_count":2176,"timestamp":"2026-06-27"}`

### Phase 9: driver.rs テスト追加（12 件）
`v275000_tests` を `v274000_tests` の直後に追加

## 依存関係

```
Phase 1（確認）
    → Phase 2（Cargo.toml）
    → Phase 3（vm.rs）
    → Phase 4（runes）
    → Phase 5（examples）、Phase 6（docs）  ← Phase 4 完了後、並列可能
    → Phase 7（CHANGELOG）
    → Phase 8（benchmarks）
    → Phase 9（driver.rs）
```

## テスト数計算

| バージョン | テスト数 |
|---|---|
| v27.4.0 完了後 | 2164 |
| v275000_tests 追加 | +12 |
| **v27.5.0 合計** | **2176** |

## リスク・注意点

- `Redshift.execute_raw` の戻り値は `Result<Int, String>` → `ok_vm(VMValue::Int(0))` を返す（Unit でなく Int）
- `connect_raw` の TODO: v28.x で postgres クレートの統合時に `_config` を実接続に渡す旨を明記
- `copy_from_s3_raw` / `unload_to_s3_raw` はそれぞれ 4 引数（conn / table(or query) / s3_uri / opts）
- `runes/redshift/` ディレクトリは新規作成（既存ファイルなし）
