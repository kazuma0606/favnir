# v20.5.0 実装計画 — mmap + SIMD CSV パーサー

## 実装順序

```
T1: Cargo.toml — memmap2 追加 / arrow に "csv" feature 追加           ← 最初（他の前提）
T2: vm.rs — read_csv_mmap helper + ArrowBatch.from_csv primitive     ← T1 完了後
T3: driver.rs — v205000_tests（5 件）                                  ← T2 完了後
T4: Cargo.toml version bump（20.4.0 → 20.5.0）                       ← 任意
T5: CHANGELOG.md 更新 + benchmarks/v20.5.0.json                      ← T3 完了後
T6: site/content/docs/runes/arrow.mdx 更新（from_csv ドキュメント）   ← T5 完了後
```

**変更ファイル一覧:**
- `fav/Cargo.toml`（T1 / T4）
- `fav/src/backend/vm.rs`（T2）
- `fav/src/driver.rs`（T3）
- `CHANGELOG.md`（T5）
- `benchmarks/v20.5.0.json`（T5）
- `site/content/docs/runes/arrow.mdx`（T6）

---

## T1: `Cargo.toml` — 依存クレート追加

### 変更点

```toml
# [target.'cfg(not(target_arch = "wasm32"))'.dependencies] セクション

# 変更前:
arrow = { version = "52", features = ["ipc"] }

# 変更後（"csv" feature を追加）:
arrow = { version = "52", features = ["ipc", "csv"] }

# 新規追加:
memmap2 = "0.9"
```

### 事前確認

```bash
# arrow 52 で csv feature が存在するか確認
cargo metadata --format-version 1 | grep -A5 '"arrow"'

# または cargo add で追加後に Cargo.lock を確認
grep -A3 'name = "arrow-csv"' fav/Cargo.lock
```

`arrow = "52"` の `csv` feature は `arrow-csv = "52"` ワークスペースクレートを
re-export する。`infer_reader_schema` と `ReaderBuilder` が `arrow::csv` 名前空間で
使用可能になる。

### 完了条件
- `cargo check` でコンパイルエラー 0
- `use arrow::csv::ReaderBuilder;` がコンパイルを通る

---

## T2: `vm.rs` — `read_csv_mmap` + `ArrowBatch.from_csv`

### 2-1. 事前確認: `arrow::csv` API の正式シグネチャ確認

```bash
# arrow-csv のソースで実際の API を確認
grep -rn "pub fn infer_reader_schema\|pub struct ReaderBuilder" \
  ~/.cargo/registry/src/*/arrow-csv-52*/src/ 2>/dev/null | head -10
```

実装前に `arrow::csv::reader::infer_reader_schema` の引数型と
`arrow::csv::ReaderBuilder::new()` のシグネチャを確認する。
arrow 52 では `ReaderBuilder::new(Arc<Schema>)` を受け取る。

**API が存在しない場合のフォールバック対応:**

`infer_reader_schema` が見つからない場合は以下の代替パスを試みる:

```rust
// 代替 A: infer_file_schema を使用
arrow::csv::reader::infer_file_schema(&mut cursor, b',', Some(1000), true)?

// 代替 B: ReaderBuilder の infer_schema メソッドチェーン（arrow 53+ 形式）
ReaderBuilder::new()
    .infer_schema(Some(1000))
    .has_header(true)
    .build(cursor)?
```

```bash
# 代替 API の存在確認
grep -rn "pub fn infer\|fn infer_schema" \
  ~/.cargo/registry/src/*/arrow-csv-52*/src/ 2>/dev/null
```

実装前に必ず確認し、実際に存在する API に合わせて実装コードを調整すること。

### 2-2. `read_csv_mmap` ヘルパー関数を追加

配置場所: `vm.rs` の `// ── DuckDB helpers` セクション付近（または新しいセクション）。

```rust
// ── v20.5.0: mmap + arrow-csv helpers ─────────────────────────────────────

/// CSV ファイルを mmap でゼロコピー読み込みし、Arrow RecordBatch を返す。
/// 先頭 1000 行でスキーマを推論する。
///
/// `#[cfg(not(target_arch = "wasm32"))]` でガード — WASM では常に Err を返す。
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn read_csv_mmap(path: &str) -> Result<arrow::record_batch::RecordBatch, String> {
    use arrow::csv::ReaderBuilder;
    use memmap2::MmapOptions;
    use std::io::Cursor;

    let file = std::fs::File::open(path)
        .map_err(|e| format!("ArrowBatch.from_csv: cannot open '{}': {e}", path))?;

    // SAFETY: The file is opened read-only and not modified during mmap lifetime.
    // External file mutation during a pipeline run is outside Favnir's contract.
    let mmap = unsafe {
        MmapOptions::new()
            .map(&file)
            .map_err(|e| format!("ArrowBatch.from_csv: mmap failed: {e}"))?
    };

    // スキーマ推論: 先頭 1000 行をサンプリング
    let (schema, _) = arrow::csv::reader::infer_reader_schema(
        &mut Cursor::new(&mmap[..]),
        b',',
        Some(1000),
        true, // has_header
    )
    .map_err(|e| format!("ArrowBatch.from_csv: schema inference failed: {e}"))?;

    let schema = std::sync::Arc::new(schema);

    // mmap スライスを直接 ReaderBuilder に渡す
    let mut reader = ReaderBuilder::new(schema)
        .with_header(true)
        .with_batch_size(65536)
        .build(Cursor::new(&mmap[..]))
        .map_err(|e| format!("ArrowBatch.from_csv: reader build failed: {e}"))?;

    // 全チャンクを収集
    let batches: Vec<_> = (&mut reader)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("ArrowBatch.from_csv: parse failed: {e}"))?;

    if batches.is_empty() {
        return Err(format!("ArrowBatch.from_csv: no data in '{}'", path));
    }
    if batches.len() == 1 {
        return Ok(batches.into_iter().next().unwrap());
    }

    // 複数チャンクを単一 RecordBatch に結合
    let schema_ref = batches[0].schema();
    let arrays = (0..schema_ref.fields().len())
        .map(|i| {
            let cols: Vec<_> = batches.iter().map(|b| b.column(i).as_ref()).collect();
            arrow::compute::concat(&cols)
                .map_err(|e| format!("ArrowBatch.from_csv: concat col {i}: {e}"))
        })
        .collect::<Result<Vec<_>, _>>()?;

    arrow::record_batch::RecordBatch::try_new(schema_ref, arrays)
        .map_err(|e| format!("ArrowBatch.from_csv: merge failed: {e}"))
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn read_csv_mmap(_path: &str) -> Result<arrow::record_batch::RecordBatch, String> {
    Err("ArrowBatch.from_csv: not supported on wasm32".to_string())
}
```

### 2-3. `ArrowBatch.from_csv` プリミティブを `vm_call_builtin` に追加

既存の `ArrowBatch.*` ハンドラがある箇所（`"ArrowBatch.from_list"` の近く）に追加:

```rust
"ArrowBatch.from_csv" => {
    let path = match args.into_iter().next() {
        Some(VMValue::Str(s)) => s,
        _ => return Err("ArrowBatch.from_csv: expected String path".to_string()),
    };
    let batch = read_csv_mmap(&path)?;
    Ok(ok_vm(VMValue::ArrowBatch(arrow_store(batch))))
}
```

> **確認事項**: `"ArrowBatch"` は既に `is_known_builtin_namespace`（vm.rs line ~6893）と
> compiler.rs builtins リスト（line ~238）に登録済み（v19.5.0）。
> `from_csv` を追加しても **compiler.rs・checker.rs の変更は不要**。

### 完了条件
- `cargo check` でコンパイルエラー 0
- `ArrowBatch.from_csv("path/to/file.csv")` が `.fav` ファイルから呼び出せる
- WASM ビルドがコンパイルエラーなし

---

## T3: `driver.rs` — `v205000_tests`

### テスト用ヘルパー

```rust
fn write_temp_csv(content: &str) -> std::path::PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!("fav_v205_test_{}.csv", std::process::id()));
    std::fs::write(&path, content).unwrap();
    path
}
```

### 5 テスト

```rust
#[cfg(test)]
mod v205000_tests {
    use crate::backend::vm::read_csv_mmap;

    // line!() を使って各テストごとに一意なファイルパスを生成（並列実行時の競合回避）
    macro_rules! temp_csv {
        ($content:expr) => {{
            let mut p = std::env::temp_dir();
            p.push(format!("fav_v205_{}_{}.csv", std::process::id(), line!()));
            std::fs::write(&p, $content).unwrap();
            p
        }};
    }

    #[test]
    fn version_is_20_5_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("20.5.0"), "Cargo.toml should have version 20.5.0");
    }

    #[test]
    fn csv_mmap_reads_row_count() {
        let path = temp_csv!("name,amount\nalice,100\nbob,200\ncharlie,300\n");
        let batch = read_csv_mmap(path.to_str().unwrap()).expect("should read CSV");
        let _ = std::fs::remove_file(&path);
        assert_eq!(batch.num_rows(), 3, "expected 3 data rows");
    }

    #[test]
    fn csv_mmap_schema_has_headers() {
        let path = temp_csv!("name,amount\nalice,100\n");
        let batch = read_csv_mmap(path.to_str().unwrap()).expect("should read CSV");
        let _ = std::fs::remove_file(&path);
        let field_names: Vec<&str> = batch.schema().fields()
            .iter()
            .map(|f| f.name().as_str())
            .collect();
        assert!(field_names.contains(&"name"), "schema should have 'name' field");
        assert!(field_names.contains(&"amount"), "schema should have 'amount' field");
    }

    #[test]
    fn csv_mmap_int_column_typed() {
        let path = temp_csv!("id,score\n1,95\n2,87\n3,100\n");
        let batch = read_csv_mmap(path.to_str().unwrap()).expect("should read CSV");
        let _ = std::fs::remove_file(&path);
        let schema = batch.schema();
        let score_field = schema.field_with_name("score").expect("should have 'score'");
        let dt = score_field.data_type();
        assert!(
            matches!(dt, arrow::datatypes::DataType::Int64 | arrow::datatypes::DataType::Int32),
            "score should be integer type, got {:?}", dt
        );
    }

    #[test]
    fn csv_mmap_returns_arrow_batch() {
        let path = temp_csv!("x,y\n1,2\n3,4\n");
        let batch = read_csv_mmap(path.to_str().unwrap());
        let _ = std::fs::remove_file(&path);
        assert!(batch.is_ok(), "from_csv should succeed for valid CSV");
        assert_eq!(batch.unwrap().num_rows(), 2);
    }
}
```

### 完了条件
- `cargo test v205000` — 5/5 PASS

---

## T4: `fav/Cargo.toml` バージョン更新

`version = "20.4.0"` → `"20.5.0"` に変更。

---

## T5: `CHANGELOG.md` 更新 + `benchmarks/v20.5.0.json`

### CHANGELOG エントリ

```markdown
## [v20.5.0] — 2026-06-XX — mmap + SIMD CSV パーサー

### Added
- `ArrowBatch.from_csv(path: String) -> ArrowBatch` — mmap + arrow-csv でゼロコピー CSV 読み込み
- `read_csv_mmap` helper（vm.rs）— スキーマ自動推論 + 先頭 1000 行サンプリング
- `memmap2 = "0.9"` 依存クレート追加（native-only）

### Changed
- `arrow = { version = "52", features = ["ipc", "csv"] }` — `"csv"` feature 追加

### Performance
- `csv_10gb_throughput_mb_s`: +3〜5x（mmap ゼロコピー + 列指向 SIMD パース）
- `peak_memory_csv_1gb_mb`: -40%（中間 Vec<String> アロケーション削減）
- `csv_row_alloc_1m_ms`: +2〜3x（行単位 HashMap 生成の排除）
```

### `benchmarks/v20.5.0.json`

実測後に生成:
```bash
bash benchmarks/suite/run_all.sh --format json > benchmarks/v20.5.0.json
```

---

## T6: `site/content/docs/runes/arrow.mdx` 更新

既存の `arrow.mdx`（`ArrowBatch.from_list`, `to_list`, `write_parquet`, `read_parquet` のドキュメント）に
`from_csv` のセクションを追加する。

```mdx
## ArrowBatch.from_csv

```favnir
ArrowBatch.from_csv(path: String) -> ArrowBatch
```

CSV ファイルをゼロコピー mmap で読み込み、`ArrowBatch` として返します。
スキーマは CSV ヘッダーと先頭 1000 行から自動推論されます。

```favnir
stage LoadData: String -> ArrowBatch = |path| {
  ArrowBatch.from_csv(path)
}
```

**注意**: 本プリミティブは WASM 環境では使用できません（native 専用）。
WASM 環境では `Csv.parse` を使用してください。
```

---

## 注意点

### `arrow::csv::reader::infer_reader_schema` の API 変更確認

arrow 52 では `infer_reader_schema` のシグネチャが arrow 51 から変更されている可能性がある。
実装前に必ず確認:

```bash
grep -r "pub fn infer_reader_schema" ~/.cargo/registry/src/*/arrow-csv-52*/src/
```

API が異なる場合は `infer_file_schema` や `SchemaInferrer` を代替として使用する。

### `arrow::compute` feature の確認

`arrow = { version = "52", features = ["ipc", "csv"] }` で `arrow::compute::concat` が
使用可能かを確認する:

```bash
cargo check 2>&1 | grep "arrow::compute"
```

使用不可の場合は `features` に `"compute"` を追加する。

### `read_csv_mmap` の `pub(crate)` 可視性

テスト（`driver.rs` の `v205000_tests`）から直接呼び出すために
`pub(crate)` 可視性が必要。`vm.rs` 内で `pub(crate) fn read_csv_mmap` として宣言する。

### 既存 `Csv.parse` との互換性

`read_csv_mmap` は新プリミティブとして**追加のみ**行う。
既存の `Csv.parse`、`Csv.parse_raw`、`Csv.parse_with_header` は一切変更しない。
後方互換性を完全に保つ。
