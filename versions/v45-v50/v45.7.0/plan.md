# Plan: v45.7.0 — エラーメッセージ改善 Phase 2 + 数値リテラル `_`

Date: 2026-07-16
Status: TODO

---

## ステップ

### Step 1 — 事前確認

```bash
cd /c/Users/yoshi/favnir/fav && cargo test -j 8 -- --test-threads=8 2>&1 | tail -5
```

2982 tests passed, 0 failed を確認。

---

### Step 2 — `error_catalog.rs`: E0201〜E0413 suggestion 追加

`fav/src/error_catalog.rs` を開き、`suggestion: None` になっているエントリを更新する。

変更対象コード（実在するエントリのみ）:
```
E0213, E0219, E0220, E0221, E0222, E0223, E0224, E0225, E0226, E0227,
E0241, E0242, E0243, E0244, E0245,
E0251, E0253, E0254,
E0274,
E0310, E0311, E0312, E0313, E0314, E0315,
E0319, E0320, E0321, E0322, E0323, E0324,
E0365, E0368, E0369, E0373, E0374,
E0380, E0381, E0382, E0383, E0384,
E0401, E0402, E0403, E0404, E0405, E0406,
E0410, E0411, E0412, E0413
```

**注意**: E0230 と E0414 は error_catalog.rs に実エントリが存在しないため変更しない。

各エントリの `suggestion: None` を `suggestion: Some("...")` に変更する。
spec.md §1 の提案テキスト一覧を参照（エラーの意味に合わせたテキストを設定）。

---

### Step 3 — `lexer.rs`: 数値リテラル `_` サポート

`fav/src/frontend/lexer.rs` の `lex_number` 関数（line ~573〜621）を修正。

**整数部スキャンループ**（整数リテラル・小数点前）:
```rust
// 変更前
while self.peek().map(|c| c.is_ascii_digit()).unwrap_or(false) {
    s.push(self.advance());
}

// 変更後
while self.peek().map(|c| c.is_ascii_digit() || c == '_').unwrap_or(false) {
    let ch = self.advance();
    if ch != '_' { s.push(ch); }
}
```

**小数部スキャンループ**（`.` 後）: 同様に修正。

指数部スキャンは `lex_number` に存在しないため修正対象外。

変更後、`s` は `_` を含まない純粋な数字文字列となり、
既存の `s.parse::<i64>()` / `s.parse::<f64>()` がそのまま動作する。

---

### Step 4 — `driver.rs`: v457000_tests 追加

`v456000_tests` モジュールの直後に `v457000_tests` モジュールを追加（3 件）:

```rust
#[cfg(test)]
mod v457000_tests {
    use super::*;
    use crate::frontend::parser::Parser;
    use crate::middle::checker::Checker;
    use crate::backend::vm::VM;
    use crate::middle::compiler::compile_program;
    use crate::backend::codegen::codegen_program;
    use crate::value::Value;

    fn run_inline(src: &str) -> Value {
        let prog = Parser::parse_str(src, "test.fav").expect("parse failed");
        let (errors, _) = Checker::check_program(&prog);
        assert!(errors.is_empty(), "type errors: {:?}", errors.iter().map(|e| &e.message).collect::<Vec<_>>());
        let ir = compile_program(&prog);
        let artifact = codegen_program(&ir);
        let fn_idx = artifact.fn_idx_by_name("main").expect("main not found");
        VM::run(&artifact, fn_idx, vec![]).expect("run failed")
    }

    #[test]
    fn e0410_suggestion() { ... }

    #[test]
    fn numeric_literal_underscore_int() { ... }

    #[test]
    fn numeric_literal_underscore_float() { ... }
}
```

テスト内容は spec.md §3 を参照。

---

### Step 5 — テスト実行

```bash
cd /c/Users/yoshi/favnir/fav && cargo test -j 8 -- --test-threads=8 2>&1 | tail -10
```

2985 tests passed, 0 failed を確認。

---

### Step 6 — Clippy

```bash
cd /c/Users/yoshi/favnir/fav && cargo clippy -- -D warnings 2>&1 | tail -10
```

警告なしを確認。

---

### Step 7 — バージョン更新・ドキュメント

1. `fav/Cargo.toml`: `version = "45.7.0"`
2. `CHANGELOG.md`: v45.7.0 エントリ追加
3. `versions/current.md`: v45.7.0（2985 tests）に更新
4. `versions/v45-v50/v45.7.0/tasks.md`: COMPLETE に更新

---

## 実装順序まとめ

```
Step 1: cargo test（事前確認: 2982 tests）
Step 2: error_catalog.rs — suggestion テキスト追加（E0201〜E0413 実在エントリ）
Step 3: lexer.rs — lex_number _ サポート（整数部・小数部のみ）
Step 4: driver.rs — v457000_tests 追加（3 件）
Step 5: cargo test（全通過確認: 2985 tests）
Step 6: cargo clippy
Step 7: バージョン・ドキュメント更新
```
