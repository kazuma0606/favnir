# Favnir v2.3.0 実装計画

作成日: 2026-05-13

---

## Phase 0 — バージョン更新

`Cargo.toml` を `version = "2.3.0"` に変更。
`src/main.rs` の HELP テキストを `v2.3.0` に更新。

---

## Phase 1 — 分割 bind のコンパイラ対応

### 現状の問題

`src/middle/compiler.rs` の `compile_stmt` にある `Stmt::Bind` アームは、
パターンが `Pattern::Record` の場合でも `ctx.define_pattern_slot()` で
1 つの匿名スロットを確保するだけ。フィールドごとのローカルが定義されないため、
`bind { x, y } <- point` の後で `x` や `y` を参照すると「global index out of bounds」になる。

### 修正方針

`Pattern::Record` の場合を特別処理する。中間変数を 1 スロット確保し、
各フィールドをフィールドアクセス + 個別スロットに展開する。

```rust
// compile_stmt 内の Stmt::Bind アーム（Pattern::Record の場合）
Pattern::Record(fields) => {
    // 1. 右辺を評価してスタックに積む
    compile_expr(expr, ctx)?;
    // 2. 中間スロットに格納
    let tmp_slot = ctx.define_anon_slot();
    ctx.emit(IRStmt::Bind(tmp_slot, IRExpr::Local(tmp_slot))); // ※実際は emit Pop→Store
    // 正確には：
    //   emit(IRStmt::Bind(tmp_slot, <rhs_expr_already_on_stack>))
    // ここでは rhs を IRExpr として渡す形に合わせる

    // 3. フィールドごとに個別スロットを定義
    for field in fields {
        match field {
            PatternField::Pun(name) => {
                let slot = ctx.define_local(name);
                let access = IRExpr::FieldAccess(Box::new(IRExpr::Local(tmp_slot)), name.clone());
                ctx.emit(IRStmt::Bind(slot, access));
            }
            PatternField::Alias(field_name, bind_name) => {
                let slot = ctx.define_local(bind_name);
                let access = IRExpr::FieldAccess(Box::new(IRExpr::Local(tmp_slot)), field_name.clone());
                ctx.emit(IRStmt::Bind(slot, access));
            }
            PatternField::Wildcard => { /* ignore */ }
        }
    }
}
```

### AST の `PatternField` 確認

`src/ast.rs` の `Pattern::Record` の内側がどう定義されているか確認し、
`Pun` / `Alias` / `Wildcard` に対応する。

---

## Phase 2 — E072 / E073 エラーコードの追加

### `src/middle/checker.rs`

チェッカーの `check_stmt` → `Stmt::Bind` アームで `Pattern::Record` の場合：

- 右辺型を解決する
- 右辺型が `Type::Record(fields_map)` でなければ **E072** を報告
- 各パターンフィールド名が `fields_map` に存在しなければ **E073** を報告

```rust
// 擬似コード
if let Pattern::Record(fields) = pat {
    let rhs_ty = check_expr(rhs_expr, ctx)?;
    match rhs_ty {
        Type::Record(field_map) => {
            for f in fields {
                let name = field_name(f);
                if name != "_" && !field_map.contains_key(&name) {
                    return Err(FavError::new(E073, span, format!("no field `{name}` on record type")));
                }
            }
        }
        _ => return Err(FavError::new(E072, span, "destructuring bind requires a record type")),
    }
}
```

---

## Phase 3 — 戻り型推論

### Phase 3-1: AST 変更 (`src/ast.rs`)

`FnDef.return_ty` を `Option<TypeExpr>` に変更する。

```rust
// 変更前
pub struct FnDef {
    pub name: String,
    pub params: Vec<Param>,
    pub return_ty: TypeExpr,
    pub effects: Vec<String>,
    pub body: Expr,
    ...
}

// 変更後
pub struct FnDef {
    pub name: String,
    pub params: Vec<Param>,
    pub return_ty: Option<TypeExpr>,   // None = 推論
    pub effects: Vec<String>,
    pub body: Expr,
    ...
}
```

### Phase 3-2: パーサー変更 (`src/frontend/parser.rs`)

`parse_fn_def` で `->` がある場合は従来通り型を解析し `Some(ty)` を設定。
`=` が来た場合は `return_ty = None` として本体式を直接解析。

```rust
// 変更前（parse_fn_def の戻り型解析部分）
self.expect(&TokenKind::Arrow)?;
let return_ty = self.parse_type_expr()?;

// 変更後
let return_ty = if self.peek_is(&TokenKind::Arrow) {
    self.expect(&TokenKind::Arrow)?;
    Some(self.parse_type_expr()?)
} else {
    None  // `=` が続く場合 → 戻り型推論
};
```

本体は `->` ありの場合は `{ ... }` ブロック、`=` の場合は単一式：

```rust
let body = if return_ty.is_some() {
    // 従来通りブロックを解析
    self.parse_block_expr()?
} else {
    // `=` を消費して単一式を解析
    self.expect(&TokenKind::Eq)?;
    self.parse_expr()?
};
```

### Phase 3-3: チェッカー変更 (`src/middle/checker.rs`)

`check_fn_def` で `return_ty` が `None` の場合、本体式の型を推論して戻り型として使用する。

```rust
let declared_return_ty = match &fn_def.return_ty {
    Some(ty_expr) => resolve_type_expr(ty_expr, ctx)?,
    None => {
        // 本体式を先にチェックして型を取得
        let inferred = check_expr(&fn_def.body, ctx)?;
        if inferred == Type::Unknown {
            return Err(FavError::new(E074, fn_def.span, "cannot infer return type"));
        }
        inferred
    }
};
```

### Phase 3-4: コンパイラ / コードジェン

`FnDef.return_ty` が `Option` になったため、コンパイラで参照している箇所を
`return_ty.unwrap_or(inferred)` もしくは型解決済みの値を使うよう修正。

WASM コードジェンの `IRFnDef.param_tys` と戻り型参照も同様に対応。

---

## Phase 4 — テスト追加

### `src/backend/vm_stdlib_tests.rs`

**分割 bind テスト**：

```rust
#[test]
fn test_destructure_bind_basic() {
    // bind { x, y } <- Point { x: 3  y: 4 }
    // IO.println_int(x) → 3
    // IO.println_int(y) → 4
    let src = r#"
        type Point = { x: Int  y: Int }
        public fn main() -> Unit !Io {
            bind pt <- Point { x: 3  y: 4 }
            bind { x, y } <- pt
            IO.println_int(x)
            IO.println_int(y)
        }
    "#;
    // run_program(src) → output contains "3\n4\n"
}

#[test]
fn test_destructure_bind_alias() {
    // bind { age: user_age } <- user で user_age が使える
}

#[test]
fn test_destructure_bind_wildcard() {
    // bind { name, _ } <- user で name だけ束縛できる
}
```

**戻り型推論テスト**：

```rust
#[test]
fn test_return_type_inference_int() {
    // fn double(n: Int) = n * 2 が型チェック通り double(5) → Int(10)
}

#[test]
fn test_return_type_inference_string() {
    // fn greet(name: String) = $"Hello {name}!" が型チェック通り
}

#[test]
fn test_return_type_inference_bool() {
    // fn is_adult(age: Int) = age >= 18
}
```

### `src/middle/checker.rs`

**E072 / E073 テスト**：

```rust
#[test]
fn test_e072_destructure_bind_non_record() {
    // bind { x } <- 42 → E072
}

#[test]
fn test_e073_destructure_bind_missing_field() {
    // type P = { x: Int }; bind { x, y } <- p → E073 (y が存在しない)
}
```

---

## Phase 5 — ドキュメント

- `versions/v2.3.0/langspec.md` を作成
  - 分割 bind 構文の完全説明
  - エイリアス・ワイルドカードの構文
  - 戻り型推論のルールと制約（再帰不可）
  - E072 / E073 / E074 エラーコード一覧

---

## テスト数の見込み

v2.2.0 ベースライン: 567

- 分割 bind VM テスト: +3
- 戻り型推論 VM テスト: +3
- チェッカー E072/E073 テスト: +2
- 合計目標: **575**（+8 程度）

---

## 注意点

### 分割 bind のコンパイラ実装

`ctx.define_anon_slot()` と `ctx.define_local(name)` の両方を使う。
中間スロットへの格納は `IRStmt::Bind(tmp_slot, rhs_ir_expr)` で行い、
フィールドアクセスは `IRExpr::FieldAccess(Box::new(IRExpr::Local(tmp_slot)), field.to_string())` で生成する。

### 戻り型推論の再帰関数制限

パーサー段階では再帰かどうか判定できないため、
チェッカーで推論時に `Type::Unknown` が返ってきたら E074 を報告して終わらせる。
再帰関数で `->` を書き忘れた場合のエラーメッセージには `hint: add explicit return type for recursive functions` を添える。

### AST 変更の波及

`FnDef.return_ty` を `Option<TypeExpr>` にすると以下の箇所が影響を受ける：
- `fmt.rs` — フォーマッタ（`None` のとき `-> ty` を出力しない）
- `wasm_codegen.rs` — `IRFnDef` の戻り型参照
- `compiler.rs` — `compile_fn_def` の戻り型参照

漏れなく対応すること。
