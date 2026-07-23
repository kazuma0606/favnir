# Plan: v47.3.0 — `List.scan` / `List.take_while` / `List.drop_while`

## 方針

全 3 関数は vm.rs・checker.rs に実装済み。
`driver.rs` にテスト 3 件を追加するのみ。

---

## 実装ステップ

### Step 1: `driver.rs` に `v473000_tests` 追加

挿入位置: `v472000_tests` モジュールの直後。

```rust
// -- v473000_tests (v47.3.0) -- List.scan / List.take_while / List.drop_while 動作確認 --
#[cfg(test)]
mod v473000_tests {
    use crate::frontend::parser::Parser;
    use super::{build_artifact, exec_artifact_main};

    #[test]
    fn list_scan_cumulative() {
        // scan([1,2,3], 0, |acc,x| acc+x) = [0,1,3,6] — init値含む length 4
        let src = r#"
fn main() -> Bool {
  bind xs     <- List.range(1, 4)
  bind totals <- List.scan(xs, 0, |acc, x| acc + x)
  List.length(totals) == 4
}
"#;
        let program = Parser::parse_str(src, "list_scan_test.fav").expect("parse");
        let artifact = build_artifact(&program);
        let value = exec_artifact_main(&artifact, None).expect("exec");
        assert_eq!(value, crate::value::Value::Bool(true));
    }

    #[test]
    fn list_take_while() {
        // take_while([1,2,3,4,5], x<3) = [1,2] — length 2
        let src = r#"
fn main() -> Bool {
  bind xs     <- List.range(1, 6)
  bind result <- List.take_while(xs, |x| x < 3)
  List.length(result) == 2
}
"#;
        let program = Parser::parse_str(src, "list_take_while_test.fav").expect("parse");
        let artifact = build_artifact(&program);
        let value = exec_artifact_main(&artifact, None).expect("exec");
        assert_eq!(value, crate::value::Value::Bool(true));
    }

    #[test]
    fn list_drop_while() {
        // drop_while([1,2,3,4,5], x<3) = [3,4,5] — length 3
        let src = r#"
fn main() -> Bool {
  bind xs     <- List.range(1, 6)
  bind result <- List.drop_while(xs, |x| x < 3)
  List.length(result) == 3
}
"#;
        let program = Parser::parse_str(src, "list_drop_while_test.fav").expect("parse");
        let artifact = build_artifact(&program);
        let value = exec_artifact_main(&artifact, None).expect("exec");
        assert_eq!(value, crate::value::Value::Bool(true));
    }
}
```

### Step 2: `Cargo.toml` バージョン更新

```toml
version = "47.3.0"
```

### Step 3: `CHANGELOG.md` 更新

```markdown
## [v47.3.0] — 2026-07-17

### Added
- `driver.rs`: `v473000_tests` 追加（`list_scan_cumulative` / `list_take_while` / `list_drop_while` 3テスト）
```

### Step 4: `versions/current.md` 更新

- 最新安定版を `v47.3.0`（3024 tests）に更新
- 進行中バージョンを `v47.4.0` に更新

---

## 注意事項

### `List.scan` の引数順と戻り値

- vm.rs line 3466: `args[0] = list, args[1] = init, args[2] = func`
- 戻り値: 初期値を含む `[init, f(init,x1), ...]` — 入力 n 要素 → 出力 n+1 要素
- テスト: `scan([1,2,3], 0, +) = [0,1,3,6]` → `length == 4`

### `List.take_while` / `List.drop_while` の引数順

- vm.rs line 3389 / 3422: `args[0] = list, args[1] = func`（リスト先・関数後）

### テスト数

| バージョン | テスト数 | 差分 |
|---|---|---|
| v47.2.0 | 3021 | ベース |
| v47.3.0 | 3024 | +3 |
