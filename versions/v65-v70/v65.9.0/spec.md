# v65.9.0 Spec — 安定化・コードフリーズ（Math & Science 前調整）

Version: 65.9.0
Status: 未着手
Base tests: 3469
Target tests: 3471

---

## 概要

v65.1〜v65.8 の全機能が正常動作することを確認する安定化バージョン。
Math Rune 群のドキュメントを整備し、Performance 1.0 → Math & Science Foundation への移行を確認する。

**確認内容**:
- v65.1〜v65.8 の全 Rune ファイル（linalg / stats / autodiff / optim / numeric / timeseries / ml）が存在し空でない
- W050〜W054 がコンパイルエラーなく動作（`cargo build` 済み）
- `site/content/docs/runes/math-runes-overview.mdx` の作成（Math Rune 群の概要ページ）

ロードマップ `roadmap-v65.1-v66.0.md` の v65.9.0 セクションに準拠。

---

## 前提確認（T0 で実施）

- `cargo test --bin fav` でベース 3469 tests passed, 0 failed を確認
- `fav/Cargo.toml` の version が `"65.0.0"` であることを確認（sub-version では更新しない）
- `driver.rs` に `v65800_tests` が存在することを確認（`v65900_tests` の挿入位置）
- `driver.rs` に `v65900_tests` が存在しないことを確認（新規追加）
- `cargo test --bin fav v65800_tests` で 2 件 PASS することを確認（前バージョンが正常）
- 7 Rune ファイルが存在することを確認:
  - `runes/linalg/linalg.fav`
  - `runes/stats/stats.fav`
  - `runes/autodiff/autodiff.fav`
  - `runes/optim/optim.fav`
  - `runes/numeric/numeric.fav`
  - `runes/timeseries/timeseries.fav`
  - `runes/ml/ml.fav`
- `versions/current.md` の「進行中バージョン」が `v65.8.0` であることを確認

---

## 実装スコープ

### 1. `site/content/docs/runes/math-runes-overview.mdx` — 新規作成

Math Rune 群の概要ページ。v65.1〜v65.8 で追加した 7 Rune を紹介する。
"Rune.linalg" を含む必要がある（`math_docs_complete` テストで検証）。

```mdx
---
title: "Math & Science Runes — 概要"
description: "Favnir の数学・科学計算 Rune 群（v65.1〜v65.8）の概要"
---

# Math & Science Runes

Favnir v65.x では、科学計算・機械学習のための Rune 群を追加しました。
型安全に数学的操作を記述できる「Math & Science Foundation」の基盤です。

## Rune 一覧

| Rune | バージョン | 概要 |
|---|---|---|
| `Rune.linalg` | v65.1.0 | 線形代数（行列演算・固有値分解・LU/QR/SVD）|
| `Rune.stats` | v65.2.0 | 統計解析（記述統計・仮説検定・回帰分析）|
| `Rune.autodiff` | v65.3.0 | 自動微分（逆伝播・Jacobian・Hessian）|
| `Rune.optim` | v65.4.0 | 最適化（SGD・Adam・L-BFGS・スケジューラ）|
| `Rune.numeric` | v65.5.0 | 数値計算（積分・ODE・FFT・補間・根探索）|
| `Rune.timeseries` | v65.6.0 | 時系列（ARIMA・SARIMA・STL・変化点検出）|
| `Rune.ml` | v65.7.0 | ML Primitives（分類・回帰・クラスタリング・次元削減）|

## Math Lint Rules（v65.8.0）

W050〜W054 により、数学 Rune 特有のアンチパターンを静的解析で検出できます。

| コード | 検出内容 |
|---|---|
| W050 | 行列次元の不一致（動的パス） |
| W051 | 数値不安定な演算（ゼロ除算リスク）|
| W052 | 統計的有意性なしの比較 |
| W053 | 自動微分ループでの in-place 変更 |
| W054 | 最適化ループの収束条件未設定 |

## 利用例

```favnir
// Rune.linalg — 行列演算
public stage MatMulStage: (Matrix<Float>, Matrix<Float>) -> Matrix<Float> = |(a, b)| {
    Rune.linalg.matmul(a, b)
}

// Rune.stats — 記述統計
public stage DescribeData: List<Float> -> Stats = |xs| {
    Rune.stats.describe(xs)
}

// Rune.ml — 分類パイプライン
public stage Classify: Matrix<Float> -> Predictions = |features| {
    Rune.ml.predict(Rune.ml.random_forest(100, 10), features)
}
[ここでコードブロックを ``` で閉じる]

## 次のマイルストーン

v66.0.0「Math & Science Foundation」宣言で、本スプリントの成果を正式リリースします。
```

> **MDX ファイル作成時の注意**: favnir コードブロックは ` ```favnir ` で始まり ` ``` ` で閉じること（plan.md 参照）。

### 2. `driver.rs` — `v65900_tests` 追加

挿入位置: `// -- v65800_tests (v65.8.0)` コメントの直前

```rust
// -- v65900_tests (v65.9.0) -- Stabilization --
#[cfg(test)]
mod v65900_tests {
    #[test]
    fn math_foundation_all_runes_stable() {
        let linalg   = include_str!("../../runes/linalg/linalg.fav");
        let stats    = include_str!("../../runes/stats/stats.fav");
        let autodiff = include_str!("../../runes/autodiff/autodiff.fav");
        let optim    = include_str!("../../runes/optim/optim.fav");
        let numeric  = include_str!("../../runes/numeric/numeric.fav");
        let ts       = include_str!("../../runes/timeseries/timeseries.fav");
        let ml       = include_str!("../../runes/ml/ml.fav");
        assert!(!linalg.is_empty(),   "linalg.fav should not be empty");
        assert!(!stats.is_empty(),    "stats.fav should not be empty");
        assert!(!autodiff.is_empty(), "autodiff.fav should not be empty");
        assert!(!optim.is_empty(),    "optim.fav should not be empty");
        assert!(!numeric.is_empty(),  "numeric.fav should not be empty");
        assert!(!ts.is_empty(),       "timeseries.fav should not be empty");
        assert!(!ml.is_empty(),       "ml.fav should not be empty");
    }

    #[test]
    fn math_docs_complete() {
        let content = include_str!("../../site/content/docs/runes/math-runes-overview.mdx");
        assert!(!content.is_empty(), "math-runes-overview.mdx should not be empty");
        assert!(
            content.contains("Rune.linalg"),
            "math-runes-overview.mdx should mention Rune.linalg"
        );
    }
}
```

挿入後、`cargo build` でエラーなしを確認。

---

## 完了条件

- `cargo build` でコンパイルエラーなし（W050〜W054 スタブ含む）
- `site/content/docs/runes/math-runes-overview.mdx` が存在し "Rune.linalg" を含む
- 7 Rune ファイルがすべて存在し空でない（`math_foundation_all_runes_stable` で検証）
- `cargo test --bin fav v65900_tests` で 2 件 PASS
  - `math_foundation_all_runes_stable` PASS
  - `math_docs_complete` PASS
- `cargo test --bin fav` で 3471 tests passed, 0 failed

---

## 非スコープ

- CHANGELOG.md 更新 — v66.0.0 宣言時に一括追記
- Cargo.toml バージョン更新 — v66.0.0 宣言時に `"66.0.0"` に更新

---

## 技術ノート

### `include_str!` パス（`fav/src/driver.rs` 起点）

- `"../../runes/linalg/linalg.fav"` → `favnir/runes/linalg/linalg.fav`
- `"../../runes/stats/stats.fav"` → `favnir/runes/stats/stats.fav`
- `"../../runes/autodiff/autodiff.fav"` → `favnir/runes/autodiff/autodiff.fav`
- `"../../runes/optim/optim.fav"` → `favnir/runes/optim/optim.fav`
- `"../../runes/numeric/numeric.fav"` → `favnir/runes/numeric/numeric.fav`
- `"../../runes/timeseries/timeseries.fav"` → `favnir/runes/timeseries/timeseries.fav`
- `"../../runes/ml/ml.fav"` → `favnir/runes/ml/ml.fav`
- `"../../site/content/docs/runes/math-runes-overview.mdx"` → `favnir/site/content/docs/runes/math-runes-overview.mdx`

### MDX ファイルの最低要件

- 空でないこと
- `"Rune.linalg"` 文字列を含むこと
- MDX 構文として有効であること（acorn パースエラー回避のため、コードブロック内に import/export を置かない）

### テスト分割の設計

- `math_foundation_all_runes_stable`: 7 Rune ファイルの存在確認（将来 v65.1〜v65.8 の各関数シグネチャ確認に拡張可能）
- `math_docs_complete`: MDX ドキュメントの存在と最低限の内容確認
