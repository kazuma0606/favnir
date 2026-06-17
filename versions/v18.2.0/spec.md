# v18.2.0 — 行多相（Row Polymorphism）仕様

## 概要

「このフィールドを持つ任意のレコード型」を型安全に扱える汎用関数を書けるようにする。
`fn f<R with { id: Int }>(row: R)` の構文で「id: Int フィールドを持つ任意のレコード R」を受け取れる。

---

## 1. 構文

### 1.1 レコード型制約付きジェネリクス

```fav
// 「id: Int フィールドを持つ任意のレコード R」を受け取る
fn get_id<R with { id: Int }>(row: R) -> Int {
  row.id
}

// 複数フィールド制約
fn validate_user<R with { id: Int, email: String }>(row: R) -> Result<R, String> {
  if String.contains(row.email, "@")
    { Result.ok(row) }
  else
    { Result.err(f"invalid email: {row.email}") }
}
```

### 1.2 交差型 `R & { field: Type }`（戻り型）

スプレッド構文（`{ ...row, field: val }`）は**値の位置のみ**。
戻り型の位置には `&`（交差型）を使う。

```fav
// ✅ 型の位置では & を使う
fn add_timestamp<R with { id: Int }>(row: R) -> R & { timestamp: String } {
  { ...row, timestamp: DateTime.format_iso(DateTime.now()) }
}

// ❌ 型の位置でのスプレッドは使わない（パースエラー）
// fn add_timestamp<R with { id: Int }>(row: R) -> { ...R, timestamp: String }
```

### 1.3 使用例

```fav
// 異なるレコード型に同じ関数を適用できる
bind user_with_ts  <- add_timestamp({ id: 1, name: "Alice" })
bind order_with_ts <- add_timestamp({ id: 42, amount: 100.0 })
// user_with_ts の型:  { id: Int, name: String } & { timestamp: String }
// order_with_ts の型: { id: Int, amount: Float } & { timestamp: String }

// パイプラインでの活用
stage Enrich<R with { id: Int, created_at: String }>(
  rows: List<R>
) -> List<R & { age_days: Int }> {
  Result.ok(List.map(rows, |row| {
    bind age <- DateTime.diff_days(DateTime.parse(row.created_at), DateTime.now())
    { ...row, age_days: age }
  }))
}
```

---

## 2. 新しい AST ノード

### 2.1 `TypeConstraint::HasField`

```rust
// ast.rs の GenericParam.bounds に追加
pub enum TypeConstraint {
    Interface(String),                           // 既存: `with Ord`
    HasField { name: String, ty: TypeExpr },     // 新規: `with { id: Int }`
}
```

`GenericParam.bounds` は既存 `Vec<String>` から `Vec<TypeConstraint>` に変更。

### 2.2 `Type::Intersection`

```rust
// checker.rs の Type enum に追加
pub enum Type {
    // ...既存...
    /// `R & { field: Type }` — intersection type (v18.2.0)
    Intersection(Box<Type>, Box<Type>),
}
```

---

## 3. パーサー変更

### 3.1 `parse_type_bounds` 拡張

`with { ... }` 形式を認識:

```
fn f<T with Ord>(...)      → TypeConstraint::Interface("Ord")
fn f<R with { id: Int }>(...)  → TypeConstraint::HasField { name: "id", ty: Int }
fn f<R with { id: Int, email: String }>(...)  → 複数 HasField
```

### 3.2 `parse_type` 拡張（交差型）

型の位置で `T & U` を認識:

```
R & { timestamp: String }
→ Type::Intersection(Type::Var("R"), Type::Record([("timestamp", Type::String)]))
```

`&` トークンは既存の `TokenKind::Amp`（ビット AND 演算子）を型文脈で流用。

---

## 4. 型チェッカー変更

### 4.1 `check_row_constraint`

`fn f<R with { id: Int }>(row: R)` の呼び出し時:
- 実際の引数型（レコード型）が制約フィールドを含むか確認
- 含まない場合 → E0337（行多相制約違反）

```rust
fn check_row_constraint(
    actual: &Type,
    constraint: &HasFieldConstraint,
) -> bool
```

### 4.2 `check_intersection_type`

`R & { timestamp: String }` の型解決:
- `R` が具体的なレコード型の場合 → フィールドをマージした具体型を返す
- `R` が型変数の場合 → `Type::Intersection` として保持

### 4.3 E0337 エラー

```
E0337: row constraint violated: type `{ name: String }` does not have field `id: Int`
```

---

## 5. `GenericParam.bounds` の変更影響

`GenericParam.bounds` が `Vec<String>` → `Vec<TypeConstraint>` に変わるため、
参照箇所を全て更新する必要がある:

| ファイル | 変更内容 |
|---|---|
| `ast.rs` | `GenericParam.bounds: Vec<TypeConstraint>` |
| `frontend/parser.rs` | `parse_type_params` で `TypeConstraint` を生成 |
| `middle/checker.rs` | bounds チェックで `TypeConstraint` を処理 |
| `middle/compiler.rs` | `compile_fn_def` で `GenericParam.bounds` イテレーション |
| `fmt.rs` | `GenericParam` のフォーマット |
| `emit_python.rs` | Python emit での bounds スキップ |
| `lineage.rs` | bounds 参照箇所（あれば） |

---

## 6. エラーコード

| コード | 説明 |
|---|---|
| `E0337` | 行多相制約違反（制約フィールドが型に存在しない） |

---

## 7. テスト（v182000_tests、5件）

| テスト名 | 内容 |
|---|---|
| `version_is_18_2_0` | Cargo.toml に "18.2.0" が含まれる |
| `row_poly_single_field` | `fn f<R with { id: Int }>(row: R) -> Int` が型チェックを通る |
| `row_poly_different_records` | 異なるレコード型（UserRow / OrderRow）に同じ関数を適用できる |
| `row_poly_intersection_return` | `-> R & { ts: String }` 戻り型が AST として解析される |
| `row_poly_field_missing` | 制約フィールドがない型を渡すと E0337 になる |

---

## 8. 完了条件

- [ ] `fn f<R with { id: Int }>(row: R) -> R` が型チェックを通る
- [ ] 異なるフィールドを持つ複数のレコード型に同じ関数を適用できる
- [ ] `-> R & { timestamp: String }` 交差型が AST として解析される
- [ ] 制約フィールドを持たない型を渡すと E0337 になる
- [ ] 複数フィールド制約 `R with { id: Int, email: String }` が動作する
- [ ] `cargo test v182000` — 5/5 PASS
- [ ] `cargo test` — リグレッションなし
