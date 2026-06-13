# v15.3.0 Plan — `fav test` DSL（ネイティブテストフレームワーク）

Date: 2026-06-14

---

## Phase A: Cargo バージョン更新

### A-1: `fav/Cargo.toml`

```toml
version = "15.3.0"
```

---

## Phase B: テスト追加（v153000_tests）

### B-1: `fav/src/driver.rs` — `v153000_tests` モジュール追加

```rust
#[cfg(test)]
mod v153000_tests {
    use std::fs;
    use std::path::Path;

    #[test]
    fn version_is_15_3_0() {
        assert_eq!(env!("CARGO_PKG_VERSION"), "15.3.0");
    }

    #[test]
    fn test_def_in_ast() {
        let ast = fs::read_to_string("src/ast.rs").unwrap();
        assert!(ast.contains("TestDef"), "ast.rs must contain TopLevel::TestDef");
    }

    #[test]
    fn assert_ok_primitive_exists() {
        let vm = fs::read_to_string("src/backend/vm.rs").unwrap();
        assert!(vm.contains("assert_ok"), "vm.rs must contain assert_ok primitive");
    }

    #[test]
    fn cmd_test_exists() {
        let driver = fs::read_to_string("src/driver.rs").unwrap();
        assert!(driver.contains("cmd_test"), "driver.rs must contain cmd_test function");
    }

    #[test]
    fn testing_doc_exists() {
        assert!(
            Path::new("../site/content/docs/language/testing.mdx").exists(),
            "site/content/docs/language/testing.mdx must exist"
        );
    }
}
```

---

## Phase C: AST — `TopLevel::TestDef` 追加

### C-1: `fav/src/ast.rs`

`TopLevel` enum に `TestDef` を追加:

```rust
pub enum TopLevel {
    // ... 既存 ...
    TestDef {
        name: String,
        body: Vec<Stmt>,
        span: Span,
    },
}
```

### C-2: `fav/src/ast.rs` — `Program` 構造体の型整理

`Program.tops: Vec<TopLevel>` に `TestDef` が含まれても既存コードが壊れないよう、
`is_test_def()` ヘルパーを追加してフィルタリングを容易にする（任意）。

---

## Phase D: パーサー — `test "..." { ... }` 構文

### D-1: `fav/src/frontend/parser.rs` — トークン `test` のハンドリング

`parse_top_level` に `test` キーワードのブランチを追加:

```rust
// lexer.rs に Token::Test を追加（または "test" を識別子として扱い、
// parse_top_level で文字列 "test" を検出する方式）
Token::Ident(s) if s == "test" => {
    self.advance();
    let name = self.expect_string_literal()?;  // "description"
    self.expect(Token::LBrace)?;
    let body = self.parse_stmts_until_rbrace()?;
    self.expect(Token::RBrace)?;
    Ok(TopLevel::TestDef { name, body, span })
}
```

`test` は新規キーワードとして追加するか、識別子の特殊処理として扱う。
既存コードへの影響を最小化するため、**識別子として検出する方式**を推奨。

### D-2: `assert_eq` / `assert_ok` / `assert_err` / `assert_true` の扱い

これらは `test` ブロック内で通常の関数呼び出しとして書ける
（特別なキーワード化は不要。`ECall("assert_eq", ...)` としてパース・コンパイルする）。

---

## Phase E: コンパイラ — `TestDef` のコンパイル

### E-1: `fav/src/ast.rs` — `Program` に `tests` フィールド追加

```rust
pub struct Program {
    pub tops: Vec<TopLevel>,
    // TopLevel::TestDef は以下に分離して収集することも可
}
```

テスト関数名の命名規則: `__test__<sanitized_name>`（スペースを `_` に変換）。

### E-2: `fav/src/middle/compiler.rs` — `compile_program` 更新

```rust
pub fn compile_program(program: &Program) -> IRProgram {
    // ... 既存の fn / stage / seq / interface / impl のコンパイル ...

    // TestDef は fns とは別に収集（fav run 時は無視するため）
    for top in &program.tops {
        if let TopLevel::TestDef { name, body, .. } = top {
            let fn_name = format!("__test__{}", name.replace(' ', "_"));
            // body を IRFnDef としてコンパイル
            // 引数なし・戻り値 Unit（失敗は panic を通じて VM エラーに変換）
            ir.test_fns.push(compile_test_fn(&fn_name, body, &mut ctx));
        }
    }
}
```

### E-3: `fav/src/middle/ir.rs` — `IRProgram` に `test_fns` フィールド追加

```rust
pub struct IRProgram {
    pub fns:       Vec<IRFnDef>,
    pub globals:   Vec<IRGlobal>,
    pub test_fns:  Vec<IRFnDef>,  // v15.3.0 追加
}
```

`fav run` のメイン実行パス（`exec_artifact_main`）は `test_fns` を無視する。

---

## Phase F: VM — アサーション primitive 拡張

### F-1: `fav/src/backend/vm.rs` — `assert_ok` / `assert_err` / `assert_true` primitive 追加

既存の `assert_eq` / `assert` / `assert_ne` に並べて追加:

```rust
"assert_ok" => {
    // Args: (r: Result<T, E>)
    // Result.ok(v) → v を返す
    // Result.err(e) → panic!("assert_ok failed: got err({e})")
    let mut it = args.into_iter();
    let r = it.next().ok_or("assert_ok: missing argument")?;
    match &r {
        VMValue::Variant(tag, inner) if tag == "ok" => {
            Ok(inner.as_deref().cloned().unwrap_or(VMValue::Unit))
        }
        VMValue::Variant(tag, inner) if tag == "err" => {
            let msg = inner.as_deref().map(|v| format!("{v}")).unwrap_or_default();
            Err(format!("assert_ok failed: got err({msg})"))
        }
        other => Err(format!("assert_ok failed: not a Result: {other:?}")),
    }
}

"assert_err" => {
    // Args: (r: Result<T, E>)
    // Result.err(e) → e を返す
    // Result.ok(v) → panic!("assert_err failed: got ok({v})")
    let mut it = args.into_iter();
    let r = it.next().ok_or("assert_err: missing argument")?;
    match &r {
        VMValue::Variant(tag, inner) if tag == "err" => {
            Ok(inner.as_deref().cloned().unwrap_or(VMValue::Unit))
        }
        VMValue::Variant(tag, inner) if tag == "ok" => {
            let msg = inner.as_deref().map(|v| format!("{v}")).unwrap_or_default();
            Err(format!("assert_err failed: got ok({msg})"))
        }
        other => Err(format!("assert_err failed: not a Result: {other:?}")),
    }
}

"assert_true" => {
    let mut it = args.into_iter();
    let b = it.next().ok_or("assert_true: missing argument")?;
    match b {
        VMValue::Bool(true) => Ok(VMValue::Unit),
        VMValue::Bool(false) => Err("assert_true failed".to_string()),
        other => Err(format!("assert_true failed: not a Bool: {other:?}")),
    }
}
```

### F-2: `assert_ok` / `assert_err` / `assert_true` を builtin namespace に登録

`compiler.rs` の builtin primitive リストに追加:

```rust
"assert_ok",
"assert_err",
"assert_true",
```

---

## Phase G: `cmd_test` 実装

### G-1: `fav/src/driver.rs` — `cmd_test` 関数

```rust
pub fn cmd_test(path: &str) {
    // 1. ファイルをパース
    let src = std::fs::read_to_string(path)
        .unwrap_or_else(|e| { eprintln!("error: {e}"); std::process::exit(1); });
    let program = Parser::parse_str(&src, path)
        .unwrap_or_else(|e| { eprintln!("error: {e}"); std::process::exit(1); });

    // 2. TestDef を収集
    let test_defs: Vec<_> = program.tops.iter()
        .filter_map(|t| if let TopLevel::TestDef { name, body, span } = t {
            Some((name, body, span))
        } else { None })
        .collect();

    if test_defs.is_empty() {
        println!("no tests found in {path}");
        return;
    }

    println!("\nrunning {} test{}", test_defs.len(),
        if test_defs.len() == 1 { "" } else { "s" });

    // 3. 各 TestDef を独立コンパイル・実行
    let mut pass = 0usize;
    let mut fail = 0usize;
    let mut failures: Vec<(String, String)> = Vec::new();

    for (name, body, _span) in &test_defs {
        // test 単体を fn __test__main() { <body> } としてコンパイル
        let test_src = build_test_wrapper(name, body, &src);
        let test_prog = match Parser::parse_str(&test_src, path) {
            Ok(p) => p,
            Err(e) => {
                println!("test {} ... FAILED (parse: {e})", name);
                fail += 1;
                failures.push((name.to_string(), format!("parse error: {e}")));
                continue;
            }
        };
        let ir = compile_program(&test_prog);
        let artifact = codegen_program(&ir);

        match exec_artifact_main_with_source(&artifact, &test_src, path) {
            Ok(_) => {
                println!("test {name} ... ok");
                pass += 1;
            }
            Err(e) => {
                println!("test {name} ... FAILED");
                fail += 1;
                failures.push((name.to_string(), e.to_string()));
            }
        }
    }

    // 4. サマリー出力
    if !failures.is_empty() {
        println!("\nfailures:");
        for (name, msg) in &failures {
            println!("  {name}: {msg}");
        }
    }
    println!("\ntest result: {}. {} passed; {} failed",
        if fail == 0 { "ok" } else { "FAILED" }, pass, fail);

    if fail > 0 {
        std::process::exit(1);
    }
}
```

**`build_test_wrapper`**: テスト本体を `public fn main(ctx: AppCtx) -> Result<Unit, String>` にラップし、
テスト対象関数の定義（`fn` / `stage` のみ、他の `test` ブロックは除く）を先頭に含める。

### G-2: `fav/src/driver.rs` — `match args[0]` に `"test"` ブランチ追加

```rust
"test" => {
    let path = args.get(1).map(|s| s.as_str()).unwrap_or_else(|| {
        eprintln!("error: fav test <file>");
        std::process::exit(1);
    });
    cmd_test(path);
}
```

---

## Phase H: checker.rs / ast_lower_checker.rs — TestDef のスキップ

### H-1: `fav/src/middle/checker.rs`

`check_with_self` で `TopLevel::TestDef` を通常の型チェックから除外:

```rust
for top in &program.tops {
    match top {
        TopLevel::TestDef { .. } => {
            // fav test 専用ブロック: fav run / fav check では型チェックをスキップ
            // （テストアサーション関数の戻り値型が未定義のためエラーになるのを防ぐ）
        }
        // ... 既存のハンドリング ...
    }
}
```

### H-2: `fav/src/middle/ast_lower_checker.rs`

`lower_program` で `TestDef` を無視（スキップ）:

```rust
TopLevel::TestDef { .. } => {
    // fav test 専用ブロック: AST lower では無視
}
```

---

## Phase I: `fav help` / `fav --help` 更新

### I-1: `fav/src/driver.rs` の `cmd_help` / `HELP_TEXT`

```
  test <file>          Run test blocks in a .fav file
```

---

## Phase J: サイトドキュメント

### J-1: `site/content/docs/language/testing.mdx`（新規作成）

```mdx
---
title: Testing
description: fav test DSL — test "..." blocks and assertion functions
---

# Testing in Favnir

## Writing Tests

```fav
test "transform trims whitespace" {
  bind result <- transform_row({ full_name: "  Alice  " })
  assert_eq(result.full_name, "Alice")
}
```

## Running Tests

```bash
fav test src/pipeline.fav
```

## Assertion Functions

| Function | Description |
|---|---|
| `assert_eq(a, b)` | Fail if a ≠ b |
| `assert_ok(r)` | Fail if r is err; returns unwrapped value |
| `assert_err(r)` | Fail if r is ok; returns unwrapped error |
| `assert_true(b)` | Fail if b is false |
```

---

## Phase K: コミット

```
feat: v15.3.0 — fav test DSL（ネイティブテストフレームワーク）

- ast.rs: TopLevel::TestDef 追加
- parser.rs: test "..." { } 構文追加
- compiler.rs / ir.rs: TestDef → IRProgram.test_fns コンパイル
- vm.rs: assert_ok / assert_err / assert_true primitive 追加
- driver.rs: cmd_test 実装（PASS/FAIL レポート）
- checker.rs: TestDef をスキップ
- site/content/docs/language/testing.mdx 新規作成
- v153000_tests: 5/5 pass
```

---

## 変更ファイル一覧

| ファイル | 変更内容 |
|---|---|
| `fav/Cargo.toml` | version → 15.3.0 |
| `fav/src/ast.rs` | `TopLevel::TestDef { name, body, span }` 追加 |
| `fav/src/frontend/parser.rs` | `test "..." { }` 構文パース追加 |
| `fav/src/middle/ir.rs` | `IRProgram.test_fns: Vec<IRFnDef>` フィールド追加 |
| `fav/src/middle/compiler.rs` | `TestDef` → `test_fns` コンパイル、`assert_ok` 等を builtin 登録 |
| `fav/src/middle/checker.rs` | `TestDef` スキップ |
| `fav/src/middle/ast_lower_checker.rs` | `TestDef` スキップ |
| `fav/src/backend/vm.rs` | `assert_ok` / `assert_err` / `assert_true` primitive 追加 |
| `fav/src/driver.rs` | `cmd_test` + `"test"` CLI ブランチ + `v153000_tests` |
| `site/content/docs/language/testing.mdx`（新規） | テスト DSL ドキュメント |

---

## 実装順序

```
A（バージョン）→ B（テスト記述）
→ C（AST: TopLevel::TestDef）
→ D（Parser: test "..." { }）
→ E（Compiler: TestDef コンパイル）
→ F（VM: assert_ok / assert_err / assert_true）
→ G（driver: cmd_test）
→ H（checker / ast_lower: TestDef スキップ）
→ I（CLI help 更新）
→ J（サイトドキュメント）
→ K（コミット）
```

コア実装は **C → D → E → F → G** の流れ。
H は `fav check` / `fav run` のリグレッションを防ぐために重要。

---

## 実装ノート

### `test` ブロックを `fav run` で無視する方法

最もシンプルな実装: `compile_program` で `TestDef` を `test_fns` に収集するが、
`build_artifact` / `exec_artifact_main` は `test_fns` を参照しない。
`fav run` 時は通常の `fns` のみからエントリポイント（`main`）を探す。

### `build_test_wrapper` の実装戦略

テスト本体に必要な `fn` / `stage` 定義を元ファイルから抽出し、
テスト本体を `public fn main(ctx: AppCtx) -> Result<Unit, String> { <body>; Result.ok(()) }` にラップして
独立した `.fav` 文字列として再パース・コンパイルする。

シンプルな代替案: `test_fns` を通常の `fns` と同じ IR に含めて `__test__<name>` として登録し、
`cmd_test` は `run_fn("__test__<name>", &artifact)` を呼ぶ方式。この方が再パース不要で安定。

### `assert_ok` の VM エラー伝播

VM の `call_builtin` が `Err(String)` を返すと `RuntimeError` として伝播する。
`cmd_test` はこの RuntimeError メッセージを FAIL の理由として表示する。
既存の `assert_eq` / `assert` / `assert_ne` も同様の仕組みで実装済み。
