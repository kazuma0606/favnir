# v19.5.0 実装計画 — Apache Arrow 統合

## 実装順序

```
T1（ast.rs — TrfDef.arrow フィールド追加）       ← 最初
T2（parser.rs — #[arrow] 解析）                  ← T1 完了後
T3（vm.rs — VMValue::ArrowBatch + primitives）   ← T2 と並列可
T4（lib.rs / main.rs）                           ← 不要（vm.rs 内で完結）
T5（driver.rs — v195000_tests）                  ← T2, T3 完了後
T6（Cargo.toml バージョン）                       ← T5 と並列可
T7（ドキュメント）                                ← T5 と並列可
```

---

## T1: `fav/src/ast.rs` — `TrfDef.arrow` フィールド追加

`TrfDef` の `stateful: bool` の後に `arrow: bool` を追加する:

```rust
pub struct TrfDef {
    pub visibility: Option<Visibility>,
    pub is_async: bool,
    pub name: String,
    pub type_params: Vec<GenericParam>,
    pub input_ty: TypeExpr,
    pub output_ty: TypeExpr,
    pub effects: Vec<Effect>,
    pub params: Vec<Param>,
    pub body: Block,
    /// v19.1.0: `#[stateful]` annotation — stage maintains state between chunks.
    pub stateful: bool,
    /// v19.5.0: `#[arrow]` annotation — stage uses Arrow RecordBatch internally.
    pub arrow: bool,
    pub span: Span,
}
```

`TrfDef` を生成しているすべての箇所で `arrow: false` を追加する（デフォルト）。

---

## T2: `fav/src/frontend/parser.rs` — `#[arrow]` 解析

`parse_stateful_annotation()` の直後に追加:

```rust
fn parse_arrow_annotation(&mut self) -> Result<bool, ParseError> {
    // Lookahead: # [ arrow ]
    let is_arrow = self.peek() == &TokenKind::Hash
        && matches!(self.tokens.get(self.pos + 1), Some(t) if t.kind == TokenKind::LBracket)
        && matches!(self.tokens.get(self.pos + 2), Some(t) if matches!(&t.kind, TokenKind::Ident(n) if n == "arrow"));
    if !is_arrow {
        return Ok(false);
    }
    self.advance(); // #
    self.advance(); // [
    self.advance(); // arrow
    self.expect(&TokenKind::RBracket)?;
    Ok(true)
}
```

`parse_item()` の `streaming_ann` / `stateful_ann` の後に追加:

```rust
let streaming_ann = self.parse_streaming_annotation()?;
let stateful_ann = self.parse_stateful_annotation()?;
let arrow_ann = self.parse_arrow_annotation()?;  // ← 追加
```

`TrfDef` を構築する箇所で `arrow: arrow_ann` を設定する。

---

## T3: `fav/src/backend/vm.rs` — Arrow サポート

### 3-A: `VMValue::ArrowBatch(u64)` 追加

`VMValue` enum に追加:

```rust
/// v19.5.0: Apache Arrow RecordBatch への opaque handle
ArrowBatch(u64),
```

以下の match 箇所に arm を追加:

```rust
// PartialEq (手動実装の箇所)
(VMValue::ArrowBatch(a), VMValue::ArrowBatch(b)) => a == b,

// vm_value_to_value()
VMValue::ArrowBatch(id) => Value::Str(format!("<arrow:{id}>")),

// value_type_name() 型名
VMValue::ArrowBatch(_) => "ArrowBatch",

// display_value() / format_value() 表示
VMValue::ArrowBatch(id) => format!("<arrow:{}>", id),

// display_vm_value_short()
VMValue::ArrowBatch(_) => "<arrow>".to_string(),
```

### 3-B: スレッドローカルストア

ファイル先頭付近（既存の `thread_local!` ブロックの近く）に追加:

```rust
thread_local! {
    static ARROW_BATCHES: std::cell::RefCell<
        std::collections::HashMap<u64, arrow::record_batch::RecordBatch>
    > = std::cell::RefCell::new(std::collections::HashMap::new());
    static NEXT_ARROW_ID: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

fn arrow_store(batch: arrow::record_batch::RecordBatch) -> u64 {
    NEXT_ARROW_ID.with(|c| {
        let id = c.get();
        c.set(id + 1);
        ARROW_BATCHES.with(|m| m.borrow_mut().insert(id, batch));
        id
    })
}

fn arrow_get(id: u64) -> Option<arrow::record_batch::RecordBatch> {
    ARROW_BATCHES.with(|m| m.borrow().get(&id).cloned())
}
```

### 3-C: `vm_call_builtin` に primitives 追加

既存の `"Parquet.*"` や `"Gen.to_parquet_raw"` の近くに追加:

```rust
"ArrowBatch.from_list" => {
    // args[0]: VMValue::List(rows of VMValue::Record)
    // returns: Result<VMValue::ArrowBatch(id), String>
    let rows = match args.into_iter().next() {
        Some(VMValue::List(list)) => list.to_vec(),
        other => return Err(format!("ArrowBatch.from_list expects List, got {:?}", other)),
    };
    match arrow_from_vm_rows(&rows) {
        Ok(batch) => {
            let id = arrow_store(batch);
            Ok(ok_vm(VMValue::ArrowBatch(id)))
        }
        Err(e) => Ok(err_vm(e)),
    }
}

"ArrowBatch.to_list" => {
    let id = match args.into_iter().next() {
        Some(VMValue::ArrowBatch(id)) => id,
        other => return Err(format!("ArrowBatch.to_list expects ArrowBatch, got {:?}", other)),
    };
    match arrow_get(id) {
        Some(batch) => {
            let rows = arrow_to_vm_rows(&batch)?;
            Ok(ok_vm(VMValue::List(FavList::from_vec(rows))))
        }
        None => Ok(err_vm(format!("ArrowBatch: invalid handle {id}"))),
    }
}

"ArrowBatch.write_parquet" => {
    let mut iter = args.into_iter();
    let id = match iter.next() {
        Some(VMValue::ArrowBatch(id)) => id,
        other => return Err(format!("ArrowBatch.write_parquet: expected ArrowBatch, got {:?}", other)),
    };
    let path = match iter.next() {
        Some(VMValue::Str(s)) => s,
        other => return Err(format!("ArrowBatch.write_parquet: expected String path, got {:?}", other)),
    };
    match arrow_get(id) {
        Some(batch) => {
            arrow_write_parquet(&batch, &path)
                .map(|_| ok_vm(VMValue::Unit))
                .unwrap_or_else(|e| ok_vm(VMValue::Variant("err".into(), Some(Box::new(VMValue::Str(e))))))
        }
        None => Ok(err_vm(format!("ArrowBatch.write_parquet: invalid handle {id}"))),
    }
}

"ArrowBatch.read_parquet" => {
    let path = match args.into_iter().next() {
        Some(VMValue::Str(s)) => s,
        other => return Err(format!("ArrowBatch.read_parquet expects String, got {:?}", other)),
    };
    match arrow_read_parquet(&path) {
        Ok(batch) => {
            let id = arrow_store(batch);
            Ok(ok_vm(VMValue::ArrowBatch(id)))
        }
        Err(e) => Ok(err_vm(e)),
    }
}
```

### 3-D: ヘルパー関数（vm.rs 末尾付近）

```rust
/// List<Record> → RecordBatch 変換（スキーマを第 1 行目から推論）
fn arrow_from_vm_rows(
    rows: &[VMValue],
) -> Result<arrow::record_batch::RecordBatch, String> {
    use arrow::array::*;
    use arrow::datatypes::*;

    if rows.is_empty() {
        let schema = std::sync::Arc::new(Schema::empty());
        return RecordBatch::new_empty(schema)
            .map_err(|e| format!("arrow empty batch: {e}"));
    }

    // 第 1 行目からフィールド名・型を推論
    let first = match &rows[0] {
        VMValue::Record(m) => m,
        other => return Err(format!("ArrowBatch.from_list: expected Record rows, got {:?}", other)),
    };

    let mut field_names: Vec<String> = first.keys().cloned().collect();
    field_names.sort(); // 決定論的な列順序

    let fields: Vec<Field> = field_names.iter().map(|name| {
        let sample = first.get(name).unwrap();
        let dtype = match sample {
            VMValue::Int(_)   => DataType::Int64,
            VMValue::Float(_) => DataType::Float64,
            VMValue::Bool(_)  => DataType::Boolean,
            _                 => DataType::Utf8,
        };
        Field::new(name.as_str(), dtype, true)
    }).collect();

    let schema = std::sync::Arc::new(Schema::new(fields.clone()));

    // 各列の配列を構築
    let arrays: Vec<std::sync::Arc<dyn Array>> = fields.iter().map(|field| {
        let name = field.name();
        match field.data_type() {
            DataType::Int64 => {
                let vals: Vec<Option<i64>> = rows.iter().map(|r| {
                    match r { VMValue::Record(m) => match m.get(name) {
                        Some(VMValue::Int(n)) => Some(*n),
                        _ => None,
                    }, _ => None }
                }).collect();
                std::sync::Arc::new(Int64Array::from(vals)) as _
            }
            DataType::Float64 => {
                let vals: Vec<Option<f64>> = rows.iter().map(|r| {
                    match r { VMValue::Record(m) => match m.get(name) {
                        Some(VMValue::Float(f)) => Some(*f),
                        _ => None,
                    }, _ => None }
                }).collect();
                std::sync::Arc::new(Float64Array::from(vals)) as _
            }
            DataType::Boolean => {
                let vals: Vec<Option<bool>> = rows.iter().map(|r| {
                    match r { VMValue::Record(m) => match m.get(name) {
                        Some(VMValue::Bool(b)) => Some(*b),
                        _ => None,
                    }, _ => None }
                }).collect();
                std::sync::Arc::new(BooleanArray::from(vals)) as _
            }
            _ => { // Utf8
                let vals: Vec<Option<String>> = rows.iter().map(|r| {
                    match r { VMValue::Record(m) => m.get(name).map(|v| match v {
                        VMValue::Str(s) => s.clone(),
                        other => format!("{:?}", other),
                    }), _ => None }
                }).collect();
                std::sync::Arc::new(StringArray::from(
                    vals.iter().map(|s| s.as_deref()).collect::<Vec<_>>()
                )) as _
            }
        }
    }).collect();

    RecordBatch::try_new(schema, arrays)
        .map_err(|e| format!("arrow RecordBatch::try_new: {e}"))
}

/// RecordBatch → List<Record> 変換
fn arrow_to_vm_rows(
    batch: &arrow::record_batch::RecordBatch,
) -> Result<Vec<VMValue>, String> {
    use arrow::array::*;

    let schema = batch.schema();
    let num_rows = batch.num_rows();
    let mut result = Vec::with_capacity(num_rows);

    for row_idx in 0..num_rows {
        let mut record: std::collections::HashMap<String, VMValue> =
            std::collections::HashMap::new();

        for (col_idx, field) in schema.fields().iter().enumerate() {
            let col = batch.column(col_idx);
            let val = match field.data_type() {
                arrow::datatypes::DataType::Int64 => {
                    let arr = col.as_any().downcast_ref::<Int64Array>().unwrap();
                    if arr.is_null(row_idx) { VMValue::Unit }
                    else { VMValue::Int(arr.value(row_idx)) }
                }
                arrow::datatypes::DataType::Float64 => {
                    let arr = col.as_any().downcast_ref::<Float64Array>().unwrap();
                    if arr.is_null(row_idx) { VMValue::Unit }
                    else { VMValue::Float(arr.value(row_idx)) }
                }
                arrow::datatypes::DataType::Boolean => {
                    let arr = col.as_any().downcast_ref::<BooleanArray>().unwrap();
                    if arr.is_null(row_idx) { VMValue::Unit }
                    else { VMValue::Bool(arr.value(row_idx)) }
                }
                _ => {
                    let arr = col.as_any().downcast_ref::<StringArray>().unwrap();
                    if arr.is_null(row_idx) { VMValue::Unit }
                    else { VMValue::Str(arr.value(row_idx).to_string()) }
                }
            };
            record.insert(field.name().clone(), val);
        }

        result.push(VMValue::Record(record));
    }

    Ok(result)
}

/// RecordBatch → Parquet ファイル書き込み
fn arrow_write_parquet(
    batch: &arrow::record_batch::RecordBatch,
    path: &str,
) -> Result<(), String> {
    use parquet::arrow::arrow_writer::ArrowWriter;
    let file = std::fs::File::create(path)
        .map_err(|e| format!("arrow_write_parquet create: {e}"))?;
    let mut writer = ArrowWriter::try_new(file, batch.schema(), None)
        .map_err(|e| format!("ArrowWriter::try_new: {e}"))?;
    writer.write(batch)
        .map_err(|e| format!("ArrowWriter::write: {e}"))?;
    writer.close()
        .map_err(|e| format!("ArrowWriter::close: {e}"))?;
    Ok(())
}

/// Parquet ファイル → RecordBatch 読み込み
fn arrow_read_parquet(
    path: &str,
) -> Result<arrow::record_batch::RecordBatch, String> {
    use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
    let file = std::fs::File::open(path)
        .map_err(|e| format!("arrow_read_parquet open: {e}"))?;
    let mut reader = ParquetRecordBatchReaderBuilder::try_new(file)
        .map_err(|e| format!("ParquetRecordBatchReaderBuilder: {e}"))?
        .build()
        .map_err(|e| format!("build reader: {e}"))?;
    reader.next()
        .ok_or_else(|| "arrow_read_parquet: no batches in file".to_string())?
        .map_err(|e| format!("read batch: {e}"))
}
```

---

## T5: `fav/src/driver.rs` — `v195000_tests` 追加

### `v194000_tests::version_is_19_4_0` に `#[ignore]` を追加

### `v195000_tests` モジュール（5件）

```rust
// ── v195000_tests (v19.5.0) — Apache Arrow 統合 ──────────────────────────────
#[cfg(test)]
mod v195000_tests {
    use crate::frontend::parser::Parser;

    #[test]
    fn version_is_19_5_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("19.5.0"), "Cargo.toml should have version 19.5.0");
    }

    #[test]
    fn arrow_batch_from_list() {
        // ArrowBatch.from_list が Record のリストを正しく変換する
        // collect { yield ... } パターンで行を作成し、from_list で変換
        let value = super::exec_single_source_with_runes(
            r#"
fn main() -> Int {
    bind rows <- collect {
        yield { id: 1, name: "Alice" };
        yield { id: 2, name: "Bob" };
        ()
    }
    bind batch <- ArrowBatch.from_list(rows)
    match batch {
        Ok(_)  => 1
        Err(_) => 0
    }
}
"#,
        );
        assert_eq!(value, crate::value::Value::Int(1));
    }

    #[test]
    fn arrow_batch_to_list() {
        // from_list → to_list のラウンドトリップで元の行数が保持される
        let value = super::exec_single_source_with_runes(
            r#"
fn main() -> Int {
    bind rows <- collect {
        yield { id: 1, name: "Alice" };
        yield { id: 2, name: "Bob" };
        yield { id: 3, name: "Charlie" };
        ()
    }
    bind batch <- ArrowBatch.from_list(rows)
    match batch {
        Err(_) => -1
        Ok(b) =>
            match ArrowBatch.to_list(b) {
                Err(_)   => -2
                Ok(back) => List.length(back)
            }
    }
}
"#,
        );
        assert_eq!(value, crate::value::Value::Int(3));
    }

    #[test]
    fn arrow_parquet_roundtrip() {
        // write_parquet → read_parquet → to_list で行数が一致する
        let value = super::exec_single_source_with_runes(
            r#"
fn main() -> Int {
    bind rows <- collect {
        yield { id: 1, score: 10 };
        yield { id: 2, score: 20 };
        ()
    }
    bind batch <- ArrowBatch.from_list(rows)
    match batch {
        Err(_) => -1
        Ok(b) =>
            match ArrowBatch.write_parquet(b, "tmp/arrow_roundtrip.parquet") {
                Err(_) => -2
                Ok(_) =>
                    match ArrowBatch.read_parquet("tmp/arrow_roundtrip.parquet") {
                        Err(_) => -3
                        Ok(b2) =>
                            match ArrowBatch.to_list(b2) {
                                Err(_)   => -4
                                Ok(back) => List.length(back)
                            }
                    }
            }
    }
}
"#,
        );
        assert_eq!(value, crate::value::Value::Int(2));
    }

    #[test]
    fn arrow_stage_executes() {
        // #[arrow] アノテーション付き TrfDef が arrow: true でパースされる
        let src = r#"
#[arrow]
stage Transform(rows: List<Int>) -> List<Int> {
    Result.ok(rows)
}
"#;
        let prog = Parser::parse_str(src, "test.fav").expect("parse");
        let trf = prog.items.iter().find_map(|item| {
            if let crate::ast::Item::TrfDef(t) = item { Some(t) } else { None }
        }).expect("TrfDef not found");
        assert!(trf.arrow, "TrfDef with #[arrow] should have arrow: true");
    }
}
```

**注意**: `exec_single_source_with_runes` が存在しない場合は `exec_project_main_source_with_runes` に合わせた呼び出しパターンを使う（既存テストを参照して適切な関数名を確認）。

---

## T6: `fav/Cargo.toml` バージョン更新

`version = "19.4.0"` → `"19.5.0"`

---

## T7: `site/content/docs/tools/arrow.mdx` 作成

- Apache Arrow とは何か（列指向ストレージの利点）
- `ArrowBatch.from_list` / `to_list` の使い方
- `ArrowBatch.write_parquet` / `read_parquet` の Parquet ゼロコピー
- `#[arrow]` stage アノテーション
- 通常の `List<T>` との互換性

---

## 注意点

### exhaustive match が必要な箇所

`VMValue::ArrowBatch(u64)` を追加すると、vm.rs 内のいくつかの match がコンパイルエラーになる可能性がある。`cargo build` のエラーを確認して必要箇所に arm を追加する。

### `FavList` vs `Vec<VMValue>`

`ArrowBatch.to_list` の戻り値は `VMValue::List(FavList::from_vec(rows))` とする。
`FavList::from_vec(v)` が存在することを事前に確認する（または `FavList::new(v, 0)` を使う）。

### `collect { yield ...; ... }` 構文

Favnir の `collect { yield expr; ... }` はリストを生成するブロック式。
最後の `()` は collect ブロックの終端を示す（`yield` の後に `()` で終了）。
