# v29.4.0 Plan — vertex-ai / sagemaker Rune 追加

**バージョン**: 29.4.0
**日付**: 2026-06-30
**前バージョン**: v29.3.0 (pinecone Rune 追加)

---

## 実装手順

### T1: Cargo.toml version 更新

```toml
version = "29.4.0"
```

### T2: runes/vertex-ai/rune.toml 作成

```toml
[rune]
name        = "vertex-ai"
version     = "1.0.0"
description = "Google Vertex AI 連携（predict / batch_predict / deploy_model / list_endpoints）"
license     = "MIT"
authors     = ["Favnir Team"]
```

### T3: runes/vertex-ai/vertex-ai.fav 作成（4 関数）

```favnir
// vertex-ai Rune -- Google Vertex AI 連携（v29.4.0）
// 接続: VERTEX_AI_PROJECT / VERTEX_AI_LOCATION / VERTEX_AI_BASE_URL 環境変数

// オンライン推論（エンドポイント URL 指定）
fn VertexAI.predict(endpoint: String, instances: List<String>) -> Result<List<String>, String> !Http =
  Http.post_json(
    Env.get_or("VERTEX_AI_BASE_URL", "https://us-central1-aiplatform.googleapis.com") ++ "/v1/" ++ endpoint ++ ":predict",
    { "instances": instances }
  )

// バッチ推論（GCS I/O）
fn VertexAI.batch_predict(model: String, gcs_input: String, gcs_output: String) -> Result<String, String> !Http =
  Http.post_json(
    Env.get_or("VERTEX_AI_BASE_URL", "https://us-central1-aiplatform.googleapis.com") ++ "/v1/projects/" ++ Env.get_or("VERTEX_AI_PROJECT", "") ++ "/locations/" ++ Env.get_or("VERTEX_AI_LOCATION", "us-central1") ++ "/batchPredictionJobs",
    { "model": model, "inputConfig": { "gcsSource": gcs_input }, "outputConfig": { "gcsDestination": gcs_output } }
  )

// モデルをエンドポイントにデプロイし、エンドポイント ID を返す
fn VertexAI.deploy_model(model_id: String, machine_type: String) -> Result<String, String> !Http =
  Http.post_json(
    Env.get_or("VERTEX_AI_BASE_URL", "https://us-central1-aiplatform.googleapis.com") ++ "/v1/projects/" ++ Env.get_or("VERTEX_AI_PROJECT", "") ++ "/locations/" ++ Env.get_or("VERTEX_AI_LOCATION", "us-central1") ++ "/endpoints",
    { "model_id": model_id, "machine_type": machine_type }
  )

// エンドポイント一覧を取得する
fn VertexAI.list_endpoints() -> Result<List<String>, String> !Http =
  Http.get_json(
    Env.get_or("VERTEX_AI_BASE_URL", "https://us-central1-aiplatform.googleapis.com") ++ "/v1/projects/" ++ Env.get_or("VERTEX_AI_PROJECT", "") ++ "/locations/" ++ Env.get_or("VERTEX_AI_LOCATION", "us-central1") ++ "/endpoints"
  )
```

### T4: runes/sagemaker/rune.toml 作成

```toml
[rune]
name        = "sagemaker"
version     = "1.0.0"
description = "AWS SageMaker 連携（invoke / create_endpoint / delete_endpoint）"
license     = "MIT"
authors     = ["Favnir Team"]
```

### T5: runes/sagemaker/sagemaker.fav 作成（3 関数）

```favnir
// sagemaker Rune -- AWS SageMaker 連携（v29.4.0）
// 接続: SAGEMAKER_REGION / SAGEMAKER_BASE_URL 環境変数

// SageMaker エンドポイントで推論を実行する
fn SageMaker.invoke(endpoint_name: String, payload: String) -> Result<String, String> !Http =
  Http.post_json(
    Env.get_or("SAGEMAKER_BASE_URL", "https://runtime.sagemaker.us-east-1.amazonaws.com") ++ "/endpoints/" ++ endpoint_name ++ "/invocations",
    { "payload": payload }
  )

// SageMaker エンドポイントを作成し、エンドポイント名を返す
fn SageMaker.create_endpoint(model: String, config: String) -> Result<String, String> !Http =
  Http.post_json(
    Env.get_or("SAGEMAKER_BASE_URL", "https://runtime.sagemaker.us-east-1.amazonaws.com") ++ "/endpoints",
    { "model": model, "config": config }
  )

// SageMaker エンドポイントを削除する
fn SageMaker.delete_endpoint(name: String) -> Result<Unit, String> !Http =
  Http.post_json(
    Env.get_or("SAGEMAKER_BASE_URL", "https://runtime.sagemaker.us-east-1.amazonaws.com") ++ "/endpoints/" ++ name ++ "/delete",
    { "name": name }
  )
```

### T6: CHANGELOG.md に [v29.4.0] セクション追加

```markdown
## [v29.4.0] — 2026-06-30

### Added
- `runes/vertex-ai/` — Google Vertex AI Rune（predict / batch_predict / deploy_model / list_endpoints）
- `runes/sagemaker/` — AWS SageMaker Rune（invoke / create_endpoint / delete_endpoint）
- `site/content/docs/runes/vertex-ai.mdx` / `sagemaker.mdx` — ドキュメント追加
- テスト数: 2330 → 2336（+6）
```

### T7: benchmarks/v29.4.0.json 作成

```json
{
  "version": "29.4.0",
  "date": "2026-06-30",
  "milestone": "Ecosystem Maturity (phase 4)",
  "test_count": 2336,
  "metrics": {
    "compile_hello_ms": 12,
    "compile_etl_ms": 38,
    "typecheck_ms": 9,
    "vm_run_ms": 4
  }
}
```

### T8: site/content/docs/runes/vertex-ai.mdx と sagemaker.mdx 作成

各 Rune の使い方・API リファレンス・ML パイプライン例を含むドキュメント。

### T9: driver.rs に v294000_tests 6 件追加

```rust
// v294000_tests (v29.4.0) -- vertex-ai / sagemaker Rune
#[cfg(test)]
mod v294000_tests {
    #[test]
    fn vertex_ai_rune_file_exists() {
        let src = include_str!("../../runes/vertex-ai/vertex-ai.fav");
        assert!(
            src.contains("predict"),
            "runes/vertex-ai/vertex-ai.fav must define predict"
        );
    }
    #[test]
    fn vertex_ai_batch_and_deploy_fn_exists() {
        let src = include_str!("../../runes/vertex-ai/vertex-ai.fav");
        assert!(
            src.contains("batch_predict") && src.contains("deploy_model"),
            "vertex-ai.fav must define batch_predict and deploy_model"
        );
    }
    #[test]
    fn vertex_ai_list_endpoints_fn_exists() {
        let src = include_str!("../../runes/vertex-ai/vertex-ai.fav");
        assert!(
            src.contains("list_endpoints"),
            "vertex-ai.fav must define list_endpoints"
        );
    }
    #[test]
    fn sagemaker_rune_file_exists() {
        let src = include_str!("../../runes/sagemaker/sagemaker.fav");
        assert!(
            src.contains("invoke"),
            "runes/sagemaker/sagemaker.fav must define invoke"
        );
    }
    #[test]
    fn sagemaker_endpoint_fns_exist() {
        let src = include_str!("../../runes/sagemaker/sagemaker.fav");
        assert!(
            src.contains("create_endpoint") && src.contains("delete_endpoint"),
            "sagemaker.fav must define create_endpoint and delete_endpoint"
        );
    }
    #[test]
    fn changelog_has_v29_4_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(
            src.contains("[v29.4.0]") || src.contains("## v29.4.0"),
            "CHANGELOG.md must contain '[v29.4.0]'"
        );
    }
}
```

### T10: cargo test --bin fav v294000 — 6/6 PASS 確認

### T11: cargo test --bin fav — 2336 tests PASS 確認

### T12: tasks.md を COMPLETE に更新

---

## テスト数カウント

| バージョン | テスト数 |
|---|---|
| v29.3.0 | 2330 |
| v29.4.0 | **2336** (+6) |
