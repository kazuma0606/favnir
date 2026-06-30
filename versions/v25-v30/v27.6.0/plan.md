# v27.6.0 実装計画 — jsonl Rune 追加

## 実装順序

### Phase 1: 事前確認
- `fav/Cargo.toml` が `27.5.0` であること
- `vm.rs` に `JSONL.*` primitive がないこと
- `runes/jsonl/` が存在しないこと
- `cargo test jsonl --bin fav` のベースライン件数を記録（0 件であること）

### Phase 2: Cargo.toml バージョン bump
`version = "27.6.0"`

### Phase 3: VM primitive 追加（vm.rs）
挿入位置: Redshift ブロック末尾（`"Redshift.unload_to_s3_raw" => Ok(err_vm(...))` の wasm32 アーム直後、行 17942 付近）、Azure Blob Storage ブロック直前。
各 primitive は `#[cfg(not(target_arch = "wasm32"))]` / `#[cfg(target_arch = "wasm32")]` 両アームで追加する。

新 4 primitives:
1. `JSONL.read_raw` — `path: String` 検証、`ok_vm(VMValue::Str("[]".into()))` 返却
2. `JSONL.write_raw` — `path: String` / `rows: String` 検証、`ok_vm(VMValue::Unit)` 返却
3. `JSONL.stream_raw` — `path: String` 検証、`ok_vm(VMValue::Str("[]".into()))` 返却
4. `JSONL.append_raw` — `path: String` / `row: String` 検証、`ok_vm(VMValue::Unit)` 返却

### Phase 4: runes/jsonl/jsonl.fav 新規作成
4 関数（read / write / stream / append）、すべて `public fn` + `!Io` エフェクト。

### Phase 5: examples/jsonl_etl.fav 新規作成
`JsonlEtlPipeline = ReadData |> WriteProcessed`

### Phase 6: ドキュメント（Phase 4 完了後、Phase 5 と並列可能）
`site/content/docs/runes/jsonl.mdx` 新規作成（delta-lake.mdx / clickhouse.mdx を参考に）

### Phase 7: CHANGELOG.md 更新
`[v27.6.0]` エントリを先頭に追加

### Phase 8: benchmarks/v27.6.0.json 新規作成
`{"version":"27.6.0","test_count":2186,"timestamp":"2026-06-27"}`

### Phase 9: driver.rs テスト追加（10 件）
`v276000_tests` を `v275000_tests` の直後に追加

### Phase 10: checker.fav 更新
`ns_to_effect` に `"JSONL" => "IO"` を追加（`!Io` エフェクト検証のため）

## 依存関係

```
Phase 1（確認）
    → Phase 2（Cargo.toml）
    → Phase 3（vm.rs）
    → Phase 4（runes）
    → Phase 5（examples）、Phase 6（docs）  ← Phase 4 完了後、並列可能
    → Phase 7（CHANGELOG）
    → Phase 10（checker.fav）  ← Phase 9（テスト確認）より前に必ず完了すること
    → Phase 8（benchmarks）
    → Phase 9（driver.rs + 全テスト確認）
```

## テスト数計算

| バージョン | テスト数 |
|---|---|
| v27.5.0 完了後 | 2176 |
| v276000_tests 追加 | +10 |
| **v27.6.0 合計** | **2186** |

## リスク・注意点

- `JSONL.read_raw` と `JSONL.stream_raw` は同じシグネチャ（path: String）で同じ戻り値（`"[]"`）になる stub。実装時に混同しないこと
- `checker.fav` の `ns_to_effect` に `"JSONL"` を追加することで v27.5.0 の [BUG] と同様の漏れを防ぐ（v27.5.0 でこのパターンを教訓として得た）
- `runes/jsonl/` ディレクトリは新規作成（既存ファイルなし）
- `!Io` エフェクト: DeltaLake / Iceberg / fs と同じ（`!Db` ではない点に注意）
