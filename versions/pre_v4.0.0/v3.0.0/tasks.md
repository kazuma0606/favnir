# Favnir v3.0.0 タスクリスト

作成日: 2026-05-13

---

## Phase 0 — バージョン更新

- [x] `Cargo.toml`: `version = "3.0.0"` に変更
- [x] `src/main.rs`: HELP テキストを `v3.0.0` に更新
- [x] `src/main.rs`: `print_welcome()` の バージョン表示を `"Favnir v3.0.0"` に更新

---

## Phase 1 — エラーコード移行（E0xxx 体系）

### `src/middle/checker.rs`

以下を一括置換（全 `type_error` 呼び出しのコード文字列）:

- [x] `"E001"` → `"E0101"`
- [x] `"E002"` → `"E0102"`
- [x] `"E003"` → `"E0103"`
- [x] `"E007"` → `"E0107"`
- [x] `"E008"` → `"E0108"`
- [x] `"E009"` → `"E0109"`
- [x] `"E010"` → `"E0110"`
- [x] `"E012"` → `"E0112"`
- [x] `"E013"` → `"E0213"`
- [x] `"E014"` → `"E0214"`
- [x] `"E015"` → `"E0215"`
- [x] `"E018"` → `"E0218"`
- [x] `"E019"` → `"E0219"`
- [x] `"E020"` → `"E0220"`
- [x] `"E021"` → `"E0221"`
- [x] `"E022"` → `"E0222"`
- [x] `"E023"` → `"E0223"`
- [x] `"E024"` → `"E0224"`
- [x] `"E025"` → `"E0225"`
- [x] `"E026"` → `"E0226"`
- [x] `"E027"` → `"E0227"`
- [x] `"E036"` → `"E0136"`
- [x] `"E041"` → `"E0241"`
- [x] `"E042"` → `"E0242"`
- [x] `"E043"` → `"E0243"`
- [x] `"E044"` → `"E0244"`
- [x] `"E045"` → `"E0245"`
- [x] `"E046"` → `"E0246"`
- [x] `"E048"` → `"E0248"`
- [x] `"E049"` → `"E0249"`
- [x] `"E051"` → `"E0251"`
- [x] `"E052"` → `"E0252"`
- [x] `"E053"` → `"E0253"`
- [x] `"E054"` → `"E0254"`
- [x] `"E065"` → `"E0365"`
- [x] `"E066"` → `"E0366"`
- [x] `"E068"` → `"E0368"`
- [x] `"E069"` → `"E0369"`
- [x] `"E070"` → `"E0370"`
- [x] `"E071"` → `"E0371"`
- [x] `"E072"` → `"E0372"`
- [x] `"E073"` → `"E0373"`
- [x] `"E074"` → `"E0274"`
- [x] `"E080"` → `"E0580"`
- [x] `"E081"` → `"E0581"`
- [x] `"E000"` → `"E0500"`

### `src/frontend/parser.rs`（deprecated keyword codes）

- [x] `"E2001"` → `"E0901"`
- [x] `"E2002"` → `"E0902"`
- [x] `"E2003"` → `"E0903"`

### テストファイルの更新（エラーコード文字列を新コードに変更）

- [x] `src/middle/checker.rs`（`#[cfg(test)]` 内）
  - 全 `assert_eq!(errors[0].code, "E0xx")` を新コードに更新
  - 全 `check_error_code(src, "E0xx")` 呼び出しを新コードに更新
- [x] `src/driver.rs`（テスト内）
  - エラーコードを文字列として比較しているテストを全て更新
- [x] `src/backend/vm_stdlib_tests.rs`（存在する場合）
- [x] その他のテストファイル（`cargo test` で確認しながら修正）

### 動作確認

- [x] `cargo test` を実行し、全テストが通ることを確認
- [x] ソースコード内に旧 3 桁コード（`"E001"`, `"E013"` 等）が残っていないことを確認
  ```bash
  grep -r '"E0[0-9][0-9]"' src/  # ヒットしないこと
  ```

---

## Phase 2 — `fav explain-error` コマンド

### `src/error_catalog.rs`（新規ファイル）

- [x] `ErrorEntry` 構造体を定義する（`code`, `title`, `description`, `example`, `fix`）
- [x] `ERROR_CATALOG: &[ErrorEntry]` を作成し、以下の主要コードのエントリを追加:
  - E0101, E0102, E0103（seq/stage 関連）
  - E0213, E0214, E0215（型不一致・未定義）
  - E0218, E0219, E0222（フィールド・関数参照）
  - E0224（非網羅的 match）
  - E0225, E0226（演算子・if 型）
  - E0241, E0242, E0243, E0244（interface 関連）
  - E0365, E0368（for / ?? 関連）
  - E0370, E0371（エフェクト関連）
  - E0580, E0581（モジュール関連）
  - E0901, E0902, E0903（deprecated keyword）
  - 合計 20 件以上
- [x] `pub fn lookup(code: &str) -> Option<&'static ErrorEntry>` を実装
- [x] `pub fn list_all() -> Vec<&'static ErrorEntry>` を実装

### `src/main.rs`

- [x] `error_catalog` モジュールを追加（`pub mod error_catalog;`）

### `src/driver.rs`

- [x] `cmd_explain_error(code: &str)` 関数を実装:
  - `error_catalog::lookup(code)` でエントリを検索
  - 見つかった場合: コード・タイトル・説明・例・修正方法を表示
  - 見つからない場合: エラーメッセージ + `--list` ヒント
- [x] `cmd_explain_error_list()` 関数を実装（全コード一覧を表示）

### `src/main.rs`

- [x] `fav explain-error <code>` サブコマンドを追加
- [x] `fav explain-error --list` オプションを追加
- [x] HELP テキストに `explain-error` を追加

### テスト（`src/driver.rs`）

- [x] `explain_error_known_code_prints_title` テスト
- [x] `explain_error_unknown_code_exits_with_error` テスト
- [x] `explain_error_list_shows_multiple_codes` テスト
- [x] `explain_error_e0213_shows_type_mismatch` テスト
- [x] `explain_error_e0901_shows_deprecated_trf` テスト

---

## Phase 3 — explain JSON スキーマ v3.0

### `src/driver.rs`

- [x] `schema_version: "1.0"` → `"3.0"` に変更
- [x] `favnir_version: "1.5.0"` → `env!("CARGO_PKG_VERSION")` に変更
- [x] `"trfs"` キー → `"stages"` にリネーム
- [x] `"flws"` キー → `"seqs"` にリネーム
- [x] 関連するデシリアライズ・参照コードを全て `stages`/`seqs` に更新

### テスト更新

- [x] `assert_eq!(value["schema_version"], "1.0")` → `"3.0"` に変更
- [x] `assert_eq!(value["favnir_version"], "1.5.0")` → `"3.0.0"` に変更
- [x] `value["trfs"]` → `value["stages"]` に変更
- [x] `value["flws"]` → `value["seqs"]` に変更

### 新テスト

- [x] `explain_json_schema_version_is_3_0` テストを追加
- [x] `explain_json_has_stages_key_not_trfs` テストを追加
- [x] `explain_json_favnir_version_is_current` テストを追加

---

## Phase 4 — selfhost lexer 完成（全トークン対応）

### `fav/selfhost/lexer/lexer.fav`

- [x] `Token` 型を定義する（`kind: String  text: String  pos: Int`）
- [x] `public fn lex(src: String) -> List<Token>` を実装
- [x] `scan_from(src, pos, acc)` 再帰ヘルパーを実装:
  - [x] 空白・改行のスキップ
  - [x] `//` コメントのスキップ（次の `\n` まで）
  - [x] 単一文字トークン: `(`, `)`, `{`, `}`, `[`, `]`, `,`, `;`, `@`, `%`
  - [x] 2 文字トークン（ルックアヘッド付き）:
    - `|>`, `->`, `=>`, `<-`, `??`, `==`, `!=`, `<=`, `>=`, `&&`, `||`
  - [x] 単独の 1 文字演算子: `=`, `<`, `>`, `+`, `-`, `*`, `/`, `.`, `:`, `|`, `&`, `!`
  - [x] 識別子・キーワードスキャン（`scan_ident_from`）:
    - [x] `scan_ident_from(src, pos) -> Int`（識別子終端の pos を返す）
    - [x] `keyword_or_ident(text: String) -> String`（キーワード判定）
    - 対応キーワード: `fn`, `public`, `stage`, `seq`, `type`, `interface`, `impl`, `bind`, `match`, `if`, `else`, `for`, `in`, `yield`, `collect`, `use`, `test`, `bench`, `async`, `import`, `true`, `false`
  - [x] 整数リテラルスキャン（`scan_int_from`）
  - [x] 浮動小数点リテラルスキャン（整数の後に `.` + 数字が続く場合）
  - [x] 文字列リテラルスキャン（`"` から次の `"` まで）
  - [x] EOF: pos >= src_len のとき `Eof` トークンを追加してリターン
- [x] `public fn main() -> Unit !Io` でデモ出力

### `fav/selfhost/lexer/lexer.test.fav`

既存テスト（4 件）に加えて以下を追加（合計 40 件以上）:
- [x] 各キーワードを単独でレキシング（20 件）
  - `fn`, `public`, `stage`, `seq`, `type`, `interface`, `impl`, `bind`,
    `match`, `if`, `else`, `for`, `in`, `yield`, `collect`, `use`,
    `test`, `bench`, `async`, `import`
- [x] `true` → `Bool` トークン、`false` → `Bool` トークン（2 件）
- [x] 2 文字トークン各種（`|>`, `->`, `=>`, `<-`, `??`, `==`, `!=` 等、10 件）
- [x] 整数リテラル（`"42"` → `Int` トークン）
- [x] 文字列リテラル（`"\"hello\""` → `Str` トークン）
- [x] コメントスキップ（`// comment\nfn` → `Keyword_fn` のみ）
- [x] 識別子（`"myVar"` → `Ident` トークン）
- [x] 複数トークン（`fn double(n: Int) -> Int` の全トークン列を確認）

---

## Phase 5 — selfhost parser 基礎実装

### `fav/selfhost/parser/ast.fav`（新規）

- [x] `ParseError = { message: String  pos: Int }` を定義
- [x] `TypeExpr` variant 型を定義（`TE_Int`, `TE_Float`, `TE_Bool`, `TE_String`, `TE_Unit`, `TE_List`, `TE_Option`, `TE_Name`）
- [x] `Expr` variant 型を定義（`E_Int`, `E_Float`, `E_Bool`, `E_Str`, `E_Unit`, `E_Ident`, `E_Call`, `E_BinOp`, `E_If`）
- [x] `Stmt` variant 型を定義（`S_Bind`, `S_Expr`）
- [x] `Param = { name: String  ty: TypeExpr }` を定義
- [x] `FnDef = { name: String  params: List<Param>  ret: TypeExpr  body: List<Stmt> }` を定義
- [x] `TopLevel` variant 型を定義（`TL_Fn`）

### `fav/selfhost/parser/parser.fav`（新規）

- [x] `ParseState = { tokens: List<Token>  pos: Int }` を定義
- [x] `ParseOk<T> = { value: T  state: ParseState }` を定義（NOTE: Favnir のジェネリクス対応に注意）
- [x] `peek(state: ParseState) -> Token` を実装
- [x] `advance(state: ParseState) -> ParseState` を実装
- [x] `expect(state: ParseState, kind: String) -> Result<ParseState, ParseError>` を実装
- [x] `parse_type_expr(state: ParseState) -> Result<..., ParseError>` を実装:
  - `Int`, `Float`, `Bool`, `String`, `Unit` → 対応する `TE_*`
  - `List<T>` → `TE_List { elem: ... }`
  - `Option<T>` → `TE_Option { elem: ... }`
  - その他識別子 → `TE_Name { name: ... }`
- [x] `parse_expr_primary(state) -> Result<..., ParseError>` を実装:
  - `Int` トークン → `E_Int`
  - `Float` トークン → `E_Float`
  - `Bool` トークン → `E_Bool`
  - `Str` トークン → `E_Str`
  - `Ident` + `(` → `E_Call`
  - `Ident` → `E_Ident`
  - `(` → `E_Unit`（`()` のみ）
  - `if` → `E_If`
- [x] `parse_expr(state)` を実装（二項演算子の優先順位付き）
- [x] `parse_stmt(state)` を実装:
  - `bind` → `S_Bind`
  - その他 → `S_Expr`
- [x] `parse_block(state)` を実装（`{` から `}` まで Stmt を収集）
- [x] `parse_params(state)` を実装（`(` から `)` まで `name: Type` を収集）
- [x] `parse_fn_def(state)` を実装
- [x] `parse_program(tokens)` を実装（トークンリストから `List<TopLevel>` を返す）

### `fav/selfhost/parser/main.fav`（新規）

- [x] `lex` を `selfhost/lexer/lexer.fav` から使えるように組み込む（または inline で定義）
- [x] `main()` でサンプル Favnir コードを lex → parse して結果を表示

### `fav/selfhost/parser/parser.test.fav`（新規）

60 件以上のテストを作成:

**TypeExpr テスト（5 件）**:
- [x] `Int` → `TE_Int`
- [x] `Float` → `TE_Float`
- [x] `Bool` → `TE_Bool`
- [x] `List<Int>` → `TE_List { elem: TE_Int }`
- [x] `Option<String>` → `TE_Option { elem: TE_String }`

**Expr テスト（15 件）**:
- [x] 整数リテラル `42` → `E_Int { value: 42 }`
- [x] 浮動小数点 `3.14` → `E_Float { value: 3.14 }`
- [x] 真偽値 `true` → `E_Bool { value: true }`
- [x] 文字列 `"hello"` → `E_Str { value: "hello" }`
- [x] 識別子 `x` → `E_Ident { name: "x" }`
- [x] 加算 `1 + 2` → `E_BinOp { op: "+" ... }`
- [x] 減算 `x - 1` → `E_BinOp { op: "-" ... }`
- [x] 乗算 `a * b` → `E_BinOp { op: "*" ... }`
- [x] 関数呼び出し `foo(1, 2)` → `E_Call { func: "foo" ... }`
- [x] `if` 式 → `E_If { ... }`
- [x] ネストした算術 `(1 + 2) * 3`
- その他 4 件

**Stmt テスト（4 件）**:
- [x] `bind x <- 42` → `S_Bind { name: "x"  value: E_Int { value: 42 } }`
- [x] `bind y <- x + 1` → `S_Bind { name: "y"  value: E_BinOp { ... } }`
- [x] 式文 `f(x)` → `S_Expr`
- [x] `bind` と式文の混在

**Block テスト（3 件）**:
- [x] 空ブロック `{}` → `[]`
- [x] 1 文のブロック
- [x] 複数文のブロック

**FnDef テスト（5 件）**:
- [x] 引数なし: `fn hello() -> Unit { ... }`
- [x] 引数 1 個: `fn double(n: Int) -> Int { bind x <- n * 2; x }`
- [x] 引数 2 個: `fn add(a: Int, b: Int) -> Int { a + b }`
- [x] List 戻り型: `fn range(...) -> List<Int> { ... }`
- [x] エラーケース: `)` が欠けている → `ParseError`

**統合テスト（10 件以上）**:
- [x] lex → parse_program の結合テスト
- [x] 複数 fn def のプログラム
- [x] コメント入りプログラム
- [x] その他エッジケース

---

## Phase 6 — driver.rs 統合テスト追加

### `src/driver.rs`

- [x] `selfhost_lexer_all_tests_pass` テストを追加
  - `selfhost/lexer/lexer.test.fav` を `fav test` 相当で実行し、全テスト通過を確認
- [x] `selfhost_lexer_tokenizes_fn_keyword` テストを追加
  - 小さなスニペットをレキシングして結果を確認
- [x] `selfhost_parser_parses_simple_fn_def` テストを追加
  - `fn double(n: Int) -> Int { n * 2 }` を lex → parse して `FnDef` が返ることを確認
- [x] `selfhost_parser_all_tests_pass` テストを追加
  - `selfhost/parser/parser.test.fav` を `fav test` 相当で実行
- [x] `selfhost_parser_returns_error_on_invalid_input` テストを追加

---

## Phase 7 — `fav explain compiler` コマンド

### `src/driver.rs`

- [x] `cmd_explain_compiler(file: &str)` 関数を実装:
  - Step 1: レキサー実行 → トークン数を表示
  - Step 2: パーサー実行 → トップレベルアイテム数を表示
  - Step 3: 型チェッカー実行 → エラー数・型推論件数を表示
  - Step 4: コンパイラ実行 → IR 関数数を表示
  - Step 5: コード生成 → バイト数を表示
  - エラーがある場合はそこで中断してエラーを表示
- [x] `cmd_explain_compiler` のテストを追加（3 件）:
  - 正常ケース: 出力に "Step 1: Lexer" が含まれることを確認
  - エラーがある場合: Step 3 で中断することを確認
  - `fav.toml` があるプロジェクトで動くことを確認

### `src/main.rs`

- [x] `fav explain compiler <file>` サブコマンドを追加:
  - `fav explain <file>` と区別するためサブコマンド形式にする
  - `cmd_explain_compiler(file)` を呼ぶ
- [x] HELP テキストに `explain compiler` を追加

---

## Phase 8 — ドキュメント・最終確認

### 最終テスト確認

- [x] `cargo build` で警告ゼロを確認
- [x] `cargo test` で全テスト通過を確認（v2.9.0: 637 → 目標 ~678）
- [x] selfhost lexer: `fav test selfhost/lexer/lexer.test.fav` で 40 件以上通過
- [x] selfhost parser: `fav test selfhost/parser/parser.test.fav` で 60 件以上通過

### ドキュメント作成

- [x] `versions/v3.0.0/langspec.md` を作成:
  - エラーコード体系 E0xxx の全リスト
  - selfhost Step 1 の実装詳細
  - explain JSON スキーマ v3.0 のフィールド仕様
  - `fav explain-error` / `fav explain compiler` コマンドの使い方
  - breaking changes のリスト

- [x] `versions/v3.0.0/migration-guide.md` を作成:
  - エラーコード旧→新対応表（全 45 コード）
  - explain JSON キー変更（`trfs`→`stages`, `flws`→`seqs`）
  - `schema_version` 変更（`1.0`→`3.0`）

- [x] `versions/v3.0.0/progress.md` を更新（全フェーズ完了）
- [x] `versions/v3.0.0/tasks.md` を更新（全チェックボックスを完了に）

---

## 完了条件チェック

- [x] `Cargo.toml` バージョンが `"3.0.0"`
- [x] ソースに旧 3 桁エラーコードが残っていない
- [x] `fav explain-error E0213` がエラー説明を表示する
- [x] `fav explain-error --list` が全コード一覧を表示する
- [x] explain JSON に `"schema_version": "3.0"` が含まれる
- [x] explain JSON に `"stages"` キーがある（`"trfs"` ではない）
- [x] explain JSON に `"seqs"` キーがある（`"flws"` ではない）
- [x] `fav run selfhost/lexer/lexer.fav` が全トークンを認識する
- [x] `fav test selfhost/lexer/lexer.test.fav` で 40 件以上が通る
- [x] `fav run selfhost/parser/main.fav` が正常終了する
- [x] `fav test selfhost/parser/parser.test.fav` で 60 件以上が通る
- [x] selfhost lexer + parser 合計で 100 件以上のテストが通る
- [x] `fav explain compiler src/main.fav` が 5 ステップのサマリーを表示する
- [x] `cargo test` 全テスト通過（目標 ~678）
- [x] `versions/v3.0.0/langspec.md` 作成済み
- [x] `versions/v3.0.0/migration-guide.md` 作成済み
