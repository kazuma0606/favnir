# v32.7.0 — Plan: 定数ジェネリクス 確認・テスト補強

## 実装方針

定数ジェネリクス（`<const N: Int>`・`where { N > 0 }`・E0335）は
v18.7.0 で完成済み。v32.7.0 は v32.1.0〜v32.6.0 と同じ「確認・記録」パターン。

---

## 変更ファイル一覧

| ファイル | 変更内容 |
|---|---|
| `fav/Cargo.toml` | version `"32.6.0"` → `"32.7.0"` |
| `fav/src/driver.rs` | `cargo_toml_version_is_32_6_0` スタブ化 + `v327000_tests` 追加 |
| `CHANGELOG.md` | `[v32.7.0]` セクションを先頭に追記 |
| `benchmarks/v32.7.0.json` | 新規作成（実測値で埋める） |
| `versions/current.md` | 最新安定版を v32.7.0 に更新 |
| `versions/v30-v35/v32.7.0/tasks.md` | COMPLETE に更新（全 [x]） |

---

## driver.rs 変更詳細

### ① `cargo_toml_version_is_32_6_0` をスタブ化

```rust
// v326000_tests 内（既存の #[test] fn を空スタブに置き換える）
fn cargo_toml_version_is_32_6_0() {
    // Stubbed: version bumped to 32.7.0 in v32.7.0.
}
```

### ② `v327000_tests` を挿入

挿入位置: `v326000_tests` の閉じ `}` 直後、`// ── v31.7.0 tests` の前。

```rust
// ── v32.7.0 tests ────────────────────────────────────────────────────────────
#[cfg(test)]
mod v327000_tests {
    use crate::frontend::parser::Parser;
    use crate::middle::checker::Checker;

    fn check_errors(src: &str) -> Vec<String> {
        let program = Parser::parse_str(src, "v327000_test.fav").expect("parse");
        Checker::check_program(&program)
            .0
            .iter()
            .map(|e| e.code.to_string())
            .collect()
    }

    #[test]
    fn cargo_toml_version_is_32_7_0() {
        let src = include_str!("../Cargo.toml");
        assert!(src.contains("32.7.0"), "Cargo.toml must contain '32.7.0'");
    }

    #[test]
    fn benchmark_v32_7_0_exists() {
        let src = include_str!("../../benchmarks/v32.7.0.json");
        assert!(src.contains("32.7.0"), "benchmarks/v32.7.0.json must contain '32.7.0'");
    }

    #[test]
    fn const_gen_chunk_size_valid() {
        // N = 5 は N > 0 を満たす → E0335 なし
        // (テスト名は v187000_tests::const_generic_valid と異なる)
        let errors = check_errors(r#"
fn safe_chunk<const N: Int where { N > 0 }>(items: Int) -> Int { 0 }
fn main() -> Int { safe_chunk<5>(100) }
"#);
        assert!(
            errors.iter().all(|e| e != "E0335"),
            "const N=5 satisfies N>0, should not produce E0335: {:?}",
            errors
        );
    }

    #[test]
    fn const_gen_chunk_size_zero_e0335() {
        // N = 0 は N > 0 を違反 → E0335
        // (テスト名は v187000_tests::const_generic_violation と異なる)
        let errors = check_errors(r#"
fn safe_chunk<const N: Int where { N > 0 }>(items: Int) -> Int { 0 }
fn main() -> Int { safe_chunk<0>(100) }
"#);
        assert!(
            errors.iter().any(|e| e == "E0335"),
            "Expected E0335 for const N=0 violating N>0, got: {:?}",
            errors
        );
    }
}
```

---

## テスト数の見通し

| ステップ | 増減 | 累計 |
|---|---|---|
| v32.6.0 完了時点 | — | 2480 |
| `cargo_toml_version_is_32_6_0` スタブ化 | 0（テストは残る） | 2480 |
| `v327000_tests` 追加（4 件） | +4 | **2484** |

---

## CHANGELOG 追記内容

```markdown
## [v32.7.0] — 2026-07-03

### Added
- `v327000_tests`: 定数ジェネリクス（Const Generics）動作確認テスト 4 件
  - `cargo_toml_version_is_32_7_0` — バージョン確認
  - `benchmark_v32_7_0_exists` — ベンチマークファイル存在確認
  - `const_gen_chunk_size_valid` — `N=5` が `N>0` を満たす → E0335 なし
  - `const_gen_chunk_size_zero_e0335` — `N=0` が `N>0` を違反 → E0335

### Notes
- `GenericParam.is_const` / `const_ty` / `const_constraint` / E0335 は v18.7.0 実装済み
- v32.7.0 はその動作を Language Power フェーズの記録として明示的に確認する
```
