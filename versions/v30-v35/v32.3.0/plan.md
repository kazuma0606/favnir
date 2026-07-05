# v32.3.0 — Plan: where 制約（関数引数）確認・テスト補強

## 実装方針

where 制約（Refinement Types、`fn_refinement_registry`・E0331・`RefinementAssert` opcode）は
v18.3.0 で完成済み。v32.3.0 は v32.1.0 / v32.2.0 と同じ「確認・記録」パターン:

1. Cargo.toml バージョン bump（32.2.0 → 32.3.0）
2. 前バージョンのバージョン確認テストをスタブ化
3. `v323000_tests` を追加（4 件）
4. CHANGELOG / benchmarks / current.md を更新

---

## 変更ファイル一覧

| ファイル | 変更内容 |
|---|---|
| `fav/Cargo.toml` | version `"32.2.0"` → `"32.3.0"` |
| `fav/src/driver.rs` | `cargo_toml_version_is_32_2_0` スタブ化 + `v323000_tests` 追加 |
| `CHANGELOG.md` | `[v32.3.0]` セクションを先頭に追記 |
| `benchmarks/v32.3.0.json` | 新規作成（実測値で埋める） |
| `versions/current.md` | 最新安定版を v32.3.0 に更新 |
| `versions/v30-v35/v32.3.0/tasks.md` | COMPLETE に更新（全 [x]） |

---

## driver.rs 変更詳細

### ① `cargo_toml_version_is_32_2_0` をスタブ化

```rust
// v322000_tests 内
fn cargo_toml_version_is_32_2_0() {
    // Stubbed: version bumped to 32.3.0 in v32.3.0.
}
```

### ② `v323000_tests` を挿入

挿入位置: `v322000_tests` の閉じ `}` 直後、`// ── v31.7.0 tests` の前。

```rust
// ── v32.3.0 tests ────────────────────────────────────────────────────────────
#[cfg(test)]
mod v323000_tests {
    use crate::frontend::parser::Parser;
    use crate::middle::checker::Checker;

    fn check_errors(src: &str) -> Vec<String> {
        let program = Parser::parse_str(src, "v323000_test.fav").expect("parse");
        Checker::check_program(&program)
            .0
            .iter()
            .map(|e| e.code.to_string())
            .collect()
    }

    #[test]
    fn cargo_toml_version_is_32_3_0() {
        let src = include_str!("../Cargo.toml");
        assert!(src.contains("32.3.0"), "Cargo.toml must contain '32.3.0'");
    }

    #[test]
    fn benchmark_v32_3_0_exists() {
        let src = include_str!("../../benchmarks/v32.3.0.json");
        assert!(src.contains("32.3.0"), "benchmarks/v32.3.0.json must contain '32.3.0'");
    }

    #[test]
    fn where_constraint_literal_pass() {
        // b=2 satisfies b != 0 → no E0331
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

    #[test]
    fn where_constraint_literal_fail_e0331() {
        // b=0 violates b != 0 → E0331
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
}
```

---

## テスト数の見通し

| ステップ | 増減 | 累計 |
|---|---|---|
| v32.2.0 完了時点 | — | 2464 |
| `cargo_toml_version_is_32_2_0` スタブ化 | 0（テストは残る） | 2464 |
| `v323000_tests` 追加（4 件） | +4 | **2468** |

---

## CHANGELOG 追記内容

```markdown
## [v32.3.0] — 2026-07-03

### Added
- `v323000_tests`: where 制約（Refinement Types）動作確認テスト 4 件
  - `cargo_toml_version_is_32_3_0` — バージョン確認
  - `benchmark_v32_3_0_exists` — ベンチマークファイル存在確認
  - `where_constraint_literal_pass` — `b=2` で `b != 0` 制約 PASS
  - `where_constraint_literal_fail_e0331` — `b=0` で E0331

### Notes
- `fn_refinement_registry`・E0331・`RefinementAssert` opcode は v18.3.0 実装済み
- v32.3.0 はその動作を Language Power フェーズの記録として明示的に確認する
```
