# v32.2.0 — Spec: 行多相 Row Polymorphism 確認・テスト補強

## 概要

v32.2.0 は **行多相（Row Polymorphism）** の確認・テスト補強バージョン。

ロードマップ v32.2 では以下の実装を目標としていたが、実際にはすでに v18.2.0 で実装済みである:

| コンポーネント | 実装済み | バージョン |
|---|---|---|
| `TypeConstraint::HasField { name, ty }` | ✓ | v18.2.0 |
| `parse_type_bounds` — `with { field: Type, ... }` 解析 | ✓ | v18.2.0 |
| `type_has_field` — レコード型のフィールド存在チェック | ✓ | v18.2.0 |
| E0337 — row constraint violated（フィールドなし違反） | ✓ | v18.2.0 |
| `v182000_tests` — 4 件のテスト（pass/fail 両方） | ✓ | v18.2.0 |

v32.2.0 では、`with { field: Type }` 構文が仕様通りに動作することを v322000_tests で
明示的に確認し、バージョンと CHANGELOG を更新して Language Power フェーズの記録を残す。

---

## 行多相仕様確認

### 構文

```favnir
// 「id: Int フィールドを持つ任意のレコード型 R」を受け取れる
fn get_id<R with { id: Int }>(row: R) -> Int {
    row.id
}
```

### `TypeConstraint::HasField` の実装（ast.rs）

```rust
pub enum TypeConstraint {
    Interface(String),                           // `with Ord`
    HasField { name: String, ty: TypeExpr },     // `with { id: Int }`  (v18.2.0)
}
```

### チェッカー（checker.rs）

| 動作 | エラーコード |
|---|---|
| 呼び出し型がフィールドを持つ → OK | — |
| 呼び出し型がフィールドを持たない → E0337 | `"row constraint violated: type ... does not have field ..."` |

`type_has_field`（checker.rs:7917）は `checker.record_fields` から Named 型のフィールドリストを参照する。

---

## 追加するテスト（v322000_tests — 4 件）

`v322000_tests` は `v321000_tests` と同じパターン:
- `use super::*` **なし**
- モジュール内に独自の `check_errors` を定義
- `use crate::frontend::parser::Parser; use crate::middle::checker::Checker;`

### テスト 1: バージョン確認

```rust
fn cargo_toml_version_is_32_2_0() {
    let src = include_str!("../Cargo.toml");
    assert!(src.contains("32.2.0"), "Cargo.toml must contain '32.2.0'");
}
```

### テスト 2: ベンチマーク存在確認

```rust
fn benchmark_v32_2_0_exists() {
    let src = include_str!("../../benchmarks/v32.2.0.json");
    assert!(src.contains("32.2.0"), "benchmarks/v32.2.0.json must contain '32.2.0'");
}
```

### テスト 3: 行多相 — フィールド制約 PASS（ポジティブ）

```rust
fn row_poly_field_constraint_pass() {
    // R with { id: Int } に id: Int を持つ型を渡す → エラーなし
    let errors = check_errors(r#"
type UserRow = { id: Int, name: String }
fn get_id<R with { id: Int }>(row: R) -> Int {
    row.id
}
fn main() -> Int {
    get_id(UserRow { id: 1, name: "Alice" })
}
"#);
    assert!(
        errors.is_empty(),
        "row_poly should pass when field is present: {:?}",
        errors
    );
}
```

### テスト 4: 行多相 — フィールドなし E0337（ネガティブ）

```rust
fn row_poly_missing_field_e0337() {
    // R with { id: Int } に id フィールドを持たない型を渡す → E0337
    let errors = check_errors(r#"
type NoId = { name: String }
fn get_id<R with { id: Int }>(row: R) -> Int {
    row.id
}
fn main() -> Int {
    get_id(NoId { name: "no id here" })
}
"#);
    assert!(
        errors.iter().any(|e| e == "E0337"),
        "Expected E0337 for missing field, got: {:?}",
        errors
    );
}
```

---

## テストモジュールの配置

`v322000_tests` は `v321000_tests` の閉じ括弧（`}`）の直後、
かつ `// ── v31.7.0 tests` コメントの前に挿入する。

```
// ...v321000_tests 閉じ }

// ── v32.2.0 tests ────────────────────────────────────────────────────────────
#[cfg(test)]
mod v322000_tests {
    ...
}

// ── v31.7.0 tests ────────────────────────────────────────────────────────────
#[cfg(test)]
mod v317000_tests {
```

---

## 完了条件

- `Cargo.toml` version = `"32.2.0"`
- `cargo_toml_version_is_32_1_0` が空スタブになっていること
- `row_poly_field_constraint_pass` テストが PASS
- `row_poly_missing_field_e0337` テストが PASS（E0337 を確認）
- `cargo test --bin fav v322000` — 4/4 PASS
- `cargo test` — 全件 PASS（0 failures）
- `CHANGELOG.md` に `[v32.2.0]` セクション
- `benchmarks/v32.2.0.json` 存在かつ `tests_passed` が実測値
- `versions/current.md` を v32.2.0 に更新
- `benchmarks/v32.2.0.json` の `milestone` フィールドが `"Language Power"` であること
- `tasks.md` がすべて `[x]` で COMPLETE に更新されていること
- site/ MDX 更新: 対象外（`row-polymorphism.mdx` 等は既に完成）
