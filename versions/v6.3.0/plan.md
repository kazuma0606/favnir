# Favnir v6.3.0 実装計画 — Self-host stage/seq

---

## 変更ファイル一覧

| ファイル | 変更内容 |
|---------|---------|
| `fav/self/compiler.fav` | Token 追加・Lexer 更新・AST 型追加・Parser 追加・Codegen 追加 |
| `fav/src/driver/self_tests.rs` | Bootstrap 比較テスト追加 |
| `fav/tmp/pipeline_test.fav` | テスト用 stage/seq プログラム（新規作成） |

変更は `fav/self/compiler.fav` の単一ファイルに集約する。
Rust 側（vm.rs / checker.rs）への追加は不要。

---

## Phase A: Token 追加

### A-1: Token 型に TkStage / TkSeq / TkAbstract を追加

`compiler.fav` の `Token` 型（先頭付近）に追加：

```favnir
type Token =
  | ...（既存）
  | TkStage
  | TkSeq
  | TkAbstract
```

`TkPipeGt` は既に Token 型に定義済みであることを確認する。

### A-2: keyword_token に対応エントリを追加

`keyword_token` 関数の `else { Option.none() }` の直前に挿入：

```favnir
if s == "stage"    { Option.some(TkStage) }
else {
if s == "seq"      { Option.some(TkSeq) }
else {
if s == "abstract" { Option.some(TkAbstract) }
else { Option.none() }
}}}
```

### A-3: scan_op で `|>` を認識

`scan_op` 関数（または `scan_two_char_op`）で、
`|` の次が `>` なら `TkPipeGt` を返すよう追加。
現状 `|` は `TkPipe`、`||` は `TkPipePipe` として処理済みのため、
`|>` の判定を `||` の前に追加する：

```favnir
if c == "|" {
    if next_char_is(tail, ">") {
        bind rest2 <- List.drop(tail, 1)
        Result.ok(ScanResult { tok: TkPipeGt  rest: rest2 })
    } else {
    if next_char_is(tail, "|") {
        bind rest2 <- List.drop(tail, 1)
        Result.ok(ScanResult { tok: TkPipePipe  rest: rest2 })
    } else {
        Result.ok(ScanResult { tok: TkPipe  rest: tail })
    }}
}
```

---

## Phase B: AST 型追加

`compiler.fav` の AST TYPES セクション（`TypeExpr` / `FnDef` / `Item` の定義付近）に追加：

### B-1: StageDef 型

```favnir
type StageDef = {
    is_public:   Bool
    is_abstract: Bool
    name:        String
    param_ty:    TypeExpr
    ret_ty:      TypeExpr
    effects:     List<String>
    body:        Option<Expr>
}
```

### B-2: SeqDef 型

```favnir
type SeqDef = {
    is_public:   Bool
    is_abstract: Bool
    name:        String
    stages:      List<String>
}
```

### B-3: Item に IStage / ISeq を追加

```favnir
type Item =
  | IFn(FnDef)
  | IType(TypeDef)
  | ITest(TestDef)
  | IStage(StageDef)
  | ISeq(SeqDef)
```

---

## Phase C: Parser 追加

### C-1: parse_effects ヘルパー

エフェクトリスト（`!Io !Db` など）をパースする補助関数：

```favnir
fn parse_effects(tokens: List<Token>) -> { effects: List<String>  rest: List<Token> } {
    // TkBang + TkIdent のペアを繰り返し読む
    // TkBang でなければ終了
    ...
}
```

### C-2: parse_stage_def

```
// stage Name: InType -> OutType !Eff = |param| { body }
// abstract stage Name: InType -> OutType !Eff
fn parse_stage_def(tokens: List<Token>, is_public: Bool, is_abstract: Bool)
    -> Result<ParseResult, String>
```

手順：
1. `TkIdent` → name
2. `TkColon` → consume
3. `parse_type_expr` → param_ty
4. `TkArrow` → consume
5. `parse_type_expr` → ret_ty
6. `parse_effects` → effects
7. `is_abstract` なら body = `None`、そうでなければ:
   - `TkEq` → consume
   - `parse_expr` → body = `Some(...)`

### C-3: parse_seq_pipeline

`=` の後に続く `Name |> Name |> Name` のリストを収集：

```favnir
fn parse_seq_pipeline(tokens: List<Token>, acc: List<String>)
    -> Result<{ stages: List<String>  rest: List<Token> }, String>
// TkIdent を読んで acc に追加
// TkPipeGt なら再帰
// それ以外なら acc を返す
```

### C-4: parse_seq_def

```
// seq Name = Stage1 |> Stage2 |> Stage3
// abstract seq Name: InType -> OutType
fn parse_seq_def(tokens: List<Token>, is_public: Bool, is_abstract: Bool)
    -> Result<ParseResult, String>
```

### C-5: parse_item の拡張

既存の `parse_item` で `TkStage` / `TkSeq` / `TkAbstract` を処理：

```favnir
TkStage => parse_stage_def(rest, is_public, false)
TkSeq   => parse_seq_def(rest, is_public, false)
TkAbstract => {
    // 次のトークンを確認
    match List.first(rest2) {
        Some(TkStage) => parse_stage_def(rest3, is_public, true)
        Some(TkSeq)   => parse_seq_def(rest3, is_public, true)
        _ => Result.err("abstract の後には stage か seq が必要です")
    }
}
```

---

## Phase D: Codegen 追加

### D-1: compile_stage

`StageDef` を `FnDef` に変換して `compile_fn` に委譲：

```favnir
fn compile_stage(ctx: CodegenCtx, def: StageDef) -> CodegenCtx {
    match def.body {
        None => ctx  // abstract はスキップ
        Some(body) => {
            bind param_name <- "input"
            bind fn_def <- FnDef {
                is_public: def.is_public
                name: def.name
                params: List.singleton(Param { name: param_name  ty: def.param_ty })
                ret: def.ret_ty
                body: body
            }
            compile_fn(ctx, fn_def)
        }
    }
}
```

### D-2: build_pipe_call

`seq` の stages リストを `ECall` のネストに変換するヘルパー：

```favnir
fn build_pipe_call(stages: List<String>, input_expr: Expr) -> Expr {
    match List.first(stages) {
        None => input_expr
        Some(name) => {
            bind rest <- List.drop(stages, 1)
            build_pipe_call(rest, ECall(name, "", EArgList(input_expr, EArgNil)))
        }
    }
}
```

### D-3: compile_seq

`SeqDef` を展開して `compile_fn` に委譲：

```favnir
fn compile_seq(ctx: CodegenCtx, def: SeqDef) -> CodegenCtx {
    match def.is_abstract {
        true => ctx  // abstract はスキップ
        false => {
            bind input_var <- EVar("input")
            bind body <- build_pipe_call(def.stages, input_var)
            // 型情報は seq 展開後の codegen では不要（VM は動的型）
            // param_ty / ret_ty は Any として扱う
            bind fn_def <- FnDef {
                is_public: def.is_public
                name: def.name
                params: List.singleton(Param { name: "input"  ty: TeSimple("Any") })
                ret: TeSimple("Any")
                body: body
            }
            compile_fn(ctx, fn_def)
        }
    }
}
```

### D-4: compile_item の拡張

```favnir
fn compile_item(ctx: CodegenCtx, item: Item) -> CodegenCtx {
    match item {
        IFn(def)    => compile_fn(ctx, def)
        IType(_)    => ctx
        ITest(_)    => ctx
        IStage(def) => compile_stage(ctx, def)  // 追加
        ISeq(def)   => compile_seq(ctx, def)    // 追加
    }
}
```

---

## Phase E: テスト追加

### E-1: fav/tmp/pipeline_test.fav の作成

```favnir
stage Double: Int -> Int = |n| { n * 2 }
stage AddOne: Int -> Int = |n| { n + 1 }
seq DoubleThenAdd = Double |> AddOne

fn main() -> Int {
    bind result <- DoubleThenAdd(5)
    result
}
```

期待される実行結果: `11`（5 * 2 + 1）

### E-2: self_tests.rs に Bootstrap 比較テスト追加

```rust
#[test]
fn bootstrap_stage_seq_matches_rust_compiler() {
    // Rust コンパイラ → pipeline_test.fav → bytecode_rust
    let bytecode_rust = compile_with_rust("fav/tmp/pipeline_test.fav");
    // compiler_artifact → pipeline_test.fav → bytecode_self
    let bytecode_self = compile_with_artifact("fav/tmp/pipeline_test.fav");
    assert_eq!(bytecode_rust, bytecode_self,
        "stage/seq: self-host compiler output diverges from Rust compiler");
}
```

### E-3: 既存テストの確認

```
cargo test bootstrap_full_self_hosting
cargo test bootstrap_stage1
```

---

## 実装上の注意

### `|>` の scan_op 位置

`scan_op` 内で `||` (TkPipePipe) の判定を先に行っている場合、
`|>` の判定は `||` の **前** に置く必要がある。
現状のコードで `||` が先に評価されているなら、`|>` が `|` + `>` に分割されるバグが生じる。

### abstract seq の型注釈

`abstract seq Name: InType -> OutType` は型情報のみを宣言する。
codegen ではスキップするため、パーサーでも型注釈は収集するが
`is_abstract: true` として `SeqDef` に格納すれば良い。
型チェッカー（checker.fav）側の対応は v6.x 後半で実施する。

### ECall の名前空間

セルフホスト codegen の `compile_expr` で `ECall(ns, name, args)` を使っている。
stage 関数呼び出しはユーザー定義関数なので `ns = ""` / `name = ステージ名` とする。

### `TeSimple("Any")` の扱い

compile_seq が生成する `FnDef` の型注釈 `TeSimple("Any")` は、
バイトコード生成では使用されない（VM は動的型付け）ため問題ない。
型チェッカーがこれを検査しないよう、checker.fav では `Any` を unknown として通過させる。
