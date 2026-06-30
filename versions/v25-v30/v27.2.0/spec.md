# v27.2.0 仕様書 — iceberg Rune 追加

## 概要

Apache Iceberg Rune を追加する。
`import rune "iceberg"` → `Iceberg.*` 名前空間で 6 関数が使用可能になる。
REST カタログ / AWS Glue カタログ経由でテーブルの読み書き・スナップショット管理ができる。

---

## 背景

ロードマップ v27.2「iceberg Rune 追加」より。Data Lakehouse フェーズの第 2 コンポーネント。
Apache Iceberg は Snowflake / AWS Glue / Apache Spark との親和性が高く、
マルチエンジン対応のデータレイクテーブル形式として急速に普及している。

ロードマップ要件:
- 6 関数実装（read / append / overwrite / time_travel / schema_evolution / list_snapshots）
- REST カタログ（ローカル）で `cargo test iceberg` が 4 件以上 PASS

---

## 実装する関数

| 関数 | シグネチャ | 内容 |
|---|---|---|
| `Iceberg.read` | `(catalog: String, table: String) -> Result<String, String> !Io` | テーブル全件読み込み（JSON 配列返却） |
| `Iceberg.append` | `(catalog: String, table: String, data: String) -> Result<Unit, String> !Io` | データ追加（新しいスナップショット作成） |
| `Iceberg.overwrite` | `(catalog: String, table: String, data: String, filter: String) -> Result<Unit, String> !Io` | 条件を満たすデータを上書き |
| `Iceberg.time_travel` | `(catalog: String, table: String, snapshot_id: Int) -> Result<String, String> !Io` | スナップショット ID 指定読み込み |
| `Iceberg.schema_evolution` | `(catalog: String, table: String, new_schema: String) -> Result<Unit, String> !Io` | スキーマ追加・型昇格（互換変更のみ） |
| `Iceberg.list_snapshots` | `(catalog: String, table: String) -> Result<String, String> !Io` | スナップショット一覧取得（JSON 配列） |

> **エフェクト**: REST カタログへの HTTP 通信・ファイルシステム操作のため `!Io` エフェクトを使用する。

---

## VM Primitive（vm.rs に追加）

| primitive 名 | 実装方針 |
|---|---|
| `Iceberg.read_raw` | stub: 引数検証のみ、`"[]"` 返却（`iceberg-rust` 統合は v28.x） |
| `Iceberg.append_raw` | stub: 引数検証、`ok_vm(VMValue::Unit)` 返却 |
| `Iceberg.overwrite_raw` | stub: 引数検証、`ok_vm(VMValue::Unit)` 返却 |
| `Iceberg.time_travel_raw` | stub: 引数検証（snapshot_id は Int）、`"[]"` 返却 |
| `Iceberg.schema_evolution_raw` | stub: 引数検証、`ok_vm(VMValue::Unit)` 返却 |
| `Iceberg.list_snapshots_raw` | stub: 引数検証、`"[]"` 返却 |

> **注意**: `iceberg-rust`（Apache 公式 Rust 実装）の Cargo 依存追加は WASM 互換性・ビルド時間への影響があるため、
> v27.2.0 では引数検証 stub とする。実 Iceberg テーブルの読み書きは v28.x で実装する。
> すべての primitive に `#[cfg(not(target_arch = "wasm32"))]` / `#[cfg(target_arch = "wasm32")]` ガードを付ける。

**挿入位置**: DeltaLake ブロック末尾（`"DeltaLake.optimize_raw" => Ok(err_vm(...))` の直後）・Azure Blob ブロックの直前。

---

## runes/iceberg/iceberg.fav

```favnir
// runes/iceberg/iceberg.fav — Apache Iceberg Rune (v27.2.0)
// iceberg-rust 統合は v28.x 以降。現バージョンは引数検証 stub。
// TODO(v28.x): iceberg-rust クレートを使った実テーブル読み書きに移行予定。
public fn read(catalog: String, table: String) -> Result<String, String> !Io {
    Iceberg.read_raw(catalog, table)
}
public fn append(catalog: String, table: String, data: String) -> Result<Unit, String> !Io {
    Iceberg.append_raw(catalog, table, data)
}
public fn overwrite(catalog: String, table: String, data: String, filter: String) -> Result<Unit, String> !Io {
    Iceberg.overwrite_raw(catalog, table, data, filter)
}
public fn time_travel(catalog: String, table: String, snapshot_id: Int) -> Result<String, String> !Io {
    Iceberg.time_travel_raw(catalog, table, snapshot_id)
}
public fn schema_evolution(catalog: String, table: String, new_schema: String) -> Result<Unit, String> !Io {
    Iceberg.schema_evolution_raw(catalog, table, new_schema)
}
public fn list_snapshots(catalog: String, table: String) -> Result<String, String> !Io {
    Iceberg.list_snapshots_raw(catalog, table)
}
```

---

## examples/iceberg_etl.fav

```favnir
// examples/iceberg_etl.fav — Iceberg ETL デモ (v27.2.0)
import rune "iceberg"

stage LoadFromIceberg: Unit -> Result<String, String> !Io = |_| {
    Iceberg.read("http://localhost:8181", "warehouse.raw_orders")
}

stage TransformData: String -> Result<String, String> = |rows_json| {
    if String.length(rows_json) > 2
    then Result.ok(rows_json)
    else Result.err("empty iceberg table")
}

stage AppendToIceberg: String -> Result<String, String> !Io = |data| {
    bind _ <- Iceberg.append("http://localhost:8181", "warehouse.processed_orders", data)
    Result.ok("appended to iceberg table")
}

seq IcebergEtlPipeline = LoadFromIceberg |> TransformData |> AppendToIceberg
```

---

## テスト

### driver.rs v272000_tests（10 件）

| テスト名 | 内容 |
|---|---|
| `iceberg_rune_has_read_fn` | `iceberg.fav` に `"fn read("` が含まれること |
| `iceberg_rune_has_append_fn` | `iceberg.fav` に `"fn append("` が含まれること |
| `iceberg_rune_has_overwrite_fn` | `iceberg.fav` に `"fn overwrite("` が含まれること |
| `iceberg_rune_has_time_travel_fn` | `iceberg.fav` に `"fn time_travel("` が含まれること |
| `iceberg_rune_has_schema_evolution_fn` | `iceberg.fav` に `"fn schema_evolution("` が含まれること |
| `iceberg_rune_has_list_snapshots_fn` | `iceberg.fav` に `"fn list_snapshots("` が含まれること |
| `iceberg_rune_vm_has_read_raw` | `vm.rs` に `"Iceberg.read_raw"` が含まれること |
| `iceberg_rune_vm_has_list_snapshots_raw` | `vm.rs` に `"Iceberg.list_snapshots_raw"` が含まれること |
| `iceberg_example_has_pipeline` | `examples/iceberg_etl.fav` に `"IcebergEtlPipeline"` が含まれること |
| `changelog_has_v27_2_0` | `CHANGELOG.md` に `"[v27.2.0]"` が含まれること |

### `cargo test iceberg` 期待値

- `v272000_tests::iceberg_rune_has_*` 6 件（read / append / overwrite / time_travel / schema_evolution / list_snapshots）
- `v272000_tests::iceberg_rune_vm_has_read_raw` 1 件
- `v272000_tests::iceberg_rune_vm_has_list_snapshots_raw` 1 件
- `v272000_tests::iceberg_example_has_pipeline` 1 件（テスト名に `iceberg` を含む）
- 合計 9 件（`changelog_has_v27_2_0` は `iceberg` を含まないため除外）（ロードマップ要件「4 件以上」超過）

---

## 完了条件

- [ ] `fav/Cargo.toml` が `version = "27.2.0"` であること
- [ ] `runes/iceberg/iceberg.fav` に `fn read(` が含まれること
- [ ] `runes/iceberg/iceberg.fav` に `fn append(` が含まれること
- [ ] `runes/iceberg/iceberg.fav` に `fn overwrite(` が含まれること
- [ ] `runes/iceberg/iceberg.fav` に `fn time_travel(` が含まれること
- [ ] `runes/iceberg/iceberg.fav` に `fn schema_evolution(` が含まれること
- [ ] `runes/iceberg/iceberg.fav` に `fn list_snapshots(` が含まれること
- [ ] `fav/src/backend/vm.rs` に `Iceberg.read_raw` が含まれること
- [ ] `fav/src/backend/vm.rs` に `Iceberg.append_raw` が含まれること
- [ ] `fav/src/backend/vm.rs` に `Iceberg.time_travel_raw` が含まれること
- [ ] `examples/iceberg_etl.fav` に `IcebergEtlPipeline` が含まれること
- [ ] `site/content/docs/runes/iceberg.mdx` が存在すること
- [ ] `CHANGELOG.md` に `[v27.2.0]` エントリが存在すること
- [ ] `benchmarks/v27.2.0.json` が存在すること（test_count: 2140）
- [ ] `fav/src/backend/vm.rs` に `Iceberg.list_snapshots_raw` が含まれること
- [ ] `v272000_tests` 10 件すべて PASS
- [ ] `cargo test iceberg --bin fav` で 9 件以上 PASS
- [ ] 総テスト数 ≥ 2142 件

---

## 注意事項

### `snapshot_id` の型

`Iceberg.time_travel` の `snapshot_id` は `Int`（内部 i64）を使用する。
Apache Iceberg 仕様の snapshot-id は 64-bit 符号付き整数（long）であり、Favnir `Int`（i64）と一致する。
v28.x 実統合時もシグネチャ変更なし（STABILITY.md v1.x ポリシー適合）。

## スコープ外（v28.x 以降）

- `iceberg-rust` クレートを使った実 Iceberg テーブルの読み書き（実データ連携）
- `Iceberg.read[T]` / `Iceberg.time_travel[T]` ジェネリック API（ロードマップ記載。v28.0 マイルストーン宣言の「iceberg Rune 5 条件クリア」の実体がこれに該当）
- REST カタログ / AWS Glue カタログへの実接続
- AWS Glue Data Catalog 対応
- Time travel の実スナップショット解決
- Change Data Feed / Incremental read
