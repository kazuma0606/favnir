#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# SAP KPI Monitor E2E デモ実行スクリプト（v98.7.0）
fav run "${SCRIPT_DIR}/pipeline_kpi_monitor.fav"
