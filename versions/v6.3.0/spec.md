# Favnir v6.3.0 仕様書 — Self-host stage/seq

作成日: 2026-05-26

---

## 概要

`compiler.fav`（Favnir 製セルフホストコンパイラ）が `stage` / `seq` / `|>` 構文を
処理できるようにする。

v6.2.0 時点の `compiler.fav` は `fn` / `type` / `match` / `bind` / `if` / `collect` /
`yield` / クロージャを処理できるが、`stage` / `seq` / `abstract` キーワードおよび
`|>` パイプ演算子は未対応。これにより Bootstrap 後の `compiler_artifact` は
stage/seq を使ったプログラムをコンパイルできない。

### ゴール

> `compiler_artifact`（Stage 2 生成物）が `stage` / `seq` / `|>` を含む
> `.fav` ファイルをコンパイルし、Rust コンパイラと同一バイトコードを出力すること。

---

## 追加する Token

`compiler.fav` の `Token` 型に以下を追加する。

```favnir
type Token =
  | ...（既存）
  | TkStage       // "stage"
  | TkSeq         // "seq"
  | TkAbstract    // "abstract"
  // TkPipeGt は既に存在（|>）
```

`keyword_token` に対応エントリを追加：

```favnir
if s == "stage"    { Option.some(TkStage) }
if s == "seq"      { Option.some(TkSeq) }
if s == "abstract" { Option.some(TkAbstract) }
```

`TkPipeGt` は v6.1.0 時点で Token 型に定義済みだが、
`scan_op` で `|>` を認識していない場合は追加が必要。

---

## 追加する AST 型

### StageDef — ステージ定義

```favnir
type StageDef = {
    is_public:   Bool
    is_abstract: Bool
    name:        String
    param_ty:    TypeExpr
    ret_ty:      TypeExpr
    effects:     List<String>   // ["Io", "Db"] など
    body:        Option<Expr>   // abstract なら None
}
```

### SeqDef — シーケンス定義

```favnir
type SeqDef = {
    is_public:   Bool
    is_abstract: Bool
    name:        String
    stages:      List<String>   // ["ParseCsv", "Validate", "Save"]
}
```

### Item への追加

```favnir
type Item =
  | IFn(FnDef)
  | IType(TypeDef)
  | ITest(TestDef)
  | IStage(StageDef)   // 追加
  | ISeq(SeqDef)       // 追加
```

### Expr への追加

パイプ式 `a |> b` は二項演算として扱う。
`EPipe` は `ECall` の sugar（左辺を右辺関数の第1引数として渡す）に lowering する。

```favnir
// パイプ式は OpPipe として二項演算に統合するか、
// または lowering 時に ECall に変換する
// → lowering で ECall に展開する方針とする
```

---

## Lexer 変更

### scan_op への `|>` 追加

`|` を読んだとき、次の文字が `>` なら `TkPipeGt`。
既に `TkPipePipe`（`||`）の判定があるため、その後に挿入。

```
'|' + '>' → TkPipeGt
'|' + '|' → TkPipePipe
'|'       → TkPipe
```

---

## Parser 変更

### parse_item への stage/seq 追加

```
TkStage    → parse_stage_def()
TkSeq      → parse_seq_def()
TkAbstract → 次のトークンを見て parse_abstract_stage() or parse_abstract_seq()
```

### stage の構文

```favnir
stage Name: InputType -> OutputType !Effect1 !Effect2 = |param| { body }
abstract stage Name: InputType -> OutputType !Effect1
```

パース手順：
1. `stage` / `abstract stage` を読む
2. 名前（`TkIdent`）
3. `:` → input type → `->` → output type
4. `!Ident` を 0 個以上読んでエフェクトリストを構築
5. `=` → lambda `|param| { body }`（abstract なら `=` 以降なし）

### seq の構文

```favnir
seq Name = Stage1 |> Stage2 |> Stage3
abstract seq Name: InputType -> OutputType
```

パース手順：
1. `seq` / `abstract seq` を読む
2. 名前（`TkIdent`）
3. `=` → `TkIdent` (|> TkIdent)* のリストを収集

---

## Codegen 変更

### stage の lowering

`stage` は Favnir レベルでは「型注釈付き `fn`」に等しい。
lowering は `StageDef` を `FnDef` に変換してから既存の `compile_fn` に渡す。

```
StageDef { name, params, ret, body: Some(expr) }
  → FnDef { name, params: [Param { name: "input", ty: param_ty }], ret: ret_ty, body: expr }
  → compile_fn と同一パス
```

`abstract stage` はインターフェース宣言のみ → バイトコード出力なし（スキップ）。

### seq の lowering

`seq Name = A |> B |> C` は以下の関数に展開：

```favnir
fn Name(input: InputType) -> OutputType = C(B(A(input)))
```

これを `FnDef` として `compile_fn` に渡す。

具体的には `stages` リストを右から折り畳んで `ECall` のネストを構築する：

```
[A, B, C] → ECall(C, ECall(B, ECall(A, EVar("input"))))
```

`abstract seq` はスキップ。

---

## Bootstrap テスト追加

### テスト対象プログラム

```favnir
// fav/tmp/pipeline_test.fav
stage Double: Int -> Int = |n| { n * 2 }
stage AddOne: Int -> Int = |n| { n + 1 }
seq DoubleThenAdd = Double |> AddOne

fn main() -> Int {
    bind result <- DoubleThenAdd(5)
    result
}
```

### テスト内容

```rust
#[test]
fn bootstrap_stage_seq_matches_rust_compiler() {
    // Rust コンパイラで pipeline_test.fav をコンパイル → bytecode_rust
    // compiler_artifact で pipeline_test.fav をコンパイル → bytecode_self
    // assert_eq!(bytecode_rust, bytecode_self)
}
```

---

## 完了条件

- [ ] `compiler.fav` が `fav check` をエラーなしで通る
- [ ] `compiler.fav` が `stage` / `seq` / `|>` を含むプログラムをコンパイルできる
- [ ] `cargo test bootstrap_stage_seq_matches_rust_compiler` が通る
- [ ] 既存の Bootstrap テスト（`bootstrap_full_self_hosting`）が引き続き通る
- [ ] `cargo test` 全テスト通過
