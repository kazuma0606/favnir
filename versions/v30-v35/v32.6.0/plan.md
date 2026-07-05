# v32.6.0 — Plan: 分散アノテーション 確認・テスト補強

## 実装方針

分散アノテーション（`Variance::Covariant`・`Variance::Contravariant`・E0334）は
v18.6.0 で完成済み。v32.6.0 は v32.1.0〜v32.5.0 と同じ「確認・記録」パターン。

---

## 変更ファイル一覧

| ファイル | 変更内容 |
|---|---|
| `fav/Cargo.toml` | version `"32.5.0"` → `"32.6.0"` |
| `fav/src/driver.rs` | `cargo_toml_version_is_32_5_0` スタブ化 + `v326000_tests` 追加 |
| `CHANGELOG.md` | `[v32.6.0]` セクションを先頭に追記 |
| `benchmarks/v32.6.0.json` | 新規作成（実測値で埋める） |
| `versions/current.md` | 最新安定版を v32.6.0 に更新 |
| `versions/v30-v35/v32.6.0/tasks.md` | COMPLETE に更新（全 [x]） |

---

## driver.rs 変更詳細

### ① `cargo_toml_version_is_32_5_0` をスタブ化

```rust
// v325000_tests 内（既存の #[test] fn を空スタブに置き換える）
fn cargo_toml_version_is_32_5_0() {
    // Stubbed: version bumped to 32.6.0 in v32.6.0.
}
```

### ② `v326000_tests` を挿入

挿入位置: `v325000_tests` の閉じ `}` 直後、`// ── v31.7.0 tests` の前。

```rust
// ── v32.6.0 tests ────────────────────────────────────────────────────────────
#[cfg(test)]
mod v326000_tests {
    use crate::frontend::parser::Parser;
    use crate::middle::checker::Checker;

    fn check_errors(src: &str) -> Vec<String> {
        let program = Parser::parse_str(src, "v326000_test.fav").expect("parse");
        // Error::code は String 型
        Checker::check_program(&program)
            .0
            .iter()
            .map(|e| e.code.to_string())
            .collect()
    }

    #[test]
    fn cargo_toml_version_is_32_6_0() {
        let src = include_str!("../Cargo.toml");
        assert!(src.contains("32.6.0"), "Cargo.toml must contain '32.6.0'");
    }

    #[test]
    fn benchmark_v32_6_0_exists() {
        let src = include_str!("../../benchmarks/v32.6.0.json");
        assert!(src.contains("32.6.0"), "benchmarks/v32.6.0.json must contain '32.6.0'");
    }

    #[test]
    fn variance_ann_covariant_output_pass() {
        // +T が出力位置にのみ使われる → E0334 なし
        // (テスト名は v186000_tests::variance_subtype_covariant と異なる)
        let errors = check_errors(r#"
interface Source<+T> {
    next: Unit -> Option<T>
}
"#);
        assert!(
            errors.iter().all(|e| e != "E0334"),
            "Covariant +T in output position should not produce E0334: {:?}",
            errors
        );
    }

    #[test]
    fn variance_ann_covariant_input_e0334() {
        // +T が入力位置（引数）に使われる → E0334
        // (テスト名は v186000_tests::variance_violation_error と異なる)
        let errors = check_errors(r#"
interface BadSource<+T> {
    write: T -> Unit
}
"#);
        assert!(
            errors.iter().any(|e| e == "E0334"),
            "Expected E0334 for covariant +T in input position, got: {:?}",
            errors
        );
    }
}
```

---

## テスト数の見通し

| ステップ | 増減 | 累計 |
|---|---|---|
| v32.5.0 完了時点 | — | 2476 |
| `cargo_toml_version_is_32_5_0` スタブ化 | 0（テストは残る） | 2476 |
| `v326000_tests` 追加（4 件） | +4 | **2480** |

---

## CHANGELOG 追記内容

```markdown
## [v32.6.0] — 2026-07-03

### Added
- `v326000_tests`: 分散アノテーション（Variance Annotations）動作確認テスト 4 件
  - `cargo_toml_version_is_32_6_0` — バージョン確認
  - `benchmark_v32_6_0_exists` — ベンチマークファイル存在確認
  - `variance_ann_covariant_output_pass` — `+T` が出力位置のみ → E0334 なし
  - `variance_ann_covariant_input_e0334` — `+T` が入力位置 → E0334

### Notes
- `Variance::Covariant`・`Variance::Contravariant`・E0334 は v18.6.0 実装済み
- v32.6.0 はその動作を Language Power フェーズの記録として明示的に確認する
```
