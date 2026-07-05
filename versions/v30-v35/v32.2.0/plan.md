# v32.2.0 — Plan: 行多相 Row Polymorphism 確認・テスト補強

## 実装方針

行多相（`TypeConstraint::HasField`・`with { field: Type }` 構文・E0337）は v18.2.0 で完成済み。
v32.2.0 は v32.1.0 と同じ「確認・記録」パターン:

1. Cargo.toml バージョン bump（32.1.0 → 32.2.0）
2. 前バージョンのバージョン確認テストをスタブ化
3. `v322000_tests` を追加（4 件）
4. CHANGELOG / benchmarks / current.md を更新

---

## 変更ファイル一覧

| ファイル | 変更内容 |
|---|---|
| `fav/Cargo.toml` | version `"32.1.0"` → `"32.2.0"` |
| `fav/src/driver.rs` | `cargo_toml_version_is_32_1_0` スタブ化 + `v322000_tests` 追加 |
| `CHANGELOG.md` | `[v32.2.0]` セクションを先頭に追記 |
| `benchmarks/v32.2.0.json` | 新規作成（実測値で埋める） |
| `versions/current.md` | 最新安定版を v32.2.0 に更新 |
| `versions/v30-v35/v32.2.0/tasks.md` | COMPLETE に更新（全 [x]） |

---

## driver.rs 変更詳細

### ① `cargo_toml_version_is_32_1_0` をスタブ化

```rust
// v321000_tests 内
fn cargo_toml_version_is_32_1_0() {
    // Stubbed: version bumped to 32.2.0 in v32.2.0.
}
```

### ② `v322000_tests` を挿入

挿入位置: `v321000_tests` の閉じ `}` 直後、`// ── v31.7.0 tests` の前。

```rust
// ── v32.2.0 tests ────────────────────────────────────────────────────────────
#[cfg(test)]
mod v322000_tests {
    use crate::frontend::parser::Parser;
    use crate::middle::checker::Checker;

    fn check_errors(src: &str) -> Vec<String> {
        let program = Parser::parse_str(src, "v322000_test.fav").expect("parse");
        Checker::check_program(&program)
            .0
            .iter()
            .map(|e| e.code.to_string())
            .collect()
    }

    #[test]
    fn cargo_toml_version_is_32_2_0() {
        let src = include_str!("../Cargo.toml");
        assert!(src.contains("32.2.0"), "Cargo.toml must contain '32.2.0'");
    }

    #[test]
    fn benchmark_v32_2_0_exists() {
        let src = include_str!("../../benchmarks/v32.2.0.json");
        assert!(src.contains("32.2.0"), "benchmarks/v32.2.0.json must contain '32.2.0'");
    }

    #[test]
    fn row_poly_field_constraint_pass() {
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

    #[test]
    fn row_poly_missing_field_e0337() {
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
}
```

---

## テスト数の見通し

| ステップ | 増減 | 累計 |
|---|---|---|
| v32.1.0 完了時点 | — | 2460 |
| `cargo_toml_version_is_32_1_0` スタブ化 | 0（テストは残る） | 2460 |
| `v322000_tests` 追加（4 件） | +4 | **2464** |

---

## CHANGELOG 追記内容

```markdown
## [v32.2.0] — 2026-07-03

### Added
- `v322000_tests`: 行多相（Row Polymorphism）動作確認テスト 4 件
  - `row_poly_field_constraint_pass` — `with { id: Int }` 制約 PASS
  - `row_poly_missing_field_e0337` — フィールドなし型を渡すと E0337

### Notes
- `TypeConstraint::HasField`・`type_has_field`・E0337 は v18.2.0 実装済み
- v32.2.0 はその動作を Language Power フェーズの記録として明示的に確認する
```
