# v41.5.0 Spec — Row polymorphism 強化

**バージョン**: v41.5.0
**テーマ**: `{ ..u, active: true }` レコードスプレッドを checker.fav に統合
**前バージョン**: v41.4.0（ガード付き match）
**目標テスト数**: 2862（前バージョン 2859 + 3）

---

## 概要

現状 `RecordSpread` は `ast_lower_checker.rs` で `sv("()")` という文字列値に
lowering されており、checker.fav の `infer_expr` で `_ => Result.err("unknown expression type")`
に落ちるバグがある。

v41.5.0 では以下 2 点を修正する:

1. **`ERecordSpread` バリアント追加**: `RecordSpread` を正しく checker.fav に渡す
2. **`TeRecord` バリアント追加**: `{ name: String }` 型アノテーションに専用の TypeExpr ノードを付与する

型推論の精度向上（フィールドセットの追跡）は v42.0 以降のスコープとし、
本バージョンでは `ERecordSpread` の型推論結果を `"Unknown"` として返すことで
`types_compatible` のルール（`"Unknown"` は任意の型と互換）を活用し、
既存の型チェックフローを壊さない。

### v41.5.0 スコープ

| 変更 | 内容 |
|---|---|
| `ast_lower_checker.rs` | `RecordSpread` → `v2("ERecordSpread", base, fields)` に修正（`sv("()")` バグ解消） |
| `ast_lower_checker.rs` | `RecordType(_, _)` → `v0("TeRecord")` に変更（`TeSimple("Any")` から精緻化） |
| `checker.fav` | `Expr` 型に `ERecordSpread(Expr, Expr)` 追加 |
| `checker.fav` | `TypeExpr` 型に `TeRecord` 追加 |
| `checker.fav` | `infer_expr` に `ERecordSpread` ケース追加 → `"Unknown"` を返す |
| `checker.fav` | `type_expr_to_str` に `TeRecord => "Any"` 追加（互換性維持） |
| `checker.fav` | `collect_type_vars_from_te` に `TeRecord => List.empty()` 追加 |
| `driver.rs` | `v41400_tests::cargo_toml_version_is_41_4_0` スタブ化、`v41500_tests` 追加（3件） |
| `Cargo.toml` | `version = "41.5.0"` |
| `CHANGELOG.md` | `[v41.5.0]` エントリ追加 |

---

## 変更ファイル一覧

| ファイル | 変更内容 |
|---|---|
| `fav/src/middle/ast_lower_checker.rs` | `RecordSpread` lowering 修正 + `RecordType` lowering 変更 |
| `fav/self/checker.fav` | `ERecordSpread` / `TeRecord` バリアント追加 + 型推論・型変換関数更新 |
| `fav/src/driver.rs` | `v41400_tests` スタブ化 + `v41500_tests` 3 件追加 |
| `fav/Cargo.toml` | `version = "41.5.0"` |
| `CHANGELOG.md` | `[v41.5.0]` エントリ |

---

## 詳細仕様

### 1. ast_lower_checker.rs — RecordSpread lowering 修正

**変更前（バグ）:**
```rust
ast::Expr::RecordSpread(_, _, _) => {
    // record spread not yet supported in checker.fav path — treat as unit
    sv("()")
}
```

**変更後:**
```rust
ast::Expr::RecordSpread(base, updates, _) => {
    // v41.5.0: lower to ERecordSpread(base, fields)
    v2("ERecordSpread", lower_expr(base), lower_field_list(updates))
}
```

---

### 2. ast_lower_checker.rs — RecordType lowering 変更

`lower_te` 関数の `RecordType` アーム（参考: 153行付近）:

**変更前:**
```rust
ast::TypeExpr::RecordType(_, _) => v1("TeSimple", sv("Any")),
```

**変更後:**
```rust
ast::TypeExpr::RecordType(_, _) => v0("TeRecord"),
```

また `te_to_string` ヘルパー（参考: 184行付近）も変更:

**変更前:**
```rust
ast::TypeExpr::RecordType(_, _) => "Any".to_string(),
```

**変更後:**
```rust
ast::TypeExpr::RecordType(_, _) => "Any".to_string(), // unchanged: TeRecord → "Any" via type_expr_to_str
```

**注意**: `te_to_string` は `lower_te` とは別の Rust-side 文字列変換。`TypeExpr::RecordType` → `"Any"` の Rust 側文字列表現は変更しない（動作に影響する箇所のため）。

---

### 3. checker.fav — ERecordSpread バリアント追加

`ERecordLit(String, Expr)` の直後に追加:

```favnir
type Expr =
    ...
    | ERecordLit(String, Expr)
    | ERecordSpread(Expr, Expr)  // v41.5.0: (base_expr, field_list)
    ...
```

---

### 4. checker.fav — TeRecord バリアント追加

`TeFn(TypeExpr, TypeExpr)` の直後に追加:

```favnir
type TypeExpr =
    | TeSimple(String)
    | TeList(TypeExpr)
    | TeOption(TypeExpr)
    | TeResult(TypeExpr, TypeExpr)
    | TeMap(TypeExpr, TypeExpr)
    | TeFn(TypeExpr, TypeExpr)
    | TeRecord          // v41.5.0: { field: Type } 型アノテーション（末尾に追加）
```

**注意**: 実際の `checker.fav` の `TypeExpr` は上記 6 バリアント（`TeSimple`〜`TeFn`）のみ。
`TeIntersection` は存在しない（spec-reviewer 指摘 [HIGH] 対応）。`TeRecord` は末尾に追加する。

---

### 5. checker.fav — infer_expr に ERecordSpread ケース追加

`ERecordLit` ケースの直後に追加:

```favnir
ERecordSpread({ _0: base, _1: fields }) =>
    // v41.5.0: 型精度は "Unknown" で近似（フィールド追跡は v42.0+ 以降）
    Result.ok("Unknown")
```

---

### 6. checker.fav — type_expr_to_str に TeRecord ケース追加

`TeFn` ケースの直後に追加:

```favnir
TeRecord => "Any"   // v41.5.0: 既存 TeSimple("Any") と同じ文字列表現で互換性維持
```

---

### 7. checker.fav — collect_type_vars_from_te に TeRecord ケース追加

`TeMap` ケースの直後に追加:

```favnir
TeRecord => List.empty()
```

---

### 8. テスト設計（v41500_tests）

#### T1: `cargo_toml_version_is_41_5_0`
```rust
#[test]
fn cargo_toml_version_is_41_5_0() {
    // NOTE: この assert は次バージョン bump 時にスタブ化すること
    let cargo = include_str!("../Cargo.toml");
    assert!(cargo.contains("41.5.0"), "Cargo.toml must contain version 41.5.0");
}
```

#### T2: `changelog_has_v41_5_0`
```rust
#[test]
fn changelog_has_v41_5_0() {
    let src = include_str!("../../CHANGELOG.md");
    assert!(src.contains("[v41.5.0]"), "CHANGELOG.md must contain [v41.5.0]");
}
```

#### T3: `record_spread_parseable`
```rust
#[test]
fn record_spread_parseable() {
    use crate::frontend::parser::Parser;
    let src = "fn extend_user(u: String) -> String { { ..u, active: true } }";
    let result = Parser::parse_str(src, "test.fav");
    assert!(result.is_ok(), "Record spread should parse: {:?}", result.err());
}
```

**注**: RecordSpread 構文 `{ ..u, active: true }` を含む関数をパースすることで、
ast_lower_checker.rs の `sv("()")` バグ修正後も lowering パスに入れることを確認する。

---

## 完了条件

- `cargo test` が 2862 tests passed, 0 failed
- `v41500_tests` 3 件すべて pass
- `RecordSpread` を含むコードが `fav check` で `"unknown expression type"` エラーを出さない
- `{ ..u, active: true }` が checker.fav で `"Unknown"` 型として処理される（エラーなし）
- `TeRecord` が `TypeExpr` 型に追加されている
- `type_expr_to_str(TeRecord)` が `"Any"` を返す（互換性維持）

---

## 設計ノート

- `sv("()")` → `v2("ERecordSpread", ...)` への変更は **バグ修正**。`sv("()")` は Rust `Value::Str("()")` なので checker.fav の pattern match で Expr バリアントとして認識されず `Result.err("unknown expression type")` を出す
- `infer_hm`（1932行）の `_ => Result.and_then(infer_expr(expr, env), ...)` フォールスルーにより `ERecordSpread` は `infer_expr` で処理される。`infer_expr` の `ERecordSpread` ケースは `ECollect` の後（1877行付近）、`_ => Result.err(...)` の前に追加すること（`ERecordLit` の直後でも可）
- `type_expr_to_str` と `collect_type_vars_from_te` には **`_ =>` catch-all が存在しない**。`TeRecord` バリアント追加後はこれら 2 関数へのケース追加が必須（T6/T7）。漏れると Favnir 実行時に pattern match 失敗でエラーになる
- `types_compatible("Unknown", "Any")` → `inferred == "Unknown"` の分岐で `true` → 型エラーにならない
- `TeRecord` → `"Any"` の文字列変換は意図的。将来フィールド型追跡を実装する際に `TeRecord` ノードを拡張する（`TeRecord(List<(String, TypeExpr)>)` 等）
- `collect_type_vars_from_te` では `TeRecord` にフィールドがない（`v0` のため）ので `List.empty()` が正しい
- `te_to_string`（Rust side）の `RecordType` → `"Any"` は変更しない（`check_body_ty` の declared 型比較で使われる Rust パス）
