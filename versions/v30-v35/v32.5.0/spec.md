# v32.5.0 — Spec: 線形型 確認・テスト補強

## 概要

v32.5.0 は **線形型（Linear Types）** の確認・テスト補強バージョン。

ロードマップ v32.5 候補「線形型の実用化（`-o` のコンパイラ強制）」に対応する。
実際にはすでに v18.5.0 で実装済みである:

| コンポーネント | 実装済み | バージョン |
|---|---|---|
| `TokenKind::LinearArrow` — `-o` トークン | ✓ | v18.5.0 |
| `TypeExpr::LinearArrow` — 線形関数型 AST ノード | ✓ | v18.5.0 |
| E0332 — linear variable used more than once | ✓ | v18.5.0 |
| E0333 — linear variable never used | ✓ | v18.5.0 |
| `v185000_tests` — 4 件のテスト（lex/parse/E0332/E0333） | ✓ | v18.5.0 |

v32.5.0 では、線形型の制約チェック（E0332 / E0333）が仕様通りに動作することを
`v325000_tests` で明示的に確認し、バージョンと CHANGELOG を更新する。

---

## 線形型仕様確認

### 概念

線形型（Linear Types）は「リソースをちょうど 1 回だけ使うことを型レベルで保証する」機能。

```favnir
// Connection は線形リソース — 一度 consume したら再利用不可
fn use_conn(c: Connection) -> String {
    "ok"
}
```

### エラーコード

| エラー | 条件 | コード |
|---|---|---|
| E0332 | 線形変数を 2 回以上使用 | `"linear variable used more than once"` |
| E0333 | 線形変数を bind して使わずに捨てる | `"linear variable never used"` |

---

## 追加するテスト（v325000_tests — 4 件）

`v325000_tests` は v32.1.0〜v32.3.0 と同じパターン:
- `use super::*` **なし**
- モジュール内に独自の `check_errors` を定義
- `use crate::frontend::parser::Parser; use crate::middle::checker::Checker;`

テスト名は v185000_tests（`linear_double_use_is_e0332` / `linear_unused_is_e0333`）と
被らないよう `linear_type_` プレフィックスを使用する。

### テスト 1: バージョン確認

```rust
fn cargo_toml_version_is_32_5_0() {
    let src = include_str!("../Cargo.toml");
    assert!(src.contains("32.5.0"), "Cargo.toml must contain '32.5.0'");
}
```

### テスト 2: ベンチマーク存在確認

```rust
fn benchmark_v32_5_0_exists() {
    let src = include_str!("../../benchmarks/v32.5.0.json");
    assert!(src.contains("32.5.0"), "benchmarks/v32.5.0.json must contain '32.5.0'");
}
```

### テスト 3: 線形変数の二重使用 → E0332

```rust
fn linear_type_double_use_e0332() {
    // Connection を 2 回 consume → E0332
    let errors = check_errors(r#"
fn open_conn() -> Connection {
    Connection
}
fn consume(c: Connection) -> String { "ok" }
fn use_twice() -> String {
    bind c <- open_conn()
    bind _a <- consume(c)
    bind _b <- consume(c)
    "done"
}
"#);
    assert!(
        errors.iter().any(|e| e == "E0332"),
        "Expected E0332 for double use of linear variable, got: {:?}",
        errors
    );
}
```

### テスト 4: 線形変数の未使用 → E0333

```rust
fn linear_type_unused_var_e0333() {
    // Connection を bind して一度も使わない → E0333
    let errors = check_errors(r#"
fn open_conn() -> Connection {
    Connection
}
fn forget_conn() -> String {
    bind _c <- open_conn()
    "done"
}
"#);
    assert!(
        errors.iter().any(|e| e == "E0333"),
        "Expected E0333 for unused linear variable, got: {:?}",
        errors
    );
}
```

---

## テストモジュールの配置

`v325000_tests` は `v324000_tests` の閉じ括弧（`}`）の直後、
かつ `// ── v31.7.0 tests` コメントの前に挿入する。

---

## 完了条件

- `Cargo.toml` version = `"32.5.0"`
- `cargo_toml_version_is_32_4_0` が空スタブになっていること
- `linear_type_double_use_e0332` テストが PASS（E0332 を確認）
- `linear_type_unused_var_e0333` テストが PASS（E0333 を確認）
- `cargo test --bin fav v325000` — 4/4 PASS
- `cargo test` — 全件 PASS（2476 件、0 failures）
- `CHANGELOG.md` に `[v32.5.0]` セクション
- `benchmarks/v32.5.0.json` 存在かつ `tests_passed` が実測値
- `benchmarks/v32.5.0.json` の `milestone` フィールドが `"Language Power"` であること
- `versions/current.md` を v32.5.0 に更新
- `tasks.md` がすべて `[x]` で COMPLETE に更新されていること
- site/ MDX 更新: 対象外（`linear-types.mdx` 等は既に完成）
