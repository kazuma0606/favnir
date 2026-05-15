# Favnir v3.0.0 仕様書

作成日: 2026-05-13

---

## テーマ

**セルフホスト完成 + 言語基盤安定化**

Favnir のパーサーを Favnir 自身で書き（セルフホスト Step 1）、
エラーコード体系を刷新して言語基盤を安定させる。

---

## 機能 1: エラーコード体系の刷新（E0xxx 体系）

### 現状

`E001`–`E081` の 3 桁体系（カテゴリの区別なし）

### 新体系

`E0xxx` の 4 桁体系。先頭 2 桁でカテゴリを表す。

| カテゴリ | 範囲 | 内容 |
|---------|------|------|
| 構文 | E0101–E0199 | パース・キーワード・構文 |
| 型 | E0201–E0299 | 型不一致・未定義・型推論 |
| エフェクト | E0301–E0399 | 宣言漏れ・不正エフェクト |
| ランタイム | E0401–E0499 | invariant 違反・実行時エラー |
| モジュール | E0501–E0599 | import・循環依存・競合 |
| 廃止コード | E0901–E0999 | 旧構文（trf/flw/cap）の移行ガイド |

### マッピング（現行 → 新コード）

**構文 (E01xx)**
| 旧 | 新 | 意味 |
|----|-----|------|
| E001 | E0101 | undefined seq step / trf |
| E002 | E0102 | undefined seq name |
| E003 | E0103 | type mismatch in seq chain |
| E007 | E0107 | duplicate type definition |
| E008 | E0108 | duplicate fn definition |
| E009 | E0109 | duplicate stage definition |
| E010 | E0110 | duplicate seq definition |
| E012 | E0112 | abstract seq field not implemented |
| E036 | E0136 | variant constructor arity |

**型 (E02xx)**
| 旧 | 新 | 意味 |
|----|-----|------|
| E013 | E0213 | type mismatch (return / assignment) |
| E014 | E0214 | undefined type |
| E015 | E0215 | wrong argument count |
| E018 | E0218 | field not found |
| E019 | E0219 | non-record field access |
| E020 | E0220 | undefined interface (cap) |
| E021 | E0221 | interface not implemented |
| E022 | E0222 | undefined function |
| E023 | E0223 | match arm type mismatch |
| E024 | E0224 | non-exhaustive match |
| E025 | E0225 | invalid binary operand types |
| E026 | E0226 | if branch type mismatch |
| E027 | E0227 | invariant type error |
| E041 | E0241 | interface method return mismatch |
| E042 | E0242 | interface method param mismatch |
| E043 | E0243 | interface method not found |
| E044 | E0244 | impl type not found |
| E045 | E0245 | duplicate impl |
| E046 | E0246 | generic type arity mismatch |
| E048 | E0248 | interface field type mismatch |
| E049 | E0249 | non-interface type constraint |
| E051 | E0251 | recursive type without indirection |
| E052 | E0252 | type state violation |
| E053 | E0253 | f-string interpolation error |
| E054 | E0254 | f-string expr type error |
| E074 | E0274 | stage in non-pipeline context |

**エフェクト (E03xx)**
| 旧 | 新 | 意味 |
|----|-----|------|
| E065 | E0365 | for-in iter type error |
| E066 | E0366 | for-in body yield type mismatch |
| E068 | E0368 | null-coalesce left not Option |
| E069 | E0369 | null-coalesce type mismatch |
| E070 | E0370 | undeclared effect |
| E071 | E0371 | effect not allowed |
| E072 | E0372 | effect propagation error |
| E073 | E0373 | async/Task effect error |

**モジュール (E05xx)**
| 旧 | 新 | 意味 |
|----|-----|------|
| E000 | E0500 | 汎用エラー (未分類) |
| E080 | E0580 | circular import |
| E081 | E0581 | namespace conflict |

**廃止コード (E09xx)**
| 旧 | 新 | 意味 |
|----|-----|------|
| E2001 | E0901 | deprecated keyword `trf` → use `stage` |
| E2002 | E0902 | deprecated keyword `flw` → use `seq` |
| E2003 | E0903 | deprecated keyword `cap` → use `interface` |

### 実装方針

- `checker.rs` の `type_error` 呼び出しを全て新コードに置き換える
- `parser.rs`・`driver.rs` の E2001/E2002/E2003 も E0901/E0902/E0903 に更新
- 旧コードは **一切使わない**（後方互換エイリアスなし、破壊的変更）
  - ただし `versions/v3.0.0/migration-guide.md` で旧→新の対応表を公開
- テストは旧コードを新コードに置き換える（全テストを通過させる）

### `fav explain-error` コマンド

```
$ fav explain-error E0213
E0213: type mismatch

  A value of one type was used where a different type was expected.

  Example:
    fn double(n: Int) -> Int {
        "not a number"  // E0213: expected Int, got String
    }

  How to fix:
    Make sure the expression type matches the declared return type.
    Use `fav check` to see the full type error with context.
```

`driver.rs` に `cmd_explain_error(code: &str)` を追加。
エラー説明は `src/error_catalog.rs`（新規ファイル）に Rust 定数として定義する。

---

## 機能 2: explain JSON スキーマ v3.0 固定

### 現状

```json
{ "schema_version": "1.0", "favnir_version": "1.5.0", ... }
```

### 変更内容

```json
{ "schema_version": "3.0", "favnir_version": "3.0.0", ... }
```

- `schema_version` を `"3.0"` に更新
- `favnir_version` を `env!("CARGO_PKG_VERSION")` で自動取得（ハードコードをやめる）
- `trfs` キーを `stages` にリネーム（`stage` が正式名称のため）
- `flws` キーを `seqs` にリネーム

スキーマ変更は **breaking change**。`versions/v3.0.0/migration-guide.md` に旧→新の対応表を記載。

---

## 機能 3: セルフホスト Step 1 — フルレキサー

### 目標

`selfhost/lexer/lexer.fav` を全トークン対応に拡張する。

現状: `+`, `-`, `*`, `/`, ` ` のみ
目標: Favnir の全トークンを認識するレキサー

### Token 型

```favnir
type Token = {
    kind: String   // "Plus", "Ident", "Int", "Keyword_fn", etc.
    text: String   // the raw text of the token
    pos:  Int      // byte offset in source
}
```

### 対応するトークン種別

**単一文字**:
`(` `LParen`, `)` `RParen`, `{` `LBrace`, `}` `RBrace`,
`[` `LBracket`, `]` `RBracket`, `,` `Comma`, `;` `Semi`,
`@` `At`, `%` `Percent`, `!` `Bang`

**2 文字（先頭 1 文字で確認後、次文字を見る）**:
`|>` `Pipe`, `->` `Arrow`, `=>` `FatArrow`, `<-` `LeftArrow`,
`??` `QuestionQuestion`, `==` `EqEq`, `!=` `BangEq`,
`<=` `LtEq`, `>=` `GtEq`, `&&` `AmpAmp`, `||` `PipePipe`

**比較・算術（単独）**:
`=` `Eq`, `<` `Lt`, `>` `Gt`, `+` `Plus`, `-` `Minus`,
`*` `Star`, `/` `Slash`, `.` `Dot`, `:` `Colon`,
`|` `Bar`, `&` `Amp`

**識別子・キーワード** (identifier then keyword-check):
`fn`, `public`, `stage`, `seq`, `type`, `interface`, `impl`,
`bind`, `match`, `if`, `else`, `for`, `in`, `yield`,
`collect`, `use`, `test`, `bench`, `async`, `import`,
`true` → `Bool` token (text="true"), `false` → `Bool` token

**リテラル**:
- 整数: `[0-9]+` → `Int`
- 浮動小数点: `[0-9]+\.[0-9]+` → `Float`
- 文字列: `"..."` → `Str`（エスケープなし）

**コメント**: `//` から行末まで → スキップ（トークンにしない）

**空白・改行**: スキップ

**EOF**: 入力終了 → `Eof`

### スキャナー設計（Favnir で実装）

```favnir
// src 文字列を受け取り、Token のリストを返す
public fn lex(src: String) -> List<Token> { ... }
```

実装方針:
1. `String.length(src)` で長さを取得
2. 現在位置 `pos: Int` を状態として持つ再帰ヘルパー `scan_from(src, pos, acc)` で実装
3. `String.char_at(src, pos)` で 1 文字ずつ取得
4. 2 文字トークンは `String.char_at(src, pos + 1)` でルックアヘッド
5. 識別子: `pos` からアルファベット・アンダースコア・数字が続く間を収集 → キーワードチェック
6. 整数: `pos` から数字が続く間を収集
7. 文字列: `"` → 閉じ `"` まで収集
8. `//`: 次の `\n` まで進める（スキップ）

制約: Favnir に `while` ループがないため、再帰関数または `List.fold` で実装する。
再帰ヘルパー `scan_from` を使い末尾再帰スタイルで書く。

---

## 機能 4: セルフホスト Step 1 — 基礎パーサー

### 目標

`selfhost/parser/parser.fav` を新規作成。Favnir の基礎構文を解析して AST を返す。

### AST 型定義

```favnir
// selfhost/parser/ast.fav

type ParseError = { message: String  pos: Int }

// 型式
type TypeExpr =
    | TInt
    | TFloat
    | TBool
    | TString
    | TUnit
    | TList  { elem: TypeExpr }
    | TOption { elem: TypeExpr }
    | TResult { ok: TypeExpr  err: TypeExpr }
    | TName  { name: String }

// 式
type Expr =
    | EInt   { value: Int }
    | EFloat { value: Float }
    | EBool  { value: Bool }
    | EStr   { value: String }
    | EUnit
    | EIdent { name: String }
    | ECall  { func: String  args: List<Expr> }
    | EBinOp { op: String  left: Expr  right: Expr }
    | EIf    { cond: Expr  then_: List<Stmt>  else_: List<Stmt> }
    | EBlock { stmts: List<Stmt> }

// 文
type Stmt =
    | SBind  { name: String  value: Expr }
    | SExpr  { value: Expr }
    | SYield { value: Expr }

// トップレベル
type Param   = { name: String  ty: TypeExpr }
type FnDef   = { name: String  params: List<Param>  ret: TypeExpr  body: List<Stmt> }
type TypeDef = { name: String  fields: List<Param> }

type TopLevel =
    | TLFn   { def: FnDef }
    | TLType { def: TypeDef }
```

### パーサー設計

```favnir
// トークンストリームを消費するコンビネータスタイル
type ParseState = { tokens: List<Token>  pos: Int }

type ParseResult<T> = {
    value: T
    state: ParseState
}
```

各パーサーは `ParseState -> Result<ParseResult<T>, ParseError>` を返す。

実装する関数:
- `parse_program(tokens) -> Result<List<TopLevel>, ParseError>`
- `parse_fn_def(state) -> Result<ParseResult<FnDef>, ParseError>`
- `parse_type_def(state) -> Result<ParseResult<TypeDef>, ParseError>`
- `parse_type_expr(state) -> Result<ParseResult<TypeExpr>, ParseError>`
- `parse_expr(state) -> Result<ParseResult<Expr>, ParseError>`
- `parse_stmt(state) -> Result<ParseResult<Stmt>, ParseError>`
- `parse_block(state) -> Result<ParseResult<List<Stmt>>, ParseError>`

`consume(state, kind)` — 指定した kind のトークンを消費して次 state を返す
`peek(state)` — 現在位置のトークンを見る
`advance(state)` — 1 トークン進める

### ファイル構成

```
selfhost/
  lexer/
    lexer.fav         ← 全トークン対応（Phase 4 で拡張）
    lexer.test.fav    ← ~40 件のテスト
  parser/
    ast.fav           ← AST 型定義（Phase 5 で新規作成）
    parser.fav        ← パーサー本体（Phase 5 で新規作成）
    parser.test.fav   ← ~60 件のテスト
    main.fav          ← `fav run` できるエントリポイント
```

---

## 機能 5: `fav explain compiler` コマンド

コンパイル工程を人間が読める形式で表示する。

```
$ fav explain compiler src/main.fav
Favnir v3.0.0 compilation pipeline

Step 1: Lexer      → 42 tokens
Step 2: Parser     → 5 top-level items
Step 3: Checker    → 0 errors, 12 types inferred
Step 4: Compiler   → 8 IR functions
Step 5: Codegen    → 284 bytes (.fvc artifact)
```

`driver.rs` に `cmd_explain_compiler(file: &str)` を追加する。
各ステップの結果（件数・要約）を計算して表示する。

---

## 完了条件

- `cargo test` 全テスト通過
- `Cargo.toml` バージョンが `"3.0.0"`
- エラーコードが全て E0xxx 体系に移行している（旧 3 桁コードがソースに残っていない）
- `fav explain-error E0213` でエラー説明が表示される
- explain JSON に `"schema_version": "3.0"` が含まれる
- explain JSON のキーが `stages`（旧 `trfs`）・`seqs`（旧 `flws`）になっている
- `fav run selfhost/lexer/lexer.fav` が動く（全トークン対応）
- `fav test selfhost/lexer/lexer.test.fav` で 40 件以上のテストが通る
- `fav run selfhost/parser/main.fav` が動く
- `fav test selfhost/parser/parser.test.fav` で 60 件以上のテストが通る
- selfhost テスト合計 100 件以上
- `fav explain compiler src/main.fav` が工程サマリーを表示する
- `versions/v3.0.0/langspec.md` 作成済み
- `versions/v3.0.0/migration-guide.md` 作成済み（エラーコード旧→新対応表）

---

## テスト数見込み

v2.9.0 ベースライン: 637

| カテゴリ | 追加件数 |
|---------|--------|
| エラーコード移行（既存テストのコード書き換え） | ±0 |
| E0xxx 新コードのテスト | +10 |
| `fav explain-error` コマンドテスト | +5 |
| explain JSON スキーマ v3.0 テスト | +3 |
| selfhost lexer テスト（driver.rs 統合） | +10 |
| selfhost parser テスト（driver.rs 統合） | +10 |
| `fav explain compiler` テスト | +3 |
| **目標合計** | **~678（+41）** |
