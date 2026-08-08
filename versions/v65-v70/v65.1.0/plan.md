# v65.1.0 Plan — Linear Algebra Rune（`Rune.linalg`）

Version: 65.1.0
Status: 未着手

---

## 作業順序

### Step 1: 前提確認

1. テスト数確認:
   ```bash
   cd /c/Users/yoshi/favnir/fav && cargo test -j 8 -- --test-threads=8 2>&1 | tail -3
   # → 3453 tests passed が表示されること
   ```

2. バージョン確認:
   ```bash
   grep 'version = ' fav/Cargo.toml | head -1
   # → version = "65.0.0"
   ```

3. linalg ディレクトリ未存在確認:
   ```bash
   ls runes/linalg/ 2>/dev/null || echo "not found (OK)"
   # → "not found (OK)" が出ること
   ```

4. driver.rs の挿入位置確認:
   ```bash
   grep -n "v65000_tests\|v65100_tests" fav/src/driver.rs
   # → v65000_tests のみ存在し v65100_tests が存在しないこと
   ```

   既存テストコマンドの動作確認（`--bin fav` 形式の前提確認）:
   ```bash
   cd /c/Users/yoshi/favnir/fav && cargo test --bin fav v65000_tests 2>&1 | tail -5
   # → 4 tests passed が表示されること（コマンド形式が正しいことを確認）
   ```

5. lint.rs の最大 W コード確認:
   ```bash
   grep -oE 'W[0-9]{3}' fav/src/lint.rs | sort -u | tail -5
   # → W041 以下であること（W042 以上が出たら要確認）
   ```

---

### Step 2: `runes/linalg/rune.toml` 作成

```toml
[rune]
name = "linalg"
version = "0.1.0"
description = "Linear algebra Rune for Favnir — type-safe matrix and vector operations"
author = "Favnir Core Team"
license = "MIT"

[exports]
main = "linalg.fav"
```

---

### Step 3: `runes/linalg/linalg.fav` 作成

spec.md に記載した全関数定義（`dot`, `matmul`, `transpose`, `inverse`, `norm`, `diag`, `trace`,
`svd`, `lu`, `qr`, `cholesky`, `eig`, `eigh`, `cosine_similarity`, `euclidean_distance`,
`manhattan_distance`）を含むスタブファイルを作成する。

実装の注意点:
- `bind` 構文を使う（`let` は Favnir に存在しない）
- スタブのため戻り値はダミー（`m` / `[]` / `0.0` 等）で OK
- 各関数に `public fn` 宣言を付ける

---

### Step 4: `driver.rs` — `v65100_tests` 追加

挿入位置: `// -- v65000_tests (v65.0.0)` コメントの直前

```rust
// -- v65100_tests (v65.1.0) -- Linear Algebra Rune --
#[cfg(test)]
mod v65100_tests {
    #[test]
    fn linalg_rune_matrix_ops() {
        let content = include_str!("../../runes/linalg/linalg.fav");
        assert!(!content.is_empty(), "linalg.fav should not be empty");
        assert!(content.contains("matmul"), "linalg.fav should define matmul");
        assert!(content.contains("dot"), "linalg.fav should define dot");
        assert!(content.contains("transpose"), "linalg.fav should define transpose");
        assert!(content.contains("cosine_similarity"), "linalg.fav should define cosine_similarity");
    }

    #[test]
    fn linalg_rune_svd_decomposition() {
        let content = include_str!("../../runes/linalg/linalg.fav");
        assert!(content.contains("svd"), "linalg.fav should define svd");
        assert!(content.contains("eig"), "linalg.fav should define eig");
        assert!(content.contains("cholesky"), "linalg.fav should define cholesky");
        assert!(content.contains("euclidean_distance"), "linalg.fav should define euclidean_distance");
    }
}
```

挿入後、`cargo build` でコンパイルエラーがないことを確認。

---

### Step 5: テスト実行

```bash
# v65100_tests のみ先に確認
cd /c/Users/yoshi/favnir/fav && cargo test --bin fav v65100_tests 2>&1 | tail -10
# → 2 tests passed が表示されること

# 全テスト確認
cargo test -j 8 -- --test-threads=8 2>&1 | tail -5
# → 3455 tests passed, 0 failed が表示されること
```

---

### Step 6: `versions/current.md` 更新

`versions/current.md` の「進行中バージョン」を v65.1.0 に更新する。

---

### Step 7: ドキュメント更新

- `versions/roadmap/roadmap-v65.1-v66.0.md` の v65.1.0 行を「完了」に更新
- 本 `tasks.md` を COMPLETE に更新

---

## リスク・注意事項

| リスク | 対策 |
|---|---|
| `include_str!` パスの誤り | `../../runes/linalg/` が `favnir/runes/linalg/` を指すことを確認 |
| `bind` 以外の構文混入 | linalg.fav 内に `let ` が含まれていないことを grep で確認 |
| `eig` と `eigh` 両方必要 | テストは `eig` のみ確認するが、スタブには両方定義すること |
| 他テストへの影響 | `include_str!` はコンパイル時解決のため、ファイルが存在しないとビルド全体が失敗する |

## 所要時間見積もり

- Step 1（前提確認）: 5 分
- Step 2〜3（Rune ファイル作成）: 10 分
- Step 4（driver.rs 更新）: 5 分
- Step 5（テスト実行）: 5 分
- Step 6〜7（ドキュメント更新）: 5 分
- **合計**: 約 30 分
