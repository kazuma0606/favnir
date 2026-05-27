# Favnir v7.6.0 Plan — CLI 部分セルフホスト化

---

## Phase A: VM プリミティブ追加

### A-1: IO.write_stderr_raw (vm.rs)

`"IO.write_stderr_raw"` 分岐を `"IO.write_file_raw"` 付近に追加：

```rust
"IO.write_stderr_raw" => {
    let msg = vm_string(v, "IO.write_stderr_raw")?;
    eprintln!("{}", msg);
    Ok(VMValue::Unit)
}
```

### A-2: IO.exit_raw (vm.rs)

```rust
"IO.exit_raw" => {
    let code = match v {
        VMValue::Int(n) => *n as i32,
        _ => 1,
    };
    std::process::exit(code)
}
```

### A-3: Compiler.check_raw (vm.rs)

`cmd_check` ロジックを内部呼び出し。`parse_and_check_source` または
`parse_check_file` を使い、エラーなし → `ok_vm(VMValue::Str("compiled"))` を返す。

```rust
"Compiler.check_raw" => {
    let path = vm_string(v, "Compiler.check_raw")?;
    let src = match std::fs::read_to_string(&path) {
        Err(e) => return Ok(err_vm(VMValue::Str(format!("cannot read {}: {}", path, e)))),
        Ok(s) => s,
    };
    match crate::middle::checker::check_source(&src) {
        Ok(_)  => Ok(ok_vm(VMValue::Str("compiled".to_string()))),
        Err(e) => Ok(err_vm(VMValue::Str(e.to_string()))),
    }
}
```

（`check_source` の実際のシグネチャは vm.rs から呼び出せる形に合わせる）

### A-4: Compiler.lineage_text_raw (vm.rs)

```rust
"Compiler.lineage_text_raw" => {
    let path = vm_string(v, "Compiler.lineage_text_raw")?;
    let src = match std::fs::read_to_string(&path) {
        Err(e) => return Ok(VMValue::Str(format!("error: {}", e))),
        Ok(s) => s,
    };
    let report = crate::driver::lineage_text(&src);
    Ok(VMValue::Str(report))
}
```

### A-5〜A-8: checker.rs

`("IO", "file_stat_raw")` の後あたりに追加：

```rust
("IO",       "write_stderr_raw")   => Some(Type::Unit),
("IO",       "exit_raw")           => Some(Type::Unit),
("Compiler", "check_raw")          => Some(Type::Result(
    Box::new(Type::String), Box::new(Type::String))),
("Compiler", "lineage_text_raw")   => Some(Type::String),
```

---

## Phase B: fav/self/cli.fav

### B-1: CliCmd 型定義

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

### B-2: ユーティリティ関数

```favnir
// flag 名と一致するか（closure 内から呼ぶヘルパー）
fn eq_str(a: String, b: String) -> Bool { a == b }

// "--" で始まらない引数か
fn not_flag(s: String) -> Bool {
    if String.starts_with(s, "--") { false } else { true }
}

// args の中から flag を探す
fn find_flag(args: List<String>, flag: String) -> Bool {
    match List.find(args, |a| eq_str(a, flag)) {
        None    => false
        Some(_) => true
    }
}

// 最初の非フラグ引数を返す
fn find_positional(args: List<String>) -> Option<String> {
    List.find(args, |a| not_flag(a))
}
```

### B-3: 引数パーサー

```favnir
fn parse_cmd(args: List<String>) -> CliCmd {
    match List.first(args) {
        None      => CmdHelp
        Some(cmd) => parse_named_cmd(cmd, args)
    }
}

fn parse_named_cmd(cmd: String, args: List<String>) -> CliCmd {
    if cmd == "check"    { parse_check_cmd(args) }
    else { if cmd == "version"   { CmdVersion }
    else { if cmd == "--version" { CmdVersion }
    else { if cmd == "help"      { CmdHelp }
    else { if cmd == "--help"    { CmdHelp }
    else { if cmd == "explain"   { parse_explain_cmd(args) }
    else { if cmd == "rune"      { parse_rune_cmd(args) }
    else { CmdUnknown(String.concat("unknown command: ", cmd)) }
    } } } } } } }
}

fn parse_check_cmd(args: List<String>) -> CliCmd {
    match List.first(List.drop(args, 1)) {
        None    => CmdUnknown("check requires a file argument")
        Some(f) => CmdCheck(f)
    }
}

fn parse_explain_cmd(args: List<String>) -> CliCmd {
    bind rest        <- List.drop(args, 1)
    bind has_lineage <- find_flag(rest, "--lineage")
    if has_lineage {
        match find_positional(rest) {
            None    => CmdUnknown("explain --lineage requires a file")
            Some(f) => CmdLineage(f)
        }
    } else {
        CmdUnknown("explain without --lineage not yet supported in self-host CLI")
    }
}

fn parse_rune_cmd(args: List<String>) -> CliCmd {
    match List.first(List.drop(args, 1)) {
        None      => CmdRuneList
        Some(sub) => parse_rune_sub(sub, args)
    }
}

fn parse_rune_sub(sub: String, args: List<String>) -> CliCmd {
    if sub == "list" { CmdRuneList }
    else { if sub == "info" {
        match List.first(List.drop(args, 2)) {
            None       => CmdUnknown("rune info requires a name")
            Some(name) => CmdRuneInfo(name)
        }
    } else {
        CmdUnknown(String.concat("unknown rune subcommand: ", sub))
    } }
}
```

### B-4: アクションハンドラー

```favnir
fn run_version() -> Unit !IO {
    IO.println("favnir 7.6.0 (self-host CLI)")
}

fn run_help() -> Unit !IO {
    bind _ <- IO.println("fav - Favnir CLI (self-hosted)")
    bind _ <- IO.println("COMMANDS:")
    bind _ <- IO.println("  check <file>              Type-check a .fav file")
    bind _ <- IO.println("  explain --lineage <file>  Static lineage analysis")
    bind _ <- IO.println("  rune list                 List installed runes")
    bind _ <- IO.println("  rune info <name>          Show rune location")
    bind _ <- IO.println("  version                   Show version")
    IO.println("  help                      Show this help")
}

fn run_check(path: String) -> Unit !IO {
    match Compiler.check_raw(path) {
        Ok(msg) => IO.println(String.concat("ok: ", msg))
        Err(e)  => {
            bind _ <- IO.write_stderr_raw(String.concat("error: ", e))
            IO.exit_raw(1)
        }
    }
}

fn run_lineage(path: String) -> Unit !IO {
    bind text <- Compiler.lineage_text_raw(path)
    IO.println(text)
}

fn print_list(items: List<String>) -> Unit !IO {
    match List.first(items) {
        None       => IO.println("")
        Some(item) => {
            bind _ <- IO.println(String.concat("  ", item))
            print_list(List.drop(items, 1))
        }
    }
}

fn run_rune_list() -> Unit !IO {
    bind cwd      <- IO.cwd_raw()
    bind rune_dir <- IO.path_join_raw(cwd, "rune_modules")
    if IO.is_dir_raw(rune_dir) {
        match IO.list_dir_raw(rune_dir) {
            Err(_)    => IO.println("(no rune_modules directory)")
            Ok(names) => print_list(names)
        }
    } else {
        IO.println("(no rune_modules installed)")
    }
}

fn run_rune_info(name: String) -> Unit !IO {
    bind cwd     <- IO.cwd_raw()
    bind rm_dir  <- IO.path_join_raw(IO.path_join_raw(cwd, "rune_modules"), name)
    if IO.is_dir_raw(rm_dir) {
        bind _ <- IO.println(String.concat("rune:   ", name))
        bind _ <- IO.println(String.concat("path:   ", rm_dir))
        IO.println("source: rune_modules")
    } else {
        bind local_dir <- IO.path_join_raw(IO.path_join_raw(cwd, "runes"), name)
        if IO.is_dir_raw(local_dir) {
            bind _ <- IO.println(String.concat("rune:   ", name))
            bind _ <- IO.println(String.concat("path:   ", local_dir))
            IO.println("source: runes (local)")
        } else {
            IO.println(String.concat("rune not found: ", name))
        }
    }
}
```

### B-5: main エントリポイント

```favnir
public fn main() -> Unit !IO {
    bind args <- IO.argv()
    bind cmd  <- parse_cmd(args)
    match cmd {
        CmdCheck(path)    => run_check(path)
        CmdVersion        => run_version()
        CmdHelp           => run_help()
        CmdLineage(path)  => run_lineage(path)
        CmdRuneList       => run_rune_list()
        CmdRuneInfo(name) => run_rune_info(name)
        CmdUnknown(msg)   => {
            bind _ <- IO.write_stderr_raw(String.concat("error: ", msg))
            IO.exit_raw(1)
        }
    }
}
```

---

## Phase C: 統合テスト（driver.rs）

```rust
mod cli_self_host_tests {
    use super::*;

    fn run_cli(src: &str, argv_str: &str) -> String {
        // argv_str を空白分割して argv に変換、src を実行して stdout を返す
        let argv: Vec<&str> = argv_str.split_whitespace().collect();
        run_fav_source_with_argv(src, &argv)
    }

    #[test]
    fn cli_version_test() {
        let src = r##"/* cli.fav contents */"##;
        let out = run_cli(src, "version");
        assert!(out.contains("favnir"), "expected 'favnir' in output: {}", out);
    }

    #[test]
    fn cli_help_test() {
        let src = r##"/* cli.fav contents */"##;
        let out = run_cli(src, "help");
        assert!(out.contains("COMMANDS:"), "expected 'COMMANDS:' in output: {}", out);
    }

    #[test]
    fn cli_check_valid_test() {
        let src = r##"/* cli.fav contents */"##;
        let out = run_cli(src, "check fav/tmp/hello.fav");
        assert!(out.starts_with("ok:"), "expected 'ok:' prefix: {}", out);
    }

    #[test]
    fn cli_rune_list_test() {
        let src = r##"/* cli.fav contents */"##;
        // rune list は panic しないことを確認
        let _out = run_cli(src, "rune list");
    }
}
```

実際のテストは `run_fav_with_argv` パターンで実装（既存の bootstrap テストを参考）。

---

## Phase D: ドキュメント

`site/content/docs/language/self-host-cli.mdx` を新規作成。

---

## Phase E: 最終確認

1. `cargo test` — 1091+ tests passing
2. `fav check fav/self/cli.fav` — no errors
3. このファイルを完了状態に更新
4. commit
