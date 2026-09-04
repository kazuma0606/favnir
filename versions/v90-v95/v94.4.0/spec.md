# Spec: v94.4.0 — コールドスタートベンチマーク

## Background

v94.3.0 で Lambda SnapStart 対応 Terraform を追加した。
v94.4.0 では SnapStart あり / なし の実際のコールドスタート時間を比較計測するベンチマークスクリプトを追加する。
ベンチマーク結果は JSON ファイルに記録し、`fav bench --sap`（v94.5.0）から参照できるようにする。

## Goals

1. `scripts/bench_sap_coldstart.sh` — SnapStart あり / なし のコールドスタートを計測するシェルスクリプト
2. スクリプト内でベンチマーク結果を `fav/tmp/sap_coldstart_bench.json` に記録するロジックを含む

## Syntax/API Examples

```bash
$ ./scripts/bench_sap_coldstart.sh

SAP Sync Lambda Cold Start Benchmark
=====================================
Without SnapStart:
  P50: 3,421 ms
  P95: 4,892 ms
  P99: 6,204 ms

With SnapStart:
  P50:   248 ms  (-92.7%)
  P95:   312 ms  (-93.6%)
  P99:   387 ms  (-93.8%)

Recommendation: SnapStart reduces cold start by ~93%.
```

出力 JSON（`fav/tmp/sap_coldstart_bench.json`）:
```json
{
  "benchmark": "sap_coldstart_bench",
  "timestamp": "2026-08-30T00:00:00Z",
  "without_snap_start": { "p50_ms": 3421, "p95_ms": 4892, "p99_ms": 6204 },
  "with_snap_start":    { "p50_ms": 248,  "p95_ms": 312,  "p99_ms": 387  },
  "reduction_pct": 92.7
}
```

## Success Criteria

- `scripts/bench_sap_coldstart.sh` が存在する
- スクリプトに `sap_coldstart_bench` が含まれる（出力 JSON パスまたはキー名として）
- `driver.rs` の `mod v94400_tests` が pass する
  - `bench_sap_coldstart_script_exists`: `scripts/bench_sap_coldstart.sh` が存在することを確認
  - `bench_sap_coldstart_output_path_defined`: スクリプトに `sap_coldstart_bench` が含まれることを確認
- `cargo test 2>&1 | grep "test result"` が 4,150 tests, 0 failures を示す（着手前: 4,148）
- `cargo clippy --locked -- -D warnings` が pass する

## Error Codes

なし（シェルスクリプト追加のみ）

## Files to Modify / Create

| ファイル | 操作 | 内容 |
|---|---|---|
| `scripts/bench_sap_coldstart.sh` | **新規作成** | SnapStart あり/なし コールドスタートベンチマークスクリプト |
| `fav/src/driver.rs` | **追加** | `mod v94400_tests`（2 件） |
| `CHANGELOG.md` | **追記** | v94.4.0 エントリ |
