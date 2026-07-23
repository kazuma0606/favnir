# Plan: v45.2.0 — `return` 型チェック + E0415

---

## Step 0 — 事前確認

```bash
cd /c/Users/yoshi/favnir/fav && cargo test -j 8 -- --test-threads=8 2>&1 | tail -5
```

期待: `test result: ok. 2968 passed; 0 failed`

---

## Step 1 — `error_catalog.rs`: E0415 追加

`fav/src/error_catalog.rs` を開き、`E0414〜E0419: 予約` コメントを確認する。
E0415 を `ReturnTypeMismatch` として正式登録し、予約コメントを更新する。

```rust
/// E0415: return type mismatch, or `return` used in seq body (v45.2.0)
pub const E0415: &str = "E0415";
```

エラーメッセージの構築:
- 型不一致: `"return type mismatch: expected {expected}, got {got}"`
- seq body: `"'return' is not allowed in 'seq' body"`

予約コメント行を更新（例: `// E0414〜E0419 予約` → `// E0415: ReturnTypeMismatch（v45.2.0）、E0414/E0416〜E0419: 予約`）。

---

## Step 2 — `checker.rs`: `Stmt::Return` stub → 実装

### 2a. `Checker` 構造体に `current_return_ty` フィールドを追加

`checker.rs` の `Checker` 構造体（`in_collect` 等のフラグと同じ場所）に追加:

```rust
current_return_ty: Option<Type>,   // None = seq body or top-level (return 禁止)
```

初期値は `None`。

### 2b. `fn` / `stage` / `seq` チェック時に戻り型をセット

- `check_fn_def`（`fn` 定義チェック関数）内:
  - 既存の `ret_resolved`（宣言戻り型）を `self.current_return_ty = Some(ret_resolved)` でセット
  - 関数ボディのチェック完了後に `self.current_return_ty = None` でリセット

- `check_trf_def`（`stage` 定義チェック関数）内:
  - 既存の `output_ty` を `self.current_return_ty = Some(output_ty)` でセット
  - チェック完了後にリセット

- `check_flw_def`（`seq` 定義チェック関数）内:
  - `self.current_return_ty = None` に明示的にセット（return 禁止コンテキスト）

### 2c. `check_return_stmt` ヘルパーを追加

既存の `type_error` / `self.errors.push` パターン（`?` 伝播ではなく collect 方式）で実装:

```rust
fn check_return_stmt(&mut self, ret: &ReturnStmt) {
    match self.current_return_ty.clone() {
        None => {
            self.type_error(E0415, "'return' is not allowed in 'seq' body", &ret.span);
        }
        Some(expected_ty) => {
            let actual_ty = self.infer_expr(&ret.expr);
            if !self.types_compatible(&actual_ty, &expected_ty) {
                self.type_error(
                    E0415,
                    &format!("return type mismatch: expected {:?}, got {:?}", expected_ty, actual_ty),
                    &ret.span,
                );
            }
        }
    }
}
```

`type_error` / `types_compatible` は checker.rs の既存メソッドを使用。

### 2d. `Stmt::Return` のアームを stub から実装に差し替え

```rust
// Before (stub from v45.1.0):
Stmt::Return(_) => { /* TODO v45.2 */ }

// After:
Stmt::Return(r) => { self.check_return_stmt(r); }
```

---

## Step 3 — `driver.rs`: テストモジュール追加 + バージョン更新

### 3a. Cargo.toml: バージョン更新

```toml
version = "45.2.0"
```

### 3b. `v452000_tests` モジュールを追加

`v451000_tests` モジュールの直前（または直後）に追加。
モジュール内にローカルヘルパー `check_src` を定義する
（既存の `v45xxx_tests` 内 checker テストのパターンを参照して合わせること）:

```rust
// -- v452000_tests (v45.2.0) -- return 型チェック + E0415 --
#[cfg(test)]
mod v452000_tests {
    use super::*;

    /// Runs the checker on src, returns collected error codes as a Vec<String>.
    /// Pattern: same as existing checker tests in this file.
    fn check_src(src: &str) -> Vec<String> {
        let tokens = crate::frontend::lexer::Lexer::new(src, "test.fav")
            .tokenize().expect("lex failed");
        let prog = crate::frontend::parser::Parser::new(tokens)
            .parse_program().expect("parse failed");
        let errors = crate::middle::checker::check_program(&prog);
        errors.iter().map(|e| format!("{:?}", e)).collect()
    }

    #[test]
    fn return_type_ok() {
        let src = "fn clamp(v: Int, lo: Int, hi: Int) -> Int {\n  if v < lo { return lo }\n  v\n}";
        let errs = check_src(src);
        assert!(errs.is_empty(), "correct return type should pass checker: {:?}", errs);
    }

    #[test]
    fn return_type_mismatch_e0415() {
        let src = "fn bad() -> Int { return \"hello\" }";
        let errs = check_src(src);
        assert!(!errs.is_empty(), "return type mismatch should produce errors");
        assert!(
            errs.iter().any(|e| e.contains("E0415") || e.contains("return type mismatch")),
            "expected E0415, got: {:?}", errs
        );
    }

    #[test]
    fn return_in_seq_e0415() {
        let src = "seq BadPipeline { stage A |> stage B }\n// return in seq body should be E0415";
        // Actual seq-with-return source depends on seq syntax; adapt to valid seq + embedded return
        let src2 = "seq S { bind x <- Ok(1); return x }";
        let errs = check_src(src2);
        assert!(
            errs.iter().any(|e| e.contains("E0415") || e.contains("not allowed")),
            "return in seq body should produce E0415: {:?}", errs
        );
    }
}
```

**注意**: `check_src` の実装は既存の checker テストヘルパーのパターンに合わせて調整する。
`check_program` の正確な関数名・シグネチャは checker.rs を参照すること。

---

## Step 4 — ビルド＆テスト

```bash
cd /c/Users/yoshi/favnir/fav && cargo test -j 8 -- --test-threads=8 2>&1 | tail -20
```

期待: `test result: ok. 2971 passed; 0 failed`

```bash
cargo clippy --locked -D warnings 2>&1 | grep -E "^error" | head -20
```

CHANGELOG.md に v45.2.0 エントリを追加する（`return` 型チェック + E0415）。
