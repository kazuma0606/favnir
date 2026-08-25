# Spec: v89.5.0 — E2E デモ完成（4 シナリオ全実行 + Lambda デプロイ）

## Background

v89.3.0 で `infra/e2e-demo/sap-odata/pipeline.fav` に全 4 業務シナリオが揃った。
v88.8.0 で Lambda 基盤（Terraform + `infra/e2e-demo/sap-odata/scripts/run.sh`）が整備済み。

本バージョンでは開発者がローカルから一発で E2E デモを実行できる
`scripts/run-sap-demo.sh` を追加し、4 シナリオ統合実行フローを完成させる。

### 現行状態確認

| 項目 | 状態 |
|---|---|
| `pipeline.fav`（4 シナリオ） | **完成済み**（v89.3.0） |
| Terraform（Lambda / IAM / S3） | **完成済み**（v88.8.0） |
| `infra/e2e-demo/sap-odata/scripts/run.sh`（Lambda 呼び出し） | **完成済み**（v88.8.0） |
| `scripts/run-sap-demo.sh`（一括スクリプト） | **本バージョンで追加** |

## Goals

1. `scripts/run-sap-demo.sh` を作成する
   - モックサーバー起動（`docker compose up -d`）
   - `infra/e2e-demo/sap-odata/scripts/run.sh` を呼び出してパイプライン実行
   - S3 出力確認（`aws s3 ls s3://favnir-sap-demo/`）
   - `LocalStack` / 本番切り替えは `AWS_ENDPOINT_URL` 環境変数で制御
2. `fav/src/driver.rs` に `mod v89500_tests` を追加する（2 件）

## API / Usage Examples

```bash
# ローカル（LocalStack）での E2E デモ実行
$ export AWS_ENDPOINT_URL=http://localhost:4566
$ scripts/run-sap-demo.sh

=== Favnir SAP E2E Demo (All 4 Scenarios) ===
[1/3] Starting SAP mock server...
[2/3] Running pipeline via Lambda...
[3/3] Checking S3 output...
Done.

# 本番 AWS で実行（AWS_ENDPOINT_URL 未設定）
$ scripts/run-sap-demo.sh
```

## スクリプト仕様

```bash
#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "=== Favnir SAP E2E Demo (All 4 Scenarios) ==="

# [1/3] モックサーバー起動（docker compose がある場合のみ）
echo "[1/3] Starting SAP mock server..."
if command -v docker &>/dev/null && [[ -f "$REPO_ROOT/infra/e2e-demo/sap-odata/docker-compose.yml" ]]; then
    docker compose -f "$REPO_ROOT/infra/e2e-demo/sap-odata/docker-compose.yml" up -d
else
    echo "  (docker-compose not available, skipping mock server)"
fi

# [2/3] パイプライン実行（Lambda 呼び出し）
echo "[2/3] Running pipeline via Lambda..."
"$REPO_ROOT/infra/e2e-demo/sap-odata/scripts/run.sh" || true

# [3/3] S3 出力確認
echo "[3/3] Checking S3 output..."
ENDPOINT_ARGS=()
if [[ -n "${AWS_ENDPOINT_URL:-}" ]]; then
    ENDPOINT_ARGS=(--endpoint-url "$AWS_ENDPOINT_URL")
fi
aws s3 ls s3://favnir-sap-demo/ "${ENDPOINT_ARGS[@]}" --recursive 2>/dev/null || true

echo "Done."
```

## Success Criteria（Rust テストで担保）

- `sap_e2e_demo_pipeline_has_journal_entry_scenario`:
  `infra/e2e-demo/sap-odata/pipeline.fav` に `"outstanding_payables"` を含む
  （v89.3.0 完成済みの確認）
- `sap_e2e_run_script_exists`:
  `scripts/run-sap-demo.sh` が存在する
- `cargo test` で 4,029 tests, 0 failures（4,027 + 2）

## Files to Modify / Create

| ファイル | 変更種別 |
|---|---|
| `scripts/run-sap-demo.sh` | 新規作成 |
| `fav/src/driver.rs` | `mod v89500_tests` 追加 |

**前提確認**:
- `pipeline.fav` は v89.3.0 で `outstanding_payables`（シナリオ 4）まで完成済み — 変更不要
- `infra/e2e-demo/sap-odata/scripts/run.sh` は v88.8.0 で作成済み — 変更不要
- `infra/e2e-demo/sap-odata/terraform/` は v88.8.0 で作成済み — **本バージョンでの Lambda デプロイ作業は不要**
- `scripts/start-sap-mock.sh` は既存（SAP モックサーバー起動スクリプト）

**スクリプト設計方針（`set -euo pipefail` との一貫性）**:
- `[1/3]` docker compose: `docker-compose.yml` 存在確認付きのため安全
- `[2/3]` `run.sh`: `|| true` で Lambda 未デプロイ環境（LocalStack 未起動等）でも停止しないよう設計
- `[3/3]` `aws s3 ls`: `2>/dev/null || true` で S3 バケット未作成でも停止しないよう設計

**Note**: CHANGELOG / MILESTONE / site MDX 更新は v90.0.0 宣言バージョンでまとめて実施する（本バージョンは対象外）
**Note**: Cargo.toml のバージョンは v90.0.0 宣言まで `89.0.0` のまま維持する。
