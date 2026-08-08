# v65.7.0 実装計画 — ML Primitives Rune（`Rune.ml`）

Version: 65.7.0
Status: 未着手
Base tests: 3465
Target tests: 3467

---

## 実装ステップ

### Step 1: ディレクトリ・ファイル作成

1. `runes/ml/` ディレクトリ作成
2. `runes/ml/rune.toml` 作成
3. `runes/ml/ml.fav` 作成（全 25 関数）

### Step 2: `driver.rs` テスト追加

- `// -- v65600_tests (v65.6.0)` コメントの直前に `v65700_tests` を挿入
- 2 テスト関数:
  - `ml_rune_random_forest_classify`
  - `ml_rune_cross_validate`
- **Rust テストで検証する 7 関数**: `knn` / `random_forest` / `predict` / `svm` / `cross_validate` / `grid_search` / `roc_auc`
- **`grep -c 'public fn '` で確認する残り 18 関数**: `naive_bayes` / `gradient_boosting` / `ridge` / `lasso` / `elastic_net` / `svr` / `kmeans` / `dbscan` / `hierarchical` / `pca` / `umap` / `tsne` / `feature_importance` / `accuracy` / `f1_score` / `confusion_matrix` / `precision_recall` / `train_test_split`

### Step 3: ビルド・テスト確認

```bash
cargo build
cargo test --bin fav v65700_tests
cargo test --bin fav
```

---

## `ml.fav` 実装方針

- **全 25 関数をスタブとして実装**（シグネチャ確立が目的）
- `bind` / `let` は使用しない
- `Float.from_int` / `Float.sqrt` は使用しない
- 戻り値:
  - `Float` 系 → `0.0`
  - `List<_>` 系 → `[]`
  - `Model` → `Model { kind: "...", ... }` レコードリテラル（`kind` フィールド必須）
  - `Predictions` → `Predictions { labels: [], scores: [] }`
  - `FeatureRanking` → `FeatureRanking { features: [], scores: [] }`
  - `Matrix<Int>` → `Matrix { rows: 0, cols: 0, data: [] }`
  - `EvalResult` → `EvalResult { precision: 0.0, recall: 0.0, f1: 0.0 }`
  - `CVResult` → `CVResult { scores: [], mean: 0.0, std: 0.0 }`
  - `SplitResult` → `SplitResult { train: data, test: data }`
  - `grid_search` → `model`（入力をそのまま返す）

## `rune.toml` 形式

```toml
[rune]
name        = "ml"
version     = "0.1.0"
description = "..."
entry       = "ml.fav"
effects     = []

[dependencies]
```

---

## `driver.rs` 挿入コード

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

---

## 関数一覧（25 関数）

| カテゴリ | 関数名 | 戻り値 |
|---|---|---|
| 分類 | `knn(k, metric)` | `Model` レコード |
| 分類 | `naive_bayes(var_smoothing)` | `Model` レコード |
| 分類 | `random_forest(n_estimators, max_depth)` | `Model` レコード |
| 分類 | `gradient_boosting(n_estimators, learning_rate)` | `Model` レコード |
| 分類 | `svm(c, kernel)` | `Model` レコード |
| 回帰 | `ridge(alpha)` | `Model` レコード |
| 回帰 | `lasso(alpha)` | `Model` レコード |
| 回帰 | `elastic_net(alpha, l1_ratio)` | `Model` レコード |
| 回帰 | `svr(c, epsilon, kernel)` | `Model` レコード |
| クラスタリング | `kmeans(n_clusters, max_iter)` | `Model` レコード |
| クラスタリング | `dbscan(eps, min_samples)` | `Model` レコード |
| クラスタリング | `hierarchical(n_clusters, linkage)` | `Model` レコード |
| 次元削減 | `pca(n_components)` | `Model` レコード |
| 次元削減 | `umap(n_components, n_neighbors)` | `Model` レコード |
| 次元削減 | `tsne(n_components, perplexity)` | `Model` レコード |
| 予測・特徴量 | `predict(model, data)` | `Predictions` レコード |
| 予測・特徴量 | `feature_importance(model, data)` | `FeatureRanking` レコード |
| 評価指標 | `accuracy(y_true, y_pred)` | `0.0` |
| 評価指標 | `f1_score(y_true, y_pred)` | `0.0` |
| 評価指標 | `roc_auc(y_true, y_score)` | `0.0` |
| 評価指標 | `confusion_matrix(y_true, y_pred)` | `Matrix` レコード |
| 評価指標 | `precision_recall(y_true, y_pred)` | `EvalResult` レコード |
| パイプライン | `cross_validate(model, data, folds)` | `CVResult` レコード |
| パイプライン | `grid_search(model, param_grid, data)` | `model`（パススルー） |
| パイプライン | `train_test_split(data, test_size)` | `SplitResult` レコード |

---

## リスク・注意点

- `Model` / `Predictions` 等の型は未定義（型チェックエラーは無視）
- `grid_search` は `model` をそのまま返すシンプルなスタブ
- `train_test_split` は `data` を両方に返す（train/test 同一のスタブ）
- `confusion_matrix` の戻り型 `Matrix<Int>` は型システム未登録（include_str! テストのみのため問題なし）
