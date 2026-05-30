# Favnir v6.0.0 仕様書 — セルフホスト

作成日: 2026-05-20

---

## 概要

Favnir コンパイラのコア（レキサー・パーサー・型チェッカー・バイトコードコンパイラ）を
Favnir 自身で実装する。実装は `fav/self/` ディレクトリ以下の `.fav` ファイル群として管理し、
現行の Rust 製 `fav` ツールチェーンで型チェック・実行できる。

### ゴール

> `fav run fav/self/fav_compiler.fav -- fav/self/fav_compiler.fav`
> Stage 1 と Stage 2 のバイトコードが一致すること（ブートストラップ確認）

### 実装ファイル構成

```
fav/self/
  lexer.fav       Phase B: Token 型 + lex 関数
  parser.fav      Phase C: AST 型 + parse 関数
  checker.fav     Phase D: Type 型 + check 関数
  codegen.fav     Phase E: Opcode 型 + compile 関数
  compiler.fav    Phase F: 全フェーズを結合したドライバ
```

---

## A. VM Primitive 追加（Phase A 前提条件）

### A-1. `IO.argv`

コンパイラドライバが入力ファイルパスをコマンドライン引数から受け取るために必須。

```favnir
IO.argv() -> List<String>    // fav run main.fav -- arg1 arg2 → ["arg1", "arg2"]
```

### A-2. `List.take_while` / `List.drop_while`

レキサーでの文字列スキャンに必須。

```favnir
List.take_while(xs: List<T>, pred: T -> Bool) -> List<T>
List.drop_while(xs: List<T>, pred: T -> Bool) -> List<T>
```

---

## B. レキサー（`fav/self/lexer.fav`）

### Token 型

```favnir
type Token =
  | TkInt(Int)
  | TkFloat(Float)
  | TkStr(String)
  | TkIdent(String)
  | TkTrue
  | TkFalse
  | TkFn
  | TkType
  | TkIf
  | TkElse
  | TkMatch
  | TkBind
  | TkImport
  | TkPublic
  | TkReturn
  | TkPlus
  | TkMinus
  | TkStar
  | TkSlash
  | TkEq
  | TkBangEq
  | TkLt
  | TkGt
  | TkLtEq
  | TkGtEq
  | TkAmpAmp
  | TkPipePipe
  | TkArrow
  | TkBackArrow
  | TkPipe
  | TkDot
  | TkComma
  | TkColon
  | TkSemicolon
  | TkLParen
  | TkRParen
  | TkLBrace
  | TkRBrace
  | TkLBracket
  | TkRBracket
  | TkEof
```

### lex 関数

```favnir
fn lex(src: String) -> Result<List<Token>, String>
```

実装方針:
- `String.chars(src)` で文字リストに変換
- `List.take_while` / `List.drop_while` で空白・コメントをスキップ
- 再帰関数 `scan(chars: List<String>, acc: List<Token>) -> Result<List<Token>, String>` でトークン列を構築
- 数値・文字列・識別子・演算子の各スキャン関数を用意

---

## C. パーサー（`fav/self/parser.fav`）

### AST 型（対象サブセット）

```favnir
type Expr =
  | ELit(Lit)
  | EVar(String)
  | EBinOp(BinOp, Expr, Expr)
  | ECall(String, List<Expr>)
  | EIf(Expr, Expr, Expr)
  | EMatch(Expr, List<MatchArm>)
  | EBlock(List<Stmt>, Expr)
  | ERecord(List<{field: String, value: Expr}>)
  | EVariant(String, Option<Expr>)

type Lit =
  | LInt(Int)
  | LFloat(Float)
  | LBool(Bool)
  | LStr(String)
  | LUnit

type BinOp =
  | BAdd | BSub | BMul | BDiv
  | BEq | BNe | BLt | BGt | BLe | BGe
  | BAnd | BOr

type Stmt =
  | SBind(String, Expr)
  | SExpr(Expr)

type MatchArm = { pat: Pat, body: Expr }

type Pat =
  | PWild
  | PVar(String)
  | PLit(Lit)
  | PVariant(String, Option<String>)
  | PRecord(List<{field: String, bind: String}>)

type TypeExpr =
  | TName(String)
  | TApp(String, List<TypeExpr>)
  | TFun(TypeExpr, TypeExpr)
  | TTuple(List<TypeExpr>)

type FnDef = { name: String, params: List<{name: String, ty: TypeExpr}>, ret: TypeExpr, body: Expr }

type TypeVariant = { name: String, payload: Option<TypeExpr> }
type TypeDef = { name: String, variants: List<TypeVariant> }

type Item =
  | IFnDef(FnDef)
  | ITypeDef(TypeDef)

type Program = { items: List<Item> }
```

### parse 関数

```favnir
fn parse(tokens: List<Token>) -> Result<Program, String>
```

- Pratt パーサー（再帰下降）
- パーサー状態: `{ tokens: List<Token>, pos: Int }`
- `Result<T, String>` を連鎖して `Result.and_then` でパイプライン化

---

## D. 型チェッカー（`fav/self/checker.fav`）

### Type 型

```favnir
type Type =
  | TInt
  | TFloat
  | TBool
  | TStr
  | TUnit
  | TList(Type)
  | TOption(Type)
  | TResult(Type, Type)
  | TMap(Type, Type)
  | TFun(Type, Type)
  | TRecord(List<{field: String, ty: Type}>)
  | TNamed(String)
  | TUnknown
```

### TyEnv

```favnir
// 型環境: 変数名 → 型の文字列表現（簡略化）
// Map<String, Type> の代わりに Map<String, String> は使わず、
// Map を Type をシリアライズして管理
type TyEnv = { bindings: Map<String, String> }
```

### check 関数

```favnir
fn check(prog: Program) -> Result<(), List<String>>
```

実装方針:
- 組み込み関数の型は `Map<String, String>` で管理（`"List.map" -> "Fn(List<T>, Fn(T, U), List<U>)"`）
- 型推論は `Type::Unknown` へのフォールバックあり（完全な HM 推論は v6.1.0）
- エラー: `List<String>` に収集して `Result.err` で返す

---

## E. バイトコードコンパイラ（`fav/self/codegen.fav`）

### Opcode 型

```favnir
type Opcode =
  | OpConst(Int)
  | OpConstUnit
  | OpConstTrue
  | OpConstFalse
  | OpLoadLocal(Int)
  | OpStoreLocal(Int)
  | OpLoadGlobal(Int)
  | OpPop
  | OpCall(Int)
  | OpReturn
  | OpAdd | OpSub | OpMul | OpDiv
  | OpEq | OpNe | OpLt | OpLe | OpGt | OpGe
  | OpAnd | OpOr
  | OpJump(Int)
  | OpJumpIfFalse(Int)
  | OpGetField(String)
  | OpBuildRecord(Int)
  | OpMakeClosure(Int)
```

### compile 関数

```favnir
fn compile(prog: Program) -> Result<List<Int>, String>
```

- `Opcode` → `List<Int>` へのエンコーディングを `Int.shl`/`Int.band` で実装
- 定数プール: `List<Lit>` を管理
- ローカル変数: `Map<String, Int>` で名前 → スロット番号変換
- `IO.write_bytes_raw` でファイルに書き出す

---

## F. 統合ドライバ（`fav/self/compiler.fav`）

```favnir
public fn main() -> Unit !Io {
    bind args <- IO.argv()
    bind path <- List.get(args, 0)
    match path {
        | None -> IO.println("usage: fav_compiler.fav <source.fav>")
        | Some(p) -> {
            bind src <- IO.read_file_raw(p)
            match src {
                | Err(e) -> IO.println(String.concat("error reading file: ", e))
                | Ok(text) ->
                    bind tokens <- Result.and_then(Lexer.lex(text), |toks| Parser.parse(toks))
                    // ... check + compile ...
            }
        }
    }
}
```

---

## G. ブートストラップ検証

```bash
# Stage 1: Rust コンパイラで Favnir 製コンパイラをコンパイル
fav build fav/self/compiler.fav -o /tmp/compiler_stage1.fvc

# Stage 2: Stage 1 のバイトコードで自身を再コンパイル
fav run /tmp/compiler_stage1.fvc -- fav/self/compiler.fav > /tmp/compiler_stage2.fvc

# 一致確認
diff /tmp/compiler_stage1.fvc /tmp/compiler_stage2.fvc
```

---

## 完了条件

- `cargo test` 全件通過
- `fav check fav/self/lexer.fav` エラーなし
- `fav check fav/self/parser.fav` エラーなし
- `fav check fav/self/checker.fav` エラーなし
- `fav check fav/self/codegen.fav` エラーなし
- `fav check fav/self/compiler.fav` エラーなし
- Rust 製 lexer との出力一致テスト通過
- ブートストラップ Stage 1 == Stage 2

---

## 対象サブセット（v6.0.0 スコープ）

以下の Favnir 機能を自己コンパイル可能にする:

| 機能 | v6.0.0 | 備考 |
|------|--------|------|
| Int/Float/Bool/String/Unit リテラル | ✓ | |
| 算術・比較・論理演算子 | ✓ | |
| 変数参照 | ✓ | |
| 関数定義 (`fn`) | ✓ | |
| 関数呼び出し | ✓ | |
| `if/else` | ✓ | |
| `match` （sum type） | ✓ | |
| `bind x <- expr` | ✓ | 純粋式限定 |
| レコード構築・フィールドアクセス | ✓ | |
| sum type 定義 (`type T = \| A \| B`) | ✓ | 再帰型含む |
| クロージャ (`\|x\| ...`) | ✗ | v6.1.0 |
| エフェクト (`!Io` 等) — codegen | ✗ | v6.1.0 |
| `for` ループ | ✗ | v6.1.0 |
| Rune import | ✗ | v6.1.0 |
