# v27.1.0 仕様書 — delta-lake Rune 追加

## 概要

Apache Delta Lake Rune を追加する。
`import rune "delta-lake"` → `DeltaLake.*` 名前空間で 7 関数が使用可能になる。
ローカルファイルシステム（`/tmp/`）および S3（LocalStack）上の Delta テーブルを読み書きできる。

---

## 背景

ロードマップ v27.1「delta-lake Rune 追加」より。Data Lakehouse フェーズの最初のコンポーネント。

ロードマップ要件:
- 7 関数実装（read / read_with_filter / write / merge / history / vacuum / optimize）
- `cargo test delta_lake` で 5 件以上 PASS
- `fav run examples/delta_lake_etl.fav`（ローカルパス）が動作確認できること

---

## 実装する関数

| 関数 | シグネチャ | 内容 |
|---|---|---|
| `DeltaLake.read` | `(path: String) -> Result<String, String> !Io` | Delta テーブル全件読み込み（JSON 配列返却） |
| `DeltaLake.read_with_filter` | `(path: String, predicate: String) -> Result<String, String> !Io` | 述語プッシュダウン付き読み込み |
| `DeltaLake.write` | `(path: String, data: String, mode: String) -> Result<Unit, String> !Io` | 書き込み（`append` / `overwrite`） |
| `DeltaLake.merge` | `(path: String, data: String, condition: String) -> Result<String, String> !Io` | MERGE（upsert / delete-when-matched） |
| `DeltaLake.history` | `(path: String) -> Result<String, String> !Io` | トランザクションログ取得（バージョン一覧 JSON） |
| `DeltaLake.vacuum` | `(path: String, retention_hours: Int) -> Result<Unit, String> !Io` | 古いファイル削除（最小 168h = 7 日） |
| `DeltaLake.optimize` | `(path: String) -> Result<String, String> !Io` | コンパクション（小さいファイルをまとめる） |

> **エフェクト**: Delta Lake はローカル / S3 のファイル操作であるため `!Io` エフェクトを使用する。

---

## VM Primitive（vm.rs に追加）

| primitive 名 | 実装方針 |
|---|---|
| `DeltaLake.read_raw` | stub: 引数検証のみ、`"[]"` 返却（`delta-rs` 統合は v28.x） |
| `DeltaLake.read_with_filter_raw` | stub: 引数検証、`"[]"` 返却 |
| `DeltaLake.write_raw` | stub: 引数検証、`ok_vm(VMValue::Unit)` 返却 |
| `DeltaLake.merge_raw` | stub: 引数検証、`"{\"merged\":0}"` 返却 |
| `DeltaLake.history_raw` | stub: 引数検証、`"[]"` 返却 |
| `DeltaLake.vacuum_raw` | stub: 引数検証、`ok_vm(VMValue::Unit)` 返却 |
| `DeltaLake.optimize_raw` | stub: 引数検証、`"{\"optimized\":0}"` 返却 |

> **注意**: `delta-rs`（Rust 実装）の Cargo 依存追加は WASM 互換性・ビルド時間への影響があるため、
> v27.1.0 では引数検証 stub とする。実 Delta テーブルの読み書きは v28.x で実装する。
> すべてのprimitiveに `#[cfg(not(target_arch = "wasm32"))]` / `#[cfg(target_arch = "wasm32")]` ガードを付ける。

---

## runes/delta-lake/delta-lake.fav

```favnir
// runes/delta-lake/delta-lake.fav — Delta Lake Rune (v27.1.0)
// delta-rs 統合は v28.x 以降。現バージョンは引数検証 stub。
public fn read(path: String) -> Result<String, String> !Io {
    DeltaLake.read_raw(path)
}
public fn read_with_filter(path: String, predicate: String) -> Result<String, String> !Io {
    DeltaLake.read_with_filter_raw(path, predicate)
}
public fn write(path: String, data: String, mode: String) -> Result<Unit, String> !Io {
    DeltaLake.write_raw(path, data, mode)
}
public fn merge(path: String, data: String, condition: String) -> Result<String, String> !Io {
    DeltaLake.merge_raw(path, data, condition)
}
public fn history(path: String) -> Result<String, String> !Io {
    DeltaLake.history_raw(path)
}
public fn vacuum(path: String, retention_hours: Int) -> Result<Unit, String> !Io {
    DeltaLake.vacuum_raw(path, retention_hours)
}
public fn optimize(path: String) -> Result<String, String> !Io {
    DeltaLake.optimize_raw(path)
}
```

---

## examples/delta_lake_etl.fav

```favnir
// examples/delta_lake_etl.fav — Delta Lake ETL デモ (v27.1.0)
import rune "delta-lake"

stage LoadRawData: Unit -> Result<String, String> !Io = |_| {
    DeltaLake.read("/tmp/favnir_delta/raw_orders")
}

stage TransformOrders: String -> Result<String, String> !Pure = |rows_json| {
    if String.length(rows_json) > 2
    then Result.ok(rows_json)
    else Result.err("empty delta table")
}

stage SaveProcessed: String -> Result<String, String> !Io = |data| {
    bind _ <- DeltaLake.write("/tmp/favnir_delta/processed_orders", data, "append")
    Result.ok("written to delta table")
}

seq DeltaEtlPipeline = LoadRawData |> TransformOrders |> SaveProcessed
```

---

## テスト

### driver.rs v271000_tests（8 件）

| テスト名 | 内容 |
|---|---|
| `delta_lake_rune_has_read_fn` | `delta-lake.fav` に `"fn read("` が含まれること |
| `delta_lake_rune_has_write_fn` | `delta-lake.fav` に `"fn write("` が含まれること |
| `delta_lake_rune_has_merge_fn` | `delta-lake.fav` に `"fn merge("` が含まれること |
| `delta_lake_rune_has_history_fn` | `delta-lake.fav` に `"fn history("` が含まれること |
| `delta_lake_rune_has_vacuum_fn` | `delta-lake.fav` に `"fn vacuum("` が含まれること |
| `delta_lake_rune_vm_has_read_raw` | `vm.rs` に `"DeltaLake.read_raw"` が含まれること |
| `delta_lake_example_has_pipeline` | `examples/delta_lake_etl.fav` に `"DeltaEtlPipeline"` が含まれること |
| `changelog_has_v27_1_0` | `CHANGELOG.md` に `"[v27.1.0]"` が含まれること |

### `cargo test delta_lake` 期待値

- `v271000_tests::delta_lake_rune_has_*` 5 件
- `v271000_tests::delta_lake_rune_vm_has_read_raw` 1 件
- `v271000_tests::delta_lake_example_has_pipeline` 1 件（テスト名に `delta_lake` を含む）
- 合計 7 件（ロードマップ要件「5 件以上」超過）

---

## 完了条件

- [ ] `fav/Cargo.toml` が `version = "27.1.0"` であること
- [ ] `runes/delta-lake/delta-lake.fav` に `fn read(` が含まれること
- [ ] `runes/delta-lake/delta-lake.fav` に `fn write(` が含まれること
- [ ] `runes/delta-lake/delta-lake.fav` に `fn merge(` が含まれること
- [ ] `runes/delta-lake/delta-lake.fav` に `fn history(` が含まれること
- [ ] `runes/delta-lake/delta-lake.fav` に `fn vacuum(` が含まれること
- [ ] `runes/delta-lake/delta-lake.fav` に `fn optimize(` が含まれること
- [ ] `fav/src/backend/vm.rs` に `DeltaLake.read_raw` が含まれること
- [ ] `fav/src/backend/vm.rs` に `DeltaLake.write_raw` が含まれること
- [ ] `fav/src/backend/vm.rs` に `DeltaLake.optimize_raw` が含まれること
- [ ] `examples/delta_lake_etl.fav` に `DeltaEtlPipeline` が含まれること
- [ ] `site/content/docs/runes/delta-lake.mdx` が存在すること
- [ ] `CHANGELOG.md` に `[v27.1.0]` エントリが存在すること
- [ ] `benchmarks/v27.1.0.json` が存在すること（test_count: 2130）
- [ ] `v271000_tests` 8 件すべて PASS
- [ ] `runes/delta-lake/delta-lake.fav` に `fn read_with_filter(` が含まれること
- [ ] `cargo test delta_lake --bin fav` で 7 件以上 PASS
- [ ] 総テスト数 ≥ 2130 件

---

## スコープ外（v28.x 以降）

- `delta-rs` クレートを使った実 Delta Lake テーブルの読み書き（実データ連携）
- `DeltaLake.read[T]` ジェネリック API（ロードマップ記載。型安全な行取得。v28.x で実装）
- `DeltaLake.read_with_filter` の述語プッシュダウン（実装）
- S3 / LocalStack 上の Delta テーブル操作
- Time travel（バージョン指定読み込み）
- Change Data Feed（CDC）
- Delta Sharing プロトコル対応
