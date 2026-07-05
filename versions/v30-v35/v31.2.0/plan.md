# v31.2.0 実装計画 — typo 候補ユーティリティ + E0011〜E0019 hint 追加

## 前提

- `fav/Cargo.toml` version = `31.1.0`
- `cargo test` — 2426 passed（0 failures）
- v31.1.0 が COMPLETE であること

---

## 実装ステップ

### Step 1: バージョンバンプ

**`fav/Cargo.toml`**
- `version = "31.1.0"` → `version = "31.2.0"`

### Step 2: driver.rs スタブ化

**`fav/src/driver.rs`**
- `v311000_tests::cargo_toml_version_is_31_1_0` をスタブ化（コメント付き）

### Step 3: levenshtein() 関数を追加

`get_help_text()` 関数（line 150 付近）の直前に追加:

```rust
fn levenshtein(s: &str, t: &str) -> usize {
    let s: Vec<char> = s.chars().collect();
    let t: Vec<char> = t.chars().collect();
    let m = s.len();
    let n = t.len();
    let mut dp = vec![vec![0usize; n + 1]; m + 1];
    for i in 0..=m { dp[i][0] = i; }
    for j in 0..=n { dp[0][j] = j; }
    for i in 1..=m {
        for j in 1..=n {
            dp[i][j] = if s[i-1] == t[j-1] {
                dp[i-1][j-1]
            } else {
                1 + dp[i-1][j].min(dp[i][j-1]).min(dp[i-1][j-1])
            };
        }
    }
    dp[m][n]
}
```

### Step 4: suggest_similar() 関数を追加

`levenshtein()` 関数の直後に追加:

```rust
fn suggest_similar<'a>(name: &str, candidates: &[&'a str]) -> Vec<&'a str> {
    let mut result: Vec<&'a str> = candidates
        .iter()
        .copied()
        .filter(|c| levenshtein(name, c) <= 2)
        .collect();
    result.truncate(3);
    result
}
```

### Step 5: get_help_text() 拡充

既存の `"E0010"` アームの直後（`"E0013"` アームの前）に追加:

```rust
"E0011" => &[
    "check the type name for typos; use `fav doc --builtins` to list built-in types",
],
"E0012" => &[
    "check that the expression type matches the expected type",
],
```

既存の `"E0015"` アームの直後（`"E0018"` アームの前）に追加:

```rust
"E0016" => &[
    "add the missing effect to the function signature: `fn foo() -> T !IO { ... }`",
],
"E0017" => &[
    "remove the unused effect declaration from the function signature",
],
```

既存の `"E0018"` アームの直後に追加:

```rust
"E0019" => &[
    "remove the circular interface inheritance",
],
```

### Step 6: v312000_tests 追加

v311000_tests の直前に追加:

```rust
// ── v31.2.0 tests ────────────────────────────────────────────────────────────
#[cfg(test)]
mod v312000_tests {
    use super::*;
    #[test]
    fn cargo_toml_version_is_31_2_0() {
        let src = include_str!("../Cargo.toml");
        assert!(src.contains("version = \"31.2.0\""), "Cargo.toml must contain version = \"31.2.0\"");
    }
    #[test]
    fn benchmark_v31_2_0_exists() {
        let src = include_str!("../../benchmarks/v31.2.0.json");
        assert!(src.contains("31.2.0"), "benchmarks/v31.2.0.json must contain '31.2.0'");
    }
    #[test]
    fn levenshtein_distance_basic() {
        assert_eq!(levenshtein("kitten", "sitting"), 3);
        assert_eq!(levenshtein("", "abc"), 3);
        assert_eq!(levenshtein("abc", "abc"), 0);
    }
    #[test]
    fn suggest_similar_finds_close_match() {
        let candidates = &["user_id2", "userId", "order_id"];
        let result = suggest_similar("user_id", candidates);
        assert!(result.contains(&"user_id2"), "should suggest user_id2");
        assert!(result.contains(&"userId"), "should suggest userId");
        assert!(!result.contains(&"order_id"), "order_id is too different");
    }
}
```

### Step 7: CHANGELOG.md 追記

先頭に追加:

```markdown
## [v31.2.0] — 2026-07-02

### Added
- `driver.rs` — `levenshtein()` / `suggest_similar()` ユーティリティ関数を追加
- `driver.rs::get_help_text()` — E0011/E0012/E0016/E0017/E0019 に hint を追加
- `benchmarks/v31.2.0.json` 追加

### Changed
- `Cargo.toml` version: `31.1.0` → `31.2.0`
```

### Step 8: benchmarks/v31.2.0.json 作成

```json
{
  "version": "31.2.0",
  "date": "2026-07-02",
  "milestone": "Real-World Readiness",
  "tests_passed": 2430,
  "tests_failed": 0,
  "notes": "levenshtein/suggest_similar utilities + E0011-E0019 hints"
}
```

> `tests_passed` は `cargo test` 実行後に実測値で更新する（+4 件 = 2430 想定）。
> **T12 で必ず実測値に書き換えること。** 上記の 2430 は暫定値。

### Step 9: versions/current.md 更新

- 「最新安定版」欄を v31.2.0 に更新
- 「次に切る版」を `v31.3.0 — TBD` に更新

---

## ファイル変更一覧

| ファイル | 種別 | 変更内容 |
|---|---|---|
| `fav/Cargo.toml` | 更新 | version `31.1.0` → `31.2.0` |
| `fav/src/driver.rs` | 更新 | v311000 スタブ化 + levenshtein/suggest_similar 追加 + get_help_text 拡充 + v312000_tests |
| `CHANGELOG.md` | 更新 | [v31.2.0] セクション追加 |
| `benchmarks/v31.2.0.json` | 新規 | ベンチマーク結果 |
| `versions/current.md` | 更新 | v31.2.0 に更新 |

---

## 完了判定

- `cargo test v312000` — 4/4 PASS
- `cargo test` — 全件 PASS（0 failures）
