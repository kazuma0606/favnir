# v23.1.0 実装計画 — `Bytes` 型

## 実装順序

```
T1（heap_val.rs）   ← 最初（HeapVal::Bytes 追加。nan_val.rs / vm.rs の依存元）
T2（nan_val.rs）    ← T1 完了後（VMValue::Bytes ↔ NanVal 変換）
T3（vm.rs）         ← T2 完了後（VMValue::Bytes + BYTES_STORE + vm_call_builtin + is_known_builtin_namespace）
T4（checker.rs）    ← T3 と並列可（namespace リスト追加のみ）
T5（compiler.rs）   ← T3 と並列可（builtins リスト追加のみ）
T6（driver.rs）     ← T3〜T5 完了後（#[ignore] + v231000_tests 追加）
T7（Cargo + docs）  ← T6 完了後（バージョン更新・CHANGELOG・benchmarks・MDX）
```

---

## T1: `fav/src/backend/heap_val.rs` — `HeapVal::Bytes(u64)` 追加

### 事前確認

```bash
grep -n "PgPool\|ArrowBatch\|BigInt" fav/src/backend/heap_val.rs | head -10
```

`PgPool(u64)` の行番号と `PartialEq` の `_` アームを確認する。

### T1-1: `HeapVal` 列挙型にバリアント追加

`PgPool(u64)` の直後、`BigInt(i64)` の前に追加:

```rust
/// v23.1.0: 生バイト列 opaque handle
Bytes(u64),
```

### T1-2: `PartialEq` 実装に追加

`(HeapVal::PgPool(a), HeapVal::PgPool(b)) => a == b,` の直後に追加:

```rust
(HeapVal::Bytes(a), HeapVal::Bytes(b)) => a == b,
```

---

## T2: `fav/src/backend/nan_val.rs` — VMValue::Bytes ↔ NanVal 変換追加

### 事前確認

```bash
grep -n "PgPool\|ArrowBatch" fav/src/backend/nan_val.rs | head -10
```

`VMValue::PgPool(id)` の行と `HeapVal::PgPool(id)` の行を確認する。

### T2-1: `From<VMValue> for NanVal`（or `impl VMValue` の to_nanval）

`VMValue::PgPool(id) => NanVal::from_heap(HeapVal::PgPool(id)),` の直後に追加:

```rust
VMValue::Bytes(id) => NanVal::from_heap(HeapVal::Bytes(id)),
```

### T2-2: `to_vmvalue()` のヒープ値マッチング

`HeapVal::PgPool(id) => VMValue::PgPool(*id),` の直後に追加:

```rust
HeapVal::Bytes(id) => VMValue::Bytes(*id),
```

---

## T3: `fav/src/backend/vm.rs` — VMValue + BYTES_STORE + vm_call_builtin

### 事前確認

```bash
# VMValue::PgPool 周辺の行番号確認
grep -n "PgPool\|ArrowBatch\|Bytes" fav/src/backend/vm.rs | head -15

# NEXT_ARROW_ID / ARROW_BATCHES の行番号確認（BYTES ストアの挿入位置）
grep -n "NEXT_ARROW_ID\|ARROW_BATCHES\|PUSHDOWN_EXPLAIN" fav/src/backend/vm.rs | head -10

# is_known_builtin_namespace の "State" 行を確認
grep -n "\"State\"\|is_known_builtin_namespace" fav/src/backend/vm.rs | head -5

# vm_call_builtin の State / Arena 周辺確認（Bytes ハンドラの挿入位置）
grep -n "\"State\.\|\"Arena\.\|\"Bytes\." fav/src/backend/vm.rs | head -5

# type_name_of / vmvalue_to_display / vmvalue_type_name 等のマッチを確認
grep -n "ArrowBatch\|PgPool" fav/src/backend/vm.rs | grep -v "fn \|//\|from_heap\|to_vmvalue\|thread_local" | head -20
```

### T3-1: `VMValue` 列挙型にバリアント追加

`/// v20.8.0: DB コネクションプール opaque handle` の直後（`PgPool(u64)` の後、閉じ `}` の前）:

```rust
/// v23.1.0: 生バイト列 opaque handle
Bytes(u64),
```

### T3-2: `PartialEq` 実装に追加

`(VMValue::PgPool(a), VMValue::PgPool(b)) => a == b,` の直後:

```rust
(VMValue::Bytes(a), VMValue::Bytes(b)) => a == b,
```

### T3-3: display / type_name 等のマッチアームに追加

`ArrowBatch` と `PgPool` を表示する match アームをすべて grep で確認し、同じ直後に `Bytes` アームを追加する。代表例:

```rust
VMValue::Bytes(id)  => Value::Str(format!("<bytes:{id}>")),
VMValue::Bytes(_)   => "Bytes",                             // type_name
VMValue::Bytes(id)  => format!("<bytes:{}>", id),           // debug display
```

### T3-4: thread-local ストア追加

`NEXT_ARROW_ID` の thread_local! ブロックの直後（`PUSHDOWN_EXPLAIN_ENABLED` の前）に追加:

```rust
// ── v23.1.0: Bytes スレッドローカルストア ────────────────────────────────────
thread_local! {
    static BYTES_STORE: std::cell::RefCell<
        std::collections::HashMap<u64, std::sync::Arc<Vec<u8>>>
    > = std::cell::RefCell::new(std::collections::HashMap::new());
    static NEXT_BYTES_ID: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

fn bytes_new(data: Vec<u8>) -> u64 {
    NEXT_BYTES_ID.with(|c| {
        let id = c.get();
        c.set(id + 1);
        BYTES_STORE.with(|m| m.borrow_mut().insert(id, std::sync::Arc::new(data)));
        id
    })
}

fn bytes_get_arc(id: u64) -> Option<std::sync::Arc<Vec<u8>>> {
    BYTES_STORE.with(|m| m.borrow().get(&id).cloned())
}
```

### T3-5: `vm_call_builtin` に Bytes ハンドラ追加

`"State."` または `"Arena."` ハンドラの直後に挿入（`is_known_builtin_namespace` を grep して挿入位置を決定）:

```rust
// ── v23.1.0: Bytes 型 ────────────────────────────────────────────────────────
// 注意: flat literal アームで追加（guard パターン `n if n.starts_with(...)` は使わない）
"Bytes.from_hex" => {
    let s = match args.into_iter().next() {
        Some(VMValue::Str(s)) => s,
        _ => return Err(err_vm(VMValue::Str("Bytes.from_hex: expected String".into()))),
    };
    let s = s.trim();
    if s.len() % 2 != 0 {
        return Err(err_vm(VMValue::Str("Bytes.from_hex: odd length".into())));
    }
    let bytes: Result<Vec<u8>, _> = (0..s.len() / 2)
        .map(|i| u8::from_str_radix(&s[i * 2..i * 2 + 2], 16))
        .collect();
    match bytes {
        Ok(b)  => Ok(ok_vm(VMValue::Bytes(bytes_new(b)))),
        Err(e) => Ok(err_vm(VMValue::Str(format!("Bytes.from_hex: {}", e)))),
    }
}
"Bytes.from_str" => {
    let s = match args.into_iter().next() {
        Some(VMValue::Str(s)) => s,
        _ => return Err(err_vm(VMValue::Str("Bytes.from_str: expected String".into()))),
    };
    Ok(ok_vm(VMValue::Bytes(bytes_new(s.into_bytes()))))
}
"Bytes.len" => {
    let id = match args.into_iter().next() {
        Some(VMValue::Bytes(id)) => id,
        _ => return Err(err_vm(VMValue::Str("Bytes.len: expected Bytes".into()))),
    };
    let len = bytes_get_arc(id).map(|a| a.len()).unwrap_or(0) as i64;
    Ok(VMValue::Int(len))
}
"Bytes.get" => {
    let mut it = args.into_iter();
    let id  = match it.next() { Some(VMValue::Bytes(id)) => id, _ => return Err(err_vm(VMValue::Str("Bytes.get: arg0 not Bytes".into()))) };
    let idx = match it.next() { Some(VMValue::Int(n))    => n,  _ => return Err(err_vm(VMValue::Str("Bytes.get: arg1 not Int".into()))) };
    match bytes_get_arc(id) {
        Some(arc) => {
            let i = idx as usize;
            if i < arc.len() {
                Ok(ok_vm(VMValue::Int(arc[i] as i64)))
            } else {
                Ok(err_vm(VMValue::Str(format!("Bytes.get: index {} out of bounds (len={})", idx, arc.len()))))
            }
        }
        None => Ok(err_vm(VMValue::Str("Bytes.get: invalid Bytes handle".into()))),
    }
}
"Bytes.slice" => {
    let mut it = args.into_iter();
    let id    = match it.next() { Some(VMValue::Bytes(id)) => id, _ => return Err(err_vm(VMValue::Str("Bytes.slice: arg0 not Bytes".into()))) };
    let start = match it.next() { Some(VMValue::Int(n))    => n,  _ => return Err(err_vm(VMValue::Str("Bytes.slice: arg1 not Int".into()))) };
    let end   = match it.next() { Some(VMValue::Int(n))    => n,  _ => return Err(err_vm(VMValue::Str("Bytes.slice: arg2 not Int".into()))) };
    match bytes_get_arc(id) {
        Some(arc) => {
            let len = arc.len();
            let s   = (start as usize).min(len);
            let e   = (end   as usize).min(len).max(s);
            Ok(ok_vm(VMValue::Bytes(bytes_new(arc[s..e].to_vec()))))
        }
        None => Ok(err_vm(VMValue::Str("Bytes.slice: invalid Bytes handle".into()))),
    }
}
"Bytes.concat" => {
    // 注意: concat は常に成功（Bytes を返す。Result ではない）
    let mut it = args.into_iter();
    let id_a = match it.next() { Some(VMValue::Bytes(id)) => id, _ => return Err(err_vm(VMValue::Str("Bytes.concat: arg0 not Bytes".into()))) };
    let id_b = match it.next() { Some(VMValue::Bytes(id)) => id, _ => return Err(err_vm(VMValue::Str("Bytes.concat: arg1 not Bytes".into()))) };
    let a = bytes_get_arc(id_a).unwrap_or_default();
    let b = bytes_get_arc(id_b).unwrap_or_default();
    let mut v = (*a).clone();
    v.extend_from_slice(&b);
    Ok(VMValue::Bytes(bytes_new(v)))
}
"Bytes.to_utf8" => {
    let id = match args.into_iter().next() { Some(VMValue::Bytes(id)) => id, _ => return Err(err_vm(VMValue::Str("Bytes.to_utf8: expected Bytes".into()))) };
    match bytes_get_arc(id) {
        Some(arc) => match std::str::from_utf8(&arc) {
            Ok(s)  => Ok(ok_vm(VMValue::Str(s.to_string()))),
            Err(e) => Ok(err_vm(VMValue::Str(format!("Bytes.to_utf8: {}", e)))),
        },
        None => Ok(err_vm(VMValue::Str("Bytes.to_utf8: invalid handle".into()))),
    }
}
"Bytes.to_hex" => {
    let id = match args.into_iter().next() { Some(VMValue::Bytes(id)) => id, _ => return Err(err_vm(VMValue::Str("Bytes.to_hex: expected Bytes".into()))) };
    let hex = bytes_get_arc(id)
        .map(|arc| arc.iter().map(|b| format!("{:02x}", b)).collect::<String>())
        .unwrap_or_default();
    Ok(VMValue::Str(hex))
}
"Bytes.read_u16" => {
    let mut it = args.into_iter();
    let id  = match it.next() { Some(VMValue::Bytes(id)) => id, _ => return Err(err_vm(VMValue::Str("Bytes.read_u16: arg0 not Bytes".into()))) };
    let off = match it.next() { Some(VMValue::Int(n))    => n as usize, _ => return Err(err_vm(VMValue::Str("Bytes.read_u16: arg1 not Int".into()))) };
    match bytes_get_arc(id) {
        Some(arc) if off + 2 <= arc.len() => {
            let v = (arc[off] as i64) << 8 | arc[off + 1] as i64;
            Ok(ok_vm(VMValue::Int(v)))
        }
        _ => Ok(err_vm(VMValue::Str("Bytes.read_u16: out of bounds".into()))),
    }
}
"Bytes.read_u24" => {
    let mut it = args.into_iter();
    let id  = match it.next() { Some(VMValue::Bytes(id)) => id, _ => return Err(err_vm(VMValue::Str("Bytes.read_u24: arg0 not Bytes".into()))) };
    let off = match it.next() { Some(VMValue::Int(n))    => n as usize, _ => return Err(err_vm(VMValue::Str("Bytes.read_u24: arg1 not Int".into()))) };
    match bytes_get_arc(id) {
        Some(arc) if off + 3 <= arc.len() => {
            let v = (arc[off] as i64) << 16 | (arc[off + 1] as i64) << 8 | arc[off + 2] as i64;
            Ok(ok_vm(VMValue::Int(v)))
        }
        _ => Ok(err_vm(VMValue::Str("Bytes.read_u24: out of bounds".into()))),
    }
}
"Bytes.read_u32" => {
    let mut it = args.into_iter();
    let id  = match it.next() { Some(VMValue::Bytes(id)) => id, _ => return Err(err_vm(VMValue::Str("Bytes.read_u32: arg0 not Bytes".into()))) };
    let off = match it.next() { Some(VMValue::Int(n))    => n as usize, _ => return Err(err_vm(VMValue::Str("Bytes.read_u32: arg1 not Int".into()))) };
    match bytes_get_arc(id) {
        Some(arc) if off + 4 <= arc.len() => {
            let v = (arc[off] as i64) << 24
                  | (arc[off + 1] as i64) << 16
                  | (arc[off + 2] as i64) << 8
                  |  arc[off + 3] as i64;
            Ok(ok_vm(VMValue::Int(v)))
        }
        _ => Ok(err_vm(VMValue::Str("Bytes.read_u32: out of bounds".into()))),
    }
}
// 注意: #[cfg] はマッチアームの属性として使えない（stable Rust 制限）
// → アーム本体内で cfg ブロックを使う
"Bytes.read_file" => {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let path = match args.into_iter().next() {
            Some(VMValue::Str(s)) => s,
            _ => return Err(err_vm(VMValue::Str("Bytes.read_file: expected String".into()))),
        };
        match std::fs::read(&path) {
            Ok(data) => Ok(ok_vm(VMValue::Bytes(bytes_new(data)))),
            Err(e)   => Ok(err_vm(VMValue::Str(format!("Bytes.read_file: {}", e)))),
        }
    }
    #[cfg(target_arch = "wasm32")]
    { Err(err_vm(VMValue::Str("Bytes.read_file: not available on wasm32".into()))) }
}
"Bytes.write_file" => {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let mut it = args.into_iter();
        let path = match it.next() {
            Some(VMValue::Str(s)) => s,
            _ => return Err(err_vm(VMValue::Str("Bytes.write_file: arg0 not String".into()))),
        };
        let id = match it.next() {
            Some(VMValue::Bytes(id)) => id,
            _ => return Err(err_vm(VMValue::Str("Bytes.write_file: arg1 not Bytes".into()))),
        };
        match bytes_get_arc(id) {
            Some(arc) => match std::fs::write(&path, arc.as_ref()) {
                Ok(())  => Ok(ok_vm(VMValue::Unit)),
                Err(e)  => Ok(err_vm(VMValue::Str(format!("Bytes.write_file: {}", e)))),
            },
            None => Ok(err_vm(VMValue::Str("Bytes.write_file: invalid Bytes handle".into()))),
        }
    }
    #[cfg(target_arch = "wasm32")]
    { Err(err_vm(VMValue::Str("Bytes.write_file: not available on wasm32".into()))) }
}
```

> **注意**: `vm_call_builtin` の match アーム追加位置は事前確認で `"State."` / `"Arena."` の終わりを特定してから挿入する。
> `ok_vm` / `err_vm` は既存ヘルパーをそのまま使用。
> `args: Vec<VMValue>` の渡し方は既存 `"ArrowBatch."` ハンドラを参照する。

### T3-6: `is_known_builtin_namespace` に `"Bytes"` 追加

`| "State"   // v22.3.0` の直後:

```rust
| "Bytes"   // v23.1.0
```

---

## T4: `fav/src/middle/checker.rs` — namespace リストに `"Bytes"` 追加

### 事前確認

```bash
grep -n "\"Arena\"\|\"Bytes\"\|\"ArrowBatch\"" fav/src/middle/checker.rs | head -5
```

### T4-1: namespace リストに追加

`"Arena",` の直後に追加:

```rust
"Bytes",
```

---

## T5: `fav/src/middle/compiler.rs` — builtins リストに追加

### 事前確認

```bash
grep -n "Arena\|\"Bytes\"\|ArrowBatch" fav/src/middle/compiler.rs | head -10
```

### T5-1: builtins リストに 13 エントリ追加

`"Arena.stats",` の直後に追加:

```rust
// v23.1.0 Bytes 型（13 関数）
"Bytes.from_hex", "Bytes.from_str",
"Bytes.len", "Bytes.get", "Bytes.slice", "Bytes.concat",
"Bytes.to_utf8", "Bytes.to_hex",
"Bytes.read_file", "Bytes.write_file",
"Bytes.read_u16", "Bytes.read_u24", "Bytes.read_u32",
```

---

## T6: `fav/src/driver.rs` — `v230000_tests` ignore + `v231000_tests` 追加

### 事前確認

```bash
grep -n "fn version_is_23_0_0\|mod v230000_tests\|mod v231000_tests" fav/src/driver.rs | head -5
```

### T6-1: `version_is_23_0_0` に `#[ignore]` 追加

T7-1（Cargo.toml バージョン更新）より前に実施すること。

### T6-2: `v231000_tests` モジュール追加（5 件）

`v230000_tests` の閉じ `}` の直後に追加:

```rust
// ── v231000_tests (v23.1.0) — Bytes 型 ──────────────────────────────────────
// 注意: exec_artifact_main は crate::value::Value を返す（VMValue ではない）
#[cfg(test)]
mod v231000_tests {
    use super::*;

    #[test]
    fn version_is_23_1_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("version = \"23.1.0\""), "Cargo.toml should have version 23.1.0");
    }

    #[test]
    fn bytes_from_hex_to_hex_roundtrip() {
        // Bytes.from_hex("414243") |> Bytes.to_hex == "414243"
        let src = r#"stage ToHex: Bytes -> String = |b| { Bytes.to_hex(b) }
stage FromHex: String -> Bytes = |s| {
  match Bytes.from_hex(s) {
    ok(b) => b
    err(_) => Bytes.from_str("error")
  }
}
seq Roundtrip = FromHex |> ToHex
public fn main() -> String { "414243" |> Roundtrip }
"#;
        let tokens = crate::frontend::lexer::Lexer::new(src, "test.fav")
            .tokenize().expect("lex");
        let prog = crate::frontend::parser::Parser::new(tokens)
            .parse_program().expect("parse");
        let artifact = build_artifact(&prog);
        let result = exec_artifact_main(&artifact, None).expect("exec");
        assert_eq!(result, crate::value::Value::Str("414243".into()),
            "from_hex/to_hex roundtrip should return original hex string");
    }

    #[test]
    fn bytes_get_correct_byte() {
        // Bytes.from_hex("ff00") の get(0) → 255
        let src = r#"stage GetFirst: Bytes -> Int = |b| {
  match Bytes.get(b, 0) {
    ok(n) => n
    err(_) => -1
  }
}
stage MakeBytes: String -> Bytes = |s| {
  match Bytes.from_hex(s) {
    ok(b) => b
    err(_) => Bytes.from_str("")
  }
}
seq GetByte = MakeBytes |> GetFirst
public fn main() -> Int { "ff00" |> GetByte }
"#;
        let tokens = crate::frontend::lexer::Lexer::new(src, "test.fav")
            .tokenize().expect("lex");
        let prog = crate::frontend::parser::Parser::new(tokens)
            .parse_program().expect("parse");
        let artifact = build_artifact(&prog);
        let result = exec_artifact_main(&artifact, None).expect("exec");
        assert_eq!(result, crate::value::Value::Int(255),
            "get(0) of 0xff00 should be 255");
    }

    #[test]
    fn bytes_concat_increases_length() {
        // from_hex("4142") (2 bytes) ++ from_hex("414243") (3 bytes) → len 5
        // 注意: Bytes.concat は Bytes を返す（Result ではない）ため bind <- は使えない
        // → Bytes.len(Bytes.concat(a, b)) でネストして呼ぶ
        let src = r#"stage MakeA: String -> Bytes = |s| {
  match Bytes.from_hex("4142") {
    ok(b) => b
    err(e) => Bytes.from_str("")
  }
}
stage MakeB: String -> Bytes = |s| {
  match Bytes.from_hex("414243") {
    ok(b) => b
    err(e) => Bytes.from_str("")
  }
}
stage ConcatLen: Bytes -> Int = |a| {
  bind b <- MakeB("")
  Bytes.len(Bytes.concat(a, b))
}
seq CheckLen = MakeA |> ConcatLen
public fn main() -> Int { "" |> CheckLen }
"#;
        let tokens = crate::frontend::lexer::Lexer::new(src, "test.fav")
            .tokenize().expect("lex");
        let prog = crate::frontend::parser::Parser::new(tokens)
            .parse_program().expect("parse");
        let artifact = build_artifact(&prog);
        let result = exec_artifact_main(&artifact, None).expect("exec");
        assert_eq!(result, crate::value::Value::Int(5),
            "concat of 2+3 bytes should have length 5");
    }

    #[test]
    fn changelog_has_v23_1_0() {
        let cl = include_str!("../../CHANGELOG.md");
        assert!(cl.contains("[v23.1.0]"), "CHANGELOG should have v23.1.0 entry");
    }
}
```

> **注意**: テストの Favnir コードは `public fn main()` が必要（`exec_artifact_main` の要件）。
> `bind` で他の stage を呼ぶ場合は stage 定義を別途書く（テスト内 Favnir は制限あり）。
> `concat_increases_length` の MakeB 呼び出しは stage 内で直接呼ぶ形に簡略化できる場合はそうする。

---

## T7: バージョン更新・CHANGELOG・benchmarks・MDX

### T7-1: `fav/Cargo.toml` バージョン更新

**注意**: T6-1（`#[ignore]` 追加）を先に実施してから変更する。

```toml
version = "23.1.0"
```

### T7-2: `CHANGELOG.md` 更新

v23.0.0 エントリの直後（現在先頭）に v23.1.0 エントリを追加:

```markdown
## [v23.1.0] — 2026-06-21 — `Bytes` 型

### 追加
- `VMValue::Bytes(u64)` opaque handle（NaN-boxing 準拠）
- `BYTES_STORE` / `NEXT_BYTES_ID` thread-local ストア
- `Bytes.*` 13 関数: `from_hex` / `from_str` / `len` / `get` / `slice` / `concat` /
  `to_utf8` / `to_hex` / `read_file` / `write_file` / `read_u16` / `read_u24` / `read_u32`
- `HeapVal::Bytes(u64)` + NanVal 変換
- checker / compiler の namespace / builtins リストに `Bytes` 追加

### テスト
- `v231000_tests` 5 件追加
```

### T7-3: `benchmarks/v23.1.0.json` 新規作成

```json
{
  "version": "23.1.0",
  "timestamp": "2026-06-21T00:00:00Z",
  "_note": "Bytes type implementation: opaque handle, 13 builtin functions, NaN-boxing compatible.",
  "metrics": {
    "test_count": 1891,
    "bytes_functions": 13,
    "bytes_wasm_functions": 11,
    "bytes_native_only_functions": 2
  },
  "_metrics_notes": {
    "bytes_wasm_functions": "wasm32 で使える 11 関数（read_file / write_file を除く）",
    "bytes_native_only_functions": "read_file / write_file（#[cfg(not(wasm32))]）",
    "test_count": "v23.0.0 完了時（1886）+ v231000_tests 5件"
  },
  "bytes_features": {
    "opaque_handle":  { "achieved": true, "version": "v23.1.0", "note": "NaN-boxing 準拠" },
    "from_hex":       { "achieved": true, "version": "v23.1.0" },
    "to_hex":         { "achieved": true, "version": "v23.1.0" },
    "from_str":       { "achieved": true, "version": "v23.1.0" },
    "to_utf8":        { "achieved": true, "version": "v23.1.0" },
    "get_slice":      { "achieved": true, "version": "v23.1.0" },
    "concat":         { "achieved": true, "version": "v23.1.0" },
    "file_io":        { "achieved": true, "version": "v23.1.0", "note": "native only" },
    "read_u16_24_32": { "achieved": true, "version": "v23.1.0", "note": "vm.fav bytecode decode 用" }
  }
}
```

### T7-4: `site/content/docs/runes/bytes.mdx` 新規作成

以下を含む MDX ドキュメント:
- `Bytes` 型の概要（vm.fav バイトコードデコード向けプリミティブ）
- `from_hex` / `to_hex` の基本例
- `read_u16` / `read_u24` / `read_u32` の使い方（バイトコードデコード例）
- WASM 制限（`read_file` / `write_file` は native のみ）
- 関数一覧表（13 件）

---

## リスクと対策

| リスク | 対策 |
|---|---|
| `vm_call_builtin` の match 構造が `n if n.starts_with("Bytes.")` パターンを受け付けない | 既存の `"ArrowBatch."` ハンドラの match アーム形式を確認してから同じ形で追加 |
| NaN-boxing で `VMValue::Bytes` が `HeapVal` を経由しない | T2 で nan_val.rs の変換を確認。`from_heap(HeapVal::Bytes(id))` を明示的に追加 |
| `Bytes.concat` で `unwrap_or_default()` が空 Vec を返す | handle が無効なケースは実際には発生しないが、エラー返しに変更することも可 |
| テスト内 Favnir コードで `bind` の多段呼び出しが失敗する | stage のシグネチャを `String -> Int` 等に統一し、入力を `""` にして簡略化 |
| `Cargo.toml` 更新より前に `#[ignore]` を忘れる | T6-1 チェック項目に「T7-1 より前」の注意を明記（tasks.md 参照） |
