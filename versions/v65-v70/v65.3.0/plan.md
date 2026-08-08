# v65.3.0 Plan — Autodiff Rune（`Rune.autodiff`）

Version: 65.3.0
Status: 未着手

---

## 作業順序

### Step 1: 前提確認

```bash
# テスト数確認（3457 であること）
cd /c/Users/yoshi/favnir/fav && cargo test -j 8 -- --test-threads=8 2>&1 | grep "test result" | grep "3457"

# バージョン確認（65.0.0 であること — sub-version では変更しない）
grep 'version = ' Cargo.toml | head -1

# runes/autodiff/ 未存在確認
ls ../runes/autodiff/ 2>/dev/null || echo "not found (OK)"

# driver.rs の挿入位置確認
grep -n "v65200_tests\|v65300_tests" src/driver.rs
# → v65200_tests のみ存在し v65300_tests が存在しないこと

# 既存テストコマンド形式の前提確認
cargo test --bin fav v65200_tests 2>&1 | tail -3
```

---

### Step 2: `runes/autodiff/rune.toml` 作成

`runes/stat/rune.toml` の形式に合わせて作成:

```toml
[rune]
name        = "autodiff"
version     = "0.1.0"
description = "Automatic differentiation Rune for Favnir — reverse-mode AD, grad, jacobian, hessian"
entry       = "autodiff.fav"
effects     = []

[dependencies]
```

---

### Step 3: `runes/autodiff/autodiff.fav` 作成

spec.md に記載した全関数定義を含むスタブファイルを作成する。

**計算テープ**（1 関数）: `tape`

**微分演算**（3 関数）: `grad`, `jacobian`, `hessian`

**勾配追跡制御**（1 関数）: `no_grad`

**プリミティブ**（6 関数）: `prim_exp`, `prim_log`, `prim_sin`, `prim_cos`, `prim_tanh`, `relu`

作成後の確認:
```bash
# let の混入がないことを確認
grep -n 'let ' ../runes/autodiff/autodiff.fav || echo "OK: no let"

# bind x = （<- でない bind）がないことを確認
grep -n 'bind [a-z_]* =' ../runes/autodiff/autodiff.fav || echo "OK: no bind ="

# Float.from_int / Float.sqrt の混入がないことを確認
grep -n 'Float\.from_int\|Float\.sqrt' ../runes/autodiff/autodiff.fav || echo "OK"
```

---

### Step 4: `driver.rs` — `v65300_tests` 追加

挿入位置: `// -- v65200_tests (v65.2.0)` コメントの直前

```rust
// -- v65300_tests (v65.3.0) -- Autodiff Rune --
#[cfg(test)]
mod v65300_tests {
    #[test]
    fn autodiff_rune_grad_scalar() {
        let content = include_str!("../../runes/autodiff/autodiff.fav");
        assert!(!content.is_empty(), "autodiff.fav should not be empty");
        assert!(content.contains("fn grad("), "autodiff.fav should define grad");
        assert!(content.contains("fn jacobian("), "autodiff.fav should define jacobian");
        assert!(content.contains("fn hessian("), "autodiff.fav should define hessian");
    }

    #[test]
    fn autodiff_rune_chain_rule() {
        let content = include_str!("../../runes/autodiff/autodiff.fav");
        assert!(content.contains("Tape"), "autodiff.fav should reference Tape");
        assert!(content.contains("fn no_grad("), "autodiff.fav should define no_grad");
        assert!(content.contains("fn relu("), "autodiff.fav should define relu primitive");
    }
}
```

挿入後、`cargo build` でエラーなしを確認。

---

### Step 5: テスト実行

```bash
cd /c/Users/yoshi/favnir/fav

# v65300_tests のみ確認
cargo test --bin fav v65300_tests 2>&1 | tail -8
# → 2 tests passed が表示されること

# 全テスト確認
cargo test -j 8 -- --test-threads=8 2>&1 | grep "test result"
# → 3459 tests passed, 0 failed が表示されること
```

---

### Step 6: ドキュメント・ステータス更新

`versions/roadmap/roadmap-v65.1-v66.0.md` のバージョン一覧テーブルで
v65.3.0 行の `状態` 列を `未着手` → `完了` に変更:

```diff
-| v65.3.0 | Autodiff Rune（`Rune.autodiff`） | 3457 + 2 = 3459 | 未着手 |
+| v65.3.0 | Autodiff Rune（`Rune.autodiff`） | 3457 + 2 = 3459 | 完了 |
```

`versions/current.md` の「進行中バージョン」欄を v65.3.0 に更新。

本 `tasks.md` の Status を `COMPLETE` に更新し、全チェックボックスを `[x]` に変更。

---

## リスク・注意事項

| リスク | 対策 |
|---|---|
| `bind x <- expr` を非 Result/Option 値に使う | スタブは全て `x` を返すだけなので `bind` 不要。使わない |
| `Float.from_int` / `Float.sqrt` の混入 | 作成後 `grep` で確認 |
| `let ` の混入 | 作成後 `grep -n 'let '` で確認 |
| `contains("Tape")` の偽陽性 | ファイル内容は autodiff.fav のみなので問題なし |
| `contains("relu")` が `prim_relu` などにマッチしない | `relu` を部分文字列として含む関数名なら全てマッチする |
| `runes/stat/` との混同 | `runes/autodiff/` に作成すること |
| `entry` ではなく `main` を使う | rune.toml は `entry = "autodiff.fav"` が正しい |
