# Plan: v46.3.0 — assertion 拡充

Date: 2026-07-17

---

## 実装手順

### Step 1 — `checker.rs`: `check_test_def` に assert_ok/assert_err を追加

**ファイル**: `fav/src/middle/checker.rs`

`check_test_def` 関数（line ≈ 2971）の `self.env.define("assert_ne"...)` の直後に追加:

```rust
self.env.define("assert_ok".to_string(), Type::Unknown);
self.env.define("assert_err".to_string(), Type::Unknown);
```

変更前:
```rust
fn check_test_def(&mut self, td: &TestDef) {
    self.env.push();
    self.env.define("assert".to_string(), Type::Fn(vec![Type::Bool], Box::new(Type::Unit)));
    self.env.define("assert_eq".to_string(), Type::Unknown);
    self.env.define("assert_ne".to_string(), Type::Unknown);
    self.check_block(&td.body);
    self.env.pop();
}
```

変更後:
```rust
fn check_test_def(&mut self, td: &TestDef) {
    self.env.push();
    self.env.define("assert".to_string(), Type::Fn(vec![Type::Bool], Box::new(Type::Unit)));
    self.env.define("assert_eq".to_string(), Type::Unknown);
    self.env.define("assert_ne".to_string(), Type::Unknown);
    // v46.3.0: assert_ok / assert_err を追加
    self.env.define("assert_ok".to_string(), Type::Unknown);
    self.env.define("assert_err".to_string(), Type::Unknown);
    self.check_block(&td.body);
    self.env.pop();
}
```

---

### Step 2 — `checker.rs`: `check_fn_def` に is_test ガードを追加

**ファイル**: `fav/src/middle/checker.rs`

`check_fn_def` 関数（line ≈ 3119）の `self.env.push();` 直後、
パラメータ型アリティ検証の前に追加:

```rust
// v46.3.0: #[test] fn の本体で assert 系 primitive を利用可能にする
if fd.is_test {
    self.env.define("assert".to_string(), Type::Fn(vec![Type::Bool], Box::new(Type::Unit)));
    self.env.define("assert_eq".to_string(), Type::Unknown);
    self.env.define("assert_ne".to_string(), Type::Unknown);
    self.env.define("assert_ok".to_string(), Type::Unknown);
    self.env.define("assert_err".to_string(), Type::Unknown);
}
```

挿入位置の確認（grep で確認）:
```
self.env.push();
// ↑ ここの直後に挿入
// Validate type arity (E023) for param and return type annotations.
for p in &fd.params {
```

---

### Step 3 — `driver.rs`: `v463000_tests` 追加

**ファイル**: `fav/src/driver.rs`

`v462000_tests` モジュール（line ≈ 46788 の `}` の後）の直後に追加:

```rust
// -- v463000_tests (v46.3.0) -- assertion 拡充: assert_ok / assert_err in #[test] fn --
#[cfg(test)]
mod v463000_tests {
    use crate::frontend::parser::Parser;
    use crate::backend::vm::VM;
    use crate::value::Value;

    #[test]
    fn assert_ok_passes() {
        let src = r#"
            #[test]
            fn test_ok() {
                assert_ok(Result.ok(42))
            }
        "#;
        let prog = Parser::parse_str(src, "test.fav").expect("parse failed");
        let artifact = super::build_artifact(&prog);
        let fn_idx = artifact
            .fn_idx_by_name("test_ok")
            .expect("test_ok should be in artifact");
        let result = VM::run(&artifact, fn_idx, vec![]).expect("assert_ok should pass");
        assert!(
            result != Value::Bool(false),
            "assert_ok should pass (got {:?})",
            result
        );
    }

    #[test]
    fn assert_err_passes() {
        let src = r#"
            #[test]
            fn test_err() {
                assert_err(Result.err("oops"))
            }
        "#;
        let prog = Parser::parse_str(src, "test.fav").expect("parse failed");
        let artifact = super::build_artifact(&prog);
        let fn_idx = artifact
            .fn_idx_by_name("test_err")
            .expect("test_err should be in artifact");
        let result = VM::run(&artifact, fn_idx, vec![]).expect("assert_err should pass");
        assert!(
            result != Value::Bool(false),
            "assert_err should pass (got {:?})",
            result
        );
    }
}
```

---

### Step 4 — バージョン・ドキュメント更新

1. `fav/Cargo.toml`: `version = "46.3.0"`
2. `CHANGELOG.md`: v46.3.0 エントリ追加
3. `versions/current.md`: v46.3.0（2999 tests）に更新、cargo install バージョンも更新

---

## 注意事項

- `build_artifact` はチェッカーを呼ばない（`compile_program` + `codegen_program` のみ）ため、
  Rust テスト（Step 3）は checker 修正（Step 1/2）がなくても動作する
- checker 修正（Step 1/2）は Favnir ユーザーが `fav check` を実行したときに
  `assert_ok`/`assert_err` で E0001 が出ないようにするための修正
- `Type::Unknown` は「型チェックをスキップ」の意味（他の assert_eq/assert_ne と同じ扱い）
