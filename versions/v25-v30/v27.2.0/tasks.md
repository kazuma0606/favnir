# v27.2.0 タスクリスト — iceberg Rune 追加

**状態**: COMPLETE
**開始日**: 2026-06-27
**完了日**: 2026-06-27

---

## タスク

| ID | タスク | 状態 |
|---|---|---|
| T0 | 事前確認: `Cargo.toml` が `27.1.0`、テスト数 2132 件、`runes/iceberg/` が存在しない、`vm.rs` に `Iceberg.read_raw` がない、`examples/iceberg_etl.fav` が存在しないことを確認 | [x] |
| T1 | `fav/Cargo.toml` を `version = "27.2.0"` に bump | [x] |
| T2 | `runes/iceberg/iceberg.fav` を新規作成（6 関数: read / append / overwrite / time_travel / schema_evolution / list_snapshots） | [x] |
| T3 | `fav/src/backend/vm.rs` に `Iceberg.*_raw` primitive 6 件追加（DeltaLake ブロック直後・Azure Blob 直前、`#[cfg]` ガード付き、stub のため rune_loader 登録なし） | [x] |
| T4 | `examples/iceberg_etl.fav` 新規作成（LoadFromIceberg \|> TransformData \|> AppendToIceberg） | [x] |
| T5 | `site/content/docs/runes/iceberg.mdx` 新規作成 | [x] |
| T6 | `CHANGELOG.md` 更新: 先頭に `[v27.2.0]` エントリ追加 | [x] |
| T7 | `benchmarks/v27.2.0.json` 新規作成（test_count: 2142） | [x] |
| T8 | `fav/src/driver.rs` 更新: `v272000_tests`（10 件）を `v271000_tests` の直後に追加 | [x] |
| T8.5 | `cargo test v272000 --bin fav` — 10/10 PASS 確認 | [x] |
| T8.6 | `cargo test iceberg --bin fav` — 9 件以上 PASS 確認 | [x] |
| T9 | `cargo test --bin fav` — 2142 件 PASS 確認（リグレッションなし） | [x] |
| T10 | spec-reviewer レビュー実施（実装前） | [x] |

---

## チェックリスト（完了条件）

- [x] `fav/Cargo.toml` が `version = "27.2.0"` であること
- [x] `runes/iceberg/iceberg.fav` に `fn read(` が含まれること
- [x] `runes/iceberg/iceberg.fav` に `fn append(` が含まれること
- [x] `runes/iceberg/iceberg.fav` に `fn overwrite(` が含まれること
- [x] `runes/iceberg/iceberg.fav` に `fn time_travel(` が含まれること
- [x] `runes/iceberg/iceberg.fav` に `fn schema_evolution(` が含まれること
- [x] `runes/iceberg/iceberg.fav` に `fn list_snapshots(` が含まれること
- [x] `fav/src/backend/vm.rs` に `Iceberg.read_raw` が含まれること
- [x] `fav/src/backend/vm.rs` に `Iceberg.append_raw` が含まれること
- [x] `fav/src/backend/vm.rs` に `Iceberg.overwrite_raw` が含まれること
- [x] `fav/src/backend/vm.rs` に `Iceberg.time_travel_raw` が含まれること
- [x] `fav/src/backend/vm.rs` に `Iceberg.schema_evolution_raw` が含まれること
- [x] `fav/src/backend/vm.rs` に `Iceberg.list_snapshots_raw` が含まれること
- [x] `examples/iceberg_etl.fav` に `IcebergEtlPipeline` が含まれること
- [x] `site/content/docs/runes/iceberg.mdx` が存在すること
- [x] `CHANGELOG.md` に `[v27.2.0]` エントリが存在すること
- [x] `benchmarks/v27.2.0.json` が存在すること（test_count: 2142）
- [x] `v272000_tests` 10 件すべて PASS
- [x] `cargo test iceberg --bin fav` で 9 件以上 PASS
- [x] 総テスト数 ≥ 2142 件

---

## メモ

### vm.rs 挿入位置

DeltaLake ブロック末尾（`"DeltaLake.optimize_raw" => Ok(err_vm(...))` の wasm32 アーム直後）に挿入。
Azure Blob ブロック（`// ── Azure Blob Storage primitives (v14.5.0)`）の直前。

### `iceberg-rust` クレートを追加しない理由

- WASM 互換性への影響（`iceberg-rust` は tokio 依存、WASM では動かない）
- ビルド時間の増大
- 現バージョンは stub で基盤を整え、v28.x で実統合する

### `#[cfg]` パターン（DeltaLake / Pulsar と同一）

各 primitive に `#[cfg(not(target_arch = "wasm32"))]` / `#[cfg(target_arch = "wasm32")]` ペアを付ける。

### テスト数計算

2132（v27.1.0 コードレビュー対応後の実測値）+ 10（v272000_tests）= 2142

> **注**: `benchmarks/v27.1.0.json` の `test_count` は `2130`（コードレビュー対応前の値）のままで実際の 2132 と乖離あり。
> v27.2.0 の起算値として実測値 2132 を使用する。

### include_str! パス

| パス | 解決先 |
|---|---|
| `../../runes/iceberg/iceberg.fav` | `favnir/runes/iceberg/iceberg.fav` |
| `backend/vm.rs` | `fav/src/backend/vm.rs` |
| `../../examples/iceberg_etl.fav` | `favnir/examples/iceberg_etl.fav` |
| `../../CHANGELOG.md` | `favnir/CHANGELOG.md` |

---

## コードレビュー指摘（実装後に記入）

| 指摘 | 対応 |
|---|---|
| — | — |
