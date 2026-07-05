# v32.1.0 — Spec: 境界付きジェネリクス T with Ord

## 概要

v32.1.0 は **境界付きジェネリクス（bounded generics）** の確認・テスト補強バージョン。

ロードマップ v32.1 では以下の実装を目標としていたが、実際にはすでに v17.1.0〜v18.x で実装済みである:

| コンポーネント | 実装済み | バージョン |
|---|---|---|
| `GenericParam.bounds: Vec<TypeConstraint>` | ✓ | v17.1.0 |
| `parse_type_params` — `with Ord` / `with Eq` 解析 | ✓ | v17.1.0 |
| `fn_bounds_registry` + `type_implements_bound` + E0325 | ✓ | v17.1.0 |
| 組み込み 4 Interface（Ord / Eq / Display / Hash） | ✓ | v17.1.0 |
| ドキュメント（`site/content/docs/language/generics.mdx`） | ✓ | 既存（更新不要） |

v32.1.0 では、組み込み 4 Interface が仕様通りに動作することを v321000_tests で明示的に確認し、
バージョンと CHANGELOG を更新して Language Power フェーズの起点を記録する。

---

## 組み込み Interface 仕様確認

`checker.rs` の `type_implements_bound` 関数が以下を実装している:

| Interface | 満たす型 | `type_implements_bound` の実装 |
|---|---|---|
| `Ord` | Int / Float / String | `matches!(ty, Type::Int \| Type::Float \| Type::String)` |
| `Eq` | 全型（全プリミティブ） | `true`（常に OK） |
| `Display` | String / Int / Float / Bool | `matches!(ty, Type::String \| Type::Int \| Type::Float \| Type::Bool)` |
| `Hash` | Int / String | `matches!(ty, Type::Int \| Type::String)` |

Bool は `Ord` を満たさないため、`fn max<T with Ord>(...)` に `Bool` を渡すと E0325 が発生する（v17.1.0 のテストで確認済み）。

---

## 追加するテスト（v321000_tests — 3 件）

### テスト 1: バージョン確認

```rust
fn cargo_toml_version_is_32_1_0() {
    let src = include_str!("../Cargo.toml");
    assert!(src.contains("32.1.0"), "Cargo.toml must contain '32.1.0'");
}
```

### テスト 2: ベンチマーク存在確認

```rust
fn benchmark_v32_1_0_exists() {
    let src = include_str!("../../benchmarks/v32.1.0.json");
    assert!(src.contains("32.1.0"), "benchmarks/v32.1.0.json must contain '32.1.0'");
}
```

### テスト 3: Display / Hash 境界の動作確認

`check_errors` は driver.rs 内の各テストモジュールにローカルで定義されるヘルパー（`super::*` では参照不可）。
`v321000_tests` モジュール内に独自の `check_errors` を定義して使用する。
`v171000_tests` と同じく `Parser::parse_str` + `Checker::check_program` を使うパターン。

```rust
fn check_errors(src: &str) -> Vec<String> {
    use crate::frontend::parser::Parser;
    use crate::middle::checker::Checker;
    let program = Parser::parse_str(src, "v321000_test.fav").expect("parse");
    Checker::check_program(&program)
        .0
        .iter()
        .map(|e| e.code.to_string())
        .collect()
}

fn bounded_generics_display_and_hash_bounds() {
    // Display bound: String を渡してもエラーなし
    let display_errors = check_errors(r#"
fn show<T with Display>(val: T) -> String {
    f"{val}"
}
fn main() -> String {
    show("hello")
}
"#);
    assert!(
        display_errors.is_empty(),
        "Display bound should pass for String: {:?}",
        display_errors
    );

    // Hash bound: Int を渡してもエラーなし
    let hash_errors = check_errors(r#"
fn hash_it<T with Hash>(val: T) -> Int {
    42
}
fn main() -> Int {
    hash_it(99)
}
"#);
    assert!(
        hash_errors.is_empty(),
        "Hash bound should pass for Int: {:?}",
        hash_errors
    );
}
```

---

## テストモジュールの配置

`v321000_tests` は `v320000_tests` の閉じ括弧（`}`）の直後、
かつ `// ── v31.7.0 tests` コメントの前に挿入する。

`v321000_tests` 自体は `use super::*` **なし**（モジュール内でインポートを完結させる）。

---

## 完了条件

- `Cargo.toml` version = `"32.1.0"`
- `cargo_toml_version_is_32_0_0` が空スタブになっていること
- `bounded_generics_display_and_hash_bounds` テストが PASS
- `cargo test --bin fav v321000` — 3/3 PASS
- `cargo test` — 全件 PASS（0 failures）
- `CHANGELOG.md` に `[v32.1.0]` セクション
- `benchmarks/v32.1.0.json` 存在かつ `tests_passed` が実測値
- `versions/current.md` を v32.1.0 に更新
- site/ MDX 更新: 対象外（`generics.mdx` は既に完成している）
