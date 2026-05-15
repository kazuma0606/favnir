# Favnir v1.6.0 タスク一覧 — 言語表現力 + 開発ループ改善

作成日: 2026-05-08

> **ゴール**: 文字列補間・レコード分解パターン・テストツール改善・ファイル監視により、
> 日常的なコーディングの表現力と開発体験を向上させる。
>
> **前提**: v1.5.0 完了（462 テスト通過）
>
> **スコープ管理が最優先。Done definition を超えない。**

---

## Phase 0: バージョン更新

### 0-1: バージョン更新

- [x] `Cargo.toml` の `version` を `"1.6.0"` に更新
- [x] `Cargo.toml` に `notify = { version = "6", default-features = false, features = ["macos_kqueue"] }` を追加
- [x] `main.rs` の HELP テキストを `v1.6.0` に更新
- [x] HELP に `watch` コマンドの説明を追加
- [x] `cargo build` が通ること

---

## Phase 1: 文字列補間

### 1-1: `ast.rs` の変更

- [x] `FStringPart` enum を追加（`Lit(String)` / `Expr(Box<Expr>)`）
- [x] `Expr::FString(Vec<FStringPart>, Span)` バリアントを追加
- [x] `Expr::span()` に `Expr::FString(_, sp) => sp` を追加

### 1-2: `lexer.rs` の変更

- [x] `TokenKind::FStringRaw(String)` を追加
- [x] `$"` を検出した場合に `lex_fstring_raw()` を呼び出す処理を追加
- [x] `lex_fstring_raw()` を実装
  - [x] `{` / `}` のネスト深さを追跡して補間式を抽出
  - [x] `\{` をリテラル `{` としてエスケープ
  - [x] 未終端 FString（末尾の `"` がない）を `LexError` として返す
- [x] `test_fstring_raw_token`: `$"Hello {name}!"` が `FStringRaw` トークンになる

### 1-3: `parser.rs` の変更

- [x] `parse_primary` に `TokenKind::FStringRaw` の分岐を追加
- [x] `parse_fstring_parts(raw: &str, base_span: Span) -> Result<Expr, ParseError>` を実装
  - [x] raw 文字列を `{...}` 境界で `Lit` / `Expr ソース` に分割
  - [x] Expr ソースを `Parser::parse_str_expr` で再帰的にパース
  - [x] `Expr::FString(parts, span)` を返す
- [x] パーサーテスト: `$"Hello {name}!"` がパースできる
- [x] パーサーテスト: `$"x={x} y={y}"` のように複数補間がパースできる
- [x] パーサーテスト: `$"literal only"` (補間なし) がパースできる

### 1-4: `checker.rs` の変更

- [x] `check_expr` の `Expr::FString` ケースを追加
  - [x] 各 `FStringPart::Expr` の型を `check_expr` で検査
  - [x] `String` / `Int` / `Float` / `Bool` → OK（自動 `Debug.show` 適用）
  - [x] ユーザー定義型で Show 実装なし → E054
  - [x] 補間式の中に `Expr::FString` があれば E053
  - [x] 戻り型: `Type::String`
- [x] テスト: `fstring_e054_no_show` — Show 未実装型で E054 が発生する
- [x] テスト: `fstring_string_type_ok` — String 型補間でエラーなし

### 1-5: `compiler.rs` の変更

- [x] `compile_expr` の `Expr::FString` ケースを追加
  - [x] `FStringPart::Lit(s)` → `IRExpr::Lit(Str(s))`
  - [x] `FStringPart::Expr(inner)` で inner が String → そのまま使用
  - [x] `FStringPart::Expr(inner)` で inner が非 String → `IRExpr::Call(Debug.show, [inner])`
  - [x] 全パートを `++` 演算（または `String.concat` 呼び出し）で連結

### 1-6: テスト

- [x] テスト: `fstring_exec_correct_output` — `$"Hello {name}!"` が正しい文字列を生成する
- [x] テスト: `fstring_int_auto_show` — `$"Age: {age}"` で Int が自動変換される
- [x] テスト: `fstring_multiple_parts` — 複数補間が正しく結合される
- [x] テスト: `fstring_escape_brace` — `\{` がリテラル `{` として扱われる
- [x] テスト: `fstring_empty_interp` — 補間なし `$"hello"` が通常文字列と同じ結果になる
- [x] `cargo test` が全通過すること

---

## Phase 2: レコード分解パターン

### 2-1: `ast.rs` の変更

- [x] `RecordPatternField` 構造体（`field: String`, `pattern: Option<Box<Pattern>>`, `span: Span`）を追加
- [x] `Pattern::Record(Vec<RecordPatternField>, Span)` バリアントを追加
- [x] `Pattern::span()` に `Pattern::Record(_, sp) => sp` を追加

### 2-2: `parser.rs` の変更

- [x] `parse_pattern` に `TokenKind::LBrace` の分岐を追加して `parse_record_pattern` を呼ぶ
- [x] `parse_record_pattern()` を実装
  - [x] `{ field1, field2 }` の pun 形式をパース
  - [x] `{ field1: pat1, field2: pat2 }` の alias 形式をパース
  - [x] フィールドを `,` で区切って複数パース
  - [x] `}` で終了
- [x] パーサーテスト: `{ name, age }` がパースできる
- [x] パーサーテスト: `{ name: n, age: a }` がパースできる
- [x] パーサーテスト: `{ name }` (単一フィールド) がパースできる
- [x] パーサーテスト: ネスト `{ address: { city } }` がパースできる

### 2-3: `checker.rs` の変更

- [x] `check_pattern` に `Pattern::Record` ケースを追加
  - [x] scrutinee の型が Record でない場合 → E055
  - [x] フィールド名が Record 型に存在しない場合 → E056
  - [x] pun (`pattern: None`) → `env.define(field_name, field_ty)` で変数を束縛
  - [x] alias (`pattern: Some(sub_pat)`) → `check_pattern(sub_pat, field_ty)` を再帰呼び出し
  - [x] 戻り型: scrutinee の型
- [x] テスト: `record_pat_check_ok` — 正しいフィールド参照で型検査が通る
- [x] テスト: `record_pat_e055_non_record` — 非レコード型への適用で E055 が発生する
- [x] テスト: `record_pat_e056_unknown_field` — 存在しないフィールドで E056 が発生する

### 2-4: `ir.rs` の変更

- [x] `IRPattern::Record(Vec<(String, IRPattern)>)` バリアントを追加
- [x] `collect_binds_in_pattern` に `IRPattern::Record` ケースを追加

### 2-5: `compiler.rs` の変更

- [x] `compile_pattern` に `Pattern::Record` ケースを追加
  - [x] pun フィールド → `(field_name, IRPattern::Bind(field_name))`
  - [x] alias フィールド → `(field_name, compile_pattern(sub_pat))`
  - [x] `IRPattern::Record(compiled_fields)` を返す

### 2-6: `vm.rs` の変更

- [x] `match_pattern` に `IRPattern::Record(fields)` ケースを追加
  - [x] value が `VMValue::Record(map)` であることを確認
  - [x] 各フィールドを `map.get(field)` で取得して sub_pattern と照合
  - [x] 全フィールドが一致すれば `true`、一つでも不一致なら `false`

### 2-7: テスト

- [x] テスト: `record_pat_exec_pun` — `{ name, age }` pun が実行時に正しく束縛される
- [x] テスト: `record_pat_exec_alias` — `{ name: n }` alias が実行時に正しく束縛される
- [x] テスト: `record_pat_partial` — 部分一致（フィールドの一部のみ指定）が動く
- [x] テスト: `record_pat_nested` — ネストしたレコード分解 `{ addr: { city } }` が動く
- [x] テスト: `record_pat_with_guard` — ガード付き `{ age } if age >= 18 ->` が動く
- [x] `cargo test` が全通過すること

---

## Phase 3: `fav test` 強化

### 3-1: `--filter` フラグの実装

- [x] `cmd_test` シグネチャに `filter: Option<&str>` を追加
- [x] テスト説明文（`test "desc" { ... }` の `desc`）に対してフィルターを適用
  - [x] `filter` が `None` → 全テスト実行
  - [x] `filter` がカンマ区切り → いずれかのパターンが部分一致すれば実行
- [x] フィルター適用で除外されたテストを `filtered` カウントに計上
- [x] `main.rs` の `test` コマンドに `--filter <pattern>` フラグを追加

### 3-2: テスト統計の改善

- [x] `TestResult` 構造体（`description`, `passed`, `error_msg`, `elapsed_ms`）を定義
- [x] 各テストの実行時間を `std::time::Instant` で計測
- [x] `format_test_results(results, filtered) -> String` を実装
  - [x] `PASS  desc  (0.2ms)` 形式の出力
  - [x] `FAIL  desc\n        error message` 形式の出力
  - [x] `test result: N passed; M failed; K filtered; finished in Xms` のサマリー
- [x] `running N tests in <file>` のヘッダー出力

### 3-3: `assert_matches` ビルトインの追加

- [x] `ast.rs` に `Expr::AssertMatches(Box<Expr>, Box<Pattern>, Span)` を追加
- [x] `parser.rs` に `parse_assert_matches()` を追加
  - [x] `assert_matches(<expr>, <pattern>)` をパース
  - [x] `parse_expr` + `parse_pattern` を呼ぶ
- [x] `checker.rs` に `Expr::AssertMatches` ケースを追加
  - [x] expr の型を検査
  - [x] pattern を expr の型に対して `check_pattern` で検査
  - [x] 戻り型: `Type::Unit`
- [x] `compiler.rs` / `vm.rs` に `Expr::AssertMatches` の実装を追加
  - [x] `match val { pattern -> () | _ -> assert_fail("assert_matches failed") }` に脱糖

### 3-4: `--no-capture` フラグの実装

- [x] `cmd_test` シグネチャに `no_capture: bool` を追加
- [x] `no_capture == false`（デフォルト）時、テスト本体の IO 出力を抑制
- [x] `main.rs` の `test` コマンドに `--no-capture` フラグを追加

### 3-5: テスト

- [x] テスト: `test_filter_matches_description` — フィルターで一致するテストだけ実行される
- [x] テスト: `test_filter_excludes_non_matching` — フィルターで一致しないテストが除外される
- [x] テスト: `test_stats_summary_format` — サマリーに passed/failed/filtered が含まれる
- [x] テスト: `assert_matches_some_ok` — `assert_matches(some_val, some(_))` が通る
- [x] テスト: `assert_matches_fail` — 不一致で assert_matches がテスト失敗を引き起こす
- [x] `cargo test` が全通過すること

---

## Phase 4: `fav watch` コマンド

### 4-1: `driver.rs` の変更

- [x] `collect_watch_paths(file: Option<&str>) -> Vec<PathBuf>` を実装
  - [x] `file` が `Some` → そのファイルのみ
  - [x] `file` が `None` → `fav.toml` の src ディレクトリから `.fav` ファイルを収集
  - [x] `fav.toml` がない場合はカレントディレクトリの `.fav` ファイルを収集
- [x] `run_watch_cmd(file: Option<&str>, cmd: &str)` を実装
  - [x] `cmd` をカンマ区切りで分割して `check` / `test` / `run` を順番に実行
- [x] `pub fn cmd_watch(file: Option<&str>, cmd: &str)` を実装
  - [x] `notify::recommended_watcher` でウォッチャーを生成
  - [x] `mpsc::channel` でイベントを受信
  - [x] 変更イベント（Create/Modify/Remove）で 80ms デバウンス後に `run_watch_cmd` を呼ぶ
  - [x] `\x1b[2J\x1b[H` でターミナルをクリア
  - [x] `[watch] watching N files for changes...` を出力

### 4-2: `main.rs` の変更

- [x] `use driver::cmd_watch` を追加
- [x] `Some("watch")` コマンドのルーティングを追加
  - [x] `--cmd <value>` フラグをパース（デフォルト `"check"`）
  - [x] ファイル引数をパース
  - [x] `cmd_watch(file, &cmd_str)` を呼ぶ

### 4-3: テスト

- [x] テスト: `watch_collect_paths_returns_fav_files` — `collect_watch_paths` が `.fav` ファイルを返す（md などは除外）
- [x] テスト: `watch_collect_paths_excludes_non_fav` — `.md` / `.toml` などが除外される
- [x] `cargo test` が全通過すること

---

## Phase 5: テスト・ドキュメント

### 5-1: example ファイルの追加

- [x] `examples/fstring_demo.fav` を作成
  - [x] 文字列補間の基本・Int 自動変換・フィールドアクセスを示す
  - [x] `fav run` / `fav check` でエラーなしを確認
- [x] `examples/record_match.fav` を作成
  - [x] pun / alias / 部分一致 / ガード付きを示す
  - [x] `fav run` でエラーなしを確認

### 5-2: langspec.md の作成

- [x] `versions/v1.6.0/langspec.md` を新規作成
  - [x] `$"..."` 文字列補間構文と自動 `Debug.show` 変換ルール
  - [x] `\{` エスケープ
  - [x] E053（入れ子補間）/ E054（Show 未実装型）エラーコード
  - [x] レコード分解パターン構文（pun / alias / partial / nested）
  - [x] E055（非レコード型）/ E056（存在しないフィールド）エラーコード
  - [x] `assert_matches(<expr>, <pattern>)` ビルトイン
  - [x] `fav test --filter` / `--no-capture` フラグ
  - [x] テスト統計出力フォーマット
  - [x] `fav watch [--cmd <check|test|run>]` コマンド

### 5-3: README.md の更新

- [x] v1.6.0 セクションを追加（文字列補間 / レコード分解 / テスト改善 / watch の紹介）

### 5-4: 全体確認

- [x] `cargo build` で Rust コンパイラ警告ゼロ
- [x] `cargo test` 全テスト通過（v1.5.0 継承 462 + 新規テスト）
- [x] `$"Hello {name}!"` が正しい文字列を生成する
- [x] `match user { { name, age } -> ... }` が型検査・実行できる
- [x] `fav test --filter "keyword"` でフィルタリングが動作する
- [x] `assert_matches(some_val, some(_))` が型検査・実行できる
- [x] `fav watch` が `.fav` ファイルの変更を検出する
- [x] `Cargo.toml` バージョンが `"1.6.0"`

---

## 全体完了条件

- [x] `cargo build` で警告ゼロ
- [x] `cargo test` 全テスト通過
- [x] `$"Hello {name}!"` が正しい文字列補間結果を生成する
- [x] 非 String 補間式に `Debug.show` が自動適用される
- [x] Show 未実装型の補間で E054 が発生する
- [x] `match user { { name, age } -> ... }` がパース・実行できる
- [x] 存在しないフィールドのレコード分解で E056 が発生する
- [x] `fav test --filter "keyword"` でフィルタリングが動作する
- [x] テスト結果に passed/failed/filtered/時間が表示される
- [x] `assert_matches(value, some(_))` が型検査・実行できる
- [x] `fav watch --cmd check` が `.fav` ファイル変更を検出して再実行する
- [x] v1.5.0 の全テストが引き続き通る
- [x] `Cargo.toml` バージョンが `"1.6.0"`

---

## 実装差異メモ（Codex 実装との突合）

### 3-4: `--no-capture` フラグ（Claude Code 実装）

Codex 実装では `no_capture: bool` は `cmd_test` シグネチャに追加されていたが、
IO 抑制の実装本体（`set_suppress_io` 呼び出し）は stub（`if !no_capture { let _ = no_capture; }`）のままだった。
Claude Code が以下を追加実装した：

- **`vm.rs`**: スレッドローカル `SUPPRESS_IO_OUTPUT: Cell<bool>` + `pub fn set_suppress_io(bool)` + `fn is_io_suppressed() -> bool` を追加
  - IO 系 5 箇所（`IO.println` / `IO.println_int` / `IO.println_float` / `IO.println_bool` / `IO.print`）に `if !is_io_suppressed()` ガードを追加
- **`driver.rs`**: `cmd_test` のテストループ前後に `set_suppress_io(true)` / `set_suppress_io(false)` を追加

### watch テスト競合（フレーキーテスト修正）

`watch_collect_paths_returns_fav_files` と `watch_collect_paths_excludes_non_fav` の両テストが
`std::env::set_current_dir` （プロセスワイド操作）を並行実行して非決定的に失敗する問題を修正。

- **`driver.rs`**: `CWD_MUTEX: LazyLock<Mutex<()>>` を追加し、両テスト冒頭で `let _cwd_guard = CWD_MUTEX.lock()...` を取得

### テスト数

- v1.5.0 時点: 462 テスト
- v1.6.0 完了時: **483 テスト**（+21 新規）

---

## 先送り一覧（守る）

| 制約 | バージョン |
|---|---|
| 文字列補間内の入れ子補間（`$"outer {$"inner"}"`) | v2.0.0 以降 |
| レコード分解でのスプレッド（`{ name, ..rest }`） | v2.0.0 以降 |
| artifact の explain metadata 圧縮（gzip） | v2.0.0 |
| `PartialFlw` を型引数に取る関数 | v2.0.0 |
| `abstract flw` 継承 | v2.0.0 以降 |
| `abstract seq` / `abstract stage` / JSON キー renaming | v2.0.0 |
| Veltra との直接統合 | v2.0.0 以降 |
| `fav explain result`（Lineage Tracking） | v2.0.0 以降 |
| エフェクトの `use` による再エクスポート | v2.0.0 |
| エフェクト階層（`effect Foo extends Bar`） | v2.0.0 以降 |
| `fav lint` カスタムルールプラグイン | v2.0.0 以降 |
| `fav test --coverage` | v1.7.0 以降 |
| `fav watch` の複数ディレクトリ監視 | v1.7.0 以降 |
