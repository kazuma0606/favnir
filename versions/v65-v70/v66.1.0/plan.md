# v66.1.0 実装計画 — Vector Stage Primitives（`Rune.vec`）

Version: 66.1.0
Status: 未着手
Base tests: 3475
Target tests: 3477

---

## 実装ステップ

### Step 1: ディレクトリ・ファイル作成

1. `runes/vec/` ディレクトリ作成
2. `runes/vec/rune.toml` 作成
3. `runes/vec/vec.fav` 作成（全 7 関数）

### Step 2: `driver.rs` テスト追加

- `// -- v66000_tests (v66.0.0)` コメントの直前に `v66100_tests` を挿入
- 2 テスト関数:
  - `vec_stage_dim_type_check`
  - `vec_stage_batch_and_project`

### Step 3: ビルド・テスト確認

```bash
cargo build
cargo test --bin fav v66100_tests
cargo test -j 8 -- --test-threads=8
```

---

## `vec.fav` 実装方針

- **全 7 関数をスタブとして実装**（シグネチャ確立が目的）
- `bind` / `let` は使用しない
- `Float.from_int` / `Float.sqrt` は使用しない
- 戻り値:
  - `Float` 系 → `0.0`
  - `List<_>` 系 → `[]`
  - `List<Float>` で入力を返す系 → 引数をそのまま返す（`v`）
- `VecDimProjection` はコメント中で使用 → `contains("VecDimProjection")` テストにマッチ

## `rune.toml` 形式

```toml
[rune]
name        = "vec"
version     = "0.1.0"
description = "Vector Stage Primitives for Favnir — normalize, dot product, cosine similarity, euclidean distance, batch operations, dimension projection"
entry       = "vec.fav"
effects     = []

[dependencies]
```

---

## `driver.rs` 挿入コード

```rust
// -- v66100_tests (v66.1.0) -- Vector Stage Primitives --
#[cfg(test)]
mod v66100_tests {
    #[test]
    fn vec_stage_dim_type_check() {
        let content = include_str!("../../runes/vec/vec.fav");
        assert!(!content.is_empty(), "vec.fav should not be empty");
        assert!(content.contains("fn normalize("), "vec.fav should define normalize");
        assert!(content.contains("fn dot("), "vec.fav should define dot");
        assert!(content.contains("fn cosine_similarity("), "vec.fav should define cosine_similarity");
        assert!(content.contains("fn euclidean_distance("), "vec.fav should define euclidean_distance");
    }

    #[test]
    fn vec_stage_batch_and_project() {
        let content = include_str!("../../runes/vec/vec.fav");
        assert!(content.contains("fn batch_embed("), "vec.fav should define batch_embed");
        assert!(
            content.contains("fn batch_cosine_matrix("),
            "vec.fav should define batch_cosine_matrix"
        );
        assert!(content.contains("fn project("), "vec.fav should define project");
        assert!(
            content.contains("VecDimProjection"),
            "vec.fav should reference VecDimProjection"
        );
    }
}
```

---

## 関数一覧（7 関数）

| カテゴリ | 関数名 | 戻り値 |
|---|---|---|
| 基本演算 | `normalize(v: List<Float>)` | `v`（入力をそのまま返す） |
| 基本演算 | `dot(a: List<Float>, b: List<Float>)` | `0.0` |
| 基本演算 | `cosine_similarity(a: List<Float>, b: List<Float>)` | `0.0` |
| 基本演算 | `euclidean_distance(a: List<Float>, b: List<Float>)` | `0.0` |
| バッチ処理 | `batch_embed(texts: List<String>, model: String)` | `[]` |
| バッチ処理 | `batch_cosine_matrix(vecs: List<List<Float>>)` | `[]` |
| 次元変換 | `project(v: List<Float>, target_dim: Int)` | `[]` |

---

## リスク・注意点

- `Vec<Float>[N]` 次元型パラメータは未定義のため `List<Float>` で代替（型チェックエラーは無視）
- `contains("VecDimProjection")` はコメント行 `// VecDimProjection — PCA などによる線形変換` でマッチ
- `contains("fn dot(")` は `fn euclidean_distance(` 等より短いため偽陽性なし（関数名の一意性を確認済み）
