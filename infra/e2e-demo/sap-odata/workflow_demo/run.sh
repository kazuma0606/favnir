#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# SAP Workflow E2E デモ実行スクリプト（v97.6.0）
# route_by_approval_result pipeline を実行する（bash run.sh で呼び出す想定）
fav run "${SCRIPT_DIR}/../pipeline_workflow.fav"
