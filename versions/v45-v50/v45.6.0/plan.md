# Plan: v45.6.0 — エラーメッセージ改善 Phase 1

Date: 2026-07-16

---

## 事前確認

1. `cargo test` 2980 passed, 0 failed を確認

---

## Step 1 — `error_catalog.rs`: `ErrorEntry` に `suggestion` フィールド追加

`ErrorEntry` struct に追加:

```rust
#[derive(serde::Serialize)]
pub struct ErrorEntry {
    pub code: &'static str,
    pub title: &'static str,
    pub category: &'static str,
    pub description: &'static str,
    pub example: &'static str,
    pub fix: &'static str,
    pub suggestion: Option<&'static str>,  // ← 追加
}
```

---

## Step 2 — `error_catalog.rs`: 全エントリに `suggestion: None,` を追加

`ErrorEntry` は `Default` 未実装のため、全エントリに明示的に追加が必要。
全 88 エントリの最後のフィールド（`fix: "..."` の行）の後に `suggestion: None,` を追加する。

具体的には各エントリ末尾の `fix: "...",` の直後、`},` の前に挿入:

```
    fix: "...",
    suggestion: None,    ← ここに追加
},
```

---

## Step 3 — `error_catalog.rs`: 主要エントリに suggestion テキストを設定

以下のエントリの `suggestion: None` を有意な値に変更:

```rust
// E0101
suggestion: Some("Check stage names for typos, or verify the return type matches the declared type."),

// E0102
suggestion: Some("Use `bind x <- expr` to introduce a variable, or check for typos in the name."),

// E0103
suggestion: Some("Add a transformation stage between them, or change one stage's type to match."),
```

---

## Step 4 — `checker.rs`: `Expr::Apply` 引数数不一致に hint 追加

`Expr::Apply` → `Type::Fn` 分岐の引数数チェック箇所（line ~4724）を修正。
現在:
```rust
self.type_error("E0101", format!("expected {} argument(s), got {}", ...), span);
Type::Error
```

変更後:
```rust
let hint = if let Expr::Ident(fn_name, _) = func.as_ref() {
    format!(
        "function `{}` expects {} argument(s), but {} were provided",
        fn_name, inst_params.len(), arg_tys.len()
    )
} else {
    format!(
        "this function expects {} argument(s), but {} were provided",
        inst_params.len(), arg_tys.len()
    )
};
self.type_error_h(
    "E0101",
    format!("expected {} argument(s), got {}", inst_params.len(), arg_tys.len()),
    span,
    vec![hint],
);
Type::Error
```

注意: `type_error_h` は `self.errors.push(...)` するだけで `Type` を返さない。元の `Type::Error` は変更後も維持すること。

---

## Step 5 — `driver.rs`: `v456000_tests` モジュール追加

```rust
// -- v456000_tests (v45.6.0) -- error message improvement: suggestions + hints --
#[cfg(test)]
mod v456000_tests {
    use crate::frontend::parser::Parser;
    use crate::middle::checker::Checker;

    fn check_with_hints(src: &str) -> Vec<(String, Vec<String>)> {
        let prog = Parser::parse_str(src, "test.fav").expect("parse failed");
        let (errors, _warnings) = Checker::check_program(&prog);
        errors.iter().map(|e| (e.code.to_string(), e.hints.clone())).collect()
    }

    #[test]
    fn e0102_suggestion_similar_name() {
        // `ordr` is a typo for `order` — E0102 hints should suggest `order`
        let src = r#"
fn order() -> Int {
    42
}
fn call_it() -> Int {
    ordr()
}
public fn main() -> Bool { true }
"#;
        let errors = check_with_hints(src);
        let e0102 = errors.iter().find(|(code, _)| code == "E0102");
        assert!(e0102.is_some(), "expected E0102, got: {:?}", errors);
        let hints = &e0102.unwrap().1;
        assert!(
            hints.iter().any(|h| h.contains("order")),
            "expected hint containing 'order', got: {:?}",
            hints
        );
    }

    #[test]
    fn e0101_suggestion_arg_count() {
        // `add(1)` has wrong arg count — E0101 hint should mention expected count
        let src = r#"
fn add(a: Int, b: Int) -> Int {
    a + b
}
fn call_it() -> Int {
    add(1)
}
public fn main() -> Bool { true }
"#;
        let errors = check_with_hints(src);
        let e0101 = errors.iter().find(|(code, _)| code == "E0101");
        assert!(e0101.is_some(), "expected E0101, got: {:?}", errors);
        let hints = &e0101.unwrap().1;
        assert!(
            hints.iter().any(|h| h.contains("2")),
            "expected hint mentioning '2' argument(s), got: {:?}",
            hints
        );
    }
}
```

---

## Step 6 — `Cargo.toml` + `CHANGELOG.md` + `versions/current.md` 更新

- `fav/Cargo.toml`: `"45.5.0"` → `"45.6.0"`
- `CHANGELOG.md`: v45.6.0 エントリ追加
- `versions/current.md`: 最新安定版を v45.6.0（2982 tests）に更新

---

## Step 7 — テスト実行

```bash
cd fav && cargo test -j 8 -- --test-threads=8
```

期待: 2982 passed, 0 failed。

```bash
cargo clippy -- -D warnings
```

クリーンであること。
