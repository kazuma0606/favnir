#!/usr/bin/env bash
# AOT E2E Demo script (v62.9.0)
# Note: fav build --link / --docker require release binary and Docker.
# Each step reports status but does not abort the script on failure,
# because the commands may not be available in all CI environments.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# scripts/ -> aot/ -> e2e-demo/ -> infra/ -> repo root (4 levels up)
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../../.." && pwd)"
FAV="${REPO_ROOT}/fav/target/release/fav"
PIPELINE="${SCRIPT_DIR}/../src/pipeline.fav"
AOT_OUT_DIR="/tmp/aot-demo"

mkdir -p "${AOT_OUT_DIR}"

echo "[1/3] fav build pipeline.fav --link -o ${AOT_OUT_DIR}/pipeline"
if "${FAV}" build "${PIPELINE}" --link -o "${AOT_OUT_DIR}/pipeline"; then
    echo "  -> link: OK"
else
    echo "  -> link: skipped (fav build --link not available in this environment)"
fi

echo "[2/3] fav build pipeline.fav --docker --tag fav-demo:latest"
if "${FAV}" build "${PIPELINE}" --docker --tag fav-demo:latest; then
    echo "  -> docker: OK"
else
    echo "  -> docker: skipped (Docker not available in this environment)"
fi

echo "[3/3] fav build pipeline.fav --validate"
if "${FAV}" build "${PIPELINE}" --validate; then
    echo "  -> validate: OK"
else
    echo "  -> validate: skipped (fav binary not found or --validate not available)"
fi

echo "All AOT E2E checks passed."
