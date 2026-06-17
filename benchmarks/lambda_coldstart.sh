#!/usr/bin/env bash
# benchmarks/lambda_coldstart.sh
#
# Measure the cold-start overhead of Favnir pipeline execution.
# Compares three modes:
#   1. fav run (full: parse + typecheck + compile + execute)
#   2. fav run --precompiled (skips parse/typecheck/compile)
#   3. native binary (skips VM interpreter)
#
# Usage:
#   bash benchmarks/lambda_coldstart.sh <pipeline.fav>
#
# Requirements: fav on PATH, hyperfine (optional, for statistical output)

set -euo pipefail

FAV_SRC="${1:-benchmarks/10gb_csv.fav}"
FAV_FAVC="${FAV_SRC%.fav}.favc"
RUNS="${RUNS:-20}"

echo "=== Favnir Cold-Start Benchmark ==="
echo "Source:  $FAV_SRC"
echo "Runs:    $RUNS"
echo ""

# ── Mode 1: Full pipeline (parse + typecheck + compile + run) ──────────────
echo "--- Mode 1: fav run (full pipeline) ---"
time_full=0
for i in $(seq 1 "$RUNS"); do
    t_start=$(date +%s%3N)
    fav run "$FAV_SRC" /dev/null 2>/dev/null || true
    t_end=$(date +%s%3N)
    time_full=$((time_full + t_end - t_start))
done
avg_full=$((time_full / RUNS))
echo "Average: ${avg_full}ms over ${RUNS} runs"
echo ""

# ── Mode 2: Precompiled (.favc) ────────────────────────────────────────────
echo "--- Mode 2: fav compile + fav run --precompiled ---"
fav compile "$FAV_SRC" -o "$FAV_FAVC"
echo "Compiled: $FAV_FAVC"

time_pre=0
for i in $(seq 1 "$RUNS"); do
    t_start=$(date +%s%3N)
    fav run --precompiled "$FAV_FAVC" 2>/dev/null || true
    t_end=$(date +%s%3N)
    time_pre=$((time_pre + t_end - t_start))
done
avg_pre=$((time_pre / RUNS))
echo "Average: ${avg_pre}ms over ${RUNS} runs"
echo ""

# ── Mode 3: Native binary ─────────────────────────────────────────────────
echo "--- Mode 3: fav build --target native ---"
FAV_NATIVE="${FAV_SRC%.fav}"
fav build --target native "$FAV_SRC" -o "$FAV_NATIVE" 2>/dev/null || {
    echo "(native build not available on this platform, skipping)"
    FAV_NATIVE=""
}

if [ -n "$FAV_NATIVE" ] && [ -x "$FAV_NATIVE" ]; then
    time_native=0
    for i in $(seq 1 "$RUNS"); do
        t_start=$(date +%s%3N)
        "$FAV_NATIVE" /dev/null 2>/dev/null || true
        t_end=$(date +%s%3N)
        time_native=$((time_native + t_end - t_start))
    done
    avg_native=$((time_native / RUNS))
    echo "Average: ${avg_native}ms over ${RUNS} runs"
fi

echo ""
echo "=== Summary ==="
echo "Full pipeline:  ${avg_full}ms"
echo "Precompiled:    ${avg_pre}ms  ($(( (avg_full - avg_pre) * 100 / (avg_full + 1) ))% faster)"
[ -n "${avg_native:-}" ] && echo "Native binary:  ${avg_native}ms  ($(( (avg_full - avg_native) * 100 / (avg_full + 1) ))% faster)"

# Cleanup
rm -f "$FAV_FAVC"
