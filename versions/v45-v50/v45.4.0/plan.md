# Plan: v45.4.0 — `match` 網羅性改善 + W034 / E0416

---

## Step 0 — 事前確認

```bash
cd /c/Users/yoshi/favnir/fav && cargo test -j 8 -- --test-threads=8 2>&1 | tail -5
```

期待: `test result: ok. 2974 passed; 0 failed`

---

## Step 1 — `error_catalog.rs`: E0416 エントリ追加

`// ── E0416〜E0419: 予約（将来拡張用）` コメントを以下に置換する:

```rust
ErrorEntry {
    code: "E0416",
    title: "non-exhaustive match in value context",
    category: "types",
    description: "A `match` expression used in value context does not cover all variants of the \
                  scrutinee's sum type. All variants must be handled when a `match` produces a value.",
    example: "type C = A | B\nlet x = match c { A -> 1 }  // E0416: B not covered",
    fix: "Add arms for missing variants, or add a wildcard `_ -> ...` arm.",
},
// ── E0417〜E0419: 予約（将来拡張用） ─────────────────────────────────────────
```

---

## Step 2 — `checker.rs`: `collect_covered_variants` フリー関数追加

`check_match_arms` 関数の直前に追加する。

```rust
/// Match アームのパターン群から「カバーされたバリアント名」の集合と catch-all フラグを返す。
/// ガード付きアームは網羅性に寄与しない。
fn collect_covered_variants(arms: &[MatchArm]) -> (Vec<String>, bool) {
    let mut covered: Vec<String> = vec![];
    let mut has_catch_all = false;
    for arm in arms {
        if arm.guard.is_some() {
            continue; // ガード付きは網羅性に寄与しない
        }
        collect_pattern_variants(&arm.pattern, &mut covered, &mut has_catch_all);
    }
    (covered, has_catch_all)
}

fn collect_pattern_variants(pat: &Pattern, covered: &mut Vec<String>, has_catch_all: &mut bool) {
    match pat {
        Pattern::Wildcard(_) | Pattern::Bind(_, _) => {
            *has_catch_all = true;
        }
        Pattern::Variant(name, _, _) => {
            covered.push(name.clone());
        }
        Pattern::Or(pats, _) => {
            for p in pats {
                collect_pattern_variants(p, covered, has_catch_all);
            }
        }
        _ => {} // Lit, Record, List — 網羅性チェックの対象外
    }
}
```

---

## Step 3 — `checker.rs`: `check_match_arms` 変更

### 3a. シグネチャに `value_ctx: bool` 追加

```rust
// Before:
fn check_match_arms(&mut self, arms: &[MatchArm], scrutinee_ty: &Type, span: &Span) -> Type {

// After:
fn check_match_arms(&mut self, arms: &[MatchArm], scrutinee_ty: &Type, span: &Span, value_ctx: bool) -> Type {
```

### 3b. 関数末尾に網羅性チェックを追加

`result_ty.unwrap_or(Type::Unit)` の直前に挿入:

```rust
// exhaustiveness check for Sum types (v45.4.0)
if let Type::Named(type_name, _) = scrutinee_ty {
    if let Some(TypeBody::Sum(variants)) = self.type_defs.get(type_name.as_str()) {
        let all_variants: Vec<String> =
            variants.iter().map(|v| v.name().to_string()).collect();
        let (covered, has_catch_all) = collect_covered_variants(arms);
        if !has_catch_all {
            let missing: Vec<&str> = all_variants
                .iter()
                .filter(|v| !covered.contains(v))
                .map(|s| s.as_str())
                .collect();
            if !missing.is_empty() {
                let msg = format!(
                    "non-exhaustive match: {} not covered",
                    missing.join(", ")
                );
                if value_ctx {
                    self.type_error("E0416", msg, span);
                } else {
                    self.type_warning("W034", msg, span);
                }
            }
        }
    }
}
```

### 3c. 既存の呼び出し元を `value_ctx: true` に更新

```rust
// check_expr 内の Expr::Match ハンドラ
Expr::Match(scrutinee, arms, span) => {
    let scrutinee_ty = self.check_expr(scrutinee);
    self.check_match_arms(arms, &scrutinee_ty, span, true)  // 値文脈
}
```

---

## Step 4 — `checker.rs`: `check_stmt` の Stmt::Expr を更新

文として使う match は stmt ctx で呼ぶ:

```rust
// Before:
Stmt::Expr(e) => {
    self.check_expr(e);
}

// After:
// 注意: Expr::Match の場合は check_expr(e) を経由しない。
// scrutinee のみ check_expr で評価し、check_match_arms を直接呼ぶ。
// check_expr(e) を先に呼ぶと check_expr 内の Expr::Match アーム（value_ctx: true）も
// 実行されて二重エラーが発生するため厳禁。
Stmt::Expr(e) => {
    if let Expr::Match(scrutinee, arms, span) = e {
        let scrutinee_ty = self.check_expr(scrutinee); // scrutinee のみ評価
        self.check_match_arms(arms, &scrutinee_ty, span, false); // 文文脈 → W034
    } else {
        self.check_expr(e);
    }
}
```

---

## Step 5 — `driver.rs`: テストモジュール追加 + バージョン更新

### 5a. Cargo.toml: バージョン更新

```toml
version = "45.4.0"
```

### 5b. `v454000_tests` モジュール追加

`v453000_tests` モジュールの直後に追加。

```rust
// -- v454000_tests (v45.4.0) -- match exhaustiveness: W034 / E0416 --
#[cfg(test)]
mod v454000_tests {
    use crate::frontend::parser::Parser;
    use crate::middle::checker::Checker;

    fn check_src(src: &str) -> (Vec<String>, Vec<String>) {
        let prog = Parser::parse_str(src, "test.fav").expect("parse failed");
        let (errors, warnings) = Checker::check_program(&prog);
        (
            errors.iter().map(|e| e.code.to_string()).collect(),
            warnings.iter().map(|w| w.code.to_string()).collect(),
        )
    }

    #[test]
    fn match_exhaustive_ok() {
        // All 3 variants covered — no error, no warning
        let src = r#"
type Color = Red | Green | Blue
fn label(c: Color) -> String {
  match c {
    Red   -> "red"
    Green -> "green"
    Blue  -> "blue"
  }
}
public fn main() -> Bool { true }
"#;
        let (errors, warnings) = check_src(src);
        assert!(errors.is_empty(), "unexpected errors: {:?}", errors);
        assert!(
            !warnings.iter().any(|w| w == "W034"),
            "unexpected W034: {:?}",
            warnings
        );
    }

    #[test]
    fn match_w034_missing_variant() {
        // Missing Blue, used as statement → W034
        let src = r#"
type Color = Red | Green | Blue
fn process(c: Color) -> Bool {
  match c {
    Red   -> { }
    Green -> { }
  };
  true
}
public fn main() -> Bool { true }
"#;
        let (errors, warnings) = check_src(src);
        assert!(errors.is_empty(), "unexpected errors: {:?}", errors);
        assert!(
            warnings.iter().any(|w| w == "W034"),
            "expected W034, got warnings: {:?}",
            warnings
        );
    }

    #[test]
    fn match_e0416_value_context() {
        // Missing Blue, used as value (let binding) → E0416
        let src = r#"
type Color = Red | Green | Blue
fn label(c: Color) -> String {
  let s = match c {
    Red   -> "red"
    Green -> "green"
  };
  s
}
public fn main() -> Bool { true }
"#;
        let (errors, _warnings) = check_src(src);
        assert!(
            errors.iter().any(|e| e == "E0416"),
            "expected E0416, got errors: {:?}",
            errors
        );
    }
}
```

---

## Step 6 — ビルド＆テスト

```bash
cd /c/Users/yoshi/favnir/fav && cargo test -j 8 -- --test-threads=8 2>&1 | tail -20
```

期待: `test result: ok. 2977 passed; 0 failed`

```bash
cargo clippy --locked -D warnings 2>&1 | grep -E "^error" | head -20
```

CHANGELOG.md に v45.4.0 エントリを追加する（match 網羅性 W034/E0416 実装）。
