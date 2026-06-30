# v29.2.0 Plan — mlflow Rune 追加

## 実装順序

| タスク | 内容 | 依存 |
|---|---|---|
| T1 | `Cargo.toml` version `29.1.0` → `29.2.0` | — |
| T2 | `runes/mlflow/rune.toml` 作成 | — |
| T3 | `runes/mlflow/mlflow.fav` 作成（8 関数）| T2 |
| T4 | `CHANGELOG.md` に `[v29.2.0]` セクション追加 | — |
| T5 | `benchmarks/v29.2.0.json` 作成 | T1 |
| T6 | `site/content/docs/runes/mlflow.mdx` 作成 | T3 |
| T7 | `driver.rs` に `v292000_tests` 6 件追加 | T3, T4 |
| T8 | `cargo test --bin fav v292000` — 6/6 PASS 確認 | T7 |
| T9 | `cargo test --bin fav` — 2324 tests PASS 確認 | T8 |
| T10 | tasks.md を COMPLETE に更新 | T9 |

## テストコード（T7）

`mlflow_rune_toml_exists` を `mlflow_end_run_and_artifact_fn_exists` に差し替え、
`end_run` / `log_artifact` のカバレッジを追加する（spec-reviewer [LOW] 対応）。

```rust
// ── v292000_tests (v29.2.0) — mlflow Rune 追加 ──────────────────────────────────────────────────────
#[cfg(test)]
mod v292000_tests {
    #[test]
    fn mlflow_rune_file_exists() {
        let src = include_str!("../../runes/mlflow/mlflow.fav");
        assert!(
            src.contains("start_run"),
            "runes/mlflow/mlflow.fav must define start_run"
        );
    }
    #[test]
    fn mlflow_end_run_and_artifact_fn_exists() {
        let src = include_str!("../../runes/mlflow/mlflow.fav");
        assert!(
            src.contains("end_run") && src.contains("log_artifact"),
            "mlflow.fav must define end_run and log_artifact"
        );
    }
    #[test]
    fn mlflow_log_metric_fn_exists() {
        let src = include_str!("../../runes/mlflow/mlflow.fav");
        assert!(src.contains("log_metric"), "mlflow.fav must define log_metric");
    }
    #[test]
    fn mlflow_log_param_fn_exists() {
        let src = include_str!("../../runes/mlflow/mlflow.fav");
        assert!(src.contains("log_param"), "mlflow.fav must define log_param");
    }
    #[test]
    fn mlflow_register_model_fn_exists() {
        let src = include_str!("../../runes/mlflow/mlflow.fav");
        assert!(src.contains("register_model"), "mlflow.fav must define register_model");
    }
    #[test]
    fn changelog_has_v29_2_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(
            src.contains("[v29.2.0]") || src.contains("## v29.2.0"),
            "CHANGELOG.md must contain '[v29.2.0]'"
        );
    }
}
```

## 配置場所

`v292000_tests` は `driver.rs` の `// ── v291000_tests` の直前に挿入する。

## rune.toml テンプレート

`[connection]` セクションは他の Rune と非標準のため含めない（spec-reviewer [MED] 対応）。
接続先情報は `mlflow.fav` 内コメントで `MLFLOW_TRACKING_URI` として記載する。

```toml
[rune]
name        = "mlflow"
version     = "1.0.0"
description = "MLflow 実験管理・モデルレジストリ連携"
license     = "MIT"
authors     = ["Favnir Team"]
```

## mlflow.fav 構造

```
// 接続: MLFLOW_TRACKING_URI 環境変数（デフォルト: http://localhost:5000）
fn MLflow.start_run(experiment_name: String) -> Result<String, String> !Http
fn MLflow.log_metric(run_id: String, key: String, value: Float, step: Int) -> Result<Unit, String> !Http
fn MLflow.log_param(run_id: String, key: String, value: String) -> Result<Unit, String> !Http
fn MLflow.log_artifact(run_id: String, local_path: String) -> Result<Unit, String> !Http
fn MLflow.end_run(run_id: String, status: String) -> Result<Unit, String> !Http
fn MLflow.register_model(run_id: String, name: String) -> Result<String, String> !Http
fn MLflow.load_model(name: String, version: String) -> Result<String, String> !Http
fn MLflow.list_experiments() -> Result<List<String>, String> !Http
```

各関数は `MLFLOW_TRACKING_URI` を `Env.get` で取得し、HTTP API を呼ぶ形式で実装する。
Rune ファイルであるため、実際の HTTP 実行は VM primitive に委譲する。
