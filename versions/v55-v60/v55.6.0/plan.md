# Plan — v55.6.0 — CEP（複合イベント処理）Stream 統合

## ステップ

### Step 1: Cargo.toml バージョン更新

`fav/Cargo.toml` の `version` を `55.6.0` に更新。

```toml
[package]
version = "55.6.0"
```

---

### Step 2: `compiler.rs` — `"CEP"` namespace 登録追加

`"State"` エントリと `"Mut"` エントリの間に追加する。

**変更後（追加箇所）:**
```rust
// v55.5.0 State（型付き VMValue State API — State.get/set/get_or_default namespace として登録）
"State",
// v55.6.0 CEP（複合イベント処理 — CEP.sequence/skip_until namespace として登録）
"CEP",
// v23.3.0 Mut コレクション（namespace として登録）
"Mut",
```

---

### Step 3: `checker.rs` — `CEP.sequence` / `CEP.skip_until` 型登録

`("Stream", "join_left")` エントリ（L6964 付近）と `("Stream", _)` ワイルドカードの間に追加する。

**変更前:**
```rust
("Stream", "join_left")  => Some(Type::Stream(Box::new(Type::Unknown))), // v55.4.0
("Stream", _) => Some(Type::Unknown),
```

**変更後:**
```rust
("Stream", "join_left")  => Some(Type::Stream(Box::new(Type::Unknown))), // v55.4.0
("Stream", _) => Some(Type::Unknown),

// CEP (v55.6.0): 複合イベント処理 VM primitive
("CEP", "sequence")   => Some(Type::List(Box::new(Type::Unknown))), // v55.6.0
("CEP", "skip_until") => Some(Type::List(Box::new(Type::Unknown))), // v55.6.0
("CEP", _)            => Some(Type::Unknown),
```

---

### Step 4: `vm.rs` — `is_known_builtin_namespace` に `"CEP"` を追加

`is_known_builtin_namespace` 関数（L8785 付近）の matches! 内の `"State"` エントリの直後に追加する。

**変更前:**
```rust
| "State"   // v22.3.0
| "Bytes"   // v23.1.0
```

**変更後:**
```rust
| "State"   // v22.3.0
| "CEP"     // v55.6.0
| "Bytes"   // v23.1.0
```

---

### Step 5: `vm.rs` — `CEP.sequence` / `CEP.skip_until` を `call_builtin` に追加

`Stream.join_left` アームの末尾（`// ── end v26.4.0 / v55.4.0 Stream.* ──` の直前）に挿入する。

```rust
// ── v55.6.0: CEP 複合イベント処理 ────────────────────────────────────────────
// CEP.sequence / CEP.skip_until は述語クロージャ呼び出しに &mut self が必要なため
// vm_call_builtin ではなく call_builtin（&mut self メソッド）に実装する。
"CEP.sequence" => {
    if args.len() != 2 {
        return Err(self.error(artifact, "CEP.sequence requires 2 arguments: (events: List, preds: List<Fn>)"));
    }
    let mut it = args.into_iter();
    let events = match it.next().unwrap() {
        VMValue::List(l) => l.to_vec(),
        other => return Err(self.error(artifact, &format!(
            "CEP.sequence: first argument must be a List, got {}", vmvalue_type_name(&other)
        ))),
    };
    let preds = match it.next().unwrap() {
        VMValue::List(l) => l.to_vec(),
        other => return Err(self.error(artifact, &format!(
            "CEP.sequence: second argument must be a List of predicates, got {}", vmvalue_type_name(&other)
        ))),
    };
    if preds.is_empty() {
        return Ok(VMValue::List(FavList::new(vec![])));
    }
    let mut results = Vec::new();
    for start in 0..events.len() {
        let first_ok = self.call_value(artifact, preds[0].clone(), vec![events[start].clone()])?;
        if !matches!(first_ok, VMValue::Bool(true)) {
            continue;
        }
        let mut current = vec![events[start].clone()];
        let mut pos = start + 1;
        let mut pred_i = 1;
        while pred_i < preds.len() && pos < events.len() {
            let m = self.call_value(artifact, preds[pred_i].clone(), vec![events[pos].clone()])?;
            if matches!(m, VMValue::Bool(true)) {
                current.push(events[pos].clone());
                pred_i += 1;
            }
            pos += 1;
        }
        if pred_i == preds.len() {
            results.push(VMValue::List(FavList::new(current)));
        }
    }
    Ok(VMValue::List(FavList::new(results)))
}
"CEP.skip_until" => {
    if args.len() != 2 {
        return Err(self.error(artifact, "CEP.skip_until requires 2 arguments: (events: List, pred: Fn)"));
    }
    let mut it = args.into_iter();
    let events = match it.next().unwrap() {
        VMValue::List(l) => l.to_vec(),
        other => return Err(self.error(artifact, &format!(
            "CEP.skip_until: first argument must be a List, got {}", vmvalue_type_name(&other)
        ))),
    };
    let pred = it.next().unwrap();
    let mut result = Vec::new();
    let mut found = false;
    for event in events {
        if !found {
            let m = self.call_value(artifact, pred.clone(), vec![event.clone()])?;
            if matches!(m, VMValue::Bool(true)) {
                found = true;
                result.push(event);
            }
        } else {
            result.push(event);
        }
    }
    Ok(VMValue::List(FavList::new(result)))
}
```

---

### Step 6: `driver.rs` — `v55600_tests` モジュール追加

`v55500_tests` の直前（`// -- v55500_tests` コメント行の前）に挿入する。

```rust
// -- v55600_tests (v55.6.0) -- CEP（複合イベント処理）Stream 統合 --
#[cfg(test)]
mod v55600_tests {
    use super::{build_artifact, exec_artifact_main};
    use crate::backend::vm::clear_state_value_store;
    use crate::frontend::parser::Parser;

    #[test]
    fn cep_stream_integration() {
        let src = r#"public fn main() -> Int {
            let events = ["login", "purchase", "logout", "login", "purchase"]
            let matches = CEP.sequence(events, [|e| e == "login", |e| e == "purchase"])
            List.length(matches)
        }"#;
        let program = Parser::parse_str(src, "cep_stream_integration.fav").expect("parse ok");
        let artifact = build_artifact(&program);
        let value = exec_artifact_main(&artifact, None).expect("exec ok");
        assert_eq!(
            value,
            crate::value::Value::Int(2),
            "CEP.sequence should find 2 (login,purchase) pairs, got {:?}", value
        );
    }

    #[test]
    fn cep_stateful_persistence() {
        clear_state_value_store();
        let src = r#"public fn main() -> Int {
            let events = ["noise", "noise", "start", "a", "b"]
            let filtered = CEP.skip_until(events, |e| e == "start")
            bind _ <- State.set("v55600_cep_len", List.length(filtered))
            bind n <- State.get_or_default("v55600_cep_len", 0)
            n
        }"#;
        let program = Parser::parse_str(src, "cep_stateful_persistence.fav").expect("parse ok");
        let artifact = build_artifact(&program);
        let value = exec_artifact_main(&artifact, None).expect("exec ok");
        assert_eq!(
            value,
            crate::value::Value::Int(3),
            "CEP.skip_until should skip 2 noises, return [start,a,b] len=3, got {:?}", value
        );
    }
}
```

---

### Step 7: テスト実行・確認

```bash
cd /c/Users/yoshi/favnir/fav && cargo build 2>&1 | tail -5
```

期待結果: `Finished`

```bash
cd /c/Users/yoshi/favnir/fav && cargo test -j 8 -- --test-threads=8 2>&1 | tail -20
```

期待結果: `3217 tests passed, 0 failed`

```bash
cd /c/Users/yoshi/favnir/fav && cargo clippy -- -D warnings 2>&1 | tail -10
```

期待結果: クリーン

---

### Step 8: ポスト処理

- `CHANGELOG.md` に v55.6.0 エントリ追加
- `versions/current.md` を v55.6.0 / 3217 tests に更新
- `versions/roadmap/roadmap-v55.1-v56.0.md` の v55.6.0 実績を COMPLETE に更新
- `versions/roadmap/roadmap-v55.1-v60.0.md` の v55.6.0 実績欄も COMPLETE に更新

---

## 注意事項

- `CEP.sequence` / `CEP.skip_until` は `call_builtin`（`&mut self`）に追加する。
  `vm_call_builtin`（free function）ではエラー型が `String` であり、
  また `self.call_value(...)` が使えないため不可。
- `"CEP"` は `compiler.rs` の namespace リスト AND `is_known_builtin_namespace` の両方に追加する。
  前者がないと `IRExpr::Global(u16::MAX)` になり、後者がないと `LoadGlobal` で名前解決に失敗する。
  v55.5.0 では `State` の `is_known_builtin_namespace` 追加は不要だった（既存登録済み）が、
  `CEP` は両方とも未登録のため両方追加が必要。
- `CEP.sequence` の `preds.is_empty()` ガードを最初に置くことで
  空述語リストへのゼロ割り等を防ぐ。
- テストの `cep_stateful_persistence` は `clear_state_value_store()` を冒頭で呼び出す
  （v55.5.0 パターンと同様）。
