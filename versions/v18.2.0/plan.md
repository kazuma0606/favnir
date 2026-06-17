# v18.2.0 — 実装計画

## 方針

- `GenericParam.bounds` の型変更が波及範囲最大（7〜10ファイル）。最初に実施。
- `Type::Intersection` は checker.rs の enum 拡張のみ。表示・型チェックを追加する。
- 行多相チェック（E0337）は call-site で引数型と制約を照合するシンプルな実装。
- v18.2.0 では「フィールドの存在」チェックのみ対応（フィールドの不在制約は v19.x 以降）。
- テストは Favnir ソースをパースして Checker を直接呼ぶ形で実装。

---

## 実装ステップ

### Step 1: `fav/src/ast.rs` — `TypeConstraint` enum 追加 + `GenericParam.bounds` 変更

**変更内容:**

1. `TypeConstraint` enum を追加:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum TypeConstraint {
    /// `with Ord` — interface bound (v17.1.0, 既存)
    Interface(String),
    /// `with { id: Int, email: String }` — record field constraints (v18.2.0)
    HasField { name: String, ty: TypeExpr },
}
```

2. `GenericParam.bounds` を変更:

```rust
// 変更前:
pub struct GenericParam {
    pub name: String,
    pub bounds: Vec<String>,   // "Ord" など
}

// 変更後:
pub struct GenericParam {
    pub name: String,
    pub bounds: Vec<TypeConstraint>,
}
```

3. `Type::Intersection` を追加:

```rust
pub enum Type {
    // ...既存...
    /// `R & { field: Type }` — intersection type (v18.2.0)
    Intersection(Box<Type>, Box<Type>),
}
```

4. `Type::display()` に `Intersection` case 追加:
   - `format!("{} & {}", lhs.display(), rhs.display())`

**影響箇所（コンパイルエラー起点）:**
- `parser.rs`: `bounds.push(name_str)` → `bounds.push(TypeConstraint::Interface(name_str))`
- `checker.rs`: `fn_bounds_registry: HashMap<String, Vec<GenericParam>>` の参照箇所で `.bounds` を処理している `type_implements_bound` 関数
- `compiler.rs`: `fn_def.type_params` / `p.bounds` のイテレーション
- `fmt.rs`: `GenericParam` フォーマット
- `emit_python.rs`: bounds スキップ処理

---

### Step 2: `fav/src/frontend/parser.rs` — `parse_type_bounds` / `parse_type` 拡張

**変更内容:**

#### `parse_type_bounds` 拡張

既存: `with Ord` → `TypeConstraint::Interface("Ord")`

追加: `with { id: Int, email: String }` → 複数 `TypeConstraint::HasField`

```rust
fn parse_type_bounds(&mut self) -> Vec<TypeConstraint> {
    let mut bounds = Vec::new();
    if self.peek() == &TokenKind::With {
        self.advance(); // consume `with`
        if self.peek() == &TokenKind::LBrace {
            // Record constraint: `with { field: Type, ... }`
            self.advance(); // consume `{`
            loop {
                let name = self.expect_ident("field name")?;
                self.expect(TokenKind::Colon)?;
                let ty = self.parse_type_expr()?;
                bounds.push(TypeConstraint::HasField { name, ty });
                if self.peek() == &TokenKind::RBrace { break; }
                if self.peek() == &TokenKind::Comma { self.advance(); }
            }
            self.expect(TokenKind::RBrace)?;
        } else {
            // Interface bound: `with Ord`
            let name = self.expect_ident("bound name")?;
            bounds.push(TypeConstraint::Interface(name));
        }
    }
    bounds
}
```

#### `parse_type_expr` 拡張（交差型）

型パース後に `&` が続く場合は `Type::Intersection` に:

```rust
// parse_type_expr の末尾で:
let lhs = parse_type_expr_inner()?;
if self.peek() == &TokenKind::Amp {
    self.advance(); // consume `&`
    let rhs = self.parse_type_expr()?;
    return Ok(TypeExpr::Intersection(Box::new(lhs), Box::new(rhs)));
}
lhs
```

`TypeExpr::Intersection` を AST に追加（checker が `Type::Intersection` に解決）。

---

### Step 3: `fav/src/middle/checker.rs` — 行多相チェック追加

**変更内容:**

#### `type_implements_bound` 変更

```rust
fn type_implements_bound(&self, ty: &Type, bound: &TypeConstraint) -> bool {
    match bound {
        TypeConstraint::Interface(name) => {
            // 既存実装
            match name.as_str() {
                "Ord" | "Eq" => matches!(ty, Type::Int | Type::Float | Type::String | Type::Bool),
                // ...
            }
        }
        TypeConstraint::HasField { name, ty: field_ty } => {
            // レコード型がフィールドを持つか確認
            self.type_has_field(ty, name, field_ty)
        }
    }
}
```

#### `type_has_field` 追加

```rust
fn type_has_field(&self, ty: &Type, field_name: &str, _field_ty: &TypeExpr) -> bool {
    match ty {
        Type::Record(fields) => fields.iter().any(|(n, _)| n == field_name),
        Type::Named(name, _) => {
            // 定義済みレコード型のフィールドを確認
            if let Some(fields) = self.record_fields.get(name) {
                fields.iter().any(|(n, _)| n == field_name)
            } else {
                false
            }
        }
        _ => false,
    }
}
```

#### call-site での E0337 チェック

`check_expr` の `Expr::Apply` → `check_generic_bounds_at_call` で:
- `TypeConstraint::HasField` の場合は `type_has_field` を呼ぶ
- 失敗したら E0337

#### `resolve_type_expr` に `TypeExpr::Intersection` 追加

```rust
TypeExpr::Intersection(lhs, rhs) => {
    let l = self.resolve_type_expr(lhs);
    let r = self.resolve_type_expr(rhs);
    Type::Intersection(Box::new(l), Box::new(r))
}
```

---

### Step 4: 波及ファイル更新

`GenericParam.bounds: Vec<TypeConstraint>` への変更に伴い、以下を更新:

#### `fav/src/middle/compiler.rs`

`compile_fn_def` 内の `type_params` 処理で bounds イテレーション:
```rust
// 変更前: p.bounds.iter().any(|b| ...)
// 変更後: p.bounds.iter().any(|b| matches!(b, TypeConstraint::Interface(_) | TypeConstraint::HasField { .. }))
```

型パラメータ名のみ取り出す箇所: `p.name.clone()` はそのまま（bounds は使わない）

#### `fav/src/fmt.rs`

```rust
// GenericParam の pretty-print
// 変更前: if !p.bounds.is_empty() { write!(" with {}", p.bounds.join(" + ")); }
// 変更後:
if !p.bounds.is_empty() {
    write!(f, " with ")?;
    for (i, b) in p.bounds.iter().enumerate() {
        if i > 0 { write!(f, " + ")?; }
        match b {
            TypeConstraint::Interface(name) => write!(f, "{}", name)?,
            TypeConstraint::HasField { name, ty } => write!(f, "{{ {}: {} }}", name, fmt_type_expr(ty))?,
        }
    }
}
```

#### `fav/src/emit_python.rs`

bounds は Python emit では無視（型消去）。イテレーション箇所のみコンパイルエラー修正。

#### `fav/src/lineage.rs`

bounds 参照箇所（`GenericParam` を扱う場合）のコンパイルエラーを修正。

---

### Step 5: `fav/src/driver.rs` — `v182000_tests` 追加

`v181000_tests` の `version_is_18_1_0` テストを削除し、新しい 5 件を追加:

```rust
mod v182000_tests {
    #[test]
    fn version_is_18_2_0() { ... }

    #[test]
    fn row_poly_single_field() {
        // fn f<R with { id: Int }>(row: R) -> Int { row.id }
        // 型チェックが通る
        let src = r#"
fn get_id<R with { id: Int }>(row: R) -> Int {
  row.id
}
"#;
        let prog = Parser::parse_str(src, "test.fav").expect("parse");
        let (errors, _) = Checker::check_program(&prog);
        assert!(errors.is_empty(), "row_poly_single_field should pass: {:?}", errors);
    }

    #[test]
    fn row_poly_different_records() {
        // 異なるレコード型に同じ関数を適用
        let src = r#"
fn get_id<R with { id: Int }>(row: R) -> Int {
  row.id
}
fn main() -> Int {
  bind a <- get_id({ id: 1, name: "Alice" })
  bind b <- get_id({ id: 2, score: 99.0 })
  a + b
}
"#;
        let prog = Parser::parse_str(src, "test.fav").expect("parse");
        let (errors, _) = Checker::check_program(&prog);
        assert!(errors.is_empty(), "row_poly_different_records should pass: {:?}", errors);
    }

    #[test]
    fn row_poly_intersection_return() {
        // `-> R & { ts: String }` がパース・型チェックを通る
        let src = r#"
fn add_ts<R with { id: Int }>(row: R) -> R & { ts: String } {
  { ...row, ts: "2026-01-01" }
}
"#;
        let prog = Parser::parse_str(src, "test.fav").expect("parse");
        // パースが通ることを確認（型チェックエラーなし）
        assert!(!prog.items.is_empty(), "program should parse");
    }

    #[test]
    fn row_poly_field_missing() {
        // 制約フィールドがない型を渡すと E0337
        let src = r#"
fn get_id<R with { id: Int }>(row: R) -> Int {
  row.id
}
fn main() -> Int {
  get_id({ name: "no id field" })
}
"#;
        let prog = Parser::parse_str(src, "test.fav").expect("parse");
        let (errors, _) = Checker::check_program(&prog);
        let has_e0337 = errors.iter().any(|e| e.code == "E0337");
        assert!(has_e0337, "expected E0337 for missing field, got: {:?}", errors);
    }
}
```

---

### Step 6: バージョン更新

- `fav/Cargo.toml`: `18.1.0` → `18.2.0`
- `cargo build` で `Cargo.lock` 更新

---

### Step 7: ドキュメント

`site/content/docs/language/row-polymorphism.mdx` 新規作成:
- `fn f<R with { id: Int }>(row: R)` の基本構文
- 交差型 `R & { field: Type }` と spread 値の使い分け
- 複数フィールド制約の例
- パイプラインでの活用例

---

## 依存関係グラフ

```
Step 1 (ast.rs — TypeConstraint / Intersection)
    |
Step 2 (parser.rs)              ← Step 1 の TypeConstraint 必須
    |
Step 3 (checker.rs)             ← Steps 1, 2 必須
Step 4 (compiler/fmt/emit)      ← Step 1 必須、Steps 2-3 と並列可
    |
Step 5 (v182000_tests)         ← Steps 1-4 すべて完了後
    |
Step 6 (バージョン更新)
Step 7 (ドキュメント)           ← Step 6 と並列可
```

---

## 注意事項

- `GenericParam.bounds: Vec<String>` → `Vec<TypeConstraint>` の変更は破壊的変更。
  コンパイルエラーを出して網羅的に修正するアプローチを取る（コンパイルエラー駆動）。

- `TypeExpr::Intersection` を AST に追加する場合、`TypeExpr` の exhaustive match 箇所を全て更新:
  - `parser.rs`（型解析）
  - `checker.rs`（`resolve_type_expr`、`validate_type_expr_arity`）
  - `fmt.rs`（型フォーマット）

- `row_poly_intersection_return` テストは「パースが通る」ことのみ確認（型チェックの完全な整合性は v18.x 後半で強化）。

- `Type::Intersection` の `display()` / `is_compatible()` を実装しないと既存テストが失敗する可能性。
  `is_compatible` では `Intersection(R, extra)` を `R のフィールド + extra のフィールドを持つ型` として処理。
  簡易実装: `Intersection` は任意の型と compatible（型チェック上は保守的に通す）。

- `Type` enum への `Intersection` 追加は `Type` の exhaustive match 箇所を全て更新:
  - `checker.rs` で `Type` を match している箇所（多数）
  - `fmt.rs` の `display`
  - `compiler.rs` の型変換箇所
