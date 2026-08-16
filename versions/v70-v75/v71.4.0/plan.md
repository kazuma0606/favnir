# v71.4.0 実装計画 — Const / Compile-Time Evaluation

---

## Step 1: AST 拡張（`fav/src/ast.rs`）

### 1a. `ConstDef` 構造体を追加

`SchemaDef` の直前などに追加:

```rust
/// `const NAME: Type = expr` — コンパイル時定数宣言（v71.4.0）
#[derive(Debug, Clone)]
pub struct ConstDef {
    pub name: String,
    pub ty: TypeExpr,
    pub value: Expr,
    pub span: Span,
}
```

### 1b. `Item::ConstDef` バリアントを追加

`Item::SchemaDef(SchemaDef)` の直前に追加:

```rust
ConstDef(ConstDef),  // v71.4.0: const NAME: Type = expr
```

### 1c. `Item::span()` に `ConstDef` アームを追加

```rust
Item::ConstDef(c) => &c.span,
```

### 1d. `TypeExpr::ConstName` バリアントを追加

`TypeExpr::ConstInt` の直後に追加:

```rust
ConstName(String, Span),  // v71.4.0: 型位置の定数名参照（Vec<Float>[EMBED_DIM]）
```

`TypeExpr::span()` に:
```rust
TypeExpr::ConstName(_, s) => s,
```

`cargo build` でエラー箇所（未処理 arm）を列挙し、次の Step 以降で対応。

---

## Step 2: パーサー（`fav/src/frontend/parser.rs`）

### 2a. `parse_base_type` の `[N]` 解析を拡張

現在: `LBracket` → 整数リテラルを `ConstInt` に変換。
変更後: 識別子（`TokenKind::Ident(name)`）の場合は `ConstName(name, span)` を返す。

```rust
// parse_base_type 内 [N] サフィックス処理
if self.check(&TokenKind::LBracket) {
    self.advance();
    let dim_span = self.current_span();
    let dim = match self.peek() {
        TokenKind::Int(n) => {
            let n = *n;
            self.advance();
            TypeExpr::ConstInt(n, dim_span)
        }
        TokenKind::Ident(name) => {
            let name = name.clone();
            self.advance();
            TypeExpr::ConstName(name, dim_span)  // v71.4.0
        }
        _ => return Err(self.error("expected integer or const name in [...]")),
    };
    self.expect(&TokenKind::RBracket)?;
    // dim を型引数に組み込む（既存ロジック）
    ...
}
```

### 2b. `parse_item` に `const` ブランチ追加

`const` は文脈キーワード（識別子として認識）なので `TokenKind::Ident("const")` でマッチ:

```rust
TokenKind::Ident(kw) if kw == "const" => {
    self.advance();
    Ok(Item::ConstDef(self.parse_const_def(start)?))
}
```

### 2c. `parse_const_def` 新規関数

```rust
fn parse_const_def(&mut self, start: usize) -> Result<ConstDef, ParseError> {
    let name = self.expect_ident()?;
    self.expect(&TokenKind::Colon)?;
    let ty = self.parse_type_expr()?;
    self.expect(&TokenKind::Eq)?;
    let value = self.parse_expr()?;
    Ok(ConstDef { name, ty, value, span: self.span_from(&start) })
}
```

`cargo test` で既存テストが pass することを確認。

---

## Step 3: チェッカー — const_env フィールド追加（`fav/src/middle/checker.rs`）

### 3a. `Checker` 構造体に `const_env` フィールド追加

```rust
const_env: HashMap<String, StaticValue>,  // v71.4.0: コンパイル時定数の値
```

`Checker::new()` と `new_with_resolver()` の初期化に `const_env: HashMap::new()` を追加。

---

## Step 4: チェッカー — const pre-pass（`register_item_signatures`）

`register_item_signatures` の先頭（alias invariant pre-pass の直後）に const pre-pass を追加:

**重要**: `eval_static_expr` のシグネチャは `&HashMap<String, Lit>` であり `StaticValue` ではない。
定数間参照（`const HALF_DIM: Int = EMBED_DIM / 2`）を解決するため、const pre-pass は
宣言順に定数を評価し、評価済みの値を `Lit` に変換して増分マップ `values` に追加する。

```rust
// v71.4.0: const pre-pass — 定数値をコンパイル時に評価して const_env に登録
// 宣言順に評価するため後方参照は不可（E0250 となる）
let mut const_lit_values: HashMap<String, Lit> = HashMap::new();
for item in &program.items {
    if let Item::ConstDef(cd) = item {
        let expected_ty = self.resolve_type_expr(&cd.ty);
        match self.eval_static_expr(&cd.value, &const_lit_values) {
            Some(val) => {
                // StaticValue → Lit 変換（次の定数の eval で参照用）
                let as_lit = match &val {
                    StaticValue::Int(n)    => Lit::Int(*n),
                    StaticValue::Float(f)  => Lit::Float(*f),
                    StaticValue::Bool(b)   => Lit::Bool(*b),
                    StaticValue::String(s) => Lit::Str(s.clone()),
                    StaticValue::Unit      => Lit::Unit,
                };
                // 型整合性チェック
                let actual_ty = match &val {
                    StaticValue::Int(_) => Type::Int,
                    StaticValue::Float(_) => Type::Float,
                    StaticValue::Bool(_) => Type::Bool,
                    StaticValue::String(_) => Type::String,
                    StaticValue::Unit => Type::Unit,
                };
                if !actual_ty.is_compatible(&expected_ty) {
                    self.type_error("E0250",   // E0248/E0249 は既存用途あり
                        format!("const `{}`: expected `{}`, got `{}`",
                            cd.name, expected_ty.display(), actual_ty.display()),
                        &cd.span);
                } else {
                    const_lit_values.insert(cd.name.clone(), as_lit);
                    self.const_env.insert(cd.name.clone(), val);
                    // 定数を環境にも定義（式位置での参照用）
                    self.env.define(cd.name.clone(), actual_ty);
                }
            }
            None => {
                self.type_error("E0250",
                    format!("const `{}` value cannot be evaluated at compile time (check for undefined forward references)", cd.name),
                    &cd.span);
            }
        }
    }
}
```

---

## Step 5: チェッカー — `ConstName` の次元解決と `resolve_type_expr` 対応

### 5a. `resolve_type_expr` に `ConstName` アームを追加

型位置で `ConstName` が出現したら `Type::Int` を返す（型推論用）:

```rust
TypeExpr::ConstName(name, span) => {
    if !self.const_env.contains_key(name) {
        self.type_error("E0247",
            format!("undefined const `{}`", name),
            span);
    }
    Type::Int  // const は現時点で Int のみ次元として有効
}
```

### 5b. 次元比較箇所（checker.rs ~line 4919）に `ConstName` 解決を追加

`Vec<T>[N]` の次元チェックは checker.rs の `type_args.get(i)` で `ConstInt` を抽出している箇所で行われる。
ここに `ConstName` ブランチを追加し、`const_env` から値を解決する:

```rust
// 既存: ConstInt の場合
let Some(crate::ast::TypeExpr::ConstInt(n, _)) = type_args.get(i) else { continue };

// 変更後: ConstInt または ConstName の両方に対応
let dim_val: i64 = match type_args.get(i) {
    Some(TypeExpr::ConstInt(n, _)) => *n,
    Some(TypeExpr::ConstName(name, span)) => {
        match self.const_env.get(name) {
            Some(StaticValue::Int(n)) => *n,
            _ => {
                self.type_error("E0247", format!("undefined const `{}`", name), span);
                continue;
            }
        }
    }
    _ => continue,
};
```

**この方式であれば `is_dim_annotated_name_mismatch` を変更する必要はない**:
次元値が `i64` として解決された後は既存の文字列エンコード（`Vec#1536` 形式）を通して比較できる。

---

## Step 6: fmt.rs — ConstName と ConstDef フォーマット

`TypeExpr::ConstName` の arm:
```rust
TypeExpr::ConstName(name, _) => format!("{}", name),
```

トップレベル const のフォーマット（`format_item` や同等関数）:
```rust
Item::ConstDef(cd) => {
    format!("const {}: {} = {}", cd.name, self.type_expr(&cd.ty), self.expr(&cd.value))
}
```

---

## Step 7: その他ファイル — コンパイルエラー解消

`cargo build` でエラーになる全箇所に `TypeExpr::ConstName` arm を追加:

- `fav/src/middle/compiler.rs`: `TypeExpr::ConstName(_, _) => Type::Int`
- `fav/src/middle/ast_lower_checker.rs`: `TypeExpr::ConstName(name, _) => v1("TeConstName", Value::Str(name.clone()))`
- `fav/src/lint.rs`: `TypeExpr::ConstName(..) => {}` — 警告対象外
- `fav/src/emit_python.rs`: `TypeExpr::ConstName(_, _) => "int".to_string()`
- `fav/src/driver.rs`: 各 `ty_to_str` 系 match に `TypeExpr::ConstName(n, _) => n.to_string()` または `=> "int"`

`Item::ConstDef` のコンパイルエラー箇所にも `| Item::ConstDef(_) => {}` などを追加。

**注意: 以下のファイルはワイルドカード `_ => {}` があるためコンパイルエラーにならないが確認が必要:**
- `fav/src/lsp/references.rs` — `collect_in_item` の `_ => {}` により `ConstDef` は自動スキップ（定数が参照検索から除外される）。現時点ではスキップのまま許容。
- `fav/src/lineage.rs` — `Item::ConstDef` を透過（skip）することを確認し、必要なら `Item::ConstDef(_) => {}` を明示追加。

`cargo build` が通ることを確認。

---

## Step 8: driver.rs — `v714000_tests` 追加

```rust
#[cfg(test)]
mod v714000_tests {
    use super::*;

    // テスト 1: const 宣言 + 算術式コンパイル時評価
    #[test]
    fn const_eval_int_expr() {
        let src = r#"
const EMBED_DIM: Int = 1536
const HALF_DIM: Int = EMBED_DIM / 2

fn get_half() -> Int { HALF_DIM }
"#;
        let result = parse_and_check(src, "test.fav");
        assert!(result.errors.is_empty(),
            "const eval should succeed: {:?}", result.errors);
    }

    // テスト 2: const を依存型次元パラメータとして使用
    #[test]
    fn const_used_in_dependent_type() {
        let src = r#"
const EMBED_DIM: Int = 1536

fn embed(text: String) -> Vec<Float>[EMBED_DIM] {
    Vec.empty()
}
"#;
        let result = parse_and_check(src, "test.fav");
        assert!(result.errors.is_empty(),
            "const in dependent type should resolve: {:?}", result.errors);
    }
}
```

`cargo test v714000` で 2 件 pass を確認。

---

## Step 9: Cargo.toml バージョン更新

- `fav/Cargo.toml` の `version` を `"71.3.0"` → `"71.4.0"` に更新
- `driver.rs` 内の `"71.3.0"` 文字列リテラルを `"71.4.0"` に一括更新（`replace_all: true`）

---

## Step 10: CHANGELOG.md 更新

```markdown
## [v71.4.0] — 2026-08-09 — Const / Compile-Time Evaluation

### Added
- `v714000_tests`: 2 件追加（3592 → 3594 tests）
  - `const_eval_int_expr`
  - `const_used_in_dependent_type`
- AST: `ConstDef` 構造体・`Item::ConstDef`・`TypeExpr::ConstName` 追加
- パーサー: `const NAME: Type = expr` 構文サポート（文脈キーワード `const`）
- パーサー: 依存型 `Vec<T>[NAME]` の次元位置で定数名を `ConstName` としてパース
- チェッカー: `const_env: HashMap<String, StaticValue>` — コンパイル時定数評価（宣言順）
- チェッカー: E0247（未定義定数参照）・E0250（定数型不一致）追加
- `error_catalog.rs`: E0247・E0250 エントリ追加
- `fmt.rs`: `ConstName`・`Item::ConstDef` のフォーマットサポート
```

---

## Step 11: versions/current.md 更新

- 「進行中バージョン」を `v71.4.0`（Const / Compile-Time Evaluation）に更新
- 「次に切る版」を `v71.5.0` に更新

---

## Step 12: 最終確認

- `cargo test v714000` で 2 件 pass
- `cargo test` 全体で 3594 件 pass（0 failures）
- `fav/Cargo.toml` が `71.4.0`
- `versions/current.md` が正しく更新されている
