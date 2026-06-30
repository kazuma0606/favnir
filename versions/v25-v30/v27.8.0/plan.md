# v27.8.0 実装計画 — dbt 連携 Rune

## フェーズ構成

### Phase 0: 事前確認
- `Cargo.toml` が `27.7.0` であること
- テスト数が 2195 件であること
- `vm.rs` に `Dbt.ref_raw` がないことを確認
- `runes/dbt/` が存在しないことを確認
- `cargo test dbt --bin fav` のベースライン件数（0 件）を記録
- `fav/self/checker.fav` の `ns_to_effect` に `"Dbt"` がないことを確認（grep）

### Phase 1: Cargo.toml バージョン bump
- `fav/Cargo.toml`: `version = "27.7.0"` → `"27.8.0"`

### Phase 2: vm.rs — 新 primitive 2 件追加
- `Dbt.ref_raw(config, model_name)` — JSONL ブロック末尾（`JSONL.append_raw` wasm32 アーム直後、行 18013 付近）に追加
- `Dbt.source_raw(config, source_name, table_name)` — `Dbt.ref_raw` の直後に追加
- 両方とも `#[cfg(not(target_arch = "wasm32"))]` / `#[cfg(target_arch = "wasm32")]` ガード付き
- Azure Blob Storage ブロック直前

### Phase 3: runes/dbt/dbt.fav — 新規作成
- `public fn ref(config, model_name) -> Result<String, String> !Db`
- `public fn source(config, source_name, table_name) -> Result<String, String> !Db`
- stub コメント（v28.x 延期注記）

### Phase 4: examples/dbt_pipeline.fav — 新規作成
- `LoadCustomerSummary` / `LoadRawEvents` stage
- `seq DbtRefPipeline` パイプライン定義

### Phase 5: fav/tests/fixtures/dbt_manifest.json — 新規作成
- `"nodes"` / `"sources"` キーを持つ最小フィクスチャ

### Phase 6: ドキュメント
- `site/content/docs/runes/dbt.mdx` 新規作成

### Phase 7: CHANGELOG 更新
- `CHANGELOG.md` 先頭に `[v27.8.0]` エントリ追加

### Phase 8: ベンチマーク JSON 作成
- `benchmarks/v27.8.0.json` 新規作成（test_count: 2203）

### Phase 9a: checker.fav 更新（Phase 9b より先に実施）
- `fav/self/checker.fav` の `ns_to_effect` に `"Dbt" => "Db"` を追加
- `"JSONL" => "IO"` ブロックの直後（`else { "" }` の直前）に挿入
- **必ず Phase 9b（テスト実行）より前に完了すること**（v27.6.0 の教訓）

### Phase 9b: driver.rs テスト追加 + 全テスト実行
- `v278000_tests`（8 件）を `v277000_tests` の直前に追加
- `cargo test v278000 --bin fav` — 8/8 PASS 確認
- `cargo test dbt --bin fav` — 7 件 PASS 確認
- `cargo test --bin fav` — 2203 件 PASS 確認（リグレッションなし）

## 注意事項

- v27.8.0 は stub 実装。`ref_raw` / `source_raw` は引数検証のみ行い、`"[]"` を返す
- `DbtConfig` 構造体型は v28.x に延期（stub では config: String でパス文字列を渡す）
- **Phase 10 は Phase 9 より先に実施**（v27.6.0 の教訓: checker.fav を後回しにすると ns_to_effect BUG になる）
- `Dbt` namespace は新規のため、`ns_to_effect` への登録は必須
- vm.rs の挿入位置: JSONL 末尾（行 18013）直後、Azure Blob 直前（行 18015）
