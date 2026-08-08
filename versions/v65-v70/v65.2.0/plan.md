# v65.2.0 Plan — Statistics Rune（`Rune.stats`）

Version: 65.2.0
Status: 未着手

---

## 作業順序

### Step 1: 前提確認

```bash
# テスト数確認（3455 であること）
cd /c/Users/yoshi/favnir/fav && cargo test -j 8 -- --test-threads=8 2>&1 | grep "test result" | grep "3455"

# バージョン確認（65.0.0 であること）
grep 'version = ' Cargo.toml | head -1

# runes/stats/ 未存在確認
ls ../runes/stats/ 2>/dev/null || echo "not found (OK)"

# runes/stat/ の rune.toml が存在することを確認（既存、削除しない）
ls ../runes/stat/rune.toml

# driver.rs の挿入位置確認
grep -n "v65100_tests\|v65200_tests" src/driver.rs
# → v65100_tests のみ存在し v65200_tests が存在しないこと

# 既存テストコマンド形式の前提確認
cargo test --bin fav v65100_tests 2>&1 | tail -3
```

---

### Step 2: `runes/stats/rune.toml` 作成

`runes/stat/rune.toml` の形式に合わせて作成:

```toml
[rune]
name        = "stats"
version     = "0.1.0"
description = "Statistics Rune for Favnir — descriptive stats, hypothesis tests, regression, anomaly detection"
entry       = "stats.fav"
effects     = []

[dependencies]
```

---

### Step 3: `runes/stats/stats.fav` 作成

spec.md に記載した全関数定義を含むスタブファイルを作成する。

**記述統計**（8 関数）: `mean`, `variance`, `std`, `median`, `percentile`, `skewness`, `kurtosis`, `describe`

**確率分布**（4 関数）: `fit`, `sample`, `pdf`, `cdf`

**仮説検定**（5 関数）: `t_test`, `chi_square`, `ks_test`, `mannwhitney`, `anova`

**回帰**（2 関数）: `linear_regression`, `logistic_regression`

**異常検知**（3 関数）: `zscore_filter`, `iqr_filter`, `isolation_forest`

実装の注意点（v65.1.0 レビュー教訓）:
- `bind x <- expr` を使う（`=` は誤り）
- `Math.sqrt` を使う（`Float.sqrt` は VM に存在しない）
- `List.zip_with(f, xs, ys)` — クロージャが第1引数
- `let` は絶対に使わない

---

### Step 4: `driver.rs` — `v65200_tests` 追加

挿入位置: `// -- v65100_tests (v65.1.0)` コメントの直前

```rust
// -- v65200_tests (v65.2.0) -- Statistics Rune --
#[cfg(test)]
mod v65200_tests {
    #[test]
    fn stats_rune_describe() {
        let content = include_str!("../../runes/stats/stats.fav");
        assert!(!content.is_empty(), "stats.fav should not be empty");
        assert!(content.contains("fn mean("), "stats.fav should define mean");
        assert!(content.contains("fn std("), "stats.fav should define std");
        assert!(content.contains("fn median("), "stats.fav should define median");
        assert!(content.contains("fn describe("), "stats.fav should define describe");
    }

    #[test]
    fn stats_rune_hypothesis_test() {
        let content = include_str!("../../runes/stats/stats.fav");
        assert!(content.contains("fn t_test("), "stats.fav should define t_test");
        assert!(content.contains("fn chi_square("), "stats.fav should define chi_square");
        assert!(content.contains("fn ks_test("), "stats.fav should define ks_test");
        assert!(
            content.contains("fn linear_regression("),
            "stats.fav should define linear_regression"
        );
        assert!(
            content.contains("fn zscore_filter("),
            "stats.fav should define zscore_filter"
        );
    }
}
```

挿入後、`cargo build` でエラーなしを確認。

---

### Step 5: テスト実行

```bash
cd /c/Users/yoshi/favnir/fav

# v65200_tests のみ確認
cargo test --bin fav v65200_tests 2>&1 | tail -8
# → 2 tests passed が表示されること

# 全テスト確認
cargo test -j 8 -- --test-threads=8 2>&1 | grep "test result"
# → 3457 tests passed, 0 failed が表示されること
```

---

### Step 6: ドキュメント・ステータス更新

`versions/roadmap/roadmap-v65.1-v66.0.md` のバージョン一覧テーブルで
v65.2.0 行の `状態` 列を `未着手` → `完了` に変更:

```
| v65.2.0 | Statistics Rune（`Rune.stats`） | 3455 + 2 = 3457 | 完了 |
```

`versions/current.md` の「進行中バージョン」欄を v65.2.0 に更新。

本 `tasks.md` の Status を `COMPLETE` に更新し、全チェックボックスを `[x]` に変更。

---

## リスク・注意事項

| リスク | 対策 |
|---|---|
| `bind x = expr` の混入 | 作成後 `grep -n 'bind.*=' stats.fav` で確認 |
| `let ` の混入 | 作成後 `grep -n '\blet ' stats.fav` で確認 |
| `runes/stat/` との混同 | `runes/stats/`（複数形）に作成すること |
| `entry` ではなく `main` を使う | rune.toml は `entry = "stats.fav"` が正しい |
| contains("std") が標準ライブラリにも当たる | `fn std(` の形式で確認しているので問題なし |
