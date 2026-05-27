# Favnir v6.6.0 仕様書 — T.validate 完成

作成日: 2026-05-27

---

## テーマ

schema 駆動のランタイムバリデーションを完全実装する。

現状の課題:
- `one_of` 制約が `FieldConstraints` にも VM にも未実装
- `TypeName.validate(record)` 構文を checker は認識するが VM が dispatch しない
- `db.query<T>` / `aws.s3.read_csv<T>` がスキーマ制約を無視する
- 統合テストが 4 件のみ（目標: 10 件以上）

---

## Phase A — `one_of` 制約の追加

### 対象ファイル
- `fav/src/schemas.rs`
- `fav/src/backend/vm.rs`

### schemas.rs の変更

`FieldConstraints` 構造体に `one_of` フィールドを追加:

```rust
pub struct FieldConstraints {
    pub constraints: Vec<String>,
    pub max_length: Option<usize>,
    pub min_length: Option<usize>,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub pattern: Option<String>,
    pub nullable: bool,
    pub one_of: Option<Vec<String>>,   // 追加
}
```

`load_schemas` で YAML の `one_of` キーをパース:

```rust
// YAML 例:
// constraints:
//   one_of: ["pending", "active", "cancelled"]
if let Some(one_of_seq) = constraints_map.get("one_of") {
    if let Some(seq) = one_of_seq.as_sequence() {
        fc.one_of = Some(seq.iter().filter_map(|v| v.as_str().map(String::from)).collect());
    }
}
```

### vm.rs の変更

`Validate.run_raw` ハンドラに `one_of` チェックを追加:

```rust
// one_of
if let Some(ref allowed) = fc.one_of {
    if !allowed.contains(val_str) {
        let mut e = HashMap::new();
        e.insert("field".into(), VMValue::Str(field_name.clone()));
        e.insert("constraint".into(), VMValue::Str("one_of".into()));
        e.insert("value".into(), VMValue::Str(val_str.clone()));
        errors.push(VMValue::Record(e));
    }
}
```

### 完了条件
- `schemas.rs` の `FieldConstraints` に `one_of: Option<Vec<String>>` がある
- YAML の `one_of: [...]` が正しくパースされる
- VM で `one_of` 違反時に ValidationError が生成される

---

## Phase B — `TypeName.validate(record)` VM dispatch

### 背景

checker は `(type_name, "validate")` を `Result<TypeName, List<ValidationError>>` 型に解決する。
しかし VM は `"TypeName.validate"` という名前のビルトインを dispatch しておらず、ランタイムエラーになる。

### 実装方針

`call_builtin` 内（または `call_builtin` の fallback）に動的 dispatch を追加:

```rust
// name = "Order.validate", "User.validate" 等
if let Some(type_name) = name.strip_suffix(".validate") {
    let schemas = SCHEMA_REGISTRY.with(|s| s.borrow().clone());
    if schemas.contains_key(type_name) {
        // args[0] を raw map として Validate.run_raw と同じロジックを実行
        return validate_record(type_name, args, &schemas);
    }
}
```

`validate_record` は `Validate.run_raw` の実装を共通関数として抽出したもの:

```rust
fn validate_record(
    type_name: &str,
    args: Vec<VMValue>,
    schemas: &ProjectSchemas,
) -> Result<VMValue, String>
```

### checker との整合

checker が `TypeName.validate(x)` を `Result<TypeName, List<ValidationError>>` と型推論するので、
VM は `Result.ok(record)` または `Result.err(List<ValidationError>)` を返す必要がある。
`ValidationError` の実体は `{ field: String  constraint: String  value: String }` レコード。

### 完了条件

```favnir
type Order = { id: Int  amount: Float }

// スキーマ: id = positive, amount = positive
bind raw <- Order { id: 1  amount: 100.0 }
match Order.validate(raw) {
  Ok(o)    => IO.println($"OK: {o.id}")
  Err(ers) => IO.println($"errors: {List.length(ers)}")
}
```
が動作する。

---

## Phase C — `db.query<T>` / `aws.s3.read_csv<T>` 自動バリデーション

### 方針

`db.query<T>` / `duckdb.query<T>` / `aws.s3.read_csv<T>` の結果行に対して、
`SCHEMA_REGISTRY` にスキーマが存在する型 `T` の場合は自動的に `Validate.run_raw` を呼ぶ。

バリデーション失敗時の挙動:
- **行単位フィルタ**: 違反行をスキップして警告（`List<T>`のうち valid なもののみ返す）
- **失敗時 Err**: 1 件でも違反があれば `Result.err` を返す（厳格モード）

v6.6.0 では **「違反があれば Err を返す」厳格モード** を採用する。
`db.query<T>` の戻り値型を `Result<List<T>, List<ValidationError>>` に変更しない点に注意:
既存コードとの互換性のため、スキーマが存在する型のみ厳格チェックを行う（オプトイン）。

### 実装箇所

`vm.rs` の `DB.query_raw` / `DuckDb.query_raw` / `AWS.S3.read_csv_raw` の
行マッピング後に以下を挿入:

```rust
// スキーマが存在する型の場合のみバリデーション
let schemas = SCHEMA_REGISTRY.with(|s| s.borrow().clone());
if let Some(type_schema) = schemas.get(type_name) {
    for row in &rows {
        let errors = validate_row_against_schema(row, type_schema);
        if !errors.is_empty() {
            return Ok(err_vm(VMValue::List(FavList::new(errors))));
        }
    }
}
```

### 完了条件

```favnir
// schemas/orders.yaml で amount: positive が設定されている場合
bind conn   <- duckdb.open(":memory:")
bind result <- duckdb.query<Order>(conn,
  "SELECT -1 AS id, -100.0 AS amount FROM (SELECT 1) t")
// result が Err(List<ValidationError>) になる
```

---

## Phase D — 統合テスト ≥ 10 件

`fav/src/backend/vm_stdlib_tests.rs` に以下のテストを追加:

| # | テスト名 | 内容 |
|---|---------|------|
| 1 | `validate_one_of_valid` | one_of 制約の許容値 → Ok |
| 2 | `validate_one_of_violation` | one_of 制約の違反値 → Err |
| 3 | `validate_type_dot_validate_ok` | `TypeName.validate` 構文 → Ok |
| 4 | `validate_type_dot_validate_err` | `TypeName.validate` 構文 → Err |
| 5 | `validate_required_field_missing` | nullable でないフィールドが null → Err |
| 6 | `validate_nullable_field_ok` | nullable フィールドが null → Ok |
| 7 | `validate_pattern_valid` | pattern 制約の許容値 → Ok |
| 8 | `validate_multi_constraint_all_pass` | 複数制約すべて通過 → Ok |
| 9 | `validate_multi_constraint_partial_fail` | 複数制約の一部失敗 → Err + 正しいフィールド名 |
| 10 | `validate_duckdb_query_schema_violation` | duckdb.query<T> + 違反データ → Err |

既存テスト（4 件）と合わせて計 14 件以上。

---

## Phase E — ドキュメント更新

### `language/schema.mdx`

`T.validate` のセクションから「v6.6.0 で完全実装予定」の Note を削除し、
完成した API として記述する:

```favnir
type Order = { id: Int  customer_name: String  email: String  amount: Float }

bind raw <- Order { id: 1  customer_name: "田中"  email: "tanaka@example.com"  amount: 12800.0 }
match Order.validate(raw) {
  Ok(order)   => IO.println($"注文 ID: {order.id}")
  Err(errors) => IO.println($"エラー: {List.length(errors)} 件")
}
```

### `stdlib/infer.mdx`

auto-validation の動作を「生成型をコードで使う」セクションに追記。

---

## 完了条件まとめ

1. `one_of` 制約が schemas.rs / vm.rs / YAML 全段階で動作する
2. `TypeName.validate(record)` が VM で実行でき `Result<T, List<ValidationError>>` を返す
3. `db.query<T>` / `duckdb.query<T>` がスキーマ違反データで `Err` を返す
4. 統合テスト ≥ 10 件がすべて通過
5. `cargo test` 全テスト通過（既存 1033 件 + 新規テスト）
6. `language/schema.mdx` から "preview" 表記を削除
