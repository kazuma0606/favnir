# v27.7.0 実装計画 — `fav infer --from delta` / `--from iceberg`

## フェーズ構成

### Phase 0: 事前確認
- `Cargo.toml` が `27.6.0` であること
- テスト数が 2186 件であること
- `vm.rs` に `DeltaLake.infer_schema_raw` / `Iceberg.infer_schema_raw` がないことを確認
- `driver.rs` に `cmd_infer_delta` がないことを確認
- `cargo test infer_delta --bin fav` のベースライン件数（0 件）を記録

### Phase 1: Cargo.toml バージョン bump
- `fav/Cargo.toml`: `version = "27.6.0"` → `"27.7.0"`

### Phase 2: vm.rs — 新 primitive 2 件追加
- `DeltaLake.infer_schema_raw(path: String)` — 既存 DeltaLake ブロック末尾（`DeltaLake.optimize_raw` の wasm32 アーム直後、行 17751 付近）に追加
- `Iceberg.infer_schema_raw(catalog: String, table: String)` — 既存 Iceberg ブロック末尾（`Iceberg.list_snapshots_raw` の wasm32 アーム直後、行 17828 付近）に追加
- 両方とも `#[cfg(not(target_arch = "wasm32"))]` / `#[cfg(target_arch = "wasm32")]` ガード付き

### Phase 3: driver.rs — 型マッピング関数 + 2 コマンド関数追加
- `fn delta_type_to_favnir(t: &str) -> &str` — match 型マッピング
- `pub fn cmd_infer_delta(path: &str, out_path: Option<&str>)` — DeltaLake.infer_schema_raw 呼び出し stub
- `pub fn cmd_infer_iceberg(catalog: &str, table: &str, out_path: Option<&str>)` — Iceberg.infer_schema_raw 呼び出し stub

### Phase 4: main.rs — `--path` / `--catalog` フラグと dispatch 追加
- `--path` フラグ（delta 用）を `infer` アームのフラグパースに追加
- `--catalog` フラグ（iceberg 用）を追加（`--table` は既存実装済みのため追加不要）
- `use driver::{..., cmd_infer_delta, cmd_infer_iceberg}` のインポートを追加
- `--from delta` / `--from iceberg` dispatch を追加（既存 snowflake パターンに倣う）

### Phase 5: ドキュメント
- `site/content/docs/` に `fav infer --from delta/iceberg` の MDX ドキュメント追加

### Phase 6: CHANGELOG 更新
- `CHANGELOG.md` 先頭に `[v27.7.0]` エントリ追加

### Phase 7: ベンチマーク JSON 作成
- `benchmarks/v27.7.0.json` 新規作成（test_count: 2194）

### Phase 8: driver.rs テスト追加
- `v277000_tests`（9 件）を `v276000_tests` の直後に追加

### Phase 9: 全テスト実行・確認
- `cargo test v277000 --bin fav` — 8/8 PASS 確認
- `cargo test infer_delta --bin fav` — PASS 確認
- `cargo test --bin fav` — 2194 件 PASS 確認（リグレッションなし）

## 注意事項

- v27.7.0 は stub 実装。`infer_schema_raw` は引数検証のみ行い、固定スキーマ JSON `"[]"` を返す
- 実際の DeltaLake / Iceberg スキーマ読み取りは v28.x（`delta-rs` / `iceberg-rust` 統合時）に延期
- `delta_type_to_favnir` は spec の型マッピング表に従う
- `--path` フラグは既存 `--db` / `--proto` と同じパース方式で追加
- checker.fav 更新不要（v27.7.0 は CLI 拡張のみ。DeltaLake/Iceberg は ns_to_effect 未登録だが、新 Rune 関数なしのためエフェクト追跡に影響しない）
