# v27.3.0 実装計画 — clickhouse Rune 追加

## 前提確認

- `fav/Cargo.toml`: `version = "27.2.0"`（→ 27.3.0 に bump）
- テスト数: 2142 件
- `runes/clickhouse/` ディレクトリが存在しないことを確認（新規作成）
- `vm.rs` に `ClickHouse.connect_raw` が存在しないことを確認
- `examples/clickhouse_analytics.fav` が存在しないことを確認

---

## 実装ステップ

### Step 1: Cargo.toml バージョン bump

`fav/Cargo.toml` の `version` を `"27.2.0"` → `"27.3.0"` に変更。

> **注**: `clickhouse-rs` クレートは追加しない。stub 実装のため既存の依存のみで実装可能。

---

### Step 2: runes/clickhouse/clickhouse.fav 新規作成

```favnir
// runes/clickhouse/clickhouse.fav — ClickHouse Rune (v27.3.0)
// clickhouse-rs 統合は v28.x 以降。現バージョンは引数検証 stub。
// TODO(v28.x): clickhouse-rs クレートを使った実接続に移行予定。
public fn connect(config: String) -> Result<String, String> !Db {
    ClickHouse.connect_raw(config)
}
public fn query(conn: String, sql: String) -> Result<String, String> !Db {
    ClickHouse.query_raw(conn, sql)
}
public fn insert(conn: String, table: String, rows: String) -> Result<Unit, String> !Db {
    ClickHouse.insert_raw(conn, table, rows)
}
public fn async_insert(conn: String, table: String, rows: String) -> Result<Unit, String> !Db {
    ClickHouse.async_insert_raw(conn, table, rows)
}
```

---

### Step 3: vm.rs に ClickHouse primitives 追加

**挿入位置**: `// ── Apache Iceberg primitives (v27.2.0)` ブロックの直後（`"Iceberg.list_snapshots_raw" => Ok(err_vm(...))` の直後）。
Azure Blob Storage ブロック（`// ── Azure Blob Storage primitives (v14.5.0)`）の直前。

追加する 4 primitive（各 `#[cfg]` ペア = 8 アーム）:

#### `ClickHouse.connect_raw`
```rust
#[cfg(not(target_arch = "wasm32"))]
"ClickHouse.connect_raw" => {
    // (config: String) -> Result<String, String>
    // Stub: clickhouse-rs 統合は v28.x 以降
    let mut it = args.into_iter();
    let _config = vm_string(it.next().ok_or("ClickHouse.connect_raw: missing config")?, "ClickHouse.connect_raw")?;
    Ok(ok_vm(VMValue::Str("clickhouse-stub-conn".to_string())))
}
#[cfg(target_arch = "wasm32")]
"ClickHouse.connect_raw" => Ok(err_vm(VMValue::Str("ClickHouse not supported on wasm32".into()))),
```

#### `ClickHouse.query_raw`
```rust
#[cfg(not(target_arch = "wasm32"))]
"ClickHouse.query_raw" => {
    // (conn: String, sql: String) -> Result<String, String>
    let mut it = args.into_iter();
    let _conn = vm_string(it.next().ok_or("ClickHouse.query_raw: missing conn")?, "ClickHouse.query_raw")?;
    let _sql  = vm_string(it.next().ok_or("ClickHouse.query_raw: missing sql")?,  "ClickHouse.query_raw")?;
    Ok(ok_vm(VMValue::Str("[]".to_string())))
}
#[cfg(target_arch = "wasm32")]
"ClickHouse.query_raw" => Ok(err_vm(VMValue::Str("ClickHouse not supported on wasm32".into()))),
```

#### `ClickHouse.insert_raw`
```rust
#[cfg(not(target_arch = "wasm32"))]
"ClickHouse.insert_raw" => {
    // (conn: String, table: String, rows: String) -> Result<Unit, String>
    let mut it = args.into_iter();
    let _conn  = vm_string(it.next().ok_or("ClickHouse.insert_raw: missing conn")?,  "ClickHouse.insert_raw")?;
    let _table = vm_string(it.next().ok_or("ClickHouse.insert_raw: missing table")?, "ClickHouse.insert_raw")?;
    let _rows  = vm_string(it.next().ok_or("ClickHouse.insert_raw: missing rows")?,  "ClickHouse.insert_raw")?;
    Ok(ok_vm(VMValue::Unit))
}
#[cfg(target_arch = "wasm32")]
"ClickHouse.insert_raw" => Ok(err_vm(VMValue::Str("ClickHouse not supported on wasm32".into()))),
```

#### `ClickHouse.async_insert_raw`
```rust
#[cfg(not(target_arch = "wasm32"))]
"ClickHouse.async_insert_raw" => {
    // (conn: String, table: String, rows: String) -> Result<Unit, String>
    let mut it = args.into_iter();
    let _conn  = vm_string(it.next().ok_or("ClickHouse.async_insert_raw: missing conn")?,  "ClickHouse.async_insert_raw")?;
    let _table = vm_string(it.next().ok_or("ClickHouse.async_insert_raw: missing table")?, "ClickHouse.async_insert_raw")?;
    let _rows  = vm_string(it.next().ok_or("ClickHouse.async_insert_raw: missing rows")?,  "ClickHouse.async_insert_raw")?;
    Ok(ok_vm(VMValue::Unit))
}
#[cfg(target_arch = "wasm32")]
"ClickHouse.async_insert_raw" => Ok(err_vm(VMValue::Str("ClickHouse not supported on wasm32".into()))),
```

---

### Step 4: examples/clickhouse_analytics.fav 新規作成

```favnir
// examples/clickhouse_analytics.fav — ClickHouse Analytics デモ (v27.3.0)
import rune "clickhouse"

stage LoadEvents: Unit -> Result<String, String> !Db = |_| {
    bind conn <- ClickHouse.connect("http://localhost:8123")
    ClickHouse.query(conn, "SELECT * FROM events LIMIT 100")
}

// seq pipeline は前ステージの成功値（String）を次ステージの引数として渡す
stage InsertProcessed: String -> Result<String, String> !Db = |data| {
    bind conn <- ClickHouse.connect("http://localhost:8123")
    bind _ <- ClickHouse.insert(conn, "processed_events", data)
    Result.ok("inserted to clickhouse")
}

seq ClickHouseAnalyticsPipeline = LoadEvents |> InsertProcessed
```

---

### Step 5: site/content/docs/runes/clickhouse.mdx 新規作成

ClickHouse Rune のドキュメント。セットアップ・4 関数リファレンス・パイプライン例・Docker 構成例を記載。

---

### Step 6: CHANGELOG.md 更新

```markdown
## [v27.3.0] — 2026-06-27 — clickhouse Rune 追加

### Added
- `runes/clickhouse/clickhouse.fav` — ClickHouse Rune（connect / query / insert / async_insert 4 関数）
- `ClickHouse.*_raw` VM primitives 4 件（`#[cfg(not(target_arch = "wasm32"))]` ガード付き、stub 実装）
- `examples/clickhouse_analytics.fav` — ClickHouse Analytics デモ（LoadEvents |> InsertProcessed）
- `site/content/docs/runes/clickhouse.mdx` — ClickHouse Rune ドキュメント
```

---

### Step 7: benchmarks/v27.3.0.json 新規作成

```json
{"version":"27.3.0","test_count":2150,"timestamp":"2026-06-27"}
```

---

### Step 8: driver.rs に v273000_tests 追加

`v272000_tests` の直後に `v273000_tests` モジュール（8 件）を追加。

---

### Step 9: テスト実行

```bash
cargo test v273000 --bin fav        # 8/8 PASS
cargo test clickhouse --bin fav     # 7 件以上 PASS
cargo test --bin fav                # 2150 件 PASS
```

> **注**: `cargo test clickhouse` は `changelog_has_v27_3_0` 以外の 9 件をすべて検出する（`clickhouse_rune_vm_has_query_raw` / `clickhouse_rune_vm_has_async_insert_raw` を含む）。

### rune_loader 登録について

ClickHouse Rune は現バージョン（v27.3.0）では stub 実装のため、rune_loader への登録は不要（delta-lake / iceberg と同様）。v28.x で clickhouse-rs 実統合時に検討する。

---

## include_str! パス（fav/src/driver.rs 基準）

| パス | 対象 |
|---|---|
| `../../runes/clickhouse/clickhouse.fav` | `favnir/runes/clickhouse/clickhouse.fav` |
| `backend/vm.rs` | `fav/src/backend/vm.rs` |
| `../../examples/clickhouse_analytics.fav` | `favnir/examples/clickhouse_analytics.fav` |
| `../../CHANGELOG.md` | `favnir/CHANGELOG.md` |

---

## 注意事項

### `clickhouse-rs` クレートを追加しない理由

- WASM 互換性への影響
- ビルド時間の増大
- 現バージョンは stub で基盤を整え、v28.x で実統合する

### `connect_raw` の戻り値

接続ハンドルとして `"clickhouse-stub-conn"` を返す。
v28.x で clickhouse-rs を統合した際は実際の接続オブジェクトを表す文字列識別子に置き換える。

### `#[cfg]` パターン（DeltaLake / Iceberg と同一）

各 primitive に `#[cfg(not(target_arch = "wasm32"))]` / `#[cfg(target_arch = "wasm32")]` ペアを付ける。

### 挿入位置の再確認

Iceberg ブロック末尾行（`"Iceberg.list_snapshots_raw" => Ok(err_vm(...))` の wasm32 アーム直後）・Azure Blob ブロックの直前。

### テスト数計算

2142（v27.2.0 完了後）+ 8（v273000_tests）= 2150
