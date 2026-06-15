# v18.3.0 実装計画 — Refinement Types

## 依存関係

```
T1（ast.rs）
  └─ T2（parser.rs）
       └─ T3（checker.rs コンパイル時チェック）
  └─ T4（codegen.rs RefinementAssert opcode）
       └─ T5（compiler.rs コード生成）
            └─ T6（v183000_tests）
T7（ドキュメント）← T5 完了後に並列可
T8（バージョン更新）← T6 完了後
```

---

## フェーズ別実装計画

### フェーズ 1: AST 変更（T1）

`fav/src/ast.rs` の `Param` 構造体に `constraint` フィールドを追加する。

```rust
pub struct Param {
    pub name: String,
    pub ty: TypeExpr,
    pub constraint: Option<Box<Expr>>,  // NEW
    pub span: Span,
}
```

`Param` を構築しているすべての箇所を `constraint: None` で更新（波及修正）。

**波及箇所の調査ポイント:**
- `parser.rs` の `parse_params` — `Param { name, ty, span }` 構築箇所
- `fmt.rs` の `Param` pretty-print
- `emit_python.rs` の param 処理
- `lineage.rs` の param 参照
- `compiler.rs` の param 参照

### フェーズ 2: パーサー拡張（T2）

`parse_params` に `where { expr }` の解析を追加する。

```
"(" param ("," param)* ")"
param ::= ident ":" type_expr ["where" "{" expr "}"]
```

- `TokenKind::Where` はすでに存在する（lexer.rs:73）
- `where` の後に `{` がなければ（型の `where` 節との区別）`parse_expr()` を呼ぶのではなく、`{` の中の式をブロックとして解析する
- 内部の式は通常の `parse_expr()` で評価。`}` で閉じる。
- `Param.constraint = Some(Box::new(expr))`

**注意:** `where` は型定義でも使われる（`type Foo where { ... }`）。引数パース中に限定する。

### フェーズ 3: 型チェック時のコンパイル時検査（T3）

`checker.rs` の `check_fn_call`（または `check_apply`）で呼び出し引数を検査する。

#### 静的チェック（E0331）

1. 呼び出し引数の式を `eval_static_expr` で評価する
2. 成功（`Some(StaticValue::...)`) なら制約式を静的に評価する
   - 制約式中の変数（引数名）を `StaticValue` で置換して評価
3. 違反（`false`）なら E0331 を発行してコンパイルエラー

**eval_static_expr の現状:** `checker.rs:4120` にあり、リテラル・`BinOp` 演算に対応。

#### 静的評価不能の場合

`eval_static_expr` が `None` を返した場合は T5 のランタイムアサーションに委ねる
（チェッカーは何もしない）。

### フェーズ 4: VM opcode 追加（T4）

`fav/src/backend/codegen.rs` の `Opcode` enum に `RefinementAssert` を追加する。

```rust
RefinementAssert {
    param_name: String,
    condition: Vec<Opcode>,  // 制約式のバイトコード（引数値はスタックに積まれている）
}
```

**代替設計:** 制約式を通常のコードとしてコンパイルし、`JumpIfTrue skip_error` パターンで実装する方が VM 変更を最小化できる。この設計を採用する。

```
DupN(0)              # 引数値をスタックにコピー
StoreLocal(tmp_slot) # 引数名スロットに格納
<constraint expr>    # 制約式をコンパイル
JumpIfTrue(ok_label)
Push(Str("refinement violated: {param_name}"))
MakeErr
Raise               # または RefinementAssert opcode で error を push
ok_label:
```

`RefinementAssert` opcode は「スタックトップが false なら error value を積む」シンプルな opcode とする:

```rust
RefinementAssert { param_name: String },
```

### フェーズ 5: コンパイラでのコード生成（T5）

`compiler.rs` の `compile_fn_def`（または `compile_call`）で:

1. 各パラメータに `constraint` がある場合:
   a. 引数値を `StoreLocal(slot)` で格納
   b. 制約式をコンパイル（引数名は `Local(slot)` として解決される）
   c. `RefinementAssert { param_name }` を emit

**コード生成タイミング:** 関数本体の**先頭**（prologue）でアサートする。

```
fn divide(a, b where { b != 0 }):
  [a を Local(0), b を Local(1) に格納]
  Load(Local(1))           # b
  Const(0)                 # 0
  Ne                       # b != 0
  RefinementAssert("b")    # false なら error
  [本体]
```

### フェーズ 6: テスト追加（T6）

`driver.rs` に `v183000_tests` モジュールを追加する（5件）。

### フェーズ 7: ドキュメント（T7）

`site/content/docs/language/refinement-types.mdx` を作成する。

### フェーズ 8: バージョン更新（T8）

`fav/Cargo.toml`: `18.2.0` → `18.3.0`

---

## 技術的注意事項

### `where` キーワードの衝突

- 型パラメータの `where` 節（`type Foo<T> where T: ...`）と区別が必要
- 引数パース中（`parse_params` 内）に限り `where { ... }` を constraint として扱う
- `}` と `,` / `)` の区別: 制約の `{` `}` ブロックは 1 レベルのネスト

### `Param` 構築の波及修正

`Param` struct に新フィールドを追加すると Rust のすべての `Param { ... }` リテラルが
コンパイルエラーになる。`constraint: None` のデフォルト値は `Default` トレイトで
補完できないので、すべての構築箇所を手動で修正する。

主な波及箇所:
- `parser.rs`: `parse_params` の `Param { name, ty, span }` → `Param { name, ty, constraint: None, span }`
- `fmt.rs`: `Param` の pretty-print（`constraint` があれば ` where { ... }` を出力）
- その他の `Param` 参照箇所（`emit_python.rs`, `lineage.rs` など）

### 制約式の変数解決

制約式 `b != 0` 中の `b` は関数引数として `Local(slot)` にマップされる。
通常の関数本体コンパイルと同じスコープルールが使える。

### RefinementAssert の実装

VM (`interpreter.rs` または `vm.rs`) に `RefinementAssert` の実行ロジックを追加:
- スタックから bool 値を pop
- `true` なら何もしない
- `false` なら `Err("refinement violated: {param_name}")` をスタックに push して
  早期リターン（または throw）

---

## リスク

| リスク | 対策 |
|---|---|
| `Param` 波及修正の漏れ | `cargo build` でコンパイルエラーを確認してから進む |
| `where` キーワード衝突 | `parse_params` コンテキスト内に限定 |
| 制約式の複雑なスコープ（他引数を参照） | v18.3.0 では同引数名のみを保証、他引数参照は best-effort |
| ランタイムエラーの伝播 | `RefinementAssert` は Result 伝播として扱う（fn の戻り型が合わない場合は panic） |
