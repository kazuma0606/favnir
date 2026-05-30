# Favnir v7.6.0 Spec — CLI 部分セルフホスト化

Date: 2026-05-28
Theme: `fav check` / `fav explain --lineage` / `fav rune` コマンドを Favnir 製 CLI に置き換える

---

## 概要

`fav/self/cli.fav` を実装し、主要サブコマンドの dispatch / arg parsing / output formatting を
Favnir で担う。Rust の `main_impl()` は「引数を VM に渡して cli.fav を起動する」薄いラッパーとなる。

重い処理（型チェック本体・lineage 解析）は今バージョンでは引き続き Rust 側で実行し、
`Compiler.check_raw` / `Compiler.lineage_text_raw` の thin primitive で橋渡しする。
型チェッカーの完全 Favnir 化は v7.7.0〜v7.9.0 で段階的に行う。

```
fav check foo.fav
  └─ main.rs (thin dispatch)
       └─ CLI.run_check("foo.fav")          [cli.fav]
            └─ Compiler.check_raw("foo.fav") [VM primitive → Rust cmd_check]
```

---

## Phase A: VM primitives

### A-1〜A-2: IO プリミティブ

| 関数 | シグネチャ | 説明 |
|------|-----------|------|
| `IO.write_stderr_raw` | `String -> Unit !IO` | stderr に 1 行出力 |
| `IO.exit_raw` | `Int -> Unit !IO` | プロセス終了（exit code） |

### A-3〜A-4: Compiler プリミティブ

| 関数 | シグネチャ | 説明 |
|------|-----------|------|
| `Compiler.check_raw` | `String -> Result<String, String> !IO` | ファイルを型チェック、ok メッセージ or エラー文字列 |
| `Compiler.lineage_text_raw` | `String -> String !IO` | lineage 解析、フォーマット済みテキストを返す |

### checker.rs 追加

```rust
("IO",       "write_stderr_raw")    => Some(Type::Unit),
("IO",       "exit_raw")            => Some(Type::Unit),
("Compiler", "check_raw")           => Some(Type::Result(Box::new(Type::String), Box::new(Type::String))),
("Compiler", "lineage_text_raw")    => Some(Type::String),
```

---

## Phase B: fav/self/cli.fav

### 型定義

```favnir
type CliCmd =
  | CmdCheck(String)
  | CmdVersion
  | CmdHelp
  | CmdLineage(String)
  | CmdRuneList
  | CmdRuneInfo(String)
  | CmdUnknown(String)
```

### 関数一覧

| 関数 | 説明 |
|------|------|
| `fn find_flag(args, flag)` | `--flag` が args に含まれるか |
| `fn not_flag(s)` | `--` で始まらない文字列か |
| `fn find_positional(args)` | 最初の非フラグ引数を返す |
| `fn parse_cmd(args)` | `List<String> -> CliCmd` |
| `fn parse_named_cmd(cmd, args)` | コマンド名で分岐 |
| `fn parse_check_cmd(args)` | check サブコマンド解析 |
| `fn parse_explain_cmd(args)` | explain サブコマンド解析 |
| `fn parse_rune_cmd(args)` | rune サブコマンド解析 |
| `fn parse_rune_sub(sub, args)` | rune サブサブコマンド解析 |
| `fn run_version()` | "favnir X.Y.Z" 出力 |
| `fn run_help()` | ヘルプテキスト出力 |
| `fn run_check(path)` | `Compiler.check_raw` 呼び出し + 出力 |
| `fn run_lineage(path)` | `Compiler.lineage_text_raw` 呼び出し + 出力 |
| `fn run_rune_list()` | rune_modules/ 一覧出力 |
| `fn print_list(items)` | リスト再帰出力 |
| `fn run_rune_info(name)` | rune 情報出力 |
| `public fn main()` | argv 取得 → parse_cmd → dispatch |

### CLI 動作仕様

```
fav check <file>              → Compiler.check_raw(file) → "ok: ..." or stderr + exit 1
fav explain --lineage <file>  → Compiler.lineage_text_raw(file) → テキスト出力
fav rune list                 → rune_modules/ 配下のディレクトリ一覧
fav rune info <name>          → rune_modules/<name> または runes/<name> のパスと source 表示
fav version / --version       → "favnir 7.6.0 (self-host CLI)"
fav help / --help             → ヘルプテキスト
unknown                       → stderr + exit 1
```

---

## Phase C: 統合テスト（driver.rs）

### cli_self_host_tests（4 件）

| テスト名 | 内容 |
|---------|------|
| `cli_version_test` | version → 出力に "favnir" を含む |
| `cli_help_test` | help → 出力に "COMMANDS:" を含む |
| `cli_check_valid_test` | check fav/tmp/hello.fav → "ok:" で始まる |
| `cli_check_missing_test` | check nonexistent.fav → Err を返す（exit しない版で確認） |

---

## Phase D: ドキュメント

`site/content/docs/language/self-host-cli.mdx`
- セルフホスト CLI のアーキテクチャ概要
- cli.fav の構造・型定義
- 使用例（`fav run fav/self/cli.fav -- check foo.fav`）
- v7.7.0 以降との関係（checker.fav 完全化ロードマップ）

---

## 完了条件

- `fav check fav/self/cli.fav` — no errors
- `cargo test` — 1091+ tests passing（+4 新規）
- `fav run fav/self/cli.fav -- version` → "favnir 7.6.0 (self-host CLI)"
- `fav run fav/self/cli.fav -- check fav/tmp/hello.fav` → "ok:" で始まる出力

---

## 設計上の制約メモ

- **`bind inside closure 不可`**: `|a| eq_str(a, flag)` のように外側関数を呼ぶ形で capture を使う
- **`else if` 非対応**: `else { if ... }` と書く
- **`||` 演算子**: Favnir でサポート済み（`TkPipePipe`）
- **String 等値**: `a == flag` は VM の `Equal` opcode で動作する（`String.eq` builtin は不要）
- **`Compiler.check_raw` の Rust 実装**: `parse_and_check_source` を使い、エラーなし → `Ok("compiled")`, エラーあり → `Err(error_text)` を返す
- **`IO.exit_raw`**: `std::process::exit(code)` を呼ぶ。checker.rs では戻り値を `Type::Unit` とする（diverging だが Favnir は diverging type を持たない）
