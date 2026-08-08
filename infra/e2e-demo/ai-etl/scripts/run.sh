#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DEMO_DIR="$(dirname "$SCRIPT_DIR")"

echo "=== Favnir AI ETL E2E Demo ==="
echo ""

# Index pipeline: CSV -> Embed -> VectorDB
echo "[1/2] Running IndexPipeline (LoadArticles -> EmbedAndSummarize -> StoreToVectorDB)..."
fav run "$DEMO_DIR/src/pipeline.fav" \
  --cluster "$DEMO_DIR/workers.yaml" \
  --checkpoint "$DEMO_DIR/checkpoints/" \
  --distributed-cache redis://localhost:6379 \
  --otel-endpoint http://localhost:4317

echo ""

# Semantic search (standalone stage)
echo "[2/2] Running SemanticSearch stage..."
fav run "$DEMO_DIR/src/pipeline.fav" \
  --stage SemanticSearch \
  --input "machine learning pipelines"

echo ""
echo "=== Demo complete ==="
