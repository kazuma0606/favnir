# Favnir v0.1.0 実装計画

更新日: 2026-04-26

## 方針

- 実装言語: Rust (host + frontend)
- 実行方式: tree-walking インタープリタ
- 目標: 純粋な pipeline を型安全に書いて動かせること

各フェーズは前のフェーズが完成してから進める。
ただし、AST 定義 (Phase 2) は Parser (Phase 1) と並行して進めてよい。

---

## Phase 1: Lexer

**目標**: `.fav` ソースを Token 列に変換する。

### 設計方針

- 手書き Lexer (外部ライブラリ不使用)
- ソース位置 (ファイル名・行・列) を Token に付与する
- エラー時は位置付きのメッセージを返す

### 対象トークン

```
// キーワード
type fn trf flw bind match if else

// 記号
<- |> | -> ! ? : , . { } ( ) [ ] = _

// リテラル
INT FLOAT STRING BOOL

// 識別子
IDENT

// 特殊
EOF
```

### 成果物

- `src/lexer.rs`
- `Token` 型 (種別 + span)
- `Span` 型 (ファイル名 + 開始位置 + 終了位置)
- 基本的なエラー型

---

## Phase 2: AST 定義

**目標**: Favnir の構文木を Rust の型として定義する。

### 設計方針

- すべての AST ノードに `Span` を持たせる
- 型注釈は構文ノードとして表現し、型検査前は `Option` で持つ

### 主なノード

```rust
// トップレベル
enum Item {
    TypeDef(TypeDef),
    FnDef(FnDef),
    TrfDef(TrfDef),
    FlwDef(FlwDef),
    Bind(BindStmt),
}

// 型式
enum TypeExpr {
    Named(Ident, Vec<TypeExpr>),  // List<T>
    Optional(Box<TypeExpr>),      // T?
    Fallible(Box<TypeExpr>),      // T!
    Arrow(Box<TypeExpr>, Box<TypeExpr>), // A -> B
}

// 式
enum Expr {
    Literal(Literal),
    Ident(Ident),
    Pipeline(Vec<Expr>),
    Apply(Box<Expr>, Vec<Expr>),
    Block(Vec<BlockItem>, Box<Expr>),
    Match(Box<Expr>, Vec<MatchArm>),
    If(Box<Expr>, Box<Expr>, Option<Box<Expr>>),
    Closure(Vec<Ident>, Box<Expr>),
}

// パターン
enum Pattern {
    Wildcard,
    Literal(Literal),
    Bind(Ident),
    Variant(Ident, Option<Box<Pattern>>),
    Record(Vec<FieldPattern>),
}
```

### 成果物

- `src/ast.rs`

---

## Phase 3: Parser

**目標**: Token 列を AST に変換する。

### 設計方針

- 手書き再帰下降パーサ
- Pratt parsing を `|>` 演算子の結合に使う
- エラー回復は最小限 (最初のエラーで停止してよい)

### 優先して実装するもの

1. `type` 定義 (record / sum)
2. `fn` 定義
3. `trf` 定義
4. `flw` 定義
5. `bind <-` 束縛
6. 式 (リテラル, 識別子, 関数適用, `|>`)
7. `match` 式
8. `if` 式
9. クロージャ
10. block

### 成果物

- `src/parser.rs`
- エラー型の拡充

---

## Phase 4: 型チェック

**目標**: AST に型を付与し、型の不整合を検出する。

### 設計方針

- 型環境 `TyEnv` は `Map<Name, Type>` のシンプルな構造
- 単相型推論のみ (多相は対応しない)
- `trf` を `Trf<Input, Output, Fx>` として表現する
- `|>` の合成では Output-Input の一致を検査する
- `match` では各アームの型が一致することを確認する

### 型の内部表現

```rust
enum Type {
    Bool,
    Int,
    Float,
    String,
    Unit,
    List(Box<Type>),
    Map(Box<Type>, Box<Type>),
    Option(Box<Type>),
    Result(Box<Type>, Box<Type>),
    Arrow(Box<Type>, Box<Type>),
    Trf(Box<Type>, Box<Type>, Effect),
    Named(String, Vec<Type>),
    Var(u32),  // 型変数 (単相推論用)
}

enum Effect {
    Pure,
    Io,
    Db,
    Network,
    Emit(Box<Type>),
    Union(Vec<Effect>),
}
```

### effect 合成規則

```
Pure + X   = X
Io + Io    = Io
Emit<A> + Emit<B> = Emit<A | B>
```

### 成果物

- `src/checker.rs`
- `src/types.rs`
- Typed AST (または型注釈を付与した AST)

---

## Phase 5: インタープリタ

**目標**: Typed AST を実行する。

### 設計方針

- tree-walking インタープリタ
- 環境 `Env` は `Map<Name, Value>` のリンクリスト (lexical scope)
- 値は Rust の `enum Value` で表現する
- `Pure` と `Io` effect のみ実行対応

### 値の表現

```rust
enum Value {
    Bool(bool),
    Int(i64),
    Float(f64),
    String(Rc<str>),
    Unit,
    List(Rc<Vec<Value>>),
    Map(Rc<HashMap<Value, Value>>),
    Closure(Env, Vec<Ident>, Expr),
    Trf(TrfDef),
    Variant(String, Option<Box<Value>>),
    Record(HashMap<String, Value>),
}
```

### 実行規則

- `bind name <- expr`: `expr` を評価し、`name` を環境に追加する
- `|>`: 左辺の結果を右辺の `trf` / `fn` に渡す
- `match`: パターンを上から順に試し、最初に合うアームを実行する
- `if`: 条件が `true` なら then、`false` なら else を実行する
- `flw`: 構成する `trf` を順に適用する

### effect の実行

- `Pure`: そのまま実行
- `Io`: `IO.print` / `IO.println` を Rust 側で実装して呼び出す
- `Db` / `Network` / `Emit`: 実行時に "effect not supported in v0.1.0" エラー

### 成果物

- `src/interpreter.rs`
- `src/env.rs`
- `src/builtins.rs` (組み込み関数)

---

## Phase 6: CLI

**目標**: `fav run` と `fav check` を動かす。

### 設計方針

- `clap` または手書きで引数解析
- `fav run <file>`: Phase 1-5 を通して実行する
- `fav check <file>`: Phase 1-4 まで実行し、型エラーを報告する

### エラー表示

```
error[E001]: type mismatch
  --> main.fav:12:5
   |
12 |     rows |> SaveUsers
   |             ^^^^^^^^^ expected List<User>, got List<Row>
```

- ファイル名・行番号・列番号を表示する
- エラーコードを振る (E001〜)

### 成果物

- `src/main.rs`
- `src/cli.rs`
- `src/diagnostics.rs`

---

## フェーズ間の依存

```
Phase 1 (Lexer)
    |
Phase 2 (AST)  <--- Phase 1 と並行可
    |
Phase 3 (Parser)
    |
Phase 4 (型チェック)
    |
Phase 5 (インタープリタ)
    |
Phase 6 (CLI)
```

---

## リポジトリ構造 (目安)

```
favnir/
├── Cargo.toml
├── fav.toml
├── src/
│   ├── main.rs
│   ├── cli.rs
│   ├── lexer.rs
│   ├── ast.rs
│   ├── parser.rs
│   ├── types.rs
│   ├── checker.rs
│   ├── interpreter.rs
│   ├── env.rs
│   ├── builtins.rs
│   └── diagnostics.rs
├── tests/
│   ├── lexer_tests.rs
│   ├── parser_tests.rs
│   ├── checker_tests.rs
│   └── interpreter_tests.rs
└── examples/
    ├── hello.fav
    ├── pipeline.fav
    └── adt_match.fav
```

---

## 完了条件

v0.1.0 完了とみなす条件:

1. `fav run hello.fav` が動く
2. `fav run pipeline.fav` で `trf` + `|>` + `flw` が動く
3. `fav run adt_match.fav` で `type` + `match` が動く
4. `fav check` で型エラー・effect エラーを検出できる
5. 型不一致のエラーメッセージに位置情報が含まれる
