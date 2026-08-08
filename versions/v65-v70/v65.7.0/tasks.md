# v65.7.0 タスクリスト

Status: COMPLETE
Version: 65.7.0
Base tests: 3465
Target tests: 3467

---

## T0: 事前確認

- [x] `cargo test --bin fav` でベース 3465 tests passed, 0 failed を確認
- [x] `fav/Cargo.toml` の version が `"65.0.0"` であることを確認（sub-version では変更しない）
- [x] `runes/ml/` ディレクトリが存在しないことを確認（新規作成対象）
- [x] `driver.rs` に `v65600_tests` が存在することを確認（`v65700_tests` の挿入位置）
- [x] `driver.rs` に `v65700_tests` が存在しないことを確認（新規追加）
- [x] `cargo test --bin fav v65600_tests` で 2 件 PASS することを確認（前バージョンが正常）
- [x] `versions/current.md` の「進行中バージョン」が `v65.6.0` であることを確認

---

## T1: Rune ファイル作成

- [x] `runes/ml/` ディレクトリ作成
- [x] `runes/ml/rune.toml` 作成（`entry` / `effects = []` / `[dependencies]` 形式）
- [x] `runes/ml/ml.fav` 作成（以下の全 25 関数を定義）
  - **分類**
  - [x] `knn` — k 近傍法
  - [x] `naive_bayes` — ナイーブベイズ
  - [x] `random_forest` — ランダムフォレスト
  - [x] `gradient_boosting` — 勾配ブースティング
  - [x] `svm` — サポートベクターマシン
  - **回帰**
  - [x] `ridge` — Ridge 回帰
  - [x] `lasso` — Lasso 回帰
  - [x] `elastic_net` — Elastic Net
  - [x] `svr` — サポートベクター回帰
  - **クラスタリング**
  - [x] `kmeans` — k-means クラスタリング
  - [x] `dbscan` — DBSCAN クラスタリング
  - [x] `hierarchical` — 階層的クラスタリング
  - **次元削減**
  - [x] `pca` — 主成分分析
  - [x] `umap` — UMAP
  - [x] `tsne` — t-SNE
  - **予測・特徴量**
  - [x] `predict` — 予測
  - [x] `feature_importance` — 特徴量重要度
  - **評価指標**
  - [x] `accuracy` — 正解率
  - [x] `f1_score` — F1 スコア
  - [x] `roc_auc` — ROC AUC
  - [x] `confusion_matrix` — 混同行列
  - [x] `precision_recall` — 適合率・再現率
  - **パイプライン**
  - [x] `cross_validate` — 交差検証
  - [x] `grid_search` — グリッドサーチ
  - [x] `train_test_split` — 学習・テスト分割
- [x] `ml.fav` 内に `let ` が含まれないことを確認
- [x] `ml.fav` 内に `bind ` が含まれないことを確認（スタブには bind 不要）
- [x] `ml.fav` 内に `Float.from_int` / `Float.sqrt` が含まれないことを確認
- [x] `grep -c 'public fn ' ml.fav` で 25 が出ることを確認

---

## T2: `driver.rs` — `v65700_tests` 追加

- [x] `// -- v65600_tests (v65.6.0)` コメントの直前に `v65700_tests` を挿入
  - [x] `ml_rune_random_forest_classify` — `fn knn(` / `fn random_forest(` / `fn predict(` / `fn svm(` を含む
  - [x] `ml_rune_cross_validate` — `fn cross_validate(` / `fn grid_search(` / `fn roc_auc(` を含む
- [x] `use super::*` は不要（`include_str!` のみ使用）
- [x] `cargo build` でエラーなし

---

## T3: ビルド・テスト

- [x] `cargo test --bin fav v65700_tests` で 2 件 PASS
  - [x] `ml_rune_random_forest_classify` PASS
  - [x] `ml_rune_cross_validate` PASS
- [x] `cargo test --bin fav` で 3467 tests passed, 0 failed を確認

---

## T4: ドキュメント・ステータス更新

- [x] `versions/roadmap/roadmap-v65.1-v66.0.md` の v65.7.0 行を「完了」に更新
- [x] `versions/current.md` の「進行中バージョン」を v65.7.0 に更新
- [x] 本 `tasks.md` を COMPLETE に更新（全チェックボックスを `[x]` に）

> **CHANGELOG 方針**: v65.1〜v65.9 では CHANGELOG.md を更新しない。v66.0.0 宣言時に一括追記する。
> **MDX 方針**: `site/` の MDX ドキュメントは v65.9.0 安定化時に一括作成するため今バージョンは省略。
