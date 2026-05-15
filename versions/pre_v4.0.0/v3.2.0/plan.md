# Favnir v3.2.0 Implementation Plan

## Overview

v3.2.0 は `csv` + `json` rune を導入する。
中心設計は **`Schema.adapt<T>`** — チェッカーが型情報を埋め込み、
VM が文字列マップを型付きレコードに変換する VM プリミティブ。

Total phases: 8

---

## Phase 0: Version Bump

**Goal**: バージョン文字列を `3.2.0` に更新する。

- `fav/Cargo.toml`: `version = "3.2.0"`
- `cargo build` で `env!("CARGO_PKG_VERSION")` 伝播を確認
- `fav --version` が `favnir 3.2.0` を表示することを確認

---

## Phase 1: `SchemaError` 型 + `FieldAttr` AST 拡張

**Goal**: 型システムと AST に必要な基盤を追加する。

### 1-A: `SchemaError` 型 (`checker.rs`)

- `Type::SchemaError` を型システムに追加（または `Type::Record` として扱う）
- チェッカーが `SchemaError` フィールドを認識して型チェックできるようにする
- `Schema` を stdlib namespace として登録

### 1-B: `FieldAttr` AST 拡張 (`ast.rs`, `frontend/parser.rs`)

フィールド定義にアノテーションを追加:
```rust
pub struct FieldDef {
    pub name: String,
    pub ty:   TypeExpr,
    pub attrs: Vec<FieldAttr>,   // 新規
}

pub struct FieldAttr {
    pub name: String,            // "col"
    pub arg:  Option<String>,    // "0", "1", ...
}
```

- パーサーで型定義フィールド前の `#[name(arg)]` 構文を解析
- アノテーションなしのフィールドは `attrs: vec![]`
- `#[col(0)]` のみ実装（他は無視して警告なし）

テスト (`src/frontend/`):
- `parse_field_with_col_attr` — `#[col(0)] id: Int` をパースして attr を確認
- `parse_type_with_multiple_col_attrs` — 複数フィールドに `#[col(n)]`

---

## Phase 2: 型メタデータシステム (`middle/compiler.rs`, `backend/vm.rs`)

**Goal**: `Schema.adapt` が実行時に型情報を参照できるよう、
コンパイラが型定義メタデータを IR に埋め込む仕組みを作る。

### 2-A: `TypeMeta` IR ノード (`src/ir.rs`)

```rust
pub struct FieldMeta {
    pub name:       String,
    pub ty:         String,    // "Int", "Float", "Bool", "String", "Option<Int>" 等
    pub col_index:  Option<usize>,  // #[col(n)] から
}

pub struct TypeMeta {
    pub type_name: String,
    pub fields:    Vec<FieldMeta>,
}
```

- `compiler.rs`: 型定義ノードをコンパイルする際に `TypeMeta` を収集
- `Artifact` に `type_metas: HashMap<String, TypeMeta>` を追加

### 2-B: `type_name_of<T>()` 組み込み形式

- チェッカーで `type_name_of<ConcreteType>()` を特殊形式として認識
- コンパイラで文字列リテラル（型名）に置換
- `Schema.adapt(raw, type_name_of<Row>())` → `Schema.adapt(raw, "Row")`

テスト:
- `type_meta_is_embedded_in_artifact` — 型を持つ .fav をコンパイルして `type_metas` を確認

---

## Phase 3: CSV VM プリミティブ (`backend/vm.rs`)

**Goal**: CSV テキストを処理する VM ビルトイン 4 種を実装する。

### 3-A: `Csv.parse_raw`

```rust
"Csv.parse_raw" => {
    // args: [text: String, delimiter: String, has_header: Bool]
    // returns: Result<List<Map<String,String>>, SchemaError>
    // 実装: csv クレートを使用
}
```

- `csv = "1"` は Cargo.toml に追加済み
- ヘッダーあり: ヘッダー行をキーとして各行をマップ化
- ヘッダーなし: `"0"`, `"1"`, ... をキーとする
- CSV パースエラー → `VMValue::Err(SchemaError { field: "", expected: "valid csv", got: msg })`

### 3-B: `Csv.write_raw`

```rust
"Csv.write_raw" => {
    // args: [rows: List<Map<String,String>>, delimiter: String]
    // returns: String
}
```

- キーの順序は最初の行のキー順で固定

### 3-C: `Schema.adapt`

```rust
"Schema.adapt" => {
    // args: [rows: List<Map<String,String>>, type_name: String]
    // artifact.type_metas から TypeMeta を検索して変換
    // returns: Result<List<T>, SchemaError>
}
```

- `has_col_attrs` の場合: `col_index` でポジション参照
- `!has_col_attrs` の場合: フィールド名でキー参照

### 3-D: `Schema.to_csv`

```rust
"Schema.to_csv" => {
    // args: [rows: List<VMValue::Record>, type_name: String]
    // returns: String
}
```

テスト (`backend/vm_stdlib_tests.rs`):
- `csv_parse_raw_header` — ヘッダーあり CSV をパースして行数・値を検証
- `csv_parse_raw_no_header` — ヘッダーなし CSV をパースして位置キーを確認
- `csv_write_raw_produces_correct_text`
- `schema_adapt_int_field` — Int フィールドの変換
- `schema_adapt_float_field`
- `schema_adapt_bool_field`
- `schema_adapt_option_field_none` — 空文字列 → `None`
- `schema_adapt_option_field_some` — 非空文字列 → `Some`
- `schema_adapt_type_mismatch_returns_err` — 変換失敗で `SchemaError`

---

## Phase 4: JSON VM プリミティブ (`backend/vm.rs`)

**Goal**: JSON テキストを処理する VM ビルトイン 4 種を実装する。

### 4-A: `Json.parse_raw` / `Json.parse_array_raw`

```rust
"Json.parse_raw" => {
    // args: [text: String]
    // returns: Result<Map<String,String>, SchemaError>
    // serde_json で Value::Object をパース → 値を文字列化してマップ化
}

"Json.parse_array_raw" => {
    // args: [text: String]
    // returns: Result<List<Map<String,String>>, SchemaError>
}
```

- `serde_json` は Cargo.toml に追加済み
- ネスト値は `serde_json::to_string` でシリアライズして文字列に平坦化

### 4-B: `Json.write_raw` / `Json.write_array_raw`

```rust
"Json.write_raw" => {
    // args: [map: Map<String,String>]
    // returns: String  (JSON オブジェクト文字列)
}

"Json.write_array_raw" => {
    // args: [rows: List<Map<String,String>>]
    // returns: String  (JSON 配列文字列)
}
```

### 4-C: `Schema.adapt_one` / `Schema.to_json` / `Schema.to_json_array`

`Schema.adapt_one`: `Map<String,String>` + type_name → `Result<T, SchemaError>`
`Schema.to_json`: `Record` + type_name → JSON オブジェクト文字列
`Schema.to_json_array`: `List<Record>` + type_name → JSON 配列文字列

テスト:
- `json_parse_raw_basic_object`
- `json_parse_array_raw_basic`
- `json_write_raw_produces_object`
- `json_write_array_raw_produces_array`
- `schema_adapt_one_from_json`

---

## Phase 5: `runes/csv/csv.fav` + `runes/json/json.fav`

**Goal**: Favnir 製 rune ファイルを作成し、VM プリミティブの薄いラッパーとして実装する。

### 5-A: `runes/csv/csv.fav`

- `public stage parse<T>` — `Csv.parse_raw` + `Schema.adapt` の合成
- `public stage parse_positional<T>` — `#[col(n)]` 対応版
- `public stage write<T>` — `Schema.to_csv`
- `public stage parse_with_opts<T>` — `CsvOptions` 対応版

### 5-B: `runes/csv/csv.test.fav`

- `test_parse_header_csv` — ヘッダーあり CSV → 型付きリスト
- `test_parse_positional_csv` — `#[col(n)]` CSV → 型付きリスト
- `test_write_csv` — 型付きリスト → CSV 文字列
- `test_parse_type_mismatch_returns_err` — SchemaError の確認
- `test_parse_option_field` — nullable フィールドの `Option<T>`
- `test_chain_integration` — `|> chain` で中断するケース

### 5-C: `runes/json/json.fav`

- `public stage parse<T>` — `Json.parse_raw` + `Schema.adapt_one`
- `public stage parse_list<T>` — `Json.parse_array_raw` + `Schema.adapt`
- `public stage write<T>` — `Schema.to_json`
- `public stage write_list<T>` — `Schema.to_json_array`

### 5-D: `runes/json/json.test.fav`

- `test_parse_object` — JSON オブジェクト → 型付き値
- `test_parse_list` — JSON 配列 → 型付きリスト
- `test_write_object` — 型付き値 → JSON 文字列
- `test_write_list` — 型付きリスト → JSON 配列文字列
- `test_parse_error_invalid_json` — 不正 JSON で SchemaError

---

## Phase 6: checker.rs / compiler.rs 型チェック統合

**Goal**: `csv.parse<T>` / `json.parse<T>` の型チェックを通す。

- `Schema.adapt` / `Schema.adapt_one` の戻り型: `Result<List<T>, SchemaError>`
- `type_name_of<T>()` の型推論: `String` として扱う
- E0501〜E0505 エラーコードを `error_catalog.rs` に追加
- `#[col(n)]` の `n` が非負整数でなければ E0503（コンパイル時）
- `#[col(n)]` と名前ベースフィールドが混在する型は E0501（一貫性エラー）

---

## Phase 7: サンプル + driver.rs 統合テスト

**Goal**: エンドツーエンドで動く examples と Rust 側統合テストを追加する。

### 7-A: `examples/csv_demo/`

```
examples/csv_demo/
  main.fav        — CSV 読み込み → 加工 → CSV 書き出し
  users.csv       — サンプルデータ
```

### 7-B: `examples/json_demo/`

```
examples/json_demo/
  main.fav        — JSON 読み込み → 型付き操作 → JSON 書き出し
  config.json     — サンプルデータ
```

### 7-C: driver.rs 統合テスト (`csv_json_tests` モジュール)

- `csv_parse_and_write_roundtrip` — パース→書き出し→パースで元データと一致
- `json_parse_and_write_roundtrip` — 同上
- `csv_schema_error_propagates` — chain と組み合わせたパイプラインでエラー伝播
- `json_schema_error_on_type_mismatch`
- `col_annotation_maps_by_position`
- `option_field_maps_empty_to_none`

---

## Phase 8: ドキュメント

**Goal**: バージョンドキュメントを作成する。

- `versions/v3.2.0/langspec.md` — v3.1.0 langspec に csv/json rune、Schema.adapt、#[col(n)] を追記
- `versions/v3.2.0/migration-guide.md` — v3.1.0 → v3.2.0（破壊的変更なし；新機能のみ）
- `versions/v3.2.0/progress.md` — 全 Phase `[x]`

---

## 依存関係グラフ

```
Phase 0 (version)
    └── Phase 1 (SchemaError + FieldAttr AST)
            └── Phase 2 (TypeMeta + type_name_of)
                    ├── Phase 3 (CSV VM prims)
                    │       └── Phase 5-A,B (csv.fav + tests)
                    ├── Phase 4 (JSON VM prims)
                    │       └── Phase 5-C,D (json.fav + tests)
                    └── Phase 6 (checker統合)
                            └── Phase 7 (examples + integration tests)
                                    └── Phase 8 (docs)
```

Phase 3 と Phase 4 は Phase 2 完了後に並行開発可能。
