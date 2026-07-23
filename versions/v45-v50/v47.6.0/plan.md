# Plan: v47.6.0 — `Option` 拡充

## 方針

全 5 関数は vm.rs・checker.rs に実装済み。
`driver.rs` にテスト 3 件を追加するのみ。

---

## 実装ステップ

### Step 1: `driver.rs` に `v476000_tests` 追加

挿入位置: `v475000_tests` モジュールの直前。

```rust
// -- v476000_tests (v47.6.0) -- Option 拡充: option_map / option_unwrap_or / option_and_then 動作確認 --
#[cfg(test)]
mod v476000_tests {
    use crate::frontend::parser::Parser;
    use super::{build_artifact, exec_artifact_main};

    #[test]
    fn option_map() {
        // Option.map(some(5), |n| n * 2) → unwrap_or → 10
        let src = r#"
fn main() -> Bool {
  bind opt    <- Option.some(5)
  bind result <- Option.map(opt, |n| n * 2)
  Option.unwrap_or(result, 0) == 10
}
"#;
        let program = Parser::parse_str(src, "option_map_test.fav").expect("parse");
        let artifact = build_artifact(&program);
        let value = exec_artifact_main(&artifact, None).expect("exec");
        assert_eq!(value, crate::value::Value::Bool(true));
    }

    #[test]
    fn option_unwrap_or() {
        // Option.unwrap_or(none(), "default") = "default"
        let src = r#"
fn main() -> Bool {
  bind opt    <- Option.none()
  bind result <- Option.unwrap_or(opt, "default")
  result == "default"
}
"#;
        let program = Parser::parse_str(src, "option_unwrap_or_test.fav").expect("parse");
        let artifact = build_artifact(&program);
        let value = exec_artifact_main(&artifact, None).expect("exec");
        assert_eq!(value, crate::value::Value::Bool(true));
    }

    #[test]
    fn option_and_then() {
        // Option.and_then(some(5), |n| some(n+1)) → unwrap_or → 6
        let src = r#"
fn main() -> Bool {
  bind opt    <- Option.some(5)
  bind result <- Option.and_then(opt, |n| Option.some(n + 1))
  Option.unwrap_or(result, 0) == 6
}
"#;
        let program = Parser::parse_str(src, "option_and_then_test.fav").expect("parse");
        let artifact = build_artifact(&program);
        let value = exec_artifact_main(&artifact, None).expect("exec");
        assert_eq!(value, crate::value::Value::Bool(true));
    }
}
```

### Step 2: `Cargo.toml` バージョン更新

```toml
version = "47.6.0"
```

### Step 3: `CHANGELOG.md` 更新

```markdown
## [v47.6.0] — 2026-07-17

### Added
- `driver.rs`: `v476000_tests` 追加（`option_map` / `option_unwrap_or` / `option_and_then` 3テスト）
```

### Step 4: `versions/current.md` 更新

- 最新安定版を `v47.6.0`（3033 tests）に更新
- 進行中バージョン・次に切る版を `v47.7.0` に更新

---

## 注意事項

### Option の VM 表現

| 値 | VMValue |
|---|---|
| `Option.some(x)` | `VMValue::Variant("some".to_string(), Some(Box::new(x)))` |
| `Option.none()` | `VMValue::Variant("none".to_string(), None)` |

`Option.map` の結果を `==` で直接比較する代わりに `Option.unwrap_or` で内部値を取り出して比較する。

### テスト数

| バージョン | テスト数 | 差分 |
|---|---|---|
| v47.5.0 | 3030 | ベース |
| v47.6.0 | 3033 | +3 |
