# Plan: v47.8.0 — `Map` 拡充

## 方針

全 5 関数は vm.rs・checker.rs に実装済み。
`driver.rs` にテスト 3 件を追加するのみ。

---

## 実装ステップ

### Step 1: `driver.rs` に `v478000_tests` 追加

挿入位置: `v477000_tests` モジュールの直前。

```rust
// -- v478000_tests (v47.8.0) -- Map 拡充: map_merge / map_filter_values / map_map_values 動作確認 --
#[cfg(test)]
mod v478000_tests {
    use crate::frontend::parser::Parser;
    use super::{build_artifact, exec_artifact_main};

    #[test]
    fn map_merge() {
        // Map.merge({"key":1}, {"key":2}) → 右辺優先で value == 2
        let src = r#"
fn main() -> Bool {
  bind m1     <- Map.set((), "key", 1)
  bind m2     <- Map.set((), "key", 2)
  bind merged <- Map.merge(m1, m2)
  Option.unwrap_or(Map.get(merged, "key"), 0) == 2
}
"#;
        let program = Parser::parse_str(src, "map_merge_test.fav").expect("parse");
        let artifact = build_artifact(&program);
        let value = exec_artifact_main(&artifact, None).expect("exec");
        assert_eq!(value, crate::value::Value::Bool(true));
    }

    #[test]
    fn map_filter_values() {
        // Map.filter_values({"a":1,"b":2}, |v| v > 1) → size == 1
        let src = r#"
fn main() -> Bool {
  bind m        <- Map.set(Map.set((), "a", 1), "b", 2)
  bind filtered <- Map.filter_values(m, |v| v > 1)
  Map.size(filtered) == 1
}
"#;
        let program = Parser::parse_str(src, "map_filter_values_test.fav").expect("parse");
        let artifact = build_artifact(&program);
        let value = exec_artifact_main(&artifact, None).expect("exec");
        assert_eq!(value, crate::value::Value::Bool(true));
    }

    #[test]
    fn map_map_values() {
        // Map.map_values({"x":5}, |v| v * 2) → "x" の value == 10
        let src = r#"
fn main() -> Bool {
  bind m      <- Map.set((), "x", 5)
  bind mapped <- Map.map_values(m, |v| v * 2)
  Option.unwrap_or(Map.get(mapped, "x"), 0) == 10
}
"#;
        let program = Parser::parse_str(src, "map_map_values_test.fav").expect("parse");
        let artifact = build_artifact(&program);
        let value = exec_artifact_main(&artifact, None).expect("exec");
        assert_eq!(value, crate::value::Value::Bool(true));
    }
}
```

### Step 2: `Cargo.toml` バージョン更新

```toml
version = "47.8.0"
```

### Step 3: `CHANGELOG.md` 更新

```markdown
## [v47.8.0] — 2026-07-18

### Added
- `driver.rs`: `v478000_tests` 追加（`map_merge` / `map_filter_values` / `map_map_values` 3テスト）
```

### Step 4: `versions/current.md` 更新

- 最新安定版を `v47.8.0`（3039 tests）に更新
- 進行中バージョン・次に切る版を `v47.9.0` に更新

### Step 5: `roadmap-v47.1-v48.0.md` 確認

- v47.8.0 の完了条件テスト数（3039）を実績で確認・必要に応じて更新

---

## 注意事項

### Map の VM 表現

Map は `VMValue::Record(IndexMap<String, VMValue>)` として実装されている。
`Map.set((), "key", val)` では `()` が Unit（空 Record）として扱われる。

### `Map.get` の戻り型

`Map.get(map, key)` は `VMValue::Variant("some", Some(Box::new(v)))` または
`VMValue::Variant("none", None)` を返す。値を取り出すには `Option.unwrap_or` を使う。

### テスト数

| バージョン | テスト数 | 差分 |
|---|---|---|
| v47.7.0 | 3036 | ベース |
| v47.8.0 | 3039 | +3 |
