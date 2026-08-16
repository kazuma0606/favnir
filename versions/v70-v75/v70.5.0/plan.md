# v70.5.0 Plan — パターンマッチ強化

Date: 2026-08-09
Status: 計画中

---

## 実装ステップ（依存順）

### Step 1: compiler.fav の `parse_arm_guard` に `TkIf` 対応を追加

`fav/self/compiler.fav` の `parse_arm_guard` 関数（line 1391）は現状 `TkWhere` のみ処理。`TkIf` を追加する。

現在の実装:
```favnir
fn parse_arm_guard(toks: List<Token>) -> Result<ExprParse, String> {
    match peek(toks) {
        Some(TkWhere) => {
            bind rest1 <- advance(toks);
            match parse_expr(rest1) {
                Err(e) => Result.err(e)
                Ok(guard_p) => Result.ok(ExprParse { expr: guard_p.expr  rest: guard_p.rest })
            }
        }
        _ => Result.ok(ExprParse { expr: ELit(LBool(true))  rest: toks })
    }
}
```

修正後:
```favnir
fn parse_arm_guard(toks: List<Token>) -> Result<ExprParse, String> {
    match peek(toks) {
        Some(TkWhere) => {
            bind rest1 <- advance(toks);
            match parse_expr(rest1) {
                Err(e) => Result.err(e)
                Ok(guard_p) => Result.ok(ExprParse { expr: guard_p.expr  rest: guard_p.rest })
            }
        }
        Some(TkIf) => {
            bind rest1 <- advance(toks);
            match parse_expr(rest1) {
                Err(e) => Result.err(e)
                Ok(guard_p) => Result.ok(ExprParse { expr: guard_p.expr  rest: guard_p.rest })
            }
        }
        _ => Result.ok(ExprParse { expr: ELit(LBool(true))  rest: toks })
    }
}
```

確認: `TkIf` トークンが compiler.fav のトークン定義に存在することを確認（`TkIf` は lexer.fav またはトークン列挙体に定義済み）。

---

### Step 2: `v705000_tests` モジュールを driver.rs 末尾に追加

`v704000_tests` の直後（driver.rs 末尾）に追加する。

```rust
#[cfg(test)]
mod v705000_tests {
    use crate::frontend::parser::Parser;
    use crate::middle::checker::Checker;

    #[test]
    fn pattern_match_nested_record() {
        // Record 型フィールドを直接パターンにマッチする場合の全パイプライン確認
        let src = concat!(
            "type Response = { code: Int body: String }\n",
            "fn classify(r: Response) -> String {\n",
            "    match r {\n",
            "        { code: 200, body } => body\n",
            "        { code: 404, _ }    => \"not found\"\n",
            "        _                   => \"error\"\n",
            "    }\n",
            "}\n",
            "public fn main() -> Bool { true }\n",
        );
        let prog = Parser::parse_str(src, "test.fav").expect("parse should succeed");
        let (errors, _) = Checker::check_program(&prog);
        assert!(
            errors.is_empty(),
            "record field pattern with literal match should type-check; errors: {:?}",
            errors
        );
        let artifact = super::build_artifact(&prog);
        assert!(
            artifact.is_ok(),
            "record field pattern should compile to artifact; err: {:?}",
            artifact.err()
        );
    }

    #[test]
    fn pattern_match_or_pattern() {
        // Or-パターン（`A | B => body`）の全パイプライン確認
        let src = concat!(
            "fn classify_event(kind: String) -> String {\n",
            "    match kind {\n",
            "        \"created\" | \"updated\" => \"write\"\n",
            "        \"deleted\" | \"expired\" => \"delete\"\n",
            "        _                     => \"unknown\"\n",
            "    }\n",
            "}\n",
            "public fn main() -> Bool { true }\n",
        );
        let prog = Parser::parse_str(src, "test.fav").expect("parse should succeed");
        let (errors, _) = Checker::check_program(&prog);
        assert!(
            errors.is_empty(),
            "or-pattern should type-check cleanly; errors: {:?}",
            errors
        );
        let artifact = super::build_artifact(&prog);
        assert!(
            artifact.is_ok(),
            "or-pattern should compile to artifact; err: {:?}",
            artifact.err()
        );
    }
}
```

確認: `cargo test v705000` で 2 件 pass することを確認。

---

### Step 3: Cargo.toml バージョン更新

- `fav/Cargo.toml` の `version = "70.4.0"` → `"70.5.0"`
- driver.rs 内の `"70.4.0"` を `replace_all: true` で `"70.5.0"` に一括更新
  - 対象: `cargo_toml_version_is_70_4_0` テスト関数名と内部の `"70.4.0"` 文字列

---

### Step 4: CHANGELOG.md 更新

```markdown
## [v70.5.0] — 2026-08-09 — パターンマッチ強化

### Added
- `v705000_tests`: 2 件追加（3567 → 3569 tests）
  - `pattern_match_nested_record` — Record フィールドパターンの parse + typecheck + compile
  - `pattern_match_or_pattern` — Or-パターンの parse + typecheck + compile

### Fixed
- `compiler.fav` `parse_arm_guard`: `TkIf` ガード構文（`x if cond`）を追加（従来は `where` のみ対応）

### Verified
- Rust パイプライン（parser / compiler.rs / codegen.rs）における Or-パターン・ガード・Record パターンの E2E コンパイル動作を確認
```

---

### Step 5: 最終確認

- `cargo test v705000` で 2 件 pass
- `cargo test` 全体で 3569 tests pass（0 failures）
- `versions/current.md` を v70.5.0 進行中に更新
