# v19.5.0 Spec — メモリレイアウト最適化（Apache Arrow 統合）

## 概要

`Value` enum の非効率なメモリレイアウトを Apache Arrow 形式に置き換える。
列指向ストレージで SIMD 最適化・Parquet 書き込みのゼロコピーを実現する。

**テーマ**: Production Performance シリーズ第5弾

---

## 動機

### 現状のメモリレイアウト問題

```rust
// 現在の VMValue enum
enum VMValue {
    Int(i64),         // 8 bytes + 1 byte tag = 9 bytes（alignment で 16 bytes）
    Float(f64),       // 8 bytes + tag = 16 bytes
    Str(String),      // 24 bytes (ptr + len + cap) + tag = 32 bytes
    Bool(bool),       // 1 byte + tag = 16 bytes（padding）
    List(FavList),    // 24 bytes + tag = 32 bytes
}

// List<Record> でのレコードリスト: 各レコードが別々のメモリ領域
// キャッシュミスが多発、SIMD 不可
```

### Arrow 形式（列指向）

```
通常の行指向:
  row 0: { id: 1, name: "Alice",   amount: 100.0 }
  row 1: { id: 2, name: "Bob",     amount: 200.0 }
  row 2: { id: 3, name: "Charlie", amount: 300.0 }

Arrow の列指向（RecordBatch）:
  id:     [1, 2, 3]                      ← i64 の連続配列（SIMD フレンドリー）
  name:   ["Alice", "Bob", "Charlie"]    ← StringArray
  amount: [100.0, 200.0, 300.0]          ← f64 の連続配列
```

---

## Favnir での使用

```favnir
// ArrowBatch.from_list / to_list — List<T> ↔ ArrowBatch 変換
fn process(rows: List<Unit>) -> Result<Int, String> {
    bind batch <- ArrowBatch.from_list(rows)     // List → ArrowBatch
    bind back  <- ArrowBatch.to_list(batch)      // ArrowBatch → List
    Result.ok(List.length(back))
}

// Parquet ゼロコピー書き込み / 読み込み
fn roundtrip(rows: List<Unit>, path: String) -> Result<List<Unit>, String> {
    bind batch  <- ArrowBatch.from_list(rows)
    bind _      <- ArrowBatch.write_parquet(batch, path)
    bind batch2 <- ArrowBatch.read_parquet(path)
    ArrowBatch.to_list(batch2)
}

// #[arrow] アノテーション付き stage — 内部的に Arrow 形式で処理
#[arrow]
stage Transform(rows: List<Row>) -> List<Row> {
    Result.ok(rows)
    // Arrow RecordBatch として内部保持 → Parquet 書き込みがゼロコピー
}
```

---

## 実装内容

### T1: `fav/src/ast.rs` — `TrfDef.arrow` フィールド追加

`TrfDef` に `pub arrow: bool` を追加する（v19.1.0 の `stateful: bool` と同じパターン）。

### T2: `fav/src/frontend/parser.rs` — `#[arrow]` 解析追加

`parse_arrow_annotation()` を追加（`parse_stateful_annotation()` と同じパターン）:
```rust
fn parse_arrow_annotation(&mut self) -> Result<bool, ParseError> {
    // lookahead: # [ arrow ]
    ...
}
```
`parse_item` で `stateful_ann` の後に `arrow_ann` を解析する。

### T3: `fav/src/backend/vm.rs` — ArrowBatch サポート追加

**3-A: `VMValue::ArrowBatch(u64)` variant 追加**

`VMValue` enum に `ArrowBatch(u64)` を追加（DbHandle と同じオパーク ID パターン）:
- `PartialEq`: `(ArrowBatch(a), ArrowBatch(b)) => a == b`
- `vm_value_to_value()`: `ArrowBatch(id) => Value::Str(format!("<arrow:{id}>"))`
- `display_value()` / `format_value()` / `value_type_name()`: 対応追加

**3-B: スレッドローカル ArrowBatch ストア**

```rust
thread_local! {
    static ARROW_BATCHES: RefCell<HashMap<u64, arrow::record_batch::RecordBatch>>
        = RefCell::new(HashMap::new());
    static NEXT_ARROW_ID: std::cell::Cell<u64> = std::cell::Cell::new(0);
}

fn store_arrow_batch(batch: RecordBatch) -> u64 { ... }
fn get_arrow_batch(id: u64) -> Option<RecordBatch> { ... }
```

**3-C: VM primitives 追加（`vm_call_builtin` に追記）**

| プリミティブ | 引数 | 戻り値 |
|---|---|---|
| `ArrowBatch.from_list` | `List<Record>` | `Result<ArrowBatch, String>` |
| `ArrowBatch.to_list` | `ArrowBatch` | `Result<List<Record>, String>` |
| `ArrowBatch.write_parquet` | `ArrowBatch, String(path)` | `Result<Unit, String>` |
| `ArrowBatch.read_parquet` | `String(path)` | `Result<ArrowBatch, String>` |

`ArrowBatch.from_list` の実装:
1. `VMValue::List(rows)` を受け取る
2. 各 row は `VMValue::Record(HashMap<String, VMValue>)` を仮定
3. 最初の row からスキーマを推論（`"Int"` → `Int64Array`, `"Float"` → `Float64Array`, `"Str"` → `StringArray`, それ以外 → `StringArray`）
4. `RecordBatch::try_new(schema, arrays)` で RecordBatch を生成
5. `ARROW_BATCHES` に格納して `VMValue::ArrowBatch(id)` を返す

`ArrowBatch.to_list` の実装:
1. `VMValue::ArrowBatch(id)` を受け取る
2. `ARROW_BATCHES` から RecordBatch を取得
3. 各列を行に変換: `Vec<HashMap<String, VMValue>>` → `VMValue::List`

`ArrowBatch.write_parquet` の実装:
1. `VMValue::ArrowBatch(id)` と `VMValue::Str(path)` を受け取る
2. `ARROW_BATCHES` から RecordBatch を取得
3. `ArrowWriter::try_new(file, schema, None)` で Parquet ファイル書き込み

`ArrowBatch.read_parquet` の実装:
1. `VMValue::Str(path)` を受け取る
2. `ParquetRecordBatchReaderBuilder::try_new(file)` で読み込み
3. 最初の RecordBatch を `ARROW_BATCHES` に格納して `VMValue::ArrowBatch(id)` を返す

### T4: `fav/src/backend/mod.rs` — mod 追加不要（vm.rs 内で実装）

### T5: `fav/src/driver.rs` — `v195000_tests` 追加

- `v194000_tests::version_is_19_4_0` に `#[ignore]` を追加
- `v195000_tests` モジュール（5件）を追加

### T6: `fav/Cargo.toml` — バージョン更新

`19.4.0` → `19.5.0`（`arrow = "52"` と `parquet = "52"` は既存なので追加不要）

### T7: `site/content/docs/tools/arrow.mdx` — ドキュメント作成

---

## テスト（v195000_tests、5件）

| テスト名 | 内容 |
|---|---|
| `version_is_19_5_0` | Cargo.toml に `"19.5.0"` が含まれる |
| `arrow_batch_from_list` | `ArrowBatch.from_list` が List → ArrowBatch に変換する |
| `arrow_batch_to_list` | `ArrowBatch.to_list` でラウンドトリップ後に元の長さが一致する |
| `arrow_parquet_roundtrip` | `write_parquet` → `read_parquet` でデータが保持される |
| `arrow_stage_executes` | `#[arrow]` 付き TrfDef が `trf_def.arrow == true` でパースされる |

---

## 完了条件

- [ ] `TrfDef.arrow: bool` が ast.rs に追加される
- [ ] `parse_arrow_annotation()` が parser.rs に追加される
- [ ] `VMValue::ArrowBatch(u64)` が vm.rs に追加される
- [ ] `ArrowBatch.from_list` / `to_list` / `write_parquet` / `read_parquet` が実装される
- [ ] スキーマ推論（Int/Float/Str の列型推論）が動作する
- [ ] `cargo test v195000` — 5/5 PASS
- [ ] `cargo test` — リグレッションなし
- [ ] `site/content/docs/tools/arrow.mdx` が存在する

---

## 技術ノート

### 既存の Arrow/Parquet 依存

`arrow = { version = "52", features = ["ipc"] }` と `parquet = "52"` は Cargo.toml に既存。
**新規依存追加不要**。

### スキーマ推論の方針

`ArrowBatch.from_list(rows)` は第 1 行目の各フィールド値から型を推論:
- `VMValue::Int(_)` → `DataType::Int64`
- `VMValue::Float(_)` → `DataType::Float64`
- `VMValue::Bool(_)` → `DataType::Boolean`
- それ以外（Str, Unit, etc.）→ `DataType::Utf8`（文字列として格納）

行が空の場合はスキーマなし（`RecordBatch::new_empty`）。

### `VMValue::ArrowBatch` の match arm 追加箇所

`DbHandle` と同じ場所に追加（4〜5 箇所）:
1. `impl PartialEq for VMValue`
2. `vm_value_to_value()` 変換関数
3. `value_type_name()` 型名返却関数
4. `display_value()` / `format_value()` 表示関数
5. `display_vm_value_short()` 短縮表示関数

### 既存 `parquet_write_rows` との棲み分け

既存の `Parquet.write_raw` → 行指向の書き込み（`Vec<HashMap<String, VMValue>>` をループ）
新規の `ArrowBatch.write_parquet` → Arrow RecordBatch から直接 Parquet を書き込む（ゼロコピーに近い）

### `#[arrow]` stage の意味

`TrfDef.arrow = true` のとき:
- v19.5.0 では: パース・型チェックのみ（VMはまだ通常通り実行）
- v20.x 以降: VM が自動的に入力 `List<T>` → `ArrowBatch` 変換・出力変換を行う予定
