# Benchmark Results

Reference performance measurements for Favnir v20.0.0 on a representative machine.

**Environment**: AMD Ryzen 9 5900X, 32GB RAM, NVMe SSD, Ubuntu 22.04

---

## Cold-Start Latency

Measured with `benchmarks/lambda_coldstart.sh` (20 runs each, small pipeline ~50 stages).

| Mode | Avg Latency | vs Full |
|---|---|---|
| `fav run` (full: parse + typecheck + compile + run) | ~320ms | baseline |
| `fav run --precompiled` (skip parse/typecheck/compile) | ~18ms | **−94%** |
| Native binary (`fav build --target native`) | ~5ms | **−98%** |

**Takeaway**: Precompiled `.favc` artifacts reduce Lambda cold starts by ~94% compared
to full `fav run`. Native binaries further improve this to ~98%.

---

## Streaming Throughput

Measured with `benchmarks/10gb_csv.fav` on a synthetic 10GB CSV (200M rows × 4 columns).

| Mode | Throughput | Peak Memory |
|---|---|---|
| Eager (no streaming) | N/A (OOM at ~8GB) | >8GB |
| `#[streaming(chunk_size=100)]` | ~280 MB/s | ~15MB |
| `#[streaming(chunk_size=1000)]` | ~340 MB/s | ~50MB |
| `#[streaming(chunk_size=10000)]` | ~360 MB/s | ~450MB |

**Takeaway**: `chunk_size=1000` provides a good balance of throughput and memory usage
for typical row-oriented pipelines (~50 bytes/row).

---

## Incremental Compilation

Measured on a project with 50 `.fav` source files (total ~5000 lines).

| Scenario | Compile Time |
|---|---|
| Cold build (no cache) | 2.4s |
| No changes (full cache hit) | 0.08s |
| 1 file changed (partial cache) | 0.18s |

**Takeaway**: After the first build, incremental compilation is 13–30x faster for
typical edit-run cycles.

---

## Parallel Compilation

Measured on a project with 50 independent `.fav` files compiled with `fav build --parallel`.

| Threads | Compile Time | Speedup |
|---|---|---|
| 1 (sequential) | 2.4s | 1.0x |
| 4 | 0.74s | 3.2x |
| 8 | 0.42s | 5.7x |
| 12 | 0.31s | 7.7x |

---

## Notes

- These are reference measurements on a specific machine; your results will vary.
- Run `bash benchmarks/lambda_coldstart.sh <your_pipeline.fav>` on your own hardware.
- Streaming throughput depends heavily on row size and transformation complexity.
