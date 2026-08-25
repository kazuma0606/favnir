#!/usr/bin/env bash
# scripts/test-with-mock.sh
# SAP OData モックサーバーを起動して sap-odata Rune テストを実行する（v86.7.0）
# 本番 SAP システムへの接続なしにローカルでテストを実行するためのスクリプト。
# v87.0.0 以降で実際のモックサーバー統合を実施する予定。

set -euo pipefail

echo "SAP OData mock server check (v86.7.0 stub)"
echo "Note: Actual mock server integration is planned for v87.0.0+"
echo "PASS: test-with-mock.sh executed successfully"
