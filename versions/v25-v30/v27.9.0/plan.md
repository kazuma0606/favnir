# v27.9.0 実装計画 — sqlite Rune 追加

## フェーズ構成

### Phase 0: 事前確認
- `Cargo.toml` が `27.8.0` であること
- テスト数が 2204 件であること
- `vm.rs` に `SQLite.open_raw` がないことを確認
- `runes/sqlite/` が存在しないことを確認
- `grep "SQLite" fav/self/checker.fav` で ns_to_effect に未登録を確認
- `cargo test sqlite --bin fav` のベースライン件数（0 件）を記録

### Phase 1: Cargo.toml バージョン bump
- `fav/Cargo.toml`: `version = "27.8.0"` → `"27.9.0"`

### Phase 2: vm.rs — 新 primitive 6 件追加
- Dbt ブロック末尾（`"Dbt.source_raw"` の wasm32 アーム直後、行 18040 付近）に追加
- Azure Blob Storage ブロック直前
- 追加順: `open_raw` → `open_memory_raw` → `query_raw` → `execute_raw` → `execute_many_raw` → `close_raw`
- 全 primitive に `#[cfg(not(target_arch = "wasm32"))]` / `#[cfg(target_arch = "wasm32")]` ガード付き
- 戻り値: open/open_memory → ハンドル文字列、query → `"[]"`、execute/execute_many → `Int(0)`、close → `Unit`

### Phase 3: runes/sqlite/sqlite.fav — 新規作成
- 6 関数: open / open_memory / query / execute / execute_many / close
- 全関数に `!Db` エフェクト
- 機能説明コメント付き

### Phase 4: examples/sqlite_etl.fav — 新規作成
- `CreateTable` / `InsertRows` stage
- `seq SqliteEtlPipeline = CreateTable` パイプライン定義

### Phase 5: ドキュメント
- `site/content/docs/runes/sqlite.mdx` 新規作成

### Phase 6: CHANGELOG 更新
- `CHANGELOG.md` 先頭に `[v27.9.0]` エントリ追加

### Phase 7: ベンチマーク JSON 作成
- `benchmarks/v27.9.0.json` 新規作成（test_count: 2220）

> **Note**: Phase 8 は欠番（本バージョンでは該当なし）。実施順: Phase 1〜7 → Phase 9a → Phase 9b

### Phase 9a: checker.fav 更新（Phase 9b より先に実施）
> **依存関係**: Phase 9b（テスト実行）は Phase 9a 完了後に実施すること。逆順は BUG の原因となる（v27.6.0 教訓）。
- `fav/self/checker.fav` の `ns_to_effect` に `"SQLite" => "Db"` を追加
- `"Dbt" => "Db"` ブロックの直後（`else { "" }` の直前）に挿入
- **必ず Phase 9b（テスト実行）より前に完了すること**

### Phase 9b: driver.rs テスト追加 + 全テスト実行
- `v279000_tests`（16 件）を `v278000_tests` の直前に追加
- `cargo test v279000 --bin fav` — 16/16 PASS 確認
- `cargo test sqlite --bin fav` — 15 件 PASS 確認
- `cargo test --bin fav` — 2220 件 PASS 確認（リグレッションなし）

## 注意事項

- `open_memory_raw` は引数なし primitive — `args.into_iter()` を使わず直接 stub 値を返す
- `execute_raw` / `execute_many_raw` の戻り値は `Int(0)`（影響行数）— `Unit` ではない
- Dbt wasm32 アームは行 18040 付近 → SQLite は行 18042 以降に挿入
- checker.fav の `"SQLite" => "Db"` は `"Dbt" => "Db"` ブロックの else 内・`else { "" }` 直前
