# v29.2.0 Spec — mlflow Rune 追加

**バージョン**: 29.2.0
**日付**: 2026-06-30
**フェーズ**: Ecosystem Maturity (phase 2)
**前バージョン**: v29.1.0 (fav publish 実装)

---

## 概要

ML 実験管理・モデルレジストリとして業界標準の MLflow を Favnir から使えるようにする。
データ前処理パイプラインの結果を実験として記録し、学習済みモデルをレジストリに登録するまでを
Favnir pipeline の `stage` として表現できる。

> **ポジショニング**: データパイプライン（Favnir）と ML パイプライン（MLflow）の橋渡し。
> `fav run etl.fav` の出力が MLflow Experiment に記録され、モデルレジストリに繋がる。

---

## 対象コンポーネント

| コンポーネント | 内容 |
|---|---|
| `runes/mlflow/mlflow.fav` | mlflow Rune 実装（8 関数）|
| `runes/mlflow/rune.toml` | Rune メタデータ（`[rune]` セクションのみ）|
| `fav/src/driver.rs` | `v292000_tests` 6 件追加 |
| `fav/Cargo.toml` | version 29.1.0 → 29.2.0 |
| `CHANGELOG.md` | `[v29.2.0]` セクション追加 |
| `benchmarks/v29.2.0.json` | ベンチマーク記録 |
| `site/content/docs/runes/mlflow.mdx` | MLflow ドキュメント |

---

## MLflow Rune API

### 実装関数

| 関数 | シグネチャ | 内容 |
|---|---|---|
| `MLflow.start_run` | `(experiment_name: String) -> Result<String, String> !Http` | 実験実行開始（run_id を返す）|
| `MLflow.log_metric` | `(run_id: String, key: String, value: Float, step: Int) -> Result<Unit, String> !Http` | メトリクス記録 |
| `MLflow.log_param` | `(run_id: String, key: String, value: String) -> Result<Unit, String> !Http` | ハイパーパラメータ記録 |
| `MLflow.log_artifact` | `(run_id: String, local_path: String) -> Result<Unit, String> !Http` | 成果物アップロード |
| `MLflow.end_run` | `(run_id: String, status: String) -> Result<Unit, String> !Http` | 実行終了（FINISHED / FAILED）|
| `MLflow.register_model` | `(run_id: String, name: String) -> Result<String, String> !Http` | モデルレジストリに登録 |
| `MLflow.load_model` | `(name: String, version: String) -> Result<String, String> !Http` | 登録済みモデル URI を取得 |
| `MLflow.list_experiments` | `() -> Result<List<String>, String> !Http` | 実験名一覧取得 |

### 設定

mlflow Rune は `MLFLOW_TRACKING_URI` 環境変数で接続先を指定する（デフォルト: `http://localhost:5000`）。

```favnir
import runes/mlflow

seq FeatureEngineeringPipeline =
  LoadRawData
  |> ExtractFeatures
  |> LogExperiment

stage LogExperiment: FeatureSet -> FeatureSet !Http = |features| {
  bind run_id <- MLflow.start_run("feature-engineering-v1")
  bind _      <- MLflow.log_param(run_id, "window_size", "7d")
  bind _      <- MLflow.log_metric(run_id, "null_rate", features.null_rate, 0)
  bind _      <- MLflow.log_artifact(run_id, "features/output.parquet")
  bind _      <- MLflow.end_run(run_id, "FINISHED")
  Result.ok(features)
}
```

---

## テスト戦略

### v292000_tests（6 件）

| テスト名 | 検証内容 |
|---|---|
| `mlflow_rune_file_exists` | `runes/mlflow/mlflow.fav` が存在し `start_run` を含む |
| `mlflow_end_run_and_artifact_fn_exists` | `mlflow.fav` に `end_run` と `log_artifact` が存在する |
| `mlflow_log_metric_fn_exists` | `mlflow.fav` に `log_metric` が存在する |
| `mlflow_log_param_fn_exists` | `mlflow.fav` に `log_param` が存在する |
| `mlflow_register_model_fn_exists` | `mlflow.fav` に `register_model` が存在する |
| `changelog_has_v29_2_0` | `CHANGELOG.md` に `[v29.2.0]` が存在する |

検証関数カバレッジ: `start_run`, `end_run`, `log_artifact`, `log_metric`, `log_param`, `register_model`（6/8 関数）

テスト数: 2318 → **2324**（+6）

---

## 完了条件

- [ ] `runes/mlflow/mlflow.fav` に 8 関数が実装されている
- [ ] `runes/mlflow/rune.toml` が存在する（`[rune]` セクションのみ）
- [ ] `cargo test --bin fav v292000` — 6/6 PASS
- [ ] `cargo test --bin fav` — 2324 tests PASS
- [ ] `CHANGELOG.md` に `[v29.2.0]` セクションあり
- [ ] `benchmarks/v29.2.0.json` 存在（test_count: 2324）
- [ ] `site/content/docs/runes/mlflow.mdx` 存在

---

## スコープ外

- MLflow サーバー起動（Docker）の自動化 — ローカル環境依存、手順を README に記載するのみ
- `cargo test mlflow` の統合テスト（実際の MLflow API 接続）— v29.2.x+ で対応
- MLflow の Python SDK 相当の全機能 — 8 関数のコア API に絞る
