# v32.4.0 — Plan: スキーマ型 確認・テスト補強

## 実装方針

スキーマ型（`TypeExpr::Schema`・`schema "uri"` 構文・`register_schema_types`）は
v18.4.0 で完成済み。v32.4.0 は v32.1.0〜v32.3.0 と同じ「確認・記録」パターン。

今回のテストは**パーサー中心**（`check_errors` 不要）:
- `schema` 型構文がパースできること
- パース後の AST が `TypeExpr::Schema` を含むこと

---

## 変更ファイル一覧

| ファイル | 変更内容 |
|---|---|
| `fav/Cargo.toml` | version `"32.3.0"` → `"32.4.0"` |
| `fav/src/driver.rs` | `cargo_toml_version_is_32_3_0` スタブ化 + `v324000_tests` 追加 |
| `CHANGELOG.md` | `[v32.4.0]` セクションを先頭に追記 |
| `benchmarks/v32.4.0.json` | 新規作成（実測値で埋める） |
| `versions/current.md` | 最新安定版を v32.4.0 に更新 |
| `versions/v30-v35/v32.4.0/tasks.md` | COMPLETE に更新（全 [x]） |

---

## driver.rs 変更詳細

### ① `cargo_toml_version_is_32_3_0` をスタブ化

```rust
// v323000_tests 内
fn cargo_toml_version_is_32_3_0() {
    // Stubbed: version bumped to 32.4.0 in v32.4.0.
}
```

### ② `v324000_tests` を挿入

挿入位置: `v323000_tests` の閉じ `}` 直後、`// ── v31.7.0 tests` の前。

```rust
// ── v32.4.0 tests ────────────────────────────────────────────────────────────
#[cfg(test)]
mod v324000_tests {
    use crate::frontend::parser::Parser;
    use crate::ast::{Item, TypeBody, TypeExpr};

    #[test]
    fn cargo_toml_version_is_32_4_0() {
        let src = include_str!("../Cargo.toml");
        assert!(src.contains("32.4.0"), "Cargo.toml must contain '32.4.0'");
    }

    #[test]
    fn benchmark_v32_4_0_exists() {
        let src = include_str!("../../benchmarks/v32.4.0.json");
        assert!(src.contains("32.4.0"), "benchmarks/v32.4.0.json must contain '32.4.0'");
    }

    #[test]
    fn schema_alias_parses() {
        // `type UserRow = schema "file:..."` should parse without errors
        let src = r#"type UserRow = schema "file:schemas/users.json""#;
        let result = Parser::parse_str(src, "v324000_test.fav");
        assert!(result.is_ok(), "schema type syntax should parse: {:?}", result.err());
    }

    #[test]
    fn schema_type_ast_is_schema_expr() {
        // The AST should contain TypeDef with TypeBody::Alias(TypeExpr::Schema(..))
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
}
```

---

## テスト数の見通し

| ステップ | 増減 | 累計 |
|---|---|---|
| v32.3.0 完了時点 | — | 2468 |
| `cargo_toml_version_is_32_3_0` スタブ化 | 0（テストは残る） | 2468 |
| `v324000_tests` 追加（4 件） | +4 | **2472** |

---

## CHANGELOG 追記内容

```markdown
## [v32.4.0] — 2026-07-03

### Added
- `v324000_tests`: スキーマ型（Schema Types）動作確認テスト 4 件
  - `cargo_toml_version_is_32_4_0` — バージョン確認
  - `benchmark_v32_4_0_exists` — ベンチマークファイル存在確認
  - `schema_alias_parses` — `schema "file:..."` 構文パース確認
  - `schema_type_ast_is_schema_expr` — AST が `TypeExpr::Schema` を含む確認

### Notes
- `TypeExpr::Schema`・`register_schema_types`・`schema_loader` は v18.4.0 実装済み
- v32.4.0 はその動作を Language Power フェーズの記録として明示的に確認する
```
