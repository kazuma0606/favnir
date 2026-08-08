# v66.1.0 Spec — Vector Stage Primitives（`Rune.vec`）

Version: 66.1.0
Status: 未着手
Base tests: 3475
Target tests: 3477

---

## 概要

ベクトル演算を型安全なステージとして提供する Rune `Rune.vec` を実装する。
正規化・内積・コサイン類似度・ユークリッド距離・バッチ処理・次元投影を提供し、
AI パイプライン（埋め込みモデル連携）の基盤となる。

```favnir
// 利用例（用途のイメージ）
// ※ Vec<Float>[N] の次元型パラメータは将来フェーズで型システムに登録する
// 今バージョンでは List<Float> をプレースホルダーとして使用

public stage CosineSim: (List<Float>, List<Float>) -> Float = |(a, b)| {
    Rune.vec.cosine_similarity(a, b)
}

public stage BatchEmbed: List<String> -> List<List<Float>> = |texts| {
    Rune.vec.batch_embed(texts, "text-embedding-3-small")
}
```

ロードマップ `roadmap-v66.1-v67.0.md` の v66.1.0 セクションに準拠。

> **スコープ縮小の明示**: ロードマップの利用例では `Vec<Float>[N]` 次元型パラメータを使用しているが、
> 型システムへの登録は将来フェーズ。本バージョンでは `List<Float>` をプレースホルダーとして
> 関数シグネチャを確立することに専念する。

---

## 前提確認（T0 で実施）

- `cargo test -j 8 -- --test-threads=8` でベース 3475 tests passed, 0 failed を確認
- `fav/Cargo.toml` の version が `"66.0.0"` であることを確認（sub-version では Cargo.toml は更新しない）
- `runes/vec/` ディレクトリが存在しないことを確認（新規作成対象）
- `driver.rs` に `v66000_tests` が存在することを確認（`v66100_tests` の挿入位置）
- `driver.rs` に `v66100_tests` が存在しないことを確認（新規追加）
- `cargo test --bin fav v66000_tests` で 4 件 PASS することを確認（前バージョンが正常）
- `versions/current.md` の「最新安定版」が `v66.0.0` であることを確認

---

## 実装スコープ

### 1. `runes/vec/rune.toml` — Rune メタデータ

```toml
[rune]
name        = "vec"
version     = "0.1.0"
description = "Vector Stage Primitives for Favnir — normalize, dot product, cosine similarity, euclidean distance, batch operations, dimension projection"
entry       = "vec.fav"
effects     = []

[dependencies]
```

### 2. `runes/vec/vec.fav` — Rune 実装スタブ

以下の全関数定義を含むスタブファイルを作成する。
今バージョンでは**関数シグネチャの確立**が目的。実際のベクトル計算は将来フェーズ。

```favnir
// Vector Stage Primitives — Rune.vec
// normalize, dot product, cosine similarity, euclidean distance, batch ops, projection
//
// NOTE: Vec<Float>[N]（次元型パラメータ）は将来フェーズで型システムに登録する。
//       今バージョンは List<Float> をプレースホルダーとして使用。
//       include_str! テストのみ（型チェックエラーは無視する）。

// --- 基本ベクトル演算 ---

// L2 正規化（ノルム = 1 に変換）
public fn normalize(v: List<Float>) -> List<Float> {
    v
}

// 内積（ドット積）
public fn dot(a: List<Float>, b: List<Float>) -> Float {
    0.0
}

// コサイン類似度（-1.0 〜 1.0）
public fn cosine_similarity(a: List<Float>, b: List<Float>) -> Float {
    0.0
}

// ユークリッド距離
public fn euclidean_distance(a: List<Float>, b: List<Float>) -> Float {
    0.0
}

// --- バッチ処理 ---

// バッチ埋め込み: テキストのリスト → 埋め込みベクトルのリスト
public fn batch_embed(texts: List<String>, model: String) -> List<List<Float>> {
    []
}

// コサイン類似度行列（N×N）
public fn batch_cosine_matrix(vecs: List<List<Float>>) -> List<List<Float>> {
    []
}

// --- 次元変換 ---

// 次元投影（次元削減 / 拡張）
// VecDimProjection — PCA などによる線形変換
public fn project(v: List<Float>, target_dim: Int) -> List<Float> {
    []
}
```

### 3. `driver.rs` — `v66100_tests` 追加

挿入位置: `// -- v66000_tests (v66.0.0)` コメントの直前

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

挿入後、`cargo build` でエラーなしを確認。

---

## 完了条件

- `runes/vec/vec.fav` が存在し空でない
- `runes/vec/rune.toml` が存在する
- `vec.fav` に全 7 関数が定義されている:
  - `normalize`, `dot`, `cosine_similarity`, `euclidean_distance`（基本演算）
  - `batch_embed`, `batch_cosine_matrix`（バッチ処理）
  - `project`（次元変換）
- `cargo test --bin fav v66100_tests` で 2 件 PASS
  - `vec_stage_dim_type_check` PASS
  - `vec_stage_batch_and_project` PASS
- `cargo test -j 8 -- --test-threads=8` で 3477 tests passed, 0 failed

---

## 非スコープ

- `Vec<Float>[N]` 次元型パラメータの型システム登録 — 将来フェーズ
- 実際のベクトル計算実装（BLAS / SIMD 等） — 将来フェーズ
- `fav check` での型チェック通過 — 今バージョンは `include_str!` テストのみ
- CHANGELOG.md 更新 — v67.0.0 宣言時に一括追記
- site/ MDX ドキュメント作成 — v66.9.0 安定化時に一括作成するため今バージョンは省略

---

## 技術ノート

### `include_str!` パス（`fav/src/driver.rs` 起点）

- `"../../runes/vec/vec.fav"` → `favnir/runes/vec/vec.fav`

### `contains` 判定の設計方針

- `contains("fn normalize(")` — `public fn normalize(` にマッチ
- `contains("fn dot(")` — `public fn dot(` にマッチ
- `contains("fn cosine_similarity(")` — `public fn cosine_similarity(` にマッチ
- `contains("fn euclidean_distance(")` — `public fn euclidean_distance(` にマッチ
- `contains("fn batch_embed(")` — `public fn batch_embed(` にマッチ
- `contains("fn batch_cosine_matrix(")` — `public fn batch_cosine_matrix(` にマッチ
- `contains("fn project(")` — `public fn project(` にマッチ
- `contains("VecDimProjection")` — コメント `// VecDimProjection — PCA` でマッチ。**注意**: コメントを変更・削除した場合は当該テストアサーションも連動して更新すること

### Favnir 構文ルール（v66.x 共通）

- `bind x <- expr` は Result/Option を返す式にのみ使用する（スタブでは不要）
- `let` は使わない
- `Math.sqrt` を使う（`Float.sqrt` は VM に存在しない）
- `Float.from_int` は VM に存在しない

### rune.toml フォーマット

- `entry = "vec.fav"`（`main` ではなく `entry`）
- `effects = []` を明示
- `[dependencies]` セクションを含める
