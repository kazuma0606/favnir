# Favnir v1.2.0 実装計画

作成日: 2026-05-06

> スコープを守ることが最優先。各フェーズの Done definition を超えない。
>
> **前提**: v1.1.0 完了
>
> **設計ドキュメント**: `dev/post-v1/roadmap/fav-standard-states.md`、`dev/post-v1/roadmap/fav-db-schema-integration.md`

---

## 実装順序

```
Phase 0 (version bump)
  → Phase 1 (AST + Lexer + Parser)       ← 全フェーズの前提
  → Phase 2 (Checker: invariant 型検査)  ← Phase 3 の前提
  → Phase 3 (Constructor 自動生成 + VM)
  → Phase 4 (std.states ルーン)          ← Phase 3 完了後
  → Phase 5 (fav explain 更新)           ← Phase 3 と並行可
  → Phase 6 (DB スキーマ CHECK 出力)     ← Phase 5 完了後
  → Phase 7 (テスト・ドキュメント)
```

---

## Phase 0: バージョン更新

### Cargo.toml

```toml
version = "1.2.0"
```

### main.rs

```rust
const HELP: &str = "fav - Favnir language toolchain v1.2.0\n...";
```

---

## Phase 1: AST + Lexer + Parser

### 1-1: ast.rs の変更

`TypeDef` に `invariants` フィールドを追加:

```rust
// ast.rs — 現在の TypeDef に invariants を追加
pub struct TypeDef {
    pub visibility:      Option<Visibility>,
    pub name:            String,
    pub type_params:     Vec<String>,
    pub with_interfaces: Vec<String>,   // v1.1.0 で追加済み
    pub body:            TypeBody,      // TypeBody::Record(Vec<Field>) or Sum(...)
    pub invariants:      Vec<Expr>,     // 追加
    pub span:            Span,
}
```

フィールドリストは `TypeBody::Record(fields)` のパターンマッチで取得する。
`invariant` は Record 型にのみ適用可能（Sum 型は v1.2.0 ではエラー）。

### 1-2: lexer.rs の変更

```rust
// lexer.rs のキーワードマップに追加
"invariant" => Token::Invariant,
```

対応バリアントを `Token` enum に追加:

```rust
Invariant,
```

### 1-3: parser.rs の変更

#### `parse_type_def` の拡張

型ブロック内でフィールド宣言の後に `invariant` を受理する:

```rust
// "type" Name ("with" ...)? ("=" ...)? "{" field* invariant* "}"
fn parse_type_def(&mut self) -> Result<TypeDef, ParseError> {
    // ... 既存のフィールドパース処理 ...

    let mut invariants = vec![];
    // フィールドパース後に invariant を受理
    while self.peek_is(Token::Invariant) {
        self.advance(); // consume "invariant"
        let expr = self.parse_expr()?;
        invariants.push(expr);
    }

    Ok(TypeDef { ..., invariants })
}
```

**注意点**:
- `invariant` はフィールド宣言が 1 つ以上ある後にのみ受理する
- 複数の `invariant` を順次パースする（空ブロックで終了）
- `invariant` 内の `<expr>` は通常の式パース（`parse_expr`）で処理する
  - フィールド名は識別子として扱われ、型検査フェーズでスコープ解決する

#### テスト

```fav
-- パーサーテスト用
type PosInt = { value: Int  invariant value > 0 }
type UserAge = { value: Int  invariant value >= 0  invariant value <= 150 }
type Email = { value: String  invariant String.contains(value, "@") }
```

---

## Phase 2: 型検査統合

### 2-1: `check_type_def` の拡張

`middle/checker.rs` の `check_type_def` 末尾に invariant 型検査を追加:

```rust
fn check_type_def(&mut self, def: &TypeDef) -> Vec<TypeError> {
    let mut errors = self.check_fields(def);  // 既存処理

    if def.invariants.is_empty() {
        return errors;
    }

    // Record 型のフィールドをスコープに追加して invariant 式を型検査
    let fields = match &def.body {
        TypeBody::Record(fields) => fields,
        TypeBody::Sum(_) => {
            // invariant は Sum 型（ADT）には非対応（v1.2.0）
            errors.push(TypeError { code: "E045".into(), ... });
            return errors;
        }
    };

    let field_scope: HashMap<String, Type> = fields.iter()
        .map(|f| (f.name.clone(), self.resolve_type_expr(&f.ty)))
        .collect();

    let saved = self.push_local_scope(field_scope);
    for inv_expr in &def.invariants {
        match self.check_expr(inv_expr) {
            Type::Bool => {}  // OK
            other => errors.push(TypeError {
                code:    "E045".into(),
                message: format!(
                    "invariant expression must be Bool, got `{}`",
                    format_type(&other)
                ),
                span: inv_expr.span(),
            }),
        }
    }
    self.pop_local_scope(saved);

    errors
}
```

### 2-2: エラーコード定義

```rust
// 既存のエラー列挙または定数に追加
pub const E045: &str = "E045";
// メッセージ: "invariant expression must have type Bool"
```

### 2-3: `bind x: T <- expr` の展開（チェッカー）

`check_stmt` の `Stmt::Bind` 処理で、型注釈が invariant 付き型を参照する場合に chain に変換:

```rust
fn check_bind_stmt(&mut self, stmt: &BindStmt) -> Vec<TypeError> {
    if let Some(ann_ty) = &stmt.type_annotation {
        let resolved = self.resolve_type(ann_ty);
        if self.has_invariants(&resolved) {
            // bind x: T <- expr  →  chain x <- T.new(expr)
            return self.check_chain_stmt(&ChainStmt {
                name: stmt.name.clone(),
                expr: Expr::Call {
                    func: Box::new(Expr::FieldAccess {
                        receiver: Box::new(Expr::Global(resolved.name())),
                        field: "new".into(),
                    }),
                    args: vec![stmt.expr.clone()],
                },
                span: stmt.span,
            });
        }
    }
    // 型注釈なし or invariant なし型: 従来通り
    self.check_bind_stmt_normal(stmt)
}
```

---

## Phase 3: コンストラクタ自動生成 + VM

### 3-1: compiler.rs でのコンストラクタ生成

`compile_program` で `TypeDef.invariants` が空でない場合、IR 関数を追加生成する:

```rust
fn compile_type_def_constructor(&mut self, def: &TypeDef) -> IRFnDef {
    // fn TypeName.new(field1: T1, field2: T2, ...) -> TypeName! { ... }
    let fn_name = format!("{}.new", decl.name);
    // フィールドリストは TypeBody::Record から取得
    let fields = match &def.body {
        TypeBody::Record(f) => f,
        TypeBody::Sum(_) => return /* Sum 型はスキップ */,
    };
    let params: Vec<(String, Type)> = fields.iter()
        .map(|f| (f.name.clone(), self.resolve_type_expr(&f.ty)))
        .collect();
    let return_ty = Type::Result(Box::new(Type::Con(def.name.clone())));

    // IR ボディ:
    // 1. レコード構築: bind t <- TypeName { field: field_arg, ... }
    // 2. 各 invariant を if !cond { return Err("...") } で評価
    // 3. return Ok(t)
    let body = self.build_constructor_body(def, fields);

    IRFnDef {
        name:      fn_name,
        params,
        return_ty,
        body,
        effects:   vec![],
        is_public: true,
    }
}
```

**invariant 式の変数解決**:
- `invariant` 式内の識別子（フィールド名）は、コンストラクタのパラメータ変数として解決する
- `check_type_def` 時に行ったスコープ管理と対称的に処理する

### 3-2: invariant 条件チェックの IR ボディ構築

```rust
fn build_constructor_body(&self, def: &TypeDef, fields: &[Field]) -> Vec<IRStmt> {
    let mut stmts = vec![];

    // 1. レコード構築
    // bind __t <- TypeName { field1: arg1, field2: arg2 }
    stmts.push(IRStmt::Bind {
        name: "__t".into(),
        expr: IRExpr::RecordConstruct { ... },
    });

    // 2. 全 invariant を AND で結合
    let combined = def.invariants.iter()
        .cloned()
        .reduce(|a, b| Expr::BinOp { op: BinOp::And, lhs: Box::new(a), rhs: Box::new(b) })
        .unwrap();

    // 3. if !(combined) { return Err("InvariantViolation: TypeName") }
    stmts.push(IRStmt::Expr(IRExpr::If {
        cond: Box::new(IRExpr::UnOp { op: UnOp::Not, expr: Box::new(compile_expr(combined)) }),
        then: Box::new(IRExpr::Return(Box::new(
            IRExpr::Call { func: "Result.err", args: vec![
                IRExpr::Lit(Lit::Str(format!("InvariantViolation: {}", def.name)))
            ]}
        ))),
        else_: None,
    }));

    // 4. return Ok(__t)
    stmts.push(IRStmt::Return(IRExpr::Call {
        func: "Result.ok",
        args: vec![IRExpr::Local("__t".into())],
    }));

    stmts
}
```

### 3-3: 静的 invariant 検査（コンパイル時 constant folding）

`check_bind_stmt` で型注釈が invariant 付き型かつ RHS がリテラルの場合:

```rust
fn try_static_invariant_check(&self, state_type: &str, lit: &Lit) -> Option<bool> {
    // invariant 式をリテラル値で評価できる場合のみ Some を返す
    // 例: PosInt の invariant `value > 0`, lit = Int(-1) → Some(false)
    // 不明な場合（ユーザー定義関数等）→ None（ランタイムにフォールバック）
    ...
}
```

- `Some(false)` → コンパイルエラー（E001 型不一致）
- `Some(true)` → `.new()` 呼び出しを省略し直接レコード構築
- `None` → 通常の `.new()` 呼び出しに変換

### 3-4: VM の動作確認

`.new()` コンストラクタはコンパイラが生成した通常の IR 関数としてアーティファクトに収録されるため、
VM 側に追加変更は不要。`Result.ok` / `Result.err` ビルトインは v0.6.0 以降で実装済み。

---

## Phase 4: `std.states` ルーン

### 4-1: `register_stdlib_states` ヘルパー

`middle/checker.rs` の `Checker::new()` 内で呼ぶ:

```rust
fn register_stdlib_states(checker: &mut Checker) {
    // (型名, 内部フィールド型, invariant 式リスト)
    let states: &[(&str, &str, &[&str])] = &[
        ("PosInt",         "Int",    &["value > 0"]),
        ("NonNegInt",      "Int",    &["value >= 0"]),
        ("Probability",    "Float",  &["value >= 0.0", "value <= 1.0"]),
        ("PortNumber",     "Int",    &["value >= 1", "value <= 65535"]),
        ("NonEmptyString", "String", &["String.length(value) > 0"]),
        ("Email",          "String", &[
            "String.contains(value, \"@\")",
            "String.length(value) > 3",
        ]),
        ("Url",            "String", &[
            "String.starts_with(value, \"http://\") || String.starts_with(value, \"https://\")",
        ]),
        ("Slug",           "String", &[
            "String.length(value) > 0",
            // slug 正規表現は v1.2.0 では String.is_slug(value) ビルトインで代用
        ]),
    ];

    for (name, field_ty, invs) in states {
        checker.register_state_type(name, field_ty, invs);
    }
}
```

`register_state_type` は内部で `TypeDef` を合成し、コンストラクタ IR を生成する。

### 4-2: String.is_slug ビルトイン（Slug 型専用）

`Slug` の invariant は正規表現相当の検査が必要なため、`String.is_slug(s: String) -> Bool` ビルトインを追加:

```rust
// vm.rs vm_call_builtin に追加
"String.is_slug" => {
    let s = args.next()...?;
    match s {
        VMValue::Str(s) => Ok(VMValue::Bool(
            s.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
            && !s.is_empty()
        )),
        _ => Err("String.is_slug requires a String".to_string()),
    }
}
```

### 4-3: `resolver.rs` の `std.states` モジュール解決

```rust
// resolver.rs の load_module に追加
"std.states" => {
    // ビルトイン std.states モジュール
    for name in &["PosInt", "NonNegInt", "Probability", "PortNumber",
                  "NonEmptyString", "Email", "Url", "Slug"] {
        scope.add_type(name, ModuleSymbol::StdState(name));
    }
    Ok(())
}
```

---

## Phase 5: `fav explain` invariant 表示

### 5-1: `cmd_explain` の変更（driver.rs）

既存の型一覧表示ループに invariant 列を追加:

```rust
// driver.rs cmd_explain
fn format_invariants(invs: &[Expr]) -> String {
    if invs.is_empty() {
        return "—".to_string();
    }
    invs.iter()
        .map(|e| format_expr_compact(e))
        .collect::<Vec<_>>()
        .join("; ")
}

// 型行の出力
println!("{:<16} {:<28} {}", type_name, fields_str, format_invariants(&type_def.invariants));
```

### 5-2: `std.states` 型の表示

内部登録された `std.states` 型には `(stdlib)` サフィックスを付けて表示:

```
TYPE              FIELDS             INVARIANTS
PosInt (stdlib)   value: Int         value > 0
Email (stdlib)    value: String      contains "@"; length > 3
Email             value: String      contains "@"; length > 3  -- ユーザー定義なら上書き
```

---

## Phase 6: DB スキーマ CHECK 出力

### 6-1: `fav explain --schema` フラグの追加

`main.rs` の explain コマンドに `--schema` フラグを追加:

```rust
// main.rs
"explain" => {
    let schema = args.contains("--schema");
    driver::cmd_explain(&file, schema);
}
```

### 6-2: `cmd_explain_schema` の実装（driver.rs）

```rust
fn cmd_explain_schema(program: &IRProgram) {
    for type_def in &program.type_defs {
        if type_def.invariants.is_empty() { continue; }
        let checks = type_def.invariants.iter()
            .map(|e| invariant_to_sql(e))
            .collect::<Vec<_>>();
        println!("-- {}", type_def.name);
        println!("CREATE TABLE {} (", to_snake_case(&type_def.name));
        for field in &type_def.fields {
            let sql_ty = favnir_type_to_sql(&field.ty);
            print!("    {} {} NOT NULL", field.name, sql_ty);
        }
        for check in &checks {
            println!(",");
            print!("    CHECK ({})", check);
        }
        println!("\n);");
    }
}
```

### 6-3: `invariant_to_sql` 変換関数

```rust
fn invariant_to_sql(expr: &Expr) -> String {
    match expr {
        Expr::BinOp { op: BinOp::Gt,  lhs, rhs } => format!("{} > {}",  sql_expr(lhs), sql_expr(rhs)),
        Expr::BinOp { op: BinOp::Gte, lhs, rhs } => format!("{} >= {}", sql_expr(lhs), sql_expr(rhs)),
        Expr::BinOp { op: BinOp::Lt,  lhs, rhs } => format!("{} < {}",  sql_expr(lhs), sql_expr(rhs)),
        Expr::BinOp { op: BinOp::Lte, lhs, rhs } => format!("{} <= {}", sql_expr(lhs), sql_expr(rhs)),
        Expr::BinOp { op: BinOp::And, lhs, rhs } =>
            format!("{} AND {}", invariant_to_sql(lhs), invariant_to_sql(rhs)),
        Expr::BinOp { op: BinOp::Or,  lhs, rhs } =>
            format!("{} OR {}",  invariant_to_sql(lhs), invariant_to_sql(rhs)),
        // String.contains(value, s) → value LIKE '%s%'
        Expr::Call { func, args } if is_builtin(func, "String.contains") =>
            format!("{} LIKE '%{}%'", sql_expr(&args[0]), string_lit(&args[1])),
        Expr::Call { func, args } if is_builtin(func, "String.starts_with") =>
            format!("{} LIKE '{}%'", sql_expr(&args[0]), string_lit(&args[1])),
        Expr::Call { func, args } if is_builtin(func, "String.length") =>
            format!("length({})", sql_expr(&args[0])),
        _ => format!("-- [unsupported invariant: {}]", format_expr_compact(expr)),
    }
}
```

---

## Phase 7: テスト・ドキュメント

### テスト追加場所

- `middle/checker.rs` の `#[cfg(test)]` 内（型検査テスト）
- `src/integration/invariant_tests.rs`（コンストラクタ + 実行テスト）

### example ファイル

```fav
-- examples/invariant_basic.fav
type Age = { value: Int  invariant value >= 0  invariant value <= 150 }
type Score = { value: Float  invariant value >= 0.0  invariant value <= 100.0 }

public fn main() -> Unit !Io {
    -- 正常ケース
    chain age <- Age.new(30)
    IO.println_int(age.value)

    -- 違反ケース
    bind result <- Age.new(-1)
    match result {
        Ok(_)  -> IO.println("unexpected ok")
        Err(e) -> IO.println(e)
    }
}
```

```fav
-- examples/std_states.fav
use std.states.Email
use std.states.PosInt

public fn main() -> Unit !Io {
    bind ok_email    <- Email.new("user@example.com")
    bind bad_email   <- Email.new("not-an-email")
    bind ok_count    <- PosInt.new(42)
    bind bad_count   <- PosInt.new(-1)

    match ok_email  { Ok(_) -> IO.println("email ok")  Err(e) -> IO.println(e) }
    match bad_email { Ok(_) -> IO.println("unexpected") Err(e) -> IO.println(e) }
    match ok_count  { Ok(n) -> IO.println_int(n.value)  Err(e) -> IO.println(e) }
    match bad_count { Ok(_) -> IO.println("unexpected") Err(e) -> IO.println(e) }
}
```

### langspec.md 更新

`versions/v1.2.0/langspec.md` を新規作成（v1.1.0/langspec.md を起点に invariant 節を追加）:
- `invariant` 構文と意味
- `.new()` コンストラクタの挙動（`T!` を返す）
- `std.states` 型一覧
- E045 エラーコードの追記

---

## 先送り一覧

| 制約 | バージョン |
|---|---|
| `fav check --sample N`（実データの invariant 適合率） | v1.5.0 |
| LSP による State 型の提案（`bind score: _ <- 95`） | v1.5.0 以降 |
| `fav state sync`（DB スキーマ → Favnir 型自動生成） | v1.5.0 以降 |
| Invariant の静的証明（SMT ソルバー連携） | v2.0.0 以降 |
| `Invariant.min` / `Invariant.max` の合成 API | v1.3.0 以降 |
| 演算子オーバーロードの delegating（`+` → `Semigroup::combine`） | v2.0.0 |
