# Plan: v47.2.0 — `List.flat_map` / `List.group_by` / `List.dedupe`

## 方針

- `List.flat_map` / `List.group_by` は vm.rs・checker.rs 実装済み → テスト追加のみ
- `List.dedupe` は未実装 → vm.rs + checker.rs に追加してからテスト

---

## 実装ステップ

### Step 1: `vm.rs` に `List.dedupe` を追加

挿入位置: `List.distinct`（line 11449）の直後。

```rust
"List.dedupe" => {
    let v = args
        .into_iter()
        .next()
        .ok_or_else(|| "List.dedupe requires 1 argument".to_string())?;
    match v {
        VMValue::List(fl) => {
            let mut seen = HashSet::new();
            let mut out = Vec::with_capacity(fl.len());
            for item in fl {
                let key = vmvalue_repr(&item);
                if seen.insert(key) {
                    out.push(item);
                }
            }
            Ok(VMValue::List(FavList::new(out)))
        }
        _ => Err("List.dedupe requires a List argument".to_string()),
    }
}
```

### Step 2: `checker.rs` に `List.dedupe` を追加

挿入位置: `("List", "distinct")` エントリ（line 6031）の直後。

```rust
("List", "dedupe") => {
    let elem = self.expect_list_arg(&arg_tys, 0, span);
    Some(Type::List(Box::new(elem)))
}
```

### Step 3: `driver.rs` に `v472000_tests` 追加

挿入位置: `v471000_tests` モジュールの直後（`v47000_tests` の前）。

```rust
// -- v472000_tests (v47.2.0) -- List.flat_map / List.group_by / List.dedupe 動作確認 --
#[cfg(test)]
mod v472000_tests {
    use crate::frontend::parser::Parser;
    use super::{build_artifact, exec_artifact_main};

    #[test]
    fn list_flat_map() {
        // flat_map([1,2,3], |x| [x]) = [1,2,3] → length 3
        let src = r#"
fn main() -> Bool {
  bind xs     <- List.range(1, 4)
  bind result <- List.flat_map(xs, |x| List.singleton(x))
  List.length(result) == 3
}
"#;
        let program = Parser::parse_str(src, "list_flat_map_test.fav").expect("parse");
        let artifact = build_artifact(&program);
        let value = exec_artifact_main(&artifact, None).expect("exec");
        assert_eq!(value, crate::value::Value::Bool(true));
    }

    #[test]
    fn list_group_by() {
        // group_by(|x| "bucket", [1,2,3]) = {"bucket": [1,2,3]} → Map.size == 1
        let src = r#"
fn main() -> Bool {
  bind xs     <- List.range(1, 4)
  bind groups <- List.group_by(|x| "bucket", xs)
  Map.size(groups) == 1
}
"#;
        let program = Parser::parse_str(src, "list_group_by_test.fav").expect("parse");
        let artifact = build_artifact(&program);
        let value = exec_artifact_main(&artifact, None).expect("exec");
        assert_eq!(value, crate::value::Value::Bool(true));
    }

    #[test]
    fn list_dedupe() {
        // dedupe([1, 2, 1]) = [1, 2] → length 2
        let src = r#"
fn main() -> Bool {
  bind xs <- List.push(List.push(List.singleton(1), 2), 1)
  bind ys <- List.dedupe(xs)
  List.length(ys) == 2
}
"#;
        let program = Parser::parse_str(src, "list_dedupe_test.fav").expect("parse");
        let artifact = build_artifact(&program);
        let value = exec_artifact_main(&artifact, None).expect("exec");
        assert_eq!(value, crate::value::Value::Bool(true));
    }
}
```

### Step 4: `Cargo.toml` バージョン更新

```toml
version = "47.2.0"
```

### Step 5: `CHANGELOG.md` 更新

```markdown
## [v47.2.0] — 2026-07-17

### Added
- `List.dedupe` VM primitive 追加（vm.rs / checker.rs）
- `driver.rs`: `v472000_tests` 追加（`list_flat_map` / `list_group_by` / `list_dedupe` 3テスト）
```

### Step 6: `versions/current.md` 更新

- 最新安定版を `v47.2.0`（3021 tests）に更新
- 進行中バージョンを `v47.3.0` に更新

---

## 注意事項

### `List.group_by` 引数順

- vm.rs での呼び出し: `args[0] = func, args[1] = list` (関数が先)
- テストの呼び出し: `List.group_by(|x| "bucket", xs)` — これが正しい引数順

### `List.flat_map` 引数順

- vm.rs での呼び出し: `args[0] = list, args[1] = func` (リストが先)
- テストの呼び出し: `List.flat_map(xs, |x| List.singleton(x))` — これが正しい引数順

### `List.dedupe` の参照実装

- `List.unique`（line 11293）: HashSet + vmvalue_repr → **これが参照実装**
- `List.distinct`（line 11449）: Vec::contains の O(n²) 実装 → 挿入位置の直前だが実装パターンは異なる
- `List.dedupe`: `List.unique` と同じ HashSet パターンで実装する

### テスト数

| バージョン | テスト数 | 差分 |
|---|---|---|
| v47.1.0 | 3018 | ベース |
| v47.2.0 | 3021 | +3 |
