# Plan: v47.7.0 — `Result` 拡充

## 方針

全 5 関数は vm.rs・checker.rs に実装済み。
`driver.rs` にテスト 3 件を追加するのみ。

---

## 実装ステップ

### Step 1: `driver.rs` に `v477000_tests` 追加

挿入位置: `v476000_tests` モジュールの直前。

```rust
// -- v477000_tests (v47.7.0) -- Result 拡充: result_map / result_map_err / result_and_then 動作確認 --
#[cfg(test)]
mod v477000_tests {
    use crate::frontend::parser::Parser;
    use super::{build_artifact, exec_artifact_main};

    #[test]
    fn result_map() {
        // Result.map(ok(5), |n| n * 2) → unwrap_or → 10
        let src = r#"
fn main() -> Bool {
  bind r      <- Result.ok(5)
  bind result <- Result.map(r, |n| n * 2)
  Result.unwrap_or(result, 0) == 10
}
"#;
        let program = Parser::parse_str(src, "result_map_test.fav").expect("parse");
        let artifact = build_artifact(&program);
        let value = exec_artifact_main(&artifact, None).expect("exec");
        assert_eq!(value, crate::value::Value::Bool(true));
    }

    #[test]
    fn result_map_err() {
        // Result.map_err(err("oops"), |e| String.concat("wrapped: ", e)) → err payload == "wrapped: oops"
        // match でエラー文字列を直接検証（is_err のみでは変換内容を確認できないため）
        let src = r#"
fn main() -> Bool {
  bind r      <- Result.err("oops")
  bind result <- Result.map_err(r, |e| String.concat("wrapped: ", e))
  match result {
    err(e) => e == "wrapped: oops"
    ok(_)  => false
  }
}
"#;
        let program = Parser::parse_str(src, "result_map_err_test.fav").expect("parse");
        let artifact = build_artifact(&program);
        let value = exec_artifact_main(&artifact, None).expect("exec");
        assert_eq!(value, crate::value::Value::Bool(true));
    }

    #[test]
    fn result_and_then() {
        // Result.and_then(ok(5), |n| ok(n+1)) → unwrap_or → 6
        let src = r#"
fn main() -> Bool {
  bind r      <- Result.ok(5)
  bind result <- Result.and_then(r, |n| Result.ok(n + 1))
  Result.unwrap_or(result, 0) == 6
}
"#;
        let program = Parser::parse_str(src, "result_and_then_test.fav").expect("parse");
        let artifact = build_artifact(&program);
        let value = exec_artifact_main(&artifact, None).expect("exec");
        assert_eq!(value, crate::value::Value::Bool(true));
    }
}
```

### Step 2: `Cargo.toml` バージョン更新

```toml
version = "47.7.0"
```

### Step 3: `CHANGELOG.md` 更新

```markdown
## [v47.7.0] — 2026-07-17

### Added
- `driver.rs`: `v477000_tests` 追加（`result_map` / `result_map_err` / `result_and_then` 3テスト）
```

### Step 4: `versions/current.md` 更新

- 最新安定版を `v47.7.0`（3036 tests）に更新
- 進行中バージョン・次に切る版を `v47.8.0` に更新

---

## 注意事項

### Result の VM 表現

| 値 | VMValue |
|---|---|
| `Result.ok(x)` | `VMValue::Variant("ok".to_string(), Some(Box::new(x)))` |
| `Result.err(e)` | `VMValue::Variant("err".to_string(), Some(Box::new(e)))` |

### `result_map_err` テストの比較方針

`match result { err(e) => e == "wrapped: oops" | ok(_) => false }` で変換後の err payload を直接検証する。
`vm_stdlib_tests.rs` に `match` による err payload 取り出しの先例があり（`test_result_map_err`）、
`is_err` のみでは変換クロージャが正しく適用されたかを確認できないため、より強い検証を採用する。

### テスト数

| バージョン | テスト数 | 差分 |
|---|---|---|
| v47.6.0 | 3033 | ベース |
| v47.7.0 | 3036 | +3 |
