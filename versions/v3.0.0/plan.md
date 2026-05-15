# Favnir v3.0.0 実装計画

作成日: 2026-05-13

---

## フェーズ一覧

| Phase | 内容 | 変更ファイル | 難度 |
|-------|------|-------------|------|
| 0 | バージョン更新 | Cargo.toml, main.rs | 低 |
| 1 | エラーコード移行（E0xxx 体系） | checker.rs, parser.rs, driver.rs, テスト全体 | 高 |
| 2 | `fav explain-error` コマンド | error_catalog.rs (新), driver.rs, main.rs | 中 |
| 3 | explain JSON スキーマ v3.0 | driver.rs | 低 |
| 4 | selfhost lexer 完成（全トークン対応） | selfhost/lexer/lexer.fav, lexer.test.fav | 高 |
| 5 | selfhost parser 基礎実装 | selfhost/parser/ast.fav (新), parser.fav (新), parser.test.fav (新), main.fav (新) | 高 |
| 6 | driver.rs 統合テスト追加 | driver.rs | 中 |
| 7 | `fav explain compiler` コマンド | driver.rs, main.rs | 中 |
| 8 | ドキュメント・最終確認 | versions/v3.0.0/langspec.md, migration-guide.md, progress.md | 低 |

---

## Phase 0: バージョン更新

**ファイル**: `fav/Cargo.toml`, `fav/src/main.rs`

```toml
# Cargo.toml
version = "3.0.0"
```

```rust
// main.rs: HELP テキスト
const HELP: &str = "Favnir v3.0.0 ...";

// print_welcome()
println!("Favnir v3.0.0");
```

---

## Phase 1: エラーコード移行

最も変更量が多いフェーズ。全 3 桁エラーコードを 4 桁に置き換える。

### 1-1: checker.rs の置き換え

`type_error("E001", ...)` → `type_error("E0101", ...)`

全マッピングは `spec.md` の対応表を参照。
`checker.rs` にあるエラーコード文字列を正規表現で一括置換後、個別確認する。

一括置換コマンドイメージ:
```
E001  → E0101
E002  → E0102
E003  → E0103
E007  → E0107
E008  → E0108
E009  → E0109
E010  → E0110
E012  → E0112
E013  → E0213
E014  → E0214
E015  → E0215
E018  → E0218
E019  → E0219
E020  → E0220
E021  → E0221
E022  → E0222
E023  → E0223
E024  → E0224
E025  → E0225
E026  → E0226
E027  → E0227
E036  → E0136
E041  → E0241
E042  → E0242
E043  → E0243
E044  → E0244
E045  → E0245
E046  → E0246
E048  → E0248
E049  → E0249
E051  → E0251
E052  → E0252
E053  → E0253
E054  → E0254
E065  → E0365
E066  → E0366
E068  → E0368
E069  → E0369
E070  → E0370
E071  → E0371
E072  → E0372
E073  → E0373
E074  → E0274
E080  → E0580
E081  → E0581
E000  → E0500
```

### 1-2: parser.rs の置き換え

`E2001` → `E0901`
`E2002` → `E0902`
`E2003` → `E0903`

### 1-3: driver.rs テストの置き換え

テスト内の `"E001"`, `"E013"` 等の文字列を新コードに置き換える。

### 1-4: cargo test を通す

置き換え後に `cargo test` を実行し、全テストが通ることを確認する。
テスト内の `assert_eq!(errors[0].code, "E013")` 等が全て新コードになっていることを確認。

---

## Phase 2: `fav explain-error` コマンド

### 2-1: `src/error_catalog.rs` 新規作成

```rust
// src/error_catalog.rs
pub struct ErrorEntry {
    pub code: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub example: &'static str,
    pub fix: &'static str,
}

pub const ERROR_CATALOG: &[ErrorEntry] = &[
    ErrorEntry {
        code: "E0101",
        title: "undefined seq step",
        description: "A stage referenced in a seq definition was not found.",
        example: r#"seq Bad = NonExistentStage |> OtherStage  // E0101"#,
        fix: "Check the stage name for typos, or define the stage before using it.",
    },
    ErrorEntry {
        code: "E0213",
        title: "type mismatch",
        description: "A value of one type was used where a different type was expected.",
        example: "fn double(n: Int) -> Int {\n    \"not a number\"  // E0213\n}",
        fix: "Make sure the expression type matches the declared return type.",
    },
    // ... 全エラーコードを追加
];

pub fn lookup(code: &str) -> Option<&'static ErrorEntry> {
    ERROR_CATALOG.iter().find(|e| e.code == code)
}
```

### 2-2: `driver.rs` に `cmd_explain_error` 追加

```rust
pub fn cmd_explain_error(code: &str) {
    match error_catalog::lookup(code) {
        Some(entry) => {
            println!("{}: {}\n", entry.code, entry.title);
            println!("  {}\n", entry.description);
            if !entry.example.is_empty() {
                println!("  Example:");
                for line in entry.example.lines() {
                    println!("    {}", line);
                }
                println!();
            }
            println!("  How to fix:");
            println!("    {}", entry.fix);
        }
        None => {
            eprintln!("error: unknown error code `{}`", code);
            eprintln!("hint: use `fav explain-error --list` to see all codes");
            process::exit(1);
        }
    }
}
```

### 2-3: `main.rs` に `explain-error` サブコマンド追加

```
fav explain-error E0213
fav explain-error --list        # 全コード一覧
```

---

## Phase 3: explain JSON スキーマ v3.0

`driver.rs` の `build_manifest_json` 関数（または `render_explain_json` 等）を変更:

```rust
// 変更前
"schema_version": "1.0",
"favnir_version": "1.5.0",
"trfs": [...],
"flws": [...],

// 変更後
"schema_version": "3.0",
"favnir_version": env!("CARGO_PKG_VERSION"),
"stages": [...],   // trfs → stages
"seqs":   [...],   // flws → seqs
```

関連テストも更新:
```rust
// 変更前
assert_eq!(value["schema_version"], "1.0");
assert_eq!(value["favnir_version"], "1.5.0");
assert!(value["trfs"].is_array());
assert!(value["flws"].is_array());

// 変更後
assert_eq!(value["schema_version"], "3.0");
assert_eq!(value["favnir_version"], "3.0.0");
assert!(value["stages"].is_array());
assert!(value["seqs"].is_array());
```

---

## Phase 4: selfhost lexer 完成

### 設計

`selfhost/lexer/lexer.fav` を全面再実装。

```favnir
// Token 型
type Token = {
    kind: String
    text: String
    pos:  Int
}

// エントリポイント
public fn lex(src: String) -> List<Token> {
    scan_from(src, 0, collect { })
}

// 再帰スキャナー
fn scan_from(src: String, pos: Int, acc: List<Token>) -> List<Token> {
    bind src_len <- String.length(src)
    if pos >= src_len {
        bind eof <- collect { yield Token { kind: "Eof"  text: ""  pos: pos }; }
        List.concat(acc, eof)
    } else {
        bind ch <- Option.unwrap_or(String.char_at(src, pos), "")
        // 空白・コメント・各トークン種別を処理
        // skip_whitespace / skip_comment / scan_ident / scan_int / scan_str / scan_op
        ...
    }
}
```

**キーワードチェック関数**:
```favnir
fn keyword_or_ident(text: String) -> String {
    if text == "fn" { "Keyword_fn" } else {
    if text == "public" { "Keyword_public" } else {
    // ... 全キーワード
    "Ident" }}...
}
```

**識別子スキャン**:
```favnir
fn scan_ident_len(src: String, pos: Int) -> Int {
    // pos から識別子文字（アルファベット・_・数字）が続く長さを返す
    // List.fold で実装
}

fn substr(src: String, from: Int, len: Int) -> String {
    // String.slice(src, from, from + len) があれば使う
    // なければ String.char_at で組み立て
}
```

### テスト（lexer.test.fav）

既存 4 件 + 以下を追加（計 40 件以上）:
- 各キーワードを単独でレキサーにかけてトークン種別を確認（20 件）
- 整数リテラル
- 文字列リテラル
- 2 文字トークン（`|>`, `->`, `=>`, `<-`, `??` 等）
- コメントスキップ
- 識別子
- 複数トークン混在（`fn double(n: Int) -> Int` 等）

---

## Phase 5: selfhost parser 基礎実装

### ファイル構成

```
selfhost/parser/
  ast.fav           ← 型定義のみ
  parser.fav        ← パーサー本体
  parser.test.fav   ← テスト
  main.fav          ← fav run 用エントリポイント
```

### ast.fav

```favnir
// AST の型定義
public type ParseError = { message: String  pos: Int }

public type TypeExpr =
    | TE_Int
    | TE_Float
    | TE_Bool
    | TE_String
    | TE_Unit
    | TE_List  { elem: TypeExpr }
    | TE_Option { elem: TypeExpr }
    | TE_Name  { name: String }

public type Expr =
    | E_Int   { value: Int }
    | E_Float { value: Float }
    | E_Bool  { value: Bool }
    | E_Str   { value: String }
    | E_Unit
    | E_Ident { name: String }
    | E_Call  { func: String  args: List<Expr> }
    | E_BinOp { op: String  left: Expr  right: Expr }
    | E_If    { cond: Expr  then_: Expr  else_: Expr }

public type Stmt =
    | S_Bind  { name: String  value: Expr }
    | S_Expr  { value: Expr }

public type Param = { name: String  ty: TypeExpr }

public type FnDef = {
    name:   String
    params: List<Param>
    ret:    TypeExpr
    body:   List<Stmt>
}

public type TopLevel =
    | TL_Fn { def: FnDef }
```

### parser.fav

```favnir
// パーサーコンビネータの基礎型
public type ParseState = {
    tokens: List<Token>
    pos:    Int
}

public type ParseOk<T> = {
    value: T
    state: ParseState
}

// 主要なパース関数（シグネチャ）
fn peek(state: ParseState) -> Token { ... }
fn advance(state: ParseState) -> ParseState { ... }
fn expect(state: ParseState, kind: String) -> Result<ParseState, ParseError> { ... }

public fn parse_program(tokens: List<Token>) -> Result<List<TopLevel>, ParseError> { ... }
fn parse_fn_def(state: ParseState) -> Result<ParseOk<FnDef>, ParseError> { ... }
fn parse_type_expr(state: ParseState) -> Result<ParseOk<TypeExpr>, ParseError> { ... }
fn parse_expr(state: ParseState) -> Result<ParseOk<Expr>, ParseError> { ... }
fn parse_stmt(state: ParseState) -> Result<ParseOk<Stmt>, ParseError> { ... }
fn parse_block(state: ParseState) -> Result<ParseOk<List<Stmt>>, ParseError> { ... }
fn parse_params(state: ParseState) -> Result<ParseOk<List<Param>>, ParseError> { ... }
```

### main.fav

```favnir
// fav run selfhost/parser/main.fav でパース結果を表示する
public fn main() -> Unit !Io {
    bind src <- "fn double(n: Int) -> Int { bind x <- n * 2; x }"
    bind tokens <- lex(src)
    bind result <- parse_program(tokens)
    match result {
        Ok(items) => IO.println($"Parsed {List.length(items)} top-level items")
        Err(e)    => IO.println($"Parse error at pos {e.pos}: {e.message}")
    }
}
```

### テスト（parser.test.fav、60 件以上）

パーサーの各部分をテスト:
- `parse_type_expr`: TE_Int, TE_Float, TE_List, TE_Option, TE_Name (各 1 件 = 5 件)
- `parse_expr`: E_Int, E_Float, E_Bool, E_Str, E_Ident, E_BinOp(+/-/*), E_If (各 1-2 件 = 15 件)
- `parse_stmt`: S_Bind, S_Expr (各 2 件 = 4 件)
- `parse_block`: 空ブロック、1文、複数文 (3 件)
- `parse_fn_def`: 引数なし、引数あり、型アノテーション付き (5 件)
- `parse_program`: 複数 fn def, エラーケース (8 件以上)
- 統合: lex → parse 結合テスト (10 件以上)

---

## Phase 6: driver.rs 統合テスト追加

selfhost の lexer・parser を `fav test` コマンド経由で実行するテストを
`src/driver.rs` に追加する。

```rust
#[test]
fn selfhost_lexer_tokenizes_keywords() {
    // fav test selfhost/lexer/lexer.test.fav を実行し全テストが通ることを確認
    let result = run_fav_test_file("selfhost/lexer/lexer.test.fav");
    assert!(result.all_passed);
}

#[test]
fn selfhost_parser_parses_fn_def() {
    // fav run selfhost/parser/main.fav が正常終了することを確認
    ...
}
```

既存の `run_fav_test_file` ヘルパーがあれば使う。なければ `cmd_test` を直接呼ぶ。

---

## Phase 7: `fav explain compiler` コマンド

### driver.rs

```rust
pub fn cmd_explain_compiler(file: &str) {
    let source = load_file(file);

    // Step 1: Lexer
    let tokens = Lexer::tokenize(&source, file).unwrap_or_else(|e| ...);
    println!("Step 1: Lexer      → {} tokens", tokens.len());

    // Step 2: Parser
    let program = Parser::parse_tokens(tokens.clone(), file).unwrap_or_else(|e| ...);
    let item_count = program.items.len();
    println!("Step 2: Parser     → {} top-level items", item_count);

    // Step 3: Checker
    let mut checker = Checker::new(PathBuf::from(file));
    let (errors, type_count) = checker.check_with_type_count(&program);
    println!("Step 3: Checker    → {} errors, {} types inferred", errors.len(), type_count);
    if !errors.is_empty() {
        for e in &errors { eprintln!("  {}: {}", e.code, e.message); }
        return;
    }

    // Step 4: Compiler (IR)
    let ir = compile_program(&program);
    println!("Step 4: Compiler   → {} IR functions", ir.functions.len());

    // Step 5: Codegen
    let artifact = codegen_program(&ir);
    let bytes = artifact.write_to_bytes();
    println!("Step 5: Codegen    → {} bytes (.fvc artifact)", bytes.len());
}
```

### main.rs

```
fav explain compiler <file>
```

サブコマンドとして追加。

---

## Phase 8: ドキュメント・最終確認

### `versions/v3.0.0/langspec.md`

- エラーコード体系 E0xxx の全リスト
- セルフホスト Step 1 の詳細（レキサー・パーサーの Favnir 実装）
- explain JSON スキーマ v3.0 の仕様（フィールド一覧）
- breaking changes のリスト

### `versions/v3.0.0/migration-guide.md`

エラーコード旧→新対応表（全コード）と、JSON スキーマ変更内容を記載。

### `versions/v3.0.0/progress.md`

全フェーズ完了後に更新。

---

## 実装上の注意点

### Phase 1（エラーコード）の注意

- `checker.rs` は 6000 行超。一括置換後に `cargo test` で確認する。
- テスト内の `"E013"` 等の文字列リテラルも全て変換が必要。
- `vm_stdlib_tests.rs` などのテストファイルも忘れずに更新。

### Phase 4（selfhost lexer）の注意

- Favnir に `while` ループがないため、再帰関数で実装する。
- 末尾再帰がスタックオーバーフローしないよう、深くネストしないように注意。
  - 長いソースをレキシングするとスタックが深くなる可能性がある。
  - `List.fold` の方が安全（スタック深さが O(1)）。
- `String.slice` が利用可能かどうか確認する（ない場合は `String.char_at` の繰り返しで実装）。

### Phase 5（selfhost parser）の注意

- パーサーコンビネータは `Result<ParseOk<T>, ParseError>` を返す。
  - `bind` を使ったチェーン: `bind r <- parse_expr(state)` → `r.value`, `r.state`
- ネストした型定義（`TypeExpr` の `TE_List { elem: TypeExpr }`）は Favnir でサポートされている。
  - ただし recursive variants には注意（現状の Favnir で循環参照が可能かどうか確認）。
- パーサーのテストは「lexer → parser」の統合テストが最も実用的。

### Phase 7（explain compiler）の注意

- `Checker::check_with_type_count` は存在しない可能性がある → `check_with_self` の戻り値を使う。
- `Lexer::tokenize` は Rust 側の lexer API の名前を確認する（`frontend::lexer::Lexer`）。
- コンパイルエラーがある場合は Step 3 で中断してエラーを表示する。
