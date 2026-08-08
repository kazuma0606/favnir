#!/usr/bin/env python
# run_pandas.py — Benchmark: 1M row CSV transform with Python pandas
# Usage: uv run python benchmarks/compare/run_pandas.py <dataset.csv>
# TODO: full implementation pending (v64.5.0 stub)
import sys
import time

dataset = sys.argv[1] if len(sys.argv) > 1 else "benchmarks/compare/data/1m_rows.csv"

start = time.time()
# Stub: simulate pandas CSV → transform workload
# Real implementation will use pandas.read_csv + DataFrame transforms + psycopg2 insert
print(f"[stub] pandas benchmark on {dataset} — not yet implemented")
elapsed = int((time.time() - start) * 1000)
print(f"elapsed: {elapsed}ms")
