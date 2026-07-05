# v32.4.0 — Spec: スキーマ型 確認・テスト補強

## 概要

v32.4.0 は **スキーマ型（Schema Types）** の確認・テスト補強バージョン。

ロードマップ v32.4 では以下の実装を目標としていたが、実際にはすでに v18.4.0 で実装済みである:

| コンポーネント | 実装済み | バージョン |
|---|---|---|
| `TypeExpr::Schema(String, Span)` — AST スキーマ型ノード | ✓ | v18.4.0 |
| `parse_type_expr` — `schema "uri"` 構文パース | ✓ | v18.4.0 |
| `register_schema_types` — チェック時 URI 解決 | ✓ | v18.4.0 |
| `resolve_schema` — キャッシュ参照（checker.rs:7724） | ✓ | v18.4.0 |
| `schema_loader` モジュール — URI パース / JSON スキーマロード | ✓ | v18.4.0 |
| `v184000_tests` — 4 件のテスト | ✓ | v18.4.0 |

v32.4.0 では、`schema` 型構文が仕様通りにパースされ AST を正しく生成することを
`v324000_tests` で明示的に確認し、バージョンと CHANGELOG を更新する。

---

## スキーマ型仕様確認

### 構文

```favnir
// DB / JSON スキーマから型を取得（コンパイル時 or `fav infer` で事前生成）
type UserRow = schema "file:schemas/users.json"
type PgUsers = schema "postgres:users"
```

### AST（ast.rs:157）

```rust
pub enum TypeExpr {
    // ...
    /// `schema "source:identifier"` — schema type import (v18.4.0)
    Schema(String, Span),
    // ...
}
```

`TypeBody::Alias(TypeExpr::Schema(uri, _))` として型定義に格納される。

### チェッカー（checker.rs）

| 動作 | 箇所 |
|---|---|
| `register_schema_types` — プログラム中の schema 宣言を URI 解決 | checker.rs:7732 |
| `resolve_schema` — schema_types キャッシュから型を返す | checker.rs:7724 |
| ファイル存在しない場合 → `Type::Unknown`（エラーなし） | checker.rs:7728 |

---

## 追加するテスト（v324000_tests — 4 件）

v324000_tests は**パーサー中心**パターン:
- `use super::*` **なし**
- `use crate::frontend::parser::Parser; use crate::ast::{Item, TypeBody, TypeExpr};`
- `check_errors` は定義しない（スキーマ型確認はパース/AST レベルで完結）

### テスト 1: バージョン確認

```rust
fn cargo_toml_version_is_32_4_0() {
    let src = include_str!("../Cargo.toml");
    assert!(src.contains("32.4.0"), "Cargo.toml must contain '32.4.0'");
}
```

### テスト 2: ベンチマーク存在確認

```rust
fn benchmark_v32_4_0_exists() {
    let src = include_str!("../../benchmarks/v32.4.0.json");
    assert!(src.contains("32.4.0"), "benchmarks/v32.4.0.json must contain '32.4.0'");
}
```

### テスト 3: スキーマ型構文パース確認

```rust
fn schema_alias_parses() {
    // postgres: URI スキーム（v184000_tests は file: を使用 — 差別化）
    let src = r#"type PgUsers = schema "postgres:users""#;
    let result = Parser::parse_str(src, "v324000_test.fav");
    assert!(result.is_ok(), "schema alias with postgres: URI should parse: {:?}", result.err());
}
```

### テスト 4: AST が TypeExpr::Schema を含む

```rust
fn schema_type_ast_is_schema_expr() {
    let src = r#"type UserRow = schema "file:schemas/users.json""#;
    let prog = Parser::parse_str(src, "v324000_test.fav").expect("parse");
    assert_eq!(prog.items.len(), 1, "expected 1 item");
    if let Item::TypeDef(td) = &prog.items[0] {
        assert_eq!(td.name, "UserRow");
        assert!(
            matches!(&td.body, TypeBody::Alias(TypeExpr::Schema(uri, _)) if uri.contains("users.json")),
            "expected TypeBody::Alias(Schema(..)), got: {:?}", td.body
        );
    } else {
        panic!("expected TypeDef item");
    }
}
```

---

## テストモジュールの配置

`v324000_tests` は `v323000_tests` の閉じ括弧（`}`）の直後、
かつ `// ── v31.7.0 tests` コメントの前に挿入する。

```
// ...v323000_tests 閉じ }

// ── v32.4.0 tests ────────────────────────────────────────────────────────────
#[cfg(test)]
mod v324000_tests {
    ...
}

// ── v31.7.0 tests ────────────────────────────────────────────────────────────
#[cfg(test)]
mod v317000_tests {
```

---

## 完了条件

- `Cargo.toml` version = `"32.4.0"`
- `cargo_toml_version_is_32_3_0` が空スタブになっていること
- `schema_alias_parses` テストが PASS
- `schema_type_ast_is_schema_expr` テストが PASS
- `cargo test --bin fav v324000` — 4/4 PASS
- `cargo test` — 全件 PASS（0 failures）
- `CHANGELOG.md` に `[v32.4.0]` セクション
- `benchmarks/v32.4.0.json` 存在かつ `tests_passed` が実測値
- `benchmarks/v32.4.0.json` の `milestone` フィールドが `"Language Power"` であること
- `versions/current.md` を v32.4.0 に更新
- `tasks.md` がすべて `[x]` で COMPLETE に更新されていること
- site/ MDX 更新: 対象外（`schema-types.mdx` 等は既に完成）
