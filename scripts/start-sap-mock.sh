#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR/../infra/e2e-demo/sap-odata"

docker compose up -d

echo "SAP OData mock server started at http://localhost:4004"
echo "Business Partners: http://localhost:4004/BusinessPartnerCollection"
echo "Sales Orders:      http://localhost:4004/SalesOrderCollection"
