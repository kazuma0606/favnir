#!/usr/bin/env bash
# run_comparison.sh — Reproducible benchmark: Favnir AOT vs pandas / Apache Beam / dbt
# Usage: bash benchmarks/compare/run_comparison.sh
#
# Runs a 1M-row CSV → Postgres transform benchmark across all tools
# and prints a comparison summary table.

set -euo pipefail

DATASET="${1:-benchmarks/compare/data/1m_rows.csv}"
RUNS="${2:-3}"

echo "Benchmark: 1M row CSV → Postgres transform (runs=$RUNS)"
echo "Dataset: $DATASET"
echo ""

run_tool() {
    local name="$1"
    local cmd="$2"
    local total=0
    for i in $(seq 1 "$RUNS"); do
        local t
        t=$(TIMEFORMAT='%R'; { time eval "$cmd" > /dev/null 2>&1; } 2>&1)
        total=$(echo "$total + $t" | bc)
    done
    local avg
    avg=$(echo "scale=0; $total / $RUNS * 1000 / 1" | bc)
    echo "  $name: ${avg} ms"
}

echo "Results:"
run_tool "Favnir AOT " "fav build pipeline.fav --link -o /tmp/fav_bench && /tmp/fav_bench"
run_tool "pandas     " "uv run python benchmarks/compare/run_pandas.py '$DATASET'"
run_tool "Apache Beam" "uv run python benchmarks/compare/run_beam.py '$DATASET'"
run_tool "dbt (SQL)  " "dbt run --select benchmark_transform"
