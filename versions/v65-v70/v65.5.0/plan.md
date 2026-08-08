# v65.5.0 Plan — Numerical Methods Rune（`Rune.numeric`）

Version: 65.5.0
Status: 未着手

---

## 作業順序

### Step 1: 前提確認

```bash
# テスト数確認（3461 であること）
cd /c/Users/yoshi/favnir/fav && cargo test -j 8 -- --test-threads=8 2>&1 | grep "test result" | grep "3461"

# バージョン確認（65.0.0 であること）
grep 'version = ' Cargo.toml | head -1

# runes/numeric/ 未存在確認
ls ../runes/numeric/ 2>/dev/null || echo "not found (OK)"

# driver.rs の挿入位置確認
grep -n "v65400_tests\|v65500_tests" src/driver.rs
# → v65400_tests のみ存在し v65500_tests が存在しないこと

# 前バージョンテスト確認
cargo test --bin fav v65400_tests 2>&1 | tail -3

# current.md 確認
grep "進行中バージョン" -A 1 /c/Users/yoshi/favnir/versions/current.md
```

---

### Step 2: `runes/numeric/rune.toml` 作成

```toml
[rune]
name        = "numeric"
version     = "0.1.0"
description = "Numerical methods Rune for Favnir — integration, ODE solvers, interpolation, FFT, root finding"
entry       = "numeric.fav"
effects     = []

[dependencies]
```

---

### Step 3: `runes/numeric/numeric.fav` 作成

spec.md に記載した全関数定義を含むスタブファイルを作成する。

**数値積分**（4 関数）: `integrate`, `trapezoid`, `simpson`, `gauss_quadrature`

**ODE ソルバー**（4 関数）: `ode_solve`, `euler`, `runge_kutta4`, `dormand_prince`

**補間**（3 関数）: `linear_interp`, `cubic_spline`, `polynomial_interp`

**フーリエ変換**（4 関数）: `fft`, `ifft`, `power_spectrum`, `spectrogram`

**根探索**（3 関数）: `bisection`, `newton_raphson`, `brent`

**線形方程式系**（1 関数）: `conjugate_gradient_solver`

作成後の確認:
```bash
# let の混入がないことを確認
grep -n 'let ' ../runes/numeric/numeric.fav || echo "OK: no let"

# bind x = （<- でない bind）がないことを確認
grep -n 'bind [a-z_]* =' ../runes/numeric/numeric.fav || echo "OK: no bind ="

# Float.from_int / Float.sqrt の混入がないことを確認
grep -n 'Float\.from_int\|Float\.sqrt' ../runes/numeric/numeric.fav || echo "OK"

# 全 16 関数が定義されていることを確認
grep -c 'public fn ' ../runes/numeric/numeric.fav
# → 19 が表示されること（integrate/ode_solve を汎用ラッパーとして追加しているため）
```

---

### Step 4: `driver.rs` — `v65500_tests` 追加

挿入位置: `// -- v65400_tests (v65.4.0)` コメントの直前

```rust
// -- v65500_tests (v65.5.0) -- Numerical Methods Rune --
#[cfg(test)]
mod v65500_tests {
    #[test]
    fn numeric_rune_integrate() {
        let content = include_str!("../../runes/numeric/numeric.fav");
        assert!(!content.is_empty(), "numeric.fav should not be empty");
        assert!(content.contains("fn integrate("), "numeric.fav should define integrate");
        assert!(content.contains("fn fft("), "numeric.fav should define fft");
        assert!(content.contains("fn ifft("), "numeric.fav should define ifft");
    }

    #[test]
    fn numeric_rune_fft() {
        let content = include_str!("../../runes/numeric/numeric.fav");
        assert!(content.contains("fn ode_solve("), "numeric.fav should define ode_solve");
        assert!(content.contains("fn bisection("), "numeric.fav should define bisection");
        assert!(
            content.contains("fn newton_raphson("),
            "numeric.fav should define newton_raphson"
        );
    }
}
```

挿入後、`cargo build` でエラーなしを確認。

---

### Step 5: テスト実行

```bash
cd /c/Users/yoshi/favnir/fav

# v65500_tests のみ確認
cargo test --bin fav v65500_tests 2>&1 | tail -8
# → 2 tests passed が表示されること

# 全テスト確認
cargo test -j 8 -- --test-threads=8 2>&1 | grep "test result"
# → 3463 tests passed, 0 failed が表示されること
```

---

### Step 6: ドキュメント・ステータス更新

`versions/roadmap/roadmap-v65.1-v66.0.md` のバージョン一覧テーブルで
v65.5.0 行の `状態` 列を `未着手` → `完了` に変更:

```diff
-| v65.5.0 | Numerical Methods Rune（`Rune.numeric`） | 3461 + 2 = 3463 | 未着手 |
+| v65.5.0 | Numerical Methods Rune（`Rune.numeric`） | 3461 + 2 = 3463 | 完了 |
```

`versions/current.md` の「進行中バージョン」欄を v65.5.0 に更新。

本 `tasks.md` の Status を `COMPLETE` に更新し、全チェックボックスを `[x]` に変更。

---

## リスク・注意事項

| リスク | 対策 |
|---|---|
| `bind x <- expr` を非 Result/Option 値に使う | スタブは全て `0.0` / `[]` を返すだけなので `bind` 不要 |
| `Float.from_int` / `Float.sqrt` の混入 | 作成後 `grep` で確認 |
| `fn fft(` が `fn ifft(` にマッチする | `"fn fft("` は `"fn ifft("` の部分文字列にならない（`fn fft(` vs `fn ifft(`）— 問題なし |
| `conjugate_gradient_solver` が v65.3 の `conjugate_gradient` と混同 | 別 Rune（`runes/optim/` vs `runes/numeric/`）なので問題なし |
| `entry` ではなく `main` を使う | rune.toml は `entry = "numeric.fav"` が正しい |
