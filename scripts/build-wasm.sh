#!/usr/bin/env bash
# Build the favnir-wasm crate and output to site/public/wasm/
# Usage: ./scripts/build-wasm.sh [--release]
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WASM_CRATE="$REPO_ROOT/crates/favnir-wasm"
OUT_DIR="$REPO_ROOT/site/public/wasm"

PROFILE="${1#--}"  # strip leading -- if present (e.g. --release → release)

echo "==> Building favnir-wasm..."

if ! command -v wasm-pack &> /dev/null; then
  echo "Error: wasm-pack not found."
  echo "Install with: cargo install wasm-pack"
  exit 1
fi

# wasm-pack on Windows doesn't accept Unix-style absolute paths.
# Convert OUT_DIR to a Windows path if cygpath is available, otherwise use relative.
if command -v cygpath &> /dev/null; then
  WIN_OUT_DIR="$(cygpath -w "$OUT_DIR")"
else
  WIN_OUT_DIR="$OUT_DIR"
fi

(cd "$WASM_CRATE" && wasm-pack build \
  --target web \
  --out-dir "$WIN_OUT_DIR" \
  ${PROFILE:+"--$PROFILE"})

echo "==> WASM build complete: $OUT_DIR"
