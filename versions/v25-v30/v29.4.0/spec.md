# v29.4.0 Spec — vertex-ai / sagemaker Rune 追加

**バージョン**: 29.4.0
**日付**: 2026-06-30
**フェーズ**: Ecosystem Maturity (phase 4)
**前バージョン**: v29.3.0 (pinecone Rune 追加)

---

## 概要

Google Vertex AI と AWS SageMaker を Favnir パイプラインから直接呼び出せるようにする。
学習済みモデルのオンライン推論・バッチ推論・エンドポイント管理を `stage` として表現できる。
MLflow（v29.2）で実験管理し、Pinecone（v29.3）でベクトル検索した結果を
Vertex AI / SageMaker で推論する **ML パイプラインの完成** を実現する。

> **ポジショニング**: AI/ML Rune 四部作（mlflow → pinecone → vertex-ai/sagemaker → ...）の第三弾。
> `fav run score.fav` でクラウド ML 推論パイプラインが動く。

---

## 対象コンポーネント

| コンポーネント | 内容 |
|---|---|
| `runes/vertex-ai/vertex-ai.fav` | VertexAI Rune 実装（4 関数）|
| `runes/vertex-ai/rune.toml` | VertexAI Rune メタデータ |
| `runes/sagemaker/sagemaker.fav` | SageMaker Rune 実装（3 関数）|
| `runes/sagemaker/rune.toml` | SageMaker Rune メタデータ |
| `fav/src/driver.rs` | `v294000_tests` 6 件追加 |
| `fav/Cargo.toml` | version 29.3.0 → 29.4.0 |
| `CHANGELOG.md` | `[v29.4.0]` セクション追加 |
| `benchmarks/v29.4.0.json` | ベンチマーク記録 |
| `site/content/docs/runes/vertex-ai.mdx` | Vertex AI ドキュメント |
| `site/content/docs/runes/sagemaker.mdx` | SageMaker ドキュメント |

---

## VertexAI Rune API

### 実装関数

| 関数 | シグネチャ | 内容 |
|---|---|---|
| `VertexAI.predict` | `(endpoint: String, instances: List<String>) -> Result<List<String>, String> !Http` | オンライン推論 |
| `VertexAI.batch_predict` | `(model: String, gcs_input: String, gcs_output: String) -> Result<String, String> !Http` | バッチ推論（GCS I/O）|
| `VertexAI.deploy_model` | `(model_id: String, machine_type: String) -> Result<String, String> !Http` | モデルデプロイ |
| `VertexAI.list_endpoints` | `() -> Result<List<String>, String> !Http` | エンドポイント一覧取得 |

### 設定

| 環境変数 | 説明 |
|---|---|
| `VERTEX_AI_PROJECT` | GCP プロジェクト ID（必須）|
| `VERTEX_AI_LOCATION` | リージョン（例: `us-central1`）|
| `VERTEX_AI_BASE_URL` | エンドポイント URL（デフォルト: `https://us-central1-aiplatform.googleapis.com`）|

---

## SageMaker Rune API

### 実装関数

| 関数 | シグネチャ | 内容 |
|---|---|---|
| `SageMaker.invoke` | `(endpoint_name: String, payload: String) -> Result<String, String> !Http` | エンドポイント推論 |
| `SageMaker.create_endpoint` | `(model: String, config: String) -> Result<String, String> !Http` | エンドポイント作成 |
| `SageMaker.delete_endpoint` | `(name: String) -> Result<Unit, String> !Http` | エンドポイント削除 |

### 設定

| 環境変数 | 説明 |
|---|---|
| `SAGEMAKER_REGION` | AWS リージョン（例: `us-east-1`）|
| `SAGEMAKER_BASE_URL` | エンドポイント URL（デフォルト: `https://runtime.sagemaker.us-east-1.amazonaws.com`）|

---

## 使用例

```favnir
import runes/vertex-ai
import runes/sagemaker

// Vertex AI エンドポイントで推論
stage ScoreWithVertexModel: List<String> -> List<String> !Http = |features| {
  VertexAI.predict(config.vertex_endpoint, features)
}

// SageMaker エンドポイントで推論
stage ScoreWithSageMaker: String -> String !Http = |payload| {
  SageMaker.invoke(config.sagemaker_endpoint, payload)
}
```

---

## テスト戦略

### v294000_tests（6 件）

| テスト名 | 検証内容 |
|---|---|
| `vertex_ai_rune_file_exists` | `runes/vertex-ai/vertex-ai.fav` が存在し `predict` を含む |
| `vertex_ai_batch_and_deploy_fn_exists` | `vertex-ai.fav` に `batch_predict` と `deploy_model` が存在する |
| `vertex_ai_list_endpoints_fn_exists` | `vertex-ai.fav` に `list_endpoints` が存在する |
| `sagemaker_rune_file_exists` | `runes/sagemaker/sagemaker.fav` が存在し `invoke` を含む |
| `sagemaker_endpoint_fns_exist` | `sagemaker.fav` に `create_endpoint` と `delete_endpoint` が存在する |
| `changelog_has_v29_4_0` | `CHANGELOG.md` に `[v29.4.0]` が存在する |

検証関数カバレッジ:
- VertexAI: `predict`, `batch_predict`, `deploy_model`, `list_endpoints`（4/4 関数 = 100%）
- SageMaker: `invoke`, `create_endpoint`, `delete_endpoint`（3/3 関数 = 100%）

テスト数: 2330 → **2336**（+6）

---

## 完了条件

- [ ] `runes/vertex-ai/vertex-ai.fav` に 4 関数が実装されている
- [ ] `runes/vertex-ai/rune.toml` が存在する（`[rune]` セクションのみ）
- [ ] `runes/sagemaker/sagemaker.fav` に 3 関数が実装されている
- [ ] `runes/sagemaker/rune.toml` が存在する（`[rune]` セクションのみ）
- [ ] `cargo test --bin fav v294000` — 6/6 PASS
- [ ] `cargo test --bin fav` — 2336 tests PASS
- [ ] `CHANGELOG.md` に `[v29.4.0]` セクションあり
- [ ] `benchmarks/v29.4.0.json` 存在（test_count: 2336）
- [ ] `site/content/docs/runes/vertex-ai.mdx` 存在
- [ ] `site/content/docs/runes/sagemaker.mdx` 存在

---

## スコープ外

- Vertex AI / SageMaker API への実際の HTTP 接続 — インフラ稼働後に有効化
- Google ADC（Application Default Credentials）認証 — 将来の認証統合フェーズで対応
- SageMaker Batch Transform ジョブ — v29.4.x+ で対応
- VertexAI の `T` 型パラメータ（`predict[T]`）— Favnir の型システム拡張が必要
