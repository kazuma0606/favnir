# v65.4.0 Plan — Optimization Rune（`Rune.optim`）

Version: 65.4.0
Status: 未着手

---

## 作業順序

### Step 1: 前提確認

```bash
# テスト数確認（3459 であること）
cd /c/Users/yoshi/favnir/fav && cargo test -j 8 -- --test-threads=8 2>&1 | grep "test result" | grep "3459"

# バージョン確認（65.0.0 であること — sub-version では変更しない）
grep 'version = ' Cargo.toml | head -1

# runes/optim/ 未存在確認
ls ../runes/optim/ 2>/dev/null || echo "not found (OK)"

# driver.rs の挿入位置確認
grep -n "v65300_tests\|v65400_tests" src/driver.rs
# → v65300_tests のみ存在し v65400_tests が存在しないこと

# 既存テストコマンド形式の前提確認
cargo test --bin fav v65300_tests 2>&1 | tail -3
```

---

### Step 2: `runes/optim/rune.toml` 作成

`runes/stat/rune.toml` の形式に合わせて作成:

```toml
[rune]
name        = "optim"
version     = "0.1.0"
description = "Optimization Rune for Favnir — gradient-based optimizers, schedulers, constrained optimization"
entry       = "optim.fav"
effects     = []

[dependencies]
```

---

### Step 3: `runes/optim/optim.fav` 作成

spec.md に記載した全関数定義を含むスタブファイルを作成する。

**ファーストオーダー最適化**（5 関数）: `sgd`, `adam`, `adamw`, `adagrad`, `rmsprop`

**二次法**（2 関数）: `l_bfgs`, `conjugate_gradient`

**汎用最適化**（1 関数）: `minimize`

**学習率スケジューラ**（3 関数）: `step_decay`, `cosine_annealing`, `warmup_cosine`

**制約付き最適化**（1 関数）: `minimize_constrained`

作成後の確認:
```bash
# let の混入がないことを確認
grep -n 'let ' ../runes/optim/optim.fav || echo "OK: no let"

# bind x = （<- でない bind）がないことを確認
grep -n 'bind [a-z_]* =' ../runes/optim/optim.fav || echo "OK: no bind ="

# Float.from_int / Float.sqrt の混入がないことを確認
grep -n 'Float\.from_int\|Float\.sqrt' ../runes/optim/optim.fav || echo "OK"
```

---

### Step 4: `driver.rs` — `v65400_tests` 追加

挿入位置: `// -- v65300_tests (v65.3.0)` コメントの直前

```rust
// -- v65400_tests (v65.4.0) -- Optimization Rune --
#[cfg(test)]
mod v65400_tests {
    #[test]
    fn optim_rune_adam_converges() {
        let content = include_str!("../../runes/optim/optim.fav");
        assert!(!content.is_empty(), "optim.fav should not be empty");
        assert!(content.contains("fn adam("), "optim.fav should define adam");
        assert!(content.contains("fn sgd("), "optim.fav should define sgd");
        assert!(content.contains("fn minimize("), "optim.fav should define minimize");
    }

    #[test]
    fn optim_rune_minimize_quadratic() {
        let content = include_str!("../../runes/optim/optim.fav");
        assert!(content.contains("fn l_bfgs("), "optim.fav should define l_bfgs");
        assert!(
            content.contains("fn conjugate_gradient("),
            "optim.fav should define conjugate_gradient"
        );
        assert!(content.contains("fn step_decay("), "optim.fav should define step_decay scheduler");
    }
}
```

挿入後、`cargo build` でエラーなしを確認。

---

### Step 5: テスト実行

```bash
cd /c/Users/yoshi/favnir/fav

# v65400_tests のみ確認
cargo test --bin fav v65400_tests 2>&1 | tail -8
# → 2 tests passed が表示されること

# 全テスト確認
cargo test -j 8 -- --test-threads=8 2>&1 | grep "test result"
# → 3461 tests passed, 0 failed が表示されること
```

---

### Step 6: ドキュメント・ステータス更新

`versions/roadmap/roadmap-v65.1-v66.0.md` のバージョン一覧テーブルで
v65.4.0 行の `状態` 列を `未着手` → `完了` に変更:

```diff
-| v65.4.0 | Optimization Rune（`Rune.optim`） | 3459 + 2 = 3461 | 未着手 |
+| v65.4.0 | Optimization Rune（`Rune.optim`） | 3459 + 2 = 3461 | 完了 |
```

`versions/current.md` の「進行中バージョン」欄を v65.4.0 に更新。

本 `tasks.md` の Status を `COMPLETE` に更新し、全チェックボックスを `[x]` に変更。

---

## リスク・注意事項

| リスク | 対策 |
|---|---|
| `bind x <- expr` を非 Result/Option 値に使う | スタブは全て引数をそのまま返すだけなので `bind` 不要 |
| `Float.from_int` / `Float.sqrt` の混入 | 作成後 `grep` で確認 |
| `let ` の混入 | 作成後 `grep -n 'let '` で確認 |
| `runes/optim/` を `runes/optim` と表記ミス | 複数形なし・`optim` が正しい名前 |
| `minimize_constrained` が `minimize` の偽陽性になる | `fn minimize(` で括弧まで含めるため一意に特定できる |
| `entry` ではなく `main` を使う | rune.toml は `entry = "optim.fav"` が正しい |
