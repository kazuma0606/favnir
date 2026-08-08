# v65.1.0 Spec — Linear Algebra Rune（`Rune.linalg`）

Version: 65.1.0
Status: 未着手
Base tests: 3453
Target tests: 3455

---

## 概要

行列・ベクトルを型付きで扱う線形代数 Rune `Rune.linalg` を実装する。
**将来フェーズで次元数を型パラメータで保証することを目標とする**（最大の差別化点）。
今バージョンでは `Matrix<Float>` / `Vec<Float>` の**型シグネチャ確立と関数スタブの実装**が目的。
`Matrix<Float>[1000, 128]` と `Matrix<Float>[1000, 32]` を別型として区別する型システム拡張は
**非スコープ**（将来フェーズで実装）。

```favnir
public stage PCA: Matrix<Float>[1000, 128] -> Matrix<Float>[1000, 32] = |m| {
    Rune.linalg.svd(m, components: 32)
}

public stage CosineSim: (Vec<Float>[128], Vec<Float>[128]) -> Float = |(a, b)| {
    Rune.linalg.cosine_similarity(a, b)
}
```

ロードマップ `roadmap-v65.1-v66.0.md` の v65.1.0 セクションに準拠。

---

## 前提確認（T0 で実施）

- `cargo test -j 8 -- --test-threads=8` でベース 3453 tests passed, 0 failed を確認
- `fav/Cargo.toml` の version が `"65.0.0"` であることを確認
- `runes/linalg/` ディレクトリが存在しないことを確認（新規作成対象）
- `driver.rs` に `v65000_tests` が存在することを確認（`v65100_tests` の挿入位置）
- `driver.rs` に `v65100_tests` が存在しないことを確認（新規追加）
- `fav/src/lint.rs` の最大 W コードが W041 以下であることを確認（W050 開始の根拠）

---

## 実装スコープ

### 1. `runes/linalg/rune.toml` — Rune メタデータ

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

### 2. `runes/linalg/linalg.fav` — Rune 実装スタブ

以下の関数定義をすべて含むスタブファイルを作成する。
実行時の数値計算は将来フェーズで実装。今バージョンでは**型定義と関数シグネチャの確立**が目的。

```favnir
// Linear Algebra Rune — Rune.linalg
// Type-safe matrix and vector operations for Favnir

// --- 基本演算 ---

// 内積（ドット積）— Vec<Float>[n] × Vec<Float>[n] → Float
public fn dot(a: Vec<Float>, b: Vec<Float>) -> Float {
    List.zip_with(a, b, |x, y| { x * y }) |> List.sum
}

// 行列積 — Matrix<Float> × Matrix<Float> → Matrix<Float>
public fn matmul(a: Matrix<Float>, b: Matrix<Float>) -> Matrix<Float> {
    a
}

// 転置
public fn transpose(m: Matrix<Float>) -> Matrix<Float> {
    m
}

// 逆行列
public fn inverse(m: Matrix<Float>) -> Matrix<Float> {
    m
}

// ノルム（L2 norm）
public fn norm(v: Vec<Float>) -> Float {
    List.map(v, |x| { x * x }) |> List.sum |> Float.sqrt
}

// 対角成分の抽出
public fn diag(m: Matrix<Float>) -> Vec<Float> {
    []
}

// トレース（対角和）
public fn trace(m: Matrix<Float>) -> Float {
    0.0
}

// --- 行列分解 ---

// 特異値分解（SVD）
public fn svd(m: Matrix<Float>, components: Int) -> Matrix<Float> {
    m
}

// LU 分解
public fn lu(m: Matrix<Float>) -> (Matrix<Float>, Matrix<Float>) {
    (m, m)
}

// QR 分解
public fn qr(m: Matrix<Float>) -> (Matrix<Float>, Matrix<Float>) {
    (m, m)
}

// Cholesky 分解（正定値対称行列）
public fn cholesky(m: Matrix<Float>) -> Matrix<Float> {
    m
}

// --- 固有値・固有ベクトル ---

// 固有値・固有ベクトル
public fn eig(m: Matrix<Float>) -> (Vec<Float>, Matrix<Float>) {
    ([], m)
}

// 対称行列専用固有値分解（より安定）
public fn eigh(m: Matrix<Float>) -> (Vec<Float>, Matrix<Float>) {
    ([], m)
}

// --- 距離・類似度 ---

// コサイン類似度
public fn cosine_similarity(a: Vec<Float>, b: Vec<Float>) -> Float {
    bind n_a <- norm(a)
    bind n_b <- norm(b)
    dot(a, b) / (n_a * n_b)
}

// ユークリッド距離
public fn euclidean_distance(a: Vec<Float>, b: Vec<Float>) -> Float {
    List.zip_with(a, b, |x, y| { (x - y) * (x - y) }) |> List.sum |> Float.sqrt
}

// マンハッタン距離
public fn manhattan_distance(a: Vec<Float>, b: Vec<Float>) -> Float {
    List.zip_with(a, b, |x, y| { Float.abs(x - y) }) |> List.sum
}
```

### 3. `driver.rs` — `v65100_tests` 追加

`v65000_tests` の直前（`// -- v65000_tests (v65.0.0)` コメントの直前）に挿入:

```rust
// -- v65100_tests (v65.1.0) -- Linear Algebra Rune --
#[cfg(test)]
mod v65100_tests {
    #[test]
    fn linalg_rune_matrix_ops() {
        let content = include_str!("../../runes/linalg/linalg.fav");
        assert!(!content.is_empty(), "linalg.fav should not be empty");
        assert!(
            content.contains("matmul"),
            "linalg.fav should define matmul"
        );
        assert!(
            content.contains("dot"),
            "linalg.fav should define dot"
        );
        assert!(
            content.contains("transpose"),
            "linalg.fav should define transpose"
        );
        assert!(
            content.contains("cosine_similarity"),
            "linalg.fav should define cosine_similarity"
        );
    }

    #[test]
    fn linalg_rune_svd_decomposition() {
        let content = include_str!("../../runes/linalg/linalg.fav");
        assert!(
            content.contains("svd"),
            "linalg.fav should define svd"
        );
        assert!(
            content.contains("eig"),
            "linalg.fav should define eig"
        );
        assert!(
            content.contains("cholesky"),
            "linalg.fav should define cholesky"
        );
        assert!(
            content.contains("euclidean_distance"),
            "linalg.fav should define euclidean_distance"
        );
    }
}
```

---

## 完了条件

- `runes/linalg/linalg.fav` が存在し空でない
- `runes/linalg/rune.toml` が存在する
- `linalg.fav` に `matmul`, `dot`, `transpose`, `cosine_similarity`, `svd`, `eig`, `cholesky`, `euclidean_distance` が定義されている
- `cargo test --bin fav v65100_tests` で 2 件 PASS
  - `linalg_rune_matrix_ops` PASS
  - `linalg_rune_svd_decomposition` PASS
- `cargo test -j 8 -- --test-threads=8` で 3455 tests passed, 0 failed

---

## 非スコープ

- 実際の数値計算実装（行列積の数値演算等）— 将来フェーズで実装。今バージョンはスタブ
- `Matrix<T>[rows, cols]` / `Vec<T>[n]` の型システム拡張（型パラメータ次元保証）— 将来フェーズ
- WASM 対応（`cfg(not(wasm32))` ガード）— 必要なら実装時に判断
- ドキュメントサイト MDX 作成 — v65.9.0（安定化）で一括作成

---

## 技術ノート

### `include_str!` パス（`fav/src/driver.rs` 起点）

- `"../../runes/linalg/linalg.fav"` → `favnir/runes/linalg/linalg.fav`
- `"../../runes/linalg/rune.toml"` → `favnir/runes/linalg/rune.toml`

### `Matrix<Float>` / `Vec<Float>` 型について

`Matrix<Float>` と `Vec<Float>` は現行型チェッカー（`checker.fav`）の**組み込み型には存在しない**。
`linalg.fav` を `fav check` に通すことは**今バージョンのスコープ外**とする。

スタブ実装として `linalg.fav` に型シグネチャを記述するが、型エラーが発生する場合は
コメントアウトまたは `// TODO: type system extension` の注記を入れる。
`driver.rs` のテストは `include_str!` で文字列として読み込むだけなので、
型チェックなしでテストが通ることを確認すれば十分。

### `bind` 構文について

Favnir には `let` 構文はない。変数への束縛はすべて `bind x = expr` を使う。
コード例も `bind` を使って記述する。

### rune.toml フォーマット

既存 rune（例: `runes/stat/rune.toml`）に合わせた `[rune]` セクション + `[exports]` セクション構成とする。
`[connection]` セクション等の外部接続設定は不要（Pure 関数ライブラリのため）。
