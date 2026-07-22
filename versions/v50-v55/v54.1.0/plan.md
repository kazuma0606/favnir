# Plan: v54.1.0 — 全エラーコード fav explain --error 対応完備

---

## ステップ 1: 事前確認

```bash
cargo test -j 8 -- --test-threads=8 2>&1 | grep "test result"
# → 3185 passed, 0 failed を確認

cargo clippy -- -D warnings
# → warnings なしであることを確認

# v54100_tests が未存在を確認
rg -n "v54100_tests" fav/src/driver.rs  # → 0 件

# v54000_tests の行番号を確認（挿入位置）
rg -n "v54000_tests" fav/src/driver.rs  # → 行番号を特定

# hello.fav が正しい内容であることを確認
cat fav/tmp/hello.fav
# → fn add(a: Int, b: Int) -> Int { a + b }
# → fn main() -> Bool { add(1, 2) == 3 }

# Cargo.toml が 54.0.0 であることを確認
grep "^version" fav/Cargo.toml  # → version = "54.0.0"

# error_catalog.rs エントリ数の確認
grep -c "    code:" fav/src/error_catalog.rs  # → 92
```

---

## ステップ 2: `driver.rs` — `v54100_tests` 追加 + `cargo_toml_version_is_54_0_0` 空化

### 2a: `v54000_tests` の直前に `v54100_tests` を追加

```rust
// -- v54100_tests (v54.1.0) -- 全エラーコード fav explain --error 対応完備 --
#[cfg(test)]
mod v54100_tests {
    use super::*;

    #[test]
    fn explain_error_all_codes_have_collect_text() {
        let all = crate::error_catalog::list_all();
        assert!(!all.is_empty(), "ERROR_CATALOG must not be empty");
        for entry in all {
            let result = cmd_explain_error_collect(entry.code);
            assert!(
                result.is_some(),
                "cmd_explain_error_collect({}) returned None",
                entry.code
            );
            let text = result.unwrap();
            assert!(
                !text.is_empty(),
                "explain text for {} must not be empty",
                entry.code
            );
        }
    }

    #[test]
    fn explain_error_e0419_exists() {
        let result = cmd_explain_error_collect("E0419");
        assert!(result.is_some(), "E0419 must have an explain entry");
        let text = result.unwrap();
        assert!(
            text.contains("E0419"),
            "explain text for E0419 must contain the code"
        );
        assert!(
            text.contains("assert_schema"),
            "explain text for E0419 must reference assert_schema"
        );
    }
}
```

### 2b: `cargo_toml_version_is_54_0_0` を空化

```rust
fn cargo_toml_version_is_54_0_0() {
    // v54.1.0 にバンプしたためアサートを空化。
}
```

`cargo build` → コンパイルエラーなし確認。

---

## ステップ 3: `fav/Cargo.toml` バージョン更新

`version = "54.0.0"` → `version = "54.1.0"`

---

## ステップ 4: テスト実行・確認

```bash
cd /c/Users/yoshi/favnir/fav && cargo test -j 8 -- --test-threads=8 2>&1 | grep "test result"
```

期待値: 3187 passed, 0 failed（テスト数 ≥ 3185 ✓）

```bash
cargo clippy -- -D warnings
```

---

## ステップ 5: 後処理

- `CHANGELOG.md`: v54.1.0 エントリ追加（v54.0.0 エントリの直上）
- `versions/current.md` を v54.1.0（3187 tests）に更新
- `roadmap-v54.1-v55.0.md` の v54.1.0 実績欄を COMPLETE に更新（3187 tests・完了日付記入）
- `tasks.md` を COMPLETE に更新（T0〜T5 全 `[x]`）

コードレビュー対応（事前判明分）:
- [MED] テスト名重複回避のため `explain_error_all_codes_have_collect_text` とする（v503000_tests と区別）
- [LOW] CHANGELOG エラーコード数は `grep -c "    code:"` の実数（92）を記載
- [LOW] roadmap 推定値「3181」→「ベース 3185 + 2 = 3187」に修正
