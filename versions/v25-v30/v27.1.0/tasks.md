# v27.1.0 タスクリスト — delta-lake Rune 追加

**状態**: COMPLETE
**開始日**: 2026-06-27
**完了日**: 2026-06-27

---

## タスク

| ID | タスク | 状態 |
|---|---|---|
| T0 | 事前確認: `Cargo.toml` が `27.0.0`、テスト数 2122 件、`runes/delta-lake/` が存在しない、`vm.rs` に `DeltaLake.read_raw` がない、`examples/delta_lake_etl.fav` が存在しないことを確認 | [x] |
| T1 | `fav/Cargo.toml` を `version = "27.1.0"` に bump | [x] |
| T2 | `runes/delta-lake/delta-lake.fav` を新規作成（7 関数: read / read_with_filter / write / merge / history / vacuum / optimize） | [x] |
| T3 | `fav/src/backend/vm.rs` に `DeltaLake.*_raw` primitive 7 件追加（Pulsar ブロック直後・Azure Blob 直前、`#[cfg]` ガード付き） | [x] |
| T4 | `examples/delta_lake_etl.fav` 新規作成（LoadRawData \|> TransformOrders \|> SaveProcessed） | [x] |
| T5 | `site/content/docs/runes/delta-lake.mdx` 新規作成 | [x] |
| T6 | `CHANGELOG.md` 更新: 先頭に `[v27.1.0]` エントリ追加 | [x] |
| T7 | `benchmarks/v27.1.0.json` 新規作成（test_count: 2130） | [x] |
| T8 | `fav/src/driver.rs` 更新: `v271000_tests`（8 件）を `v270000_tests` の直後に追加 | [x] |
| T8.5 | `cargo test v271000 --bin fav` — 8/8 PASS 確認 | [x] |
| T8.6 | `cargo test delta_lake --bin fav` — 7 件以上 PASS 確認 | [x] |
| T9 | `cargo test --bin fav` — 2130 件 PASS 確認（リグレッションなし） | [x] |
| T10 | spec-reviewer レビュー実施（実装前） | [x] |

---

## チェックリスト（完了条件）

- [x] `fav/Cargo.toml` が `version = "27.1.0"` であること
- [x] `runes/delta-lake/delta-lake.fav` に `fn read(` が含まれること
- [x] `runes/delta-lake/delta-lake.fav` に `fn write(` が含まれること
- [x] `runes/delta-lake/delta-lake.fav` に `fn merge(` が含まれること
- [x] `runes/delta-lake/delta-lake.fav` に `fn history(` が含まれること
- [x] `runes/delta-lake/delta-lake.fav` に `fn vacuum(` が含まれること
- [x] `runes/delta-lake/delta-lake.fav` に `fn optimize(` が含まれること
- [x] `fav/src/backend/vm.rs` に `DeltaLake.read_raw` が含まれること
- [x] `fav/src/backend/vm.rs` に `DeltaLake.write_raw` が含まれること
- [x] `fav/src/backend/vm.rs` に `DeltaLake.optimize_raw` が含まれること
- [x] `examples/delta_lake_etl.fav` に `DeltaEtlPipeline` が含まれること
- [x] `site/content/docs/runes/delta-lake.mdx` が存在すること
- [x] `CHANGELOG.md` に `[v27.1.0]` エントリが存在すること
- [x] `benchmarks/v27.1.0.json` が存在すること（test_count: 2130）
- [x] `v271000_tests` 8 件すべて PASS
- [x] `runes/delta-lake/delta-lake.fav` に `fn read_with_filter(` が含まれること
- [x] `cargo test delta_lake --bin fav` で 7 件以上 PASS
- [x] 総テスト数 ≥ 2130 件

---

## メモ

### vm.rs 挿入位置

Pulsar ブロック末尾（`"Pulsar.nack_raw" => Ok(err_vm(...))` の直後）に挿入。
Azure Blob ブロック（`// ── Azure Blob Storage primitives (v14.5.0)`）の直前。

### write_raw のバリデーション

`mode` が `"append"` でも `"overwrite"` でもない場合は `err_vm` を返す。
これは stub でも有効な軽量バリデーション（実 delta-rs 実装時も同条件）。

### vacuum_raw の下限チェック

`retention_hours < 168` の場合は `err_vm`（Delta Lake 公式仕様: 最小 7 日）。

### `delta-rs` クレートを追加しない理由

- WASM 互換性への影響（`delta-rs` は `tokio` 依存、WASM では動かない）
- ビルド時間の増大（`deltalake` クレートはビルドが重い）
- 現バージョンは stub で基盤を整え、v28.x で実統合する

### include_str! パス

| パス | 解決先 |
|---|---|
| `../../runes/delta-lake/delta-lake.fav` | `favnir/runes/delta-lake/delta-lake.fav` |
| `backend/vm.rs` | `fav/src/backend/vm.rs` |
| `../../examples/delta_lake_etl.fav` | `favnir/examples/delta_lake_etl.fav` |
| `../../CHANGELOG.md` | `favnir/CHANGELOG.md` |

---

## コードレビュー指摘（実装後に記入）

| 指摘 | 対応 |
|---|---|
| [MED] vm.rs: `write_raw` / `vacuum_raw` のエラーメッセージで `_raw` suffix 欠落 | `"DeltaLake.write_raw:"` / `"DeltaLake.vacuum_raw:"` に修正 |
| [MED] v271000_tests に `optimize_fn` / `read_with_filter_fn` テスト欠落 | 2 テスト追加（計 10 件、test_count 2130 → 2132） |
| [LOW] `delta_lake_etl.fav` の `TransformOrders` に `!Pure` が不要 | `!Pure` を削除（`delta-lake.mdx` のサンプルも同様に修正） |
