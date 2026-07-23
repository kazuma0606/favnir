# Spec: v47.2.0 — `List.flat_map` / `List.group_by` / `List.dedupe`

## 概要

- `List.flat_map` / `List.group_by`: vm.rs・checker.rs に実装済み → テスト追加のみ
- `List.dedupe`: 未実装 → vm.rs・checker.rs に追加し、テストを追加する

---

## 問題

| 関数 | vm.rs | checker.rs | 状態 |
|---|---|---|---|
| `List.flat_map` | ✅ line 3486 | ✅ line 5943 | テストなし |
| `List.group_by` | ✅ line 3895 | ✅ line 6018 | テストなし |
| `List.dedupe` | ❌ | ❌ | 未実装 |

`List.unique`（line 11293）と `List.distinct`（line 11449）は類似の dedup 操作として実装済みだが、
ロードマップに指定された `List.dedupe` は未実装。

---

## 解決策

### 1. `List.dedupe` を vm.rs に追加

`List.unique`（line 11293）と同じロジック（HashSet + `vmvalue_repr` による重複判定）。
挿入位置: `List.distinct`（line 11449）の直後。

注意: `List.distinct`（挿入先直前）は `Vec::contains` による O(n²) 実装で、
`List.dedupe` が使う HashSet ロジックとは実装が異なる。`List.unique` が正しい参照実装。

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

### 2. `List.dedupe` を checker.rs に追加

`("List", "distinct")` エントリの直後に追加。

```rust
("List", "dedupe") => {
    let elem = self.expect_list_arg(&arg_tys, 0, span);
    Some(Type::List(Box::new(elem)))
}
```

### 3. `driver.rs` に `v472000_tests` 追加

挿入位置: `v471000_tests` の直後。

---

## テスト（+3）

| テスト名 | 内容 |
|---|---|
| `list_flat_map` | `List.flat_map(range(1,4), \|x\| List.singleton(x))` → `List.length == 3` |
| `list_group_by` | `List.group_by(\|x\| "bucket", range(1,4))` → `Map.size == 1` |
| `list_dedupe` | `dedupe([1, 2, 1])` → `List.length == 2` |

### テストコード

```favnir
// list_flat_map
fn main() -> Bool {
  bind xs     <- List.range(1, 4)
  bind result <- List.flat_map(xs, |x| List.singleton(x))
  List.length(result) == 3
}

// list_group_by
fn main() -> Bool {
  bind xs     <- List.range(1, 4)
  bind groups <- List.group_by(|x| "bucket", xs)
  Map.size(groups) == 1
}

// list_dedupe
fn main() -> Bool {
  bind xs  <- List.push(List.push(List.singleton(1), 2), 1)
  bind ys  <- List.dedupe(xs)
  List.length(ys) == 2
}
```

### 注意事項

- `List.flat_map`: 引数順は `(list, func)` — リスト先、クロージャ後
- `List.group_by`: 引数順は `(func, list)` — クロージャ先、リスト後
- `Map.size` は `VMValue::Record` を受け付ける（`List.group_by` の戻り値と一致）
- `List.range(1, 4)` = `[1, 2, 3]`（exclusive end、3 要素）
- `List.push(List.push(List.singleton(1), 2), 1)` = `[1, 2, 1]`（3 要素、重複あり）

---

## 完了条件

- `cargo test` 3021 passed, 0 failed（3018 + 3 件）
- `cargo clippy -- -D warnings` クリーン
- `fav/Cargo.toml` version → `"47.2.0"`
- `CHANGELOG.md` に v47.2.0 エントリ追加
- `versions/current.md` を v47.2.0（3021 tests）に更新
- `tasks.md` を COMPLETE に更新
