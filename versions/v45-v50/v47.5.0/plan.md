# Plan: v47.5.0 — `Float` / `Int` 拡充

## 実装ステップ

### Step 1: `vm.rs` に 5 primitive 追加

挿入位置: `"Float.to_bits"` アーム（line 9563）の直前。

```rust
// -- v47.5.0: Float.round / Float.clamp / Float.abs / Int.to_hex / Int.abs --
"Float.round" => {
    let mut it = args.into_iter();
    let v = it.next().ok_or_else(|| "Float.round requires 2 arguments".to_string())?;
    let n = it.next().ok_or_else(|| "Float.round requires 2 arguments".to_string())?;
    match (v, n) {
        (VMValue::Float(f), VMValue::Int(n)) => {
            let factor = 10_f64.powi(n as i32);
            Ok(VMValue::Float((f * factor).round() / factor))
        }
        _ => Err("Float.round requires (Float, Int) arguments".to_string()),
    }
}
"Float.clamp" => {
    let mut it = args.into_iter();
    let v  = it.next().ok_or_else(|| "Float.clamp requires 3 arguments".to_string())?;
    let lo = it.next().ok_or_else(|| "Float.clamp requires 3 arguments".to_string())?;
    let hi = it.next().ok_or_else(|| "Float.clamp requires 3 arguments".to_string())?;
    match (v, lo, hi) {
        (VMValue::Float(f), VMValue::Float(lo), VMValue::Float(hi)) => {
            Ok(VMValue::Float(f.clamp(lo, hi)))
        }
        _ => Err("Float.clamp requires (Float, Float, Float) arguments".to_string()),
    }
}
"Float.abs" => {
    let v = args.into_iter().next()
        .ok_or_else(|| "Float.abs requires 1 argument".to_string())?;
    match v {
        VMValue::Float(f) => Ok(VMValue::Float(f.abs())),
        _ => Err("Float.abs requires a Float argument".to_string()),
    }
}
"Int.to_hex" => {
    let v = args.into_iter().next()
        .ok_or_else(|| "Int.to_hex requires 1 argument".to_string())?;
    match v {
        VMValue::Int(n) => Ok(VMValue::Str(format!("{:x}", n))),
        _ => Err("Int.to_hex requires an Int argument".to_string()),
    }
}
"Int.abs" => {
    let v = args.into_iter().next()
        .ok_or_else(|| "Int.abs requires 1 argument".to_string())?;
    match v {
        VMValue::Int(n) => Ok(VMValue::Int(n.abs())),
        _ => Err("Int.abs requires an Int argument".to_string()),
    }
}
```

### Step 2: `checker.rs` に型シグネチャ追加

挿入位置: `("Int", "bit_not")` / `("Int", "shift_right")` アーム（line 5831）の直後。

```rust
// Float/Int 拡充 v47.5.0
("Float", "round") => Some(Type::Float),
("Float", "clamp") | ("Float", "abs") => Some(Type::Float),
("Int", "to_hex") => Some(Type::String),
("Int", "abs") => Some(Type::Int),
```

### Step 3: `driver.rs` に `v475000_tests` 追加

挿入位置: `v474000_tests` モジュールの直前。

```rust
// -- v475000_tests (v47.5.0) -- Float/Int 拡充: float_round / float_clamp / int_to_hex --
#[cfg(test)]
mod v475000_tests {
    use crate::frontend::parser::Parser;
    use super::{build_artifact, exec_artifact_main};

    #[test]
    fn float_round() {
        // Float.round(3.14159, 2) = 3.14
        let src = r#"
fn main() -> Bool {
  bind result <- Float.round(3.14159, 2)
  result == 3.14
}
"#;
        let program = Parser::parse_str(src, "float_round_test.fav").expect("parse");
        let artifact = build_artifact(&program);
        let value = exec_artifact_main(&artifact, None).expect("exec");
        assert_eq!(value, crate::value::Value::Bool(true));
    }

    #[test]
    fn float_clamp() {
        // Float.clamp(150.0, 0.0, 100.0) = 100.0
        let src = r#"
fn main() -> Bool {
  bind result <- Float.clamp(150.0, 0.0, 100.0)
  result == 100.0
}
"#;
        let program = Parser::parse_str(src, "float_clamp_test.fav").expect("parse");
        let artifact = build_artifact(&program);
        let value = exec_artifact_main(&artifact, None).expect("exec");
        assert_eq!(value, crate::value::Value::Bool(true));
    }

    #[test]
    fn int_to_hex() {
        // Int.to_hex(255) = "ff"
        let src = r#"
fn main() -> Bool {
  bind result <- Int.to_hex(255)
  result == "ff"
}
"#;
        let program = Parser::parse_str(src, "int_to_hex_test.fav").expect("parse");
        let artifact = build_artifact(&program);
        let value = exec_artifact_main(&artifact, None).expect("exec");
        assert_eq!(value, crate::value::Value::Bool(true));
    }
}
```

### Step 4: `Cargo.toml` バージョン更新

```toml
version = "47.5.0"
```

### Step 5: `CHANGELOG.md` 更新

```markdown
## [v47.5.0] — 2026-07-17

### Added
- `Float.round` / `Float.clamp` / `Float.abs` / `Int.to_hex` / `Int.abs` VM primitive 追加（vm.rs / checker.rs）
- `driver.rs`: `v475000_tests` 追加（`float_round` / `float_clamp` / `int_to_hex` 3テスト）
```

### Step 6: `versions/current.md` 更新

- 最新安定版を `v47.5.0`（3030 tests）に更新
- 進行中バージョン・次に切る版を `v47.6.0` に更新

---

## 注意事項

### `Float.round` の精度

`(3.14159_f64 * 100.0).round() / 100.0` は Rust では `3.14` と同じ f64 ビットパターンになるため `== 3.14` で比較可能。

### テスト数

| バージョン | テスト数 | 差分 |
|---|---|---|
| v47.4.0 | 3027 | ベース |
| v47.5.0 | 3030 | +3 |
