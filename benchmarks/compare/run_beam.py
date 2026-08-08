#!/usr/bin/env python
# run_beam.py — Benchmark: 1M row CSV transform with Apache Beam
# Usage: uv run python benchmarks/compare/run_beam.py <dataset.csv>
# TODO: full implementation pending (v64.5.0 stub)
import sys
import time

dataset = sys.argv[1] if len(sys.argv) > 1 else "benchmarks/compare/data/1m_rows.csv"

start = time.time()
# Stub: simulate Apache Beam CSV → transform workload
# Real implementation will use apache_beam.Pipeline + ReadFromText + transforms + WriteToBigQuery
print(f"[stub] Apache Beam benchmark on {dataset} — not yet implemented")
elapsed = int((time.time() - start) * 1000)
print(f"elapsed: {elapsed}ms")
