# v32.3.0 — Spec: where 制約（関数引数）確認・テスト補強

## 概要

v32.3.0 は **where 制約（Refinement Types）** の確認・テスト補強バージョン。

ロードマップ v32.3 では以下の実装を目標としていたが、実際にはすでに v18.3.0 で実装済みである:

| コンポーネント | 実装済み | バージョン |
|---|---|---|
| `fn f(x: Int where { x > 0 })` — 引数 where 構文パース | ✓ | v18.3.0 |
| `fn_refinement_registry` — 関数ごとの制約登録 | ✓ | v18.3.0 |
| E0331 — refinement violated（コンパイル時・リテラル検査） | ✓ | v18.3.0 |
| `RefinementAssert` opcode — 実行時制約チェック（変数の場合） | ✓ | v18.3.0 |
| `v183000_tests` — 4 件のテスト（pass/fail/runtime/range） | ✓ | v18.3.0 |

v32.3.0 では、`where` 制約が仕様通りに動作することを `v323000_tests` で
明示的に確認し、バージョンと CHANGELOG を更新して Language Power フェーズの記録を残す。

---

## where 制約仕様確認

### 構文

```favnir
// 引数に事前条件を付ける
fn divide(a: Int, b: Int where { b != 0 }) -> Int {
    a / b
}

// 複合制約
fn set_age(age: Int where { age >= 0 && age <= 150 }) -> Int {
    age
}
```

### チェッカー（checker.rs）

| ケース | 動作 | エラーコード |
|---|---|---|
| リテラル引数が制約を満たす | コンパイル時 OK | — |
| リテラル引数が制約を違反 | コンパイル時エラー | E0331 |
| 変数引数（静的評価不可） | `RefinementAssert` opcode を挿入（実行時チェック） | — |

`fn_refinement_registry`（checker.rs:982）が `fn` 宣言時に制約を登録し、
呼び出し時（checker.rs:4820）にリテラル評価を試みて E0331 を発行する。

---

## 追加するテスト（v323000_tests — 4 件）

`v323000_tests` は `v321000_tests` / `v322000_tests` と同じパターン:
- `use super::*` **なし**
- モジュール内に独自の `check_errors` を定義
- `use crate::frontend::parser::Parser; use crate::middle::checker::Checker;`

### テスト 1: バージョン確認

```rust
fn cargo_toml_version_is_32_3_0() {
    let src = include_str!("../Cargo.toml");
    assert!(src.contains("32.3.0"), "Cargo.toml must contain '32.3.0'");
}
```

### テスト 2: ベンチマーク存在確認

```rust
fn benchmark_v32_3_0_exists() {
    let src = include_str!("../../benchmarks/v32.3.0.json");
    assert!(src.contains("32.3.0"), "benchmarks/v32.3.0.json must contain '32.3.0'");
}
```

### テスト 3: where 制約 PASS（リテラルが制約を満たす）

```rust
fn where_constraint_literal_pass() {
    let errors = check_errors(r#"
fn divide(a: Int, b: Int where { b != 0 }) -> Int {
    a / b
}
fn main() -> Int {
    divide(10, 2)
}
"#);
    assert!(
        errors.iter().all(|e| e != "E0331"),
        "where constraint should pass for b=2: {:?}",
        errors
    );
}
```

### テスト 4: where 制約 E0331（リテラルが制約を違反）

```rust
fn where_constraint_literal_fail_e0331() {
    let errors = check_errors(r#"
fn divide(a: Int, b: Int where { b != 0 }) -> Int {
    a / b
}
fn main() -> Int {
    divide(10, 0)
}
"#);
    assert!(
        errors.iter().any(|e| e == "E0331"),
        "Expected E0331 for b=0 violating b != 0, got: {:?}",
        errors
    );
}
```

---

## テストモジュールの配置

`v323000_tests` は `v322000_tests` の閉じ括弧（`}`）の直後、
かつ `// ── v31.7.0 tests` コメントの前に挿入する。

```
// ...v322000_tests 閉じ }

// ── v32.3.0 tests ────────────────────────────────────────────────────────────
#[cfg(test)]
mod v323000_tests {
    ...
}

// ── v31.7.0 tests ────────────────────────────────────────────────────────────
#[cfg(test)]
mod v317000_tests {
```

---

## 完了条件

- `Cargo.toml` version = `"32.3.0"`
- `cargo_toml_version_is_32_2_0` が空スタブになっていること
- `where_constraint_literal_pass` テストが PASS
- `where_constraint_literal_fail_e0331` テストが PASS（E0331 を確認）
- `cargo test --bin fav v323000` — 4/4 PASS
- `cargo test` — 全件 PASS（0 failures）
- `CHANGELOG.md` に `[v32.3.0]` セクション
- `benchmarks/v32.3.0.json` 存在かつ `tests_passed` が実測値
- `benchmarks/v32.3.0.json` の `milestone` フィールドが `"Language Power"` であること
- `versions/current.md` を v32.3.0 に更新
- `tasks.md` がすべて `[x]` で COMPLETE に更新されていること
- site/ MDX 更新: 対象外（`refinement-types.mdx` 等は既に完成）
