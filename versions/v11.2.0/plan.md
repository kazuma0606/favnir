# Favnir v11.2.0 実装計画

Date: 2026-06-06
Theme: stage / seq → Python パイプライン変換

---

## 変更対象ファイル

- `fav/src/emit_python.rs` — TrfDef / FlwDef 変換の追加
- `fav/Cargo.toml` — version `"11.2.0"`
- `fav/self/cli.fav` — version 文字列更新

---

## Phase A — `to_snake` ユーティリティ追加

`emit_python.rs` の `// ── Util ──` セクションに追加:

```rust
/// PascalCase / camelCase → snake_case
/// LoadAll → load_all, ValidateTxn → validate_txn
pub fn to_snake(name: &str) -> String {
    let mut out = String::new();
    let chars: Vec<char> = name.chars().collect();
    for (i, &ch) in chars.iter().enumerate() {
        if ch.is_uppercase() && i > 0 {
            // 前の文字が小文字、または次の文字が小文字のときに _ を挿入
            let prev_lower = chars[i - 1].is_lowercase() || chars[i - 1].is_ascii_digit();
            let next_lower = chars.get(i + 1).map(|c| c.is_lowercase()).unwrap_or(false);
            if prev_lower || next_lower {
                out.push('_');
            }
        }
        for c in ch.to_lowercase() {
            out.push(c);
        }
    }
    out
}
```

---

## Phase B — `TrfDef` 変換（stage）

`emit_program` の `TrfDef` アーム（現在は `TODO` コメント出力）を実装に切り替える。

`emit_python.rs` に `emit_trf_def` メソッドを追加:

```rust
impl Emitter {
    fn emit_trf_def(&mut self, td: &ast::TrfDef) {
        // エフェクトコメント
        if !td.effects.is_empty() {
            let eff_strs: Vec<&str> = td.effects.iter().map(|e| {
                if let Effect::Unknown(s) = e { return s.as_str(); }
                map_effect(e)
            }).collect();
            self.line(&format!("# effects: {}", eff_strs.join(", ")));
        }

        // stage の closure パラメータ名を取得
        // TrfDef.params は stage 本体の引数（通常1つ: stage Foo: A -> B = |param| { ... }）
        let param_name = td.params.first()
            .map(|p| p.name.as_str())
            .unwrap_or("x");
        let input_ty = map_type(&td.input_ty);
        let output_ty = map_type(&td.output_ty);
        let fn_name = to_snake(&td.name);

        self.line(&format!(
            "def {}({}: {}) -> {}:",
            fn_name, param_name, input_ty, output_ty
        ));
        self.indent += 1;
        self.emit_block_body(&td.body);
        self.indent -= 1;
        self.blank();
    }
}
```

`emit_program` の TrfDef アームを更新:

```rust
ast::Item::TrfDef(td) => self.emit_trf_def(td),
```

---

## Phase C — `FlwDef` 変換（seq）

`emit_python.rs` に `emit_flw_def` メソッドを追加:

```rust
impl Emitter {
    fn emit_flw_def(&mut self, fd: &ast::FlwDef) {
        let fn_name = to_snake(&fd.name);

        if fd.steps.is_empty() {
            self.line(&format!("def {}(x):", fn_name));
            self.indent += 1;
            self.line("return x");
            self.indent -= 1;
            self.blank();
            return;
        }

        // par ステップがあるか確認
        let has_par = fd.steps.iter().any(|s| matches!(s, ast::FlwStep::Par(_)));

        if has_par {
            self.emit_flw_with_par(fn_name, &fd.steps);
        } else {
            // 単純チェーン: def name(x): return c(b(a(x)))
            let chain = self.build_chain_expr("x", &fd.steps);
            self.line(&format!("def {}(x):", fn_name));
            self.indent += 1;
            self.line(&format!("return {}", chain));
            self.indent -= 1;
            self.blank();
        }
    }

    /// シンプルチェーン式を組み立てる: "x" → a(x) → b(a(x)) → c(b(a(x)))
    fn build_chain_expr(&self, input: &str, steps: &[ast::FlwStep]) -> String {
        let mut expr = input.to_string();
        for step in steps {
            match step {
                ast::FlwStep::Stage(name) => {
                    expr = format!("{}({})", to_snake(name), expr);
                }
                ast::FlwStep::Par(names) => {
                    // par は emit_flw_with_par で処理するためここには来ない
                    let calls: Vec<String> = names.iter()
                        .map(|n| format!("{}({})", to_snake(n), expr))
                        .collect();
                    expr = format!("[{}]", calls.join(", "));
                }
            }
        }
        expr
    }

    /// par ステップを含む seq の変換
    fn emit_flw_with_par(&mut self, fn_name: String, steps: &[ast::FlwStep]) {
        self.line(&format!("def {}(x):", fn_name));
        self.indent += 1;

        let mut cur = "x".to_string();
        let mut step_var_counter = 0usize;

        for (i, step) in steps.iter().enumerate() {
            match step {
                ast::FlwStep::Stage(name) => {
                    let next_var = if i == steps.len() - 1 {
                        // 最後のステップは return で出力
                        self.line(&format!("return {}({})", to_snake(name), cur));
                        break;
                    } else {
                        let v = format!("_step{}", step_var_counter);
                        step_var_counter += 1;
                        self.line(&format!("{} = {}({})", v, to_snake(name), cur));
                        v
                    };
                    cur = next_var;
                }
                ast::FlwStep::Par(names) => {
                    self.line("from concurrent.futures import ThreadPoolExecutor");
                    self.line(&format!("with ThreadPoolExecutor() as _pool:"));
                    self.indent += 1;
                    let submits: Vec<String> = names.iter()
                        .map(|n| format!("_pool.submit({}, {})", to_snake(n), cur))
                        .collect();
                    self.line(&format!("_futures = [{}]", submits.join(", ")));
                    self.line("_par_results = [_f.result() for _f in _futures]");
                    self.indent -= 1;
                    cur = "_par_results".to_string();
                }
            }
        }

        self.indent -= 1;
        self.blank();
    }
}
```

`emit_program` の FlwDef アームを更新:

```rust
ast::Item::FlwDef(fd) => self.emit_flw_def(fd),
```

---

## Phase D — `IO.argv()` の正式変換 + `fn main()` ガード

### D-1: `IO.argv()` の変換

`emit_apply` の IO セクションに `argv` を追加:

```rust
("IO", "argv") if a.is_empty() => return "sys.argv[1:]".to_string(),
("IO", "argv_all") if a.is_empty() => return "sys.argv".to_string(),
// 既存の IO.println → print() も維持
("IO", "println") if a.len() == 1 => return format!("print({})", a[0]),
```

### D-2: `fn main()` ガードの生成

`emit_program` で `fn main` を検出したフラグを立て、末尾に追加:

```rust
fn emit_program(&mut self, prog: &Program, source_path: &str) -> String {
    self.emit_prelude(source_path);
    let mut has_main = false;
    for item in &prog.items {
        match item {
            ast::Item::TypeDef(td) => self.emit_type_def(td),
            ast::Item::FnDef(fd) => {
                if fd.name == "main" { has_main = true; }
                self.emit_fn_def(fd);
            }
            ast::Item::TrfDef(td) => self.emit_trf_def(td),
            ast::Item::FlwDef(fd) => self.emit_flw_def(fd),
            _ => {}
        }
    }
    if has_main {
        self.line("if __name__ == \"__main__\":");
        self.indent += 1;
        self.line("main()");
        self.indent -= 1;
        self.blank();
    }
    self.buf.clone()
}
```

---

## Phase E — テスト（v11200_tests）

`emit_python.rs` に `v11200_tests` モジュールを追加:

```rust
#[cfg(test)]
mod v11200_tests {
    use super::*;

    #[test]
    fn transpile_stage_basic() {
        let src = r#"stage Foo: Int -> Int = |x| { x }"#;
        let out = emit_python_str(src);
        assert!(out.contains("def foo(x: int) -> int:"), "stage def:\n{}", out);
        assert!(out.contains("return x"), "stage body:\n{}", out);
    }

    #[test]
    fn transpile_stage_effects_comment() {
        let src = r#"stage Bar: String -> String !IO = |s| { s }"#;
        let out = emit_python_str(src);
        assert!(out.contains("# effects: IO"), "effect comment:\n{}", out);
    }

    #[test]
    fn transpile_stage_multiline_body() {
        let src = r#"
stage Validate: List<Int> -> List<Int> !IO = |rows| {
  bind valid <- List.filter(rows, |x| x > 0)
  bind _ <- IO.println("done")
  valid
}"#;
        let out = emit_python_str(src);
        assert!(out.contains("def validate(rows: List[int]) -> List[int]:"), "sig:\n{}", out);
        assert!(out.contains("valid = [_x for _x in rows"), "filter:\n{}", out);
        assert!(out.contains("print("), "println→print:\n{}", out);
        assert!(out.contains("return valid"), "return:\n{}", out);
    }

    #[test]
    fn transpile_seq_two_stages() {
        let src = r#"
stage Load: Int -> String = |x| { Int.to_string(x) }
stage Upper: String -> String = |s| { s }
seq Pipe = Load |> Upper
"#;
        let out = emit_python_str(src);
        assert!(out.contains("def pipe(x):"), "seq def:\n{}", out);
        assert!(out.contains("upper(load(x))"), "chain:\n{}", out);
    }

    #[test]
    fn transpile_seq_three_stages() {
        let src = r#"
stage A: Int -> Int = |x| { x }
stage B: Int -> Int = |x| { x }
stage C: Int -> Int = |x| { x }
seq Pipeline = A |> B |> C
"#;
        let out = emit_python_str(src);
        assert!(out.contains("def pipeline(x):"), "def:\n{}", out);
        assert!(out.contains("c(b(a(x)))"), "3-chain:\n{}", out);
    }

    #[test]
    fn transpile_seq_snake_case() {
        let src = r#"
stage LoadAll: Int -> Int = |x| { x }
stage WriteOutput: Int -> Int = |x| { x }
seq AnalyzePipeline = LoadAll |> WriteOutput
"#;
        let out = emit_python_str(src);
        assert!(out.contains("def load_all("), "load_all:\n{}", out);
        assert!(out.contains("def write_output("), "write_output:\n{}", out);
        assert!(out.contains("def analyze_pipeline(x):"), "analyze_pipeline:\n{}", out);
        assert!(out.contains("write_output(load_all(x))"), "chain:\n{}", out);
    }

    #[test]
    fn transpile_main_guard() {
        let src = r#"fn main() -> Unit !IO { IO.println("hi") }"#;
        let out = emit_python_str(src);
        assert!(out.contains("def main()"), "main def:\n{}", out);
        assert!(
            out.contains("if __name__ == \"__main__\":"),
            "__main__ guard:\n{}",
            out
        );
        assert!(out.contains("    main()"), "main() call:\n{}", out);
    }

    #[test]
    fn transpile_io_argv() {
        let src = r#"fn f() -> List<String> !IO { IO.argv() }"#;
        let out = emit_python_str(src);
        assert!(out.contains("sys.argv[1:]"), "argv:\n{}", out);
    }
}
```

---

## Phase F — バージョン更新

```toml
# fav/Cargo.toml
version = "11.2.0"
```

```fav
# fav/self/cli.fav
IO.println("favnir 11.2.0 (self-host CLI)")
```

---

## 実装順序

1. `to_snake` 関数追加
2. `emit_trf_def` 追加 + `emit_program` の TrfDef アーム更新
3. `emit_flw_def` + `build_chain_expr` + `emit_flw_with_par` 追加 + FlwDef アーム更新
4. `IO.argv()` 正式変換を `emit_apply` に追加
5. `emit_program` に `has_main` フラグ + `__main__` ガード追加
6. `v11200_tests` テスト 8 件追加
7. `cargo test v11200 --lib` 全件通過確認
8. `cargo test --lib` 全件通過確認（691 件）
9. Cargo.toml / cli.fav バージョン更新
10. コミット
