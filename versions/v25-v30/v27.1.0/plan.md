# v27.1.0 実装計画 — delta-lake Rune 追加

## 前提確認

- `fav/Cargo.toml`: `version = "27.0.0"`
- テスト数: 2122 件
- `runes/delta-lake/` ディレクトリが存在しないことを確認（新規作成）
- `vm.rs` に `DeltaLake.read_raw` が存在しないことを確認
- `examples/delta_lake_etl.fav` が存在しないことを確認

---

## 実装ステップ

### Step 1: Cargo.toml バージョン bump

`fav/Cargo.toml` の `version` を `"27.0.0"` → `"27.1.0"` に変更。

> **注**: `delta-rs` クレートは追加しない。stub 実装のため既存の依存のみで実装可能。

---

### Step 2: runes/delta-lake/delta-lake.fav 新規作成

```favnir
// runes/delta-lake/delta-lake.fav — Delta Lake Rune (v27.1.0)
// delta-rs 統合は v28.x 以降。現バージョンは引数検証 stub。
// TODO(v28.x): delta-rs クレートを使った実テーブル読み書きに移行予定。
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

### Step 3: vm.rs に DeltaLake primitives 追加

**挿入位置**: `// ── Pulsar primitives (v26.9.0)` ブロックの直後（`// ── Azure Blob Storage` の直前）。

追加する 7 primitive（各 `#[cfg]` ペア = 14 アーム）:

#### `DeltaLake.read_raw`
```rust
#[cfg(not(target_arch = "wasm32"))]
"DeltaLake.read_raw" => {
    // (path: String) -> Result<String, String>
    // Stub: delta-rs 統合は v28.x 以降
    let mut it = args.into_iter();
    let _path = vm_string(it.next().ok_or("DeltaLake.read_raw: missing path")?, "DeltaLake.read_raw")?;
    Ok(ok_vm(VMValue::Str("[]".to_string())))
}
#[cfg(target_arch = "wasm32")]
"DeltaLake.read_raw" => Ok(err_vm(VMValue::Str("DeltaLake not supported on wasm32".into()))),
```

#### `DeltaLake.read_with_filter_raw`
```rust
#[cfg(not(target_arch = "wasm32"))]
"DeltaLake.read_with_filter_raw" => {
    // (path: String, predicate: String) -> Result<String, String>
    let mut it = args.into_iter();
    let _path      = vm_string(it.next().ok_or("DeltaLake.read_with_filter_raw: missing path")?,      "DeltaLake.read_with_filter_raw")?;
    let _predicate = vm_string(it.next().ok_or("DeltaLake.read_with_filter_raw: missing predicate")?, "DeltaLake.read_with_filter_raw")?;
    Ok(ok_vm(VMValue::Str("[]".to_string())))
}
#[cfg(target_arch = "wasm32")]
"DeltaLake.read_with_filter_raw" => Ok(err_vm(VMValue::Str("DeltaLake not supported on wasm32".into()))),
```

#### `DeltaLake.write_raw`
```rust
#[cfg(not(target_arch = "wasm32"))]
"DeltaLake.write_raw" => {
    // (path: String, data: String, mode: String) -> Result<Unit, String>
    let mut it = args.into_iter();
    let _path = vm_string(it.next().ok_or("DeltaLake.write_raw: missing path")?, "DeltaLake.write_raw")?;
    let _data = vm_string(it.next().ok_or("DeltaLake.write_raw: missing data")?, "DeltaLake.write_raw")?;
    let mode  = vm_string(it.next().ok_or("DeltaLake.write_raw: missing mode")?, "DeltaLake.write_raw")?;
    if mode != "append" && mode != "overwrite" {
        return Ok(err_vm(VMValue::Str(format!("DeltaLake.write: invalid mode '{}', must be 'append' or 'overwrite'", mode))));
    }
    Ok(ok_vm(VMValue::Unit))
}
#[cfg(target_arch = "wasm32")]
"DeltaLake.write_raw" => Ok(err_vm(VMValue::Str("DeltaLake not supported on wasm32".into()))),
```

#### `DeltaLake.merge_raw`
```rust
#[cfg(not(target_arch = "wasm32"))]
"DeltaLake.merge_raw" => {
    // (path: String, data: String, condition: String) -> Result<String, String>
    let mut it = args.into_iter();
    let _path      = vm_string(it.next().ok_or("DeltaLake.merge_raw: missing path")?,      "DeltaLake.merge_raw")?;
    let _data      = vm_string(it.next().ok_or("DeltaLake.merge_raw: missing data")?,      "DeltaLake.merge_raw")?;
    let _condition = vm_string(it.next().ok_or("DeltaLake.merge_raw: missing condition")?, "DeltaLake.merge_raw")?;
    Ok(ok_vm(VMValue::Str("{\"merged\":0}".to_string())))
}
#[cfg(target_arch = "wasm32")]
"DeltaLake.merge_raw" => Ok(err_vm(VMValue::Str("DeltaLake not supported on wasm32".into()))),
```

#### `DeltaLake.history_raw`
```rust
#[cfg(not(target_arch = "wasm32"))]
"DeltaLake.history_raw" => {
    // (path: String) -> Result<String, String>
    let mut it = args.into_iter();
    let _path = vm_string(it.next().ok_or("DeltaLake.history_raw: missing path")?, "DeltaLake.history_raw")?;
    Ok(ok_vm(VMValue::Str("[]".to_string())))
}
#[cfg(target_arch = "wasm32")]
"DeltaLake.history_raw" => Ok(err_vm(VMValue::Str("DeltaLake not supported on wasm32".into()))),
```

#### `DeltaLake.vacuum_raw`
```rust
#[cfg(not(target_arch = "wasm32"))]
"DeltaLake.vacuum_raw" => {
    // (path: String, retention_hours: Int) -> Result<Unit, String>
    let mut it = args.into_iter();
    let _path = vm_string(it.next().ok_or("DeltaLake.vacuum_raw: missing path")?, "DeltaLake.vacuum_raw")?;
    let retention = match it.next().ok_or("DeltaLake.vacuum_raw: missing retention_hours")? {
        VMValue::Int(n) => n,
        _ => return Err("DeltaLake.vacuum_raw: retention_hours must be an Int".to_string()),
    };
    if retention < 168 {
        return Ok(err_vm(VMValue::Str(format!(
            "DeltaLake.vacuum: retention_hours {} is below minimum 168 (7 days)", retention
        ))));
    }
    Ok(ok_vm(VMValue::Unit))
}
#[cfg(target_arch = "wasm32")]
"DeltaLake.vacuum_raw" => Ok(err_vm(VMValue::Str("DeltaLake not supported on wasm32".into()))),
```

#### `DeltaLake.optimize_raw`
```rust
#[cfg(not(target_arch = "wasm32"))]
"DeltaLake.optimize_raw" => {
    // (path: String) -> Result<String, String>
    let mut it = args.into_iter();
    let _path = vm_string(it.next().ok_or("DeltaLake.optimize_raw: missing path")?, "DeltaLake.optimize_raw")?;
    Ok(ok_vm(VMValue::Str("{\"optimized\":0}".to_string())))
}
#[cfg(target_arch = "wasm32")]
"DeltaLake.optimize_raw" => Ok(err_vm(VMValue::Str("DeltaLake not supported on wasm32".into()))),
```

---

### Step 4: examples/delta_lake_etl.fav 新規作成

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

### Step 5: site/content/docs/runes/delta-lake.mdx 新規作成

Delta Lake Rune のドキュメント。セットアップ・環境変数・7 関数リファレンス・パイプライン例を記載。

---

### Step 6: CHANGELOG.md 更新

```markdown
## [v27.1.0] — 2026-06-27 — delta-lake Rune 追加

### Added
- `runes/delta-lake/delta-lake.fav` — Delta Lake Rune（read / read_with_filter / write / merge / history / vacuum / optimize 7 関数）
- `DeltaLake.*_raw` VM primitives 7 件（`#[cfg(not(wasm32))]` ガード付き、stub 実装）
- `examples/delta_lake_etl.fav` — Delta Lake ETL デモ（LoadRawData |> TransformOrders |> SaveProcessed）
- `site/content/docs/runes/delta-lake.mdx` — Delta Lake Rune ドキュメント
```

---

### Step 7: benchmarks/v27.1.0.json 新規作成

```json
{"version":"27.1.0","test_count":2130,"timestamp":"2026-06-27"}
```

---

### Step 8: driver.rs に v271000_tests 追加

`v270000_tests` の直後に `v271000_tests` モジュール（8 件）を追加。

---

### Step 9: テスト実行

```bash
cargo test v271000 --bin fav        # 8/8 PASS
cargo test delta_lake --bin fav     # 7 件以上 PASS
cargo test --bin fav                # 2130 件 PASS
```

> **注**: `cargo test delta_lake` は `delta_lake_example_has_pipeline` も検出する（テスト名に `delta_lake` を含む）ため 7 件。

### rune_loader 登録について

delta-lake Rune は現バージョン（v27.1.0）では stub 実装のため、rune_loader への登録は不要（parser/checker 側で `import rune "delta-lake"` のパスをファイルシステムから解決する既存機構を使用）。v28.x で delta-rs 実統合時に `rune_loader.rs` 更新を検討する。

---

## include_str! パス（fav/src/driver.rs 基準）

| パス | 対象 |
|---|---|
| `../../runes/delta-lake/delta-lake.fav` | `favnir/runes/delta-lake/delta-lake.fav` |
| `backend/vm.rs` | `fav/src/backend/vm.rs` |
| `../../examples/delta_lake_etl.fav` | `favnir/examples/delta_lake_etl.fav` |
| `../../CHANGELOG.md` | `favnir/CHANGELOG.md` |

---

## 注意事項

### DeltaLake.write_raw の `mode` バリデーション

`mode` は `"append"` または `"overwrite"` のみ受け付ける。
不正な mode 値（`"upsert"` など）は `err_vm` を返す。
これはテスト時に意図しない silent failure を防ぐための軽量バリデーション。

### DeltaLake.vacuum_raw の `retention_hours` 下限チェック

Delta Lake の公式仕様では vacuumの最小保持期間は 7 日（168 時間）。
`retention_hours < 168` の場合は `err_vm` を返す。
この検証は stub でも有効（実 delta-rs 実装時も同条件）。

### `#[cfg]` パターン（Pulsar と同一）

各 primitive に `#[cfg(not(target_arch = "wasm32"))]` / `#[cfg(target_arch = "wasm32")]` ペアを付ける。

### 挿入位置の再確認

Pulsar ブロック末尾行（`"Pulsar.nack_raw" => Ok(err_vm(...))` の直後）・Azure Blob ブロックの直前。
