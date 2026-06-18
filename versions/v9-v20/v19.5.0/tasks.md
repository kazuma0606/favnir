# v19.5.0 — Apache Arrow 統合 タスク

## ステータス: COMPLETE

---

## タスク一覧

### T1: `fav/src/ast.rs` — `TrfDef.arrow` フィールド追加

- [x] `TrfDef` struct に `pub arrow: bool` フィールドを追加（`stateful` の直後）:
  ```rust
  /// v19.5.0: `#[arrow]` annotation — stage uses Arrow RecordBatch internally.
  pub arrow: bool,
  ```
- [x] `TrfDef` を生成しているすべての箇所（parser.rs 等）で `arrow: false` を追加
- [x] `cargo build` でコンパイルエラーが 0 であることを確認

---

### T2: `fav/src/frontend/parser.rs` — `#[arrow]` 解析追加

- [x] `parse_stateful_annotation()` の直後に `parse_arrow_annotation()` を追加:
  ```rust
  fn parse_arrow_annotation(&mut self) -> Result<bool, ParseError> {
      // Lookahead: # [ arrow ]
      let is_arrow = self.peek() == &TokenKind::Hash
          && matches!(self.tokens.get(self.pos + 1), Some(t) if t.kind == TokenKind::LBracket)
          && matches!(self.tokens.get(self.pos + 2), Some(t)
              if matches!(&t.kind, TokenKind::Ident(n) if n == "arrow"));
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
- [x] `parse_item()` で `stateful_ann` の後に `arrow_ann` を解析:
  ```rust
  let stateful_ann = self.parse_stateful_annotation()?;
  let arrow_ann = self.parse_arrow_annotation()?;   // ← 追加
  ```
- [x] `TrfDef` を構築する箇所で `arrow: arrow_ann` を設定
- [x] `cargo build` でコンパイルエラーが 0 であることを確認

---

### T3: `fav/src/backend/vm.rs` — ArrowBatch サポート追加

**3-A: `VMValue::ArrowBatch(u64)` variant 追加**

- [x] `VMValue` enum の `TxHandle` の後に追加:
  ```rust
  /// v19.5.0: Apache Arrow RecordBatch への opaque handle
  ArrowBatch(u64),
  ```
- [x] `PartialEq` の手動実装箇所に追加:
  ```rust
  (VMValue::ArrowBatch(a), VMValue::ArrowBatch(b)) => a == b,
  ```
- [x] `vm_value_to_value()` に追加:
  ```rust
  VMValue::ArrowBatch(id) => Value::Str(format!("<arrow:{id}>")),
  ```
- [x] `value_type_name()` に追加:
  ```rust
  VMValue::ArrowBatch(_) => "ArrowBatch",
  ```
- [x] `display_value()` または `format_value()` に追加:
  ```rust
  VMValue::ArrowBatch(id) => format!("<arrow:{}>", id),
  ```
- [x] `display_vm_value_short()` に追加:
  ```rust
  VMValue::ArrowBatch(_) => "<arrow>".to_string(),
  ```

**3-B: スレッドローカルストア追加**

- [x] vm.rs の先頭付近（既存 `thread_local!` の近く）に追加:
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

**3-C: vm_call_builtin に primitives 追加**

- [x] 既存の `"Gen.to_parquet_raw"` の近くに以下を追加:

  `ArrowBatch.from_list`:
  ```rust
  "ArrowBatch.from_list" => {
      let rows = match args.into_iter().next() {
          Some(VMValue::List(list)) => list.to_vec(),
          other => return Err(format!("ArrowBatch.from_list expects List, got {:?}", other)),
      };
      match arrow_from_vm_rows(&rows) {
          Ok(batch) => Ok(ok_vm(VMValue::ArrowBatch(arrow_store(batch)))),
          Err(e)    => Ok(err_vm(e)),
      }
  }
  ```

  `ArrowBatch.to_list`:
  ```rust
  "ArrowBatch.to_list" => {
      let id = match args.into_iter().next() {
          Some(VMValue::ArrowBatch(id)) => id,
          other => return Err(format!("ArrowBatch.to_list expects ArrowBatch, got {:?}", other)),
      };
      match arrow_get(id) {
          Some(batch) => {
              let rows = arrow_to_vm_rows(&batch)?;
              Ok(ok_vm(VMValue::List(FavList::new(rows))))
          }
          None => Ok(err_vm(format!("ArrowBatch: invalid handle {id}"))),
      }
  }
  ```

  `ArrowBatch.write_parquet`:
  ```rust
  "ArrowBatch.write_parquet" => {
      let mut it = args.into_iter();
      let id = match it.next() {
          Some(VMValue::ArrowBatch(id)) => id,
          other => return Err(format!("ArrowBatch.write_parquet: expected ArrowBatch, got {:?}", other)),
      };
      let path = match it.next() {
          Some(VMValue::Str(s)) => s,
          other => return Err(format!("ArrowBatch.write_parquet: expected path, got {:?}", other)),
      };
      match arrow_get(id) {
          Some(batch) => match arrow_write_parquet(&batch, &path) {
              Ok(_)  => Ok(ok_vm(VMValue::Unit)),
              Err(e) => Ok(err_vm(e)),
          },
          None => Ok(err_vm(format!("ArrowBatch.write_parquet: invalid handle {id}"))),
      }
  }
  ```

  `ArrowBatch.read_parquet`:
  ```rust
  "ArrowBatch.read_parquet" => {
      let path = match args.into_iter().next() {
          Some(VMValue::Str(s)) => s,
          other => return Err(format!("ArrowBatch.read_parquet expects String, got {:?}", other)),
      };
      match arrow_read_parquet(&path) {
          Ok(batch) => Ok(ok_vm(VMValue::ArrowBatch(arrow_store(batch)))),
          Err(e)    => Ok(err_vm(e)),
      }
  }
  ```

**3-D: ヘルパー関数追加（vm.rs 末尾付近）**

- [x] `arrow_from_vm_rows(rows: &[VMValue]) -> Result<RecordBatch, String>`
  - 空リストの場合: `RecordBatch::new_empty(Arc::new(Schema::empty()))`
  - 第 1 行目 (`VMValue::Record(m)`) からフィールド名・型を推論:
    - `VMValue::Int(_)` → `DataType::Int64`
    - `VMValue::Float(_)` → `DataType::Float64`
    - `VMValue::Bool(_)` → `DataType::Boolean`
    - それ以外 → `DataType::Utf8`
  - フィールド名をソートして列順序を決定論的にする
  - 各列の `Array`（`Int64Array` / `Float64Array` / `BooleanArray` / `StringArray`）を構築
  - `RecordBatch::try_new(schema, arrays)` で生成

- [x] `arrow_to_vm_rows(batch: &RecordBatch) -> Result<Vec<VMValue>, String>`
  - `batch.num_rows()` 行分ループ
  - 各行について `HashMap<String, VMValue>` を構築
  - `DataType::Int64` → `VMValue::Int`, `Float64` → `VMValue::Float`, `Boolean` → `VMValue::Bool`, それ以外 → `VMValue::Str`
  - null 値は `VMValue::Unit`
  - `VMValue::Record(map)` として返す

- [x] `arrow_write_parquet(batch: &RecordBatch, path: &str) -> Result<(), String>`
  - `std::fs::File::create(path)` → `ArrowWriter::try_new(file, schema, None)` → `writer.write(batch)` → `writer.close()`

- [x] `arrow_read_parquet(path: &str) -> Result<RecordBatch, String>`
  - `std::fs::File::open(path)` → `ParquetRecordBatchReaderBuilder::try_new(file).build()` → `reader.next()`

- [x] `cargo build` でコンパイルエラーが 0 であることを確認

---

### T4: exhaustive match の確認・修正

- [x] `cargo build` で `VMValue::ArrowBatch` に関する非網羅的 match エラーを確認
- [x] 発生した箇所に `VMValue::ArrowBatch(_) => { /* skip */ }` arm を追加
- [x] `cargo build` でコンパイルエラーが 0 であることを確認

---

### T5: `fav/src/driver.rs` — `v195000_tests` 追加

- [x] `v194000_tests::version_is_19_4_0` に `#[ignore]` を追加
- [x] `v195000_tests` モジュールを追加（5件）:

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
          // Map.set(Map.set((), "id", 1), "score", 10) でレコードを生成
          let value = super::exec_project_main_source_with_runes(
              r#"
  public fn main() -> Int {
      bind rows <- collect {
          yield Map.set(Map.set((), "id", 1), "score", 10);
          yield Map.set(Map.set((), "id", 2), "score", 20);
          ()
      }
      match ArrowBatch.from_list(rows) {
          Ok(_)  => 1
          Err(e) => 0
      }
  }
  "#,
          );
          assert_eq!(value, crate::value::Value::Int(1));
      }

      #[test]
      fn arrow_batch_to_list() {
          // from_list → to_list のラウンドトリップで元の行数が保持される
          let value = super::exec_project_main_source_with_runes(
              r#"
  public fn main() -> Int {
      bind rows <- collect {
          yield Map.set(Map.set((), "id", 1), "name", "Alice");
          yield Map.set(Map.set((), "id", 2), "name", "Bob");
          yield Map.set(Map.set((), "id", 3), "name", "Charlie");
          ()
      }
      match ArrowBatch.from_list(rows) {
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
          let value = super::exec_project_main_source_with_runes(
              r#"
  public fn main() -> Int {
      bind rows <- collect {
          yield Map.set(Map.set((), "id", 1), "score", 10);
          yield Map.set(Map.set((), "id", 2), "score", 20);
          ()
      }
      match ArrowBatch.from_list(rows) {
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

---

### T6: `fav/Cargo.toml` バージョン更新

- [x] `version = "19.4.0"` → `"19.5.0"` に変更
- [x] 注意: `arrow = { version = "52", features = ["ipc"] }` と `parquet = "52"` は既存なので**追加不要**

---

### T7: `site/content/docs/tools/arrow.mdx`（新規作成）

- [x] Apache Arrow の概要と列指向ストレージの利点
- [x] `ArrowBatch.from_list` / `ArrowBatch.to_list` の使い方
- [x] `ArrowBatch.write_parquet` / `ArrowBatch.read_parquet` の Parquet ゼロコピー
- [x] `#[arrow]` stage アノテーションの使い方
- [x] 通常の `List<T>` との互換性と使い分け

---

## テスト（v195000_tests、5件）

| テスト名 | 内容 |
|---|---|
| `version_is_19_5_0` | Cargo.toml に `"19.5.0"` が含まれる |
| `arrow_batch_from_list` | `ArrowBatch.from_list` が Record リストを Arrow に変換し Ok を返す |
| `arrow_batch_to_list` | `from_list` → `to_list` ラウンドトリップで元の行数 (3) が一致 |
| `arrow_parquet_roundtrip` | `write_parquet` → `read_parquet` → `to_list` で行数 (2) が一致 |
| `arrow_stage_executes` | `#[arrow]` 付き TrfDef が `trf_def.arrow == true` でパースされる |

---

## 完了条件チェックリスト

- [x] `TrfDef.arrow: bool` が ast.rs に追加される
- [x] `parse_arrow_annotation()` が parser.rs に追加される
- [x] `VMValue::ArrowBatch(u64)` が vm.rs に追加される（exhaustive match 対応済み）
- [x] `ArrowBatch.from_list` / `to_list` / `write_parquet` / `read_parquet` が vm.rs に追加される
- [x] スキーマ推論（Int/Float/Bool/Str の列型推論）が動作する
- [x] Parquet ラウンドトリップ（write → read → to_list）が動作する
- [x] `cargo test v195000` — 5/5 PASS
- [x] `cargo test` — リグレッションなし
- [x] `site/content/docs/tools/arrow.mdx` が存在する

---

## 優先度

```
T1（ast.rs — TrfDef.arrow フィールド追加）    ← 最初
T2（parser.rs — #[arrow] 解析）               ← T1 完了後
T3（vm.rs — VMValue::ArrowBatch + primitives）← T1 と並列可
T4（exhaustive match 確認・修正）              ← T3 完了後（cargo build で確認）
T5（v195000_tests 追加）                      ← T2, T3, T4 完了後
T6（Cargo.toml バージョン）                    ← T5 と並列可
T7（ドキュメント）                             ← T5 と並列可
```

---

## 重要な技術ノート

### `Map.set((), "key", value)` — Favnir でのレコード生成

Favnir にはレコードリテラル構文がない。レコードは `Map.set` で構築する:
```favnir
bind row <- Map.set(Map.set((), "id", 1), "score", 10)
// 結果: VMValue::Record({"id": VMValue::Int(1), "score": VMValue::Int(10)})
```
`()` → `VMValue::Unit` → `Map.set` では空の `HashMap::new()` として扱われる。

### `collect { yield ...; () }` — リスト生成

```favnir
bind rows <- collect {
    yield Map.set((), "id", 1);
    yield Map.set((), "id", 2);
    ()
}
// rows: VMValue::List([Record({"id": 1}), Record({"id": 2})])
```
ブロック末尾の `()` は collect の終端マーカー。

### `RecordBatch` の `clone()`

Arrow の `RecordBatch` は `Clone` を実装している（`Arc<ArrayData>` ベース）。
`ARROW_BATCHES` に挿入後、`to_list` / `write_parquet` で `cloned()` して取得できる。

### `tmp/` ディレクトリの確認

`arrow_parquet_roundtrip` テストは `tmp/arrow_roundtrip.parquet` に書き込む。
`fav/tmp/` ディレクトリが存在するか事前確認（既存の parquet テストで使用済みのため存在する）。

### `vm_call_builtin` vs `call_builtin` メソッド

ステートレスな（self 不要の）primitives は `vm_call_builtin` 自由関数に追加する。
`ArrowBatch.*` はすべてステートレスなので `vm_call_builtin` に追加する。
