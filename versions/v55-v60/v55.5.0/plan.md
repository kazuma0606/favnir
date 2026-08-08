# Plan — v55.5.0 — Stateful stage（累積状態）

## ステップ

### Step 1: Cargo.toml バージョン更新

`fav/Cargo.toml` の `version` を `55.5.0` に更新。

```toml
[package]
version = "55.5.0"
```

---

### Step 2: `vm.rs` — `STATE_VALUE_STORE` thread-local 追加

`STATE_STORE` / `STATE_BACKEND` の thread-local ブロック（L1422〜L1428）の直後に挿入する。

**変更後（追加箇所）:**
```rust
/// v55.5.0: 型付き State ストア（String key → VMValue）
/// State.get / State.set / State.get_or_default で使用する。
/// State.get_raw / State.set_raw は引き続き STATE_STORE（String→String）を使用。
thread_local! {
    static STATE_VALUE_STORE: std::cell::RefCell<std::collections::HashMap<String, VMValue>>
        = std::cell::RefCell::new(std::collections::HashMap::new());
}
```

**挿入位置の確認方法**: `set_state_backend` 関数定義（`pub fn set_state_backend(backend: &str)`）の直前。

---

### Step 3: `vm.rs` — `State.get` / `State.set` / `State.get_or_default` primitive 追加

`vm_call_builtin` の `State.delete_raw` アームの直後（`// ── v23.1.0: Bytes 型 ──` の直前）に挿入する。

```rust
// ── v55.5.0: State.get / State.set / State.get_or_default（型付き VMValue State API）──
"State.get" => {
    let key = match args.into_iter().next() {
        Some(VMValue::Str(s)) => s,
        _ => return Err("State.get requires a String key".to_string()),
    };
    let val = STATE_VALUE_STORE.with(|c| c.borrow().get(&key).cloned());
    Ok(ok_vm(match val {
        Some(v) => VMValue::Variant("some".to_string(), Some(Box::new(v))),
        None    => VMValue::Variant("none".to_string(), None),
    }))
}
"State.set" => {
    let mut it = args.into_iter();
    let key = match it.next() {
        Some(VMValue::Str(s)) => s,
        _ => return Err("State.set: key must be a String".to_string()),
    };
    let value = match it.next() {
        Some(v) => v,
        None => return Err("State.set: missing value argument".to_string()),
    };
    STATE_VALUE_STORE.with(|c| c.borrow_mut().insert(key, value));
    Ok(ok_vm(VMValue::Unit))
}
"State.get_or_default" => {
    let mut it = args.into_iter();
    let key = match it.next() {
        Some(VMValue::Str(s)) => s,
        _ => return Err("State.get_or_default: key must be a String".to_string()),
    };
    let default_val = match it.next() {
        Some(v) => v,
        None => return Err("State.get_or_default: missing default argument".to_string()),
    };
    let val = STATE_VALUE_STORE.with(|c| c.borrow().get(&key).cloned())
        .unwrap_or(default_val);
    Ok(ok_vm(val))
}
```

---

### Step 4: `error_catalog.rs` — E0421 stub 追加

E0420 エントリの直後（`// ── E05xx: モジュール ──` コメントの直前）に挿入する。

```rust
// v55.5.0: Stateful stage — !State エフェクト enforcement stub
ErrorEntry {
    code: "E0421",
    title: "State operation without !State effect",
    category: "streaming",
    description: "A `State.get` / `State.set` / `State.get_or_default` call was used in a stage \
                  that does not declare the `!State` effect. Declare `!State` in the stage signature \
                  to enable stateful accumulation.",
    example: "stage Count: Stream<Int> -> Stream<Int> = |s| {\n  bind n <- State.get_or_default(\"c\", 0)\n  Ok(n)  // E0421: missing !State\n}",
    fix: "Add `!State` to the stage effect list: `stage Count: Stream<Int> -> Stream<Int> = |s| !State { ... }`",
    suggestion: Some("Declare `!State` in the stage signature to enable stateful accumulation."),
},
```

---

### Step 5: `checker.rs` — `State.get_or_default` 型登録

`("State", "get")` エントリ（L6446 付近）の直後に 1 行追加する。

**変更前:**
```rust
("State", "get") => {
    self.require_state_effect(span);
    Some(Type::Option(Box::new(Type::String)))
}
("State", "set") | ("State", "delete") => {
```

**変更後:**
```rust
("State", "get") => {
    self.require_state_effect(span);
    Some(Type::Option(Box::new(Type::String)))
}
("State", "get_or_default") => Some(Type::Unknown), // v55.5.0
("State", "set") | ("State", "delete") => {
```

---

### Step 6: `driver.rs` — `v55500_tests` モジュール追加

`v55400_tests` の直前（`// -- v55400_tests` コメント行の前）に挿入する。

```rust
// -- v55500_tests (v55.5.0) -- Stateful stage（累積状態）--
#[cfg(test)]
mod v55500_tests {
    use super::{build_artifact, exec_artifact_main};
    use crate::frontend::parser::Parser;

    #[test]
    fn stateful_stage_accumulates() {
        // State.set で Int 値を保存し、State.get_or_default で取得できることを検証
        let src = r#"public fn main() -> Int {
            bind _ <- State.set("v55500_counter", 42)
            bind val <- State.get_or_default("v55500_counter", 0)
            val
        }"#;
        let program = Parser::parse_str(src, "stateful_accumulate.fav").expect("parse ok");
        let artifact = build_artifact(&program);
        let value = exec_artifact_main(&artifact, None).expect("exec ok");
        assert_eq!(
            value,
            crate::value::Value::Int(42),
            "State.get_or_default should return stored value 42, got {:?}", value
        );
    }

    #[test]
    fn stateful_stage_persists() {
        // State.set で Bool 値を保存し、State.get_or_default でデフォルト値を上書きすることを検証
        let src = r#"public fn main() -> Bool {
            bind _ <- State.set("v55500_ready", true)
            bind val <- State.get_or_default("v55500_ready", false)
            val
        }"#;
        let program = Parser::parse_str(src, "stateful_persist.fav").expect("parse ok");
        let artifact = build_artifact(&program);
        let value = exec_artifact_main(&artifact, None).expect("exec ok");
        assert_eq!(
            value,
            crate::value::Value::Bool(true),
            "State.get_or_default should return stored Bool true, not default false, got {:?}", value
        );
    }
}
```

---

### Step 7: テスト実行・確認

```bash
cd /c/Users/yoshi/favnir/fav && cargo build 2>&1 | tail -5
```

期待結果: `Finished` — STATE_VALUE_STORE の `'static` 違反がないことを確認

```bash
cd /c/Users/yoshi/favnir/fav && cargo test -j 8 -- --test-threads=8 2>&1 | tail -20
```

期待結果: `3215 tests passed, 0 failed`

```bash
cd /c/Users/yoshi/favnir/fav && cargo clippy -- -D warnings 2>&1 | tail -10
```

期待結果: クリーン

---

### Step 8: ポスト処理

- `CHANGELOG.md` に v55.5.0 エントリ追加
- `versions/current.md` を v55.5.0 / 3215 tests に更新
- `versions/roadmap/roadmap-v55.1-v56.0.md` の v55.5.0 実績を COMPLETE に更新（3215 tests 訂正含む）
- `versions/roadmap/roadmap-v55.1-v60.0.md` の v55.5.0 実績欄も COMPLETE に更新

---

## 注意事項

- `STATE_VALUE_STORE` の定義位置: `STATE_STORE` / `STATE_BACKEND` ブロック直後。`set_state_backend` 関数より前。
  `VMValue` を thread_local! の `RefCell<HashMap<String, VMValue>>` に入れるには `VMValue: 'static` が必要。
  `VMValue` は全フィールドが所有型（`Arc`, `Box`, `Vec`, `HashMap`）のため `'static` を満たす。
- `State.get` / `State.set` / `State.get_or_default` は `vm_call_builtin`（L10013）に追加する。
  `call_builtin`（L3602）ではなく `vm_call_builtin` であることに注意（エラー型が `String`）。
- 既存の `("State", "get")` と `("State", "set")` のチェッカーエントリは v22.3.0 からあり、
  `require_state_effect` は v35.x で no-op 化済み。`State.get_or_default` のみ追加登録が必要。
- `v55400_tests` に `cargo_toml_version_is_55_4_0` が存在しないため削除タスクなし。
- テストキーに `v55500_` プレフィックスを使用: thread_local 汚染を防ぐため（テスト実行順序依存を排除）。
