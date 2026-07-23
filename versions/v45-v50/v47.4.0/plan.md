# Plan: v47.4.0 — `String` 拡充

## 方針

全 5 関数は vm.rs・checker.rs に実装済み。
`driver.rs` にテスト 3 件を追加するのみ。

---

## 実装ステップ

### Step 1: `driver.rs` に `v474000_tests` 追加

挿入位置: `v473000_tests` モジュールの直後。

```rust
// -- v474000_tests (v47.4.0) -- String 拡充: pad_left / trim_start / repeat 動作確認 --
#[cfg(test)]
mod v474000_tests {
    use crate::frontend::parser::Parser;
    use super::{build_artifact, exec_artifact_main};

    #[test]
    fn string_pad_left() {
        // pad_left("42", 6, "0") = "000042"
        let src = r#"
fn main() -> Bool {
  bind result <- String.pad_left("42", 6, "0")
  result == "000042"
}
"#;
        let program = Parser::parse_str(src, "string_pad_left_test.fav").expect("parse");
        let artifact = build_artifact(&program);
        let value = exec_artifact_main(&artifact, None).expect("exec");
        assert_eq!(value, crate::value::Value::Bool(true));
    }

    #[test]
    fn string_trim_start() {
        // trim_start("  hello  ") = "hello  " (先頭空白のみ除去)
        let src = r#"
fn main() -> Bool {
  bind result <- String.trim_start("  hello  ")
  result == "hello  "
}
"#;
        let program = Parser::parse_str(src, "string_trim_start_test.fav").expect("parse");
        let artifact = build_artifact(&program);
        let value = exec_artifact_main(&artifact, None).expect("exec");
        assert_eq!(value, crate::value::Value::Bool(true));
    }

    #[test]
    fn string_repeat() {
        // repeat("ab", 3) = "ababab"
        let src = r#"
fn main() -> Bool {
  bind result <- String.repeat("ab", 3)
  result == "ababab"
}
"#;
        let program = Parser::parse_str(src, "string_repeat_test.fav").expect("parse");
        let artifact = build_artifact(&program);
        let value = exec_artifact_main(&artifact, None).expect("exec");
        assert_eq!(value, crate::value::Value::Bool(true));
    }
}
```

### Step 2: `Cargo.toml` バージョン更新

```toml
version = "47.4.0"
```

### Step 3: `CHANGELOG.md` 更新

```markdown
## [v47.4.0] — 2026-07-17  ← 実装時に実際の日付に置換すること

### Added
- `driver.rs`: `v474000_tests` 追加（`string_pad_left` / `string_trim_start` / `string_repeat` 3テスト）
```

### Step 4: `versions/current.md` 更新

- 最新安定版を `v47.4.0`（3027 tests）に更新
- 進行中バージョン・次に切る版を `v47.5.0` に更新

---

## 注意事項

### 引数順まとめ

| 関数 | 引数順 | vm.rs 行 |
|---|---|---|
| `String.pad_left` | `(str, width: Int, fill: String)` | 10672 |
| `String.pad_right` | `(str, width: Int, fill: String)` | 10703 |
| `String.repeat` | `(str, n: Int)` | 10883 |
| `String.trim_start` | `(str)` | 10927 |
| `String.trim_end` | `(str)` | 10935 |

### テスト数

| バージョン | テスト数 | 差分 |
|---|---|---|
| v47.3.0 | 3024 | ベース |
| v47.4.0 | 3027 | +3 |
