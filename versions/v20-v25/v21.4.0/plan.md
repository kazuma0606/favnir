# v21.4.0 Plan — `fav lint` 強化（W010〜W019）

## 前提確認

```bash
# 既存 lint.rs の末尾確認（追加箇所）
grep -n "check_ambient_effects\|check_deprecated_rune_calls\|^pub fn lint_program" fav/src/lint.rs | head -10
# 既存 cmd_explain_hint の最後の W コード確認
grep -n "\"W009\"\|\"W010\"" fav/src/driver.rs | head -5
# 現在バージョン確認
grep "^version" fav/Cargo.toml | head -1
```

---

## T1: `fav/src/lint.rs` — W010〜W019 実装

### 追加する関数（各ルール独立）

```rust
// W010: stage too large
fn check_w010_stage_too_large(program: &Program, errors: &mut Vec<LintError>)

// W011: effectless IO call
fn check_w011_effectless_io_call(program: &Program, errors: &mut Vec<LintError>)

// W012: unused type
fn check_w012_unused_type(program: &Program, errors: &mut Vec<LintError>)

// W013: List.map |> List.filter chain
fn check_w013_map_filter_chain(program: &Program, errors: &mut Vec<LintError>)
fn check_w013_expr(expr: &Expr, errors: &mut Vec<LintError>)

// W014: redundant Result.ok
fn check_w014_redundant_result_ok(program: &Program, errors: &mut Vec<LintError>)
fn check_w014_block(block: &Block, errors: &mut Vec<LintError>)

// W015: rebind in same block
fn check_w015_rebind_in_block(program: &Program, errors: &mut Vec<LintError>)
fn check_w015_block(block: &Block, errors: &mut Vec<LintError>)

// W016: wildcard-only match
fn check_w016_wildcard_only_match(program: &Program, errors: &mut Vec<LintError>)
fn check_w016_expr(expr: &Expr, errors: &mut Vec<LintError>)

// W017: deep nesting
fn check_w017_deep_nesting(program: &Program, errors: &mut Vec<LintError>)
fn nesting_depth(expr: &Expr) -> usize

// W018: magic number
fn check_w018_magic_number(program: &Program, errors: &mut Vec<LintError>)
fn check_w018_expr(expr: &Expr, errors: &mut Vec<LintError>)

// W019: String.concat chain
fn check_w019_string_concat_chain(program: &Program, errors: &mut Vec<LintError>)
fn check_w019_expr(expr: &Expr, errors: &mut Vec<LintError>)
```

### `lint_program` への統合

```rust
pub fn lint_program(program: &Program) -> Vec<LintError> {
    let mut errors = Vec::new();
    // ... 既存 L001-L008 ...
    // v21.4.0: W010-W019
    check_w010_stage_too_large(program, &mut errors);
    check_w011_effectless_io_call(program, &mut errors);
    check_w012_unused_type(program, &mut errors);
    check_w013_map_filter_chain(program, &mut errors);
    check_w014_redundant_result_ok(program, &mut errors);
    check_w015_rebind_in_block(program, &mut errors);
    check_w016_wildcard_only_match(program, &mut errors);
    check_w017_deep_nesting(program, &mut errors);
    check_w018_magic_number(program, &mut errors);
    check_w019_string_concat_chain(program, &mut errors);
    errors
}
```

### 実装詳細

**W010** (`stage_too_large`):
```rust
fn check_w010_stage_too_large(program: &Program, errors: &mut Vec<LintError>) {
    for item in &program.items {
        if let Item::TrfDef(td) = item {
            let n = td.body.stmts.len();
            if n > 30 {
                errors.push(LintError::new(
                    "W010",
                    format!("stage `{}` has {} statements (>30); consider splitting into smaller stages", td.name, n),
                    td.span.clone(),
                ));
            }
        }
    }
}
```

**W011** (`effectless_io_call`):
```rust
// W011_AMBIENT は新規追加（lint.rs に AMBIENT_NAMESPACES という既存定数はない）
const W011_AMBIENT: &[&str] = &[
    "IO", "Postgres", "AWS", "Snowflake", "Http", "Grpc",
    "Llm", "Queue", "Cache", "Slack", "Email",
];

fn check_w011_effectless_io_call(program: &Program, errors: &mut Vec<LintError>) {
    for item in &program.items {
        if let Item::TrfDef(td) = item {
            if td.effects.is_empty() {
                if let Some((ns, method, span)) = find_ambient_call_in_block(&td.body) {
                    errors.push(LintError::new(
                        "W011",
                        format!("stage `{}` calls `{}.{}` but declares no effects; add `!Io` or use ctx", td.name, ns, method),
                        span,
                    ));
                }
            }
        }
    }
}

fn find_ambient_call_in_block(block: &Block) -> Option<(String, String, Span)> { ... }
fn find_ambient_call_in_expr(expr: &Expr) -> Option<(String, String, Span)> { ... }
```

**W012** (`unused_type`):
```rust
fn check_w012_unused_type(program: &Program, errors: &mut Vec<LintError>) {
    // collect defined types
    let mut defined: HashMap<String, Span> = HashMap::new();
    for item in &program.items {
        if let Item::TypeDef(td) = item {
            if td.visibility.is_none() { // pub は除外
                defined.insert(td.name.clone(), td.span.clone());
            }
        }
    }
    // collect used type names from TypeExprs
    let mut used: HashSet<String> = HashSet::new();
    collect_used_type_names_program(program, &mut used);
    for (name, span) in &defined {
        if !used.contains(name) {
            errors.push(LintError::new(
                "W012",
                format!("type `{}` is defined but never used", name),
                span.clone(),
            ));
        }
    }
}
```

**W013** (`map_filter_chain`):
```rust
// Pipeline の連続ステップを検査
// is_list_map(step): Apply(FieldAccess(Ident("List"), "map"), ..) または FieldAccess(Ident("List"), "map") のどちらも検出
// is_list_filter(step): 同上（filter）
// 連続する steps[i] が is_list_map かつ steps[i+1] が is_list_filter → W013
```

**W014** (`redundant_result_ok`):
```rust
// Stmt::Bind(b) で b.pattern == Pattern::Bind(name, _)（Wildcard でない）かつ
// b.expr == Apply(FieldAccess(Ident("Result"), "ok"), [inner]) → W014
// 注意: BindStmt.name フィールドは存在しない。b.pattern: Pattern を使う
```

**W015** (`rebind_in_block`):
```rust
// block.stmts 内で Stmt::Bind(b) の b.pattern が Pattern::Bind(name, _) で同名が 2 回以上
// Pattern::Wildcard(_) はスキップ（name を &str で比較）
// 注意: BindStmt.name フィールドは存在しない。b.pattern: Pattern から取り出す
```

**W016** (`wildcard_only_match`):
```rust
// Match { arms: [MatchArm { pattern: Pattern::Wildcard(..), .. }] } の arms.len() == 1
```

**W017** (`deep_nesting`):
```rust
fn nesting_depth(expr: &Expr) -> usize {
    match expr {
        Expr::Match(_, arms, _) => 1 + arms.iter().map(|a| nesting_depth(&a.body)).max().unwrap_or(0),
        Expr::If(_, then, else_, _) => {
            let d = nesting_depth(then);
            let d2 = else_.as_ref().map(|e| nesting_depth(e)).unwrap_or(0);
            1 + d.max(d2)
        }
        // 他の式は子を再帰して max を取る
        _ => children_depth(expr),
    }
}
// depth > 4 なら W017
```

**W018** (`magic_number`):
```rust
// Expr::Lit(Lit::Int(n), span) で n.abs() > 100 → W018
// Expr::Lit(Lit::Float(f), span) で f.abs() > 100.0 → W018
// 注意: TypeExpr 内のリテラル（ConstInt）は除外。Expr コンテキストのみ。
```

**W019** (`string_concat_chain`):
```rust
// is_string_concat(expr): Apply(FieldAccess(Ident("String"), "concat"), ..)
// W019: is_string_concat(expr) かつ 引数に is_string_concat な expr がある
```

---

## T2: `fav/src/driver.rs` — `cmd_explain_hint` 更新

**事前確認**: `grep -n "cmd_explain_hint\|\"W009\"" fav/src/driver.rs | head -5`
- `cmd_explain_hint` が存在しない場合: 新規作成が必要（`pub fn cmd_explain_hint(code: &str)` として追加）
- `"W009"` が存在しない場合: W010〜W019 のエントリのみ追加すればよい

```rust
// 既存の "W009" ブロックの後に追加（W009 がない場合は新規 match ブロックへ）
"W010" => &["split the stage into smaller, focused stages"],
"W011" => &["add `!Io` (or appropriate effect) to the stage signature", "or pass the capability through a ctx argument"],
"W012" => &["remove the unused type definition, or make it `pub` if referenced externally"],
"W013" => &["replace `List.map(f) |> List.filter(g)` with `List.filter_map(|x| { ... })`"],
"W014" => &["remove the `Result.ok(...)` wrapper; bind directly from the inner expression"],
"W015" => &["rename the second binding, or use `bind _` to intentionally discard a value"],
"W016" => &["add specific match arms before `_ =>`; if a wildcard is intentional, suppress with `// lint:ignore W016`"],
"W017" => &["extract the inner match into a helper function to reduce nesting"],
"W018" => &["extract the magic number to a named constant: `let MAX_AMOUNT = 9999`"],
"W019" => &["use an f-string: `f\"{a}{b}{c}\"` instead of chaining String.concat"],
```

---

## T3: `fav/Cargo.toml` バージョン更新

```
version = "21.3.0"  →  version = "21.4.0"
```

`version_is_21_3_0` テストに `#[ignore]` を追加。

---

## T4: CHANGELOG + `site/content/docs/tools/lint.mdx` 更新

**CHANGELOG.md** 先頭に追加:
```markdown
## [21.4.0] — 2026-06-20

### Added
- `fav lint` W010〜W019: 10 の新しい静的解析ルール
  - W010: stage_too_large（30 stmt 超の stage を分割推奨）
  - W011: effectless_io_call（エフェクト宣言なしで ambient 呼び出し）
  - W012: unused_type（未使用 TypeDef）
  - W013: map_filter_chain（List.map |> List.filter → filter_map 推奨）
  - W014: redundant_result_ok（Result.ok ラップ不要）
  - W015: rebind_in_block（同一ブロック内での rebind）
  - W016: wildcard_only_match（match が _ => のみ）
  - W017: deep_nesting（4 レベル超ネスト）
  - W018: magic_number（100 超のリテラル）
  - W019: string_concat_chain（String.concat 連鎖 → f-string 推奨）
- `fav explain --lint W010` 〜 `--lint W019` でルール説明表示
```

**`site/content/docs/tools/lint.mdx`**:
- 既存 W001〜W009 に続いて W010〜W019 のセクションを追記
- 各ルールの説明・悪い例・良い例を記載
- `lint.mdx` が存在しない場合は新規作成

---

## T5: `fav/src/driver.rs` — `v214000_tests` 追加

`v213000_tests` の後に追加:

```rust
#[cfg(test)]
mod v214000_tests {
    use crate::lint::lint_program;
    use crate::frontend::parser::Parser;

    fn parse_lint(src: &str) -> Vec<String> {
        let prog = Parser::parse_str(src, "test.fav").expect("parse");
        lint_program(&prog).iter().map(|e| e.code.to_string()).collect()
    }

    #[test]
    fn version_is_21_4_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("\"21.4.0\""), "Cargo.toml version should be 21.4.0");
    }

    #[test]
    fn lint_w010_stage_too_large() {
        // 31 stmts: 30 bind + 1 final expr
        let stmts = (1..=30).map(|i| format!("bind x{i} <- {i}")).collect::<Vec<_>>().join("\n");
        let src = format!("stage Big: Int -> Int = |x| {{\n{stmts}\nx\n}}");
        let codes = parse_lint(&src);
        assert!(codes.contains(&"W010".to_string()), "expected W010, got: {:?}", codes);
    }

    #[test]
    fn lint_w011_effectless_io_call() {
        let src = "stage NoEff: String -> Unit = |s| IO.println(s)";
        let codes = parse_lint(src);
        assert!(codes.contains(&"W011".to_string()), "expected W011, got: {:?}", codes);
    }

    #[test]
    fn lint_w012_unused_type() {
        let src = "type Ghost = { name: String }";
        let codes = parse_lint(src);
        assert!(codes.contains(&"W012".to_string()), "expected W012, got: {:?}", codes);
    }

    #[test]
    fn lint_w013_map_filter_chain() {
        let src = "fn main() -> List<Int> = List.map(ns, |x| x) |> List.filter(|x| x > 0)";
        let codes = parse_lint(src);
        assert!(codes.contains(&"W013".to_string()), "expected W013, got: {:?}", codes);
    }

    #[test]
    fn lint_w014_redundant_result_ok() {
        let src = "fn main() -> Result<Int> = { bind x <- Result.ok(42)\nx }";
        let codes = parse_lint(src);
        assert!(codes.contains(&"W014".to_string()), "expected W014, got: {:?}", codes);
    }

    #[test]
    fn lint_w015_rebind_in_block() {
        let src = "fn main() -> Int = { bind x <- 1\nbind x <- 2\nx }";
        let codes = parse_lint(src);
        assert!(codes.contains(&"W015".to_string()), "expected W015, got: {:?}", codes);
    }

    #[test]
    fn lint_w016_wildcard_only_match() {
        let src = "fn main() -> Int = match 42 { _ => 0 }";
        let codes = parse_lint(src);
        assert!(codes.contains(&"W016".to_string()), "expected W016, got: {:?}", codes);
    }

    #[test]
    fn lint_w017_deep_nesting() {
        // 5重ネスト → W017 (depth > 4)
        let src = "fn main() -> Int = match 1 { _ => match 2 { _ => match 3 { _ => match 4 { _ => match 5 { _ => 0 } } } } }";
        let codes = parse_lint(src);
        assert!(codes.contains(&"W017".to_string()), "expected W017, got: {:?}", codes);
    }

    #[test]
    fn lint_w017_no_w017_at_4_levels() {
        // 4重ネストは W017 が出ない（> 4 = 5以上で発火）
        let src = "fn main() -> Int = match 1 { _ => match 2 { _ => match 3 { _ => match 4 { _ => 0 } } } }";
        let codes = parse_lint(src);
        assert!(!codes.contains(&"W017".to_string()), "W017 should not fire at depth 4, got: {:?}", codes);
    }

    #[test]
    fn lint_w018_magic_number() {
        let src = "fn main() -> Int = 9999";
        let codes = parse_lint(src);
        assert!(codes.contains(&"W018".to_string()), "expected W018, got: {:?}", codes);
    }

    #[test]
    fn lint_w019_string_concat_chain() {
        let src = "fn main() -> String = String.concat(String.concat(\"a\", \"b\"), \"c\")";
        let codes = parse_lint(src);
        assert!(codes.contains(&"W019".to_string()), "expected W019, got: {:?}", codes);
    }
}
```

---

## 実装順序

```
T1（lint.rs — W010〜W019 実装）    ← 最初、最大タスク
T2（driver.rs — explain_hint 追加） ← T1 完了後（lint_program が通る前提）
T3（Cargo.toml バージョン）         ← T1 と並列可
T4（CHANGELOG + MDX）               ← T3 完了後
T5（driver.rs テスト）              ← T1 完了後（lint_program を使用）
```

---

## リスクと対策

| リスク | 対策 |
|--------|------|
| W011 が W008/E0023 と誤検知の重複 | W011 は TrfDef 限定。FnDef は対象外。既存 E0023 と住み分け |
| W012 が型引数（`List<Ghost>`）を見落とす | `collect_used_type_names` は TypeExpr::Named の再帰で型引数も収集 |
| W015 が `bind _ <- ...` を誤検知 | 名前が `"_"` の場合はスキップ |
| W018 が型リテラル（const generics）を誤検知 | `check_w018_expr` は Expr コンテキストのみ。TypeExpr::ConstInt は Expr と別系統 |
| W017 の深さ計算が O(n²) で遅い | テストソースは小規模。実用上は問題なし |
| 既存 lint テスト（lint_tests）がリグレッション | W010〜W019 はすべて新規コードで既存ルールを変更しない |
