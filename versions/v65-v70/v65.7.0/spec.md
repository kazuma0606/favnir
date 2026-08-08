# v65.7.0 Spec — ML Primitives Rune（`Rune.ml`）

Version: 65.7.0
Status: 未着手
Base tests: 3465
Target tests: 3467

---

## 概要

古典的 ML アルゴリズムを型安全なステージとして提供する Rune `Rune.ml` を実装する。
LLM に依存しない「型で守られた ML」の基盤。
既存の `runes/scikit/`（scikit-learn 連携）とは別物——ネイティブ実装スタブ。

```favnir
// 利用例（用途のイメージ）
// ※ ロードマップ例は名前付き引数の擬似コード（bind 構文・Cosine 識別子）だが、
//   現バージョンのスタブ実装は位置引数・String リテラルで定義する。
//   bind を使わずインライン呼び出しが正しい構文。
public stage ClassifyCustomers: CustomerFeatures -> Segment = |features| {
    Rune.ml.predict(Rune.ml.knn(5, "Cosine"), features)
}
```

ロードマップ `roadmap-v65.1-v66.0.md` の v65.7.0 セクションに準拠。

---

## 前提確認（T0 で実施）

- `cargo test -j 8 -- --test-threads=8` でベース 3465 tests passed, 0 failed を確認
- `fav/Cargo.toml` の version が `"65.0.0"` であることを確認（sub-version では Cargo.toml は更新しない）
- `runes/ml/` ディレクトリが存在しないことを確認（新規作成対象）
- `driver.rs` に `v65600_tests` が存在することを確認（`v65700_tests` の挿入位置）
- `driver.rs` に `v65700_tests` が存在しないことを確認（新規追加）
- `cargo test --bin fav v65600_tests` で 2 件 PASS することを確認（前バージョンが正常）
- `versions/current.md` の「進行中バージョン」が `v65.6.0` であることを確認

---

## 実装スコープ

### 1. `runes/ml/rune.toml` — Rune メタデータ

```toml
[rune]
name        = "ml"
version     = "0.1.0"
description = "ML Primitives Rune for Favnir — classification, regression, clustering, dimensionality reduction, evaluation, pipeline"
entry       = "ml.fav"
effects     = []

[dependencies]
```

### 2. `runes/ml/ml.fav` — Rune 実装スタブ

以下の全 25 関数定義を含むスタブファイルを作成する。
今バージョンでは**関数シグネチャの確立**が目的。実際の ML 計算は将来フェーズ。

```favnir
// ML Primitives Rune — Rune.ml
// Classification, Regression, Clustering, Dimensionality Reduction, Evaluation, Pipeline
//
// NOTE: Model, Predictions, FeatureRanking, Matrix<Float>, Dataset, EvalResult,
//       SplitResult, CVResult 等の型は将来フェーズで型システムに登録する。
//       今バージョンは include_str! テストのみ（型チェックエラーは無視する）。

// --- 分類 ---

public fn knn(k: Int, metric: String) -> Model {
    Model { kind: "knn", k: k, metric: metric }
}

public fn naive_bayes(var_smoothing: Float) -> Model {
    Model { kind: "naive_bayes", var_smoothing: var_smoothing }
}

public fn random_forest(n_estimators: Int, max_depth: Int) -> Model {
    Model { kind: "random_forest", n_estimators: n_estimators, max_depth: max_depth }
}

public fn gradient_boosting(n_estimators: Int, learning_rate: Float) -> Model {
    Model { kind: "gradient_boosting", n_estimators: n_estimators, learning_rate: learning_rate }
}

public fn svm(c: Float, kernel: String) -> Model {
    Model { kind: "svm", c: c, kernel: kernel }
}

// --- 回帰 ---

public fn ridge(alpha: Float) -> Model {
    Model { kind: "ridge", alpha: alpha }
}

public fn lasso(alpha: Float) -> Model {
    Model { kind: "lasso", alpha: alpha }
}

public fn elastic_net(alpha: Float, l1_ratio: Float) -> Model {
    Model { kind: "elastic_net", alpha: alpha, l1_ratio: l1_ratio }
}

public fn svr(c: Float, epsilon: Float, kernel: String) -> Model {
    Model { kind: "svr", c: c, epsilon: epsilon, kernel: kernel }
}

// --- クラスタリング ---

public fn kmeans(n_clusters: Int, max_iter: Int) -> Model {
    Model { kind: "kmeans", n_clusters: n_clusters, max_iter: max_iter }
}

public fn dbscan(eps: Float, min_samples: Int) -> Model {
    Model { kind: "dbscan", eps: eps, min_samples: min_samples }
}

public fn hierarchical(n_clusters: Int, linkage: String) -> Model {
    Model { kind: "hierarchical", n_clusters: n_clusters, linkage: linkage }
}

// --- 次元削減 ---

public fn pca(n_components: Int) -> Model {
    Model { kind: "pca", n_components: n_components }
}

public fn umap(n_components: Int, n_neighbors: Int) -> Model {
    Model { kind: "umap", n_components: n_components, n_neighbors: n_neighbors }
}

public fn tsne(n_components: Int, perplexity: Float) -> Model {
    Model { kind: "tsne", n_components: n_components, perplexity: perplexity }
}

// --- 予測・特徴量 ---

public fn predict(model: Model, data: Matrix<Float>) -> Predictions {
    Predictions { labels: [], scores: [] }
}

public fn feature_importance(model: Model, data: Dataset) -> FeatureRanking {
    FeatureRanking { features: [], scores: [] }
}

// --- 評価指標 ---

public fn accuracy(y_true: List<Int>, y_pred: List<Int>) -> Float {
    0.0
}

public fn f1_score(y_true: List<Int>, y_pred: List<Int>) -> Float {
    0.0
}

public fn roc_auc(y_true: List<Int>, y_score: List<Float>) -> Float {
    0.0
}

public fn confusion_matrix(y_true: List<Int>, y_pred: List<Int>) -> Matrix<Int> {
    Matrix { rows: 0, cols: 0, data: [] }
}

public fn precision_recall(y_true: List<Int>, y_pred: List<Int>) -> EvalResult {
    EvalResult { precision: 0.0, recall: 0.0, f1: 0.0 }
}

// --- パイプライン ---

public fn cross_validate(model: Model, data: Dataset, folds: Int) -> CVResult {
    CVResult { scores: [], mean: 0.0, std: 0.0 }
}

public fn grid_search(model: Model, param_grid: List<String>, data: Dataset) -> Model {
    model
}

public fn train_test_split(data: Dataset, test_size: Float) -> SplitResult {
    SplitResult { train: data, test: data }
}
```

### 3. `driver.rs` — `v65700_tests` 追加

挿入位置: `// -- v65600_tests (v65.6.0)` コメントの直前

```rust
// -- v65700_tests (v65.7.0) -- ML Primitives Rune --
#[cfg(test)]
mod v65700_tests {
    #[test]
    fn ml_rune_random_forest_classify() {
        let content = include_str!("../../runes/ml/ml.fav");
        assert!(!content.is_empty(), "ml.fav should not be empty");
        assert!(content.contains("fn knn("), "ml.fav should define knn");
        assert!(content.contains("fn random_forest("), "ml.fav should define random_forest");
        assert!(content.contains("fn predict("), "ml.fav should define predict");
        assert!(content.contains("fn svm("), "ml.fav should define svm");
    }

    #[test]
    fn ml_rune_cross_validate() {
        let content = include_str!("../../runes/ml/ml.fav");
        assert!(content.contains("fn cross_validate("), "ml.fav should define cross_validate");
        assert!(content.contains("fn grid_search("), "ml.fav should define grid_search");
        assert!(content.contains("fn roc_auc("), "ml.fav should define roc_auc");
    }
}
```

挿入後、`cargo build` でエラーなしを確認。

---

## 完了条件

- `runes/ml/ml.fav` が存在し空でない
- `runes/ml/rune.toml` が存在する
- `ml.fav` に全 25 関数が定義されている:
  - `knn`, `naive_bayes`, `random_forest`, `gradient_boosting`, `svm`（分類）
  - `ridge`, `lasso`, `elastic_net`, `svr`（回帰）
  - `kmeans`, `dbscan`, `hierarchical`（クラスタリング）
  - `pca`, `umap`, `tsne`（次元削減）
  - `predict`, `feature_importance`（予測・特徴量）
  - `accuracy`, `f1_score`, `roc_auc`, `confusion_matrix`, `precision_recall`（評価指標）
  - `cross_validate`, `grid_search`, `train_test_split`（パイプライン）
  - ※ テストで検証するのはうち 7 要素（`knn`/`random_forest`/`predict`/`svm`/`cross_validate`/`grid_search`/`roc_auc`）。残りは tasks.md T1 チェックボックスで確認する
- `cargo test --bin fav v65700_tests` で 2 件 PASS
  - `ml_rune_random_forest_classify` PASS
  - `ml_rune_cross_validate` PASS
- `cargo test --bin fav` で 3467 tests passed, 0 failed

---

## 非スコープ

- 実際の ML アルゴリズム実装 — 将来フェーズ
- `Model` / `Predictions` / `FeatureRanking` / `Matrix<Float>` / `Dataset` / `EvalResult` / `CVResult` / `SplitResult` の型システム登録 — 将来フェーズ
- `fav check` での型チェック通過 — 今バージョンは `include_str!` テストのみ（型未定義エラーは無視する）
- CHANGELOG.md 更新 — v66.0.0 宣言時に一括追記
- site/ MDX ドキュメント作成 — v65.9.0 安定化時に一括作成するため今バージョンは省略

---

## 技術ノート

### `include_str!` パス（`fav/src/driver.rs` 起点）

- `"../../runes/ml/ml.fav"` → `favnir/runes/ml/ml.fav`

### `contains` 判定の設計方針

- `contains("fn knn(")` — `public fn knn(` にマッチ
- `contains("fn random_forest(")` — `public fn random_forest(` にマッチ
- `contains("fn predict(")` — `public fn predict(` にマッチ
- `contains("fn svm(")` — `public fn svm(` にマッチ
- `contains("fn cross_validate(")` — `public fn cross_validate(` にマッチ
- `contains("fn grid_search(")` — `public fn grid_search(` にマッチ
- `contains("fn roc_auc(")` — `public fn roc_auc(` にマッチ

### v65.1.0〜v65.6.0 レビューで確立した構文ルール

- `bind x <- expr` は Result/Option を返す式にのみ使用する（スタブでは不要）
- `let` は使わない
- `Math.sqrt` を使う（`Float.sqrt` は VM に存在しない）
- `Float.from_int` は VM に存在しない

### `pca` と `Rune.linalg` の依存関係

ロードマップでは `pca（Rune.linalg 上）` と注記されている。
今バージョンはスタブ実装のため依存は発生しないが、
**将来フェーズで `pca` を `Rune.linalg` の行列演算の上に再実装予定**。

### rune.toml フォーマット

- `entry = "ml.fav"`（`main` ではなく `entry`）
- `effects = []` を明示
- `[dependencies]` セクションを含める
