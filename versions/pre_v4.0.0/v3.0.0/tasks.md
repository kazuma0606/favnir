# Favnir v3.0.0 タスクリスト

作成日: 2026-05-13

---

## Phase 0 — バージョン更新

- [ ] `Cargo.toml`: `version = "3.0.0"` に変更
- [ ] `src/main.rs`: HELP テキストを `v3.0.0` に更新
- [ ] `src/main.rs`: `print_welcome()` の バージョン表示を `"Favnir v3.0.0"` に更新

---

## Phase 1 — エラーコード移行（E0xxx 体系）

### `src/middle/checker.rs`

以下を一括置換（全 `type_error` 呼び出しのコード文字列）:

- [ ] `"E001"` → `"E0101"`
- [ ] `"E002"` → `"E0102"`
- [ ] `"E003"` → `"E0103"`
- [ ] `"E007"` → `"E0107"`
- [ ] `"E008"` → `"E0108"`
- [ ] `"E009"` → `"E0109"`
- [ ] `"E010"` → `"E0110"`
- [ ] `"E012"` → `"E0112"`
- [ ] `"E013"` → `"E0213"`
- [ ] `"E014"` → `"E0214"`
- [ ] `"E015"` → `"E0215"`
- [ ] `"E018"` → `"E0218"`
- [ ] `"E019"` → `"E0219"`
- [ ] `"E020"` → `"E0220"`
- [ ] `"E021"` → `"E0221"`
- [ ] `"E022"` → `"E0222"`
- [ ] `"E023"` → `"E0223"`
- [ ] `"E024"` → `"E0224"`
- [ ] `"E025"` → `"E0225"`
- [ ] `"E026"` → `"E0226"`
- [ ] `"E027"` → `"E0227"`
- [ ] `"E036"` → `"E0136"`
- [ ] `"E041"` → `"E0241"`
- [ ] `"E042"` → `"E0242"`
- [ ] `"E043"` → `"E0243"`
- [ ] `"E044"` → `"E0244"`
- [ ] `"E045"` → `"E0245"`
- [ ] `"E046"` → `"E0246"`
- [ ] `"E048"` → `"E0248"`
- [ ] `"E049"` → `"E0249"`
- [ ] `"E051"` → `"E0251"`
- [ ] `"E052"` → `"E0252"`
- [ ] `"E053"` → `"E0253"`
- [ ] `"E054"` → `"E0254"`
- [ ] `"E065"` → `"E0365"`
- [ ] `"E066"` → `"E0366"`
- [ ] `"E068"` → `"E0368"`
- [ ] `"E069"` → `"E0369"`
- [ ] `"E070"` → `"E0370"`
- [ ] `"E071"` → `"E0371"`
- [ ] `"E072"` → `"E0372"`
- [ ] `"E073"` → `"E0373"`
- [ ] `"E074"` → `"E0274"`
- [ ] `"E080"` → `"E0580"`
- [ ] `"E081"` → `"E0581"`
- [ ] `"E000"` → `"E0500"`

### `src/frontend/parser.rs`（deprecated keyword codes）

- [ ] `"E2001"` → `"E0901"`
- [ ] `"E2002"` → `"E0902"`
- [ ] `"E2003"` → `"E0903"`

### テストファイルの更新（エラーコード文字列を新コードに変更）

- [ ] `src/middle/checker.rs`（`#[cfg(test)]` 内）
  - 全 `assert_eq!(errors[0].code, "E0xx")` を新コードに更新
  - 全 `check_error_code(src, "E0xx")` 呼び出しを新コードに更新
- [ ] `src/driver.rs`（テスト内）
  - エラーコードを文字列として比較しているテストを全て更新
- [ ] `src/backend/vm_stdlib_tests.rs`（存在する場合）
- [ ] その他のテストファイル（`cargo test` で確認しながら修正）

### 動作確認

- [ ] `cargo test` を実行し、全テストが通ることを確認
- [ ] ソースコード内に旧 3 桁コード（`"E001"`, `"E013"` 等）が残っていないことを確認
  ```bash
  grep -r '"E0[0-9][0-9]"' src/  # ヒットしないこと
  ```

---

## Phase 2 — `fav explain-error` コマンド

### `src/error_catalog.rs`（新規ファイル）

- [ ] `ErrorEntry` 構造体を定義する（`code`, `title`, `description`, `example`, `fix`）
- [ ] `ERROR_CATALOG: &[ErrorEntry]` を作成し、以下の主要コードのエントリを追加:
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
- [ ] `pub fn lookup(code: &str) -> Option<&'static ErrorEntry>` を実装
- [ ] `pub fn list_all() -> Vec<&'static ErrorEntry>` を実装

### `src/main.rs`

- [ ] `error_catalog` モジュールを追加（`pub mod error_catalog;`）

### `src/driver.rs`

- [ ] `cmd_explain_error(code: &str)` 関数を実装:
  - `error_catalog::lookup(code)` でエントリを検索
  - 見つかった場合: コード・タイトル・説明・例・修正方法を表示
  - 見つからない場合: エラーメッセージ + `--list` ヒント
- [ ] `cmd_explain_error_list()` 関数を実装（全コード一覧を表示）

### `src/main.rs`

- [ ] `fav explain-error <code>` サブコマンドを追加
- [ ] `fav explain-error --list` オプションを追加
- [ ] HELP テキストに `explain-error` を追加

### テスト（`src/driver.rs`）

- [ ] `explain_error_known_code_prints_title` テスト
- [ ] `explain_error_unknown_code_exits_with_error` テスト
- [ ] `explain_error_list_shows_multiple_codes` テスト
- [ ] `explain_error_e0213_shows_type_mismatch` テスト
- [ ] `explain_error_e0901_shows_deprecated_trf` テスト

---

## Phase 3 — explain JSON スキーマ v3.0

### `src/driver.rs`

- [ ] `schema_version: "1.0"` → `"3.0"` に変更
- [ ] `favnir_version: "1.5.0"` → `env!("CARGO_PKG_VERSION")` に変更
- [ ] `"trfs"` キー → `"stages"` にリネーム
- [ ] `"flws"` キー → `"seqs"` にリネーム
- [ ] 関連するデシリアライズ・参照コードを全て `stages`/`seqs` に更新

### テスト更新

- [ ] `assert_eq!(value["schema_version"], "1.0")` → `"3.0"` に変更
- [ ] `assert_eq!(value["favnir_version"], "1.5.0")` → `"3.0.0"` に変更
- [ ] `value["trfs"]` → `value["stages"]` に変更
- [ ] `value["flws"]` → `value["seqs"]` に変更

### 新テスト

- [ ] `explain_json_schema_version_is_3_0` テストを追加
- [ ] `explain_json_has_stages_key_not_trfs` テストを追加
- [ ] `explain_json_favnir_version_is_current` テストを追加

---

## Phase 4 — selfhost lexer 完成（全トークン対応）

### `fav/selfhost/lexer/lexer.fav`

- [ ] `Token` 型を定義する（`kind: String  text: String  pos: Int`）
- [ ] `public fn lex(src: String) -> List<Token>` を実装
- [ ] `scan_from(src, pos, acc)` 再帰ヘルパーを実装:
  - [ ] 空白・改行のスキップ
  - [ ] `//` コメントのスキップ（次の `\n` まで）
  - [ ] 単一文字トークン: `(`, `)`, `{`, `}`, `[`, `]`, `,`, `;`, `@`, `%`
  - [ ] 2 文字トークン（ルックアヘッド付き）:
    - `|>`, `->`, `=>`, `<-`, `??`, `==`, `!=`, `<=`, `>=`, `&&`, `||`
  - [ ] 単独の 1 文字演算子: `=`, `<`, `>`, `+`, `-`, `*`, `/`, `.`, `:`, `|`, `&`, `!`
  - [ ] 識別子・キーワードスキャン（`scan_ident_from`）:
    - [ ] `scan_ident_from(src, pos) -> Int`（識別子終端の pos を返す）
    - [ ] `keyword_or_ident(text: String) -> String`（キーワード判定）
    - 対応キーワード: `fn`, `public`, `stage`, `seq`, `type`, `interface`, `impl`, `bind`, `match`, `if`, `else`, `for`, `in`, `yield`, `collect`, `use`, `test`, `bench`, `async`, `import`, `true`, `false`
  - [ ] 整数リテラルスキャン（`scan_int_from`）
  - [ ] 浮動小数点リテラルスキャン（整数の後に `.` + 数字が続く場合）
  - [ ] 文字列リテラルスキャン（`"` から次の `"` まで）
  - [ ] EOF: pos >= src_len のとき `Eof` トークンを追加してリターン
- [ ] `public fn main() -> Unit !Io` でデモ出力

### `fav/selfhost/lexer/lexer.test.fav`

既存テスト（4 件）に加えて以下を追加（合計 40 件以上）:
- [ ] 各キーワードを単独でレキシング（20 件）
  - `fn`, `public`, `stage`, `seq`, `type`, `interface`, `impl`, `bind`,
    `match`, `if`, `else`, `for`, `in`, `yield`, `collect`, `use`,
    `test`, `bench`, `async`, `import`
- [ ] `true` → `Bool` トークン、`false` → `Bool` トークン（2 件）
- [ ] 2 文字トークン各種（`|>`, `->`, `=>`, `<-`, `??`, `==`, `!=` 等、10 件）
- [ ] 整数リテラル（`"42"` → `Int` トークン）
- [ ] 文字列リテラル（`"\"hello\""` → `Str` トークン）
- [ ] コメントスキップ（`// comment\nfn` → `Keyword_fn` のみ）
- [ ] 識別子（`"myVar"` → `Ident` トークン）
- [ ] 複数トークン（`fn double(n: Int) -> Int` の全トークン列を確認）

---

## Phase 5 — selfhost parser 基礎実装

### `fav/selfhost/parser/ast.fav`（新規）

- [ ] `ParseError = { message: String  pos: Int }` を定義
- [ ] `TypeExpr` variant 型を定義（`TE_Int`, `TE_Float`, `TE_Bool`, `TE_String`, `TE_Unit`, `TE_List`, `TE_Option`, `TE_Name`）
- [ ] `Expr` variant 型を定義（`E_Int`, `E_Float`, `E_Bool`, `E_Str`, `E_Unit`, `E_Ident`, `E_Call`, `E_BinOp`, `E_If`）
- [ ] `Stmt` variant 型を定義（`S_Bind`, `S_Expr`）
- [ ] `Param = { name: String  ty: TypeExpr }` を定義
- [ ] `FnDef = { name: String  params: List<Param>  ret: TypeExpr  body: List<Stmt> }` を定義
- [ ] `TopLevel` variant 型を定義（`TL_Fn`）

### `fav/selfhost/parser/parser.fav`（新規）

- [ ] `ParseState = { tokens: List<Token>  pos: Int }` を定義
- [ ] `ParseOk<T> = { value: T  state: ParseState }` を定義（NOTE: Favnir のジェネリクス対応に注意）
- [ ] `peek(state: ParseState) -> Token` を実装
- [ ] `advance(state: ParseState) -> ParseState` を実装
- [ ] `expect(state: ParseState, kind: String) -> Result<ParseState, ParseError>` を実装
- [ ] `parse_type_expr(state: ParseState) -> Result<..., ParseError>` を実装:
  - `Int`, `Float`, `Bool`, `String`, `Unit` → 対応する `TE_*`
  - `List<T>` → `TE_List { elem: ... }`
  - `Option<T>` → `TE_Option { elem: ... }`
  - その他識別子 → `TE_Name { name: ... }`
- [ ] `parse_expr_primary(state) -> Result<..., ParseError>` を実装:
  - `Int` トークン → `E_Int`
  - `Float` トークン → `E_Float`
  - `Bool` トークン → `E_Bool`
  - `Str` トークン → `E_Str`
  - `Ident` + `(` → `E_Call`
  - `Ident` → `E_Ident`
  - `(` → `E_Unit`（`()` のみ）
  - `if` → `E_If`
- [ ] `parse_expr(state)` を実装（二項演算子の優先順位付き）
- [ ] `parse_stmt(state)` を実装:
  - `bind` → `S_Bind`
  - その他 → `S_Expr`
- [ ] `parse_block(state)` を実装（`{` から `}` まで Stmt を収集）
- [ ] `parse_params(state)` を実装（`(` から `)` まで `name: Type` を収集）
- [ ] `parse_fn_def(state)` を実装
- [ ] `parse_program(tokens)` を実装（トークンリストから `List<TopLevel>` を返す）

### `fav/selfhost/parser/main.fav`（新規）

- [ ] `lex` を `selfhost/lexer/lexer.fav` から使えるように組み込む（または inline で定義）
- [ ] `main()` でサンプル Favnir コードを lex → parse して結果を表示

### `fav/selfhost/parser/parser.test.fav`（新規）

60 件以上のテストを作成:

**TypeExpr テスト（5 件）**:
- [ ] `Int` → `TE_Int`
- [ ] `Float` → `TE_Float`
- [ ] `Bool` → `TE_Bool`
- [ ] `List<Int>` → `TE_List { elem: TE_Int }`
- [ ] `Option<String>` → `TE_Option { elem: TE_String }`

**Expr テスト（15 件）**:
- [ ] 整数リテラル `42` → `E_Int { value: 42 }`
- [ ] 浮動小数点 `3.14` → `E_Float { value: 3.14 }`
- [ ] 真偽値 `true` → `E_Bool { value: true }`
- [ ] 文字列 `"hello"` → `E_Str { value: "hello" }`
- [ ] 識別子 `x` → `E_Ident { name: "x" }`
- [ ] 加算 `1 + 2` → `E_BinOp { op: "+" ... }`
- [ ] 減算 `x - 1` → `E_BinOp { op: "-" ... }`
- [ ] 乗算 `a * b` → `E_BinOp { op: "*" ... }`
- [ ] 関数呼び出し `foo(1, 2)` → `E_Call { func: "foo" ... }`
- [ ] `if` 式 → `E_If { ... }`
- [ ] ネストした算術 `(1 + 2) * 3`
- その他 4 件

**Stmt テスト（4 件）**:
- [ ] `bind x <- 42` → `S_Bind { name: "x"  value: E_Int { value: 42 } }`
- [ ] `bind y <- x + 1` → `S_Bind { name: "y"  value: E_BinOp { ... } }`
- [ ] 式文 `f(x)` → `S_Expr`
- [ ] `bind` と式文の混在

**Block テスト（3 件）**:
- [ ] 空ブロック `{}` → `[]`
- [ ] 1 文のブロック
- [ ] 複数文のブロック

**FnDef テスト（5 件）**:
- [ ] 引数なし: `fn hello() -> Unit { ... }`
- [ ] 引数 1 個: `fn double(n: Int) -> Int { bind x <- n * 2; x }`
- [ ] 引数 2 個: `fn add(a: Int, b: Int) -> Int { a + b }`
- [ ] List 戻り型: `fn range(...) -> List<Int> { ... }`
- [ ] エラーケース: `)` が欠けている → `ParseError`

**統合テスト（10 件以上）**:
- [ ] lex → parse_program の結合テスト
- [ ] 複数 fn def のプログラム
- [ ] コメント入りプログラム
- [ ] その他エッジケース

---

## Phase 6 — driver.rs 統合テスト追加

### `src/driver.rs`

- [ ] `selfhost_lexer_all_tests_pass` テストを追加
  - `selfhost/lexer/lexer.test.fav` を `fav test` 相当で実行し、全テスト通過を確認
- [ ] `selfhost_lexer_tokenizes_fn_keyword` テストを追加
  - 小さなスニペットをレキシングして結果を確認
- [ ] `selfhost_parser_parses_simple_fn_def` テストを追加
  - `fn double(n: Int) -> Int { n * 2 }` を lex → parse して `FnDef` が返ることを確認
- [ ] `selfhost_parser_all_tests_pass` テストを追加
  - `selfhost/parser/parser.test.fav` を `fav test` 相当で実行
- [ ] `selfhost_parser_returns_error_on_invalid_input` テストを追加

---

## Phase 7 — `fav explain compiler` コマンド

### `src/driver.rs`

- [ ] `cmd_explain_compiler(file: &str)` 関数を実装:
  - Step 1: レキサー実行 → トークン数を表示
  - Step 2: パーサー実行 → トップレベルアイテム数を表示
  - Step 3: 型チェッカー実行 → エラー数・型推論件数を表示
  - Step 4: コンパイラ実行 → IR 関数数を表示
  - Step 5: コード生成 → バイト数を表示
  - エラーがある場合はそこで中断してエラーを表示
- [ ] `cmd_explain_compiler` のテストを追加（3 件）:
  - 正常ケース: 出力に "Step 1: Lexer" が含まれることを確認
  - エラーがある場合: Step 3 で中断することを確認
  - `fav.toml` があるプロジェクトで動くことを確認

### `src/main.rs`

- [ ] `fav explain compiler <file>` サブコマンドを追加:
  - `fav explain <file>` と区別するためサブコマンド形式にする
  - `cmd_explain_compiler(file)` を呼ぶ
- [ ] HELP テキストに `explain compiler` を追加

---

## Phase 8 — ドキュメント・最終確認

### 最終テスト確認

- [ ] `cargo build` で警告ゼロを確認
- [ ] `cargo test` で全テスト通過を確認（v2.9.0: 637 → 目標 ~678）
- [ ] selfhost lexer: `fav test selfhost/lexer/lexer.test.fav` で 40 件以上通過
- [ ] selfhost parser: `fav test selfhost/parser/parser.test.fav` で 60 件以上通過

### ドキュメント作成

- [ ] `versions/v3.0.0/langspec.md` を作成:
  - エラーコード体系 E0xxx の全リスト
  - selfhost Step 1 の実装詳細
  - explain JSON スキーマ v3.0 のフィールド仕様
  - `fav explain-error` / `fav explain compiler` コマンドの使い方
  - breaking changes のリスト

- [ ] `versions/v3.0.0/migration-guide.md` を作成:
  - エラーコード旧→新対応表（全 45 コード）
  - explain JSON キー変更（`trfs`→`stages`, `flws`→`seqs`）
  - `schema_version` 変更（`1.0`→`3.0`）

- [ ] `versions/v3.0.0/progress.md` を更新（全フェーズ完了）
- [ ] `versions/v3.0.0/tasks.md` を更新（全チェックボックスを完了に）

---

## 完了条件チェック

- [ ] `Cargo.toml` バージョンが `"3.0.0"`
- [ ] ソースに旧 3 桁エラーコードが残っていない
- [ ] `fav explain-error E0213` がエラー説明を表示する
- [ ] `fav explain-error --list` が全コード一覧を表示する
- [ ] explain JSON に `"schema_version": "3.0"` が含まれる
- [ ] explain JSON に `"stages"` キーがある（`"trfs"` ではない）
- [ ] explain JSON に `"seqs"` キーがある（`"flws"` ではない）
- [ ] `fav run selfhost/lexer/lexer.fav` が全トークンを認識する
- [ ] `fav test selfhost/lexer/lexer.test.fav` で 40 件以上が通る
- [ ] `fav run selfhost/parser/main.fav` が正常終了する
- [ ] `fav test selfhost/parser/parser.test.fav` で 60 件以上が通る
- [ ] selfhost lexer + parser 合計で 100 件以上のテストが通る
- [ ] `fav explain compiler src/main.fav` が 5 ステップのサマリーを表示する
- [ ] `cargo test` 全テスト通過（目標 ~678）
- [ ] `versions/v3.0.0/langspec.md` 作成済み
- [ ] `versions/v3.0.0/migration-guide.md` 作成済み
