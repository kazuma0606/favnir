# v20.5.0 Spec — mmap + SIMD CSV パーサー

## 概要

v20.5.0 は Favnir の **I/O 層を根本的に改善**する。
現状の `csv` クレートによる行単位 CSV 読み込みを、
`memmap2`（ゼロコピー mmap）+ `arrow::csv`（列指向 SIMD パース）に置き換え、
大容量 CSV ファイルを直接 `ArrowBatch` として取り込む新プリミティブ `ArrowBatch.from_csv` を追加する。

**テーマ**: Runtime Excellence シリーズ第5弾 — I/O 層の根本的改善

---

## 動機と期待効果

### 現状の問題

```
現状（3 段階コピー）:
  File
  → read() syscall（カーネル → ユーザー空間へバイトコピー）
  → csv クレート（行単位パース → Vec<StringRecord>）
  → Vec<String> アロケーション × N 行
  → VMValue::Record(HashMap) 変換 × N 行
```

- `read()` 呼び出しのたびにカーネル/ユーザー空間コンテキストスイッチが発生
- 各行ごとに `String` + `HashMap` をアロケートし、GC 圧力が高い
- 行指向パースのため SIMD 最適化が効きにくい

### 最適化後

```
最適化後（ゼロコピー + 列指向）:
  File
  → memmap2（mmap syscall 1 回でアドレス空間にマッピング）
  → arrow::csv::ReaderBuilder（列指向 SIMD パース + スキーマ推論）
  → Arrow RecordBatch（列指向メモリ、全フィールドが連続配列）
  → VMValue::ArrowBatch(id) として返却（ARROW_BATCHES に格納）
```

| 改善点 | 内容 |
|---|---|
| syscall 削減 | `read()` N 回 → `mmap` 1 回 |
| アロケーション削減 | `Vec<String>` × N 行 → 列配列 × フィールド数 |
| SIMD パース | `arrow::csv` の列指向パーサーが CPU ベクトル命令を活用 |
| ゼロコピー | ファイルデータをカーネルバッファからコピーせず直接参照 |

### 期待改善（v20.4.0 比）

| ベンチマーク | v20.4.0 基準 | 期待改善 |
|---|---|---|
| `csv_10gb_throughput_mb_s` | ~340 MB/s | **+3〜5x**（> 1 GB/s） |
| `peak_memory_csv_1gb_mb` | ~2,400 MB | **-40%**（ゼロコピー） |
| `csv_row_alloc_1m_ms` | ~210ms | **+2〜3x** |

---

## アーキテクチャ

### 新プリミティブ: `ArrowBatch.from_csv`

```favnir
// 現状: Csv.parse（文字列 → List<Record>）
stage LoadCsv: String -> List<Row> = |path| {
  bind content <- IO.read_file(path)
  Csv.parse(content)
}

// 最適化後: ArrowBatch.from_csv（ファイルパス → ArrowBatch）
stage LoadCsv: String -> ArrowBatch = |path| {
  ArrowBatch.from_csv(path)
}
// → mmap + arrow-csv で直接 RecordBatch を生成
// → 後続 stage は ArrowBatch を DuckDB プッシュダウンにも渡せる（v20.4.0 との連携）
```

### データフロー

```
ArrowBatch.from_csv(path)
  ↓
read_csv_mmap(path: &str) → Result<RecordBatch, String>
  ├── File::open(path)
  ├── MmapOptions::new().map(&file)       // ゼロコピーマッピング
  ├── infer_schema(&mmap[..], n=1000)     // 先頭 1000 行でスキーマ推論
  ├── ReaderBuilder::new(schema)
  │     .with_header(true)
  │     .build(Cursor::new(&mmap[..]))    // mmap スライスを直接渡す
  └── reader.collect::<Result<Vec<_>>>()  // RecordBatch 収集
        → merge_batches(batches)          // 単一 RecordBatch に結合

VMValue::ArrowBatch(arrow_store(batch))  // ARROW_BATCHES に格納
```

---

## 新依存クレート

| クレート | バージョン | 理由 |
|---|---|---|
| `memmap2` | `"0.9"` | ゼロコピー mmap |
| `arrow` の `"csv"` feature | （既存 `arrow = "52"` に追加） | `arrow::csv::ReaderBuilder` |

### `Cargo.toml` の変更

```toml
# [target.'cfg(not(target_arch = "wasm32"))'.dependencies]

# 既存行を変更（"csv" feature 追加）:
arrow = { version = "52", features = ["ipc", "csv"] }

# 新規追加:
memmap2 = "0.9"
```

> **実装注意**: `arrow::csv` の `ReaderBuilder` は `arrow` クレートの `csv` feature で有効になる。
> `arrow-csv = "52"` を別途追加する必要はない（同一 workspace クレート）。

---

## `read_csv_mmap` 実装詳細

```rust
// fav/src/backend/vm.rs

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn read_csv_mmap(path: &str) -> Result<arrow::record_batch::RecordBatch, String> {
    use arrow::csv::ReaderBuilder;
    use memmap2::MmapOptions;
    use std::io::Cursor;

    let file = std::fs::File::open(path)
        .map_err(|e| format!("ArrowBatch.from_csv: cannot open '{}': {e}", path))?;

    // ゼロコピー mmap — ファイルの全データをアドレス空間にマッピング
    let mmap = unsafe {
        MmapOptions::new()
            .map(&file)
            .map_err(|e| format!("ArrowBatch.from_csv: mmap failed: {e}"))?
    };

    // スキーマ推論: 先頭 1000 行をサンプリング
    let (schema, _) = arrow::csv::reader::infer_reader_schema(
        &mut Cursor::new(&mmap[..]),
        b',',           // delimiter
        Some(1000),     // max_records
        true,           // has_header
    )
    .map_err(|e| format!("ArrowBatch.from_csv: schema inference failed: {e}"))?;

    let schema = std::sync::Arc::new(schema);

    // mmap スライスを直接 ReaderBuilder に渡す（ゼロコピー）
    let mut reader = ReaderBuilder::new(schema)
        .with_header(true)
        .with_batch_size(65536) // 64k 行ずつチャンク処理
        .build(Cursor::new(&mmap[..]))
        .map_err(|e| format!("ArrowBatch.from_csv: reader build failed: {e}"))?;

    // 全チャンクを収集して単一 RecordBatch に結合
    let batches: Vec<_> = (&mut reader)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("ArrowBatch.from_csv: parse failed: {e}"))?;

    if batches.is_empty() {
        return Err(format!("ArrowBatch.from_csv: no data in '{}'", path));
    }
    if batches.len() == 1 {
        return Ok(batches.into_iter().next().unwrap());
    }

    // 複数チャンクを結合
    let schema_ref = batches[0].schema();
    let arrays: Vec<_> = (0..schema_ref.fields().len())
        .map(|i| {
            let cols: Vec<_> = batches.iter().map(|b| b.column(i).as_ref()).collect();
            arrow::compute::concat(&cols)
                .map_err(|e| format!("ArrowBatch.from_csv: concat failed: {e}"))
        })
        .collect::<Result<Vec<_>, _>>()?;

    arrow::record_batch::RecordBatch::try_new(schema_ref, arrays)
        .map_err(|e| format!("ArrowBatch.from_csv: merge failed: {e}"))
}

// WASM スタブ
#[cfg(target_arch = "wasm32")]
pub(crate) fn read_csv_mmap(_path: &str) -> Result<arrow::record_batch::RecordBatch, String> {
    Err("ArrowBatch.from_csv: not supported on wasm32".to_string())
}
```

---

## `ArrowBatch.from_csv` プリミティブ

```rust
// vm.rs の vm_call_builtin 内、ArrowBatch セクションに追加

"ArrowBatch.from_csv" => {
    let path = match args.into_iter().next() {
        Some(VMValue::Str(s)) => s,
        _ => return Err("ArrowBatch.from_csv: expected String path".to_string()),
    };
    let batch = read_csv_mmap(&path)?;
    Ok(ok_vm(VMValue::ArrowBatch(arrow_store(batch))))
}
```

> **注意**: `"ArrowBatch"` は既に `is_known_builtin_namespace`（vm.rs）と
> compiler.rs（builtin 名リスト）に登録済み（v19.5.0 で追加）。
> `from_csv` を追加しても compiler.rs・checker.rs の変更は不要。

---

## 変更ファイル一覧

| ファイル | 変更内容 |
|---|---|
| `fav/Cargo.toml` | `arrow` に `"csv"` feature 追加 / `memmap2 = "0.9"` 追加（native-only）/ version `20.4.0` → `20.5.0` |
| `fav/src/backend/vm.rs` | `read_csv_mmap` helper（`#[cfg(not(wasm32))]`） + `ArrowBatch.from_csv` primitive（`vm_call_builtin`） |
| `fav/src/driver.rs` | `v205000_tests` モジュール（5 件） |
| `CHANGELOG.md` | v20.5.0 エントリ追加 |
| `benchmarks/v20.5.0.json` | 実測ベンチマーク結果 |
| `site/content/docs/runes/arrow.mdx` | `ArrowBatch.from_csv` ドキュメント追加 |

---

## テスト（v205000_tests、5 件）

| テスト名 | 内容 |
|---|---|
| `version_is_20_5_0` | `Cargo.toml` に `"20.5.0"` が含まれる |
| `csv_mmap_reads_row_count` | 3 行 CSV → `read_csv_mmap` → `RecordBatch.num_rows() == 3` |
| `csv_mmap_schema_has_headers` | ヘッダー `"name,amount"` が RecordBatch のフィールド名と一致する |
| `csv_mmap_int_column_typed` | integer 列が `DataType::Int64`（または Int32）として推論される |
| `csv_mmap_returns_arrow_batch` | `read_csv_mmap` が `Ok(RecordBatch)` を返す（正常系） |

### テスト用ヘルパー

```rust
// driver.rs v205000_tests 内

fn write_temp_csv(content: &str) -> std::path::PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!("fav_v205_test_{}.csv", std::process::id()));
    std::fs::write(&path, content).unwrap();
    path
}
```

---

## 完了条件

- [ ] `ArrowBatch.from_csv(path)` で大容量 CSV が `ArrowBatch` として読み込める
- [ ] スキーマが CSV ヘッダーから自動推論される（文字列・整数・浮動小数点の区別）
- [ ] `memmap2` を使用したゼロコピー読み込みが実装されている
- [ ] WASM ビルドで `read_csv_mmap` が `#[cfg(not(wasm32))]` でガードされている
- [ ] `cargo test v205000` — 5/5 PASS
- [ ] `cargo test` — リグレッションなし（全既存テストが PASS）
- [ ] `fav/Cargo.toml` version が `20.5.0`
- [ ] `CHANGELOG.md` に v20.5.0 エントリが追加されている
- [ ] `benchmarks/v20.5.0.json` が生成されている
- [ ] `csv_10gb_throughput_mb_s` が v20.4.0 比 +3x 以上改善

---

## 技術ノート

### `arrow::csv` の `infer_reader_schema` API

`arrow::csv::reader::infer_reader_schema` は `Read` を実装する任意の型を受け取る。
`Cursor::new(&mmap[..])` で mmap スライスを `Read` として渡せる。

```rust
// 正式シグネチャ（arrow 52）
pub fn infer_reader_schema<R: Read>(
    reader: &mut R,
    delimiter: u8,
    max_read_records: Option<usize>,
    has_header: bool,
) -> Result<(Schema, usize), ArrowError>
```

実装前に `cargo doc -p arrow --open` または `cargo metadata` でシグネチャを確認する。

### mmap の安全性

`MmapOptions::new().map(&file)` は `unsafe` ブロックが必要。
これはファイルの内容が mmap の有効期間中に変更された場合（external mutation）に
未定義動作になるためだが、Favnir のユースケース（静的 CSV ファイルの読み込み）では
実質的に安全。`unsafe` ブロックにコメントを付けること:

```rust
// SAFETY: The file is opened read-only and not modified during mmap lifetime.
// External file mutation during a pipeline run is outside Favnir's contract.
let mmap = unsafe { MmapOptions::new().map(&file)? };
```

### v20.4.0 との連携

`ArrowBatch.from_csv` で読み込んだデータは `VMValue::ArrowBatch` として返るため、
v20.4.0 の DuckDB プッシュダウン（`__duckdb_push` builtin）と自動的に連携する。

```favnir
seq FastPipeline
= |path| ArrowBatch.from_csv(path)   // mmap 読み込み（v20.5.0）
|> |batch| List.filter(batch, ...)    // DuckDB プッシュダウン（v20.4.0）
```

### `arrow::compute::concat` の使用

複数チャンクの結合に `arrow::compute::concat` を使用する。
arrow 52 はデフォルトフィーチャーに `compute` を含むため、
`arrow = { version = "52", features = ["ipc", "csv"] }` で通常は使用可能
（`default-features = false` を指定しない限り）。

T1 の `cargo check` で `use of undeclared crate or module 'compute'` が出た場合は
`features` に `"compute"` を明示追加する:
```toml
arrow = { version = "52", features = ["ipc", "csv", "compute"] }
```

### スコープ外（v20.6 以降）

- io_uring 非同期 I/O（Linux）
- `Csv.read_file_mmap` （旧 List<Record> 互換インターフェースの高速化）
- TSV / カスタム区切り文字の mmap 対応（`ArrowBatch.from_tsv` 等）
- スキーマ明示指定（`ArrowBatch.from_csv_with_schema`）
