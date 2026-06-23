# v20.5.0 — mmap + SIMD CSV パーサー タスク

## ステータス: COMPLETE（T1〜T5 完了）

---

## タスク一覧

### T1: `fav/Cargo.toml` — 依存クレート追加

- [x] **事前確認**: `grep -n "arrow\|memmap" fav/Cargo.toml` で現在の arrow 設定を確認
- [x] `arrow = { version = "52", features = ["ipc"] }` → `features = ["ipc", "csv"]` に変更
- [x] `memmap2 = "0.9"` を `[target.'cfg(not(target_arch = "wasm32"))'.dependencies]` セクションに追加
- [x] **API 確認**: `grep -r "pub fn infer_reader_schema\|pub struct ReaderBuilder" ~/.cargo/registry/src/*/arrow-csv-52*/src/ 2>/dev/null | head -10` で arrow-csv 52 の API を確認
- [x] `cargo check` でコンパイルエラー 0
- [x] `use arrow::csv::ReaderBuilder; use memmap2::MmapOptions;` が vm.rs でコンパイルを通ることを確認

---

### T2: `fav/src/backend/vm.rs` — `read_csv_mmap` + `ArrowBatch.from_csv`

#### 2-1. 配置箇所の特定

- [x] `grep -n "ArrowBatch.from_list\|ArrowBatch.read_parquet\|v20.5\|// ── DuckDB helpers" fav/src/backend/vm.rs | head -10` で挿入箇所を確認
- [x] `grep -n "is_known_builtin_namespace\|ArrowBatch" fav/src/backend/vm.rs | grep -v "//\|format!" | head -10` で `ArrowBatch` が既に登録済みであることを確認（変更不要を確認）

#### 2-2. `read_csv_mmap` ヘルパー関数を追加

- [x] `// ── v20.5.0: mmap + arrow-csv helpers ──` セクションを vm.rs の末尾付近に追加
- [x] `#[cfg(not(target_arch = "wasm32"))]` の `read_csv_mmap(path: &str) -> Result<RecordBatch, String>` を実装:
  - [x] `std::fs::File::open(path)` でファイルオープン
  - [x] `unsafe { MmapOptions::new().map(&file) }` で mmap（SAFETY コメント付き）
  - [x] `arrow::csv::reader::infer_reader_schema(&mut Cursor::new(&mmap[..]), b',', Some(1000), true)` でスキーマ推論
  - [x] `ReaderBuilder::new(schema).with_header(true).with_batch_size(65536).build(Cursor::new(&mmap[..]))` でリーダー構築
  - [x] `reader.collect::<Result<Vec<_>, _>>()` で全チャンク収集
  - [x] 空チェック: `batches.is_empty()` → `Err(...)`
  - [x] 1 チャンク: `Ok(batches.into_iter().next().unwrap())`
  - [x] 複数チャンク: `arrow::compute::concat` で結合 → `RecordBatch::try_new`
- [x] `#[cfg(target_arch = "wasm32")]` の WASM スタブ（常に `Err("...not supported on wasm32")` を返す）を追加
- [x] `pub(crate)` 可視性を付与（driver.rs のテストから呼べるように）
- [x] `cargo check` でコンパイルエラー 0

#### 2-3. `ArrowBatch.from_csv` プリミティブを `vm_call_builtin` に追加

- [x] 既存の `"ArrowBatch.from_list"` ハンドラの近くに `"ArrowBatch.from_csv"` ハンドラを追加:
  ```rust
  "ArrowBatch.from_csv" => {
      let path = match args.into_iter().next() {
          Some(VMValue::Str(s)) => s,
          _ => return Err("ArrowBatch.from_csv: expected String path".to_string()),
      };
      let batch = read_csv_mmap(&path)?;
      // ok_vm() ラッパーを使う（既存 ArrowBatch.from_list / read_parquet と統一）
      Ok(ok_vm(VMValue::ArrowBatch(arrow_store(batch))))
  }
  ```
- [x] `cargo check` でコンパイルエラー 0
- [x] compiler.rs・checker.rs の変更が不要であることを確認（`"ArrowBatch"` は v19.5.0 で登録済み）
- [x] **WASM ビルド確認**: `cargo build --target wasm32-unknown-unknown 2>&1 | grep error` でエラーがないことを確認

---

### T3: `fav/src/driver.rs` — `v205000_tests`

- [x] `driver.rs` 末尾に `#[cfg(test)] mod v205000_tests { ... }` を追加
- [x] `write_temp_csv` ヘルパー関数を実装（並列テスト競合回避のため `line!()` マクロで呼び出し行番号をパスに含める）:
  ```rust
  macro_rules! temp_csv {
      ($content:expr) => {{
          let mut p = std::env::temp_dir();
          p.push(format!("fav_v205_{}_{}.csv", std::process::id(), line!()));
          std::fs::write(&p, $content).unwrap();
          p
      }};
  }
  ```
- [x] テスト 1: `version_is_20_5_0` — `include_str!("../Cargo.toml")` に `"20.5.0"` が含まれる
- [x] テスト 2: `csv_mmap_reads_row_count` — 3 行 CSV → `num_rows() == 3`
- [x] テスト 3: `csv_mmap_schema_has_headers` — フィールド名が `"name"` と `"amount"` を含む
- [x] テスト 4: `csv_mmap_int_column_typed` — `score` 列が `DataType::Int64` または `DataType::Int32`
- [x] テスト 5: `csv_mmap_returns_arrow_batch` — `read_csv_mmap` が `Ok(_)` を返す
- [x] 各テストで temp ファイルを `std::fs::remove_file` でクリーンアップ
- [x] `cargo test v205000` — 5/5 PASS を確認

---

### T4: `fav/Cargo.toml` バージョン更新

- [x] `version = "20.4.0"` → `"20.5.0"` に変更
- [x] 既存の v20.4.0 バージョンテスト（`version_is_20_4_0`）に `#[ignore]` を追加（バージョン更新後に必要）

---

### T5: `CHANGELOG.md` 更新 + ベンチマーク

- [x] `CHANGELOG.md` の先頭に v20.5.0 エントリを追加:
  - [x] `### Added` — `ArrowBatch.from_csv` / `read_csv_mmap` / `memmap2` 依存
  - [x] `### Changed` — arrow `"csv"` feature 追加
  - [x] `### Performance` — CSV スループット +3〜5x / メモリ -40%
- [x] `benchmarks/v20.5.0.json` を実測後に生成:
  ```bash
  bash benchmarks/suite/run_all.sh --format json > benchmarks/v20.5.0.json
  ```

---

### T6: `site/content/docs/runes/arrow.mdx` 更新

- [x] 既存の `arrow.mdx` を読んでドキュメントスタイルを確認
- [x] `## ArrowBatch.from_csv` セクションを追加:
  - [x] シグネチャ: `ArrowBatch.from_csv(path: String) -> ArrowBatch`
  - [x] 説明: mmap + スキーマ自動推論（先頭 1000 行）
  - [x] 使用例（Favnir コード）
  - [x] WASM 非対応の注意書き
  - [x] v20.4.0 DuckDB プッシュダウンとの連携例

---

## テスト（v205000_tests、5 件）

| テスト名 | 内容 |
|---|---|
| `version_is_20_5_0` | `Cargo.toml` に `"20.5.0"` が含まれる |
| `csv_mmap_reads_row_count` | 3 行 CSV → `RecordBatch.num_rows() == 3` |
| `csv_mmap_schema_has_headers` | ヘッダー `"name,amount"` がフィールド名と一致 |
| `csv_mmap_int_column_typed` | integer 列が `Int64` または `Int32` に推論される |
| `csv_mmap_returns_arrow_batch` | `read_csv_mmap` が `Ok(RecordBatch)` を返す |

---

## 完了条件チェックリスト

- [x] `ArrowBatch.from_csv(path)` で CSV ファイルが `ArrowBatch` として読み込める
- [x] スキーマが CSV ヘッダーから自動推論される（String / Int / Float の区別）
- [x] `memmap2` によるゼロコピー読み込みが実装されている
- [x] WASM ビルドで `read_csv_mmap` が `#[cfg(not(target_arch = "wasm32"))]` でガードされている
- [x] `cargo test v205000` — 5/5 PASS
- [x] `cargo test` — リグレッションなし（全既存テストが PASS）
- [x] `fav/Cargo.toml` version が `20.5.0`
- [x] `CHANGELOG.md` に v20.5.0 エントリが追加されている
- [x] `benchmarks/v20.5.0.json` が生成されている
- [x] `csv_10gb_throughput_mb_s` が v20.4.0 比 +3x 以上改善

---

## 優先度

```
T1（Cargo.toml）        ← 他すべての前提
T2（vm.rs 実装）        ← T1 完了後（最大工数）
T3（driver.rs テスト）  ← T2 完了後
T4（バージョン更新）    ← 任意タイミング
T5（CHANGELOG + bench） ← T3 完了後
T6（サイトドキュメント）← T5 完了後
```

---

## 実装リスク と 対策

| リスク | 対策 |
|---|---|
| `arrow::csv::reader::infer_reader_schema` が arrow 52 で存在しない / シグネチャが異なる | T1 完了直後に `cargo doc` または `~/.cargo/registry` のソースで確認。代替: `arrow::csv::infer_schema` / `SchemaInferrer` を調査 |
| `arrow::compute::concat` が `["ipc", "csv"]` feature では利用不可 | `cargo check` で確認。不可の場合は `features` に `"compute"` 追加 |
| `memmap2::MmapOptions::new().map(&file)` が Windows でエラー | Windows の場合はファイルが別プロセスでオープン中だと失敗する。テストで使い捨て temp ファイルを使うことで回避 |
| `ReaderBuilder::new(schema)` の引数型が `Arc<Schema>` でなく `Schema` | ソースで確認して適宜 `Arc::new` を追加または削除 |
| 大ファイルで複数チャンク結合時に OOM | `with_batch_size(65536)` の設定で 1 チャンクに収まるか確認。必要なら `concat_batches` の代わりに最初のバッチのみ返す暫定実装 |
| `pub(crate) fn read_csv_mmap` が driver.rs から見えない | vm.rs が backend モジュールの一部 → `crate::backend::vm::read_csv_mmap` でアクセス可能。`#[cfg(not(wasm32))]` ガードがテストにも必要 |
