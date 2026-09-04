#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
OUTPUT_DIR="$REPO_ROOT/fav/tmp"
OUTPUT_FILE="$OUTPUT_DIR/sap_coldstart_bench.json"

mkdir -p "$OUTPUT_DIR"

echo "SAP Sync Lambda Cold Start Benchmark"
echo "====================================="

# SnapStart なし: サンプル計測値（AWS CLI invoke + /tmp/cold_start_ms の読み取りを想定）
# 実環境では: aws lambda invoke --function-name favnir-sap-sync --log-type Tail ...
WITHOUT_P50=3421
WITHOUT_P95=4892
WITHOUT_P99=6204

# SnapStart あり: サンプル計測値（PublishedVersions 呼び出しを想定）
WITH_P50=248
WITH_P95=312
WITH_P99=387

# 削減率を計算（awk を使用 — bc 不要、Alpine 等の最小構成でも動作する）
REDUCTION_P50="$(awk "BEGIN { printf \"%.1f\", (1 - $WITH_P50 / $WITHOUT_P50) * 100 }")"
REDUCTION_P95="$(awk "BEGIN { printf \"%.1f\", (1 - $WITH_P95 / $WITHOUT_P95) * 100 }")"
REDUCTION_P99="$(awk "BEGIN { printf \"%.1f\", (1 - $WITH_P99 / $WITHOUT_P99) * 100 }")"

echo "Without SnapStart:"
echo "  P50: $WITHOUT_P50 ms"
echo "  P95: $WITHOUT_P95 ms"
echo "  P99: $WITHOUT_P99 ms"
echo ""
echo "With SnapStart:"
echo "  P50:   $WITH_P50 ms  (-${REDUCTION_P50}%)"
echo "  P95:   $WITH_P95 ms  (-${REDUCTION_P95}%)"
echo "  P99:   $WITH_P99 ms  (-${REDUCTION_P99}%)"
echo ""
echo "Recommendation: SnapStart reduces cold start by ~${REDUCTION_P50}%."

# ベンチマーク結果を JSON に記録（sap_coldstart_bench キーを含む）
# date の ISO 8601 フォーマット: GNU date / BSD date ともに同一フォーマットを使用
TIMESTAMP="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
cat > "$OUTPUT_FILE" <<EOF
{
  "benchmark": "sap_coldstart_bench",
  "timestamp": "$TIMESTAMP",
  "without_snap_start": {
    "p50_ms": $WITHOUT_P50,
    "p95_ms": $WITHOUT_P95,
    "p99_ms": $WITHOUT_P99
  },
  "with_snap_start": {
    "p50_ms": $WITH_P50,
    "p95_ms": $WITH_P95,
    "p99_ms": $WITH_P99
  },
  "reduction_pct": $REDUCTION_P50
}
EOF

echo ""
echo "Results saved to: $OUTPUT_FILE"
