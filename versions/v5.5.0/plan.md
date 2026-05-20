# Favnir v5.5.0 実装計画 — 標準ライブラリ型シグネチャ補完

---

## 変更ファイル一覧

| ファイル | 変更内容 |
|---------|---------|
| `fav/src/middle/checker.rs` | List/String/Option/Result の型シグネチャ追加 + Map 節強化 |
| `fav/src/backend/vm.rs` | `Map.remove` / `Map.contains_key` / `String.from_chars` 追加 |

テスト追加先:
| ファイル | 変更内容 |
|---------|---------|
| `fav/src/middle/checker.rs` の末尾テスト群 | 型シグネチャ確認テスト |
| `fav/src/backend/vm_stdlib_tests.rs` | 新規 vm 関数テスト |

---

## Phase A: checker.rs — List 型シグネチャ追加

`check_builtin_apply` の `("List", "count") => ...` の直後に追記する。

```rust
("List", "flat_map") => {
    let _elem = self.expect_list_arg(&arg_tys, 0, span);
    let out = if let Some(f_ty) = arg_tys.get(1) {
        f_ty.as_callable()
            .map(|(_, o)| match o.as_ref() {
                Type::List(inner) => *inner.clone(),
                _ => Type::Unknown,
            })
            .unwrap_or(Type::Unknown)
    } else {
        Type::Unknown
    };
    Some(Type::List(Box::new(out)))
}
("List", "sort") => {
    let elem = self.expect_list_arg(&arg_tys, 0, span);
    Some(Type::List(Box::new(elem)))
}
("List", "find") => {
    let elem = self.expect_list_arg(&arg_tys, 0, span);
    Some(Type::Option(Box::new(elem)))
}
("List", "any") | ("List", "all") => {
    let _ = self.expect_list_arg(&arg_tys, 0, span);
    Some(Type::Bool)
}
("List", "index_of") => {
    let _ = self.expect_list_arg(&arg_tys, 0, span);
    Some(Type::Option(Box::new(Type::Int)))
}
("List", "zip") => {
    let left = self.expect_list_arg(&arg_tys, 0, span);
    let right = self.expect_list_arg(&arg_tys, 1, span);
    // Returns List<{first: A, second: B}>
    // Approximate as List<Unknown> until full row polymorphism
    let _ = (left, right);
    Some(Type::List(Box::new(Type::Unknown)))
}
("List", "range") => Some(Type::List(Box::new(Type::Int))),
("List", "reverse") | ("List", "concat") => {
    let elem = self.expect_list_arg(&arg_tys, 0, span);
    Some(Type::List(Box::new(elem)))
}
("List", "take") | ("List", "drop") => {
    let elem = self.expect_list_arg(&arg_tys, 0, span);
    Some(Type::List(Box::new(elem)))
}
```

---

## Phase B: checker.rs — String 型シグネチャ追加

`("String", "contains") | ...` の節の前後に追記する。

```rust
("String", "concat") => Some(Type::String),
("String", "replace") => Some(Type::String),
("String", "slice") => Some(Type::String),
("String", "repeat") => Some(Type::String),
("String", "char_at") => Some(Type::Option(Box::new(Type::String))),
("String", "to_int") => Some(Type::Option(Box::new(Type::Int))),
("String", "to_float") => Some(Type::Option(Box::new(Type::Float))),
("String", "from_chars") => Some(Type::String),
```

---

## Phase C: checker.rs — Option コンビネータ追加

`("Option", "unwrap_or") => ...` の後に追記する。

```rust
("Option", "and_then") => {
    let out = arg_tys
        .get(1)
        .and_then(|f| f.as_callable().map(|(_, o)| o.clone()))
        .unwrap_or(Type::Unknown);
    Some(Type::Option(Box::new(out)))
}
("Option", "or_else") => {
    let inner = match arg_tys.first() {
        Some(Type::Option(t)) => *t.clone(),
        _ => Type::Unknown,
    };
    Some(Type::Option(Box::new(inner)))
}
("Option", "is_some") | ("Option", "is_none") => Some(Type::Bool),
("Option", "to_result") => {
    let ok_ty = match arg_tys.first() {
        Some(Type::Option(t)) => *t.clone(),
        _ => Type::Unknown,
    };
    let err_ty = arg_tys.get(1).cloned().unwrap_or(Type::Unknown);
    Some(Type::Result(Box::new(ok_ty), Box::new(err_ty)))
}
```

---

## Phase D: checker.rs — Result コンビネータ追加

`("Result", "unwrap_or") => ...` の後に追記する。

```rust
("Result", "and_then") => {
    let out = arg_tys
        .get(1)
        .and_then(|f| f.as_callable().map(|(_, o)| o.clone()))
        .unwrap_or(Type::Unknown);
    let err_ty = match arg_tys.first() {
        Some(Type::Result(_, err)) => *err.clone(),
        _ => Type::Unknown,
    };
    Some(Type::Result(Box::new(out), Box::new(err_ty)))
}
("Result", "map_err") => {
    let ok_ty = match arg_tys.first() {
        Some(Type::Result(ok, _)) => *ok.clone(),
        _ => Type::Unknown,
    };
    let new_err = arg_tys
        .get(1)
        .and_then(|f| f.as_callable().map(|(_, o)| o.clone()))
        .unwrap_or(Type::Unknown);
    Some(Type::Result(Box::new(ok_ty), Box::new(new_err)))
}
("Result", "is_ok") | ("Result", "is_err") => Some(Type::Bool),
("Result", "to_option") => {
    let ok_ty = match arg_tys.first() {
        Some(Type::Result(ok, _)) => *ok.clone(),
        _ => Type::Unknown,
    };
    Some(Type::Option(Box::new(ok_ty)))
}
```

---

## Phase E: checker.rs — Map 節強化 + 新規追加

既存の `("Map", "get") | ("Map", "set") | ("Map", "keys") | ("Map", "values") | ("Map", _)` を
以下に置き換える。

```rust
("Map", "get") => Some(Type::Option(Box::new(Type::Unknown))),
("Map", "set") => Some(Type::Map(Box::new(Type::Unknown), Box::new(Type::Unknown))),
("Map", "keys") => Some(Type::List(Box::new(Type::Unknown))),
("Map", "values") => Some(Type::List(Box::new(Type::Unknown))),
("Map", "size") => Some(Type::Int),
("Map", "is_empty") => Some(Type::Bool),
("Map", "contains_key") => Some(Type::Bool),
("Map", "remove") => Some(Type::Map(Box::new(Type::Unknown), Box::new(Type::Unknown))),
("Map", "merge") => Some(Type::Map(Box::new(Type::Unknown), Box::new(Type::Unknown))),
("Map", "map_values") => Some(Type::Map(Box::new(Type::Unknown), Box::new(Type::Unknown))),
("Map", "filter_values") => Some(Type::Map(Box::new(Type::Unknown), Box::new(Type::Unknown))),
("Map", "to_list") => Some(Type::List(Box::new(Type::Unknown))),
("Map", "from_list") => Some(Type::Map(Box::new(Type::Unknown), Box::new(Type::Unknown))),
("Map", _) => Some(Type::Unknown),
```

---

## Phase F: vm.rs — 新規関数追加

### `Map.remove`

`"Map.from_list"` ブロックの前後に追加する。

```rust
"Map.remove" => {
    let mut it = args.into_iter();
    let map = it.next().ok_or_else(|| "Map.remove requires 2 arguments".to_string())?;
    let key = it.next().ok_or_else(|| "Map.remove requires 2 arguments".to_string())?;
    match (map, key) {
        (VMValue::Record(mut m), VMValue::Str(k)) => {
            m.remove(&k);
            Ok(VMValue::Record(m))
        }
        _ => Err("Map.remove requires (Map, String)".to_string()),
    }
}
```

### `Map.contains_key`

```rust
"Map.contains_key" => {
    let mut it = args.into_iter();
    let map = it.next().ok_or_else(|| "Map.contains_key requires 2 arguments".to_string())?;
    let key = it.next().ok_or_else(|| "Map.contains_key requires 2 arguments".to_string())?;
    match (map, key) {
        (VMValue::Record(m), VMValue::Str(k)) => {
            Ok(VMValue::Bool(m.contains_key(&k)))
        }
        _ => Err("Map.contains_key requires (Map, String)".to_string()),
    }
}
```

### `String.from_chars`

`"String.char_at"` ブロックの直後に追加する。

```rust
"String.from_chars" => {
    let v = args
        .into_iter()
        .next()
        .ok_or_else(|| "String.from_chars requires 1 argument".to_string())?;
    match v {
        VMValue::List(chars) => {
            let mut result = String::new();
            for c in chars {
                match c {
                    VMValue::Str(s) => result.push_str(&s),
                    other => return Err(format!(
                        "String.from_chars: each element must be String, got {}",
                        vmvalue_type_name(&other)
                    )),
                }
            }
            Ok(VMValue::Str(result))
        }
        _ => Err("String.from_chars requires a List<String> argument".to_string()),
    }
}
```

---

## Phase G: テスト追加

### vm_stdlib_tests.rs

```rust
#[test]
fn test_map_remove() { /* Map.remove("a") が消える */ }

#[test]
fn test_map_contains_key() { /* Map.contains_key(...) が Bool を返す */ }

#[test]
fn test_string_from_chars() { /* String.from_chars(String.chars("hello")) == "hello" */ }

#[test]
fn test_list_flat_map() { /* List.flat_map([1, 2], |x| [x, x*2]) == [1, 2, 2, 4] */ }

#[test]
fn test_list_sort() { /* List.sort([3,1,2], |a,b| a - b) == [1,2,3] */ }

#[test]
fn test_list_zip() { /* List.zip([1,2], ["a","b"]) の first/second */ }

#[test]
fn test_list_take_drop() { /* take(3) / drop(2) */ }

#[test]
fn test_option_and_then() { /* None -> None, Some(x) -> f(x) */ }

#[test]
fn test_result_and_then() { /* Err を通過 / Ok に f を適用 */ }
```

### checker.rs テスト

```rust
#[test]
fn test_list_flat_map_type() {
    check_ok("fn f(xs: List<Int>) -> List<String> { List.flat_map(xs, |x| [\"a\"]) }")
}

#[test]
fn test_option_and_then_type() {
    check_ok("fn f(o: Option<Int>) -> Option<String> { Option.and_then(o, |x| Option.some(\"ok\")) }")
}

#[test]
fn test_result_and_then_type() {
    check_ok("fn f(r: Result<Int, String>) -> Result<String, String> { Result.and_then(r, |x| Result.ok(\"ok\")) }")
}

#[test]
fn test_map_remove_type() {
    check_ok("fn f(m: Map<String, Int>) -> Map<String, Int> { Map.remove(m, \"key\") }")
}

#[test]
fn test_string_from_chars_type() {
    check_ok("fn f(cs: List<String>) -> String { String.from_chars(cs) }")
}
```

---

## 実装順序

1. Phase A: List 型シグネチャ追加 → `cargo test` で型チェックテスト確認
2. Phase B: String 型シグネチャ追加
3. Phase F-1: `String.from_chars` を vm.rs に追加
4. Phase C: Option 型シグネチャ追加
5. Phase D: Result 型シグネチャ追加
6. Phase E: Map 型シグネチャ強化
7. Phase F-2: `Map.remove` / `Map.contains_key` を vm.rs に追加
8. Phase G: テスト追加
9. `cargo test` 全件通過確認

---

## 注意点

### `as_callable` の扱い

`Type::Func(inputs, output)` が callable になる。
ただし checker.rs で closure の型が正確に解決されない場合があるため、
`and_then` / `map` 系は `unwrap_or(Type::Unknown)` でフォールバックする。

### 副作用なし

追加する全関数は `!` エフェクトなし。`require_xxx_effect` 呼び出し不要。

### 後方互換

既存のコードに影響なし（今まで Unknown だったものに具体型を付けるだけ）。
ただし `List.any` / `List.sort` 等が今まで型エラーになっていた場合、
型シグネチャ追加後は通るようになる（破壊的変更ではなく修正）。
