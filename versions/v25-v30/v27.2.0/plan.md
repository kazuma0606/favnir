# v27.2.0 実装計画 — iceberg Rune 追加

## 前提確認

- `fav/Cargo.toml`: `version = "27.1.0"`（→ 27.2.0 に bump）
- テスト数: 2132 件
- `runes/iceberg/` ディレクトリが存在しないことを確認（新規作成）
- `vm.rs` に `Iceberg.read_raw` が存在しないことを確認
- `examples/iceberg_etl.fav` が存在しないことを確認

---

## 実装ステップ

### Step 1: Cargo.toml バージョン bump

`fav/Cargo.toml` の `version` を `"27.1.0"` → `"27.2.0"` に変更。

> **注**: `iceberg-rust` クレートは追加しない。stub 実装のため既存の依存のみで実装可能。

---

### Step 2: runes/iceberg/iceberg.fav 新規作成

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

### Step 3: vm.rs に Iceberg primitives 追加

**挿入位置**: `// ── DeltaLake primitives (v27.1.0)` ブロックの直後（`"DeltaLake.optimize_raw" => Ok(err_vm(...))` の直後）。
Azure Blob Storage ブロック（`// ── Azure Blob Storage primitives (v14.5.0)`）の直前。

追加する 6 primitive（各 `#[cfg]` ペア = 12 アーム）:

#### `Iceberg.read_raw`
```rust
#[cfg(not(target_arch = "wasm32"))]
"Iceberg.read_raw" => {
    // (catalog: String, table: String) -> Result<String, String>
    // Stub: iceberg-rust 統合は v28.x 以降
    let mut it = args.into_iter();
    let _catalog = vm_string(it.next().ok_or("Iceberg.read_raw: missing catalog")?, "Iceberg.read_raw")?;
    let _table   = vm_string(it.next().ok_or("Iceberg.read_raw: missing table")?,   "Iceberg.read_raw")?;
    Ok(ok_vm(VMValue::Str("[]".to_string())))
}
#[cfg(target_arch = "wasm32")]
"Iceberg.read_raw" => Ok(err_vm(VMValue::Str("Iceberg not supported on wasm32".into()))),
```

#### `Iceberg.append_raw`
```rust
#[cfg(not(target_arch = "wasm32"))]
"Iceberg.append_raw" => {
    // (catalog: String, table: String, data: String) -> Result<Unit, String>
    let mut it = args.into_iter();
    let _catalog = vm_string(it.next().ok_or("Iceberg.append_raw: missing catalog")?, "Iceberg.append_raw")?;
    let _table   = vm_string(it.next().ok_or("Iceberg.append_raw: missing table")?,   "Iceberg.append_raw")?;
    let _data    = vm_string(it.next().ok_or("Iceberg.append_raw: missing data")?,    "Iceberg.append_raw")?;
    Ok(ok_vm(VMValue::Unit))
}
#[cfg(target_arch = "wasm32")]
"Iceberg.append_raw" => Ok(err_vm(VMValue::Str("Iceberg not supported on wasm32".into()))),
```

#### `Iceberg.overwrite_raw`
```rust
#[cfg(not(target_arch = "wasm32"))]
"Iceberg.overwrite_raw" => {
    // (catalog: String, table: String, data: String, filter: String) -> Result<Unit, String>
    let mut it = args.into_iter();
    let _catalog = vm_string(it.next().ok_or("Iceberg.overwrite_raw: missing catalog")?, "Iceberg.overwrite_raw")?;
    let _table   = vm_string(it.next().ok_or("Iceberg.overwrite_raw: missing table")?,   "Iceberg.overwrite_raw")?;
    let _data    = vm_string(it.next().ok_or("Iceberg.overwrite_raw: missing data")?,    "Iceberg.overwrite_raw")?;
    let _filter  = vm_string(it.next().ok_or("Iceberg.overwrite_raw: missing filter")?,  "Iceberg.overwrite_raw")?;
    Ok(ok_vm(VMValue::Unit))
}
#[cfg(target_arch = "wasm32")]
"Iceberg.overwrite_raw" => Ok(err_vm(VMValue::Str("Iceberg not supported on wasm32".into()))),
```

#### `Iceberg.time_travel_raw`
```rust
#[cfg(not(target_arch = "wasm32"))]
"Iceberg.time_travel_raw" => {
    // (catalog: String, table: String, snapshot_id: Int) -> Result<String, String>
    let mut it = args.into_iter();
    let _catalog = vm_string(it.next().ok_or("Iceberg.time_travel_raw: missing catalog")?, "Iceberg.time_travel_raw")?;
    let _table   = vm_string(it.next().ok_or("Iceberg.time_travel_raw: missing table")?,   "Iceberg.time_travel_raw")?;
    let _snapshot_id = match it.next().ok_or("Iceberg.time_travel_raw: missing snapshot_id")? {
        VMValue::Int(n) => n,
        _ => return Err("Iceberg.time_travel_raw: snapshot_id must be an Int".to_string()),
    };
    Ok(ok_vm(VMValue::Str("[]".to_string())))
}
#[cfg(target_arch = "wasm32")]
"Iceberg.time_travel_raw" => Ok(err_vm(VMValue::Str("Iceberg not supported on wasm32".into()))),
```

#### `Iceberg.schema_evolution_raw`
```rust
#[cfg(not(target_arch = "wasm32"))]
"Iceberg.schema_evolution_raw" => {
    // (catalog: String, table: String, new_schema: String) -> Result<Unit, String>
    let mut it = args.into_iter();
    let _catalog    = vm_string(it.next().ok_or("Iceberg.schema_evolution_raw: missing catalog")?,    "Iceberg.schema_evolution_raw")?;
    let _table      = vm_string(it.next().ok_or("Iceberg.schema_evolution_raw: missing table")?,      "Iceberg.schema_evolution_raw")?;
    let _new_schema = vm_string(it.next().ok_or("Iceberg.schema_evolution_raw: missing new_schema")?, "Iceberg.schema_evolution_raw")?;
    Ok(ok_vm(VMValue::Unit))
}
#[cfg(target_arch = "wasm32")]
"Iceberg.schema_evolution_raw" => Ok(err_vm(VMValue::Str("Iceberg not supported on wasm32".into()))),
```

#### `Iceberg.list_snapshots_raw`
```rust
#[cfg(not(target_arch = "wasm32"))]
"Iceberg.list_snapshots_raw" => {
    // (catalog: String, table: String) -> Result<String, String>
    let mut it = args.into_iter();
    let _catalog = vm_string(it.next().ok_or("Iceberg.list_snapshots_raw: missing catalog")?, "Iceberg.list_snapshots_raw")?;
    let _table   = vm_string(it.next().ok_or("Iceberg.list_snapshots_raw: missing table")?,   "Iceberg.list_snapshots_raw")?;
    Ok(ok_vm(VMValue::Str("[]".to_string())))
}
#[cfg(target_arch = "wasm32")]
"Iceberg.list_snapshots_raw" => Ok(err_vm(VMValue::Str("Iceberg not supported on wasm32".into()))),
```

---

### Step 4: examples/iceberg_etl.fav 新規作成

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

### Step 5: site/content/docs/runes/iceberg.mdx 新規作成

Iceberg Rune のドキュメント。セットアップ・6 関数リファレンス・パイプライン例・REST カタログ注記を記載。

---

### Step 6: CHANGELOG.md 更新

```markdown
## [v27.2.0] — 2026-06-27 — iceberg Rune 追加

### Added
- `runes/iceberg/iceberg.fav` — Apache Iceberg Rune（read / append / overwrite / time_travel / schema_evolution / list_snapshots 6 関数）
- `Iceberg.*_raw` VM primitives 6 件（`#[cfg(not(target_arch = "wasm32"))]` ガード付き、stub 実装）
- `examples/iceberg_etl.fav` — Iceberg ETL デモ（LoadFromIceberg |> TransformData |> AppendToIceberg）
- `site/content/docs/runes/iceberg.mdx` — Apache Iceberg Rune ドキュメント
```

---

### Step 7: benchmarks/v27.2.0.json 新規作成

```json
{"version":"27.2.0","test_count":2142,"timestamp":"2026-06-27"}
```

---

### Step 8: driver.rs に v272000_tests 追加

`v271000_tests` の直後に `v272000_tests` モジュール（10 件）を追加。
`list_snapshots` の Rune 確認テストと vm.rs 確認テストを必ず含める（v27.1.0 で同種の欠落が指摘された先例に対応）。

---

### Step 9: テスト実行

```bash
cargo test v272000 --bin fav        # 10/10 PASS
cargo test iceberg --bin fav        # 9 件以上 PASS
cargo test --bin fav                # 2142 件 PASS
```

> **注**: `cargo test iceberg` は `changelog_has_v27_2_0` 以外の 9 件をすべて検出する（テスト名に `iceberg` を含む）。

### rune_loader 登録について

Iceberg Rune は現バージョン（v27.2.0）では stub 実装のため、rune_loader への登録は不要（delta-lake と同様）。v28.x で iceberg-rust 実統合時に検討する。

---

## include_str! パス（fav/src/driver.rs 基準）

| パス | 対象 |
|---|---|
| `../../runes/iceberg/iceberg.fav` | `favnir/runes/iceberg/iceberg.fav` |
| `backend/vm.rs` | `fav/src/backend/vm.rs` |
| `../../examples/iceberg_etl.fav` | `favnir/examples/iceberg_etl.fav` |
| `../../CHANGELOG.md` | `favnir/CHANGELOG.md` |

---

## 注意事項

### `iceberg-rust` クレートを追加しない理由

- WASM 互換性への影響（`iceberg-rust` は tokio 依存、WASM では動かない）
- ビルド時間の増大
- 現バージョンは stub で基盤を整え、v28.x で実統合する

### `#[cfg]` パターン（DeltaLake / Pulsar と同一）

各 primitive に `#[cfg(not(target_arch = "wasm32"))]` / `#[cfg(target_arch = "wasm32")]` ペアを付ける。

### 挿入位置の再確認

DeltaLake ブロック末尾行（`"DeltaLake.optimize_raw" => Ok(err_vm(...))` の wasm32 アーム直後）・Azure Blob ブロックの直前。
