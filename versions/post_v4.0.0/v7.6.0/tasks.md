# Favnir v7.6.0 Tasks

Date: 2026-05-28
Theme: CLI 部分セルフホスト化（cli.fav — check / explain --lineage / rune / version / help）

---

## Phase A: VM プリミティブ追加

### vm.rs

- [x] A-1: `IO.write_stderr_raw(msg: String) -> Unit` — `eprintln!` ラッパー
- [x] A-2: `IO.exit_raw(code: Int) -> Unit` — `std::process::exit(code)`
- [x] A-3: `Compiler.check_raw(path: String) -> Result<String, String>` — Rust 型チェックを呼び出し
- [x] A-4: `Compiler.lineage_text_raw(path: String) -> String` — Rust lineage 解析を呼び出し

### checker.rs

- [x] A-5: `("IO", "write_stderr_raw")` → `Type::Unit`
- [x] A-6: `("IO", "exit_raw")` → `Type::Unit`
- [x] A-7: `("Compiler", "check_raw")` → `Type::Result(String, String)`
- [x] A-8: `("Compiler", "lineage_text_raw")` → `Type::String`

---

## Phase B: fav/self/cli.fav

- [x] B-1: `type CliCmd = CmdCheck | CmdVersion | CmdHelp | CmdLineage | CmdRuneList | CmdRuneInfo | CmdUnknown` 型定義
- [x] B-2: `fn eq_str / fn not_flag / fn find_flag / fn find_positional` — ユーティリティ関数
- [x] B-3: `fn parse_cmd / parse_named_cmd / parse_check_cmd / parse_explain_cmd / parse_rune_cmd / parse_rune_sub` — 引数パーサー
- [x] B-4: `fn run_version / run_help` — 情報出力
- [x] B-5: `fn run_check(path)` — `Compiler.check_raw` を呼び出し
- [x] B-6: `fn run_lineage(path)` — `Compiler.lineage_text_raw` を呼び出し
- [x] B-7: `fn print_list / fn run_rune_list` — rune_modules/ 一覧
- [x] B-8: `fn run_rune_info(name)` — rune パス情報出力
- [x] B-9: `public fn main()` — argv 取得 → parse_cmd → dispatch
- [x] B-10: `fav check fav/self/cli.fav` — no errors

---

## Phase C: テスト（driver.rs）

### cli_self_host_tests（4 件）

- [x] C-1: `cli_version_test` — version → 出力に "favnir" を含む
- [x] C-2: `cli_help_test` — help → 出力に "COMMANDS:" を含む
- [x] C-3: `cli_check_valid_test` — check fav/tmp/hello.fav → "ok:" で始まる
- [x] C-4: `cli_rune_list_test` — rune list → panic しない

---

## Phase D: ドキュメント

- [x] D-1: `site/content/docs/language/self-host-cli.mdx` 作成
  - セルフホスト CLI アーキテクチャ（cli.fav / VM primitive / Rust bridge の関係）
  - cli.fav の CliCmd 型・パーサー構造
  - 使用例（`fav run fav/self/cli.fav -- check foo.fav`）
  - v7.7.0〜 へのロードマップ（checker.fav が完成したら Compiler.check_raw を置き換え）

---

## Phase E: 最終確認

- [x] E-1: `cargo test` — 1091+ tests passing（+4 新規）
- [x] E-2: `fav check fav/self/cli.fav` — no errors
- [x] E-3: このファイルを完了状態に更新
- [x] E-4: commit

---

## 完了条件

- `fav/self/cli.fav` が `fav check` を通る
- 統合テスト 4 件追加済み
- 既存テスト 1087 件が全件通る（1091+ passing）
- ドキュメント 1 ページ追加

---

## 実装ノート（既知の制約）

- `bind inside closure 不可` → `|a| eq_str(a, flag)` のように外側ヘルパーを呼ぶ
- `else if` 非対応 → `else { if ... }` と書く
- `IO.exit_raw` は diverging だが Favnir に diverging type はない → `Type::Unit` で登録
- `Compiler.check_raw` の Rust 実装は `Checker::check_program(&program)` を呼ぶ
- **重要**: `"Compiler"` を compiler.rs のビルトイン名前空間リスト（2箇所）と vm.rs の `is_known_builtin_namespace` に追加する必要があった
  - 追加しないと `Compiler.check_raw` が `VMValue::VariantCtor` として解釈されランタイムで silent fail する
  - 同様に `Cache`/`Queue`/`Email` も vm.rs に追加（既存コードの整合性）
- 既存の `IO.list_dir_raw` / `IO.is_dir_raw` / `IO.path_join_raw` / `IO.cwd_raw` は v7.3.0〜v7.5.0 で追加済み
