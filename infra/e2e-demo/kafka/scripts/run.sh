#!/usr/bin/env bash
# infra/e2e-demo/kafka/scripts/run.sh
# Kafka / MSK E2E デモ実行スクリプト (v15.4.0)
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DEMO_DIR="$(dirname "$SCRIPT_DIR")"
FAV_DIR="$(cd "$DEMO_DIR/../../../fav" && pwd)"

echo "=== Favnir Kafka E2E Demo ==="

# 環境変数チェック
: "${KAFKA_BOOTSTRAP_BROKERS:?KAFKA_BOOTSTRAP_BROKERS must be set}"
: "${KAFKA_SASL_USERNAME:?KAFKA_SASL_USERNAME must be set}"
: "${KAFKA_SASL_PASSWORD:?KAFKA_SASL_PASSWORD must be set}"

echo "Brokers: $KAFKA_BOOTSTRAP_BROKERS"
echo "Running pipeline..."

cd "$FAV_DIR"
cargo run --bin fav -- run --legacy "$DEMO_DIR/src/pipeline.fav"

echo "=== PASS ==="
