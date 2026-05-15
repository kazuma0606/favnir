# Favnir v2.2.0 実装プラン

作成日: 2026-05-13

---

## Phase 0 — バージョン更新

```toml
# Cargo.toml
version = "2.2.0"
```

```rust
// src/main.rs
const HELP: &str = "fav v2.2.0 ...";
```

---

## Phase 1 — variant 大文字小文字の正規化

### 1-1. `src/middle/compiler.rs`

`compile_pattern` 関数の `Pattern::Variant` アームに正規化を追加する。

**変更前**:
```rust
Pattern::Variant(name, inner, _) => IRPattern::Variant(
    name.clone(),
    inner.as_ref().map(|p| Box::new(compile_pattern(p, ctx))),
),
```

**変更後**:
```rust
Pattern::Variant(name, inner, _) => {
    // Normalize built-in variant names so that Ok/Err/Some/None
    // match the lowercase tags produced by Result.ok / Result.err /
    // Option.some / Option.none at runtime.
    let normalized = match name.as_str() {
        "Ok"   => "ok",
        "Err"  => "err",
        "Some" => "some",
        "None" => "none",
        other  => other,
    };
    IRPattern::Variant(
        normalized.to_string(),
        inner.as_ref().map(|p| Box::new(compile_pattern(p, ctx))),
    )
}
```

**注意**:
- ユーザー定義 ADT（`Circle`, `Square` 等）は `other` に入るので変換されない
- `Ok` は `ok` に変換されるが、`ok` は `ok` のまま → どちらも動作する
- 既存テストへの影響なし

---

## Phase 2 — pipe match エンドツーエンドテスト

### 2-1. `src/backend/vm_stdlib_tests.rs`

```rust
#[test]
fn test_pipe_match_ok() {
    let src = r#"
public fn main() -> Int {
    bind result <- Result.ok(5)
    result |> match {
        Ok(v)  => v
        Err(_) => 0
    }
}
"#;
    assert_eq!(eval(src), Value::Int(5));
}

#[test]
fn test_pipe_match_err() {
    let src = r#"
public fn main() -> Int {
    bind result <- Result.err("oops")
    result |> match {
        Ok(v)  => v
        Err(_) => -1
    }
}
"#;
    assert_eq!(eval(src), Value::Int(-1));
}

#[test]
fn test_pipe_match_option_some() {
    let src = r#"
public fn main() -> Int {
    bind opt <- Option.some(42)
    opt |> match {
        Some(v) => v
        None    => 0
    }
}
"#;
    assert_eq!(eval(src), Value::Int(42));
}

#[test]
fn test_pipe_match_option_none() {
    let src = r#"
public fn main() -> Int {
    bind opt <- Option.none()
    opt |> match {
        Some(v) => v
        None    => -1
    }
}
"#;
    assert_eq!(eval(src), Value::Int(-1));
}

#[test]
fn test_pipe_match_chained() {
    // パイプラインの途中で |> match を使う
    let src = r#"
fn double(n: Int) -> Int { Result.ok(n * 2) }

public fn main() -> Int {
    double(7) |> match {
        Ok(v)  => v
        Err(_) => 0
    }
}
"#;
    assert_eq!(eval(src), Value::Int(14));
}
```

---

## Phase 3 — pattern guard テスト補完

### 3-1. `src/backend/vm_stdlib_tests.rs`

```rust
#[test]
fn test_pattern_guard_fallthrough() {
    // ガード不成立時に次アームへフォールスルーすることを確認
    let src = r#"
public fn main() -> String {
    match 15 {
        n where n > 20 => "big"
        n where n > 10 => "medium"
        _              => "small"
    }
}
"#;
    assert_eq!(eval(src), Value::Str("medium".into()));
}

#[test]
fn test_pattern_guard_all_fail() {
    let src = r#"
public fn main() -> String {
    match 5 {
        n where n > 20 => "big"
        n where n > 10 => "medium"
        _              => "small"
    }
}
"#;
    assert_eq!(eval(src), Value::Str("small".into()));
}

#[test]
fn test_pattern_guard_record() {
    // ロードマップ完了条件: match u { { age } where age >= 18 => "adult" _ => "minor" }
    let src = r#"
type User = { name: String  age: Int }

public fn main() -> String {
    bind u <- User { name: "Alice"  age: 20 }
    match u {
        { age } where age >= 18 => "adult"
        _                       => "minor"
    }
}
"#;
    assert_eq!(eval(src), Value::Str("adult".into()));
}

#[test]
fn test_pattern_guard_record_minor() {
    let src = r#"
type User = { name: String  age: Int }

public fn main() -> String {
    bind u <- User { name: "Bob"  age: 15 }
    match u {
        { age } where age >= 18 => "adult"
        _                       => "minor"
    }
}
"#;
    assert_eq!(eval(src), Value::Str("minor".into()));
}

#[test]
fn test_pattern_guard_compound_and() {
    // ガード内で v2.1.0 の && を使う
    let src = r#"
public fn main() -> String {
    match 25 {
        n where n >= 18 && n < 65 => "working-age"
        n where n >= 65           => "senior"
        _                         => "youth"
    }
}
"#;
    assert_eq!(eval(src), Value::Str("working-age".into()));
}
```

### 3-2. `src/middle/checker.rs`（E027 追加テスト）

```rust
#[test]
fn test_guard_non_bool_compound() {
    // ガードが整数式 (Bool でない) → E027
    let errs = check_err("fn f(x: Int) -> Int { match x { n where n + 1 => n _ => 0 } }");
    assert!(errs.iter().any(|e| e.contains("E027")));
}
```

---

## Phase 4 — テスト・ドキュメント

### 4-1. テスト一覧（Phase 1-3 合計）

| テスト名 | 場所 | 期待値 |
|---|---|---|
| `test_pipe_match_ok` | vm_stdlib_tests | `Int(5)` |
| `test_pipe_match_err` | vm_stdlib_tests | `Int(-1)` |
| `test_pipe_match_option_some` | vm_stdlib_tests | `Int(42)` |
| `test_pipe_match_option_none` | vm_stdlib_tests | `Int(-1)` |
| `test_pipe_match_chained` | vm_stdlib_tests | `Int(14)` |
| `test_pattern_guard_fallthrough` | vm_stdlib_tests | `Str("medium")` |
| `test_pattern_guard_all_fail` | vm_stdlib_tests | `Str("small")` |
| `test_pattern_guard_record` | vm_stdlib_tests | `Str("adult")` |
| `test_pattern_guard_record_minor` | vm_stdlib_tests | `Str("minor")` |
| `test_pattern_guard_compound_and` | vm_stdlib_tests | `Str("working-age")` |
| `test_guard_non_bool_compound` | checker | E027 |

### 4-2. ドキュメント

- `versions/v2.2.0/langspec.md` — v2.2.0 言語仕様書
  - `|> match {}` 構文の説明
  - `where` ガードの説明と優先順位
  - E027 エラーコードの説明
  - `Ok` / `Err` / `Some` / `None` パターン名の大文字表記サポートの記載

---

## 変更ファイル一覧

| ファイル | 変更内容 |
|---|---|
| `Cargo.toml` | `version = "2.2.0"` |
| `src/main.rs` | HELP テキスト `v2.2.0` |
| `src/middle/compiler.rs` | `compile_pattern`: Ok/Err/Some/None を正規化 |
| `src/backend/vm_stdlib_tests.rs` | pipe match テスト 5 件、pattern guard テスト 5 件追加 |
| `src/middle/checker.rs` | E027 追加テスト 1 件 |
| `versions/v2.2.0/langspec.md` | NEW: v2.2.0 言語仕様書 |

---

## 実装上の注意事項

### variant 正規化の範囲

`Ok` / `Err` / `Some` / `None` のみ正規化する。
他の大文字始まりの名前（ユーザー定義 ADT、将来の組み込み型）は変換しない。

将来 `Result` / `Option` 以外の組み込み型が variant を持つ場合は、
このテーブルを拡張する。

### `None` パターン（ペイロードなし）

`None` は `Pattern::Variant("None", None)` として解析される（ペイロードなし）。
正規化で `"none"` に変換され、`VMValue::Variant("none", None)` と一致する。

### 後方互換性の確認

```
ok(v) / Ok(v) → 両方 "ok" に解決 → 動作する
none / None   → 両方 "none" に解決 → 動作する
Circle(r)     → 変換なし → 動作する（ユーザー ADT）
```

既存テスト `test_match_variant_with_payload`（`Circle` / `Square`）は変化なし。
