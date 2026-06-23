# v23.3.0 — 可変コレクション `Mut<T>` 実装プラン

## 実装順序

```
T1（heap_val.rs）  ← 最初（HeapVal::MutList/MutMap の定義。以降の依存元）
T2（nan_val.rs）   ← T1 完了後
T3（vm.rs）        ← T2 完了後（最大タスク）
T4（checker.rs）   ← T3 と並列可
T5（compiler.rs）  ← T3 と並列可
T6（driver.rs）    ← T3〜T5 完了後、T7-1 より前に #[ignore] を実施
T7（docs）         ← T6 完了後（#[ignore] 確認後にバージョン更新）
```

---

## T1: `fav/src/backend/heap_val.rs` — `HeapVal::MutList(u64)` / `HeapVal::MutMap(u64)` 追加

### 事前確認

```bash
grep -n "Bytes\|BigInt\|PgPool" fav/src/backend/heap_val.rs | head -10
```

### 追加コード

`HeapVal::Bytes(u64)` の直後、`BigInt(i64)` の前に追加:

```rust
/// v23.3.0: 可変リスト opaque handle
MutList(u64),
/// v23.3.0: 可変マップ opaque handle
MutMap(u64),
```

`PartialEq` の `Bytes` アームの直後に追加:

```rust
(HeapVal::MutList(a), HeapVal::MutList(b)) => a == b,
(HeapVal::MutMap(a),  HeapVal::MutMap(b))  => a == b,
```

---

## T2: `fav/src/backend/nan_val.rs` — `from_vmvalue` / `to_vmvalue` メソッドに MutList/MutMap 追加

> **注意**: `nan_val.rs` には `From<VMValue>` trait 実装ではなく `from_vmvalue` と `to_vmvalue` という
> `impl NanVal` のメソッドが定義されている。これらのメソッドへの追加が必要。

### 事前確認

```bash
grep -n "Bytes\|PgPool\|from_vmvalue\|to_vmvalue" fav/src/backend/nan_val.rs | head -10
```

### `from_vmvalue` メソッドへの追加

`VMValue::Bytes(id) => NanVal::from_heap(HeapVal::Bytes(id)),` の直後:

```rust
VMValue::MutList(id) => NanVal::from_heap(HeapVal::MutList(id)),
VMValue::MutMap(id)  => NanVal::from_heap(HeapVal::MutMap(id)),
```

### `to_vmvalue` メソッドへの追加

`HeapVal::Bytes(id) => VMValue::Bytes(*id),` の直後:

```rust
HeapVal::MutList(id) => VMValue::MutList(*id),
HeapVal::MutMap(id)  => VMValue::MutMap(*id),
```

---

## T3: `fav/src/backend/vm.rs` — VMValue + STORE + vm_call_builtin

### 事前確認

```bash
grep -n "VMValue::Bytes\|BYTES_STORE\|NEXT_BYTES_ID" fav/src/backend/vm.rs | head -10
```

### T3-1: `VMValue` 列挙型に 2 バリアント追加

`VMValue::Bytes(u64)` の直後に追加:

```rust
/// v23.3.0: 可変リスト opaque handle
MutList(u64),
/// v23.3.0: 可変マップ opaque handle
MutMap(u64),
```

### T3-2: 各マッチアームに追加

**PartialEq** — `(VMValue::Bytes(a), VMValue::Bytes(b)) => a == b,` の直後:
```rust
(VMValue::MutList(a), VMValue::MutList(b)) => a == b,
(VMValue::MutMap(a),  VMValue::MutMap(b))  => a == b,
```

**VMValue→Value 変換** — `VMValue::Bytes(id) => Value::Str(format!("<bytes:{id}>"))` の直後:
```rust
VMValue::MutList(id) => Value::Str(format!("<mut-list:{id}>")),
VMValue::MutMap(id)  => Value::Str(format!("<mut-map:{id}>")),
```

**debug display** — `VMValue::Bytes(id) => format!("<bytes:{}>", id)` の直後:
```rust
VMValue::MutList(id) => format!("<mut-list:{}>", id),
VMValue::MutMap(id)  => format!("<mut-map:{}>", id),
```

**vmvalue_type_name** — `VMValue::Bytes(_) => "Bytes"` の直後:
```rust
VMValue::MutList(_) => "MutList",
VMValue::MutMap(_)  => "MutMap",
```

**stringify** — `VMValue::Bytes(_) => "<bytes>".to_string()` の直後:
```rust
VMValue::MutList(_) => "<mut-list>".to_string(),
VMValue::MutMap(_)  => "<mut-map>".to_string(),
```

**HeapVal type_name** — `HeapVal::Bytes(_) => "Bytes"` の直後:
```rust
HeapVal::MutList(_) => "MutList",
HeapVal::MutMap(_)  => "MutMap",
```

### T3-3: thread-local ストレージと helper 関数

`BYTES_STORE` / `NEXT_BYTES_ID` の定義の直後に追加。
**BYTES_STORE と同じパターン**: `Cell<u64>` でカウンタ、`RefCell<HashMap<u64, Vec<...>>>` でストア（二重 RefCell は使わない）。

```rust
// v23.3.0: Mut コレクションストレージ（GC 未実装のためメモリリークあり、v25.x で対応予定）
thread_local! {
    static MUT_LIST_STORE: std::cell::RefCell<
        std::collections::HashMap<u64, Vec<VMValue>>
    > = std::cell::RefCell::new(std::collections::HashMap::new());
    static NEXT_MUT_LIST_ID: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
    static MUT_MAP_STORE: std::cell::RefCell<
        std::collections::HashMap<u64, Vec<(VMValue, VMValue)>>
    > = std::cell::RefCell::new(std::collections::HashMap::new());
    static NEXT_MUT_MAP_ID: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

fn mut_list_new() -> u64 {
    NEXT_MUT_LIST_ID.with(|c| {
        let id = c.get();
        c.set(id + 1);
        MUT_LIST_STORE.with(|m| m.borrow_mut().insert(id, Vec::new()));
        id
    })
}

fn mut_map_new() -> u64 {
    NEXT_MUT_MAP_ID.with(|c| {
        let id = c.get();
        c.set(id + 1);
        MUT_MAP_STORE.with(|m| m.borrow_mut().insert(id, Vec::new()));
        id
    })
}
```

### T3-4: `vm_call_builtin` — 10 ハンドラを追加

`"Bytes.write_file"` ハンドラの直後（もしくは Bytes ブロックの末尾）に追加。

> **重要な制約**:
> - `err_vm` は `VMValue` を受け取る: `err_vm(VMValue::Str("msg".to_string()))` と書く（`String` は不可）
> - `ok_vm` も `VMValue` を受け取る: `ok_vm(VMValue::Unit)`
> - `MUT_LIST_STORE.with(|s| s.borrow_mut().get_mut(&id).ok_or_else(...).map(...))` で `Result<VMValue, String>` を得る（`?` は使わずそのまま返す）
> - `Mut.push` / `Mut.set` / `Mut.delete` は `ok_vm(VMValue::Unit)` を返す（Result として）

```rust
// v23.3.0: Mut.* handlers
"Mut.list" => {
    Ok(VMValue::MutList(mut_list_new()))
}
"Mut.push" => {
    let mut it = args.into_iter();
    let handle = it.next().ok_or_else(|| "Mut.push requires 2 arguments".to_string())?;
    let val    = it.next().ok_or_else(|| "Mut.push requires 2 arguments".to_string())?;
    let id = match handle {
        VMValue::MutList(id) => id,
        _ => return Err("Mut.push: first argument must be a MutList".to_string()),
    };
    MUT_LIST_STORE.with(|s| {
        s.borrow_mut()
            .get_mut(&id)
            .ok_or_else(|| format!("Mut.push: invalid MutList handle {}", id))
            .map(|vec| vec.push(val))
    })?;
    Ok(ok_vm(VMValue::Unit))
}
"Mut.pop" => {
    let handle = args.into_iter().next()
        .ok_or_else(|| "Mut.pop requires 1 argument".to_string())?;
    let id = match handle {
        VMValue::MutList(id) => id,
        _ => return Err("Mut.pop: argument must be a MutList".to_string()),
    };
    MUT_LIST_STORE.with(|s| {
        s.borrow_mut()
            .get_mut(&id)
            .ok_or_else(|| format!("Mut.pop: invalid MutList handle {}", id))
            .map(|vec| {
                vec.pop()
                    .map(ok_vm)
                    .unwrap_or_else(|| err_vm(VMValue::Str("Mut.pop: list is empty".to_string())))
            })
    })
}
"Mut.peek" => {
    let handle = args.into_iter().next()
        .ok_or_else(|| "Mut.peek requires 1 argument".to_string())?;
    let id = match handle {
        VMValue::MutList(id) => id,
        _ => return Err("Mut.peek: argument must be a MutList".to_string()),
    };
    MUT_LIST_STORE.with(|s| {
        s.borrow()
            .get(&id)
            .ok_or_else(|| format!("Mut.peek: invalid MutList handle {}", id))
            .map(|vec| {
                vec.last()
                    .cloned()
                    .map(ok_vm)
                    .unwrap_or_else(|| err_vm(VMValue::Str("Mut.peek: list is empty".to_string())))
            })
    })
}
"Mut.len" => {
    let handle = args.into_iter().next()
        .ok_or_else(|| "Mut.len requires 1 argument".to_string())?;
    match handle {
        VMValue::MutList(id) => {
            MUT_LIST_STORE.with(|s| {
                s.borrow()
                    .get(&id)
                    .ok_or_else(|| format!("Mut.len: invalid MutList handle {}", id))
                    .map(|vec| VMValue::Int(vec.len() as i64))
            })
        }
        VMValue::MutMap(id) => {
            MUT_MAP_STORE.with(|s| {
                s.borrow()
                    .get(&id)
                    .ok_or_else(|| format!("Mut.len: invalid MutMap handle {}", id))
                    .map(|vec| VMValue::Int(vec.len() as i64))
            })
        }
        _ => Err("Mut.len: argument must be a MutList or MutMap".to_string()),
    }
}
"Mut.map" => {
    Ok(VMValue::MutMap(mut_map_new()))
}
"Mut.set" => {
    let mut it = args.into_iter();
    let handle = it.next().ok_or_else(|| "Mut.set requires 3 arguments".to_string())?;
    let key    = it.next().ok_or_else(|| "Mut.set requires 3 arguments".to_string())?;
    let val    = it.next().ok_or_else(|| "Mut.set requires 3 arguments".to_string())?;
    let id = match handle {
        VMValue::MutMap(id) => id,
        _ => return Err("Mut.set: first argument must be a MutMap".to_string()),
    };
    MUT_MAP_STORE.with(|s| {
        s.borrow_mut()
            .get_mut(&id)
            .ok_or_else(|| format!("Mut.set: invalid MutMap handle {}", id))
            .map(|vec| {
                if let Some(entry) = vec.iter_mut().find(|(k, _)| k == &key) {
                    entry.1 = val;
                } else {
                    vec.push((key, val));
                }
            })
    })?;
    Ok(ok_vm(VMValue::Unit))
}
"Mut.get" => {
    let mut it = args.into_iter();
    let handle = it.next().ok_or_else(|| "Mut.get requires 2 arguments".to_string())?;
    let key    = it.next().ok_or_else(|| "Mut.get requires 2 arguments".to_string())?;
    let id = match handle {
        VMValue::MutMap(id) => id,
        _ => return Err("Mut.get: first argument must be a MutMap".to_string()),
    };
    MUT_MAP_STORE.with(|s| {
        s.borrow()
            .get(&id)
            .ok_or_else(|| format!("Mut.get: invalid MutMap handle {}", id))
            .map(|vec| {
                vec.iter()
                    .find(|(k, _)| k == &key)
                    .map(|(_, v)| ok_vm(v.clone()))
                    .unwrap_or_else(|| err_vm(VMValue::Str("Mut.get: key not found".to_string())))
            })
    })
}
"Mut.delete" => {
    let mut it = args.into_iter();
    let handle = it.next().ok_or_else(|| "Mut.delete requires 2 arguments".to_string())?;
    let key    = it.next().ok_or_else(|| "Mut.delete requires 2 arguments".to_string())?;
    let id = match handle {
        VMValue::MutMap(id) => id,
        _ => return Err("Mut.delete: first argument must be a MutMap".to_string()),
    };
    MUT_MAP_STORE.with(|s| {
        s.borrow_mut()
            .get_mut(&id)
            .ok_or_else(|| format!("Mut.delete: invalid MutMap handle {}", id))
            .map(|vec| vec.retain(|(k, _)| k != &key))
    })?;
    Ok(ok_vm(VMValue::Unit))
}
"Mut.has" => {
    let mut it = args.into_iter();
    let handle = it.next().ok_or_else(|| "Mut.has requires 2 arguments".to_string())?;
    let key    = it.next().ok_or_else(|| "Mut.has requires 2 arguments".to_string())?;
    let id = match handle {
        VMValue::MutMap(id) => id,
        _ => return Err("Mut.has: first argument must be a MutMap".to_string()),
    };
    MUT_MAP_STORE.with(|s| {
        s.borrow()
            .get(&id)
            .ok_or_else(|| format!("Mut.has: invalid MutMap handle {}", id))
            .map(|vec| VMValue::Bool(vec.iter().any(|(k, _)| k == &key)))
    })
}
```

### T3-5: `is_known_builtin_namespace` に `"Mut"` 追加

`| "Bytes"   // v23.1.0` の直後に追加:

```rust
| "Mut"     // v23.3.0
```

---

## T4: `fav/src/middle/checker.rs` — namespace リスト + builtin_ret_ty 更新

### 事前確認

```bash
grep -n '"Bytes"\|// Mut\|// Int bit' fav/src/middle/checker.rs | head -5
```

### T4-1: namespace リスト

`"Bytes",` の直後に追加:

```rust
"Mut",
```

### T4-2: `builtin_ret_ty` に追加

既存の `// Int bit operations v23.2.0` エントリの直後に追加:

```rust
// Mut コレクション v23.3.0
("Mut", "list") | ("Mut", "map") => Some(Type::Unknown),
("Mut", "push") | ("Mut", "set") | ("Mut", "delete") =>
    Some(Type::Result(Box::new(Type::Unit), Box::new(Type::String))),
("Mut", "pop") | ("Mut", "peek") | ("Mut", "get") =>
    Some(Type::Result(Box::new(Type::Unknown), Box::new(Type::String))),
("Mut", "len") => Some(Type::Int),
("Mut", "has") => Some(Type::Bool),
```

---

## T5: `fav/src/middle/compiler.rs` — builtins リスト更新

### 事前確認

```bash
grep -n '"Bytes"\|"Arena\.stats"' fav/src/middle/compiler.rs | head -5
```

### 追加コード

`"Bytes",` の直後に追加:

```rust
// v23.3.0 Mut コレクション（namespace として登録）
"Mut",
```

---

## T6: `fav/src/driver.rs` — `#[ignore]` + `v233000_tests` 追加

### T6-1: `#[ignore]` 追加（Cargo.toml 変更前に実施）

`v232000_tests::version_is_23_2_0` に `#[ignore]` を追加。

```bash
grep -n "fn version_is_23_2_0\|mod v232000_tests" fav/src/driver.rs | head -5
```

> **順序制約**: T7-1（Cargo.toml 更新）より前に `#[ignore]` を追加すること。
> `version_is_23_2_0` テストが Cargo.toml の "23.3.0" 文字列で失敗するため。

### T6-2: `v233000_tests` モジュール追加

`v232000_tests` モジュールの直後に追加。テストは `Lexer → Parser → build_artifact → exec_artifact_main(&artifact, None)` の 4 ステップパターン。

```rust
// ── v233000_tests (v23.3.0) — Mut<T> 可変コレクション ──────────────────────────
#[cfg(test)]
mod v233000_tests {
    use super::*;

    #[test]
    fn version_is_23_3_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("version = \"23.3.0\""), "Cargo.toml should have version 23.3.0");
    }

    #[test]
    fn mut_list_push_pop_correct() {
        // push(42), push(99), pop -> ok(99)
        let src = r#"
public fn main() -> Int {
  bind stack <- Mut.list()
  bind _p1 <- Mut.push(stack, 42)
  bind _p2 <- Mut.push(stack, 99)
  bind result <- Mut.pop(stack)
  match result {
    ok(v) => v
    err(_) => -1
  }
}
"#;
        let tokens = crate::frontend::lexer::Lexer::new(src, "test.fav")
            .tokenize().expect("lex");
        let prog = crate::frontend::parser::Parser::new(tokens)
            .parse_program().expect("parse");
        let artifact = build_artifact(&prog);
        let result = exec_artifact_main(&artifact, None).expect("exec");
        assert_eq!(result, crate::value::Value::Int(99), "pop should return last pushed value");
    }

    #[test]
    fn mut_list_len_after_push() {
        let src = r#"
public fn main() -> Int {
  bind stack <- Mut.list()
  bind _p1 <- Mut.push(stack, 10)
  bind _p2 <- Mut.push(stack, 20)
  Mut.len(stack)
}
"#;
        let tokens = crate::frontend::lexer::Lexer::new(src, "test.fav")
            .tokenize().expect("lex");
        let prog = crate::frontend::parser::Parser::new(tokens)
            .parse_program().expect("parse");
        let artifact = build_artifact(&prog);
        let result = exec_artifact_main(&artifact, None).expect("exec");
        assert_eq!(result, crate::value::Value::Int(2), "len should be 2 after 2 pushes");
    }

    #[test]
    fn mut_map_set_get_correct() {
        let src = r#"
public fn main() -> Int {
  bind m <- Mut.map()
  bind _s1 <- Mut.set(m, "key", 42)
  bind result <- Mut.get(m, "key")
  match result {
    ok(v) => v
    err(_) => -1
  }
}
"#;
        let tokens = crate::frontend::lexer::Lexer::new(src, "test.fav")
            .tokenize().expect("lex");
        let prog = crate::frontend::parser::Parser::new(tokens)
            .parse_program().expect("parse");
        let artifact = build_artifact(&prog);
        let result = exec_artifact_main(&artifact, None).expect("exec");
        assert_eq!(result, crate::value::Value::Int(42), "get should return the set value");
    }

    #[test]
    fn changelog_has_v23_3_0() {
        let cl = include_str!("../../CHANGELOG.md");
        assert!(cl.contains("[v23.3.0]"), "CHANGELOG.md should have [v23.3.0] entry");
    }
}
```

---

## T7: Cargo.toml / CHANGELOG / benchmarks / MDX

### T7-1: `fav/Cargo.toml` バージョン更新（T6-1 の `#[ignore]` 追加後）

```toml
version = "23.3.0"
```

### T7-2: `CHANGELOG.md` — v23.3.0 エントリ追加（v23.2.0 の上）

```markdown
## [v23.3.0] — 2026-06-21

### Added
- `Mut.list` / `Mut.push` / `Mut.pop` / `Mut.peek` / `Mut.len`（可変リスト）
- `Mut.map` / `Mut.set` / `Mut.get` / `Mut.delete` / `Mut.has`（可変マップ）
- `VMValue::MutList(u64)` / `VMValue::MutMap(u64)` opaque handle（NaN-boxing 準拠）
- thread-local `MUT_LIST_STORE` / `MUT_MAP_STORE`（`Vec<(VMValue, VMValue)>` 線形探索）
- `site/content/docs/runes/mut.mdx` — Mut コレクションドキュメント
```

### T7-3: `benchmarks/v23.3.0.json` — 新規作成

`test_count` は `cargo test --bin fav` 実行後の実測値を記入。

### T7-4: `site/content/docs/runes/mut.mdx` — 新規作成

`Mut.list` / `Mut.map` の使い方・vm.fav での活用例・スコープ外事項（線形型強制・関数値格納は将来課題）。

---

## 検証手順

```bash
cd /c/Users/yoshi/favnir/fav

# 単体テスト
cargo test v233000 --bin fav

# リグレッションなし確認
cargo test --bin fav
```

期待: v233000_tests 5/5 PASS、全体リグレッションなし。
