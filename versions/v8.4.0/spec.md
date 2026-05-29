# Favnir v8.4.0 Spec

Date: 2026-05-30
Theme: `fav run --self-host` の型チェックを checker.fav へ切替

---

## 背景

v8.3.0 で `fav run --self-host` が実現したが、型チェックステップは依然 Rust checker
（`load_and_check_program` → `Checker::check_program`）を使っている。

```
v8.3.0 時点の --self-host パス:
  parse:      Rust
  typecheck:  Rust  ← ここを Favnir へ
  compile:    Favnir (compiler.fav) ✓
  execute:    Rust VM
```

`fav check` はすでに checker.fav を使っているので（`check_single_file` →
`checker_fav_runner::run_checker_fav`）、同じパスを `--self-host` でも踏む。

---

## 目標

`fav run --self-host <file>` の型チェックを checker.fav 経由にし、
型チェック・コンパイルが共に Favnir 実装となる状態にする。

```
v8.4.0 の --self-host パス:
  parse:      Rust  (AST を checker.fav に渡すため必要)
  typecheck:  Favnir (checker.fav) ✓
  compile:    Favnir (compiler.fav) ✓
  execute:    Rust VM
```

---

## 設計

### 新関数: `check_file_with_fav(path: &str) -> Result<(), Vec<TypeError>>`

`driver.rs` に追加するヘルパー。`check_single_file` の内部ロジックを参考に、
パス解決・パース・lower・`run_checker_fav` を一連で行う。

```rust
fn check_file_with_fav(path: &str) -> Result<(), Vec<TypeError>> {
    let source = load_file(path);
    let program = Parser::parse_str(&source, path)?;
    let prog_vm = ast_lower_checker::lower_program(&program);
    checker_fav_runner::run_checker_fav(prog_vm)
        .map_err(|msgs| msgs_to_type_errors(msgs))
}
```

### `cmd_run_self_hosted` の変更点

```rust
// Before (v8.3.0):
let (_, source_path) = load_and_check_program(file);  // Rust checker

// After (v8.4.0):
let (source_path, _proj) = find_entry(file);           // path resolution のみ
check_file_with_fav(&source_path).unwrap_or_else(|errors| {
    for e in &errors { eprintln!("{}", format_diagnostic(..., e)); }
    process::exit(1);
});
```

---

## スコープ外（次バージョン以降）

- Rune import のマージ（`load_all_items`）: rune import を含むファイルの `--self-host` 実行
  は checker.fav の rune 解決が未実装のため対象外。通常の `fav run` を使うこと。
- `fav run`（フラグなし）の Favnir 化: v8.5.0 候補。
- checker.fav の infer_arg_tys 逆順バグ修正: v9.0.0 候補。
